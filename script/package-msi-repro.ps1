param(
    [string]$SourceDir = "",
    [string]$OutputMsi = "",
    [string]$ProductName = "sonar",
    [string]$Manufacturer = "Sonar Team",
    [string]$Version = "",
    [string]$UpgradeCode = "",
    [string]$PackageCode = "",
    [string]$InternalMode = "",
    [string]$SourceDateEpoch = "1700000000",
    [switch]$Help
)

$ErrorActionPreference = "Stop"

if ($Help) {
    @"
usage: pwsh -File script/package-msi-repro.ps1 -SourceDir <install-root> [-OutputMsi <out.msi>]

Build a normalized Windows MSI from an installation root directory.
The script must run on Windows with WiX Toolset v3 available as candle.exe and light.exe.
"@ | Write-Output
    exit 0
}

$script:FREESECT = [Convert]::ToUInt32("FFFFFFFF", 16)
$script:ENDOFCHAIN = [Convert]::ToUInt32("FFFFFFFE", 16)
$script:FATSECT = [Convert]::ToUInt32("FFFFFFFD", 16)
$script:DIFSECT = [Convert]::ToUInt32("FFFFFFFC", 16)

function Require-Command {
    param([string]$Name)

    $command = Get-Command $Name -ErrorAction SilentlyContinue
    if (-not $command) {
        throw "Missing command: $Name"
    }

    return $command.Source
}

function Convert-ToXmlValue {
    param([string]$Value)
    return [System.Security.SecurityElement]::Escape($Value)
}

function New-DeterministicGuid {
    param([string]$Seed)

    $md5 = [System.Security.Cryptography.MD5]::Create()
    $bytes = $md5.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($Seed))

    $bytes[6] = ($bytes[6] -band 0x0f) -bor 0x30
    $bytes[8] = ($bytes[8] -band 0x3f) -bor 0x80

    return ([System.Guid]::new($bytes)).ToString().ToUpperInvariant()
}

function New-WixId {
    param(
        [string]$Prefix,
        [string]$Value
    )

    $sha1 = [System.Security.Cryptography.SHA1]::Create()
    $bytes = $sha1.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($Value))
    $hex = -join ($bytes | ForEach-Object { $_.ToString("x2") })
    return "${Prefix}_$($hex.Substring(0, 20))"
}

function Resolve-SourceDir {
    if ($SourceDir) {
        return (Resolve-Path $SourceDir).Path
    }

    $candidates = @(
        "src-tauri\target\release\repro-msi-root",
        "src-tauri\target\release\bundle\msi\root"
    )

    foreach ($candidate in $candidates) {
        if (Test-Path $candidate -PathType Container) {
            return (Resolve-Path $candidate).Path
        }
    }

    throw "Source directory not found. Pass SourceDir explicitly."
}

function Normalize-Tree {
    param(
        [string]$Root,
        [DateTime]$Timestamp
    )

    Get-ChildItem -LiteralPath $Root -Recurse -Force |
        Sort-Object FullName |
        ForEach-Object {
            $_.LastWriteTimeUtc = $Timestamp
            $_.CreationTimeUtc = $Timestamp
            $_.LastAccessTimeUtc = $Timestamp
        }

    $rootItem = Get-Item -LiteralPath $Root -Force
    $rootItem.LastWriteTimeUtc = $Timestamp
    $rootItem.CreationTimeUtc = $Timestamp
    $rootItem.LastAccessTimeUtc = $Timestamp
}

function Write-InputManifest {
    param(
        [string]$Root,
        [string]$ManifestPath
    )

    $lines = Get-ChildItem -LiteralPath $Root -Recurse -File -Force |
        Sort-Object FullName |
        ForEach-Object {
            $relative = [System.IO.Path]::GetRelativePath($Root, $_.FullName).Replace("\", "/")
            $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $_.FullName).Hash.ToLowerInvariant()
            "$hash  $relative"
        }

    [System.IO.File]::WriteAllLines($ManifestPath, $lines, [System.Text.UTF8Encoding]::new($false))
}

