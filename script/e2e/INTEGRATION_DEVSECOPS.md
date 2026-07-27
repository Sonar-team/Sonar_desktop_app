# Intégration DevSecOps du gate E2E signé SONAR

Ce runbook décrit le circuit de confiance utilisé pour installer et tester sur
une VM Linux exactement le paquet SONAR signé et publié. Il concerne les
pipelines de tags `vX.Y.Z`.

Le gate est composé de deux chaînes complémentaires :

```text
GitHub Actions publish
  └─ build → hashes → signatures Sigstore → kit hors ligne signé → release
                                                                  │
GitLab CI e2e:signed-linux-vm                                     │
  └─ attend la release publique ◄─────────────────────────────────┘
     → vérifie le kit et le .deb
     → Ansible installe ce .deb sur la VM
     → Xvfb exécute les parcours E2E
     → GitLab conserve les preuves
```

Le `.deb` reconstruit par GitLab n'est jamais substitué au paquet signé. Le
gate télécharge l'artefact de la release GitHub correspondant exactement à
`CI_COMMIT_TAG` et `CI_COMMIT_SHA`.

## Composants versionnés

| Fichier                                                                          | Rôle                                                                    |
| -------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| [`create-offline-verification-kit.sh`](../ci/create-offline-verification-kit.sh) | Assemble et signe le kit durant la release                              |
| [`verify-offline-release-kit.sh`](../ci/verify-offline-release-kit.sh)           | Vérifie hors ligne l'archive, l'identité, les hashes et chaque artefact |
| [`verify-offline-release-kit.ps1`](../ci/verify-offline-release-kit.ps1)         | Même vérification sur un poste Windows sans Bash                        |
| [`download-signed-release-for-e2e.sh`](../ci/download-signed-release-for-e2e.sh) | Attend et télécharge le kit signé exact                                 |
| [`sigstore-trusted-root.json`](../../security/sigstore-trusted-root.json)        | Racine Sigstore figée et revue dans le dépôt                            |
| [`e2e-vm.yml`](../../.gitlab/ci/e2e-vm.yml)                                      | Job GitLab bloquant sur les tags                                        |
| [`sonar-e2e.yml`](../../ci/ansible/sonar-e2e.yml)                                | Installation et exécution distante avec Ansible                         |
| [`run-sonar-x11-e2e.sh`](./run-sonar-x11-e2e.sh)                                 | Parcours fonctionnels du vrai binaire Tauri                             |

La version de Cosign est centralisée dans `config/build-versions.env`. Le
workflow et l'image du runner GitLab doivent utiliser cette même version.

## Contenu du kit hors ligne

À partir de la prochaine release produite avec ce mécanisme, chaque plateforme
publie une archive :

```text
sonar-offline-kit-<version>-<plateforme>.tar.gz
<plateforme>-sonar-offline-kit-<version>-<plateforme>.tar.gz.sigstore.json
```

L'archive contient :

- les binaires et installateurs de la plateforme ;
- la signature Sigstore individuelle de chaque artefact ;
- le manifeste de hashes signé de la release ;
- un manifeste `SHA256SUMS` couvrant le contenu du kit ;
- la racine Sigstore utilisée par la version ;
- le binaire Cosign vérifié par l'action d'installation épinglée ;
- les scripts de vérification Bash et PowerShell ;
- pour Linux, `dist/`, le harness X11 et les fixtures E2E du même build.

Le kit est lui-même signé et reçoit une attestation de provenance. Son SHA-256
est ajouté au corps de la release.

La release `v4.8.3`, antérieure à cette intégration, possède des signatures
individuelles mais pas encore l'archive hors ligne. Le premier kit sera publié
par le prochain tag construit après l'intégration de ces changements.

## Amorçage de la confiance hors ligne

Un kit ne peut pas se déclarer lui-même fiable. Avant l'isolement du poste ou
de l'image de VM, provisionner séparément :

1. Cosign dans la version déclarée par `COSIGN_VERSION` ;
2. `security/sigstore-trusted-root.json` ;
3. `script/ci/verify-offline-release-kit.sh` ;
4. le tag exact que l'opérateur est autorisé à installer.

Ces trois fichiers doivent venir de l'image de confiance ou d'un support
d'administration contrôlé. Le vérificateur n'exécute volontairement pas le
Cosign embarqué dans une archive encore non vérifiée.

La racine actuellement versionnée a été générée avec Cosign 3.0.6 depuis les
services par défaut Sigstore. Son SHA-256 est figé dans
`SIGSTORE_TRUSTED_ROOT_SHA256`, dans `config/build-versions.env`, et contrôlé
par la CI :

```bash
sha256sum security/sigstore-trusted-root.json
```

Lors d'une rotation Sigstore, générer une nouvelle racine dans un fichier
temporaire, examiner le diff, vérifier une release connue, puis faire valider le
changement par une pull request :

