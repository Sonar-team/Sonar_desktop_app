# Sécuriser une application Tauri contre les attaques supply chain

*Guide pratique illustré par les cas SolarWinds et XZ Utils, avec les
contre-mesures réellement en place dans SONAR.*

---

## Pourquoi ce guide

Une application Tauri cumule **trois écosystèmes de dépendances** (npm/Deno
pour le frontend, crates.io pour le backend Rust, paquets système pour le
build natif), **une chaîne de build** (toolchains, CI, bundlers) et **un canal
de distribution** (releases GitHub, installateurs natifs). Chacun de ces
maillons est une cible : compromettre l'un d'eux suffit à livrer du code
malveillant à tous les utilisateurs, avec la signature et la réputation du
projet légitime.

Deux attaques historiques structurent ce guide, parce qu'elles illustrent les
deux extrémités du spectre :

### Le cas SolarWinds (SUNBURST, 2020) — compromettre la chaîne de build

Les attaquants n'ont **jamais modifié le code source** de SolarWinds Orion.
Ils ont compromis l'**environnement de build** : un implant (SUNSPOT)
surveillait le serveur de compilation et substituait un fichier source à la
volée, pendant la compilation, puis restaurait l'original. Le binaire
troyanisé était ensuite **signé avec le certificat légitime** de SolarWinds et
distribué par le canal de mise à jour officiel à ~18 000 organisations. La
revue de code était inutile : le dépôt était propre. Seule une comparaison
entre le binaire livré et un binaire reconstruit indépendamment depuis les
sources aurait révélé la supercherie.

**Leçon : il faut pouvoir prouver que le binaire publié correspond au code
source, par un build reproductible et une provenance vérifiable.**

### Le cas XZ Utils (CVE-2024-3094, 2024) — compromettre la confiance humaine

Un contributeur sous pseudonyme ("Jia Tan") a passé **plus de deux ans** à
gagner la confiance du mainteneur épuisé de xz/liblzma, jusqu'à obtenir les
droits de mainteneur. La porte dérobée visait OpenSSH (via la liaison
systemd → liblzma) et présentait deux caractéristiques redoutables :

- le script d'injection (`build-to-host.m4` modifié) n'était présent **que
  dans les tarballs de release**, pas dans le dépôt git — comparer le code
  source ne montrait rien ;
- la charge utile était dissimulée dans des **fichiers de test binaires**,
  zone morte que personne ne relit.

L'attaque n'a été découverte que par hasard, par un ingénieur intrigué par
des connexions SSH 500 ms trop lentes.

**Leçon : il faut que les artefacts publiés soient construits par un
pipeline auditable directement depuis le dépôt, que chaque dépendance soit
auditée, et qu'aucun mainteneur seul ne puisse altérer silencieusement la
chaîne.**

---

## Cartographie de la surface d'attaque d'une application Tauri

```
   [npm / crates.io / apt]          [Toolchains: Rust, Node, Deno, Tauri CLI]
            │                                       │
            ▼                                       ▼
   1. Dépendances ──────────► 2. Environnement de build ──► 3. CI (GitHub Actions)
                                                                    │
                                                                    ▼
   6. Poste utilisateur ◄──── 5. Distribution (release) ◄──── 4. Artefacts
```

Chaque section ci-dessous suit le même format : **chemin d'attaque** →
**cas réel** → **contre-mesures dans SONAR** (avec les fichiers concernés) →
**vérification**.

---

## Risque 1 — Dépendance malveillante (typosquatting, version piégée, dependency confusion)

**Chemin d'attaque.** Un attaquant publie un paquet au nom proche d'un paquet
légitime (`packet-parser` vs `packet_parser`), prend le contrôle d'un compte
de mainteneur npm/crates.io et pousse une version piégée, ou exploite une
résolution de registre ambiguë pour servir son paquet à la place du vôtre.
Le code malveillant s'exécute au build (script `postinstall`, `build.rs`) ou
au runtime chez tous vos utilisateurs.

