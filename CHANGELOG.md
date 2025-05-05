# Changelog
## [2.2.1] - 2025-05-05

## Fix 

* Compatibilité mac os

## [2.2.1] - 2025-05-05

### ✨ Fonctionnalités

* Ajout de la fonctionnalité "stop record".
* Ajout de l’icône `stop.svg` dans `src/assets`.
* Compatibilité améliorée entre Windows 11 et Ubuntu pour les timestamps des paquets réseau (`tv_sec`, `tv_usec`).
* Ajout d'une gestion conditionnelle multiplateforme avec `#[cfg(target_os = "...")]` pour la conversion des timestamps.

### 🛠 Corrections

* Correction d’un bug de compilation sous Windows 11 (mismatch de types `i32` vs `i64`).
* Le fichier `.gitignore` n’ignore plus les `.svg` du dossier `src/assets`.

### 🎨 Interface

* Amélioration de la top bar.
* Amélioration de la status bar.

### 🔧 Technique

* Tag `app-v2.2.0` ajouté à `main` après merge.
* Nettoyage de warnings (`unused import: info`) dans le module `commandes`.
* Suppression de la page de nommage de fichier au démarrage de SONAR. La discussion est ouverte pour une réintégration éventuelle au moment de la sauvegarde.
* Retrait de la fonctionnalité d'automatisation de la sauvegarde : cette fonction n'a jamais été utilisée et ne répondait à aucun besoin identifié jusqu'à présent.


## [1.15.0] - 2024-11-07

### NEW

- Intégration de la structure `PacketKey` pour distinguer les paquets sans
  considérer leur taille (`packet_size`) dans la clé, permettant une meilleure
  gestion des doublons et l'accumulation des tailles des paquets dans
  `PacketStats`.
- Ajout de la fonctionnalité de conversion de `PacketKey` en `PacketInfos` pour
  assurer la compatibilité avec les méthodes existantes nécessitant
  `PacketInfos`.

### FIX

- Résolution d'un problème de type qui empêchait l'exportation correcte des
  données de la matrice de paquets vers les fichiers CSV et Excel. Les méthodes
  d'enregistrement ont été adaptées pour utiliser `PacketKey` et `PacketStats`.
- Mise à jour des méthodes du front-end pour traiter correctement la structure
  de l'API, en tenant compte des nouvelles propriétés `infos` et `stats`. Cela
  garantit un affichage précis des données, y compris la taille totale des
  paquets et le nombre d'occurrences.

### IMPROVEMENT

- Refactoring de la méthode `get_matrice_data` pour une sérialisation plus
  claire et un traitement efficace des données.
- Amélioration des journaux de debug pour une meilleure traçabilité des paquets
  et de leur traitement dans l'application.

Cette version améliore la gestion des paquets avec des tailles différentes, la
stabilité et la clarté du code tout en offrant une meilleure expérience
utilisateur dans l'interface de visualisation.

---

## [1.14.1] - 2024-10-31

### FIX

Reload ... ! Résolution d'une erreur de parsing des paquets DNS qui provoquait
le blocage de l'application Sonar. Ce correctif améliore la stabilité et la
fiabilité de l'analyse des paquets DNS.

---

## [1.12.0] - 2024-07-04

### Ajouté

parse pacquet 7

---

## [1.11.1] - 2024-05-17

### Ajouté

Pipeline cicd pour raspberry pi

---

## [1.11.0] - 2024-05-02

### Ajouté

- Affichage des adresses IP publiques dans la vue graphique. Cela permet aux
  utilisateurs de visualiser les adresses IP directement depuis l'interface
  graphique de l'application.

### Modifié

- Modification de l'entrée pour la durée du relevé dans l'interface utilisateur
  de Vue.js pour accepter des valeurs jusqu'à 48 heures. Auparavant, l'entrée
  était limitée à 24 heures.
- Adaptation du type d'entrée pour la durée de relevé de `type="time"` à
  `type="text"` pour permettre la saisie manuelle de la durée en format
  "HH:MM:SS", permettant ainsi de saisir des durées supérieures à 24 heures.
