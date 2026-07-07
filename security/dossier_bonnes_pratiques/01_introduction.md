# 1. Introduction — la menace sur la chaîne d'approvisionnement logicielle

## 1.1 Contexte : le logiciel open source dans les organisations

Les logiciels open source sont aujourd'hui présents dans la quasi-totalité
des systèmes d'information, y compris au sein des administrations. Cette
adoption s'accompagne d'un déplacement de la menace : plutôt que d'attaquer
frontalement une cible durcie, l'attaquant compromet **un maillon de la
chaîne qui produit ou distribue le logiciel** — une dépendance, un outil de
compilation, un serveur d'intégration continue, un canal de mise à jour.

Cette classe d'attaques, dite *supply chain*, présente un rapport
coût/efficacité redoutable pour l'attaquant :

- **une seule compromission touche tous les utilisateurs** du logiciel en
  aval, y compris les mieux défendus ;
- le code malveillant arrive **porté par la confiance du projet légitime** :
  signature valide, canal officiel, réputation établie ;
- les mécanismes de défense classiques (revue de code, antivirus, analyse
  du code source) sont **structurellement aveugles** à certains de ces
  vecteurs, comme le démontrent les deux cas ci-dessous.

Un projet open source hébergé sur GitHub ajoute des spécificités : le code
est public, les contributions extérieures sont bienvenues par principe, la
chaîne de build (GitHub Actions) exécute du code tiers, et les artefacts de
release sont le point de contact direct avec les utilisateurs.

## 1.2 Étude de cas n° 1 — SUNBURST / SolarWinds (2020) : compromettre la fabrication

En décembre 2020 est découverte l'une des compromissions les plus graves
jamais documentées. Des attaquants — attribués publiquement au service de
renseignement extérieur russe (SVR) — ont pénétré l'infrastructure de
l'éditeur SolarWinds et y ont déployé un implant (SUNSPOT) sur les
**serveurs de compilation** du produit Orion, une plateforme de supervision
utilisée par des dizaines de milliers d'organisations, dont plusieurs
agences fédérales américaines.

Le mode opératoire mérite l'attention :

- le **code source du dépôt n'a jamais été modifié** : l'implant substituait
  un fichier source à la volée, pendant la compilation, puis restaurait
  l'original — la fenêtre de modification ne durait que le temps du build ;
- le binaire troyanisé était ensuite **signé avec le certificat légitime**
  de l'éditeur et distribué par le canal de mise à jour officiel ;
- environ **18 000 organisations** ont installé la mise à jour compromise ;
- la porte dérobée est restée indétectée **plus de neuf mois**, jusqu'à sa
  découverte fortuite par un prestataire de sécurité lui-même victime.

**Enseignement.** La revue de code, l'audit du dépôt et la signature des
binaires n'offrent aucune protection lorsque c'est l'environnement de
fabrication qui est compromis. La seule parade structurelle est de rendre le
build **reproductible** — deux compilations indépendantes du même code
produisent le même binaire au bit près — et la **provenance vérifiable** :
tout écart entre le binaire publié et un binaire reconstruit par un tiers
devient alors une preuve de compromission.

## 1.3 Étude de cas n° 2 — XZ Utils (2024) : compromettre la confiance

En mars 2024, une porte dérobée (CVE-2024-3094, score CVSS 10.0) est
découverte dans XZ Utils, bibliothèque de compression présente dans la
quasi-totalité des distributions Linux. Le vecteur n'est pas technique mais
**humain** : un contributeur opérant sous le pseudonyme « Jia Tan » a
consacré **plus de deux ans** à gagner la confiance du mainteneur historique
— isolé et en difficulté personnelle — jusqu'à obtenir les droits de
mainteneur du projet.

Le mode opératoire combine trois dissimulations :

- le script d'activation de la porte dérobée n'était présent **que dans les
  archives de release** (*tarballs*) fabriquées à la main par le mainteneur,
  et **pas dans le dépôt git** : comparer le code source publié ne révélait
  rien ;
- la charge utile était enfouie dans des **fichiers de test binaires**,
  zone que ni les relecteurs ni les outils d'analyse n'examinent ;
- la cible finale était **OpenSSH** — l'accès distant de millions de
  serveurs — atteint par rebond via la chaîne de dépendances
  (systemd → liblzma).

La compromission n'a été découverte que par le hasard d'un ingénieur
intrigué par un ralentissement de 500 millisecondes des connexions SSH,
quelques semaines avant l'intégration de la version piégée dans les
versions stables de Debian et Fedora.

**Enseignement.** Un projet open source ne peut pas fonder sa sécurité sur
la confiance accordée à un individu. Il faut que les artefacts publiés
soient **produits mécaniquement par une chaîne auditable** à partir du dépôt
public, que chaque dépendance soit **auditée et figée**, et que les zones
mortes de la revue (fichiers binaires, scripts générés, gros diffs
mécaniques) soient **réduites par construction**.

## 1.4 Pourquoi une application Tauri est particulièrement exposée

Tauri est un framework de développement d'applications de bureau associant
un frontend web (HTML/JS, écosystème **npm**) et un backend **Rust**
(écosystème **crates.io**), assemblés par une chaîne de build faisant appel
à des **paquets système** (bibliothèques natives Linux/Windows/macOS). Une
application Tauri hérite donc simultanément de **trois surfaces d'attaque
supply chain** :

| Écosystème | Ordre de grandeur | Risques dominants |
|------------|-------------------|-------------------|
| npm / Deno (frontend) | dizaines à centaines de paquets | typosquatting, scripts d'installation, mainteneur compromis |
| crates.io (backend Rust) | centaines de crates transitives | version piégée, `build.rs` malveillant |
| Paquets système (build natif) | dizaines de bibliothèques | dérive de version, dépôt compromis |

S'y ajoutent la chaîne d'intégration continue (GitHub Actions, qui exécute
des actions tierces avec accès aux secrets du dépôt) et le canal de
distribution (releases GitHub : binaires et installateurs téléchargés
directement par les utilisateurs).

Le chapitre suivant analyse les risques pesant sur une telle application
**lorsqu'aucune mesure spécifique n'est prise** — situation qui correspond à
la configuration par défaut d'un projet créé sans démarche DevSecOps.
