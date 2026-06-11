param(
  [string] $InstallerPath = "src-tauri/windows/npcap-1.87.exe"
)

$ErrorActionPreference = "Stop"

$resolvedInstaller = (Resolve-Path -LiteralPath $InstallerPath).Path
$arguments = @("/S", "/winpcap_mode=yes")

Write-Host "Installing Npcap for Windows smoke tests: $resolvedInstaller $($arguments -join ' ')"

$process = Start-Process `
  -FilePath $resolvedInstaller `
  -ArgumentList $arguments `
  -Wait `
  -PassThru

if ($process.ExitCode -notin @(0, 3010)) {
  throw "Npcap installer failed with exit code $($process.ExitCode)"
}

$system32 = Join-Path $env:WINDIR "System32"
$npcapDir = Join-Path $system32 "Npcap"
$requiredDlls = @(
  (Join-Path $system32 "wpcap.dll"),
  (Join-Path $system32 "Packet.dll")
)

foreach ($dll in $requiredDlls) {
  if (-not (Test-Path -LiteralPath $dll)) {
    throw "Npcap WinPcap-compatible DLL is missing after install: $dll"
  }
}

if ($env:GITHUB_PATH) {
  Add-Content -LiteralPath $env:GITHUB_PATH -Value $system32
  if (Test-Path -LiteralPath $npcapDir) {
    Add-Content -LiteralPath $env:GITHUB_PATH -Value $npcapDir
  }
}

Write-Host "Npcap is installed and wpcap.dll is available."
