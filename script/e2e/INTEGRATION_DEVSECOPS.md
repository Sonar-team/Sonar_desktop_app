# Intégration DevSecOps des tests E2E SONAR sur une VM Linux

Ce document décrit l'intégration du harness X11 de SONAR dans une chaîne
GitLab CI pilotant une VM Linux avec Ansible. L'objectif est de valider le
paquet réellement produit et installé, avec le vrai binaire Tauri, le WebView,
les dialogues natifs et les commandes Rust.

Le script exécuté est
[`run-sonar-x11-e2e.sh`](./run-sonar-x11-e2e.sh). Son contrat fonctionnel et la
liste détaillée des contrôles sont présentés dans le [README E2E](./README.md).

## Résultat attendu

Le chemin nominal est le suivant :

```text
job GitLab build
    │
    ├── paquet sonar_*.deb
    └── dist/ du même build
            │
            ▼
job GitLab e2e-vm ── Ansible/SSH ──► VM de test propre
                                            │
                                            ├── installe le paquet
                                            ├── déploie le harness et les fixtures
                                            ├── démarre Xvfb automatiquement
                                            └── exécute les parcours E2E
                                                        │
                                                        ▼
                         GitLab conserve résumé, logs, captures et exports
```

Le job est bloquant : un retour non nul du script doit faire échouer la
pipeline. Ne pas utiliser `allow_failure: true` pour ce contrôle.

## Périmètre de validation

Le harness couvre notamment :

- le smoke test du binaire installé et l'ouverture de la fenêtre Tauri ;
- les panneaux Configuration et Filtre BPF ;
- l'import d'un PCAP puis le rendu du graphe ;
- l'activation et l'arrêt de ForceAtlas2 ;
- l'import et la gestion des labels ;
- les exports PNG, matrice CSV, labels CSV et logs ;
- le reset puis l'import d'une matrice CSV ;
- les dialogues de fichiers natifs GTK ;
- les erreurs CSP et les panic Rust visibles dans les logs ;
- la présence des assets attendus dans `dist/`, si ce dossier est déployé.

La capture réseau en direct est volontairement désactivée par défaut. Les
imports PCAP testent le pipeline sans donner de privilèges réseau au job.

## Contrat d'entrée et de sortie

| Élément          | Contrat                                                                          |
| ---------------- | -------------------------------------------------------------------------------- |
| Paquet           | Un unique paquet `.deb` provenant du job `build`                                 |
| Binaire installé | `/usr/bin/sonar` pour le paquet Debian actuel                                    |
| Harness          | `script/e2e/` et `script/ci/smoke-test-release-binary.sh` du même commit         |
| Fixtures         | Un PCAP, une matrice CSV et un fichier de labels CSV non sensibles               |
| Assets           | Le dossier `dist/` provenant du même job que le paquet                           |
| Affichage        | Xvfb créé automatiquement quand `DISPLAY` est absent                             |
| Succès           | Code retour `0` et première ligne de `summary.txt` égale à `SONAR X11 E2E: PASS` |
| Échec            | Code retour non nul, avec logs et capture `failure.png` lorsque possible         |

Le nombre de lignes `PASS` peut varier si Openbox ou la capture live sont
activés. Le gate doit donc reposer sur le code retour et `summary.txt`, pas sur
un nombre de contrôles codé en dur.

## Préparation de la VM

### Système recommandé

- VM Debian ou Ubuntu amd64 compatible avec le paquet produit ;
- VM éphémère recréée depuis une image maîtrisée pour chaque pipeline ;
- compte SSH non-root dédié à la CI avec élévation contrôlée pour
  l'installation des paquets ;
- au moins 2 vCPU, 4 Gio de RAM et 5 Gio d'espace libre ;
- aucun serveur X ni processus SONAR déjà actif.

Une VM persistante reste possible. Dans ce cas, sérialiser les jobs avec
`resource_group` dans GitLab et restaurer la VM après chaque exécution pour
éviter les états résiduels.

### Dépendances de test

Le paquet SONAR installe ses dépendances runtime. Le harness demande en plus :

```bash
apt-get install --yes \
  build-essential pkg-config libx11-dev libxtst-dev \
  xvfb openbox x11-utils wmctrl xclip imagemagick dbus-x11 file
```

