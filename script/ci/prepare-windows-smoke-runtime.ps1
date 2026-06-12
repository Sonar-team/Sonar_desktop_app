param(
  [Parameter(Mandatory = $true)]
  [string] $BinaryPath,

  [string] $InstallerPath = "src-tauri/windows/npcap-1.87.exe"
)

$ErrorActionPreference = "Stop"

function Convert-ToGitBashPath {
  param([Parameter(Mandatory = $true)][string] $Path)

  $forwardPath = $Path -replace "\\", "/"
  if ($forwardPath -match "^([A-Za-z]):/(.*)$") {
    return "/$($matches[1].ToLower())/$($matches[2])"
  }

  return $forwardPath
}

$resolvedBinary = (Resolve-Path -LiteralPath $BinaryPath).Path
$resolvedInstaller = (Resolve-Path -LiteralPath $InstallerPath).Path

$sevenZip = Get-Command 7z.exe -ErrorAction SilentlyContinue
if (-not $sevenZip) {
  $sevenZip = Get-Command 7z -ErrorAction SilentlyContinue
}
if (-not $sevenZip) {
  $defaultSevenZip = Join-Path $env:ProgramFiles "7-Zip\7z.exe"
  if (Test-Path -LiteralPath $defaultSevenZip) {
    $sevenZip = [pscustomobject]@{ Source = $defaultSevenZip }
  }
}
if (-not $sevenZip) {
  throw "7-Zip is required to prepare the Windows smoke runtime."
}

$runnerTemp = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [IO.Path]::GetTempPath() }
$runtimeDir = Join-Path $runnerTemp "sonar-windows-smoke-runtime"
$extractDir = Join-Path $runtimeDir "npcap-extract"

if (Test-Path -LiteralPath $runtimeDir) {
  Remove-Item -LiteralPath $runtimeDir -Recurse -Force
}
New-Item -ItemType Directory -Path $runtimeDir, $extractDir | Out-Null

Write-Host "Preparing Windows smoke runtime in $runtimeDir"
Write-Host "Extracting Npcap DLLs from $resolvedInstaller"

& $sevenZip.Source x "-o$extractDir" -y $resolvedInstaller "wpcap_x64.dll" "Packet_x64.dll"
if ($LASTEXITCODE -ne 0) {
  throw "7-Zip failed to extract Npcap smoke DLLs with exit code $LASTEXITCODE"
}

$smokeExe = Join-Path $runtimeDir (Split-Path -Leaf $resolvedBinary)
Copy-Item -LiteralPath $resolvedBinary -Destination $smokeExe -Force
Copy-Item -LiteralPath (Join-Path $extractDir "wpcap_x64.dll") -Destination (Join-Path $runtimeDir "wpcap.dll") -Force
Copy-Item -LiteralPath (Join-Path $extractDir "Packet_x64.dll") -Destination (Join-Path $runtimeDir "Packet.dll") -Force

if ($env:GITHUB_OUTPUT) {
  Add-Content -LiteralPath $env:GITHUB_OUTPUT -Value "smoke_binary_path=$(Convert-ToGitBashPath -Path $smokeExe)"
  Add-Content -LiteralPath $env:GITHUB_OUTPUT -Value "smoke_binary_path_windows=$smokeExe"
}

Write-Host "Windows smoke executable: $smokeExe"
