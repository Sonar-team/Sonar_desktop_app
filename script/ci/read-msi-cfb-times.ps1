param(
    [string]$MsiPath = "",
    [switch]$Json,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

if ($Help) {
    @"
usage: pwsh -File script/ci/read-msi-cfb-times.ps1 -MsiPath <path> [-Json]

Read the root Compound File Binary directory timestamps from an MSI file.
"@ | Write-Output
    exit 0
}

if (-not $MsiPath) {
    throw "MsiPath is required. Use -Help for usage."
}

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

function Convert-FileTimeToText {
    param([uint64]$Value)

    if ($Value -eq 0) {
        return "<zero>"
    }

    try {
        return [DateTime]::FromFileTimeUtc([int64]$Value).ToString("o")
    } catch {
        return "<invalid:$Value>"
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
        [int]$SectorSize
    )

    $firstDifatSector = Read-UInt32LE -Bytes $Bytes -Offset 68
    $difatSectorCount = Read-UInt32LE -Bytes $Bytes -Offset 72
    $difatSectorIds = Get-DifatSectorIds -Bytes $Bytes -SectorSize $SectorSize -FirstDifatSector $firstDifatSector -DifatSectorCount $difatSectorCount

    $fat = [System.Collections.Generic.List[uint32]]::new()
    foreach ($sectorId in $difatSectorIds) {
        $sectorBytes = Read-Sector -Bytes $Bytes -SectorId $sectorId -SectorSize $SectorSize
        for ($offset = 0; $offset -lt $SectorSize; $offset += 4) {
            $fat.Add((Read-UInt32LE -Bytes $sectorBytes -Offset $offset))
        }
    }

    return $fat.ToArray()
}

function Read-Chain {
    param(
        [byte[]]$Bytes,
        [uint32]$StartSector,
        [uint32[]]$Fat,
        [int]$SectorSize,
        [string]$ChainName
    )

    if ($StartSector -eq $script:ENDOFCHAIN) {
        return [byte[]]::new(0)
    }

    $memory = [System.IO.MemoryStream]::new()
    $seen = @{}
    $sector = $StartSector

    while ($sector -ne $script:ENDOFCHAIN) {
        if (
            $sector -eq $script:FREESECT -or
            $sector -eq $script:FATSECT -or
            $sector -eq $script:DIFSECT
        ) {
            throw "Invalid $ChainName entry: $sector"
        }

        if ([int64]$sector -ge $Fat.Length) {
            throw "$ChainName sector $sector is outside FAT bounds"
        }

        $key = $sector.ToString()
        if ($seen.ContainsKey($key)) {
            throw "$ChainName contains a cycle at sector $sector"
        }
        $seen[$key] = $true

        $chunk = Read-Sector -Bytes $Bytes -SectorId $sector -SectorSize $SectorSize
        $memory.Write($chunk, 0, $chunk.Length)

        $sector = $Fat[[int]$sector]
    }

    return $memory.ToArray()
}

function Read-DirectoryEntry {
    param(
        [byte[]]$Bytes,
        [int]$Offset,
        [int]$Index
    )

    $nameLength = Read-UInt16LE -Bytes $Bytes -Offset ($Offset + 64)
    $name = ""
    if ($nameLength -gt 2 -and $nameLength -le 64) {
        $nameBytes = [byte[]]::new($nameLength - 2)
        [Buffer]::BlockCopy($Bytes, $Offset, $nameBytes, 0, $nameBytes.Length)
        $name = [System.Text.Encoding]::Unicode.GetString($nameBytes)
    }

    $creationFileTime = Read-UInt64LE -Bytes $Bytes -Offset ($Offset + 100)
    $modifiedFileTime = Read-UInt64LE -Bytes $Bytes -Offset ($Offset + 108)

    return [pscustomobject]@{
        Index = $Index
        Name = $name
        ObjectType = $Bytes[$Offset + 66]
        CreationFileTime = $creationFileTime
        CreationUtc = Convert-FileTimeToText $creationFileTime
        ModifiedFileTime = $modifiedFileTime
        ModifiedUtc = Convert-FileTimeToText $modifiedFileTime
    }
}

function Read-MsiRootCfbTimestamps {
    param([string]$Path)

    $resolved = (Resolve-Path -LiteralPath $Path).Path
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
    $fat = Read-Fat -Bytes $bytes -SectorSize $sectorSize
    $directoryBytes = Read-Chain -Bytes $bytes -StartSector $firstDirectorySector -Fat $fat -SectorSize $sectorSize -ChainName "CFB directory chain"

    if ($directoryBytes.Length -lt 128) {
        throw "$resolved does not contain a complete CFB root directory entry"
    }

    $rootEntry = Read-DirectoryEntry -Bytes $directoryBytes -Offset 0 -Index 0
    if ($rootEntry.Name -ne "Root Entry") {
        throw "$resolved has unexpected CFB root entry name: '$($rootEntry.Name)'"
    }

    return [pscustomobject]@{
        Path = $resolved
        SectorSize = $sectorSize
        RootEntry = $rootEntry
    }
}

$result = Read-MsiRootCfbTimestamps -Path $MsiPath

if ($Json) {
    $result | ConvertTo-Json -Depth 4
} else {
    Write-Output "Path: $($result.Path)"
    Write-Output "SectorSize: $($result.SectorSize)"
    Write-Output "RootEntry.Name: $($result.RootEntry.Name)"
    Write-Output "RootEntry.ObjectType: $($result.RootEntry.ObjectType)"
    Write-Output "RootEntry.CreationFileTime: $($result.RootEntry.CreationFileTime)"
    Write-Output "RootEntry.CreationUtc: $($result.RootEntry.CreationUtc)"
    Write-Output "RootEntry.ModifiedFileTime: $($result.RootEntry.ModifiedFileTime)"
    Write-Output "RootEntry.ModifiedUtc: $($result.RootEntry.ModifiedUtc)"
}