`openbox` et `wmctrl` sont facultatifs techniquement, mais recommandés sur
l'image de test afin de garder un comportement graphique identique entre les
exécutions. `cc`, les headers X11 et XTest sont nécessaires, car le petit pilote
X11 est compilé localement au début du scénario.

### Réseau

Xvfb est lancé avec `-nolisten tcp`. La VM n'a donc pas besoin d'exposer un
port graphique. Seul SSH depuis le runner ou le contrôleur Ansible est requis.
Les accès sortants de la VM peuvent être interdits une fois les paquets système
disponibles depuis un miroir interne.

## Préparation de GitLab

Configurer les éléments suivants dans les variables CI/CD protégées :

| Variable                    | Type conseillé                            | Usage                           |
| --------------------------- | ----------------------------------------- | ------------------------------- |
| `SONAR_E2E_SSH_KEY`         | File, protégée                            | Clé privée du compte Ansible    |
| `SONAR_E2E_KNOWN_HOSTS`     | File, protégée                            | Empreinte SSH vérifiée de la VM |
| Inventaire ou adresse de VM | Variable protégée ou inventaire dynamique | Cible Ansible                   |

Ne pas désactiver `StrictHostKeyChecking` et ne pas écrire de clé privée dans
le dépôt. Pour une VM créée dynamiquement, faire produire l'inventaire et
`known_hosts` par l'étape de provisioning approuvée.

Exemple minimal pour une VM déjà provisionnée :

```ini
[sonar_e2e]
sonar-e2e-linux ansible_host=192.0.2.10 ansible_user=sonar-ci
```

L'adresse ci-dessus est un exemple réservé à la documentation. Le compte
`sonar-ci` doit pouvoir utiliser `become` uniquement pour les tâches système
prévues par le playbook.

Le job `build` doit publier :

```yaml
artifacts:
  paths:
    - src-tauri/target/release/bundle/deb/*.deb
    - dist/
```

Le build doit laisser exactement un paquet Debian dans ses artefacts. Comme
`src-tauri/target/` est mis en cache dans la CI actuelle, supprimer les anciens
paquets du dossier de sortie avant le build ou faire échouer le job E2E lorsque
plusieurs `.deb` sont trouvés. Il ne faut pas sélectionner silencieusement un
ancien paquet.

## Playbook Ansible de référence

L'exemple suivant est volontairement autonome. Il installe le paquet, déploie
uniquement les fichiers utiles depuis le checkout du runner, exécute SONAR avec
un utilisateur non privilégié, puis récupère les preuves avant de propager un
éventuel échec.

Les variables `sonar_deb_local`, `sonar_deb_sha256`, `ci_pipeline_id` et
`local_artifacts_dir` sont fournies par le job GitLab.

