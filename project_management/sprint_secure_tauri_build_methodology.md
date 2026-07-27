# Sprint: methodologie de build securise pour application Tauri

> Statut : archivé — méthodologie livrée, écarts d'implémentation transférés
> Dernière revue : 13/07/2026
> Suites actives : #94, #146 et #162 en P0 ; #143 et #163 en P1.

## Objectif

Formaliser une methodologie de build securise pour SONAR et, plus largement,
pour une application Tauri. Le but est de transformer les protections deja
mises en place en un processus reutilisable, auditable et applicable a chaque
release.

La methodologie doit couvrir:

- la maitrise des dependances frontend, Rust et systeme;
- la maitrise des versions d'outils;
- la reproductibilite des builds;
- la securisation des artefacts de release;
- la verification par un mainteneur ou un auditeur externe.

## Contexte

Le sprint precedent a pose une base solide:

- versions Rust, Node.js, Deno, Tauri CLI et Vite centralisees dans
  `config/build-versions.env`;
- dependances frontend controlees par `deno.json`, `deno.lock` et
  `package.json`;
- installation frontend gelee avec `deno install --frozen`;
- dependances Rust verrouillees par `Cargo.lock` et vendoring dans
  `src-tauri/vendor/`;
- Docker base image epinglee par digest;
- verification SHA256 des archives Node.js et Deno dans le Dockerfile;
- paquets APT epingles ou recuperes depuis des snapshots dates;
- environnement reproductible centralise dans `security/repro-env.ts`;
- controle de reproductibilite avant publication;
- hashes SHA256, signatures Sigstore/cosign et attestations GitHub pour les
  releases.

Ce nouveau sprint ne vise pas seulement a ajouter des scripts. Il vise a
documenter une methode complete: quoi verifier, dans quel ordre, avec quels
fichiers de reference, et avec quels criteres de validation.

## Probleme a resoudre

Les protections existent, mais elles sont dispersees entre plusieurs fichiers:

- workflows GitHub Actions;
- scripts CI;
- configuration Deno/npm;
- configuration Cargo;
- Dockerfile;
- documentation de reproductibilite;
- scripts de release trust.

Sans methodologie explicite, un mainteneur peut:

- mettre a jour une version sans mettre a jour tous les fichiers alignes;
- publier une release sans SBOM ou sans verifier l'attestation;
- confondre reproductibilite, signature et provenance;
- ne pas savoir quels controles sont bloquants et lesquels sont seulement
  diagnostiques;
- introduire un outil ou une action CI non epinglee.

## Perimetre du sprint

### Inclus

- Creer une methodologie de build securise Tauri adaptee au projet.
- Definir les entrees de build qui doivent etre stables.
- Definir les controles CI obligatoires avant release.
- Definir le flux de release securise.
- Definir une checklist mainteneur avant publication.
- Definir une checklist auditeur apres publication.
- Lister les attaques supply chain couvertes.
- Lister les limites actuelles et les actions de durcissement restantes.

### Hors perimetre

- Reecrire completement le pipeline de release.
- Garantir immediatement la reproductibilite byte-for-byte de tous les
  installateurs natifs.
- Remplacer Tauri bundler.
- Traiter la securite runtime de l'application, sauf lorsque la configuration
  Tauri influence le build ou les permissions de release.

## Methodologie cible

### 1. Verrouiller les sources de dependances

Objectif: empecher les changements silencieux de dependances.

Regles:

- Le frontend doit etre resolu depuis `deno.json`, `deno.lock` et
  `package.json`.
- Les installs frontend CI doivent utiliser `deno install --frozen`.
- Les versions npm doivent rester exactes dans `package.json`.
- Les overrides npm doivent etre documentes quand ils forcent une dependance
  sensible.
- Rust doit utiliser `Cargo.lock` et `src-tauri/vendor/`.
- `cargo deny` doit refuser les sources inconnues.
- Tout changement de lockfile doit etre relu comme un changement de code.

Validation:

```bash
deno install --frozen
cargo deny check
cargo audit
```

### 2. Verrouiller les outils de build

Objectif: eviter qu'un meme commit soit construit par des outils differents.

Regles:

- `config/build-versions.env` est la source canonique.
- Rust, Node.js, Deno, Tauri CLI et Vite doivent rester alignes avec ce fichier.
- Le Dockerfile doit utiliser une image Rust epinglee par digest.
- Les archives Node.js et Deno telechargees doivent etre verifiees par SHA256.
- Les workflows CI doivent utiliser les versions exportees par
  `script/ci/export-build-versions.sh`.

Validation:

```bash
./script/ci/check-build-versions.sh
```

### 3. Stabiliser l'environnement systeme

Objectif: eviter que des mises a jour OS modifient le resultat de build.

Regles:

- Les paquets APT necessaires au build doivent etre epingles.
- Les jobs Linux doivent appliquer `script/ci/use-apt-snapshot.sh` avant
  installation.
- Les versions de paquets doivent etre mises a jour volontairement, avec le
  timestamp de snapshot correspondant.

Validation:

```bash
./script/ci/use-apt-snapshot.sh
```

### 4. Construire avec l'environnement reproductible

Objectif: reduire les differences de build non liees au code source.

Regles:

- Les builds de release doivent passer par `security/repro-env.ts`.
- `SOURCE_DATE_EPOCH` doit etre stable.
- Les chemins locaux doivent etre remappes avec `--remap-path-prefix`.
- Le flag `/Brepro` doit etre active pour les builds Windows MSVC lorsque
  applicable.
- La reproductibilite doit porter d'abord sur les artefacts non signes.

Validation:

```bash
./security/repro-check.sh
```

### 5. Distinguer reproductibilite et confiance de release

Objectif: ne pas melanger le payload reproductible avec les metadonnees
variables.

Regles:

- La reproductibilite compare les artefacts non signes.
- Les signatures, attestations, SBOM et manifestes sont ajoutes apres le build.
- Les signatures doivent etre detachees pour ne pas modifier les octets de
  l'artefact.
- Les artefacts publies doivent avoir un SHA256 public.

Flux attendu:

1. Verifier la reproductibilite du binaire non signe.
2. Construire les binaires et bundles natifs.
3. Generer les hashes SHA256.
4. Generer les attestations de provenance.
5. Signer les artefacts avec Sigstore/cosign.
6. Publier hashes, signatures, attestations, SBOM et manifestes.

### 6. Verifier les releases publiees

Objectif: permettre a un utilisateur, integrateur ou auditeur de verifier ce qui
a ete livre.

Verifications attendues:

```bash
sha256sum <artifact>
gh attestation verify <artifact> -R Sonar-team/Sonar_desktop_app
cosign verify-blob \
  --trusted-root security/sigstore-trusted-root.json \
  --certificate-identity \
  "https://github.com/Sonar-team/Sonar_desktop_app/.github/workflows/publish.yml@refs/tags/<tag>" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  --bundle <artifact>.sigstore.json \
  <artifact>
```

Le sprint doit preciser les commandes finales exactes selon le format des
bundles publies.

## Attaques couvertes

- Dependency drift: changement silencieux de dependances.
- Dependency confusion: resolution depuis une source inattendue.
- Typosquatting ou paquet npm/Rust introduit sans revue.
- Lockfile tampering: modification non relue de `deno.lock` ou `Cargo.lock`.
- Compromission d'une action CI referencee par tag flottant.
- Build environment drift: outils ou paquets systeme differents entre deux
  builds.
- Artifact substitution: remplacement d'un binaire apres publication.
- Provenance spoofing: impossibilite de prouver quel workflow a construit
  l'artefact.
- Path leakage: chemins locaux integres dans les binaires.
- Timestamp drift: timestamps non controles dans les sorties de build.

## Livrables attendus

1. Document de methodologie de build securise Tauri.
2. Checklist mainteneur avant release.
3. Checklist auditeur apres release.
4. Tableau des controles CI: bloquant, diagnostique, manuel.
5. Procedure de mise a jour des versions de build.
6. Procedure de verification d'une release publiee.
7. Liste des ecarts actuels par rapport a la methodologie cible.

## User Stories

### US-01 - Mainteneur release

En tant que mainteneur, je veux une checklist de build securise avant release,
afin de publier une version sans oublier les controles critiques.

Critieres d'acceptation:

- La checklist indique les commandes a lancer.
- La checklist distingue controles bloquants et controles diagnostiques.
- La checklist reference les fichiers source de verite.

### US-02 - Reviewer securite

En tant que reviewer securite, je veux savoir quelles attaques supply chain sont
couvertes, afin d'evaluer le niveau de confiance du pipeline.