**Cas réel.** C'est le mécanisme XZ côté distribution : une version piégée
d'une dépendance de confiance. Les campagnes npm (event-stream 2018,
ua-parser-js 2021) suivent le même schéma.

**Contre-mesures dans SONAR :**

- **Versions exactes et lockfiles gelés.** Le frontend est résolu depuis
  `package.json` (versions exactes, sans `^`), `deno.json` et `deno.lock` ;
  la CI installe avec `deno install --frozen` : toute divergence entre le
  lockfile et la résolution fait échouer le build.
- **Vendoring complet des crates Rust.** Les ~540 dépendances Rust sont
  copiées dans `src-tauri/vendor/` et `src-tauri/.cargo/config.toml` remplace
  crates.io par ce répertoire (`replace-with = "vendored-sources"`). Le build
  ne télécharge **rien** : une version piégée publiée sur crates.io ne peut
  pas entrer sans un commit relu qui modifie le vendor.
- **Statut cargo-vet bloquant.** `src-tauri/supply-chain/` (`audits.toml`,
  `config.toml`, `imports.lock`) distingue les audits importés des exemptions
  explicites. La CI contrôle les graphes `src-tauri` et `sonar-rust` avec un
  store partagé : l'ajout d'une crate ou version non couverte échoue jusqu'à
  audit ou exemption relue.
- **cargo-deny et cargo-audit en CI** (`.github/workflows/rust-ci.yml`) :
  sources inconnues refusées, licences contrôlées (`src-tauri/deny.toml`),
  vulnérabilités connues (RUSTSEC) bloquantes — les exceptions sont
  documentées avec leur justification dans `deny.toml`.

**Vérification :**

```bash
deno install --frozen
cd src-tauri && cargo vet --locked --frozen && cargo deny check && cargo audit
```

---

## Risque 2 — Altération silencieuse d'un lockfile

**Chemin d'attaque.** Une PR anodine ("bump deps") modifie `Cargo.lock` ou
`deno.lock` pour pointer vers une version ou une source compromise. Le diff
d'un lockfile fait des centaines de lignes que personne ne lit ; le paquet
piégé passe en production dans le bruit.

**Cas réel.** Vecteur privilégié des attaques par mainteneur compromis : dans
le cas XZ, les changements dangereux étaient noyés dans des modifications
d'apparence routinière.

**Contre-mesures dans SONAR :**

- **Tout changement de lockfile est traité comme un changement de code** et
  relu comme tel (règle formalisée dans
  `project_management/sprint_secure_tauri_build_methodology.md`).
- **Le vendoring rend le changement visible.** Modifier une version Rust ne
  touche pas qu'une ligne de lockfile : le contenu de la crate apparaît dans
  le diff de `src-tauri/vendor/`, et le statut cargo-vet doit suivre.
- **Dependabot** ouvre des PR de mise à jour individuelles et traçables,
  plutôt que des bumps groupés invérifiables.

---

## Risque 3 — Dérive de l'environnement de build

**Chemin d'attaque.** Le build utilise "la dernière version" d'un outil
(Rust, Node, une image Docker `latest`, un paquet apt non épinglé). Un
attaquant qui compromet l'un de ces canaux — ou une simple mise à jour
malveillante en amont — modifie le binaire produit sans qu'aucune ligne du
dépôt ne change. Impossible ensuite de savoir quel environnement a produit
quel binaire.

**Cas réel.** Étape préparatoire de SolarWinds : la maîtrise de
l'environnement de build par l'attaquant était totale, et personne ne pouvait
reconstruire un binaire de référence pour comparer.

**Contre-mesures dans SONAR :**

- **Une source canonique unique des versions d'outillage** :
  `config/build-versions.env` (Rust, Node, Deno, Tauri CLI, Vite…).
  Les workflows CI chargent ces versions via
  `script/ci/export-build-versions.sh`.
- **Alignement vérifié en CI** : `script/ci/check-build-versions.sh` échoue
  si `rust-toolchain.toml`, `package.json`, le `Dockerfile`, les workflows ou
  les références d'outillage divergent du fichier canonique. Il vérifie aussi
  qu'aucun installeur Npcap/WinPcap n'est présent dans le dépôt.
