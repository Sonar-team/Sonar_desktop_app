[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Archive,

    [Parameter(Mandatory = $true)]
    [string]$Bundle,

    [Parameter(Mandatory = $true)]
    [string]$TrustedRoot,

    [Parameter(Mandatory = $true)]
    [ValidatePattern('^v[0-9]+\.[0-9]+\.[0-9]+$')]
    [string]$ExpectedTag,

    [ValidatePattern('^$|^[0-9a-f]{40}$')]
    [string]$ExpectedCommit = '',

    [string]$Platform = 'windows-2022',

    [Parameter(Mandatory = $true)]
    [string]$ExtractTo,

    [string]$Cosign = 'cosign'
)

$ErrorActionPreference = 'Stop'
$ExpectedRepository = 'Sonar-team/Sonar_desktop_app'
$ExpectedOidcIssuer = 'https://token.actions.githubusercontent.com'

function Invoke-CosignVerification {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Artifact,

        [Parameter(Mandatory = $true)]
        [string]$SignatureBundle,

        [Parameter(Mandatory = $true)]
        [string]$Identity
    )

    $CosignArguments = @(
        'verify-blob',
        '--trusted-root', $TrustedRoot,
        '--certificate-identity', $Identity,
        '--certificate-oidc-issuer', $ExpectedOidcIssuer
    )
    if ($ExpectedCommit) {
        $CosignArguments += @('--certificate-github-workflow-sha', $ExpectedCommit)
    }
    $CosignArguments += @('--bundle', $SignatureBundle, $Artifact)

    & $Cosign @CosignArguments

    if ($LASTEXITCODE -ne 0) {
        throw "Invalid Sigstore signature for $Artifact"
    }
}

foreach ($RequiredFile in @($Archive, $Bundle, $TrustedRoot)) {
    if (-not (Test-Path -LiteralPath $RequiredFile -PathType Leaf)) {
        throw "Required file not found: $RequiredFile"
    }
}

$CosignCommand = Get-Command $Cosign -CommandType Application -ErrorAction Stop |
    Select-Object -First 1
$TarCommand = Get-Command tar -CommandType Application -ErrorAction Stop |
    Select-Object -First 1
$Cosign = $CosignCommand.Source
$Archive = (Resolve-Path -LiteralPath $Archive).Path
$Bundle = (Resolve-Path -LiteralPath $Bundle).Path
$TrustedRoot = (Resolve-Path -LiteralPath $TrustedRoot).Path

if (Test-Path -LiteralPath $ExtractTo) {
    if (Get-ChildItem -LiteralPath $ExtractTo -Force | Select-Object -First 1) {
        throw "Extraction directory is not empty: $ExtractTo"
    }
}
else {
    New-Item -ItemType Directory -Path $ExtractTo | Out-Null
}
$ExtractTo = (Resolve-Path -LiteralPath $ExtractTo).Path

