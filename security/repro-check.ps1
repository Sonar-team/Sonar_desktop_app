# Portage Windows de security/repro-check.sh : démontre qu'un build natif
# Windows de SONAR est reproductible. Deux checkouts vierges du même commit
# (chacun son target\ et ses node_modules), deux builds via repro-env.ts
# (SOURCE_DATE_EPOCH figée, /Brepro MSVC), comparaison des SHA-256 des
# sonar.exe. Code retour 0 uniquement si les hashes sont identiques.
#
# Prérequis (versions épinglées dans config\build-versions.env) :
#   - toolchain Rust MSVC (rustup), VS Build Tools + SDK Windows
#   - Deno, Git
# Le SDK Npcap est versionné dans le dépôt : rien d'autre à installer.
#
# Usage :
#   pwsh -File security\repro-check.ps1
#   pwsh -File security\repro-check.ps1 -Runs 3 -KeepCheckouts

[CmdletBinding()]
param(
    [int]$Runs = 2,
    [string]$Workdir = (Join-Path ([System.IO.Path]::GetTempPath()) 'repro-check-sonar'),
    [string]$FixedEpoch = '1700000000',
    [switch]$KeepCheckouts
)

$ErrorActionPreference = 'Stop'

function Write-Log([string]$Message) {
    Write-Host ("[{0}] {1}" -f (Get-Date -Format 'HH:mm:ss'), $Message)
}

function Assert-Command([string]$Name) {
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Commande requise introuvable : $Name"
    }
}

function Read-BuildVersions([string]$Root) {
    $versions = @{}
    Get-Content (Join-Path $Root 'config/build-versions.env') | ForEach-Object {
        if ($_ -match '^([A-Z_]+)=(.*)$') {
            $versions[$Matches[1]] = $Matches[2].Trim('"')
        }
    }
    return $versions
}

$projectRoot = (git rev-parse --show-toplevel).Trim()
if (-not $projectRoot) { throw 'Ce script doit être lancé depuis le dépôt SONAR.' }

Assert-Command git
Assert-Command deno
Assert-Command rustc
Assert-Command cargo

# Préflight : l'environnement doit correspondre aux versions épinglées —
# une divergence d'outillage rend la comparaison de hashes sans valeur.
$versions = Read-BuildVersions $projectRoot
$rustc = (rustc --version).Split(' ')[1]
if ($rustc -ne $versions['RUST_VERSION']) {
    throw "rustc $rustc ≠ version épinglée $($versions['RUST_VERSION']) (rustup toolchain install $($versions['RUST_VERSION']))"
}
$deno = (deno --version | Select-Object -First 1).Split(' ')[1]
if ($deno -ne $versions['DENO_VERSION']) {
    throw "deno $deno ≠ version épinglée $($versions['DENO_VERSION'])"
}

$commit = (git -C $projectRoot rev-parse HEAD).Trim()
Write-Log "Projet : $projectRoot"
Write-Log "Commit : $commit"
Write-Log "Runs : $Runs — workspace : $Workdir"

if (Test-Path $Workdir) { Remove-Item -Recurse -Force $Workdir }
New-Item -ItemType Directory -Path $Workdir | Out-Null

$binRel = 'src-tauri/target/release/sonar.exe'
$hashes = @()

for ($i = 1; $i -le $Runs; $i++) {
    $checkout = Join-Path $Workdir "checkout-run$i"
    Write-Log "=== Run $i/$Runs : checkout isolé + build ==="

    git clone --quiet --local --no-hardlinks $projectRoot $checkout
    if ($LASTEXITCODE -ne 0) { throw "git clone a échoué (run $i)" }

    Push-Location $checkout
    try {
        deno install --frozen
        if ($LASTEXITCODE -ne 0) { throw "deno install a échoué (run $i)" }

        $env:SOURCE_DATE_EPOCH = $FixedEpoch
        deno run -A ./security/repro-env.ts run deno task tauri build --ci --no-sign --no-bundle
        if ($LASTEXITCODE -ne 0) { throw "build tauri a échoué (run $i)" }
    }
    finally {
        Pop-Location
        Remove-Item Env:\SOURCE_DATE_EPOCH -ErrorAction SilentlyContinue
    }

    $bin = Join-Path $checkout $binRel
    if (-not (Test-Path $bin)) { throw "Binaire introuvable : $bin" }

    $outdir = Join-Path $Workdir "run$i"
    New-Item -ItemType Directory -Path $outdir | Out-Null
    Copy-Item $bin (Join-Path $outdir 'sonar.exe')

    $hash = (Get-FileHash -Algorithm SHA256 (Join-Path $outdir 'sonar.exe')).Hash.ToLowerInvariant()
    $hashes += $hash
    Write-Log "sonar.exe run$i : $hash"

    # Le checkout isolé est volumineux (target\) : nettoyé une fois le
    # binaire archivé.
    if (-not $KeepCheckouts) { Remove-Item -Recurse -Force $checkout }
}

Write-Host ''
Write-Host '=== Comparaison ==='
$status = 0
for ($i = 1; $i -lt $hashes.Count; $i++) {
    if ($hashes[$i] -ne $hashes[0]) {
        Write-Error "ÉCHEC : le binaire du run$($i + 1) diffère du run1" -ErrorAction Continue
        $status = 1
    }
}

if ($status -eq 0) {
    Write-Host "OK : $Runs builds natifs isolés, binaire identique ($($hashes[0]))"
} else {
    Write-Host 'Les hashes diffèrent — build natif Windows non reproductible sur cette machine.'
}
exit $status
