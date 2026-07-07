# 2. Analyse de risques d'une application Tauri open source non sécurisée

Cette analyse considère une application Tauri développée sur GitHub **dans
sa configuration par défaut**, c'est-à-dire sans démarche de sécurisation de
la chaîne d'approvisionnement. La structure s'inspire de la méthode
EBIOS Risk Manager (ANSSI) : valeurs métier et biens supports, sources de
risque, événements redoutés, puis cotation des risques.

## 2.1 Valeurs métier et biens supports

**Valeurs métier** (ce que l'on cherche à protéger) :

- **V1 — Intégrité du logiciel livré** : l'application installée par
  l'utilisateur correspond exactement au code source publié.
- **V2 — Confiance des utilisateurs et de l'institution** : la réputation du
  projet et de ses porteurs.
- **V3 — Données des utilisateurs finaux** : dans le cas d'un analyseur de
  trafic réseau, des captures potentiellement sensibles (cartographie d'un
  SI, adresses, protocoles métier).

**Biens supports** (ce par quoi l'attaque passe) :

- **B1** — Dépendances applicatives (npm/Deno, crates.io) et leurs registres
- **B2** — Fichiers de verrouillage (`package.json`, lockfiles, `Cargo.lock`)
- **B3** — Outillage de build (Rust, Node.js, Deno, Tauri CLI, images Docker)
- **B4** — Paquets système de l'environnement de compilation
- **B5** — Chaîne d'intégration continue (workflows GitHub Actions, actions
  tierces, secrets du dépôt)
- **B6** — Artefacts de release (binaires, installateurs MSI/NSIS/DEB/RPM/DMG)
- **B7** — Comptes des mainteneurs et droits sur le dépôt

## 2.2 Sources de risque

| Source | Motivation type | Capacité | Référence |
|--------|-----------------|----------|-----------|
| Acteur étatique / APT | espionnage, prépositionnement | très élevée, patiente | SUNBURST, XZ Utils |
| Cybercriminalité organisée | monétisation (vol de données, rançon, cryptominage) | élevée, opportuniste | event-stream (2018), ua-parser-js (2021) |
| Attaquant opportuniste | accès aux secrets CI, rebond | moyenne, automatisée | tj-actions/changed-files (2025) |
| Contributeur malveillant | sabotage, porte dérobée ciblée | variable, favorisée par l'ouverture du projet | XZ Utils |

Il faut souligner un point propre à l'open source : pour un attaquant
patient, **un petit projet est une cible d'autant plus intéressante qu'il est
utilisé par des organisations sensibles** tout en étant maintenu par une
équipe réduite — c'est très exactement le profil exploité dans l'affaire XZ.

## 2.3 Événements redoutés

- **ER1 — Distribution d'un binaire piégé** aux utilisateurs via le canal
  officiel (gravité : critique — atteinte à V1, V2, V3).
- **ER2 — Exfiltration des secrets de la CI** (jetons, clés de signature),
  permettant ER1 par rebond (gravité : majeure).
- **ER3 — Introduction durable d'une dépendance vulnérable ou malveillante**
  dans le produit (gravité : majeure).
- **ER4 — Impossibilité de prouver l'intégrité** d'une release après un doute
  ou une alerte, imposant un retrait complet par précaution (gravité :
  significative — atteinte à V2).

## 2.4 Cotation des risques (application non sécurisée)

Échelles : vraisemblance V1 (peu probable) à V4 (quasi certain) ;
gravité G1 (mineure) à G4 (critique).

| # | Risque | Chemin résumé | Vrais. | Grav. | Criticité |
|---|--------|---------------|:------:|:-----:|:---------:|
| R1 | Dépendance malveillante (typosquatting, version piégée, compte de mainteneur amont volé) | registre public → résolution automatique (`^x.y.z`) → exécution au build ou au runtime | V3 | G4 | **Critique** |
| R2 | Altération silencieuse d'un lockfile | PR « bump deps » de centaines de lignes non relues → source ou version compromise | V3 | G3 | **Élevée** |
| R3 | Dérive de l'environnement de build | outils installés « en dernière version », image Docker par tag flottant, paquets apt non épinglés | V4 | G2 | **Élevée** |
| R4 | Injection pendant la compilation (scénario SUNBURST) | environnement de build compromis → binaire ≠ source, indétectable par revue | V1 | G4 | **Élevée** |
| R5 | Artefact publié ≠ dépôt (scénario XZ) | release fabriquée à la main par un mainteneur → supplément invisible dans l'artefact | V2 | G4 | **Critique** |
| R6 | Compromission d'une action CI tierce | action référencée par tag flottant → republication malveillante → secrets exfiltrés, artefacts altérés | V3 | G3 | **Élevée** |
| R7 | Vulnérabilité héritée d'une dépendance transitive | CVE dans une bibliothèque embarquée, inventaire inexistant → exposition ignorée | V4 | G2 | **Élevée** |
| R8 | Ingénierie sociale d'un mainteneur | contributeur « serviable » → droits étendus → charge glissée dans une zone non relue | V2 | G4 | **Critique** |
| R9 | Code hostile dans le webview (défense en profondeur) | dépendance frontend compromise → exfiltration réseau ou abus des commandes natives Tauri | V2 | G3 | **Élevée** |

## 2.5 Constat

Sur une application Tauri en configuration par défaut :

- **aucun des neuf risques n'est couvert** : les versions flottantes sont la
  norme npm, les builds ne sont pas reproductibles, les releases peuvent
  être téléversées manuellement, les actions CI sont référencées par tag,
  aucun inventaire de dépendances n'est publié ;
- quatre risques atteignent une criticité **critique ou élevée avec gravité
  G4** — c'est-à-dire des scénarios où l'institution qui déploie
  l'application installe un implant avec la bénédiction du canal officiel ;
- l'événement redouté ER4 est certain en cas d'incident : sans hash publié,
  sans provenance ni signature détachée, **il est impossible de blanchir les
  releases saines** — le projet entier doit être présumé compromis, comme ce
  fut le cas pour les utilisateurs de SolarWinds Orion.

Le chapitre 3 déroule les scénarios d'attaque correspondant à ces risques ;
le chapitre 4 présente les contre-mesures qui les neutralisent, telles que
déployées sur le projet démonstrateur.