```yaml
---
- name: Valider le paquet SONAR sur une VM Linux
  hosts: sonar_e2e
  gather_facts: true
  become: true

  vars:
    controller_repo_root: "{{ lookup('ansible.builtin.env', 'CI_PROJECT_DIR') }}"
    e2e_user: sonar-e2e
    e2e_home: /var/lib/sonar-e2e
    remote_root: '/opt/sonar-e2e/{{ ci_pipeline_id }}'
    remote_fixtures: '{{ remote_root }}/fixtures'
    remote_artifacts: '/var/tmp/sonar-e2e-{{ ci_pipeline_id }}'
    remote_archive: '/var/tmp/sonar-e2e-{{ ci_pipeline_id }}.tar.gz'
    remote_deb: '/var/tmp/sonar-under-test-{{ ci_pipeline_id }}.deb'
    sonar_binary: /usr/bin/sonar

  pre_tasks:
    - name: Vérifier la famille de système prise en charge
      ansible.builtin.assert:
        that:
          - ansible_facts.os_family == "Debian"
        fail_msg: 'Ce playbook de référence attend une VM Debian ou Ubuntu.'

    - name: Vérifier les variables obligatoires
      ansible.builtin.assert:
        that:
          - sonar_deb_local | length > 0
          - sonar_deb_sha256 | length == 64
          - ci_pipeline_id | string | length > 0
          - local_artifacts_dir | length > 0

  tasks:
    - name: Installer les dépendances du harness
      ansible.builtin.apt:
        name:
          - build-essential
          - pkg-config
          - libx11-dev
          - libxtst-dev
          - xvfb
          - openbox
          - x11-utils
          - wmctrl
          - xclip
          - imagemagick
          - dbus-x11
          - file
        state: present
        update_cache: true

    - name: Créer l'utilisateur de test non privilégié
      ansible.builtin.user:
        name: '{{ e2e_user }}'
        home: '{{ e2e_home }}'
        create_home: true
        shell: /usr/sbin/nologin
        system: true

    - name: Copier le paquet construit
      ansible.builtin.copy:
        src: '{{ sonar_deb_local }}'
        dest: '{{ remote_deb }}'
        owner: root
        group: root
        mode: '0644'

    - name: Calculer l'empreinte du paquet reçu
      ansible.builtin.stat:
        path: '{{ remote_deb }}'
        checksum_algorithm: sha256
      register: remote_deb_stat

    - name: Vérifier l'intégrité du paquet transféré
      ansible.builtin.assert:
        that:
          - remote_deb_stat.stat.checksum == sonar_deb_sha256
        fail_msg: 'Le SHA-256 du paquet copié sur la VM est différent.'

    - name: Installer le paquet SONAR à valider
      ansible.builtin.apt:
        deb: '{{ remote_deb }}'
        state: present

    - name: Créer les répertoires du scénario
      ansible.builtin.file:
        path: '{{ item }}'
        state: directory
        owner: '{{ e2e_user }}'
        group: '{{ e2e_user }}'
        mode: '0750'
      loop:
        - '{{ remote_root }}'
        - '{{ remote_root }}/script'
        - '{{ remote_root }}/script/ci'
        - '{{ remote_root }}/script/e2e'
        - '{{ remote_fixtures }}'
        - '{{ remote_artifacts }}'

    - name: Déployer le harness X11
      ansible.builtin.copy:
        src: '{{ controller_repo_root }}/script/e2e/'
        dest: '{{ remote_root }}/script/e2e/'
        owner: '{{ e2e_user }}'
        group: '{{ e2e_user }}'
        mode: preserve

    - name: Déployer le smoke test du binaire
      ansible.builtin.copy:
        src: '{{ controller_repo_root }}/script/ci/smoke-test-release-binary.sh'
        dest: '{{ remote_root }}/script/ci/smoke-test-release-binary.sh'
        owner: '{{ e2e_user }}'
        group: '{{ e2e_user }}'
        mode: '0750'

    - name: Déployer le dist issu du même build
      ansible.builtin.copy:
        src: '{{ controller_repo_root }}/dist/'
        dest: '{{ remote_root }}/dist/'
        owner: '{{ e2e_user }}'
        group: '{{ e2e_user }}'
        mode: preserve

    - name: Déployer les fixtures déterministes
      ansible.builtin.copy:
        src: '{{ item.src }}'
        dest: '{{ remote_fixtures }}/{{ item.dest }}'
        owner: '{{ e2e_user }}'
        group: '{{ e2e_user }}'
        mode: '0640'
      loop:
        - src: '{{ controller_repo_root }}/src-tauri/test_files/pcaps/import/pcap_tshark_corpus/industrial_ethernet.pcap'
          dest: industrial_ethernet.pcap
        - src: '{{ controller_repo_root }}/src-tauri/test_files/20260703_NP_Matrice.csv'
          dest: matrice.csv
        - src: '{{ controller_repo_root }}/src-tauri/test_files/20260703_NP_Labels.csv'
          dest: labels.csv

    - name: Exécuter et collecter le scénario E2E
      block:
        - name: Lancer le binaire installé dans un nouvel affichage Xvfb
          become_user: '{{ e2e_user }}'
          ansible.builtin.command:
            argv:
              - /usr/bin/env
              - -u
              - DISPLAY
              - -u
              - DBUS_SESSION_BUS_ADDRESS
              - '{{ remote_root }}/script/e2e/run-sonar-x11-e2e.sh'
              - --binary
              - '{{ sonar_binary }}'
              - --artifacts
              - '{{ remote_artifacts }}'
          environment:
            HOME: '{{ e2e_home }}'
            SONAR_E2E_PCAP: '{{ remote_fixtures }}/industrial_ethernet.pcap'
            SONAR_E2E_MATRIX: '{{ remote_fixtures }}/matrice.csv'
            SONAR_E2E_LABELS: '{{ remote_fixtures }}/labels.csv'
          register: sonar_e2e_result
          changed_when: false
          failed_when: false

        - name: Afficher la sortie synthétique du scénario
          ansible.builtin.debug:
            var: sonar_e2e_result.stdout_lines

      always:
        - name: Archiver les preuves sur la VM
          ansible.builtin.command:
            argv:
              - /usr/bin/tar
              - -C
              - /var/tmp
              - -czf
              - '{{ remote_archive }}'
              - '{{ remote_artifacts | basename }}'
          changed_when: true

        - name: Créer le dossier local de collecte
          become: false
          delegate_to: localhost
          ansible.builtin.file:
            path: '{{ local_artifacts_dir }}'
            state: directory
            mode: '0750'

        - name: Récupérer les preuves sur le runner GitLab
          ansible.builtin.fetch:
            src: '{{ remote_archive }}'
            dest: '{{ local_artifacts_dir }}/{{ inventory_hostname }}.tar.gz'
            flat: true

    - name: Bloquer la pipeline si le scénario a échoué
      ansible.builtin.fail:
        msg: >-
          Le scénario E2E SONAR a échoué avec le code
          {{ sonar_e2e_result.rc }}. Consulter l'archive de preuves.
      when: sonar_e2e_result.rc != 0
```