- **Image Docker épinglée par digest SHA256** (pas par tag) et **archives
  Node.js/Deno vérifiées par SHA256** au téléchargement dans le `Dockerfile`.
- **Paquets système figés dans le temps** : `script/ci/use-apt-snapshot.sh`
  pointe apt vers des snapshots datés (`APT_SNAPSHOT_TIMESTAMP`) et les
  paquets sont épinglés à version exacte (`LINUX_APT_PACKAGES`). Deux builds
  du même commit voient exactement les mêmes paquets, même à des mois
  d'écart.

**Vérification :**

```bash
./script/ci/check-build-versions.sh
```

---

## Risque 4 — Injection pendant la compilation (le scénario SolarWinds)

**Chemin d'attaque.** Le serveur de build est compromis. Le code source est
propre, la revue ne voit rien, mais le binaire produit contient une charge
malveillante injectée entre la lecture des sources et l'édition de liens.
C'est exactement SUNSPOT/SUNBURST.

**Contre-mesure de fond : le build reproductible.** Si deux builds
indépendants du même commit produisent le même binaire octet pour octet,
alors un binaire publié qui diffère du rebuild d'un auditeur est la preuve
d'une injection. La reproductibilité transforme "faites-nous confiance" en
"vérifiez par vous-même".

**Ce que SONAR met en place :**

- **Environnement de reproductibilité centralisé** : `security/repro-env.ts`
  injecte les mêmes variables dans les builds de release, les commandes
  locales et les contrôles CI — `SOURCE_DATE_EPOCH` stable (timestamps
  déterministes), `--remap-path-prefix` (aucun chemin local ne fuit dans le
  binaire), flag MSVC `/Brepro` sous Windows.
- **Contrôle de reproductibilité exécutable** : `security/repro-check.sh`
  construit deux fois le binaire et compare les SHA256 ; le workflow
  `publish.yml` contient un job `verify unsigned reproducible binary` qui
  bloque la release si le binaire n'est pas reproductible.
- **Périmètre explicite** : la reproductibilité porte sur le **binaire non
  signé** ; signatures, hashes et attestations sont ajoutés *après*, en
  fichiers détachés, pour ne pas modifier les octets du payload (voir
  `project_management/sprint_review_reproducible_builds.md`).

**Vérification (auditeur externe) :**

```bash
./security/repro-check.sh    # rebuild local, comparer au SHA256 publié
```

---

## Risque 5 — Artefact publié différent du dépôt (le scénario XZ)

**Chemin d'attaque.** Le dépôt git est sain, mais l'artefact téléchargé par
les utilisateurs contient un supplément : la porte dérobée XZ ne vivait que
dans les tarballs de release, générés à la main par le mainteneur compromis.
Personne ne comparait tarball et dépôt.

**Contre-mesures dans SONAR :**

- **Aucun artefact fabriqué à la main.** Toutes les releases sortent du
  workflow `publish.yml`, qui construit depuis le commit taggé. Il n'existe
  pas d'étape locale opaque entre le dépôt et le fichier téléchargé.
- **Attestations de provenance GitHub** (`actions/attest-build-provenance`,
  via `script/ci/generate-attestation-subjects.sh`) : chaque artefact est lié
  cryptographiquement au dépôt, au commit et au workflow qui l'a produit.
  Un artefact substitué ou fabriqué ailleurs ne peut pas présenter cette
  preuve.
- **Signatures Sigstore/cosign détachées**
  (`script/ci/sign-release-artifacts.sh`) : chaque fichier de release a son
  bundle `.sigstore.json` publié à côté.
- **SHA256 publics** de tous les artefacts, listés dans le corps de la
  release GitHub.

**Vérification (utilisateur ou auditeur) :**

```bash
sha256sum sonar_4.0.1_amd64.deb        # comparer à la valeur publiée
gh attestation verify sonar_4.0.1_amd64.deb -R Sonar-team/Sonar_desktop_app
cosign verify-blob --bundle sonar_4.0.1_amd64.deb.sigstore.json sonar_4.0.1_amd64.deb
```