- Mise à jour de la fonction `validateTime` pour valider les heures, les minutes
  et les secondes manuellement en utilisant une nouvelle logique qui supporte
  jusqu'à 48 heures.

### Corrigé

- Mise à jour de la fonction de récupération des informations système pour
  utiliser `whoami` via Rust et traiter la sortie pour obtenir spécifiquement le
  nom de la machine et la version du noyau.

---

## [1.9.0] - 2024-03-20

### Nouvelles fonctionnalités

- **Tableau avec vutify**:

---

## [1.8.0] - 2024-03-19

### Nouvelles fonctionnalités

- **Visualisation des Réseaux**: Implémentation d'une fonctionnalité de
  visualisation de réseaux améliorée, offrant des vues en courbes pour les
  connexions et un système de couleurs dynamique basé sur les types de
  protocoles.

---

## [1.7.0] - 2024-03-18

### Nouvelles fonctionnalités

- **Type d'IP** : Implémentation d'une nouvelle fonctionnalité permettant de
  déterminer le type d'une adresse IP (privée, APIPA, multicast, loopback,
  lien-local, ULA, publique ou inconnue) à partir d'une chaîne de caractères.
  Cette amélioration apporte une capacité critique à l'analyse et à la
  classification des adresses IP dans divers contextes de réseau.

### Améliorations

- **Détection des adresses APIPA** : Amélioration de la précision dans la
  détection des adresses IP APIPA (Automatic Private IP Addressing), permettant
  une identification plus fiable des appareils configurés automatiquement sans
  serveur DHCP.

- **Support Multicast IPv4** : Extension du support pour identifier les adresses
  multicast IPv4, facilitant la gestion et le filtrage des paquets destinés à
  des groupes d'écoute multicast.

- **Prise en charge IPv6** : Renforcement de la prise en charge des adresses
  IPv6 avec l'identification spécifique des adresses lien-local et ULA (Unique
  Local Address), améliorant ainsi la capacité à traiter et analyser le trafic
  IPv6 moderne.

### Corrections de bugs

- **Correction de la classification Loopback IPv6** : Résolution d'un problème
  où les adresses loopback IPv6 (`::1`) étaient incorrectement classifiées comme
  publiques, assurant désormais une identification correcte comme adresses
  loopback.

### Documentation

- **Mise à jour de la documentation** : Ajout de documentation pour la nouvelle
  fonctionnalité de type d'IP, incluant des exemples d'utilisation et des
  descriptions des différents types d'adresses IP supportés.

### Tests

- **Amélioration des tests unitaires** : Ajout et mise à jour de tests unitaires
  pour couvrir les nouvelles fonctionnalités et améliorations, notamment pour la
  détection des types d'adresses IP et la correction de la classification des
  adresses IPv6 loopback.

---
## [1.6.0] - 2024-02-26

### UI/UX

- Tableau des trames en temps réel présentant désormais 5 lignes vides par défaut pour une meilleure visibilité initiale.
- Ajustement de la hauteur des lignes du tableau des trames en temps réel pour améliorer la cohérence visuelle.

### Nouvelles fonctionnalités

- **Filtre ip** : Ajout d'un filtre pour IPv4 permettant une meilleure catégorisation et recherche des trames réseau.
- **rm lo on linux** :
---

## [1.5.0] - 2024-02-15

### Nouvelles fonctionnalités

- **colonne l7** :
- **documentation**

---

## [1.4.0] - 2024-02-15

### Corrections de bugs

---

## [1.3.3] - 2024-02-15

### Corrections de bugs

- **Liste des interfaces sur Windows** : Correction d'un problème où les noms
  des interfaces réseau étaient mal affichés sur Windows, apparaissant comme des
  UUID au lieu de noms conviviaux. Maintenant, les adresses MAC des interfaces
  sont utilisées pour permettre une identification plus aisée des interfaces
  réseau sur cette plateforme.

---

## [1.3.2] - 2024-02-13

### Nouvelles fonctionnalités

