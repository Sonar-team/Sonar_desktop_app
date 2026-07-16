# Position licences de SONAR

Document de référence pour les questions de licence du projet, issu des
audits des 10–11/07/2026 (issues [#152](https://github.com/Sonar-team/Sonar_desktop_app/issues/152)
et [#138](https://github.com/Sonar-team/Sonar_desktop_app/issues/138)).

## Licence du projet : AGPL-3.0-only (#152)

SONAR est publié sous **GNU Affero General Public License version 3
uniquement** (SPDX : `AGPL-3.0-only`).

L'audit relevait une divergence : l'avis de concession de `LICENSE.md`
disait « or (at your option) any later version » alors que les manifests
(`src-tauri/Cargo.toml`, workspace `sonar-rust`) déclaraient
`AGPL-3.0-only`. La forme « only » était le choix délibéré (répété dans
chaque manifest) ; l'avis de `LICENSE.md` était un boilerplate copié.

Alignement effectué le 11/07/2026 :

- `LICENSE.md` : avis corrigé en « version 3 only » avec l'identifiant SPDX ;
- `package.json` : champ `"license": "AGPL-3.0-only"` ajouté (repris par le
  SBOM frontend) ;
- manifests Cargo : déjà `AGPL-3.0-only`, inchangés.

Conséquence assumée : le projet ne pourra pas être relicencié
automatiquement vers une future AGPL v4 ; tout changement de licence
restera une décision explicite des ayants droit.

## Npcap : redistribution retirée, suivi maintenu (#138)

La version gratuite de Npcap n'autorise normalement pas sa redistribution
externe avec un autre produit sans accord ou licence OEM :
<https://npcap.com/oem/redist>.

**Décision provisoire (12/07/2026).**

1. L'installeur `npcap-*.exe` est retiré du dépôt et de tous les bundles SONAR.
   Les bibliothèques `.lib` du SDK restent présentes uniquement pour permettre
   la compilation Windows ; elles n'installent ni pilote ni runtime.
2. Npcap devient un prérequis installé séparément par l'utilisateur depuis
   <https://npcap.com/#download>, avec l'option **WinPcap API-compatible Mode**.
3. Le bundle Windows est temporairement limité à NSIS : son hook vérifie le
   service Npcap, l'option de compatibilité et les DLL nécessaires. Si le
   prérequis manque, il explique la situation et propose d'ouvrir uniquement
   la page officielle.
4. La CI refuse la présence d'un installeur Npcap/WinPcap dans le dépôt ou le
   bundle. Elle ne télécharge pas non plus Npcap sur ses runners tant que ce
   cas d'usage n'a pas été clarifié.

L'issue #138 **reste ouverte**. Le projet demandera à Nmap s'il existe une
autorisation ou une formule adaptée à un projet open source non commercial.
Npcap ne devra être réintroduit dans un bundle qu'après obtention d'un droit
écrit explicite. Le traitement des anciens tags/releases contenant déjà
l'installeur reste également suivi dans cette issue.

## SBOM (#137)

- SBOM backend : `cargo cyclonedx` sur `src-tauri` (inclut désormais
  `sonar-flows-core` depuis sa version crates.io exacte et vendored).
- SBOM frontend : généré depuis `deno.lock` par
  `script/ci/generate-frontend-sbom-from-lock.sh` (Syft ne cataloguait pas
  `deno.lock` et produisait un SBOM vide). Reproductible : ni timestamp ni
  serialNumber, tri stable par purl.