---

## Risque 6 — Compromission de la CI elle-même

**Chemin d'attaque.** Les workflows référencent des actions tierces par tag
flottant (`uses: some/action@v4`). L'auteur de l'action — ou un attaquant qui
vole son compte — republie le tag avec du code qui exfiltre les secrets du
job ou altère les artefacts. C'est l'attaque tj-actions/changed-files
(mars 2025), qui a exposé les secrets de milliers de dépôts.

**Contre-mesures dans SONAR :**

- **Toutes les actions GitHub sont épinglées par SHA de commit complet**, pas
  par tag : `actions/checkout@34e11487…`, `dtolnay/rust-toolchain@3c5f7ea2…`,
  `sigstore/cosign-installer@6f9f1778…`, etc. Republier un tag ne change rien ;
  seul un commit relu peut faire évoluer la version utilisée.
- **Permissions minimales par job** : les workflows déclarent
  `permissions: contents: read` par défaut et n'élèvent
  (`attestations: write`, `security-events: write`) que là où c'est requis.
- **Outils CI installés en versions verrouillées** (`cargo install --locked`,
  versions issues de `build-versions.env`).

---

## Risque 7 — Vulnérabilités héritées des dépendances

**Chemin d'attaque.** Pas besoin de malveillance : une CVE dans une
dépendance transitive (image, parsing, compression) suffit à exposer les
utilisateurs. Sans inventaire, ni vous ni eux ne savez que le composant
vulnérable est embarqué.

**Contre-mesures dans SONAR :**

- **SBOM publié à chaque release** : `script/ci/generate-sbom-artifacts.sh`
  produit les inventaires CycloneDX backend (cargo-cyclonedx) et frontend
  (syft), attachés à la release et eux-mêmes attestés. Un intégrateur peut
  croiser le SBOM avec les bases de CVE sans rien nous demander.
- **Scans continus** : `cargo audit` (RUSTSEC) à chaque CI, **Trivy** sur le
  dépôt (planifié chaque semaine, `trivy.yml`) et **Trivy sur les artefacts
  de release** (`trivy-release-artifacts.yml`) avant publication.
- **Qualité du code surveillée** : SonarQube (`sonarcube.yml`) et couverture
  (`covecode.yml`).

---

## Risque 8 — Le facteur humain (le cœur du cas XZ)

**Chemin d'attaque.** Ingénierie sociale patiente : un contributeur serviable
obtient des droits, puis glisse la charge dans une zone que personne ne relit
(fichiers de test binaires, scripts de build générés, gros diffs
mécaniques).

**Contre-mesures dans SONAR :**

- **Réduire les zones mortes de la revue** : le vendoring et cargo-vet
  transforment "une ligne de lockfile" en diff de contenu visible ; les
  scripts de build (`build.rs`, `security/*.ts`, `script/ci/*.sh`) sont
  courts, versionnés et commentés — pas de script généré opaque.
- **Le pipeline ne dépend pas d'un humain de confiance** : les artefacts
  sortent de la CI avec provenance, pas du poste d'un mainteneur. Même un
  mainteneur légitime ne peut pas publier un binaire divergent sans casser
  l'attestation.
- **Méthodologie écrite et checklists** :
  `project_management/sprint_secure_tauri_build_methodology.md` définit quoi
  vérifier, dans quel ordre, et distingue contrôles bloquants et
  diagnostiques — la sécurité ne repose pas sur la mémoire d'une personne.

---

## Risque 9 — Surface d'attaque au runtime (défense en profondeur)

**Chemin d'attaque.** Si malgré tout un code hostile atteint le webview
(dépendance frontend compromise, XSS), il tente d'exfiltrer des données ou
d'invoquer des commandes Tauri sensibles.

**Contre-mesures dans SONAR :**