function Invoke-ComMethod {
    param(
        [object]$Object,
        [string]$Name,
        [object[]]$Arguments = @()
    )

    return $Object.GetType().InvokeMember(
        $Name,
        [System.Reflection.BindingFlags]::InvokeMethod,
        $null,
        $Object,
        $Arguments
    )
}

function Set-ComIndexedProperty {
    param(
        [object]$Object,
        [string]$Name,
        [object[]]$Arguments
    )

    $Object.GetType().InvokeMember(
        $Name,
        [System.Reflection.BindingFlags]::SetProperty,
        $null,
        $Object,
        $Arguments
    ) | Out-Null
}

function Get-ComIndexedProperty {
    param(
        [object]$Object,
        [string]$Name,
        [object[]]$Arguments
    )

    return $Object.GetType().InvokeMember(
        $Name,
        [System.Reflection.BindingFlags]::GetProperty,
        $null,
        $Object,
        $Arguments
    )
}

function Set-MsiSummaryInformation {
    param(
        [string]$MsiPath,
        [string]$PackageCode,
        [DateTime]$Timestamp
    )

    $installer = $null
    $database = $null
    $summary = $null

    try {
        $installer = New-Object -ComObject WindowsInstaller.Installer
        $database = Invoke-ComMethod -Object $installer -Name "OpenDatabase" -Arguments @($MsiPath, 2)
        $summary = Get-ComIndexedProperty -Object $database -Name "SummaryInformation" -Arguments @(20)

        # PID_REVNUMBER is the package code. PID_CREATE_DTM and PID_LASTSAVE_DTM
        # otherwise carry build-time metadata and make byte-for-byte output drift.
        Set-ComIndexedProperty -Object $summary -Name "Property" -Arguments @(9, "{$PackageCode}")
        Set-ComIndexedProperty -Object $summary -Name "Property" -Arguments @(12, $Timestamp)
        Set-ComIndexedProperty -Object $summary -Name "Property" -Arguments @(13, $Timestamp)

        Invoke-ComMethod -Object $summary -Name "Persist" | Out-Null
        Invoke-ComMethod -Object $database -Name "Commit" | Out-Null
    } finally {
        foreach ($object in @($summary, $database, $installer)) {
            if ($null -ne $object -and [System.Runtime.InteropServices.Marshal]::IsComObject($object)) {
                [void][System.Runtime.InteropServices.Marshal]::FinalReleaseComObject($object)
            }
        }

        [System.GC]::Collect()
        [System.GC]::WaitForPendingFinalizers()
    }
}

function Invoke-SummaryInformationChildProcess {
    param(
        [string]$MsiPath,
        [string]$PackageCode,
        [string]$TimestampEpoch
    )

    & pwsh `
        -NoProfile `
        -File $PSCommandPath `
        -InternalMode "set-summary" `
        -OutputMsi $MsiPath `
        -PackageCode $PackageCode `
        -SourceDateEpoch $TimestampEpoch

    if ($LASTEXITCODE -ne 0) {
        throw "MSI summary information update failed with exit code $LASTEXITCODE"
    }
}

function Read-AllBytesWithRetry {
    param(
        [string]$Path,
        [int]$MaxAttempts = 30,
        [int]$DelayMilliseconds = 500
    )

    $lastError = $null
    for ($attempt = 1; $attempt -le $MaxAttempts; $attempt++) {
        try {
            return [System.IO.File]::ReadAllBytes($Path)
        } catch [System.IO.IOException] {
            $lastError = $_.Exception
            Start-Sleep -Milliseconds $DelayMilliseconds
        }
    }

    throw "Could not read $Path after $MaxAttempts attempts: $($lastError.Message)"
}

function Write-AllBytesWithRetry {
    param(
        [string]$Path,
        [byte[]]$Bytes,
        [int]$MaxAttempts = 30,
        [int]$DelayMilliseconds = 500
    )

    $lastError = $null
    for ($attempt = 1; $attempt -le $MaxAttempts; $attempt++) {
        try {
            [System.IO.File]::WriteAllBytes($Path, $Bytes)
            return
        } catch [System.IO.IOException] {
            $lastError = $_.Exception
            Start-Sleep -Milliseconds $DelayMilliseconds
        }
    }

    throw "Could not write $Path after $MaxAttempts attempts: $($lastError.Message)"
}

