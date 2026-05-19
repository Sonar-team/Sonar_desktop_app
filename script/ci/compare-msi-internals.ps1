param(
    [Parameter(Mandatory = $true)]
    [string]$FirstMsi,
    [Parameter(Mandatory = $true)]
    [string]$SecondMsi,
    [string]$OutputDir = ""
)

$ErrorActionPreference = "Stop"

$script:FREESECT = [Convert]::ToUInt32("FFFFFFFF", 16)
$script:ENDOFCHAIN = [Convert]::ToUInt32("FFFFFFFE", 16)
$script:FATSECT = [Convert]::ToUInt32("FFFFFFFD", 16)
$script:DIFSECT = [Convert]::ToUInt32("FFFFFFFC", 16)

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

function Convert-ToHex32 {
    param([uint32]$Value)

    return ("0x{0:x8}" -f $Value)
}

function Convert-ToManifestValue {
    param([object]$Value)

    if ($null -eq $Value) {
        return ""
    }

    return $Value.ToString().Replace("`t", "\t").Replace("`r", "\r").Replace("`n", "\n")
}

function Convert-FileTimeToText {
    param([uint64]$Value)

    if ($Value -eq 0) {
        return ""
    }

    try {
        return [DateTime]::FromFileTimeUtc([int64]$Value).ToString("o")
    } catch {
        return $Value.ToString()
    }
}

function Get-Sha256Hex {
    param([byte[]]$Bytes)

    $sha256 = [System.Security.Cryptography.SHA256]::Create()
    try {
        return -join ($sha256.ComputeHash($Bytes) | ForEach-Object { $_.ToString("x2") })
    } finally {
        $sha256.Dispose()
    }
}