```bash
cosign trusted-root create \
  --with-default-services \
  --out trusted-root.candidate.json
```

Ne jamais remplacer automatiquement la racine pendant une installation sur un
poste isolé.

## Vérification manuelle hors ligne

Sur une station d'administration connectée, télécharger l'archive et son
bundle, puis les transférer ensemble avec le tag attendu. Sur le poste isolé :

```bash
./verify-offline-release-kit.sh \
  --archive sonar-offline-kit-4.9.0-ubuntu-22.04.tar.gz \
  --bundle ubuntu-22.04-sonar-offline-kit-4.9.0-ubuntu-22.04.tar.gz.sigstore.json \
  --trusted-root /opt/sonar-trust/sigstore-trusted-root.json \
  --expected-tag v4.9.0 \
  --expected-commit <SHA-1 Git de 40 caractères> \
  --platform ubuntu-22.04 \
  --cosign /opt/sonar-trust/cosign \
  --extract-to /var/tmp/sonar-verified-v4.9.0
```

Sur un poste Windows isolé, la commande équivalente est :

```powershell
.\verify-offline-release-kit.ps1 `
  -Archive .\sonar-offline-kit-4.9.0-windows-2022.tar.gz `
  -Bundle .\windows-2022-sonar-offline-kit-4.9.0-windows-2022.tar.gz.sigstore.json `
  -TrustedRoot C:\ProgramData\SONAR\Trust\sigstore-trusted-root.json `
  -ExpectedTag v4.9.0 `
  -ExpectedCommit <SHA-1 Git de 40 caractères> `
  -Platform windows-2022 `
  -Cosign C:\ProgramData\SONAR\Trust\cosign.exe `
  -ExtractTo C:\ProgramData\SONAR\Verified\v4.9.0
```

La commande ne contacte ni Fulcio ni Rekor. Le bundle et la racine fournissent
le matériel de vérification. Elle contrôle successivement :

1. la signature de l'archive avant son extraction ;
2. l'identité exacte du workflow `publish.yml` et du tag demandé ;
3. le commit Git exact, lorsqu'il est fourni par la CI ou l'opérateur ;
4. l'absence de chemin dangereux ou de lien symbolique dans l'archive ;
5. tous les hashes internes ;
6. la signature du manifeste de release ;
7. la signature et le SHA-256 de chaque artefact.

Le succès se termine par :

```text
SONAR_OFFLINE_KIT_VERIFIED=/chemin/du/kit
SONAR_SIGNED_DEB=/chemin/du/paquet.deb
```

Une version ancienne mais correctement signée reste techniquement valide. Le
paramètre obligatoire `--expected-tag` empêche qu'elle soit installée par
substitution lorsque l'opérateur attend une autre version. Le commit attendu
doit venir de la fiche de release ou du canal d'autorisation, jamais du kit à
contrôler. GitLab fournit automatiquement `CI_COMMIT_SHA`.

## Activation dans GitLab

Le dépôt inclut déjà `.gitlab/ci/e2e-vm.yml` et le stage `e2e`. Le job reste
absent tant que la variable suivante n'est pas définie :

```text
SONAR_E2E_VM_ENABLED=true
```

Le motif de tags `v*` doit être protégé dans GitLab et GitHub afin que seuls les
mainteneurs de release autorisés puissent déclencher cette chaîne.

Configurer ensuite ces variables CI/CD protégées :

| Variable                      | Type                      | Contenu                                                                               |
| ----------------------------- | ------------------------- | ------------------------------------------------------------------------------------- |
| `SONAR_E2E_RUNNER_IMAGE`      | Variable                  | Image immuable contenant Bash, `gh`, Cosign, Ansible, SSH, `jq`, `tar` et `sha256sum` |
| `SONAR_GITHUB_TOKEN`          | Variable masquée/protégée | Jeton GitHub en lecture seule des releases                                            |
| `SONAR_SIGSTORE_TRUSTED_ROOT` | File protégée             | Copie approuvée de la racine Sigstore versionnée                                      |
| `SONAR_E2E_INVENTORY`         | File protégée             | Inventaire Ansible du groupe `sonar_e2e`                                              |
| `SONAR_E2E_SSH_KEY`           | File protégée             | Clé privée SSH du compte d'automatisation                                             |
| `SONAR_E2E_KNOWN_HOSTS`       | File protégée             | Empreinte SSH vérifiée de la VM                                                       |

L'image `SONAR_E2E_RUNNER_IMAGE` doit être épinglée par digest et contenir la
version exacte de Cosign déclarée dans `config/build-versions.env`. Le jeton
GitHub n'a besoin que de lire les releases et ne doit pas permettre leur
modification. Le job exige aussi que la racine fournie par la variable protégée
soit identique à celle revue dans le dépôt.

Exemple minimal d'inventaire :

```ini
[sonar_e2e]
sonar-e2e-linux ansible_host=192.0.2.10 ansible_user=sonar-ci
```