function Read-UInt16LE {
    param(
        [byte[]]$Bytes,
        [int]$Offset
    )

    return [BitConverter]::ToUInt16($Bytes, $Offset)
}

function Read-UInt32LE {
    param(
        [byte[]]$Bytes,
        [int]$Offset
    )

    return [BitConverter]::ToUInt32($Bytes, $Offset)
}

function Read-UInt64LE {
    param(
        [byte[]]$Bytes,
        [int]$Offset
    )

    return [BitConverter]::ToUInt64($Bytes, $Offset)
}

function Write-UInt64LE {
    param(
        [byte[]]$Bytes,
        [int]$Offset,
        [uint64]$Value
    )

    $valueBytes = [BitConverter]::GetBytes($Value)
    [Buffer]::BlockCopy($valueBytes, 0, $Bytes, $Offset, $valueBytes.Length)
}

function Read-CfbSector {
    param(
        [byte[]]$Bytes,
        [uint32]$SectorId,
        [int]$SectorSize
    )

    $offset = [int64]$SectorSize + ([int64]$SectorId * [int64]$SectorSize)
    if ($offset -lt $SectorSize -or ($offset + $SectorSize) -gt $Bytes.Length) {
        throw "CFB sector $SectorId is outside file bounds"
    }

    $sector = [byte[]]::new($SectorSize)
    [Buffer]::BlockCopy($Bytes, [int]$offset, $sector, 0, $SectorSize)
    return $sector
}

function Get-CfbDifatSectorIds {
    param(
        [byte[]]$Bytes,
        [int]$SectorSize,
        [uint32]$FirstDifatSector,
        [uint32]$DifatSectorCount
    )

    $ids = [System.Collections.Generic.List[uint32]]::new()
    for ($i = 0; $i -lt 109; $i++) {
        $id = Read-UInt32LE -Bytes $Bytes -Offset (76 + ($i * 4))
        if ($id -ne $script:FREESECT -and $id -ne $script:ENDOFCHAIN) {
            $ids.Add($id)
        }
    }

    $sector = $FirstDifatSector
    $entriesPerSector = [int]($SectorSize / 4)
    for ($i = 0; $i -lt $DifatSectorCount; $i++) {
        if ($sector -eq $script:ENDOFCHAIN -or $sector -eq $script:FREESECT) {
            break
        }

        $difatBytes = Read-CfbSector -Bytes $Bytes -SectorId $sector -SectorSize $SectorSize
        for ($entryIndex = 0; $entryIndex -lt ($entriesPerSector - 1); $entryIndex++) {
            $id = Read-UInt32LE -Bytes $difatBytes -Offset ($entryIndex * 4)
            if ($id -ne $script:FREESECT -and $id -ne $script:ENDOFCHAIN) {
                $ids.Add($id)
            }
        }

        $sector = Read-UInt32LE -Bytes $difatBytes -Offset (($entriesPerSector - 1) * 4)
    }

    return $ids.ToArray()
}

function Read-CfbFat {
    param(
        [byte[]]$Bytes,
        [int]$SectorSize
    )

    $firstDifatSector = Read-UInt32LE -Bytes $Bytes -Offset 68
    $difatSectorCount = Read-UInt32LE -Bytes $Bytes -Offset 72
    $difatSectorIds = Get-CfbDifatSectorIds -Bytes $Bytes -SectorSize $SectorSize -FirstDifatSector $firstDifatSector -DifatSectorCount $difatSectorCount

    $fat = [System.Collections.Generic.List[uint32]]::new()
    foreach ($sectorId in $difatSectorIds) {
        $sectorBytes = Read-CfbSector -Bytes $Bytes -SectorId $sectorId -SectorSize $SectorSize
        for ($offset = 0; $offset -lt $SectorSize; $offset += 4) {
            $fat.Add((Read-UInt32LE -Bytes $sectorBytes -Offset $offset))
        }
    }

    return $fat.ToArray()
}

