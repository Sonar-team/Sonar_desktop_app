# Inventaire Npcap des releases historiques — 27/07/2026

Réalisé pour l'issue [#169](https://github.com/Sonar-team/Sonar_desktop_app/issues/169)
(phases non destructives : inventaire, classification, préservation des
métadonnées).

**Mise à jour du 27/07/2026** : sur instruction du mainteneur, les 39 assets
`*-setup.exe` NSIS embarquant Npcap ont été supprimés des releases GitHub
(manifeste avec SHA-256 : `assets_supprimes_2026-07-27.csv`) et chaque
release concernée porte une note de retrait. Vérifié après traitement :
plus aucun asset « setup » via l'API et 404 sur les URLs de téléchargement.
**Seconde mise à jour du 27/07/2026** : conformément à la réponse de Nmap
reçue par mail et sur instruction du mainteneur, les **119 tags** concernés
ont été supprimés (remote + local, manifeste
`tags_supprimes_2026-07-27.csv`). Vérifié : plus aucun des 119 tags sur le
remote (57 tags sains restants), archives source → 404, archives des tags
sains (ex. v4.8.3) → 200, releases et assets sains intacts. Résidu : les
commits restent joignables par SHA dans l'historique de `main` ; une purge
totale demanderait `git filter-repo` + force-push (non réalisée).

## Méthode

- Parcours de tous les tags Git : `git ls-tree -r` à la recherche de
  `npcap-*.exe`, relevé du blob et calcul du SHA-256.
- Classification par tag : installeur déclaré dans `tauri.conf.json` /
  `tauri.windows.conf.json` (ère NSIS, ressource embarquée) ou seulement
  présent dans l'arbre avec un fragment WiX non câblé (`fragmentPaths`
  absent de la conf — ère MSI).
- Export complet des métadonnées de releases via l'API GitHub.
- Vérification par sondage : téléchargement et inspection (`7z l`) de
  4 installateurs publiés couvrant les deux ères.

## Résultats

### Binaires Npcap suivis dans Git (5 versions, 5 blobs)

| Fichier | Blob (SHA-1) | Taille | SHA-256 |
|---|---|---|---|
| npcap-1.80.exe | 691a5540cdba681a52fd375ffb876c3b2ab2c79e | 1 202 472 | ac4f26d7d9f994d6f04141b2266f02682def51af63c09c96a7268552c94a6535 |
| npcap-1.85.exe | 791020dcbff25235b6c5eac32f28bdb32604669e | 1 317 016 | 4038de8dfdd254d21ebc3269c21f138695a07de2d2fbe57181ce4c0da87531c7 |
| npcap-1.86.exe | d04b0408c560473352b87121b416802d8f445582 | 1 313 480 | a4f7ab0c5850b819dc7a131213c16a9deaf8d32bceed7131fea38740ea788503 |
| npcap-1.87.exe | ebebbe201f24ddc9a1697d9165ad36a09ef76bb6 | 1 318 072 | 142c234f09a9618d7cdc2c37f607d6fc06615fad5581f728b60d4ad659f906bd |
| npcap-1.88.exe | f4d3b3bd7d53e2a4d041c1357c12bb604ae21a21 | 1 320 424 | a2f4ec1e5ea353ff67efd24b2ebf081ba44532410fae8d5e146af0310aa4f56b |

### Périmètre

- **119 tags** contiennent un `npcap-*.exe` dans leur arbre (de `app-v1.2.4`
  à `v4.3.1`) — plus large que l'audit initial du 20/07 qui partait de
  `app-v2.1.0`.
- **89 releases publiques** pointent vers ces tags ; leurs **archives source
  automatiques** (`.zip`/`.tar.gz` générées par GitHub) distribuent donc les
  octets de l'installeur Npcap.
- **45 releases** relèvent de l'ère « ressource NSIS » (Npcap déclaré dans la
  conf Tauri et exécuté par `hooks.nsh`), dont **39 publient un
  `*-setup.exe` NSIS** : ces installateurs SONAR embarquent et exécutent
  Npcap. Confirmé par sondage : `sonar_4.3.1_x64-setup.exe` contient
  `windows/npcap-1.88.exe`.
- **44 releases** relèvent de l'ère « fragment WiX dormant » : le fragment
  `npcap.wxs` existait dans l'arbre mais n'était pas référencé par
  `fragmentPaths` — les MSI construits ne l'embarquent pas. Confirmé par
  sondage : `sonar_2.1.0_x64-setup.exe` et `sonar_2.1.0_x64_fr-FR.msi`
  ne contiennent aucune trace Npcap. Pour ces releases, seule l'archive
  source est concernée.
- Le MSI de v4.3.1 (`sonar_4.3.1_x64_fr-FR.msi`) ne contient pas Npcap :
  l'embarquement de l'ère récente passe par NSIS uniquement.
- Les tags `v4.4.0` et suivants sont propres (retrait effectif par
  `a877cb4d` le 12/07/2026).

### Fichiers préservés dans ce dossier

- `inventaire_tags_releases_2026-07-27.csv` — les 119 tags avec
  classification, blob, SHA-256, URL de release et assets Windows.
- `releases_concernees_meta_2026-07-27.json` — métadonnées API complètes
  des 89 releases concernées (ids, assets, tailles, dates, URLs), pour
  audit et restauration documentaire après tout traitement.

## Ce qui distribue effectivement Npcap aujourd'hui

1. Les archives source automatiques des 119 tags (GitHub les génère tant que
   le tag existe) ;
2. Les 39 assets `*-setup.exe` NSIS des releases de l'ère « ressource ».

## Brouillon de demande à Nmap (à envoyer par un mainteneur)

> Subject: Historical redistribution of Npcap installers in SONAR releases — remediation guidance
>
> Hello,
>
> We maintain SONAR (https://github.com/Sonar-team/Sonar_desktop_app), an
> open-source passive network mapping tool. During a license review we found
> that historical releases of our project redistributed the free Npcap
> installer without an OEM license, which we understand is not permitted:
>
> - 119 Git tags (roughly 2024–2026) tracked a copy of `npcap-*.exe`
>   (versions 1.80, 1.85, 1.86, 1.87, 1.88) in the source tree, so the
>   auto-generated source archives of those tags still serve those bytes;
> - 39 published Windows NSIS installers embed and silently run the Npcap
>   installer.
>
> Current releases (v4.4.0 and later) no longer ship or embed Npcap: our
> installer now directs users to https://npcap.com/#download and we build
> against the SDK only.
>
> We would like your guidance on the expected handling of the historical
> artifacts: is deleting the affected release assets and tags (or rewriting
> history to purge the blobs) required, or is another remediation acceptable?
> We are ready to proceed as soon as you confirm the expected course of
> action.
>
> Thank you,

## Options pour la suite (décision à documenter après réponse de Nmap)

| Option | Effet | Coût / risque |
|---|---|---|
| Supprimer les 39 assets `*-setup.exe` | Stoppe la distribution des installateurs embarquant Npcap | Faible ; les métadonnées sont préservées ici |
| Supprimer les 119 tags (et releases associées) | Stoppe aussi les archives source | Casse les liens publics et les références d'audit ; irréversible |
| Réécriture d'historique (`git filter-repo` sur les 5 blobs) + force-push | Purge les octets du dépôt lui-même | Invalide tous les clones/forks, SHAs et attestations antérieurs ; le plus lourd |
| Conservation si Nmap l'autorise par écrit | Aucun | Nécessite la consigne écrite |

La décision retenue et sa date devront être ajoutées à
`security/licences.md` (critère de l'issue #169).
