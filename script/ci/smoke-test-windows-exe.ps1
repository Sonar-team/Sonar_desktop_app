param(
  [Parameter(Mandatory = $true)]
  [string] $ExePath,

  [int] $TimeoutSeconds = 20
)

$ErrorActionPreference = "Stop"

$resolvedExe = (Resolve-Path -LiteralPath $ExePath).Path
$workDir = Split-Path -Parent $resolvedExe

Write-Host "Smoke launching Windows executable: $resolvedExe"

$process = Start-Process `
  -FilePath $resolvedExe `
  -WorkingDirectory $workDir `
  -PassThru

$windowObserved = $false
$deadline = (Get-Date).AddSeconds($TimeoutSeconds)

try {
  while ((Get-Date) -lt $deadline) {
    Start-Sleep -Milliseconds 500
    $process.Refresh()

    if ($process.HasExited) {
      throw "Executable exited before creating a main window. Exit code: $($process.ExitCode)"
    }

    if ($process.MainWindowHandle -ne [IntPtr]::Zero) {
      $windowObserved = $true
      Write-Host "Observed main window handle $($process.MainWindowHandle) with title '$($process.MainWindowTitle)'."
      break
    }
  }

  if (-not $windowObserved) {
    Write-Warning "Executable stayed alive for $TimeoutSeconds seconds, but no main window handle was observed on this runner."
  }
} finally {
  if ($null -ne $process) {
    $process.Refresh()
    if (-not $process.HasExited) {
      Write-Host "Stopping smoke test process $($process.Id)."
      Stop-Process -Id $process.Id -Force
      Wait-Process -Id $process.Id -Timeout 10 -ErrorAction SilentlyContinue
    }
  }
}