function Assert-CfbSignature {
    param(
        [byte[]]$Bytes,
        [string]$Path
    )

    $signature = [byte[]](0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1)
    for ($i = 0; $i -lt $signature.Length; $i++) {
        if ($Bytes[$i] -ne $signature[$i]) {
            throw "$Path is not an MSI/CFB file"
        }
    }
}

function Update-CfbDirectoryTimes {
    param([byte[]]$Bytes)

    $sectorShift = Read-UInt16LE -Bytes $Bytes -Offset 30
    $sectorSize = 1 -shl $sectorShift
    $firstDirectorySector = Read-UInt32LE -Bytes $Bytes -Offset 48
    $fat = Read-CfbFat -Bytes $Bytes -SectorSize $sectorSize
    $seen = @{}
    $sector = $firstDirectorySector
    $normalizedEntries = 0

    while ($sector -ne $script:ENDOFCHAIN) {
        if (
            $sector -eq $script:FREESECT -or
            $sector -eq $script:FATSECT -or
            $sector -eq $script:DIFSECT
        ) {
            throw "Invalid CFB directory chain entry: $sector"
        }

        if ([int64]$sector -ge $fat.Length) {
            throw "CFB directory sector $sector is outside FAT bounds"
        }

        $key = $sector.ToString()
        if ($seen.ContainsKey($key)) {
            throw "CFB directory chain contains a cycle at sector $sector"
        }
        $seen[$key] = $true

        $sectorOffset = [int64]$sectorSize + ([int64]$sector * [int64]$sectorSize)
        if ($sectorOffset -lt $sectorSize -or ($sectorOffset + $sectorSize) -gt $Bytes.Length) {
            throw "CFB directory sector $sector is outside file bounds"
        }

        for ($entryOffset = 0; $entryOffset -lt $sectorSize; $entryOffset += 128) {
            $entryBase = [int]($sectorOffset + $entryOffset)
            $nameLength = Read-UInt16LE -Bytes $Bytes -Offset ($entryBase + 64)
            $objectType = $Bytes[$entryBase + 66]
            if ($nameLength -gt 0 -or $objectType -ne 0) {
                $normalizedEntries += 1
            }

            # Also clear unused directory slots, because stale bytes in the
            # directory sector are part of the MSI hash even when WiX ignores them.
            Write-UInt64LE -Bytes $Bytes -Offset ($entryBase + 100) -Value 0
            Write-UInt64LE -Bytes $Bytes -Offset ($entryBase + 108) -Value 0
        }

        $sector = $fat[[int]$sector]
    }

    return $normalizedEntries
}

function Get-CfbDirectoryTimestampDrift {
    param([byte[]]$Bytes)

    return @(Get-CfbDirectoryTimestampDriftEntries -Bytes $Bytes).Count
}

