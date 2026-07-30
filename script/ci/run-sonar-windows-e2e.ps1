# E2E Windows (runner self-hosted, #146 phase 1) : installe le paquet NSIS
# de release, vérifie que SONAR démarre et détecte réellement Npcap (via
# --sonar-smoke-test, cf. src-tauri/src/startup_smoke.rs), puis désinstalle.
#
# Contrairement à validate-windows-release-binary.ps1 (inspection statique du
# PE, utilisable sur un runner GitHub-hosted sans Npcap), ce script exécute
# vraiment le binaire installé : il suppose que Npcap est déjà présent sur la
# machine, installé manuellement en mode "WinPcap API-compatible" (cf.
# security/licences.md, #138). Ne doit tourner que sur un runner self-hosted
# dédié — jamais sur un workflow déclenché par pull_request.
#
# La machine est persistante et réexécutée à chaque run : le script se
# nettoie avant (désinstalle tout SONAR résiduel) et après (finally), pour
# rester idempotent.

param(
  [Parameter(Mandatory = $true)]
  [string] $InstallerPath,

  [string] $ArtifactsDir = "windows-e2e-artifacts"
)

$ErrorActionPreference = "Stop"

$resolvedInstaller = (Resolve-Path -LiteralPath $InstallerPath).Path
New-Item -ItemType Directory -Force -Path $ArtifactsDir | Out-Null
$artifactsDir = (Resolve-Path -LiteralPath $ArtifactsDir).Path

$uninstallRoots = @(
  "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*",
  "HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*",
  "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*"
)

function Get-SonarUninstallEntries {
  foreach ($root in $uninstallRoots) {
    Get-ItemProperty -Path $root -ErrorAction SilentlyContinue |
      Where-Object { $_.DisplayName -like "*SONAR*" -or $_.DisplayName -like "*sonar*" }
  }
}

# Sépare l'exécutable des éventuels arguments dans une UninstallString NSIS
# (typiquement `"C:\...\uninstall.exe"` ou `"C:\...\uninstall.exe" /flag`).
function Split-UninstallString([string] $uninstallString) {
  if ($uninstallString -match '^\s*"([^"]+)"\s*(.*)$') {
    return [PSCustomObject]@{ Exe = $matches[1]; Args = $matches[2].Trim() }
  }
  $parts = $uninstallString -split ' ', 2
  $extraArgs = if ($parts.Length -gt 1) { $parts[1].Trim() } else { "" }
  return [PSCustomObject]@{ Exe = $parts[0]; Args = $extraArgs }
}

function Uninstall-Sonar {
  foreach ($entry in @(Get-SonarUninstallEntries)) {
    if (-not $entry.UninstallString) { continue }
    Write-Host "Désinstallation d'une installation SONAR existante : $($entry.DisplayName)"
    $split = Split-UninstallString $entry.UninstallString
    if (-not (Test-Path -LiteralPath $split.Exe)) { continue }
    $argList = @($split.Args, "/S") | Where-Object { $_ -ne "" }
    Start-Process -FilePath $split.Exe -ArgumentList $argList -Wait -ErrorAction SilentlyContinue
  }
}

function Get-SonarExePath {
  $entry = Get-SonarUninstallEntries | Select-Object -First 1
  if (-not $entry) { return $null }

  $candidateDirs = @()
  if ($entry.InstallLocation) { $candidateDirs += $entry.InstallLocation }
  if ($entry.UninstallString) {
    $candidateDirs += (Split-Path -Parent (Split-UninstallString $entry.UninstallString).Exe)
  }

  foreach ($dir in $candidateDirs) {
    if ($dir -and (Test-Path -LiteralPath $dir)) {
      $exe = Get-ChildItem -LiteralPath $dir -Filter "sonar.exe" -File -Recurse -ErrorAction SilentlyContinue |
        Select-Object -First 1
      if ($exe) { return $exe.FullName }
    }
  }
  return $null
}

$summaryLines = New-Object System.Collections.Generic.List[string]
function Add-Summary([string] $line) {
  Write-Host $line
  $summaryLines.Add($line)
}

