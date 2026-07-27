#Requires -Version 7.2

<#
.SYNOPSIS
Construit la version Windows NSIS officielle de SONAR et produit ses preuves.

.DESCRIPTION
Ce script est destine au poste Windows securise de release. Il :

1. verifie le tag, le commit, la proprete Git et les versions d'outils ;
2. execute les controles applicatifs puis construit uniquement l'installateur NSIS ;
3. fait signer par Tauri l'executable et l'installateur avec SignTool et le
   certificat Authenticode expose par le HSM dans le magasin Windows ;
4. verifie les signatures Authenticode et leur horodatage ;
5. genere une provenance SLSA v1, la signe avec Cosign via un HSM/KMS ou une
   cle de securite, puis verifie immediatement la preuve avec la cle publique ;
6. cree un kit autonome contenant l'installateur, les preuves et les empreintes.

La cle privee Authenticode et la cle Cosign ne sont jamais exportees. Le script
ne pretend pas rendre le build reproductible : il atteste quel code et quels
outils ont produit l'artefact effectivement livre.

Le poste peut etre deconnecte d'Internet si les dependances Deno/Cargo/NSIS sont
deja presentes et si les services HSM/KMS et TSA requis restent accessibles.

.PARAMETER ReleaseTag
Tag Git de release au format vX.Y.Z. La version doit correspondre a package.json
et src-tauri/tauri.conf.json.

.PARAMETER ExpectedCommit
SHA-1 Git complet (40 caracteres) approuve pour la release.

.PARAMETER AuthenticodeThumbprint
Empreinte SHA-1 du certificat de signature de code installe dans le magasin
Windows et relie a la cle privee du HSM.

.PARAMETER CertificateStore
Magasin contenant le certificat : CurrentUser (defaut) ou LocalMachine.

.PARAMETER TimestampUrl
URL du serveur d'horodatage Authenticode. RFC 3161 est utilise par defaut.

.PARAMETER SignToolCsp
Nom du CSP/KSP du HSM. Facultatif ; a fournir avec SignToolKeyContainer lorsque
le middleware HSM ne relie pas directement la cle privee au certificat.

.PARAMETER SignToolKeyContainer
Nom du conteneur de cle du HSM. Facultatif ; a fournir avec SignToolCsp.

.PARAMETER CosignKeyReference
Reference HSM/KMS acceptee par Cosign, par exemple azurekms://, awskms://,
gcpkms://, hashivault:// ou pkcs11:. Les chemins de cles locales sont refuses.

.PARAMETER CosignSecurityKey
Utilise le mode --sk de Cosign pour une cle materielle compatible PIV. Ce mode
forme un jeu de parametres alternatif a CosignKeyReference.

.PARAMETER CosignPublicKey
Cle publique PEM correspondant a la cle Cosign du HSM. Elle est copiee dans le
kit et utilisee pour verifier la signature et la provenance avant publication.

.PARAMETER OutputDirectory
Repertoire absolu, hors du depot, dans lequel creer le kit. Il doit etre vide ou
inexistant. Un kit partiel est conserve en cas d'echec pour faciliter l'audit.

.PARAMETER AllowUnsignedTag
Mode derogatoire : autorise un tag Git non signe. Sans ce commutateur, la
signature du tag doit etre validee par `git tag -v` sur le poste de release.

.EXAMPLE
./script/release/win_officiel_builder_hsm_prov.ps1 `
  -ReleaseTag v4.8.3 `
  -ExpectedCommit 0123456789abcdef0123456789abcdef01234567 `
  -AuthenticodeThumbprint 89ABCDEF0123456789ABCDEF0123456789ABCDEF `
  -CertificateStore LocalMachine `
  -TimestampUrl https://tsa.entreprise.example/rfc3161 `
  -CosignKeyReference 'azurekms://coffre.example.net/cles/sonar-provenance' `
  -CosignPublicKey C:\SONAR-TRUST\cosign-sonar.pub `
  -OutputDirectory C:\SONAR-RELEASES\v4.8.3

.NOTES
Prerequis : PowerShell 7.2+, Git, Node, Deno, Rust/Cargo, Tauri CLI, Cosign,
SignTool, 7-Zip, middleware HSM et acces au serveur TSA. Les versions attendues
sont celles de config/build-versions.env.

.LINK
script/release/WIN_OFFICIEL_BUILDER_HSM_PROV.md
#>