function Get-CfbDirectoryTimestampDriftEntries {
    param([byte[]]$Bytes)

    $sectorShift = Read-UInt16LE -Bytes $Bytes -Offset 30
    $sectorSize = 1 -shl $sectorShift
    $firstDirectorySector = Read-UInt32LE -Bytes $Bytes -Offset 48
    $fat = Read-CfbFat -Bytes $Bytes -SectorSize $sectorSize
    $seen = @{}
    $sector = $firstDirectorySector
    $driftEntries = [System.Collections.Generic.List[object]]::new()

    while ($sector -ne $script:ENDOFCHAIN) {
        if (
            $sector -eq $script:FREESECT -or
            $sector -eq $script:FATSECT -or
            $sector -eq $script:DIFSECT
        ) {
            throw "Invalid CFB directory chain entry: $sector"
        }

        if ([int64]$sector -ge $fat.Length) {
            throw "CFB directory sector $sector is outside FAT bounds"
        }

        $key = $sector.ToString()
        if ($seen.ContainsKey($key)) {
            throw "CFB directory chain contains a cycle at sector $sector"
        }
        $seen[$key] = $true

        $sectorOffset = [int64]$sectorSize + ([int64]$sector * [int64]$sectorSize)
        if ($sectorOffset -lt $sectorSize -or ($sectorOffset + $sectorSize) -gt $Bytes.Length) {
            throw "CFB directory sector $sector is outside file bounds"
        }

        for ($entryOffset = 0; $entryOffset -lt $sectorSize; $entryOffset += 128) {
            $entryBase = [int]($sectorOffset + $entryOffset)
            $nameLength = Read-UInt16LE -Bytes $Bytes -Offset ($entryBase + 64)
            $objectType = $Bytes[$entryBase + 66]
            if ($nameLength -le 0 -and $objectType -eq 0) {
                continue
            }

            $creationTime = Read-UInt64LE -Bytes $Bytes -Offset ($entryBase + 100)
            $modifiedTime = Read-UInt64LE -Bytes $Bytes -Offset ($entryBase + 108)
            if ($creationTime -ne 0 -or $modifiedTime -ne 0) {
                $name = ""
                if ($nameLength -gt 2 -and $nameLength -le 64) {
                    $nameBytes = [byte[]]::new($nameLength - 2)
                    [Buffer]::BlockCopy($Bytes, $entryBase, $nameBytes, 0, $nameBytes.Length)
                    $name = [System.Text.Encoding]::Unicode.GetString($nameBytes)
                }

                $driftEntries.Add([pscustomobject]@{
                    Sector = [uint32]$sector
                    EntryOffset = [int64]$entryBase
                    ObjectType = [int]$objectType
                    Name = $name
                    CreationFileTime = $creationTime
                    ModifiedFileTime = $modifiedTime
                })
            }
        }

        $sector = $fat[[int]$sector]
    }

    return $driftEntries.ToArray()
}

function Format-CfbTimestampDrift {
    param([object[]]$Entries)

    if (-not $Entries -or $Entries.Count -eq 0) {
        return ""
    }

    return @($Entries | Select-Object -First 3 | ForEach-Object {
            "sector=$($_.Sector) offset=$($_.EntryOffset) type=$($_.ObjectType) name='$($_.Name)' creation=$($_.CreationFileTime) modified=$($_.ModifiedFileTime)"
        }) -join "; "
}

function Write-CfbDirectoryTimesInPlace {
    param(
        [string]$Path,
        [byte[]]$Bytes
    )

    $sectorShift = Read-UInt16LE -Bytes $Bytes -Offset 30
    $sectorSize = 1 -shl $sectorShift
    $firstDirectorySector = Read-UInt32LE -Bytes $Bytes -Offset 48
    $fat = Read-CfbFat -Bytes $Bytes -SectorSize $sectorSize
    $seen = @{}
    $sector = $firstDirectorySector
    $normalizedEntries = 0
    $zeroTimes = [byte[]]::new(16)
    $stream = $null

    try {
        $stream = [System.IO.File]::Open(
            $Path,
            [System.IO.FileMode]::Open,
            [System.IO.FileAccess]::ReadWrite,
            [System.IO.FileShare]::None
        )

        while ($sector -ne $script:ENDOFCHAIN) {
            if (
                $sector -eq $script:FREESECT -or
                $sector -eq $script:FATSECT -or
                $sector -eq $script:DIFSECT
            ) {
                throw "Invalid CFB directory chain entry: $sector"
            }

            if ([int64]$sector -ge $fat.Length) {
                throw "CFB directory sector $sector is outside FAT bounds"
            }

            $key = $sector.ToString()
            if ($seen.ContainsKey($key)) {
                throw "CFB directory chain contains a cycle at sector $sector"
            }
            $seen[$key] = $true

            $sectorOffset = [int64]$sectorSize + ([int64]$sector * [int64]$sectorSize)
            if ($sectorOffset -lt $sectorSize -or ($sectorOffset + $sectorSize) -gt $Bytes.Length) {
                throw "CFB directory sector $sector is outside file bounds"
            }

            for ($entryOffset = 0; $entryOffset -lt $sectorSize; $entryOffset += 128) {
                $entryBase = [int64]($sectorOffset + $entryOffset)
                $nameLength = Read-UInt16LE -Bytes $Bytes -Offset ([int]($entryBase + 64))
                $objectType = $Bytes[[int]($entryBase + 66)]
                if ($nameLength -gt 0 -or $objectType -ne 0) {
                    $normalizedEntries += 1
                }

                $stream.Seek($entryBase + 100, [System.IO.SeekOrigin]::Begin) | Out-Null
                $stream.Write($zeroTimes, 0, $zeroTimes.Length)
            }

            $sector = $fat[[int]$sector]
        }

        $stream.Flush($true)
    } finally {
        if ($null -ne $stream) {
            $stream.Dispose()
        }
    }

    return $normalizedEntries
}