- **CSP stricte** (`src-tauri/tauri.conf.json`) :
  scripts, styles, images et polices limités à `'self'`; inline, `data:`,
  objets, formulaires et frames interdits. Les connexions sont limitées à
  l'IPC Tauri et `blob:` au worker local ForceAtlas2. Une politique distincte
  n'autorise les WebSockets Vite qu'en développement.
  Une charge injectée ne peut ni charger de code externe ni téléphoner à la
  maison.
- **Permissions Tauri déclaratives** (`src-tauri/capabilities/`) : le
  frontend ne conserve que `core`, les dialogues, l'écriture du fichier choisi,
  les logs et la fermeture de l'application ; aucun accès récursif aux dossiers
  personnels.

---

## Tableau récapitulatif

| # | Risque | Cas réel | Contre-mesure principale | Référence |
|---|--------|----------|--------------------------|-----------|
| 1 | Dépendance malveillante | XZ, event-stream | Lockfiles gelés, vendoring, cargo-vet, cargo-deny/audit | `src-tauri/vendor/`, `supply-chain/`, `deny.toml` |
| 2 | Lockfile altéré | XZ | Lockfile relu comme du code, diff vendor visible | `sprint_secure_tauri_build_methodology.md` |
| 3 | Dérive d'environnement | SolarWinds (prérequis) | Versions canoniques + digests + snapshots apt | `config/build-versions.env`, `check-build-versions.sh` |
| 4 | Injection au build | **SolarWinds** | Builds reproductibles vérifiés avant release | `security/repro-env.ts`, `repro-check.sh` |
| 5 | Artefact ≠ dépôt | **XZ** | CI seule source d'artefacts + attestations + cosign + SHA256 | `publish.yml`, `sign-release-artifacts.sh` |
| 6 | CI compromise | tj-actions 2025 | Actions épinglées par SHA, permissions minimales | `.github/workflows/*.yml` |
| 7 | CVE transitives | — | SBOM CycloneDX, cargo-audit, Trivy | `generate-sbom-artifacts.sh`, `trivy*.yml` |
| 8 | Facteur humain | **XZ** | Provenance CI, pas de zone morte de revue, méthodologie écrite | `project_management/` |
| 9 | Runtime webview | — | CSP stricte, permissions Tauri | `tauri.conf.json`, `capabilities/` |

---

## Checklist express

**Mainteneur, avant release :**

1. `./script/ci/check-build-versions.sh` — outillage aligné
2. `deno install --frozen` — frontend gelé
3. `cargo deny check && cargo audit` (dans `src-tauri/`) — dépendances saines
4. `./security/repro-check.sh` — binaire reproductible
5. Tag → le workflow `publish.yml` fait le reste (build, hashes,
   attestations, signatures, SBOM) ; ne jamais uploader un artefact à la main.

**Auditeur, après release :**

1. `sha256sum <artefact>` vs valeur publiée
2. `gh attestation verify <artefact> -R Sonar-team/Sonar_desktop_app`
3. `cosign verify-blob --bundle <artefact>.sigstore.json <artefact>`
4. Croiser le SBOM publié avec les bases de vulnérabilités
5. Optionnel : rebuild local du commit taggé et comparaison des SHA256

---

## Limites connues et durcissements restants

Par honnêteté — le point faible de SolarWinds comme de XZ était justement ce
que personne n'admettait ne pas couvrir :

- La **reproductibilité porte sur le binaire**, pas encore sur tous les
  installateurs natifs (MSI/NSIS/DMG/DEB/RPM embarquent des métadonnées
  variables) ; ceux-ci sont couverts par hashes + attestations + signatures.
- La **vérification post-release** (attestation, signature) reste une étape
  manuelle documentée, pas encore automatisée.
- Les **exemptions cargo-vet** (`supply-chain/config.toml`) marquent des
  crates non encore auditées en profondeur : la couverture d'audit progresse
  release après release.
- Les **exceptions cargo-deny** (`deny.toml`) sont justifiées et suivies,
  mais dépendent de migrations amont (GTK/WebKit).

Ces écarts sont suivis dans `project_management/` et ont vocation à devenir
des contrôles bloquants à mesure que l'outillage mûrit.