[CmdletBinding(DefaultParameterSetName = "HsmReference")]
param(
  [Parameter(Mandatory = $true)]
  [ValidatePattern('^v[0-9]+\.[0-9]+\.[0-9]+$')]
  [string] $ReleaseTag,

  [Parameter(Mandatory = $true)]
  [ValidatePattern('^[0-9a-fA-F]{40}$')]
  [string] $ExpectedCommit,

  [Parameter(Mandatory = $true)]
  [ValidatePattern('^[0-9a-fA-F]{40}$')]
  [string] $AuthenticodeThumbprint,

  [ValidateSet("CurrentUser", "LocalMachine")]
  [string] $CertificateStore = "CurrentUser",

  [Parameter(Mandatory = $true)]
  [uri] $TimestampUrl,

  [switch] $UseLegacyAuthenticodeTimestamp,

  [string] $SignToolCsp,

  [string] $SignToolKeyContainer,

  [Parameter(Mandatory = $true, ParameterSetName = "HsmReference")]
  [string] $CosignKeyReference,

  [Parameter(Mandatory = $true, ParameterSetName = "SecurityKey")]
  [switch] $CosignSecurityKey,

  [Parameter(Mandatory = $true)]
  [string] $CosignPublicKey,

  [uri] $CosignTimestampUrl,

  [string] $CosignTimestampCertificateChain,

  [Parameter(Mandatory = $true)]
  [string] $OutputDirectory,

  [string] $BuilderId = "urn:sonar:builder:windows-hsm:v1",

  [string] $SourceRepositoryUri,

  [switch] $AllowUnsignedTag
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

function Invoke-External {
  param(
    [Parameter(Mandatory = $true)] [string] $FilePath,
    [string[]] $ArgumentList = @(),
    [Parameter(Mandatory = $true)] [string] $WorkingDirectory,
    [Parameter(Mandatory = $true)] [string] $Description
  )

  Write-Host "==> $Description"
  Push-Location -LiteralPath $WorkingDirectory
  try {
    & $FilePath @ArgumentList
    if ($LASTEXITCODE -ne 0) {
      throw "$Description a echoue avec le code $LASTEXITCODE."
    }
  }
  finally {
    Pop-Location
  }
}

function Get-ExternalOutput {
  param(
    [Parameter(Mandatory = $true)] [string] $FilePath,
    [string[]] $ArgumentList = @(),
    [Parameter(Mandatory = $true)] [string] $WorkingDirectory,
    [Parameter(Mandatory = $true)] [string] $Description
  )

  Push-Location -LiteralPath $WorkingDirectory
  try {
    $lines = @(& $FilePath @ArgumentList 2>&1)
    $exitCode = $LASTEXITCODE
    $output = ($lines | ForEach-Object { $_.ToString() }) -join "`n"
    if ($exitCode -ne 0) {
      throw "$Description a echoue avec le code ${exitCode}:`n$output"
    }
    return $output.Trim()
  }
  finally {
    Pop-Location
  }
}

function Get-RequiredCommandPath {
  param([Parameter(Mandatory = $true)] [string] $Name)

  $command = Get-Command $Name -ErrorAction SilentlyContinue
  if (-not $command) {
    throw "Commande requise introuvable : $Name"
  }
  return $command.Source
}

function Get-SignToolPath {
  $command = Get-Command "signtool.exe" -ErrorAction SilentlyContinue
  if ($command) {
    return $command.Source
  }

  $programFilesX86 = ${env:ProgramFiles(x86)}
  if ($programFilesX86) {
    $kitsBin = Join-Path $programFilesX86 "Windows Kits\10\bin"
    if (Test-Path -LiteralPath $kitsBin) {
      $candidate = Get-ChildItem -LiteralPath $kitsBin -Filter "signtool.exe" -File -Recurse |
        Where-Object { $_.FullName -match '[\\/]x64[\\/]signtool\.exe$' } |
        Sort-Object FullName -Descending |
        Select-Object -First 1
      if ($candidate) {
        return $candidate.FullName
      }
    }
  }

  throw "signtool.exe x64 est introuvable. Installez le Windows SDK."
}

function Read-BuildVersions {
  param([Parameter(Mandatory = $true)] [string] $Path)

  $versions = @{}
  foreach ($line in Get-Content -LiteralPath $Path) {
    if ($line -match '^([A-Z0-9_]+)=(.*)$') {
      $value = $Matches[2].Trim()
      if ($value.Length -ge 2 -and $value.StartsWith('"') -and $value.EndsWith('"')) {
        $value = $value.Substring(1, $value.Length - 2)
      }
      $versions[$Matches[1]] = $value
    }
  }
  return $versions
}

function Get-VersionMatch {
  param(
    [Parameter(Mandatory = $true)] [string] $Text,
    [Parameter(Mandatory = $true)] [string] $Pattern,
    [Parameter(Mandatory = $true)] [string] $ToolName
  )

  $match = [regex]::Match($Text, $Pattern, [Text.RegularExpressions.RegexOptions]::IgnoreCase)
  if (-not $match.Success) {
    throw "Impossible de lire la version de $ToolName dans : $Text"
  }
  return $match.Groups[1].Value
}

function Assert-Version {
  param(
    [Parameter(Mandatory = $true)] [string] $Name,
    [Parameter(Mandatory = $true)] [string] $Actual,
    [Parameter(Mandatory = $true)] [string] $Expected
  )

  if ($Actual -ne $Expected) {
    throw "Version $Name incorrecte : attendue $Expected, trouvee $Actual."
  }
  Write-Host "Version $Name validee : $Actual"
}

function Get-Sha256 {
  param([Parameter(Mandatory = $true)] [string] $Path)
  return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Assert-AuthenticodeSignature {
  param(
    [Parameter(Mandatory = $true)] [string] $Path,
    [Parameter(Mandatory = $true)] [string] $ExpectedThumbprint,
    [Parameter(Mandatory = $true)] [string] $SignToolPath,
    [Parameter(Mandatory = $true)] [string] $WorkingDirectory
  )

  Invoke-External -FilePath $SignToolPath `
    -ArgumentList @("verify", "/pa", "/all", "/v", "/tw", $Path) `
    -WorkingDirectory $WorkingDirectory `
    -Description "Verification SignTool de $(Split-Path -Leaf $Path)"

  $signature = Get-AuthenticodeSignature -LiteralPath $Path
  if ($signature.Status -ne [System.Management.Automation.SignatureStatus]::Valid) {
    throw "Signature Authenticode invalide pour ${Path}: $($signature.StatusMessage)"
  }
  if (-not $signature.SignerCertificate) {
    throw "Aucun certificat signataire trouve pour $Path"
  }
  if ($signature.SignerCertificate.Thumbprint.ToUpperInvariant() -ne $ExpectedThumbprint) {
    throw "Le certificat signataire de $Path ne correspond pas a l'empreinte approuvee."
  }
  if (-not $signature.TimeStamperCertificate) {
    throw "La signature de $Path ne contient pas d'horodatage."
  }
}

if (-not $IsWindows) {
  throw "Ce script doit etre execute sur le poste Windows officiel de build."
}

if ($TimestampUrl.Scheme -notin @("http", "https")) {
  throw "TimestampUrl doit etre une URL HTTP(S)."
}
if (($SignToolCsp -and -not $SignToolKeyContainer) -or
    ($SignToolKeyContainer -and -not $SignToolCsp)) {
  throw "SignToolCsp et SignToolKeyContainer doivent etre fournis ensemble."
}
if ($PSCmdlet.ParameterSetName -eq "HsmReference" -and
    $CosignKeyReference -notmatch '^(?:[a-z][a-z0-9+.-]*://|pkcs11:)') {
  throw "CosignKeyReference doit etre une reference HSM/KMS, pas un chemin de cle locale."
}
if (($CosignTimestampUrl -and -not $CosignTimestampCertificateChain) -or
    ($CosignTimestampCertificateChain -and -not $CosignTimestampUrl)) {
  throw "CosignTimestampUrl et CosignTimestampCertificateChain doivent etre fournis ensemble."
}
if ($CosignTimestampUrl -and $CosignTimestampUrl.Scheme -notin @("http", "https")) {
  throw "CosignTimestampUrl doit etre une URL HTTP(S)."
}

$scriptPath = $MyInvocation.MyCommand.Path
$repositoryRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot "..\..")).Path
$tauriDirectory = Join-Path $repositoryRoot "src-tauri"
$buildVersionsPath = Join-Path $repositoryRoot "config\build-versions.env"
$cosignPublicKeyPath = (Resolve-Path -LiteralPath $CosignPublicKey).Path
$cosignTimestampChainPath = if ($CosignTimestampCertificateChain) {
  (Resolve-Path -LiteralPath $CosignTimestampCertificateChain).Path
} else {
  $null
}

if (-not [IO.Path]::IsPathFullyQualified($OutputDirectory)) {
  throw "OutputDirectory doit etre un chemin absolu."
}
$outputPath = [IO.Path]::GetFullPath($OutputDirectory)
$repositoryPrefix = $repositoryRoot.TrimEnd([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar) +
  [IO.Path]::DirectorySeparatorChar
if ($outputPath.Equals($repositoryRoot, [StringComparison]::OrdinalIgnoreCase) -or
    $outputPath.StartsWith($repositoryPrefix, [StringComparison]::OrdinalIgnoreCase)) {
  throw "OutputDirectory doit etre situe hors du depot source."
}
if (Test-Path -LiteralPath $outputPath) {
  if (Get-ChildItem -LiteralPath $outputPath -Force | Select-Object -First 1) {
    throw "OutputDirectory existe deja et n'est pas vide : $outputPath"
  }
} else {
  New-Item -ItemType Directory -Path $outputPath -Force | Out-Null
}

$artifactDirectory = New-Item -ItemType Directory -Path (Join-Path $outputPath "artifacts")
$provenanceDirectory = New-Item -ItemType Directory -Path (Join-Path $outputPath "provenance")
$signatureDirectory = New-Item -ItemType Directory -Path (Join-Path $outputPath "signatures")
$trustDirectory = New-Item -ItemType Directory -Path (Join-Path $outputPath "trust")
$logDirectory = New-Item -ItemType Directory -Path (Join-Path $outputPath "logs")
$transcriptPath = Join-Path $logDirectory "build.log"
$workDirectory = Join-Path ([IO.Path]::GetTempPath()) ("sonar-win-officiel-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $workDirectory | Out-Null

$previousCargoTargetDirectory = $env:CARGO_TARGET_DIR
$previousCi = $env:CI
$transcriptStarted = $false
$buildSucceeded = $false

try {
  Start-Transcript -LiteralPath $transcriptPath | Out-Null
  $transcriptStarted = $true

  $git = Get-RequiredCommandPath "git"
  $deno = Get-RequiredCommandPath "deno"
  $node = Get-RequiredCommandPath "node"
  $rustc = Get-RequiredCommandPath "rustc"
  $cargo = Get-RequiredCommandPath "cargo"
  $cosign = Get-RequiredCommandPath "cosign"
  $signTool = Get-SignToolPath
  $pwsh = Get-RequiredCommandPath "pwsh"

  $expectedCommitNormalized = $ExpectedCommit.ToLowerInvariant()
  $authenticodeThumbprintNormalized = $AuthenticodeThumbprint.ToUpperInvariant()
  $headCommit = (Get-ExternalOutput -FilePath $git -ArgumentList @("rev-parse", "HEAD") `
    -WorkingDirectory $repositoryRoot -Description "Lecture du commit HEAD").ToLowerInvariant()
  if ($headCommit -ne $expectedCommitNormalized) {
    throw "HEAD ($headCommit) ne correspond pas au commit approuve ($expectedCommitNormalized)."
  }

  $tagCommit = (Get-ExternalOutput -FilePath $git -ArgumentList @("rev-parse", "$ReleaseTag^{commit}") `
    -WorkingDirectory $repositoryRoot -Description "Resolution du tag $ReleaseTag").ToLowerInvariant()
  if ($tagCommit -ne $expectedCommitNormalized) {
    throw "Le tag $ReleaseTag pointe vers $tagCommit au lieu de $expectedCommitNormalized."
  }
  if (-not $AllowUnsignedTag) {
    [void](Get-ExternalOutput -FilePath $git -ArgumentList @("tag", "-v", $ReleaseTag) `
      -WorkingDirectory $repositoryRoot -Description "Verification cryptographique du tag $ReleaseTag")
  } else {
    Write-Warning "DEROGATION : la signature du tag Git n'est pas exigee."
  }

  $gitStatus = Get-ExternalOutput -FilePath $git `
    -ArgumentList @("status", "--porcelain=v1", "--untracked-files=all") `
    -WorkingDirectory $repositoryRoot -Description "Verification de la proprete Git"
  if ($gitStatus) {
    throw "Le depot doit etre strictement propre avant un build officiel :`n$gitStatus"
  }

  $releaseVersion = $ReleaseTag.Substring(1)
  $packageConfig = Get-Content -LiteralPath (Join-Path $repositoryRoot "package.json") -Raw | ConvertFrom-Json
  $tauriConfig = Get-Content -LiteralPath (Join-Path $tauriDirectory "tauri.conf.json") -Raw | ConvertFrom-Json
  if ($packageConfig.version -ne $releaseVersion -or $tauriConfig.version -ne $releaseVersion) {
    throw "Le tag $ReleaseTag, package.json ($($packageConfig.version)) et tauri.conf.json ($($tauriConfig.version)) doivent avoir la meme version."
  }

  if (-not $SourceRepositoryUri) {
    $remoteNames = @(Get-ExternalOutput -FilePath $git -ArgumentList @("remote") `
      -WorkingDirectory $repositoryRoot -Description "Lecture des remotes Git" -ErrorAction Stop) -split "`n"
    $remoteName = if ($remoteNames -contains "origin") { "origin" } else { $remoteNames[0].Trim() }
    if (-not $remoteName) {
      throw "Aucun remote Git disponible ; fournissez SourceRepositoryUri."
    }
    $SourceRepositoryUri = Get-ExternalOutput -FilePath $git -ArgumentList @("remote", "get-url", $remoteName) `
      -WorkingDirectory $repositoryRoot -Description "Lecture de l'URL source"
  }

  $versions = Read-BuildVersions -Path $buildVersionsPath
  foreach ($requiredVersion in @("RUST_VERSION", "NODE_VERSION", "DENO_VERSION", "TAURI_CLI_VERSION", "COSIGN_VERSION")) {
    if (-not $versions.ContainsKey($requiredVersion)) {
      throw "Version canonique absente de config/build-versions.env : $requiredVersion"
    }
  }

  $denoOutput = Get-ExternalOutput -FilePath $deno -ArgumentList @("--version") `
    -WorkingDirectory $repositoryRoot -Description "Lecture de la version Deno"
  $nodeOutput = Get-ExternalOutput -FilePath $node -ArgumentList @("--version") `
    -WorkingDirectory $repositoryRoot -Description "Lecture de la version Node"
  $rustOutput = Get-ExternalOutput -FilePath $rustc -ArgumentList @("--version") `
    -WorkingDirectory $repositoryRoot -Description "Lecture de la version Rust"
  $tauriOutput = Get-ExternalOutput -FilePath $deno -ArgumentList @("task", "tauri", "--version") `
    -WorkingDirectory $repositoryRoot -Description "Lecture de la version Tauri CLI"
  $cosignOutput = Get-ExternalOutput -FilePath $cosign -ArgumentList @("version") `
    -WorkingDirectory $repositoryRoot -Description "Lecture de la version Cosign"

  $actualVersions = [ordered]@{
    deno = Get-VersionMatch -Text $denoOutput -Pattern '(?m)^deno\s+v?([0-9]+\.[0-9]+\.[0-9]+)' -ToolName "Deno"
    node = Get-VersionMatch -Text $nodeOutput -Pattern '^v?([0-9]+\.[0-9]+\.[0-9]+)' -ToolName "Node"
    rust = Get-VersionMatch -Text $rustOutput -Pattern '^rustc\s+([0-9]+\.[0-9]+\.[0-9]+)' -ToolName "Rust"
    tauri = Get-VersionMatch -Text $tauriOutput -Pattern '(?m)(?:tauri-cli|tauri)\s+v?([0-9]+\.[0-9]+\.[0-9]+)' -ToolName "Tauri CLI"
    cosign = Get-VersionMatch -Text $cosignOutput -Pattern '(?m)GitVersion:\s*v?([0-9]+\.[0-9]+\.[0-9]+)' -ToolName "Cosign"
  }
  Assert-Version -Name "Deno" -Actual $actualVersions.deno -Expected $versions.DENO_VERSION
  Assert-Version -Name "Node" -Actual $actualVersions.node -Expected $versions.NODE_VERSION
  Assert-Version -Name "Rust" -Actual $actualVersions.rust -Expected $versions.RUST_VERSION
  Assert-Version -Name "Tauri CLI" -Actual $actualVersions.tauri -Expected $versions.TAURI_CLI_VERSION
  Assert-Version -Name "Cosign" -Actual $actualVersions.cosign -Expected $versions.COSIGN_VERSION

  $certificatePath = "Cert:\$CertificateStore\My\$authenticodeThumbprintNormalized"
  if (-not (Test-Path -LiteralPath $certificatePath)) {
    throw "Certificat Authenticode introuvable : $certificatePath"
  }
  $certificate = Get-Item -LiteralPath $certificatePath
  $now = Get-Date
  if ($now -lt $certificate.NotBefore -or $now -gt $certificate.NotAfter) {
    throw "Le certificat Authenticode n'est pas valide a la date du build."
  }
  if (-not $certificate.HasPrivateKey -and -not $SignToolCsp) {
    throw "Le certificat n'expose aucune cle privee. Verifiez le middleware HSM ou fournissez SignToolCsp et SignToolKeyContainer."
  }
  $ekuOids = @($certificate.EnhancedKeyUsageList | ForEach-Object { $_.ObjectId.Value })
  if ($ekuOids.Count -gt 0 -and $ekuOids -notcontains "1.3.6.1.5.5.7.3.3") {
    throw "Le certificat selectionne n'autorise pas la signature de code."
  }
  Write-Host "Certificat Authenticode valide : $($certificate.Subject)"

  $invocationId = [guid]::NewGuid().ToString()
  $cosignSignerArguments = if ($PSCmdlet.ParameterSetName -eq "SecurityKey") {
    @("--sk")
  } else {
    @("--key", $CosignKeyReference)
  }
  $cosignTimestampArguments = if ($CosignTimestampUrl) {
    @("--timestamp-server-url", $CosignTimestampUrl.AbsoluteUri)
  } else {
    @()
  }
  $preflightTimestampVerificationArguments = if ($cosignTimestampChainPath) {
    @("--use-signed-timestamps", "--timestamp-certificate-chain", $cosignTimestampChainPath)
  } else {
    @()
  }
  $cosignPreflightPath = Join-Path $workDirectory "cosign-hsm-preflight.txt"
  $cosignPreflightBundle = Join-Path $workDirectory "cosign-hsm-preflight.sigstore.json"
  "SONAR official builder preflight $invocationId $expectedCommitNormalized" |
    Set-Content -LiteralPath $cosignPreflightPath -Encoding utf8NoBOM
  Invoke-External -FilePath $cosign `
    -ArgumentList (@("sign-blob", "--yes", "--use-signing-config=false", "--new-bundle-format=true", "--bundle", $cosignPreflightBundle) +
      $cosignSignerArguments + $cosignTimestampArguments + @($cosignPreflightPath)) `
    -WorkingDirectory $repositoryRoot -Description "Preflight de la cle Cosign HSM"
  Invoke-External -FilePath $cosign `
    -ArgumentList (@("verify-blob", "--private-infrastructure", "--key", $cosignPublicKeyPath, "--bundle", $cosignPreflightBundle) +
      $preflightTimestampVerificationArguments + @($cosignPreflightPath)) `
    -WorkingDirectory $repositoryRoot -Description "Verification de la cle publique Cosign"

  $targetDirectory = Join-Path $workDirectory "target"
  $overlayPath = Join-Path $workDirectory "tauri.windows.official.json"
  $signArguments = @("sign", "/v", "/sha1", $authenticodeThumbprintNormalized, "/fd", "SHA256")
  if ($CertificateStore -eq "LocalMachine") {
    $signArguments += "/sm"
  }
  if ($SignToolCsp) {
    $signArguments += @("/csp", $SignToolCsp, "/kc", $SignToolKeyContainer)
  }
  if ($UseLegacyAuthenticodeTimestamp) {
    $signArguments += @("/t", $TimestampUrl.AbsoluteUri)
  } else {
    $signArguments += @("/tr", $TimestampUrl.AbsoluteUri, "/td", "SHA256")
  }
  $signArguments += "%1"

  $overlay = [ordered]@{
    bundle = [ordered]@{
      targets = @("nsis")
      windows = [ordered]@{
        signCommand = [ordered]@{
          cmd = $signTool
          args = $signArguments
        }
      }
    }
  }
  $overlay | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $overlayPath -Encoding utf8NoBOM

  $env:CARGO_TARGET_DIR = $targetDirectory
  $env:CI = "true"
  $buildStartedOn = (Get-Date).ToUniversalTime()

  Invoke-External -FilePath $deno -ArgumentList @("install", "--frozen") `
    -WorkingDirectory $repositoryRoot -Description "Installation verrouillee des dependances Deno"
  Invoke-External -FilePath $deno -ArgumentList @("task", "typecheck") `
    -WorkingDirectory $repositoryRoot -Description "Typecheck frontend"
  Invoke-External -FilePath $deno -ArgumentList @("task", "lint") `
    -WorkingDirectory $repositoryRoot -Description "Lint frontend"
  Invoke-External -FilePath $deno -ArgumentList @("task", "test") `
    -WorkingDirectory $repositoryRoot -Description "Tests unitaires frontend"
  Invoke-External -FilePath $cargo -ArgumentList @("test", "--locked") `
    -WorkingDirectory $tauriDirectory -Description "Tests Rust"
  Invoke-External -FilePath $deno -ArgumentList @("task", "tauri", "build", "--ci", "--config", $overlayPath) `
    -WorkingDirectory $repositoryRoot -Description "Build officiel Tauri NSIS signe"

  $postBuildHead = (Get-ExternalOutput -FilePath $git -ArgumentList @("rev-parse", "HEAD") `
    -WorkingDirectory $repositoryRoot -Description "Revalidation du commit apres build").ToLowerInvariant()
  $postBuildStatus = Get-ExternalOutput -FilePath $git `
    -ArgumentList @("status", "--porcelain=v1", "--untracked-files=all") `
    -WorkingDirectory $repositoryRoot -Description "Revalidation de la proprete Git apres build"
  if ($postBuildHead -ne $expectedCommitNormalized -or $postBuildStatus) {
    throw "Le build a modifie ou remplace la source approuvee ; aucune provenance ne sera signee.`n$postBuildStatus"
  }

  $validatePeScript = Join-Path $repositoryRoot "script\ci\validate-windows-release-binary.ps1"
  $validateBundleScript = Join-Path $repositoryRoot "script\ci\check-windows-bundles-no-npcap.ps1"
  $releaseBinary = Join-Path $targetDirectory "release\sonar.exe"
  if (-not (Test-Path -LiteralPath $releaseBinary -PathType Leaf)) {
    throw "Executable Windows attendu introuvable : $releaseBinary"
  }
  Invoke-External -FilePath $pwsh -ArgumentList @("-NoProfile", "-File", $validatePeScript, "-BinaryPath", $releaseBinary) `
    -WorkingDirectory $repositoryRoot -Description "Validation de la structure PE et des imports"
  Invoke-External -FilePath $pwsh -ArgumentList @("-NoProfile", "-File", $validateBundleScript, "-TargetDirectory", $targetDirectory) `
    -WorkingDirectory $repositoryRoot -Description "Validation du contenu NSIS"

  $installers = @(Get-ChildItem -LiteralPath $targetDirectory -Recurse -File -Filter "*setup.exe" |
    Where-Object { $_.FullName -match '[\\/]bundle[\\/]nsis[\\/]' })
  if ($installers.Count -ne 1) {
    throw "Un seul installateur NSIS etait attendu ; trouve : $($installers.Count)."
  }
  Assert-AuthenticodeSignature -Path $releaseBinary -ExpectedThumbprint $authenticodeThumbprintNormalized `
    -SignToolPath $signTool -WorkingDirectory $repositoryRoot
  Assert-AuthenticodeSignature -Path $installers[0].FullName -ExpectedThumbprint $authenticodeThumbprintNormalized `
    -SignToolPath $signTool -WorkingDirectory $repositoryRoot

  $artifactPath = Join-Path $artifactDirectory.FullName $installers[0].Name
  Copy-Item -LiteralPath $installers[0].FullName -Destination $artifactPath
  Assert-AuthenticodeSignature -Path $artifactPath -ExpectedThumbprint $authenticodeThumbprintNormalized `
    -SignToolPath $signTool -WorkingDirectory $repositoryRoot

  $artifactSha256 = Get-Sha256 -Path $artifactPath
  $buildFinishedOn = (Get-Date).ToUniversalTime()
  $predicatePath = Join-Path $provenanceDirectory.FullName "sonar-windows-nsis-provenance.slsa-v1.json"
  $predicate = [ordered]@{
    buildDefinition = [ordered]@{
      buildType = "urn:sonar:build:tauri-windows-nsis:v1"
      externalParameters = [ordered]@{
        source = $SourceRepositoryUri
        gitTag = $ReleaseTag
        expectedCommit = $expectedCommitNormalized
        platform = "windows-x86_64"
        bundle = "nsis"
      }
      internalParameters = [ordered]@{
        sourceTreeClean = $true
        tagSignatureRequired = (-not $AllowUnsignedTag.IsPresent)
        authenticodeCertificateThumbprint = $authenticodeThumbprintNormalized
        authenticodeCertificateStore = $CertificateStore
        authenticodeTimestampProtocol = if ($UseLegacyAuthenticodeTimestamp) { "Authenticode" } else { "RFC3161" }
        script = "script/release/win_officiel_builder_hsm_prov.ps1"
      }
      resolvedDependencies = @(
        [ordered]@{
          uri = $SourceRepositoryUri
          digest = [ordered]@{ gitCommit = $expectedCommitNormalized }
        },
        [ordered]@{
          uri = "$SourceRepositoryUri#deno.lock"
          digest = [ordered]@{ sha256 = Get-Sha256 -Path (Join-Path $repositoryRoot "deno.lock") }
        },
        [ordered]@{
          uri = "$SourceRepositoryUri#src-tauri/Cargo.lock"
          digest = [ordered]@{ sha256 = Get-Sha256 -Path (Join-Path $tauriDirectory "Cargo.lock") }
        },
        [ordered]@{
          uri = "$SourceRepositoryUri#config/build-versions.env"
          digest = [ordered]@{ sha256 = Get-Sha256 -Path $buildVersionsPath }
        },
        [ordered]@{
          uri = "$SourceRepositoryUri#script/release/win_officiel_builder_hsm_prov.ps1"
          digest = [ordered]@{ sha256 = Get-Sha256 -Path $scriptPath }
        }
      )
    }
    runDetails = [ordered]@{
      builder = [ordered]@{
        id = $BuilderId
        version = [ordered]@{
          deno = $actualVersions.deno
          node = $actualVersions.node
          rust = $actualVersions.rust
          tauriCli = $actualVersions.tauri
          cosign = $actualVersions.cosign
          powershell = $PSVersionTable.PSVersion.ToString()
          windows = [Environment]::OSVersion.VersionString
        }
      }
      metadata = [ordered]@{
        invocationId = "urn:uuid:$invocationId"
        startedOn = $buildStartedOn.ToString("o")
        finishedOn = $buildFinishedOn.ToString("o")
      }
    }
  }
  $predicate | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $predicatePath -Encoding utf8NoBOM

  $artifactSignatureBundle = Join-Path $signatureDirectory.FullName ($installers[0].Name + ".sigstore.json")
  $attestationBundle = Join-Path $provenanceDirectory.FullName ($installers[0].Name + ".provenance.sigstore.json")

  Invoke-External -FilePath $cosign `
    -ArgumentList (@("sign-blob", "--yes", "--use-signing-config=false", "--new-bundle-format=true", "--bundle", $artifactSignatureBundle) +
      $cosignSignerArguments + $cosignTimestampArguments + @($artifactPath)) `
    -WorkingDirectory $repositoryRoot -Description "Signature Cosign HSM de l'installateur"
  Invoke-External -FilePath $cosign `
    -ArgumentList (@("attest-blob", "--yes", "--use-signing-config=false", "--new-bundle-format=true", "--predicate", $predicatePath,
      "--type", "slsaprovenance1", "--bundle", $attestationBundle) + $cosignSignerArguments + $cosignTimestampArguments + @($artifactPath)) `
    -WorkingDirectory $repositoryRoot -Description "Attestation SLSA v1 Cosign HSM"

  $kitPublicKey = Join-Path $trustDirectory.FullName "cosign-sonar.pub"
  Copy-Item -LiteralPath $cosignPublicKeyPath -Destination $kitPublicKey
  $timestampVerificationArguments = @()
  if ($cosignTimestampChainPath) {
    $kitTimestampChain = Join-Path $trustDirectory.FullName "cosign-tsa-chain.pem"
    Copy-Item -LiteralPath $cosignTimestampChainPath -Destination $kitTimestampChain
    $timestampVerificationArguments = @("--use-signed-timestamps", "--timestamp-certificate-chain", $kitTimestampChain)
  }

  Invoke-External -FilePath $cosign `
    -ArgumentList (@("verify-blob", "--private-infrastructure", "--key", $kitPublicKey, "--bundle", $artifactSignatureBundle) +
      $timestampVerificationArguments + @($artifactPath)) `
    -WorkingDirectory $repositoryRoot -Description "Verification locale de la signature Cosign"
  Invoke-External -FilePath $cosign `
    -ArgumentList (@("verify-blob-attestation", "--private-infrastructure", "--key", $kitPublicKey, "--bundle", $attestationBundle,
      "--type", "slsaprovenance1") + $timestampVerificationArguments + @($artifactPath)) `
    -WorkingDirectory $repositoryRoot -Description "Verification locale de la provenance SLSA"

  $metadataPath = Join-Path $outputPath "metadata.json"
  $metadata = [ordered]@{
    schemaVersion = 1
    product = "SONAR"
    version = $releaseVersion
    artifact = [ordered]@{
      path = "artifacts/$($installers[0].Name)"
      sha256 = $artifactSha256
      format = "NSIS"
      architecture = "x86_64"
    }
    source = [ordered]@{
      repository = $SourceRepositoryUri
      tag = $ReleaseTag
      commit = $expectedCommitNormalized
      clean = $true
    }
    authenticode = [ordered]@{
      certificateThumbprint = $authenticodeThumbprintNormalized
      certificateSubject = $certificate.Subject
      certificateIssuer = $certificate.Issuer
      certificateSerialNumber = $certificate.SerialNumber
      certificateNotAfter = $certificate.NotAfter.ToUniversalTime().ToString("o")
      timestamped = $true
    }
    provenance = [ordered]@{
      predicateType = "https://slsa.dev/provenance/v1"
      builderId = $BuilderId
      signingMode = if ($PSCmdlet.ParameterSetName -eq "SecurityKey") { "cosign-security-key" } else { "cosign-hsm-kms" }
      publicKeySha256 = Get-Sha256 -Path $kitPublicKey
      privateInfrastructure = $true
    }
    invocationId = "urn:uuid:$invocationId"
  }
  $metadata | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $metadataPath -Encoding utf8NoBOM

  $verificationPath = Join-Path $outputPath "VERIFY.txt"
  $timestampVerificationText = if ($cosignTimestampChainPath) {
    ' --use-signed-timestamps --timestamp-certificate-chain "trust\cosign-tsa-chain.pem"'
  } else {
    ""
  }
  @"
Verification depuis la racine du kit :

1. Authentifier le manifeste avant de recalculer ses empreintes :
   cosign verify-blob --private-infrastructure --key "trust\cosign-sonar.pub" --bundle "signatures\SHA256SUMS.sigstore.json"$timestampVerificationText "SHA256SUMS"
2. Recalculer chaque empreinte listee dans SHA256SUMS.
3. Verifier Authenticode :
   signtool verify /pa /all /v /tw "artifacts\$($installers[0].Name)"
4. Verifier la signature Cosign de l'installateur :
   cosign verify-blob --private-infrastructure --key "trust\cosign-sonar.pub" --bundle "signatures\$($installers[0].Name).sigstore.json"$timestampVerificationText "artifacts\$($installers[0].Name)"
5. Verifier la provenance SLSA et le digest du sujet :
   cosign verify-blob-attestation --private-infrastructure --key "trust\cosign-sonar.pub" --bundle "provenance\$($installers[0].Name).provenance.sigstore.json" --type slsaprovenance1$timestampVerificationText "artifacts\$($installers[0].Name)"

La cle publique doit etre comparee a une copie de confiance diffusee par un canal separe.
"@ | Set-Content -LiteralPath $verificationPath -Encoding utf8NoBOM

  $hashManifestPath = Join-Path $outputPath "SHA256SUMS"
  $filesToHash = @(Get-ChildItem -LiteralPath $outputPath -Recurse -File |
    Where-Object { $_.FullName -ne $hashManifestPath -and $_.FullName -ne $transcriptPath } |
    Sort-Object FullName)
  $hashLines = foreach ($file in $filesToHash) {
    $relativePath = [IO.Path]::GetRelativePath($outputPath, $file.FullName).Replace('\', '/')
    "$(Get-Sha256 -Path $file.FullName)  $relativePath"
  }
  $hashLines | Set-Content -LiteralPath $hashManifestPath -Encoding utf8NoBOM

  $hashManifestSignatureBundle = Join-Path $signatureDirectory.FullName "SHA256SUMS.sigstore.json"
  Invoke-External -FilePath $cosign `
    -ArgumentList (@("sign-blob", "--yes", "--use-signing-config=false", "--new-bundle-format=true", "--bundle", $hashManifestSignatureBundle) +
      $cosignSignerArguments + $cosignTimestampArguments + @($hashManifestPath)) `
    -WorkingDirectory $repositoryRoot -Description "Signature Cosign HSM du manifeste SHA256"
  Invoke-External -FilePath $cosign `
    -ArgumentList (@("verify-blob", "--private-infrastructure", "--key", $kitPublicKey, "--bundle", $hashManifestSignatureBundle) +
      $timestampVerificationArguments + @($hashManifestPath)) `
    -WorkingDirectory $repositoryRoot -Description "Verification locale du manifeste SHA256"

  $buildSucceeded = $true
  Write-Host "Kit officiel cree et verifie : $outputPath" -ForegroundColor Green
  Write-Host "SHA-256 installateur : $artifactSha256"
}
finally {
  if ($null -eq $previousCargoTargetDirectory) {
    Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
  } else {
    $env:CARGO_TARGET_DIR = $previousCargoTargetDirectory
  }
  if ($null -eq $previousCi) {
    Remove-Item Env:CI -ErrorAction SilentlyContinue
  } else {
    $env:CI = $previousCi
  }
  if ($transcriptStarted) {
    Stop-Transcript | Out-Null
  }
  if (Test-Path -LiteralPath $workDirectory) {
    Remove-Item -LiteralPath $workDirectory -Recurse -Force
  }
  if (-not $buildSucceeded) {
    Write-Warning "Build officiel interrompu. Le kit partiel est conserve dans $outputPath."
  }
}
