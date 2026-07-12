param(
  [Parameter(Mandatory = $true)]
  [string] $BinaryPath
)

$ErrorActionPreference = "Stop"

$resolvedBinary = (Resolve-Path -LiteralPath $BinaryPath).Path
$bytes = [IO.File]::ReadAllBytes($resolvedBinary)

if ($bytes.Length -lt 256) {
  throw "Windows release binary is unexpectedly small: $resolvedBinary"
}
if ($bytes[0] -ne 0x4d -or $bytes[1] -ne 0x5a) {
  throw "Windows release binary has no MZ header: $resolvedBinary"
}

$peOffset = [BitConverter]::ToInt32($bytes, 0x3c)
if ($peOffset -lt 64 -or ($peOffset + 6) -gt $bytes.Length) {
  throw "Windows release binary has an invalid PE offset: $resolvedBinary"
}

$signature = [Text.Encoding]::ASCII.GetString($bytes, $peOffset, 4)
if ($signature -ne "PE`0`0") {
  throw "Windows release binary has no PE signature: $resolvedBinary"
}

$machine = [BitConverter]::ToUInt16($bytes, $peOffset + 4)
if ($machine -ne 0x8664) {
  throw ("Expected an x86_64 PE binary, found machine type 0x{0:X4}" -f $machine)
}

# Le smoke de démarrage réel nécessite un runtime Npcap. Tant que l'usage de
# Npcap sur les runners n'est pas clarifié avec Nmap (#138), on analyse la
# table d'import du PE sans télécharger ni redistribuer les DLL. Packet.dll est
# une dépendance de wpcap.dll et n'est pas forcément importée directement.
$dumpbin = Get-Command dumpbin.exe -ErrorAction SilentlyContinue
$dumpbinPath = if ($dumpbin) { $dumpbin.Source } else { $null }

if (-not $dumpbinPath) {
  $vswherePath = Join-Path "${env:ProgramFiles(x86)}" "Microsoft Visual Studio\Installer\vswhere.exe"
  if (Test-Path -LiteralPath $vswherePath) {
    $visualStudioPath = (& $vswherePath -latest -products * `
      -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
      -property installationPath | Select-Object -First 1)
    if ($visualStudioPath) {
      $dumpbinPath = Get-ChildItem `
        -Path (Join-Path $visualStudioPath "VC\Tools\MSVC\*\bin\Hostx64\x64\dumpbin.exe") `
        -File |
        Sort-Object FullName -Descending |
        Select-Object -First 1 -ExpandProperty FullName
    }
  }
}
if (-not $dumpbinPath) {
  throw "dumpbin.exe was not found on the Windows build runner."
}

$imports = (& $dumpbinPath /DEPENDENTS $resolvedBinary | Out-String)
if ($LASTEXITCODE -ne 0) {
  throw "dumpbin failed to inspect $resolvedBinary with exit code $LASTEXITCODE"
}
if ($imports -notmatch "(?im)^\s*wpcap\.dll\s*$") {
  throw "Expected direct runtime import wpcap.dll was not found in $resolvedBinary"
}

Write-Host "Windows PE structure and direct wpcap.dll import validated: $resolvedBinary"