- **Ajout de code coverage** : Implémentation d'outils de couverture de code
  pour garantir la qualité des suites de tests et identifier les parties du code
  non testées.

---

## [1.3.1] - 2024-02-13

### Nouvelles fonctionnalités

- **Ajout de la colonne Packet Size** : Une nouvelle colonne pour la taille des
  paquets a été ajoutée pour fournir plus de détails sur chaque paquet capturé.
  Cela permet une analyse plus approfondie du trafic réseau en offrant une
  visibilité sur la taille des paquets en plus de leurs métadonnées existantes.

---

#### Version 1.2.1

**Nouvelles fonctionnalités :**

- **info bulle avec ip sur les nodes**

---

#### Version 1.1.1

**Nouvelles fonctionnalités :**

- **Enregistrement de la vue graphique au format SVG :** Il est désormais
  possible d'enregistrer la vue graphique de vos données réseau au format SVG.
  Cette fonctionnalité permet une préservation de haute qualité de vos
  visualisations pour une utilisation dans des rapports ou des présentations.
  Pour sauvegarder votre visualisation, sélectionnez l'option 'Sauvegarder en
  SVG' depuis la vue graphique.

- **Affichage des protocoles sur les arêtes :** Les visualisations graphiques
  ont été améliorées pour afficher les protocoles qui interagissent entre les
  adresses MAC. Cette mise à jour enrichit l'analyse en offrant une
  compréhension immédiate des types de communications se déroulant au sein de
  votre réseau, permettant ainsi d'identifier plus facilement les modèles de
  trafic et les éventuelles anomalies.

---

#### Version 1.1.0

**Nouvelles fonctionnalités :**

- **Sauvegarde au format Excel :** Vous pouvez maintenant sauvegarder vos
  données non seulement au format CSV, mais également au format Excel (XLSX).
  Cette option offre une plus grande flexibilité pour le traitement et l'analyse
  des données en dehors de l'application. Pour utiliser cette fonctionnalité,
  sélectionnez simplement l'option 'Sauvegarder en Excel' dans la section de
  sauvegarde des données.
- **Vue Graphique :** Une nouvelle fonctionnalité de visualisation graphique a
  été ajoutée pour vous permettre de voir les tendances et les analyses de vos
  données de manière plus intuitive. Accédez à des graphiques dynamiques et
  interactifs qui présentent vos données de réseau de manière visuelle,
  facilitant ainsi la compréhension et l'interprétation des informations
  complexes.

---

#### Version 1.0.1

**Nouvelles fonctionnalités :**

- **Gestion TCP/IP :** Sonar inclut désormais des capacités améliorées pour la
  gestion des protocoles TCP/IP. Cette fonctionnalité vise à améliorer l'aspect
  communication réseau du logiciel, en assurant un transfert de données plus
  robuste et efficace sur le réseau.

- **Sauvegarde en CSV :** Une nouvelle fonctionnalité a été ajoutée pour
  permettre aux utilisateurs d'exporter des données au format CSV
  (Comma-Separated Values). Cette fonctionnalité est particulièrement utile pour
  l'analyse de données et la création de rapports, car elle permet une
  manipulation facile des données et une intégration avec divers outils qui
  prennent en charge le CSV.

**Améliorations :**

- Optimisations générales des performances de l'application principale.
- Amélioration de l'interface utilisateur pour une meilleure facilité
  d'utilisation.

**Corrections de bugs :**

- Correction de bugs mineurs concernant des problèmes signalés dans la version
  précédente.

---

#### Version 1.0.0

**Première publication :**

- Implémentation des fonctionnalités de base de Sonar.
- Les fonctionnalités principales incluent des pratiques de développement Agile,
  une intégration avec GitHub pour le contrôle de version, et un accent sur Rust
  pour la performance et la fiabilité.
- Mise en place initiale des protocoles de test et d'assurance qualité.
- Mise en place de la documentation avec des fichiers markdown pour les README
  et les directives de contribution.
- Stratégie d'intégration front-end et back-end utilisant Tauri et Vue.js.