function Normalize-MsiCfbDirectoryTimes {
    param([string]$MsiPath)

    $resolved = (Resolve-Path -LiteralPath $MsiPath).Path
    $maxAttempts = 20
    $normalizedEntries = 0

    for ($attempt = 1; $attempt -le $maxAttempts; $attempt++) {
        $bytes = Read-AllBytesWithRetry -Path $resolved
        Assert-CfbSignature -Bytes $bytes -Path $resolved
        $normalizedEntries = Update-CfbDirectoryTimes -Bytes $bytes
        $memoryDriftEntries = @(Get-CfbDirectoryTimestampDriftEntries -Bytes $bytes)
        if ($memoryDriftEntries.Count -ne 0) {
            throw "Could not normalize MSI CFB directory timestamps in memory: $(Format-CfbTimestampDrift $memoryDriftEntries)"
        }

        $normalizedEntries = Write-CfbDirectoryTimesInPlace -Path $resolved -Bytes $bytes

        Start-Sleep -Milliseconds 1000

        $verifiedBytes = Read-AllBytesWithRetry -Path $resolved
        Assert-CfbSignature -Bytes $verifiedBytes -Path $resolved
        $driftEntries = @(Get-CfbDirectoryTimestampDriftEntries -Bytes $verifiedBytes)
        if ($driftEntries.Count -eq 0) {
            Write-Output "Normalized MSI CFB directory timestamps for $normalizedEntries entries"
            return
        }

        Write-Output "MSI CFB directory timestamps still nonzero for $($driftEntries.Count) entries after attempt ${attempt}: $(Format-CfbTimestampDrift $driftEntries); retrying"
        Start-Sleep -Milliseconds 1000
    }

    throw "Could not normalize MSI CFB directory timestamps after $maxAttempts attempts"
}

function Copy-NormalizedTree {
    param(
        [string]$Source,
        [string]$Destination,
        [DateTime]$Timestamp
    )

    New-Item -ItemType Directory -Force -Path $Destination | Out-Null

    Get-ChildItem -LiteralPath $Source -Recurse -Force |
        Sort-Object FullName |
        ForEach-Object {
            $relative = [System.IO.Path]::GetRelativePath($Source, $_.FullName)
            $target = Join-Path $Destination $relative

            if ($_.PSIsContainer) {
                New-Item -ItemType Directory -Force -Path $target | Out-Null
            } else {
                New-Item -ItemType Directory -Force -Path (Split-Path -Parent $target) | Out-Null
                Copy-Item -LiteralPath $_.FullName -Destination $target -Force
            }
        }

    Normalize-Tree -Root $Destination -Timestamp $Timestamp
}