Critieres d'acceptation:

- Chaque protection est reliee a une classe d'attaque.
- Les limites restantes sont explicites.
- Les ecarts ont une action de remediation.

### US-03 - Auditeur externe

En tant qu'auditeur externe, je veux verifier une release publiee, afin de
confirmer son origine et son integrite.

Critieres d'acceptation:

- Les commandes de verification SHA256, provenance et signature sont documentees.
- Les fichiers attendus dans une release sont listes.
- La verification ne depend pas d'une etape locale non documentee.

### US-04 - Developpeur frontend

En tant que developpeur frontend, je veux savoir comment modifier les
dependances sans casser la securite du build, afin de livrer une evolution sans
introduire de drift.

Critieres d'acceptation:

- La procedure de mise a jour `package.json` / `deno.lock` est documentee.
- Toute modification du lockfile est traitee comme un changement sensible.
- `deno install --frozen` reste le controle CI de reference.

### US-05 - Developpeur Rust/Tauri

En tant que developpeur Rust/Tauri, je veux savoir comment ajouter une
dependance ou un plugin Tauri, afin de garder Cargo, vendoring et permissions
coherents.

Critieres d'acceptation:

- La procedure mentionne `Cargo.lock`, `src-tauri/vendor/` et `cargo deny`.
- Les permissions Tauri associees au plugin sont revues.
- Les nouvelles permissions sont justifiees dans la revue.

## Plan d'execution

### Etape 1 - Inventaire

- Lister tous les inputs du build:
  - toolchains;
  - lockfiles;
  - vendor Rust;
  - paquets systeme;
  - actions CI;
  - scripts de release;
  - configuration Tauri.
- Identifier la source de verite de chaque input.

### Etape 2 - Classification des controles

Classer chaque controle:

- bloquant release;
- bloquant PR;
- diagnostique;
- manuel post-release.

Exemples:

- `deno install --frozen`: bloquant release.
- `cargo audit`: bloquant PR/release selon criticite.
- Trivy release artifacts: bloquant release si HIGH/CRITICAL exploitable.
- Verification provenance post-release: manuel ou automatise selon maturite.

### Etape 3 - Procedure de build securise

Ecrire la procedure standard:

1. Charger les versions canonique.
2. Installer les dependances avec lockfiles geles.
3. Verifier l'alignement des versions.
4. Lancer les audits.
5. Construire avec l'environnement reproductible.
6. Comparer les builds si applicable.
7. Generer et publier les preuves de confiance.

### Etape 4 - Procedure de verification

Documenter comment verifier:

- hash SHA256;
- signature Sigstore;
- attestation GitHub;
- SBOM;
- correspondance source manifest / release.

### Etape 5 - Ecarts et durcissements

Produire une liste d'ecarts:

- SBOM a publier automatiquement dans le workflow release;
- workflows non-release encore bases sur `yarn` a aligner avec Deno frozen;
- installateurs natifs non encore pleinement reproductibles;
- verification post-release a automatiser;
- politique d'ajout de permissions Tauri a formaliser.

## Criteres d'acceptation du sprint

- Une methodologie complete existe dans `project_management/`.
- Elle decrit le flux de build securise de bout en bout.
- Elle couvre frontend, Rust, Tauri, OS packages, CI et release artifacts.
- Elle separe clairement:
  - reproductibilite;
  - signature;
  - provenance;
  - SBOM;
  - scan de vulnerabilites.
- Elle inclut une checklist mainteneur et une checklist auditeur.
- Elle liste les attaques supply chain couvertes.
- Elle liste les limites restantes et actions associees.

## Risques

- Methodologie trop generale et pas assez executable.
- Commandes de verification Sigstore/provenance incompletes selon les artefacts
  reels de release.
- Confusion entre build reproductible et artefact signe.
- Publication SBOM non automatisee si elle reste hors workflow `publish`.
- Durcissement Tauri incomplet si les permissions desktop ne sont pas revues en
  meme temps que les plugins.

## Definition of Done

- Le document de methodologie est relu par un mainteneur.
- Les commandes mentionnees correspondent aux scripts presents dans le repo.
- Les fichiers de reference sont cites explicitement.
- Les ecarts actuels sont transformes en backlog d'actions.
- La prochaine release peut etre verifiee en suivant uniquement la procedure
  documentee.
