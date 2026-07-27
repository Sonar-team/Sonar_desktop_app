# Position licences de SONAR

Document de référence pour les questions de licence du projet, issu des audits
des 10–11/07/2026 (issues
[#152](https://github.com/Sonar-team/Sonar_desktop_app/issues/152) et
[#138](https://github.com/Sonar-team/Sonar_desktop_app/issues/138)).

## Licence du projet : AGPL-3.0-only (#152)

SONAR est publié sous **GNU Affero General Public License version 3 uniquement**
(SPDX : `AGPL-3.0-only`).

L'audit relevait une divergence : l'avis de concession de `LICENSE.md` disait «
or (at your option) any later version » alors que les manifests
(`src-tauri/Cargo.toml`, workspace `sonar-rust`) déclaraient `AGPL-3.0-only`. La
forme « only » était le choix délibéré (répété dans chaque manifest) ; l'avis de
`LICENSE.md` était un boilerplate copié.

Alignement effectué le 11/07/2026 :

- `LICENSE.md` : avis corrigé en « version 3 only » avec l'identifiant SPDX ;
- `package.json` : champ `"license": "AGPL-3.0-only"` ajouté (repris par le SBOM
  frontend) ;
- manifests Cargo : déjà `AGPL-3.0-only`, inchangés.

Conséquence assumée : le projet ne pourra pas être relicencié automatiquement
vers une future AGPL v4 ; tout changement de licence restera une décision
explicite des ayants droit.

## Npcap : SDK de compilation et runtime externe (#138)

**Décision finale (20/07/2026).** Nmap/Npcap a confirmé par écrit que l'approche
recommandée pour SONAR consiste à compiler avec le SDK Npcap, tout en
distribuant l'application sans le runtime ni l'installeur Npcap. Le SDK normal
produit des applications compatibles avec les éditions gratuite et OEM.

1. Aucun installeur `npcap-*.exe` ou `winpcap-*.exe` ne doit être conservé dans
   le dépôt, intégré à un bundle, téléchargé ou exécuté par SONAR.
2. Les bibliothèques `.lib` du SDK restent présentes uniquement pour permettre
   la compilation Windows ; elles n'installent ni pilote ni runtime.
3. Npcap est un prérequis installé séparément par l'utilisateur depuis la
   [page officielle](https://npcap.com/#download), avec l'option **WinPcap
   API-compatible Mode** requise par SONAR.
4. Le hook NSIS vérifie le service, le mode compatible et les DLL nécessaires.
   Si le prérequis manque, il explique la situation et propose uniquement
   d'ouvrir la page officielle.
5. La CI refuse tout installeur Npcap/WinPcap ajouté au dépôt ou aux bundles. Un
   test automatisé nécessitant une installation silencieuse doit utiliser un
   environnement disposant des droits OEM adaptés, plutôt que de télécharger la version
   gratuite.

La version gratuite peut convenir à un utilisateur qui l'installe manuellement,
respecte la limite générale de cinq systèmes par organisation et n'a besoin ni
d'installation silencieuse ni de support commercial. Une organisation qui
dépasse ce seuil ou automatise le déploiement doit acquérir sa propre licence
[Npcap OEM Internal-use](https://npcap.com/oem/internal), avec une maintenance
active si elle veut bénéficier des mises à jour et du support commercial. Chaque
utilisateur ou organisation reste responsable du respect des conditions et
exceptions applicables.

Une licence Internal-use n'autorise pas SONAR à redistribuer Npcap. Toute future
intégration de l'installeur dans SONAR nécessiterait au préalable une licence
[Npcap OEM Redistribution](https://npcap.com/oem/redist), une autorisation
écrite, une validation juridique ainsi que les contrôles de hash, Authenticode
et notice de licence. Cette option n'est pas retenue.

Les tarifs de Npcap OEM dépendent du palier et peuvent évoluer. Les conditions et
tarifs officiels prévalent sur toute estimation historique. La décision de licence
courante est close dans #138.
L'assainissement des releases historiques qui contiennent un installeur reste
suivi dans #169 ; les tests Windows avec Npcap présent, absent ou incompatible
restent suivis dans #146.

**Décision du 27/07/2026 (#169)** : sur instruction du mainteneur, les
39 installateurs NSIS `*-setup.exe` qui embarquaient et exécutaient Npcap
(releases `app-v3.9.6` à `v4.3.1`) ont été supprimés des releases GitHub, et
chaque release concernée porte une note de retrait renvoyant vers
npcap.com. Empreintes SHA-256 et métadonnées préservées dans
`security/npcap_remediation/` (manifeste `assets_supprimes_2026-07-27.csv`).
Les installateurs de l'ère MSI, vérifiés sains (fragment WiX non câblé),
sont conservés.

**Complément du 27/07/2026** : conformément à la réponse reçue de Nmap par
mail et sur instruction du mainteneur, les **119 tags** dont l'arbre
contenait un `npcap-*.exe` ont été supprimés du dépôt distant et local
(manifeste `tags_supprimes_2026-07-27.csv` : tag, SHA visé, date). Leurs
archives source automatiques ne sont plus servies (404 vérifié) ; les pages
de releases et leurs assets sains restent en place. Résidu connu : les
commits historiques restent joignables par SHA sur `main`, une purge
complète exigerait une réécriture d'historique — non réalisée à ce stade.

## SBOM (#137)

- SBOM backend : `cargo cyclonedx` sur `src-tauri` (inclut désormais
  `sonar-flows-core` depuis sa version crates.io exacte et vendored).
- SBOM frontend : généré depuis `deno.lock` par
  `script/ci/generate-frontend-sbom-from-lock.sh` (Syft ne cataloguait pas
  `deno.lock` et produisait un SBOM vide). Reproductible : ni timestamp ni
  serialNumber, tri stable par purl.