Ce playbook ne passe pas `--build` : la cible du test est obligatoirement le
binaire issu du paquet installé, et non un nouveau binaire reconstruit sur la
VM.

## Job GitLab de référence

Ajouter un stage `e2e` après `build`, puis inclure par exemple un fichier
`.gitlab/ci/e2e-vm.yml`.

L'image du job doit contenir Bash, une version épinglée d'Ansible et un client
SSH. Utiliser de préférence une image interne référencée par digest et placer
sa référence complète dans `ANSIBLE_RUNNER_IMAGE`.

```yaml
e2e:linux-vm:
  stage: e2e
  image: '${ANSIBLE_RUNNER_IMAGE}'
  needs:
    - job: build
      artifacts: true
  timeout: 30 minutes
  interruptible: true

  # À conserver si plusieurs pipelines partagent la même VM.
  resource_group: sonar-e2e-linux-vm

  before_script:
    - install -m 600 "$SONAR_E2E_SSH_KEY" "$CI_PROJECT_DIR/.ansible-key"
    - install -m 644 "$SONAR_E2E_KNOWN_HOSTS" "$CI_PROJECT_DIR/.known_hosts"
    - export ANSIBLE_PRIVATE_KEY_FILE="$CI_PROJECT_DIR/.ansible-key"
    - export ANSIBLE_HOST_KEY_CHECKING=True
    - export ANSIBLE_SSH_ARGS="-o UserKnownHostsFile=$CI_PROJECT_DIR/.known_hosts"

  script:
    - |
      mapfile -t sonar_debs < <(
        find src-tauri/target/release/bundle/deb \
          -maxdepth 1 -type f -name 'sonar_*.deb' -print
      )
      if (( ${#sonar_debs[@]} != 1 )); then
        printf 'Un seul paquet SONAR est attendu, trouvé : %s\n' \
          "${#sonar_debs[@]}" >&2
        exit 1
      fi
      sonar_deb="${sonar_debs[0]}"
      sonar_sha256="$(sha256sum "$sonar_deb" | cut -d ' ' -f 1)"

      set +e
      ansible-playbook \
        --inventory ci/ansible/inventory.ini \
        ci/ansible/sonar-e2e.yml \
        --extra-vars "sonar_deb_local=$CI_PROJECT_DIR/$sonar_deb" \
        --extra-vars "sonar_deb_sha256=$sonar_sha256" \
        --extra-vars "ci_pipeline_id=$CI_PIPELINE_ID" \
        --extra-vars "local_artifacts_dir=$CI_PROJECT_DIR/e2e-artifacts"
      e2e_status=$?
      set -e

      for archive in e2e-artifacts/*.tar.gz; do
        [[ -e "$archive" ]] || continue
        mkdir -p "${archive%.tar.gz}"
        tar -xzf "$archive" -C "${archive%.tar.gz}"
      done
      exit "$e2e_status"

  artifacts:
    when: always
    paths:
      - e2e-artifacts/
    expire_in: 14 days

  rules:
    - if: '$CI_PIPELINE_SOURCE == "merge_request_event"'
    - if: '$CI_COMMIT_BRANCH == "main"'
    - if: '$CI_COMMIT_TAG'
```

Dans `.gitlab-ci.yml`, déclarer le stage et l'include :

