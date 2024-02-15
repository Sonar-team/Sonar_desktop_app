# Changelog
---

## [1.4.0] - 2024-02-15

### Corrections de bugs

- **filtre les Type d'ip** : 


---

## [1.3.3] - 2024-02-15

### Corrections de bugs

- **Liste des interfaces sur Windows** : Correction d'un problème où les noms des interfaces réseau étaient mal affichés sur Windows, apparaissant comme des UUID au lieu de noms conviviaux. Maintenant, les adresses MAC des interfaces sont utilisées pour permettre une identification plus aisée des interfaces réseau sur cette plateforme.

---

## [1.3.2] - 2024-02-13

### Nouvelles fonctionnalités

- **Ajout de code coverage** :  Implémentation d'outils de couverture de code pour garantir la qualité des suites de tests et identifier les parties du code non testées.

---

## [1.3.1] - 2024-02-13

### Nouvelles fonctionnalités

- **Ajout de la colonne Packet Size** : Une nouvelle colonne pour la taille des paquets a été ajoutée pour fournir plus de détails sur chaque paquet capturé. Cela permet une analyse plus approfondie du trafic réseau en offrant une visibilité sur la taille des paquets en plus de leurs métadonnées existantes.

---

#### Version 1.2.1

**Nouvelles fonctionnalités :**
- **info bulle avec ip sur les nodes** 

---

#### Version 1.1.1

**Nouvelles fonctionnalités :**
- **Enregistrement de la vue graphique au format SVG :** Il est désormais possible d'enregistrer la vue graphique de vos données réseau au format SVG. Cette fonctionnalité permet une préservation de haute qualité de vos visualisations pour une utilisation dans des rapports ou des présentations. Pour sauvegarder votre visualisation, sélectionnez l'option 'Sauvegarder en SVG' depuis la vue graphique.

- **Affichage des protocoles sur les arêtes :** Les visualisations graphiques ont été améliorées pour afficher les protocoles qui interagissent entre les adresses MAC. Cette mise à jour enrichit l'analyse en offrant une compréhension immédiate des types de communications se déroulant au sein de votre réseau, permettant ainsi d'identifier plus facilement les modèles de trafic et les éventuelles anomalies.

---

#### Version 1.1.0

**Nouvelles fonctionnalités :**
- **Sauvegarde au format Excel :** Vous pouvez maintenant sauvegarder vos données non seulement au format CSV, mais également au format Excel (XLSX). Cette option offre une plus grande flexibilité pour le traitement et l'analyse des données en dehors de l'application. Pour utiliser cette fonctionnalité, sélectionnez simplement l'option 'Sauvegarder en Excel' dans la section de sauvegarde des données.
- **Vue Graphique :** Une nouvelle fonctionnalité de visualisation graphique a été ajoutée pour vous permettre de voir les tendances et les analyses de vos données de manière plus intuitive. Accédez à des graphiques dynamiques et interactifs qui présentent vos données de réseau de manière visuelle, facilitant ainsi la compréhension et l'interprétation des informations complexes.

---

#### Version 1.0.1

**Nouvelles fonctionnalités :**
- **Gestion TCP/IP :** Sonar inclut désormais des capacités améliorées pour la gestion des protocoles TCP/IP. Cette fonctionnalité vise à améliorer l'aspect communication réseau du logiciel, en assurant un transfert de données plus robuste et efficace sur le réseau.

- **Sauvegarde en CSV :** Une nouvelle fonctionnalité a été ajoutée pour permettre aux utilisateurs d'exporter des données au format CSV (Comma-Separated Values). Cette fonctionnalité est particulièrement utile pour l'analyse de données et la création de rapports, car elle permet une manipulation facile des données et une intégration avec divers outils qui prennent en charge le CSV.

**Améliorations :**
- Optimisations générales des performances de l'application principale.
- Amélioration de l'interface utilisateur pour une meilleure facilité d'utilisation.

**Corrections de bugs :**
- Correction de bugs mineurs concernant des problèmes signalés dans la version précédente.

---

#### Version 1.0.0

**Première publication :**
- Implémentation des fonctionnalités de base de Sonar.
- Les fonctionnalités principales incluent des pratiques de développement Agile, une intégration avec GitHub pour le contrôle de version, et un accent sur Rust pour la performance et la fiabilité.
- Mise en place initiale des protocoles de test et d'assurance qualité.
- Mise en place de la documentation avec des fichiers markdown pour les README et les directives de contribution.
- Stratégie d'intégration front-end et back-end utilisant Tauri et Vue.js.
