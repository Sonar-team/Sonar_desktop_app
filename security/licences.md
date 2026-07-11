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

## Npcap : pas de redistribution publique sans licence OEM (#138)

**Constat.** L'installeur Windows embarque `npcap-1.88.exe` et le lance
avec élévation (hook NSIS). La version gratuite de Npcap n'autorise pas la
redistribution externe sans licence OEM : <https://npcap.com/oem/redist>.

**Position (11/07/2026).**

1. **Aucune release Windows publique ne doit être publiée** tant que l'une
   de ces conditions n'est pas remplie :
   - acquisition d'une licence **Npcap OEM** (redistribution autorisée) ;
   - **retrait** de l'installeur du bundle, avec documentation d'une
     installation préalable de Npcap par l'utilisateur ;
   - distribution **restreinte** (interne, hors des cas visés par la
     clause de redistribution — à faire valider juridiquement).
2. En attendant la décision, l'embarquement est **limité au seul build
   Windows** (`src-tauri/tauri.windows.conf.json`) : les paquets Linux
   (deb/rpm) et macOS (dmg) ne redistribuent plus l'EXE Npcap — ils
   l'embarquaient jusqu'ici sans raison.
3. Durcissements à câbler dans la CI (suivis dans #135/#136) si la
   redistribution est retenue :
   - vérification d'un hash attendu de `npcap-1.88.exe` au build ;
   - validation Authenticode de l'installeur ;
   - notice de licence Npcap visible dans l'installeur NSIS.

**Décision finale (OEM / retrait / distribution restreinte) : à trancher
par le mainteneur avant le prochain tag Windows public.**

## SBOM (#137)

- SBOM backend : `cargo cyclonedx` sur `src-tauri` (inclut désormais
  `sonar-flows-core` via le chemin workspace).
- SBOM frontend : généré depuis `deno.lock` par
  `script/ci/generate-frontend-sbom-from-lock.sh` (Syft ne cataloguait pas
  `deno.lock` et produisait un SBOM vide). Reproductible : ni timestamp ni
  serialNumber, tri stable par purl.
