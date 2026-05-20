param(
    [string]$SourceDir = "",
    [string]$OutputMsi = "",
    [string]$ProductName = "sonar",
    [string]$Manufacturer = "Sonar Team",
    [string]$Version = "",
    [string]$UpgradeCode = "",
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

    $installer = New-Object -ComObject WindowsInstaller.Installer
    $database = Invoke-ComMethod -Object $installer -Name "OpenDatabase" -Arguments @($MsiPath, 1)
    $summary = Get-ComIndexedProperty -Object $database -Name "SummaryInformation" -Arguments @(20)

    # PID_REVNUMBER is the package code. PID_CREATE_DTM and PID_LASTSAVE_DTM
    # otherwise carry build-time metadata and make byte-for-byte output drift.
    Set-ComIndexedProperty -Object $summary -Name "Property" -Arguments @(9, "{$PackageCode}")
    Set-ComIndexedProperty -Object $summary -Name "Property" -Arguments @(12, $Timestamp)
    Set-ComIndexedProperty -Object $summary -Name "Property" -Arguments @(13, $Timestamp)

    Invoke-ComMethod -Object $summary -Name "Persist" | Out-Null
    Invoke-ComMethod -Object $database -Name "Commit" | Out-Null
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

function Normalize-MsiCfbDirectoryTimes {
    param([string]$MsiPath)

    $resolved = (Resolve-Path -LiteralPath $MsiPath).Path
    $bytes = [System.IO.File]::ReadAllBytes($resolved)

    $signature = [byte[]](0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1)
    for ($i = 0; $i -lt $signature.Length; $i++) {
        if ($bytes[$i] -ne $signature[$i]) {
            throw "$resolved is not an MSI/CFB file"
        }
    }

    $sectorShift = Read-UInt16LE -Bytes $bytes -Offset 30
    $sectorSize = 1 -shl $sectorShift
    $firstDirectorySector = Read-UInt32LE -Bytes $bytes -Offset 48
    $fat = Read-CfbFat -Bytes $bytes -SectorSize $sectorSize
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
        if ($sectorOffset -lt $sectorSize -or ($sectorOffset + $sectorSize) -gt $bytes.Length) {
            throw "CFB directory sector $sector is outside file bounds"
        }

        for ($entryOffset = 0; $entryOffset -lt $sectorSize; $entryOffset += 128) {
            $entryBase = [int]($sectorOffset + $entryOffset)
            $nameLength = Read-UInt16LE -Bytes $bytes -Offset ($entryBase + 64)
            $objectType = $bytes[$entryBase + 66]
            if ($nameLength -gt 0 -or $objectType -ne 0) {
                Write-UInt64LE -Bytes $bytes -Offset ($entryBase + 100) -Value 0
                Write-UInt64LE -Bytes $bytes -Offset ($entryBase + 108) -Value 0
                $normalizedEntries += 1
            }
        }

        $sector = $fat[[int]$sector]
    }

    [System.IO.File]::WriteAllBytes($resolved, $bytes)
    Write-Output "Normalized MSI CFB directory timestamps for $normalizedEntries entries"
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

        Set-MsiSummaryInformation -MsiPath $OutputMsi -PackageCode $packageCode -Timestamp $timestamp
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

Main
