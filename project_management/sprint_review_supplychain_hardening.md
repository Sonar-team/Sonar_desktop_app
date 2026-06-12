# Sprint Compilation Securisee

Voici la liste de ce qui a ete mis en place.

## Reproductibilite

- Build reproductible limite au binaire, pas aux bundles.
- Script central : `security/repro-env.ts`.
- Variables injectees :
  - `SOURCE_DATE_EPOCH`
  - `RUSTFLAGS`
  - `--remap-path-prefix=$repo=/workspace`
  - remap Rust/Cargo vers chemins stables
  - `/Brepro` pour Windows MSVC
- Gate CI : `security/repro-check.sh`.
- Le binaire Linux est builde deux fois avec flags reproductibles et compare byte-to-byte.
- La release n'est creee que si ce check passe.

## Manifest SHA256 Source

- `source-manifest.sha256` genere.
- Environ `32361` entrees.
- Generation via `security/generate_manifest.sh`.
- Le manifest est produit depuis l'arbre Git indexe avec `git write-tree` + `git archive`, donc il evite d'embarquer les fichiers locaux non stages.
- Exclusions prevues : `.codex/*`, `.agents/*`, `sbom/*`, le manifest lui-meme.

## Versions Et Digests Pinnes

- Fichier canonique : `config/build-versions.env`.
- Versions figees :
  - Rust `1.95.0`
  - Node `24.14.0`
  - Deno `2.7.13`
  - Tauri CLI `2.11.1`
- Image Docker Rust pinnee par digest :
  - `sha256:5b1e3484...`
- Image coverage tarpaulin pinnee par digest.
- Verification d'alignement via `script/ci/check-build-versions.sh`.
- Les workflows utilisent des actions GitHub pinnees par commit SHA.

## APT / Environnement Systeme

- Dependances Linux installees depuis snapshot APT.
- Timestamp snapshot :
  - `20260510T000000Z`
- Script : `script/ci/use-apt-snapshot.sh`.
- Versions de paquets Linux figees dans `config/build-versions.env`.

## Release Securisee

- Workflow principal : `.github/workflows/publish.yml`.
- Ordre impose :
  1. verifier reproductibilite du binaire ;
  2. creer la release GitHub ;
  3. builder les binaires par plateforme ;
  4. smoke test du binaire ;
  5. builder les bundles Tauri ;
  6. publier artefacts ;
  7. generer SHA256 ;
  8. generer provenance ;
  9. signer avec Cosign ;
  10. publier hashes dans le corps de release.

## SHA256 / Hashes / Digests

- Script : `script/ci/generate-release-hashes.sh`.
- Genere un `release-hashes-<platform>.md` par plateforme.
- Les hashes SHA256 sont ajoutes au corps de la release via `script/ci/update-release-body.sh`.
- Les fichiers de hashes eux-memes sont inclus dans les sujets d'attestation.

## Provenance

- Action utilisee : `actions/attest`.
- Permissions activees :
  - `id-token: write`
  - `attestations: write`
- Script de sujets : `script/ci/generate-attestation-subjects.sh`.
- Les artefacts de release + fichiers SHA256 sont listes comme sujets de provenance.

## Cosign / Sigstore

- Installation via `sigstore/cosign-installer`.
- Script : `script/ci/sign-release-artifacts.sh`.
- Signature detached de chaque artefact avec :
  - `cosign sign-blob --bundle ...sigstore.json`
- Upload des bundles Sigstore via `script/ci/upload-sigstore-bundles.sh`.

## Smoke Test Binaire

- Nouveau mode CLI : `--sonar-smoke-test`.
- Fichier Rust : `src-tauri/src/startup_smoke.rs`.
- Le test verifie les logs obligatoires :
  - `Using device ...`
  - `SONAR_STARTUP_VALIDATION=OK`
- Script CI : `script/ci/smoke-test-release-binary.sh`.
- Windows a un runtime smoke dedie avec DLL Npcap extraites :
  - `script/ci/prepare-windows-smoke-runtime.ps1`

## Bundles Publies

- Bundles generes par Tauri, non reproductibles volontairement.
- Artefacts publies dans `app-v3.13.19` :
  - Linux : `sonar-linux-x86_64`, `.deb`, `.rpm`
  - Windows : `sonar-windows-x86_64.exe`, setup `.exe`, `.msi`
  - macOS : `sonar-macos-aarch64`, `.dmg`

## Controles Rust / Supply Chain

- `cargo fmt --check`
- `cargo test`
- `cargo audit`
- `cargo clippy -D warnings`
- `cargo deny check`
- `cargo outdated`
- `cargo udeps`
- `deny.toml` ajoute/configure :
  - licences autorisees
  - sources inconnues interdites
  - advisories suivies avec exemptions documentees

## Coverage

- Workflow coverage repare.
- Rust coverage via tarpaulin.
- Frontend coverage via Deno.
- Upload separe Codecov :
  - flag `rust`
  - flag `frontend`

## Etat Valide

- Release stable : `app-v3.13.19`
- Publish vert.
- Rust CI vert.
- Clippy vert.
- Trivy vert.
- Coverage vert.
- Seul point hors sprint encore a traiter : SonarQube config/token sur certaines PR, pas un probleme de compilation/release.