function Read-Sector {
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

function Read-Chain {
    param(
        [byte[]]$Bytes,
        [uint32]$StartSector,
        [uint32[]]$Fat,
        [int]$SectorSize,
        [int64]$MaxBytes = -1,
        [string]$ChainName = "CFB sector chain"
    )

    if ($StartSector -eq $script:ENDOFCHAIN) {
        return [byte[]]::new(0)
    }

    $memory = [System.IO.MemoryStream]::new()
    $seen = @{}
    $sector = $StartSector
    $remaining = $MaxBytes

    while ($sector -ne $script:ENDOFCHAIN) {
        if (
            $sector -eq $script:FREESECT -or
            $sector -eq $script:FATSECT -or
            $sector -eq $script:DIFSECT
        ) {
            throw "Invalid CFB sector chain entry: $(Convert-ToHex32 $sector)"
        }

        if ([int64]$sector -ge $Fat.Length) {
            throw "CFB sector $sector is outside FAT bounds"
        }

        $key = $sector.ToString()
        if ($seen.ContainsKey($key)) {
            throw "$ChainName contains a cycle at sector $sector"
        }
        $seen[$key] = $true

        $chunk = Read-Sector -Bytes $Bytes -SectorId $sector -SectorSize $SectorSize
        if ($MaxBytes -ge 0) {
            if ($remaining -le 0) {
                break
            }

            $take = [Math]::Min([int64]$chunk.Length, $remaining)
            $memory.Write($chunk, 0, [int]$take)
            $remaining -= $take
        } else {
            $memory.Write($chunk, 0, $chunk.Length)
        }

        $sector = $Fat[[int]$sector]
    }

    return $memory.ToArray()
}

function Read-MiniChain {
    param(
        [byte[]]$MiniStream,
        [uint32]$StartSector,
        [uint32[]]$MiniFat,
        [int]$MiniSectorSize,
        [int64]$MaxBytes,
        [string]$ChainName = "Mini FAT sector chain"
    )

    if ($StartSector -eq $script:ENDOFCHAIN -or $MaxBytes -eq 0) {
        return [byte[]]::new(0)
    }

    $memory = [System.IO.MemoryStream]::new()
    $seen = @{}
    $sector = $StartSector
    $remaining = $MaxBytes

    while ($sector -ne $script:ENDOFCHAIN) {
        if ($sector -eq $script:FREESECT) {
            throw "Invalid mini FAT sector chain entry: $(Convert-ToHex32 $sector)"
        }

        if ([int64]$sector -ge $MiniFat.Length) {
            throw "Mini FAT sector $sector is outside mini FAT bounds"
        }

        $key = $sector.ToString()
        if ($seen.ContainsKey($key)) {
            throw "$ChainName contains a cycle at sector $sector"
        }
        $seen[$key] = $true

        $offset = [int64]$sector * [int64]$MiniSectorSize
        if ($offset -lt 0 -or $offset -ge $MiniStream.Length) {
            throw "Mini stream sector $sector is outside mini stream bounds"
        }

        $take = [Math]::Min([int64]$MiniSectorSize, [int64]$MiniStream.Length - $offset)
        if ($remaining -ge 0) {
            $take = [Math]::Min($take, $remaining)
        }

        if ($take -le 0) {
            break
        }

        $chunk = [byte[]]::new([int]$take)
        [Buffer]::BlockCopy($MiniStream, [int]$offset, $chunk, 0, [int]$take)
        $memory.Write($chunk, 0, $chunk.Length)

        if ($remaining -ge 0) {
            $remaining -= $take
            if ($remaining -le 0) {
                break
            }
        }

        $sector = $MiniFat[[int]$sector]
    }

    return $memory.ToArray()
}

function Get-DifatSectorIds {
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

        $difatBytes = Read-Sector -Bytes $Bytes -SectorId $sector -SectorSize $SectorSize
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

function Read-Fat {
    param(
        [byte[]]$Bytes,
        [uint32[]]$FatSectorIds,
        [int]$SectorSize
    )

    $entries = [System.Collections.Generic.List[uint32]]::new()
    foreach ($sectorId in $FatSectorIds) {
        $fatBytes = Read-Sector -Bytes $Bytes -SectorId $sectorId -SectorSize $SectorSize
        for ($offset = 0; $offset -lt $fatBytes.Length; $offset += 4) {
            $entries.Add((Read-UInt32LE -Bytes $fatBytes -Offset $offset))
        }
    }

    return $entries.ToArray()
}

function Read-DirectoryEntries {
    param([byte[]]$DirectoryBytes)

    $entries = [System.Collections.Generic.List[object]]::new()
    for ($offset = 0; ($offset + 128) -le $DirectoryBytes.Length; $offset += 128) {
        $nameLength = Read-UInt16LE -Bytes $DirectoryBytes -Offset ($offset + 64)
        $entryType = [int]$DirectoryBytes[$offset + 66]
        if ($entryType -eq 0 -or $nameLength -lt 2) {
            continue
        }

        $name = [System.Text.Encoding]::Unicode.GetString($DirectoryBytes, $offset, $nameLength - 2)
        $entries.Add([pscustomobject]@{
            Index = [int]($offset / 128)
            Name = $name
            Type = $entryType
            Color = [int]$DirectoryBytes[$offset + 67]
            LeftSibling = Read-UInt32LE -Bytes $DirectoryBytes -Offset ($offset + 68)
            RightSibling = Read-UInt32LE -Bytes $DirectoryBytes -Offset ($offset + 72)
            Child = Read-UInt32LE -Bytes $DirectoryBytes -Offset ($offset + 76)
            StateBits = Read-UInt32LE -Bytes $DirectoryBytes -Offset ($offset + 96)
            CreatedUtc = Convert-FileTimeToText (Read-UInt64LE -Bytes $DirectoryBytes -Offset ($offset + 100))
            ModifiedUtc = Convert-FileTimeToText (Read-UInt64LE -Bytes $DirectoryBytes -Offset ($offset + 108))
            StartSector = Read-UInt32LE -Bytes $DirectoryBytes -Offset ($offset + 116)
            Size = [uint64](Read-UInt64LE -Bytes $DirectoryBytes -Offset ($offset + 120))
        })
    }

    return $entries.ToArray()
}

function Test-CabBytes {
    param([byte[]]$Bytes)

    return (
        $Bytes.Length -ge 4 -and
        $Bytes[0] -eq 0x4d -and
        $Bytes[1] -eq 0x53 -and
        $Bytes[2] -eq 0x43 -and
        $Bytes[3] -eq 0x46
    )
}

function Convert-ToStreamManifest {
    param([object[]]$Streams)

    return @($Streams | Sort-Object Name | ForEach-Object {
            "stream`t$(Convert-ToManifestValue $_.Name)`t$($_.Size)`t$($_.Sha256)`t$($_.IsCab)`t$(Convert-ToManifestValue $_.ReadError)"
        })
}

function Convert-ToCabManifest {
    param([object[]]$Streams)

    return @($Streams | Where-Object { $_.IsCab } | Sort-Object Name | ForEach-Object {
            "cab`t$(Convert-ToManifestValue $_.Name)`t$($_.Size)`t$($_.Sha256)"
        })
}

function Convert-ToLayoutManifest {
    param(
        [object]$Parsed,
        [object[]]$Entries
    )

    $lines = [System.Collections.Generic.List[string]]::new()
    $lines.Add("file`tSize`t$($Parsed.FileSize)")
    $lines.Add("header`tSha256`t$($Parsed.HeaderSha256)")
    $lines.Add("header`tMajorVersion`t$($Parsed.MajorVersion)")
    $lines.Add("header`tSectorSize`t$($Parsed.SectorSize)")
    $lines.Add("header`tMiniSectorSize`t$($Parsed.MiniSectorSize)")
    $lines.Add("header`tFatSectorCount`t$($Parsed.FatSectorCount)")
    $lines.Add("header`tDirectoryStartSector`t$(Convert-ToHex32 $Parsed.DirectoryStartSector)")
    $lines.Add("header`tMiniStreamCutoff`t$($Parsed.MiniStreamCutoff)")
    $lines.Add("header`tMiniFatStartSector`t$(Convert-ToHex32 $Parsed.MiniFatStartSector)")
    $lines.Add("header`tMiniFatSectorCount`t$($Parsed.MiniFatSectorCount)")
    $lines.Add("header`tDifatStartSector`t$(Convert-ToHex32 $Parsed.DifatStartSector)")
    $lines.Add("header`tDifatSectorCount`t$($Parsed.DifatSectorCount)")

    foreach ($entry in ($Entries | Sort-Object Index)) {
        $lines.Add(
            "entry`t$($entry.Index)`t$($entry.Type)`t$(Convert-ToManifestValue $entry.Name)`t$($entry.Color)`t$(Convert-ToHex32 $entry.LeftSibling)`t$(Convert-ToHex32 $entry.RightSibling)`t$(Convert-ToHex32 $entry.Child)`t$($entry.StateBits)`t$(Convert-ToHex32 $entry.StartSector)`t$($entry.Size)`t$(Convert-ToManifestValue $entry.CreatedUtc)`t$(Convert-ToManifestValue $entry.ModifiedUtc)"
        )
    }

    return $lines.ToArray()
}

function Convert-ToSectorManifest {
    param(
        [byte[]]$Bytes,
        [uint32[]]$Fat,
        [int]$SectorSize
    )

    $lines = [System.Collections.Generic.List[string]]::new()
    $header = [byte[]]::new($SectorSize)
    [Buffer]::BlockCopy($Bytes, 0, $header, 0, $SectorSize)
    $lines.Add("header`tsha256`t$(Get-Sha256Hex $header)")

    $sectorCount = [int](($Bytes.Length - $SectorSize) / $SectorSize)
    for ($sectorId = 0; $sectorId -lt $sectorCount; $sectorId++) {
        $sector = Read-Sector -Bytes $Bytes -SectorId ([uint32]$sectorId) -SectorSize $SectorSize
        $fatValue = ""
        if ($sectorId -lt $Fat.Length) {
            $fatValue = Convert-ToHex32 $Fat[$sectorId]
        }

        $lines.Add("sector`t$sectorId`t$fatValue`t$(Get-Sha256Hex $sector)")
    }

    return $lines.ToArray()
}

function Read-CfbFile {
    param([string]$Path)

    $resolved = (Resolve-Path -LiteralPath $Path).Path
    $bytes = [System.IO.File]::ReadAllBytes($resolved)
    if ($bytes.Length -lt 512) {
        throw "$Path is too small to be a CFB file"
    }

    $magic = @(0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1)
    for ($i = 0; $i -lt $magic.Length; $i++) {
        if ($bytes[$i] -ne $magic[$i]) {
            throw "$Path is not an MSI/CFB file"
        }
    }

    $majorVersion = Read-UInt16LE -Bytes $bytes -Offset 26
    $byteOrder = Read-UInt16LE -Bytes $bytes -Offset 28
    if ($byteOrder -ne 0xfffe) {
        throw "$Path uses unsupported CFB byte order: $byteOrder"
    }

    $sectorSize = 1 -shl (Read-UInt16LE -Bytes $bytes -Offset 30)
    $miniSectorSize = 1 -shl (Read-UInt16LE -Bytes $bytes -Offset 32)
    $fatSectorCount = Read-UInt32LE -Bytes $bytes -Offset 44
    $directoryStartSector = Read-UInt32LE -Bytes $bytes -Offset 48
    $miniStreamCutoff = Read-UInt32LE -Bytes $bytes -Offset 56
    $miniFatStartSector = Read-UInt32LE -Bytes $bytes -Offset 60
    $miniFatSectorCount = Read-UInt32LE -Bytes $bytes -Offset 64
    $difatStartSector = Read-UInt32LE -Bytes $bytes -Offset 68
    $difatSectorCount = Read-UInt32LE -Bytes $bytes -Offset 72

    $fatSectorIds = Get-DifatSectorIds `
        -Bytes $bytes `
        -SectorSize $sectorSize `
        -FirstDifatSector $difatStartSector `
        -DifatSectorCount $difatSectorCount

    if ($fatSectorIds.Length -lt $fatSectorCount) {
        throw "$Path declares $fatSectorCount FAT sectors but only $($fatSectorIds.Length) were found"
    }

    if ($fatSectorIds.Length -gt $fatSectorCount) {
        $fatSectorIds = @($fatSectorIds | Select-Object -First $fatSectorCount)
    }

    $fat = Read-Fat -Bytes $bytes -FatSectorIds $fatSectorIds -SectorSize $sectorSize
    $directoryBytes = Read-Chain `
        -Bytes $bytes `
        -StartSector $directoryStartSector `
        -Fat $fat `
        -SectorSize $sectorSize `
        -ChainName "$Path directory stream"

    $entries = Read-DirectoryEntries -DirectoryBytes $directoryBytes
    $rootEntry = @($entries | Where-Object { $_.Type -eq 5 } | Select-Object -First 1)
    if (-not $rootEntry) {
        throw "$Path does not contain a CFB root entry"
    }

    $miniFatBytes = [byte[]]::new(0)
    if ($miniFatStartSector -ne $script:ENDOFCHAIN -and $miniFatSectorCount -gt 0) {
        try {
            $miniFatBytes = Read-Chain `
                -Bytes $bytes `
                -StartSector $miniFatStartSector `
                -Fat $fat `
                -SectorSize $sectorSize `
                -MaxBytes ([int64]$miniFatSectorCount * [int64]$sectorSize) `
                -ChainName "$Path mini FAT"
        } catch {
            Write-Output "::warning::Failed to read MSI mini FAT from ${Path}: $($_.Exception.Message)"
            $miniFatBytes = [byte[]]::new(0)
        }
    }

    $miniFat = [System.Collections.Generic.List[uint32]]::new()
    for ($offset = 0; ($offset + 4) -le $miniFatBytes.Length; $offset += 4) {
        $miniFat.Add((Read-UInt32LE -Bytes $miniFatBytes -Offset $offset))
    }

    $miniStream = [byte[]]::new(0)
    if ($rootEntry.StartSector -ne $script:ENDOFCHAIN -and $rootEntry.Size -gt 0) {
        try {
            $miniStream = Read-Chain `
                -Bytes $bytes `
                -StartSector $rootEntry.StartSector `
                -Fat $fat `
                -SectorSize $sectorSize `
                -MaxBytes ([int64]$rootEntry.Size) `
                -ChainName "$Path root mini stream"
        } catch {
            Write-Output "::warning::Failed to read MSI root mini stream from ${Path}: $($_.Exception.Message)"
            $miniStream = [byte[]]::new(0)
        }
    }

    $streams = [System.Collections.Generic.List[object]]::new()
    foreach ($entry in ($entries | Where-Object { $_.Type -eq 2 })) {
        $streamBytes = [byte[]]::new(0)
        $readError = ""

        try {
            if ($entry.Size -eq 0) {
                $streamBytes = [byte[]]::new(0)
            } elseif ($entry.Size -lt $miniStreamCutoff) {
                $streamBytes = Read-MiniChain `
                    -MiniStream $miniStream `
                    -StartSector $entry.StartSector `
                    -MiniFat $miniFat.ToArray() `
                    -MiniSectorSize $miniSectorSize `
                    -MaxBytes ([int64]$entry.Size) `
                    -ChainName "$Path stream '$($entry.Name)' mini stream"
            } else {
                $streamBytes = Read-Chain `
                    -Bytes $bytes `
                    -StartSector $entry.StartSector `
                    -Fat $fat `
                    -SectorSize $sectorSize `
                    -MaxBytes ([int64]$entry.Size) `
                    -ChainName "$Path stream '$($entry.Name)'"
            }
        } catch {
            $readError = $_.Exception.Message
            Write-Output "::warning::Failed to read MSI stream '$($entry.Name)' from ${Path}: $readError"
        }

        if ($readError) {
            $streamSha256 = "READ_ERROR"
            $isCab = $false
        } else {
            $streamSha256 = Get-Sha256Hex $streamBytes
            $isCab = Test-CabBytes $streamBytes
        }

        $streams.Add([pscustomobject]@{
            Name = $entry.Name
            Size = [uint64]$entry.Size
            StartSector = $entry.StartSector
            Sha256 = $streamSha256
            IsCab = $isCab
            ReadError = $readError
        })
    }

    $header = [byte[]]::new($sectorSize)
    [Buffer]::BlockCopy($bytes, 0, $header, 0, $sectorSize)

    return [pscustomobject]@{
        Path = $resolved
        FileSize = $bytes.Length
        HeaderSha256 = Get-Sha256Hex $header
        MajorVersion = $majorVersion
        Bytes = $bytes
        SectorSize = $sectorSize
        MiniSectorSize = $miniSectorSize
        FatSectorCount = $fatSectorCount
        DirectoryStartSector = $directoryStartSector
        MiniStreamCutoff = $miniStreamCutoff
        MiniFatStartSector = $miniFatStartSector
        MiniFatSectorCount = $miniFatSectorCount
        DifatStartSector = $difatStartSector
        DifatSectorCount = $difatSectorCount
        Fat = $fat
        Entries = $entries
        Streams = $streams.ToArray()
        StreamManifest = Convert-ToStreamManifest $streams.ToArray()
        CabManifest = Convert-ToCabManifest $streams.ToArray()
    }
}

function Write-Manifest {
    param(
        [string]$Path,
        [string[]]$Lines
    )

    [System.IO.File]::WriteAllLines($Path, $Lines, [System.Text.UTF8Encoding]::new($false))
}

function Test-ManifestEqual {
    param(
        [string[]]$First,
        [string[]]$Second
    )

    return (($First -join "`n") -eq ($Second -join "`n"))
}

function Show-ManifestDiff {
    param(
        [string]$Label,
        [string[]]$First,
        [string[]]$Second
    )

    $diff = @(Compare-Object -ReferenceObject $First -DifferenceObject $Second -SyncWindow 0 | Select-Object -First 80)
    if ($diff.Length -eq 0) {
        return
    }

    Write-Output "$Label diff sample:"
    $diff | Format-Table -AutoSize | Out-String | Write-Output
}

function Main {
    $first = Read-CfbFile -Path $FirstMsi
    $second = Read-CfbFile -Path $SecondMsi

    $firstLayout = Convert-ToLayoutManifest -Parsed $first -Entries $first.Entries
    $secondLayout = Convert-ToLayoutManifest -Parsed $second -Entries $second.Entries
    $firstSectors = Convert-ToSectorManifest -Bytes $first.Bytes -Fat $first.Fat -SectorSize $first.SectorSize
    $secondSectors = Convert-ToSectorManifest -Bytes $second.Bytes -Fat $second.Fat -SectorSize $second.SectorSize

    if ($OutputDir) {
        New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
        Write-Manifest -Path (Join-Path $OutputDir "first.cfb-streams.tsv") -Lines $first.StreamManifest
        Write-Manifest -Path (Join-Path $OutputDir "second.cfb-streams.tsv") -Lines $second.StreamManifest
        Write-Manifest -Path (Join-Path $OutputDir "first.cfb-cabs.tsv") -Lines $first.CabManifest
        Write-Manifest -Path (Join-Path $OutputDir "second.cfb-cabs.tsv") -Lines $second.CabManifest
        Write-Manifest -Path (Join-Path $OutputDir "first.cfb-layout.tsv") -Lines $firstLayout
        Write-Manifest -Path (Join-Path $OutputDir "second.cfb-layout.tsv") -Lines $secondLayout
        Write-Manifest -Path (Join-Path $OutputDir "first.cfb-sectors.tsv") -Lines $firstSectors
        Write-Manifest -Path (Join-Path $OutputDir "second.cfb-sectors.tsv") -Lines $secondSectors
        Write-Output "MSI internal manifests written to $OutputDir"
    }

    $streamsEqual = Test-ManifestEqual -First $first.StreamManifest -Second $second.StreamManifest
    $cabsEqual = Test-ManifestEqual -First $first.CabManifest -Second $second.CabManifest
    $layoutEqual = Test-ManifestEqual -First $firstLayout -Second $secondLayout
    $sectorsEqual = Test-ManifestEqual -First $firstSectors -Second $secondSectors

    if ($streamsEqual) {
        Write-Output "MSI internal stream contents are identical"
    } else {
        Write-Output "::warning::MSI internal stream contents differ"
        Show-ManifestDiff -Label "MSI internal stream content" -First $first.StreamManifest -Second $second.StreamManifest
    }

    if ($cabsEqual) {
        Write-Output "MSI embedded CAB streams are identical"
    } else {
        Write-Output "::warning::MSI embedded CAB streams differ"
        Show-ManifestDiff -Label "MSI embedded CAB stream" -First $first.CabManifest -Second $second.CabManifest
    }

    if ($layoutEqual) {
        Write-Output "MSI CFB directory/header layout is identical"
    } else {
        Write-Output "::warning::MSI CFB directory/header layout differs"
        Show-ManifestDiff -Label "MSI CFB layout" -First $firstLayout -Second $secondLayout
    }

    if ($sectorsEqual) {
        Write-Output "MSI CFB sector bytes are identical"
    } else {
        Write-Output "::warning::MSI CFB sector bytes differ"
        Show-ManifestDiff -Label "MSI CFB sector" -First $firstSectors -Second $secondSectors
    }

    if ($streamsEqual -and $cabsEqual -and (-not $sectorsEqual)) {
        Write-Output "::warning::MSI logical streams and embedded CABs are identical; remaining drift is in the OLE/CFB container layout or unused sector bytes."
    }
}

try {
    Main
} catch {
    Write-Output "::warning::MSI internal diagnostic failed: $($_.Exception.Message)"
    exit 0
}
