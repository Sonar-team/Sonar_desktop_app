# Priorisation SONAR — bêta avancée vers 1.0 Pro

> Statut : référentiel actif
> Dernière revue : 20/07/2026
> Source de vérité opérationnelle : issues GitHub
> Sprint actif : [#165](https://github.com/Sonar-team/Sonar_desktop_app/issues/165)

## Grille de priorité

| Niveau | Signification | Règle |
|---|---|---|
| P0 | Résultat faux, perte de données, parcours cœur bloqué ou release non fiable | Bloque la sortie de bêta |
| P1 | Fonction ou garantie obligatoire pour une 1.0 Pro exploitable | Doit être fermé avant la Release Candidate, sauf acceptation écrite |
| P2 | Amélioration importante, mais compatible avec une première 1.0 limitée | Planifier après les P0/P1 |
| P3 | Dette, confort ou ticket à requalifier | Ne doit pas interrompre la trajectoire 1.0 |

La priorité mesure l'impact. L'ordre d'exécution tient aussi compte des
dépendances : un P1 peut commencer avant un P0 s'il prépare celui-ci, sans
devenir pour autant un blocage plus grave.

## Démarrage immédiat

| Ordre | Issue | Action |
|---:|---|---|
| 1 | [#166](https://github.com/Sonar-team/Sonar_desktop_app/issues/166) | Supprimer l'interblocage start/stop |
| 2 | [#167](https://github.com/Sonar-team/Sonar_desktop_app/issues/167) | Refuser imports vides et resets concurrents |
| 3 | [#154](https://github.com/Sonar-team/Sonar_desktop_app/issues/154) | Stabiliser l'identité d'actif avant le format de projet |
| 4 | [#151](https://github.com/Sonar-team/Sonar_desktop_app/issues/151) | Compléter corpus multi-DLT, malformé et fuzzing |
| 5 | [#168](https://github.com/Sonar-team/Sonar_desktop_app/issues/168) | Prouver l'exactitude PCAP → matrice avec TShark |
| 6 | [#88](https://github.com/Sonar-team/Sonar_desktop_app/issues/88) | Finir la validation Windows et frontend des chemins |
| 7 | [#150](https://github.com/Sonar-team/Sonar_desktop_app/issues/150) | Fermer catégories fines, export/projet et preuves restantes |

Deux chantiers peuvent commencer en parallèle :

- [#161](https://github.com/Sonar-team/Sonar_desktop_app/issues/161) pour les
  correctifs frontend exacts et isolés : double batch, double fichier et
  déverrouillage garanti ;
- [#162](https://github.com/Sonar-team/Sonar_desktop_app/issues/162) pour poser
  le workflow qualité commun à toutes les futures releases.

## Flux A — fidélité et sessions

`#166/#167 → #154 → #151/#168 → #159`

Les lots #87, #139, #142 et #158 sont livrés. Le lot cœur de #150 est livré,
mais l'issue reste ouverte pour ses garanties transverses.

- Le schéma de projet [#159](https://github.com/Sonar-team/Sonar_desktop_app/issues/159)
  ne doit pas être figé avant l'identité d'actif #154.
- La performance [#132](https://github.com/Sonar-team/Sonar_desktop_app/issues/132)
  vient après la comptabilité exacte ; aucun gain ne doit masquer une perte.

## Flux B — parcours produit

`#142 → #159 → #160 → #161 → #111/#102 → #144/#145 → #98`

- [#160](https://github.com/Sonar-team/Sonar_desktop_app/issues/160) absorbe la
  cohérence filtre/matrice/graphe/export attendue par #90.
- L'accessibilité #144 est appliquée pendant le développement, puis auditée
  avant la Release Candidate.
- La VAE #98 est préparée tôt mais exécutée sur une candidate complète.

## Flux C — distribution professionnelle

`#162/#136 → #143/#96 → #94 → #146 → #163 → Release Candidate`

- #162 peut démarrer immédiatement.
- #146 est la gate finale : application installée, réellement lancée et testée
  sur chaque plateforme.
- #138 reste ouverte mais P2 : Npcap demeure un prérequis externe détecté par
  NSIS, avec redirection vers la page officielle. La discussion Nmap est une
  seconde phase.

## Classement exhaustif

### P0 ouverts — bloquent la sortie de bêta

- #94, #136, #146, #150, #151, #154
- #159, #160, #161, #162, #166, #167
- #165 est l'issue de suivi du sprint P0

### P0 livrés récemment

- #87, #139, #142 et #158

### P1 — obligatoire avant 1.0 Pro

- #88, #89, #90, #92, #96, #97, #98, #102
- #109, #111, #132, #143, #144, #145, #163, #168

### P2 — important après la stabilisation

- #101, #107, #112, #119, #120, #121, #138, #156, #164

Pour #164, l'inventaire minimal et le rapport attestable doivent être extraits
en issues P1 ; baseline/diff avancé et SFMS complet restent P2.

### P3 — ne bloque pas la 1.0

- #91, #118, #124, #133

#118 doit être revalidée puis fermée si le chemin MSI est toujours
volontairement désactivé. #133 attend la stabilisation des API par #150/#154.

## Quick wins autorisés

- rendre la fixture absente explicitement visible dans #151 ;
- tester espaces et Unicode dans #88 ;
- supprimer le double traitement des batchs dans #161 ;
- dédupliquer les chemins avant import dans #161 ;
- garantir le `finally` de l'import dans #161 ;
- supprimer routes/tests hérités évidents dans #145 ;
- retirer les traces console non nécessaires de #112.

Chaque quick win doit avoir un test et ne doit pas retarder une tâche P0.

## Gates de passage

### Vers le sprint produit

- paquets intégralement classés ;
- imports/captures atomiques ;
- arrêt sans perte silencieuse ;
- contrat IPC généré ;
- identité d'actif stable.

### Vers la Release Candidate

- aucun P0 ouvert ;
- tous les P1 fermés ou dérogation écrite et limitée ;
- projets récupérables et matrice fonctionnelle ;
- E2E des parcours cœur sur chaque OS ;
- installateurs signés/notarifiés ;
- documentation et VAE terminées.

### Vers la 1.0 Pro

- zéro anomalie P0/P1 issue de la VAE ;
- artefacts installés sur machines propres ;
- rapport de qualité et manifest vérifiables ;
- politique support, sécurité et limites publiée.