```yaml
stages:
  - build
  - e2e
  - sonarqube

include:
  - local: '.gitlab/ci/build.yml'
  - local: '.gitlab/ci/e2e-vm.yml'
  - local: '.gitlab/ci/sonarqube.yml'
```

Les chemins `ci/ansible/` et la méthode de création de la VM sont des exemples
à adapter à l'infrastructure de l'équipe. Le harness SONAR lui-même reste dans
`script/e2e/`.

## Politique de sécurité

### Exécution normale

- exécuter SONAR et Xvfb avec `sonar-e2e`, jamais avec `root` ;
- réserver `become` à l'installation des paquets et à la préparation de la VM ;
- ne pas utiliser `xhost +` et ne pas exposer Xvfb sur TCP ;
- utiliser uniquement les fixtures versionnées, jamais un PCAP de production ;
- utiliser un dossier d'artefacts unique par pipeline ;
- détruire ou restaurer la VM après le test ;
- limiter la rétention et l'accès aux artefacts GitLab, car les logs et les
  données XDG peuvent contenir les données importées pendant le scénario.

### Capture réseau en direct

Le mode live n'est pas nécessaire au gate standard. S'il est ajouté à une
pipeline spécialisée :

```bash
setcap cap_net_raw,cap_net_admin=eip /usr/bin/sonar
```

puis ajouter `--live-capture` à la commande E2E. Cette capability doit être
accordée uniquement dans une VM éphémère dédiée et après l'installation du
paquet. Ne pas contourner cette exigence en exécutant toute l'application avec
`sudo`.

## Artefacts à conserver

Le dossier produit contient notamment :

- `summary.txt` : verdict et chemins des fixtures ;
- `runtime.log` et `startup-smoke.log` : sorties du binaire ;
- les captures `01-startup.png` à `10-matrix-imported.png` ;
- `native-dialog-*-before.png` : preuve des dialogues natifs ;
- `graph-export.png`, `matrix-export.csv` et `labels-export.csv` ;
- `logs-export.log/` : export applicatif des logs ;
- `failure.png` lorsqu'une erreur survient après l'ouverture de SONAR ;
- les répertoires XDG isolés de la session.

Le job GitLab doit publier ces éléments avec `artifacts: when: always`, y
compris lorsque Ansible renvoie un échec.

## Critères d'acceptation du gate

L'intégration est considérée opérationnelle lorsque :

1. le paquet testé vient exactement du job `build` de la même pipeline ;
2. son SHA-256 est vérifié après le transfert sur la VM ;
3. le paquet est installé sur une VM propre avant le test ;
4. le scénario s'exécute sans `DISPLAY` hérité et crée son propre Xvfb ;
5. le job échoue pour tout code retour non nul ;
6. `summary.txt` contient `SONAR X11 E2E: PASS` en cas de succès ;
7. les preuves sont récupérées et publiées même en cas d'échec ;
8. le job n'est ni autorisé à échouer ni relancé automatiquement pour masquer
   un test instable ;
9. aucun privilège de capture réseau n'est accordé au gate standard.

## Diagnostic rapide

| Symptôme                      | Vérification                                                        |
| ----------------------------- | ------------------------------------------------------------------- |
| `commande requise absente`    | Comparer les paquets installés avec la liste des dépendances        |
| `Xvfb ne répond pas`          | Lire `xvfb.log`, vérifier `/tmp` et les numéros `DISPLAY` 90 à 119  |
| Dialogue natif introuvable    | Vérifier `dbus-x11`, `xclip`, GTK et les captures `native-dialog-*` |
| Fenêtre SONAR déjà présente   | Recréer la VM ou supprimer proprement la session précédente         |
| Fixture absente               | Vérifier les trois variables `SONAR_E2E_*` et les droits de lecture |
| Asset absent                  | Vérifier que `dist/` vient du même build et a été copié sur la VM   |
| Import PCAP ou labels bloqué  | Lire `runtime.log` et la capture correspondant à l'étape            |
| Échec uniquement en mode live | Vérifier `getcap /usr/bin/sonar` et l'interface de capture de la VM |

Pour reproduire une exécution CI manuellement sur la VM, utiliser le même
compte non privilégié et la même commande Ansible. L'option `--keep-open` est
réservée au diagnostic interactif et ne doit jamais être utilisée dans le gate
automatique.