L'adresse est réservée à la documentation. Ne jamais désactiver la vérification
de la clé d'hôte SSH. Le compte `sonar-ci` peut utiliser `become` uniquement
pour l'installation des dépendances et du paquet.

## Déroulement du job GitLab

Le job `e2e:signed-linux-vm` :

1. attend au maximum deux heures que la release GitHub du même tag devienne
   publique ;
2. télécharge uniquement le kit Linux et son bundle Sigstore ;
3. vérifie le kit avec la racine versionnée, l'identité, le tag et le
   `CI_COMMIT_SHA` exacts ;
4. transmet à Ansible le chemin du `.deb` déjà vérifié ;
5. recalcule son SHA-256 après le transfert SSH ;
6. installe `/usr/bin/sonar` depuis ce paquet ;
7. exécute les E2E avec Xvfb et un compte non privilégié ;
8. récupère les preuves avant de propager un éventuel échec.

Le job ne passe jamais `--build` au harness. Il teste le binaire du paquet
signé, pas une reconstruction effectuée sur la VM.

Si GitHub ne publie pas la release, si l'identité ne correspond pas, si une
signature est absente ou si Ansible échoue, le job GitLab échoue aussi.

## VM de test

La cible de référence est une VM Debian ou Ubuntu amd64 propre, avec au moins
2 vCPU, 4 Gio de RAM et 5 Gio libres. Le playbook installe :

```text
build-essential  pkg-config  libx11-dev  libxtst-dev
xvfb  openbox  x11-utils  wmctrl  xclip
imagemagick  dbus-x11  file
```

SONAR et Xvfb s'exécutent avec l'utilisateur système `sonar-e2e`, jamais avec
`root`. Xvfb n'écoute pas sur TCP. Les données, caches et configurations sont
isolés par le harness avec `XDG_*`.

Une VM éphémère est recommandée. Si plusieurs pipelines partagent une VM, le
`resource_group` GitLab empêche leur exécution simultanée, mais la VM doit
quand même être restaurée entre les campagnes.

## Preuves conservées

GitLab publie avec `artifacts: when: always` :

- `verification.log`, qui prouve la validation Sigstore du kit ;
- `summary.txt` et `runtime.log` ;
- les captures des écrans et dialogues natifs ;
- les exports PNG, matrice, labels et logs ;
- `failure.png` lorsqu'une erreur apparaît après l'ouverture de SONAR ;
- les répertoires XDG isolés de la session.

Limiter l'accès et la rétention de ces artefacts : ils peuvent contenir les
données importées pendant la recette. Les fixtures versionnées ne doivent
contenir aucune donnée de production.

## Capture réseau en direct

Le gate standard n'accorde aucune capability réseau. L'import PCAP valide le
pipeline fonctionnel sans privilège supplémentaire.

Pour une campagne spécialisée sur une VM éphémère :

```bash
setcap cap_net_raw,cap_net_admin=eip /usr/bin/sonar
```

puis ajouter `--live-capture` au harness. Ne jamais remplacer cette procédure
par l'exécution complète de SONAR avec `sudo`.

## Critères d'acceptation

L'intégration est opérationnelle lorsque :

1. le job ne s'exécute que pour un tag `vX.Y.Z` explicitement attendu ;
2. le kit et le paquet sont signés par le workflow SONAR exact ;
3. le tag du certificat correspond à `CI_COMMIT_TAG` ;
4. la racine et Cosign viennent de l'image de confiance ;
5. Ansible installe le `.deb` extrait du kit vérifié ;
6. tous les parcours E2E réussissent sur une VM propre ;
7. les preuves sont récupérées même lorsque le scénario échoue ;
8. le job n'utilise ni `allow_failure` ni `--keep-open` ;
9. aucun privilège de capture n'est présent dans le gate standard.

## Diagnostic rapide

| Symptôme                       | Vérification                                                            |
| ------------------------------ | ----------------------------------------------------------------------- |
| Release signée introuvable     | Contrôler le tag, le workflow GitHub `publish` et le jeton de lecture   |
| Identité ou signature invalide | Vérifier le tag attendu, la racine versionnée et le bundle téléchargé   |
| Version Cosign différente      | Reconstruire l'image du runner avec `COSIGN_VERSION`                    |
| Racine embarquée différente    | Examiner une rotation Sigstore ou une archive altérée                   |
| Aucun `.deb` unique            | Vérifier le contenu du kit Linux et la collecte des bundles             |
| `Xvfb ne répond pas`           | Lire `xvfb.log` et vérifier les numéros `DISPLAY` 90 à 119              |
| Dialogue natif introuvable     | Vérifier `dbus-x11`, GTK, `xclip` et les captures du dialogue           |
| Asset absent                   | Vérifier `validation/dist/` dans le kit signé                           |
| Échec fonctionnel              | Lire `runtime.log`, `summary.txt` et la capture correspondant à l'étape |