function Write-WixSource {
    param(
        [string]$Root,
        [string]$WxsPath,
        [string]$PackageCode,
        [string]$ProductCode,
        [string]$StableUpgradeCode
    )

    $directories = Get-ChildItem -LiteralPath $Root -Recurse -Directory -Force |
        Sort-Object FullName
    $files = Get-ChildItem -LiteralPath $Root -Recurse -File -Force |
        Sort-Object FullName

    $directoryIds = @{}
    $directoryIds["."] = "INSTALLFOLDER"
    foreach ($directory in $directories) {
        $relative = [System.IO.Path]::GetRelativePath($Root, $directory.FullName).Replace("\", "/")
        $directoryIds[$relative] = New-WixId -Prefix "DIR" -Value $relative
    }

    $directoryFragments = New-Object System.Collections.Generic.List[string]
    foreach ($directory in $directories) {
        $relative = [System.IO.Path]::GetRelativePath($Root, $directory.FullName).Replace("\", "/")
        $parentRelative = [System.IO.Path]::GetDirectoryName($relative)
        if ([string]::IsNullOrEmpty($parentRelative)) {
            $parentRelative = "."
        } else {
            $parentRelative = $parentRelative.Replace("\", "/")
        }

        $directoryName = Convert-ToXmlValue ([System.IO.Path]::GetFileName($directory.FullName))
        $directoryFragments.Add("    <DirectoryRef Id=""$($directoryIds[$parentRelative])""><Directory Id=""$($directoryIds[$relative])"" Name=""$directoryName"" /></DirectoryRef>")
    }

    $componentRefs = New-Object System.Collections.Generic.List[string]
    $componentFragments = New-Object System.Collections.Generic.List[string]
    foreach ($file in $files) {
        $relative = [System.IO.Path]::GetRelativePath($Root, $file.FullName).Replace("\", "/")
        $parentRelative = [System.IO.Path]::GetDirectoryName($relative)
        if ([string]::IsNullOrEmpty($parentRelative)) {
            $parentRelative = "."
        } else {
            $parentRelative = $parentRelative.Replace("\", "/")
        }

        $componentId = New-WixId -Prefix "CMP" -Value $relative
        $fileId = New-WixId -Prefix "FIL" -Value $relative
        $componentGuid = New-DeterministicGuid "component:$relative"
        $source = Convert-ToXmlValue $file.FullName
        $name = Convert-ToXmlValue $file.Name

        $componentRefs.Add("      <ComponentRef Id=""$componentId"" />")
        $componentFragments.Add("    <DirectoryRef Id=""$($directoryIds[$parentRelative])""><Component Id=""$componentId"" Guid=""$componentGuid"" Win64=""yes""><File Id=""$fileId"" Name=""$name"" Source=""$source"" KeyPath=""yes"" /></Component></DirectoryRef>")
    }

    $escapedProductName = Convert-ToXmlValue $ProductName
    $escapedManufacturer = Convert-ToXmlValue $Manufacturer

    $lines = New-Object System.Collections.Generic.List[string]
    $lines.Add('<?xml version="1.0" encoding="utf-8"?>')
    $lines.Add('<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">')
    $lines.Add("  <Product Id=""$ProductCode"" Name=""$escapedProductName"" Language=""1033"" Version=""$Version"" Manufacturer=""$escapedManufacturer"" UpgradeCode=""$StableUpgradeCode"">")
    $lines.Add("    <Package Id=""$PackageCode"" InstallerVersion=""500"" Compressed=""yes"" InstallScope=""perMachine"" Platform=""x64"" />")
    $lines.Add('    <MediaTemplate EmbedCab="yes" CompressionLevel="high" />')
    $lines.Add('    <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />')
    $lines.Add('    <Directory Id="TARGETDIR" Name="SourceDir">')
    $lines.Add('      <Directory Id="ProgramFiles64Folder">')
    $lines.Add("        <Directory Id=""INSTALLFOLDER"" Name=""$escapedProductName"" />")
    $lines.Add('      </Directory>')
    $lines.Add('    </Directory>')
    $lines.Add('    <Feature Id="DefaultFeature" Title="Application" Level="1">')
    foreach ($componentRef in $componentRefs) {
        $lines.Add($componentRef)
    }
    $lines.Add('    </Feature>')
    $lines.Add('  </Product>')
    $lines.Add('  <Fragment>')
    foreach ($fragment in $directoryFragments) {
        $lines.Add($fragment)
    }
    foreach ($fragment in $componentFragments) {
        $lines.Add($fragment)
    }
    $lines.Add('  </Fragment>')
    $lines.Add('</Wix>')

    [System.IO.File]::WriteAllLines($WxsPath, $lines, [System.Text.UTF8Encoding]::new($false))
}

function Main {
    $candle = Require-Command "candle.exe"
    $light = Require-Command "light.exe"

    $source = Resolve-SourceDir
    if (-not $Version) {
        $packageJson = Get-Content -LiteralPath "package.json" -Raw | ConvertFrom-Json
        $script:Version = $packageJson.version
    }

    if (-not $UpgradeCode) {
        $script:UpgradeCode = New-DeterministicGuid ("upgrade:{0}:{1}" -f $ProductName, $Manufacturer)
    }

    if (-not $OutputMsi) {
        $OutputMsi = Join-Path (Get-Location) "dist\repro-msi\$ProductName-$Version.msi"
    } elseif (-not [System.IO.Path]::IsPathRooted($OutputMsi)) {
        $OutputMsi = Join-Path (Get-Location) $OutputMsi
    }

    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $OutputMsi) | Out-Null

    $timestamp = [System.DateTimeOffset]::FromUnixTimeSeconds([int64]$SourceDateEpoch).UtcDateTime
    $workdir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
    $staging = Join-Path $workdir "root"
    $wxs = Join-Path $workdir "package.wxs"
    $wixobj = Join-Path $workdir "package.wixobj"
    $manifest = "$OutputMsi.input-sha256"

    try {
        Copy-NormalizedTree -Source $source -Destination $staging -Timestamp $timestamp
        Write-InputManifest -Root $staging -ManifestPath $manifest

        $manifestSeed = Get-ChildItem -LiteralPath $staging -Recurse -File -Force |
            Sort-Object FullName |
            ForEach-Object {
                $relative = [System.IO.Path]::GetRelativePath($staging, $_.FullName).Replace("\", "/")
                $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $_.FullName).Hash.ToLowerInvariant()
                "$relative $hash"
            }

        $packageCode = New-DeterministicGuid (("package:{0}:{1}:" -f $ProductName, $Version) + ($manifestSeed -join "`n"))
        $productCode = New-DeterministicGuid ("product:{0}:{1}:{2}" -f $ProductName, $Version, $Manufacturer)

        Write-WixSource -Root $staging -WxsPath $wxs -PackageCode $packageCode -ProductCode $productCode -StableUpgradeCode $UpgradeCode

        & $candle -nologo -arch x64 -out $wixobj $wxs
        if ($LASTEXITCODE -ne 0) {
            throw "candle.exe failed with exit code $LASTEXITCODE"
        }

        & $light -nologo -sw1076 -out $OutputMsi $wixobj
        if ($LASTEXITCODE -ne 0) {
            throw "light.exe failed with exit code $LASTEXITCODE"
        }

        $outputItem = Get-Item -LiteralPath $OutputMsi
        $outputItem.LastWriteTimeUtc = $timestamp
        $outputItem.CreationTimeUtc = $timestamp
        $outputItem.LastAccessTimeUtc = $timestamp

        Invoke-SummaryInformationChildProcess -MsiPath $OutputMsi -PackageCode $packageCode -TimestampEpoch $SourceDateEpoch
        Normalize-MsiCfbDirectoryTimes -MsiPath $OutputMsi

        $outputItem = Get-Item -LiteralPath $OutputMsi
        $outputItem.LastWriteTimeUtc = $timestamp
        $outputItem.CreationTimeUtc = $timestamp
        $outputItem.LastAccessTimeUtc = $timestamp

        Write-Output $OutputMsi
    } finally {
        if (Test-Path $workdir) {
            Remove-Item -LiteralPath $workdir -Recurse -Force
        }
    }
}

if ($InternalMode) {
    if ($InternalMode -eq "set-summary") {
        if (-not $OutputMsi) {
            throw "OutputMsi is required for set-summary mode."
        }

        if (-not $PackageCode) {
            throw "PackageCode is required for set-summary mode."
        }

        $timestamp = [System.DateTimeOffset]::FromUnixTimeSeconds([int64]$SourceDateEpoch).UtcDateTime
        Set-MsiSummaryInformation -MsiPath $OutputMsi -PackageCode $PackageCode -Timestamp $timestamp
        exit 0
    }

    throw "Unknown internal mode: $InternalMode"
}

Main