$exitCode = 1
try {
  Add-Summary "== SONAR Windows E2E (installation + détection Npcap) =="
  Add-Summary "Installeur : $resolvedInstaller"

  Add-Summary "-- Nettoyage préalable (désinstallation d'un éventuel résidu) --"
  Uninstall-Sonar

  Add-Summary "-- Installation silencieuse --"
  $installProc = Start-Process -FilePath $resolvedInstaller -ArgumentList "/S" -Wait -PassThru
  if ($installProc.ExitCode -ne 0) {
    throw "L'installeur a retourné le code $($installProc.ExitCode)"
  }

  $sonarExe = Get-SonarExePath
  if (-not $sonarExe) {
    throw "sonar.exe introuvable après installation silencieuse (clé de désinstallation absente ou chemin d'installation invalide)."
  }
  Add-Summary "Binaire installé : $sonarExe"

  Add-Summary "-- Exécution de --sonar-smoke-test --"
  $stdoutLog = Join-Path $artifactsDir "smoke-stdout.log"
  $stderrLog = Join-Path $artifactsDir "smoke-stderr.log"
  $fileLog = Join-Path $artifactsDir "smoke-file.log"
  Remove-Item -Path $stdoutLog, $stderrLog, $fileLog -ErrorAction SilentlyContinue

  $env:SONAR_SMOKE_LOG_PATH = $fileLog
  try {
    $smokeProc = Start-Process -FilePath $sonarExe -ArgumentList "--sonar-smoke-test" `
      -RedirectStandardOutput $stdoutLog -RedirectStandardError $stderrLog -Wait -PassThru
  }
  finally {
    Remove-Item Env:\SONAR_SMOKE_LOG_PATH -ErrorAction SilentlyContinue
  }

  $stdoutContent = if (Test-Path -LiteralPath $stdoutLog) { Get-Content -Raw -LiteralPath $stdoutLog } else { "" }
  $fileContent = if (Test-Path -LiteralPath $fileLog) { Get-Content -Raw -LiteralPath $fileLog } else { "" }
  $combined = "$stdoutContent`n$fileContent"

  Add-Summary "Code de sortie du smoke test : $($smokeProc.ExitCode)"
  if ($stdoutContent) {
    Add-Summary "--- stdout ---"
    Add-Summary $stdoutContent
  }

  if ($smokeProc.ExitCode -ne 0) {
    throw "Le smoke test s'est terminé avec le code $($smokeProc.ExitCode)"
  }
  if ($combined -notmatch "SONAR_STARTUP_VALIDATION=OK") {
    throw "Marqueur SONAR_STARTUP_VALIDATION=OK absent des logs du smoke test."
  }
  # startup_smoke.rs retourne toujours exit 0 / VALIDATION=OK même si aucune
  # interface capturable n'est trouvée : la vraie preuve de détection Npcap
  # est la présence de SONAR_SMOKE_DEVICE=, pas seulement le code de sortie.
  if ($combined -match "SONAR_SMOKE_DEVICE_UNAVAILABLE" -or $combined -match "SONAR_STARTUP_VALIDATION=FAILED") {
    throw "Npcap n'est pas détecté correctement sur ce runner (mode 'WinPcap API-compatible' probablement manquant, ou service Npcap absent) -- ceci est un problème de provisioning du runner self-hosted, pas nécessairement un bug SONAR."
  }
  if ($combined -notmatch "SONAR_SMOKE_DEVICE=") {
    throw "Marqueur SONAR_SMOKE_DEVICE=<interface> absent des logs du smoke test."
  }

  Add-Summary "Npcap correctement détecté : SONAR a résolu une interface de capture."
  $exitCode = 0
}
catch {
  Add-Summary "ERREUR : $($_.Exception.Message)"
}
finally {
  Add-Summary "-- Désinstallation silencieuse (nettoyage) --"
  Uninstall-Sonar

  $stillInstalled = Get-SonarExePath
  if ($stillInstalled) {
    Add-Summary "AVERTISSEMENT : sonar.exe toujours présent après désinstallation : $stillInstalled"
  }

  $summaryPath = Join-Path $artifactsDir "summary.txt"
  $summaryLines | Set-Content -LiteralPath $summaryPath -Encoding utf8
}

if ($exitCode -eq 0) {
  Write-Host "PASS : installation + détection Npcap validées."
} else {
  Write-Host "FAIL : voir $artifactsDir\summary.txt"
}
exit $exitCode
