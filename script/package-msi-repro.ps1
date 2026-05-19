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

function Set-MsiSummaryInformation {
    param(
        [string]$MsiPath,
        [string]$PackageCode,
        [DateTime]$Timestamp
    )

    $installer = New-Object -ComObject WindowsInstaller.Installer
    $database = Invoke-ComMethod -Object $installer -Name "OpenDatabase" -Arguments @($MsiPath, 1)
    $summary = Invoke-ComMethod -Object $database -Name "SummaryInformation" -Arguments @(20)

    # PID_REVNUMBER is the package code. PID_CREATE_DTM and PID_LASTSAVE_DTM
    # otherwise carry build-time metadata and make byte-for-byte output drift.
    Set-ComIndexedProperty -Object $summary -Name "Property" -Arguments @(9, "{$PackageCode}")
    Set-ComIndexedProperty -Object $summary -Name "Property" -Arguments @(12, $Timestamp)
    Set-ComIndexedProperty -Object $summary -Name "Property" -Arguments @(13, $Timestamp)

    Invoke-ComMethod -Object $summary -Name "Persist" | Out-Null
    Invoke-ComMethod -Object $database -Name "Commit" | Out-Null
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
