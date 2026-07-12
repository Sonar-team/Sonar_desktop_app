param(
  [string] $TargetDirectory = "src-tauri/target"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $TargetDirectory)) {
  throw "Tauri target directory not found: $TargetDirectory"
}

$bundles = @(
  Get-ChildItem -LiteralPath $TargetDirectory -Recurse -File -Filter "*setup.exe" |
    Where-Object { $_.FullName -match "[\\/]bundle[\\/]" }
)
if ($bundles.Count -ne 1) {
  throw "Expected exactly one Windows NSIS bundle, found $($bundles.Count)."
}

$msiBundles = @(
  Get-ChildItem -LiteralPath $TargetDirectory -Recurse -File -Filter "*.msi" |
    Where-Object { $_.FullName -match "[\\/]bundle[\\/]" }
)
if ($msiBundles.Count -ne 0) {
  throw "MSI bundles are disabled until they implement the Npcap prerequisite flow (#138)."
}

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
  throw "7-Zip is required to inspect the Windows NSIS bundle."
}

$listing = (& $sevenZip.Source l -slt $bundles[0].FullName | Out-String)
if ($LASTEXITCODE -ne 0) {
  throw "7-Zip failed to inspect $($bundles[0].FullName) with exit code $LASTEXITCODE"
}
if ($listing -match "(?i)(npcap|winpcap)[^\r\n]*\.exe") {
  throw "The Windows bundle contains a forbidden Npcap/WinPcap installer."
}

Write-Host "Windows NSIS bundle contains no Npcap/WinPcap installer: $($bundles[0].FullName)"
