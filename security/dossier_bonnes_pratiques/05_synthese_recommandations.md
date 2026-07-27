# 5. Synthèse et recommandations

## 5.1 Ce que démontre le retour d'expérience

Les deux compromissions de référence — SUNBURST et XZ Utils — établissent
que la menace supply chain contourne les défenses traditionnelles : ni la
revue de code, ni la signature des binaires, ni l'antivirus ne protègent
lorsque c'est la **fabrication** ou la **distribution** du logiciel qui est
compromise.

Le projet démonstrateur montre qu'une réponse de niveau élevé est
**atteignable pour un projet open source à effectifs réduits**, avec des
outils exclusivement libres et gratuits, en s'appuyant sur trois piliers :

1. **Maîtrise des dépendances** — versions figées, vendoring, audit continu,
   de sorte que rien n'entre dans le produit sans revue explicite.
2. **Intégrité de la fabrication** — environnement de build canonique et
   reproductible, permettant de prouver que le binaire correspond au code.
3. **Provenance et transparence de la distribution** — artefacts produits
   uniquement par la CI, attestés, signés, hachés, accompagnés d'un
   inventaire logiciel (SBOM) vérifiable par un tiers.

La propriété la plus importante n'est pas telle mesure isolée, mais la
**vérifiabilité par un tiers** : un auditeur externe, une administration qui
déploie l'application, peut confirmer l'intégrité et l'origine d'une release
**sans faire confiance aux mainteneurs**, en rejouant des commandes
documentées. C'est le renversement exact du modèle qui a rendu SolarWinds et
XZ possibles.

## 5.2 Correspondance avec les référentiels publics

La démarche recoupe les attentes des principaux cadres de référence, ce qui
facilite son adoption dans un contexte institutionnel :

| Exigence de référentiel                             | Mise en œuvre dans la démarche                                                |
| --------------------------------------------------- | ----------------------------------------------------------------------------- |
| Maîtrise et inventaire des composants (SBOM)        | SBOM CycloneDX backend + frontend, publié et attesté                          |
| Intégrité et provenance des artefacts (esprit SLSA) | attestations de provenance GitHub, builds issus de la seule CI                |
| Builds reproductibles / vérifiables                 | `repro-env.ts` + `repro-check.sh` + job CI bloquant                           |
| Gestion des vulnérabilités connues                  | cargo-audit, Trivy dépôt et artefacts, mises à jour tracées                   |
| Sécurité du développement (esprit NIST SSDF)        | méthodologie écrite, revue des lockfiles, actions CI épinglées                |
| Moindre privilège                                   | permissions CI minimales par job, permissions Tauri déclaratives, CSP stricte |

_(Les intitulés de référentiels sont donnés à titre indicatif ; une mise en
correspondance formelle avec un référentiel précis peut être produite sur
demande.)_

## 5.3 Recommandations pour une adoption institutionnelle

Pour une administration souhaitant appliquer ou exiger cette démarche :

1. **Exiger la vérifiabilité, pas la confiance.** Conditionner l'homologation
   d'un logiciel open source à la présence de hash publics, d'attestations de
   provenance et d'un SBOM — critères objectifs et automatisables.
2. **Traiter le fichier de verrouillage et la configuration CI comme du code
   sensible**, soumis à revue au même titre que le code métier.
3. **Privilégier les projets à build reproductible** pour les usages
   sensibles, et se réserver la possibilité de reconstruire et comparer.
4. **Intégrer le SBOM au processus de veille** en vulnérabilités du SI, pour
   détecter l'exposition à une CVE amont sans dépendre du rythme du projet.
5. **Soutenir les mainteneurs.** L'affaire XZ rappelle que l'épuisement d'un
   mainteneur isolé est un risque de sécurité national : le soutien
   (financier, humain, en revue) aux projets open source critiques est une
   mesure de sécurité à part entière.

## 5.4 Limites connues et trajectoire de durcissement

Par souci d'honnêteté méthodologique — l'angle mort assumé étant précisément
ce qui a permis les deux cas de référence :

- La reproductibilité porte aujourd'hui sur le **binaire**, pas encore sur
  tous les installateurs natifs (MSI, NSIS, DEB, RPM, DMG), qui embarquent
  des métadonnées variables ; ceux-ci restent couverts par hash, attestation
  et signature.
- La **vérification post-release** (attestation, signature) est documentée
  mais reste une étape manuelle, à automatiser.
- Certaines crates figurent en **exemption cargo-vet** (audit approfondi non
  encore réalisé) ; la couverture progresse à chaque release.
- Quelques **exceptions cargo-deny** subsistent, justifiées et suivies,
  dépendant de migrations en amont (pile GTK/WebKit).

Ces écarts sont documentés et suivis comme un arriéré d'actions, avec pour
objectif de les transformer progressivement en contrôles bloquants.

## 5.5 Checklists opérationnelles

**Mainteneur — avant publication d'une release :**

1. `./script/ci/check-build-versions.sh` — outillage aligné sur la source
   canonique
2. `deno install --frozen` — frontend gelé conforme au lockfile
3. `cargo vet --locked --frozen && cargo deny check && cargo audit` (dans
   `src-tauri/`) — statut d'audit, licences et vulnérabilités contrôlés
4. `./security/repro-check.sh` — binaire reproductible
5. Créer le tag → le workflow `publish.yml` produit build, hashes,
   attestations, signatures et SBOM. **Ne jamais téléverser un artefact à la
   main.**

**Auditeur / intégrateur — après publication :**

1. `sha256sum <artefact>` comparé à la valeur publiée
2. `gh attestation verify <artefact> -R <org>/<repo>`
3. `script/ci/verify-offline-release-kit.sh` avec le tag exact, Cosign et la
   racine de confiance provisionnés hors du kit
4. Croiser le SBOM publié avec les bases de vulnérabilités
5. Optionnel : reconstruire le commit taggé et comparer les empreintes SHA256

## 5.6 Glossaire

- **Supply chain (chaîne d'approvisionnement logicielle)** : ensemble des
  éléments — dépendances, outils, infrastructure — participant à la
  production et à la distribution d'un logiciel.
- **SBOM (Software Bill of Materials)** : inventaire structuré et lisible par
  machine de tous les composants d'un logiciel et de leurs versions.
- **Build reproductible** : propriété selon laquelle recompiler le même code
  source dans le même environnement produit un binaire identique au bit près.
- **Provenance / attestation** : preuve cryptographique liant un artefact au
  code source, au commit et au processus (workflow) qui l'a produit.
- **Sigstore / cosign** : infrastructure et outil de signature d'artefacts
  logiciels, avec vérification publique.
- **Vendoring** : copie des dépendances dans le dépôt du projet, pour
  s'affranchir des registres distants au moment du build.
- **CSP (Content Security Policy)** : politique restreignant les ressources
  qu'une page (ici le webview Tauri) est autorisée à charger et contacter.
- **cargo-vet / cargo-deny / cargo-audit** : outils de l'écosystème Rust pour
  l'audit, le contrôle de licences/sources et la détection de vulnérabilités
  des dépendances.
- **Épinglage par empreinte (SHA-pinning)** : référencement d'une dépendance
  ou d'une action CI par son empreinte de commit plutôt que par un tag
  mutable.

---

_Dossier établi à partir du projet SONAR (analyseur de trafic réseau open
source, application Tauri). L'ensemble des mesures citées est vérifiable dans
le dépôt public du projet ; les fichiers de référence sont indiqués au fil du
document._
