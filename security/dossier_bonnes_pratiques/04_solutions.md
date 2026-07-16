# 4. Solutions — contre-mesures déployées et preuves de faisabilité

Ce chapitre présente, pour chaque scénario du chapitre 3, les contre-mesures
effectivement mises en œuvre sur le projet démonstrateur SONAR. Chaque mesure
est rattachée à un ou plusieurs fichiers réels du dépôt, ce qui la rend
**auditable** : un tiers peut vérifier son existence et son fonctionnement.

L'ensemble s'inscrit dans une méthodologie formalisée
(`project_management/sprint_secure_tauri_build_methodology.md`) et s'aligne
sur les principes des référentiels publics : maîtrise des dépendances,
intégrité de la fabrication, provenance des artefacts, transparence de la
composition logicielle.

## 4.1 Contre le scénario A — maîtriser les dépendances

**Principe : rien n'entre dans le produit sans être figé, audité et relu.**

- **Versions exactes et installations gelées.** Le frontend est déclaré en
  versions exactes (`package.json`, sans plage flottante) et résolu depuis
  `deno.json` / `deno.lock` ; la CI installe avec `deno install --frozen`,
  qui échoue à la moindre divergence avec le fichier de verrouillage.
- **Vendoring complet des dépendances Rust.** Les quelque 540 crates du
  projet sont copiées dans `src-tauri/vendor/`, et `src-tauri/.cargo/config.toml`
  redirige crates.io vers ce répertoire local
  (`replace-with = "vendored-sources"`). **Le build ne télécharge aucune
  dépendance** : une version piégée publiée en amont ne peut pas entrer sans
  un commit explicite et relu modifiant le vendor.
- **Audit par dépendance** avec cargo-vet (`src-tauri/supply-chain/` :
  `audits.toml`, `config.toml`, `imports.lock`), qui trace le statut de
  chaque crate.
- **Contrôles automatiques en intégration continue**
  (`.github/workflows/rust-ci.yml`) : `cargo deny check` (sources inconnues
  refusées, licences contrôlées via `src-tauri/deny.toml`) et `cargo audit`
  (vulnérabilités RUSTSEC).

*Vérification :* `deno install --frozen` puis, dans `src-tauri/`,
`cargo deny check && cargo audit`.

## 4.2 Contre le scénario B — traiter le lockfile comme du code

- **Règle formalisée** : tout changement de fichier de verrouillage est relu
  comme un changement de code, jamais approuvé en bloc.
- **Le vendoring rend le changement lisible** : modifier une version Rust
  ne se réduit pas à une ligne de lockfile — le contenu de la crate apparaît
  dans le diff de `src-tauri/vendor/`, et le statut cargo-vet doit être mis à
  jour, ce qui expose l'ajout à la revue.
- **Mises à jour tracées** via Dependabot (PR individuelles et vérifiables)
  plutôt que des bumps groupés opaques.

## 4.3 Contre le scénario C (SUNBURST) — le build reproductible

**Principe : pouvoir prouver que le binaire publié correspond au code
source.** Si deux compilations indépendantes du même commit produisent le
même binaire au bit près, alors un binaire publié divergent est une preuve
d'injection.

- **Environnement de reproductibilité centralisé** (`security/repro-env.ts`) :
  applique les mêmes paramètres aux builds de release, aux commandes locales
  et aux contrôles CI — horodatage déterministe (`SOURCE_DATE_EPOCH` stable),
  suppression des chemins locaux dans le binaire (`--remap-path-prefix`),
  activation du drapeau de reproductibilité MSVC (`/Brepro`) sous Windows.
- **Contrôle de reproductibilité exécutable** (`security/repro-check.sh`) :
  compile deux fois et compare les empreintes SHA256.
- **Étape bloquante en CI** : le workflow `.github/workflows/publish.yml`
  contient un job « verify unsigned reproducible binary » qui **interrompt la
  publication** si le binaire n'est pas reproductible.
- **Maîtrise de l'environnement de fabrication**, condition nécessaire à la
  reproductibilité : source canonique unique des versions d'outillage
  (`config/build-versions.env`), alignement vérifié
  (`script/ci/check-build-versions.sh`), image Docker épinglée par empreinte
  SHA256, archives Node.js/Deno vérifiées par SHA256, paquets système figés
  sur des snapshots datés (`script/ci/use-apt-snapshot.sh`).

*Vérification (auditeur) :* `./security/repro-check.sh`, puis comparaison au
SHA256 publié dans la release.

## 4.4 Contre le scénario D (XZ) — provenance et absence de fabrication manuelle

**Principe : les artefacts ne sortent que de la CI, et sont liés
cryptographiquement au dépôt.**

- **Aucun artefact fabriqué à la main.** Toutes les releases sont produites
  par le workflow `publish.yml` à partir du commit taggé ; il n'existe aucune
  étape locale opaque entre le dépôt et le fichier téléchargé.
- **Attestations de provenance GitHub** (action `attest-build-provenance`,
  via `script/ci/generate-attestation-subjects.sh`) : chaque artefact est lié
  au dépôt, au commit et au workflow qui l'a produit. Un artefact fabriqué
  ailleurs ne peut pas présenter cette preuve.