$WorkflowIdentity = "https://github.com/$ExpectedRepository/.github/workflows/publish.yml@refs/tags/$ExpectedTag"
Invoke-CosignVerification `
    -Artifact $Archive `
    -SignatureBundle $Bundle `
    -Identity $WorkflowIdentity

$ArchiveEntries = & $TarCommand.Source -tzf $Archive
if ($LASTEXITCODE -ne 0) {
    throw 'Unable to list the verified archive.'
}
foreach ($Entry in $ArchiveEntries) {
    if ($Entry -match '(^/|(^|/)\.\.(/|$))') {
        throw "Unsafe path in archive: $Entry"
    }
}

$VerboseEntries = & $TarCommand.Source -tvzf $Archive
if ($LASTEXITCODE -ne 0) {
    throw 'Unable to inspect the verified archive.'
}
foreach ($Entry in $VerboseEntries) {
    if ($Entry -notmatch '^[-d]') {
        throw 'The archive contains a symbolic link or unsupported entry.'
    }
}

& $TarCommand.Source -xzf $Archive -C $ExtractTo
if ($LASTEXITCODE -ne 0) {
    throw 'Unable to extract the verified archive.'
}

$Version = $ExpectedTag.Substring(1)
$KitName = "sonar-offline-kit-$Version-$Platform"
$KitRoot = Join-Path $ExtractTo $KitName
if (-not (Test-Path -LiteralPath $KitRoot -PathType Container)) {
    throw "Expected kit directory not found: $KitName"
}

$TopLevelEntries = @(Get-ChildItem -LiteralPath $ExtractTo -Force)
if ($TopLevelEntries.Count -ne 1) {
    throw 'The archive must contain exactly one top-level directory.'
}

$Metadata = Join-Path $KitRoot 'metadata.env'
$ChecksumManifest = Join-Path $KitRoot 'SHA256SUMS'
$ReleaseHashes = Join-Path $KitRoot "hashes/release-hashes-$Platform.md"
$ReleaseHashesBundle = Join-Path $KitRoot "signatures/$Platform-release-hashes-$Platform.md.sigstore.json"
$EmbeddedRoot = Join-Path $KitRoot 'trust/sigstore-trusted-root.json'

foreach ($RequiredKitFile in @(
        $Metadata,
        $ChecksumManifest,
        $ReleaseHashes,
        $ReleaseHashesBundle,
        $EmbeddedRoot
    )) {
    if (-not (Test-Path -LiteralPath $RequiredKitFile -PathType Leaf)) {
        throw "Kit file not found: $RequiredKitFile"
    }
}

$ProvisionedRootHash = (Get-FileHash -LiteralPath $TrustedRoot -Algorithm SHA256).Hash
$EmbeddedRootHash = (Get-FileHash -LiteralPath $EmbeddedRoot -Algorithm SHA256).Hash
if ($ProvisionedRootHash -cne $EmbeddedRootHash) {
    throw 'The trusted root embedded in the kit differs from the provisioned root.'
}

foreach ($Line in Get-Content -LiteralPath $ChecksumManifest) {
    if ($Line -notmatch '^([0-9a-fA-F]{64}) [ *](.+)$') {
        throw "Invalid SHA256SUMS line: $Line"
    }
    $ExpectedHash = $Matches[1]
    $RelativePath = $Matches[2].Replace('/', [IO.Path]::DirectorySeparatorChar)
    $CheckedPath = Join-Path $KitRoot $RelativePath
    if (-not (Test-Path -LiteralPath $CheckedPath -PathType Leaf)) {
        throw "Checksummed file not found: $RelativePath"
    }
    $ActualHash = (Get-FileHash -LiteralPath $CheckedPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($ActualHash -cne $ExpectedHash) {
        throw "Invalid internal checksum: $RelativePath"
    }
}

$MetadataLines = @(Get-Content -LiteralPath $Metadata)
$RequiredMetadata = @(
    'format_version=1',
    "repository=$ExpectedRepository",
    "release_tag=$ExpectedTag",
    "platform=$Platform",
    "certificate_identity=$WorkflowIdentity",
    "certificate_oidc_issuer=$ExpectedOidcIssuer"
)
foreach ($ExpectedLine in $RequiredMetadata) {
    if ($MetadataLines -cnotcontains $ExpectedLine) {
        throw "Unexpected or missing metadata: $ExpectedLine"
    }
}

$SourceCommitLine = $MetadataLines |
    Where-Object { $_ -match '^source_commit=([0-9a-f]{40})$' } |
    Select-Object -First 1
if (-not $SourceCommitLine) {
    throw 'Invalid source commit metadata.'
}
$SourceCommit = $SourceCommitLine.Split('=', 2)[1]
if ($ExpectedCommit -and $SourceCommit -cne $ExpectedCommit) {
    throw 'The kit source commit differs from the expected Git commit.'
}

$CosignVersionLine = $MetadataLines |
    Where-Object { $_ -match '^cosign_version=([0-9]+\.[0-9]+\.[0-9]+)$' } |
    Select-Object -First 1
if (-not $CosignVersionLine) {
    throw 'Invalid Cosign version metadata.'
}
$ExpectedCosignVersion = $CosignVersionLine.Split('=', 2)[1]
$CosignVersionOutput = (& $Cosign version 2>&1 | Out-String)
if ($LASTEXITCODE -ne 0 -or $CosignVersionOutput -notmatch "GitVersion:\s+v?$([regex]::Escape($ExpectedCosignVersion))") {
    throw 'The verifier Cosign version does not match the kit.'
}

Invoke-CosignVerification `
    -Artifact $ReleaseHashes `
    -SignatureBundle $ReleaseHashesBundle `
    -Identity $WorkflowIdentity

$ReleaseHashLines = @(Get-Content -LiteralPath $ReleaseHashes)
$Artifacts = @(Get-ChildItem -LiteralPath (Join-Path $KitRoot 'artifacts') -File | Sort-Object Name)
if ($Artifacts.Count -eq 0) {
    throw 'The kit contains no release artifact.'
}

foreach ($Artifact in $Artifacts) {
    $ArtifactBundle = Join-Path $KitRoot "signatures/$Platform-$($Artifact.Name).sigstore.json"
    if (-not (Test-Path -LiteralPath $ArtifactBundle -PathType Leaf)) {
        throw "Signature missing for $($Artifact.Name)"
    }

    Invoke-CosignVerification `
        -Artifact $Artifact.FullName `
        -SignatureBundle $ArtifactBundle `
        -Identity $WorkflowIdentity

    $ArtifactHash = (Get-FileHash -LiteralPath $Artifact.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    $ExpectedReleasePath = "release-artifacts/$($Artifact.Name)"
    $HashFound = $false
    foreach ($HashLine in $ReleaseHashLines) {
        if ($HashLine -match '^([0-9a-fA-F]{64}) [ *](.+)$' -and
            $Matches[1] -ceq $ArtifactHash -and
            $Matches[2] -ceq $ExpectedReleasePath) {
            $HashFound = $true
            break
        }
    }
    if (-not $HashFound) {
        throw "Signed hash missing for $($Artifact.Name)"
    }
}

Write-Output "SONAR_OFFLINE_KIT_VERIFIED=$KitRoot"
$DebPackages = @($Artifacts | Where-Object { $_.Extension -ceq '.deb' })
if ($DebPackages.Count -eq 1) {
    Write-Output "SONAR_SIGNED_DEB=$($DebPackages[0].FullName)"
}