- **Signatures Sigstore/cosign détachées**
  (`script/ci/sign-release-artifacts.sh`) : chaque fichier a son bundle
  `.sigstore.json` publié à côté, sans modifier les octets de l'artefact.
- **Empreintes SHA256 publiques** de tous les artefacts, listées dans le
  corps de la release.

*Vérification (utilisateur) :*

```
sha256sum <artefact>                     # comparer à la valeur publiée
gh attestation verify <artefact> -R <org>/<repo>
cosign verify-blob --bundle <artefact>.sigstore.json <artefact>
```

## 4.5 Contre le scénario E — durcir l'intégration continue

- **Toutes les actions GitHub sont épinglées par empreinte de commit
  complète**, jamais par tag flottant. Exemples réels dans les workflows :
  `actions/checkout@34e11487…`, `dtolnay/rust-toolchain@3c5f7ea2…`,
  `sigstore/cosign-installer@6f9f1778…`. Republier un tag n'a alors aucun
  effet : seule une modification relue du workflow peut changer la version
  utilisée.
- **Permissions minimales par job** : les workflows déclarent
  `permissions: contents: read` par défaut et n'élèvent les droits
  (`attestations: write`, `security-events: write`) que là où c'est
  strictement nécessaire.
- **Outils CI installés en versions verrouillées** (`cargo install --locked`).

## 4.6 Contre le scénario F — inventaire et scan continu

- **SBOM publié à chaque release** : `script/ci/generate-sbom-artifacts.sh`
  produit les inventaires CycloneDX du backend (cargo-cyclonedx) et du
  frontend (syft), attachés à la release et eux-mêmes attestés. Un
  intégrateur peut croiser cet inventaire avec les bases de vulnérabilités
  **sans rien demander au projet**.
- **Analyses continues** : `cargo audit` (RUSTSEC) à chaque CI, Trivy sur le
  dépôt (`.github/workflows/trivy.yml`, exécution hebdomadaire planifiée) et
  Trivy sur les artefacts de release (`trivy-release-artifacts.yml`) avant
  publication.
- **Qualité et couverture** suivies (SonarQube, `covecode.yml`).

## 4.7 Contre le scénario G — défense en profondeur au runtime

- **Politique de sécurité de contenu (CSP) stricte**
  (`src-tauri/tauri.conf.json`) :
  `default-src 'self'; connect-src ipc: http://ipc.localhost; img-src 'self' data:; style-src 'self' 'unsafe-inline'; worker-src 'self' blob:`.
  Aucun script distant ni connexion réseau sortante depuis le webview : une
  charge injectée ne peut ni charger de code externe ni exfiltrer vers un
  serveur tiers. La source `blob:` est limitée aux workers locaux nécessaires
  au layout ForceAtlas2.
- **Permissions Tauri déclaratives** (`src-tauri/capabilities/`) : le
  frontend n'accède qu'aux commandes explicitement exposées ; l'ajout d'un
  plugin impose la revue de ses permissions.

## 4.8 Le facteur humain (transverse aux scénarios D et E)

Au-delà de l'outillage, la démarche vise à **réduire les zones mortes de la
revue** — précisément ce qu'exploitait l'attaque XZ :

- vendoring et cargo-vet transforment une ligne de lockfile en diff de
  contenu visible ;
- les scripts de build (`build.rs`, `security/*.ts`, `script/ci/*.sh`) sont
  courts, versionnés et commentés — aucun script généré opaque ;
- la chaîne de publication **ne dépend d'aucun individu de confiance** :
  même un mainteneur légitime ne peut pas publier un binaire divergent sans
  rompre l'attestation de provenance.

## 4.9 Tableau de correspondance scénario → contre-mesure

| Scénario | Risque | Contre-mesure principale | Fichiers de preuve |
|----------|--------|--------------------------|--------------------|
| A — dépendance piégée | R1, R3 | vendoring, lockfiles gelés, cargo-vet/deny/audit | `src-tauri/vendor/`, `supply-chain/`, `deny.toml`, `rust-ci.yml` |
| B — lockfile empoisonné | R2 | lockfile relu comme du code, diff vendor visible | méthodologie, `vendor/` |
| C — injection au build | R4 (SUNBURST) | build reproductible vérifié avant release | `repro-env.ts`, `repro-check.sh`, `publish.yml` |
| D — release manuelle | R5, R8 (XZ) | CI seule source + attestations + cosign + SHA256 | `publish.yml`, `generate-attestation-subjects.sh`, `sign-release-artifacts.sh` |
| E — action CI détournée | R6 | actions épinglées par SHA, permissions minimales | `.github/workflows/*.yml` |
| F — vulnérabilité dormante | R7 | SBOM CycloneDX, cargo-audit, Trivy | `generate-sbom-artifacts.sh`, `trivy*.yml` |
| G — évasion webview | R9 | CSP stricte, permissions Tauri | `tauri.conf.json`, `capabilities/` |

**Bilan.** Les neuf risques critiques ou élevés identifiés au chapitre 2 sont
couverts par au moins une mesure vérifiable, et les scénarios de gravité
maximale (C façon SUNBURST, D façon XZ) le sont par les mesures les plus
robustes de la démarche — reproductibilité et provenance. Surtout,
l'événement redouté ER4 (impossibilité de blanchir une release) est levé :
en cas d'alerte, hash, provenance et signature permettent de **prouver
release par release** ce qui est sain.
