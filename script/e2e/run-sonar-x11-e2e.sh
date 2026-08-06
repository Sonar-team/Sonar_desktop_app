#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
original_args=("$@")

binary_path="$ROOT_DIR/src-tauri/target/release/sonar"
artifacts_dir=""
build_binary=0
keep_open=0
live_capture=0
live_capture_seconds=5

usage() {
  cat <<'EOF'
Usage: run-sonar-x11-e2e.sh [options]

Options:
  --build                 Compile le binaire release avant le test.
  --binary PATH           Binaire SONAR à tester.
  --artifacts DIR         Dossier conservant logs, captures et exports.
  --live-capture          Teste aussi Démarrer/Arrêter (capabilities requises).
  --live-seconds N        Durée de la capture live (défaut : 5 secondes).
  --keep-open             Laisse SONAR ouvert à la fin du scénario.
  -h, --help              Affiche cette aide.

Sans DISPLAY, le script démarre automatiquement Xvfb en 1920x1080.
EOF
}

while (($# > 0)); do
  case "$1" in
    --build)
      build_binary=1
      shift
      ;;
    --binary)
      binary_path="${2:?--binary attend un chemin}"
      shift 2
      ;;
    --artifacts)
      artifacts_dir="${2:?--artifacts attend un chemin}"
      shift 2
      ;;
    --live-capture)
      live_capture=1
      shift
      ;;
    --live-seconds)
      live_capture_seconds="${2:?--live-seconds attend un entier}"
      shift 2
      ;;
    --keep-open)
      keep_open=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Option inconnue : $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ ! "$live_capture_seconds" =~ ^[1-9][0-9]*$ ]]; then
  echo "--live-seconds doit être un entier strictement positif" >&2
  exit 2
fi

if [[ -z "${DISPLAY:-}" && -z "${DBUS_SESSION_BUS_ADDRESS:-}" &&
      "${SONAR_E2E_DBUS_BOOTSTRAPPED:-0}" != "1" ]] &&
   command -v dbus-run-session >/dev/null 2>&1; then
  exec env SONAR_E2E_DBUS_BOOTSTRAPPED=1 dbus-run-session -- \
    "$0" "${original_args[@]}"
fi

if [[ -z "$artifacts_dir" ]]; then
  tmp_root="${RUNNER_TEMP:-${TMPDIR:-/tmp}}"
  artifacts_dir="$(mktemp -d "${tmp_root%/}/sonar-x11-e2e.XXXXXX")"
else
  mkdir -p "$artifacts_dir"
  artifacts_dir="$(cd "$artifacts_dir" && pwd)"
fi

runtime_log="$artifacts_dir/runtime.log"
summary_file="$artifacts_dir/summary.txt"
input_binary="$artifacts_dir/sonar-x11-input"
main_id=""
app_pid=""
xvfb_pid=""
wm_pid=""
runtime_dir_created=""
pass_count=0
dialog_count=0
wm_available=0

log() {
  printf '[SONAR E2E] %s\n' "$*"
}

pass() {
  pass_count=$((pass_count + 1))
  log "PASS $pass_count — $*"
}

fail() {
  log "FAIL — $*" >&2
  return 1
}

capture_window() {
  local name="$1"
  local output="$artifacts_dir/$name.png"
  [[ -n "$main_id" ]] || return 0
  import -window "$main_id" "$output"
  identify "$output" >/dev/null
  log "Capture : $output"
}

cleanup() {
  local status="$?"
  local preserve_session=0
  set +e

  if ((keep_open == 1 && status == 0)); then
    preserve_session=1
  fi

  if ((status != 0)) && [[ -n "$main_id" ]]; then
    capture_window "failure" >/dev/null 2>&1 || true
  fi

  if [[ -n "$app_pid" ]] && kill -0 "$app_pid" 2>/dev/null; then
    if ((preserve_session == 1)); then
      log "SONAR reste ouvert (PID $app_pid)."
    else
      kill "$app_pid" 2>/dev/null || true
      wait "$app_pid" 2>/dev/null || true
    fi
  fi

  if ((preserve_session == 0)) && [[ -n "$xvfb_pid" ]] &&
    kill -0 "$xvfb_pid" 2>/dev/null; then
    kill "$xvfb_pid" 2>/dev/null || true
    wait "$xvfb_pid" 2>/dev/null || true
  fi

  if ((preserve_session == 0)) && [[ -n "$wm_pid" ]] &&
    kill -0 "$wm_pid" 2>/dev/null; then
    kill "$wm_pid" 2>/dev/null || true
    wait "$wm_pid" 2>/dev/null || true
  fi

  if ((preserve_session == 0)) && [[ -n "$runtime_dir_created" ]]; then
    rmdir "$runtime_dir_created" 2>/dev/null || true
  fi

  log "Artefacts : $artifacts_dir"
  exit "$status"
}
trap cleanup EXIT

require_command() {
  local command_name="$1"
  if ! command -v "$command_name" >/dev/null 2>&1; then
    fail "commande requise absente : $command_name"
  fi
}

wait_for_log() {
  local pattern="$1"
  local timeout_seconds="${2:-30}"
  local deadline=$((SECONDS + timeout_seconds))
  while ((SECONDS < deadline)); do
    if [[ -f "$runtime_log" ]] && grep -Fq -- "$pattern" "$runtime_log"; then
      return 0
    fi
    if [[ -n "$app_pid" ]] && ! kill -0 "$app_pid" 2>/dev/null; then
      fail "SONAR s'est arrêté en attendant le log : $pattern"
    fi
    sleep 0.25
  done
  fail "log absent après ${timeout_seconds}s : $pattern"
}

wait_for_file() {
  local path="$1"
  local timeout_seconds="${2:-20}"
  local deadline=$((SECONDS + timeout_seconds))
  while ((SECONDS < deadline)); do
    if [[ -s "$path" ]]; then
      return 0
    fi
    sleep 0.25
  done
  fail "fichier absent ou vide : $path"
}

send_key() {
  "$input_binary" key-id "$main_id" "$1"
}

send_ctrl() {
  "$input_binary" ctrl-id "$main_id" "$1"
}

send_ctrl_shift() {
  "$input_binary" ctrl-shift-id "$main_id" "$1"
}

click_main() {
  "$input_binary" click-id "$main_id" "$1" "$2"
}

# Boutons de la barre d'outils (TopBar.vue), repérés par leur RANG (1 = le
# premier bouton à gauche). Les boutons ont un pas régulier : la position est
# calculée au lieu d'être codée en dur, de sorte qu'insérer un bouton dans
# TopBar.vue ne demande que de décaler les rangs ci-dessous — c'est ce qui
# avait été oublié à l'ajout des boutons projet (#159), faisant cliquer le
# scénario à côté de l'export des labels.
TOOLBAR_Y=46
TOOLBAR_FIRST_X=35
# Pas horizontal en dixièmes de pixel (bash n'a pas de flottants).
TOOLBAR_STEP_DECI=394

click_toolbar() {
  local rank="$1"
  local x=$((TOOLBAR_FIRST_X + (rank - 1) * TOOLBAR_STEP_DECI / 10))
  click_main "$x" "$TOOLBAR_Y"
}

focus_id() {
  local window_id="$1"
  if ((wm_available == 1)); then
    wmctrl -ia "$window_id"
  else
    "$input_binary" focus-id "$window_id"
  fi
  sleep 0.2
}

fill_native_dialog() {
  local selected_path="$1"
  local dialog_id

  dialog_id="$("$input_binary" wait-dialog "$main_id" 15000)"
  dialog_count=$((dialog_count + 1))
  log "Dialogue natif $dialog_count : $dialog_id"
  import -window "$dialog_id" \
    "$artifacts_dir/native-dialog-${dialog_count}-before.png"

  printf '%s' "$selected_path" |
    xclip -selection clipboard >"$artifacts_dir/xclip.log" 2>&1
  focus_id "$dialog_id"
  "$input_binary" ctrl-id "$dialog_id" l
  sleep 0.2
  "$input_binary" ctrl-id "$dialog_id" v
  sleep 0.3
  "$input_binary" key-id "$dialog_id" Return
  if ! "$input_binary" wait-gone "$dialog_id" 2000 2>/dev/null; then
    import -window "$dialog_id" \
      "$artifacts_dir/native-dialog-${dialog_count}-after.png" || true
    # Certains sélecteurs GTK valident d'abord le chemin puis le bouton
    # d'ouverture : un second Entrée couvre ce comportement sans cliquer sur
    # une position dépendante du thème de la VM.
    focus_id "$dialog_id"
    "$input_binary" key-id "$dialog_id" Return
    "$input_binary" wait-gone "$dialog_id" 15000
  fi
  focus_id "$main_id"
}

# Répond « oui » à une boîte de confirmation native (pas un sélecteur de
# fichiers). Depuis #159, réinitialiser un relevé non enregistré en ouvre une :
# sans y répondre, le scénario resterait bloqué sur le dialogue.
confirm_native_dialog() {
  local dialog_id
  dialog_id="$("$input_binary" wait-dialog "$main_id" 15000)"
  dialog_count=$((dialog_count + 1))
  log "Dialogue de confirmation $dialog_count : $dialog_id"
  import -window "$dialog_id" \
    "$artifacts_dir/native-dialog-${dialog_count}-before.png"

  # Le focus initial de GTK est sur le bouton de REFUS (« No ») : appuyer
  # sur Entrée annulerait l'action. Le bouton d'acceptation est le dernier
  # de la rangée du bas ; on le vise via la géométrie réelle du dialogue
  # plutôt qu'avec des coordonnées fixes dépendantes du thème.
  local width height
  read -r width height _ _ < <("$input_binary" geometry-id "$dialog_id")
  focus_id "$dialog_id"
  "$input_binary" click-id "$dialog_id" \
    "$((width * 3 / 4))" "$((height - 20))"
  if ! "$input_binary" wait-gone "$dialog_id" 5000 2>/dev/null; then
    import -window "$dialog_id" \
      "$artifacts_dir/native-dialog-${dialog_count}-after.png" || true
    fail "confirmation native non validée (dialogue $dialog_id)"
  fi
  focus_id "$main_id"
}

assert_bundled_assets() {
  local asset_prefix
  [[ -d "$ROOT_DIR/dist/assets" ]] || {
    log "Dist absent : contrôle statique des assets ignoré."
    return 0
  }

  for asset_prefix in StoreLogo mdi--gear import_csv TablerCpu select-chevron warning-sign; do
    if ! compgen -G "$ROOT_DIR/dist/assets/${asset_prefix}-*" >/dev/null; then
      fail "asset Vite absent : $asset_prefix"
    fi
  done
  if grep -R -I -E -q 'data:(image|font)' "$ROOT_DIR/dist"; then
    fail "une image ou police data: subsiste dans dist"
  fi
  pass "assets CSP présents et non incorporés en data:"
}

for command_name in cc pkg-config xclip import identify grep file unzip; do
  require_command "$command_name"
done

if ! pkg-config --exists x11 xtst; then
  fail "bibliothèques de développement X11/XTest absentes (pkg-config x11 xtst)"
fi
read -r -a x11_cflags <<<"$(pkg-config --cflags x11 xtst)"
read -r -a x11_libs <<<"$(pkg-config --libs x11 xtst)"

if ((build_binary == 1)); then
  require_command deno
  log "Compilation du binaire release…"
  (cd "$ROOT_DIR" && deno task tauri build --no-bundle)
fi

if [[ ! -x "$binary_path" ]]; then
  fail "binaire introuvable ou non exécutable : $binary_path (utiliser --build)"
fi
binary_path="$(cd "$(dirname "$binary_path")" && pwd)/$(basename "$binary_path")"

pcap_fixture="${SONAR_E2E_PCAP:-$ROOT_DIR/src-tauri/test_files/pcaps/import/pcap_tshark_corpus/industrial_ethernet.pcap}"
matrix_fixture="${SONAR_E2E_MATRIX:-$ROOT_DIR/src-tauri/test_files/20260703_NP_Matrice.csv}"
labels_fixture="${SONAR_E2E_LABELS:-$ROOT_DIR/src-tauri/test_files/20260703_NP_Labels.csv}"

for fixture in "$pcap_fixture" "$matrix_fixture" "$labels_fixture"; do
  if [[ ! -f "$fixture" ]]; then
    fail "fixture absente : $fixture"
  fi
done

cc -std=c11 -D_DEFAULT_SOURCE -O2 -Wall -Wextra -Werror \
  "${x11_cflags[@]}" \
  "$ROOT_DIR/script/e2e/sonar-x11-input.c" \
  -o "$input_binary" \
  "${x11_libs[@]}"
pass "pilote X11 compilé"

if [[ -z "${DISPLAY:-}" ]]; then
  require_command Xvfb
  require_command xdpyinfo
  for display_number in $(seq 90 119); do
    if [[ ! -e "/tmp/.X11-unix/X$display_number" &&
          ! -e "/tmp/.X${display_number}-lock" ]]; then
      export DISPLAY=":$display_number"
      break
    fi
  done
  if [[ -z "${DISPLAY:-}" ]]; then
    fail "aucun numéro DISPLAY libre entre :90 et :119"
  fi

  Xvfb "$DISPLAY" -screen 0 1920x1080x24 -nolisten tcp -noreset \
    >"$artifacts_dir/xvfb.log" 2>&1 &
  xvfb_pid="$!"
  display_deadline=$((SECONDS + 10))
  until xdpyinfo -display "$DISPLAY" >/dev/null 2>&1; do
    if ((SECONDS >= display_deadline)); then
      fail "Xvfb ne répond pas sur $DISPLAY"
    fi
    sleep 0.2
  done
  pass "écran Xvfb prêt sur $DISPLAY"

  if command -v openbox >/dev/null 2>&1 && command -v wmctrl >/dev/null 2>&1; then
    openbox >"$artifacts_dir/openbox.log" 2>&1 &
    wm_pid="$!"
    wm_deadline=$((SECONDS + 10))
    until wmctrl -m >/dev/null 2>&1; do
      if ((SECONDS >= wm_deadline)); then
        fail "openbox ne fournit pas de gestionnaire EWMH sur $DISPLAY"
      fi
      sleep 0.2
    done
    wm_available=1
    pass "gestionnaire de fenêtres openbox prêt"
  else
    log "Aucun window manager : focus X11 direct utilisé avec Xvfb."
  fi
else
  pass "écran X11 existant utilisé : $DISPLAY"
  if command -v wmctrl >/dev/null 2>&1 && wmctrl -m >/dev/null 2>&1; then
    wm_available=1
  else
    log "Gestionnaire EWMH non détecté : focus X11 direct utilisé."
  fi
fi

if "$input_binary" wait-window SONAR 300 >/dev/null 2>&1; then
  fail "une fenêtre SONAR existe déjà sur $DISPLAY"
fi

if [[ -z "${XDG_RUNTIME_DIR:-}" ]]; then
  runtime_dir_created="$(mktemp -d "${TMPDIR:-/tmp}/sonar-e2e-runtime.XXXXXX")"
  chmod 700 "$runtime_dir_created"
  export XDG_RUNTIME_DIR="$runtime_dir_created"
fi
export XDG_DATA_HOME="$artifacts_dir/xdg-data"
export XDG_CONFIG_HOME="$artifacts_dir/xdg-config"
export XDG_CACHE_HOME="$artifacts_dir/xdg-cache"
export GDK_BACKEND=x11
export GDK_SCALE=1
export GDK_DPI_SCALE=1
export LIBGL_ALWAYS_SOFTWARE=1
mkdir -p "$XDG_DATA_HOME" "$XDG_CONFIG_HOME" "$XDG_CACHE_HOME"

log "Smoke test du binaire : $binary_path"
"$ROOT_DIR/script/ci/smoke-test-release-binary.sh" "$binary_path" 30 \
  >"$artifacts_dir/startup-smoke.log" 2>&1
pass "smoke test de démarrage"

log "Lancement du binaire release…"
RUST_LOG="info,sonar=debug" "$binary_path" >"$runtime_log" 2>&1 &
app_pid="$!"
main_id="$("$input_binary" wait-window SONAR 30000)"
focus_id "$main_id"

read -r main_width main_height _ _ < <("$input_binary" geometry-id "$main_id")
if ((main_width < 1400 || main_height < 800)); then
  fail "fenêtre trop petite pour le scénario : ${main_width}x${main_height}"
fi
center_x=$((main_width / 2))
center_y=$((main_height / 2))
capture_window "01-startup"
pass "binaire lancé, fenêtre SONAR ${main_width}x${main_height}"

assert_bundled_assets

send_ctrl comma
sleep 1
capture_window "02-configuration"
send_ctrl comma
pass "panneau de configuration"

send_ctrl f
sleep 1
capture_window "03-filtre-bpf"
send_ctrl f
pass "panneau de filtre BPF"

if ((live_capture == 1)); then
  require_command getcap
  if ! getcap "$binary_path" | grep -Eq 'cap_net_raw.*(eip|ep)|cap_net_admin.*(eip|ep)'; then
    fail "--live-capture requiert cap_net_raw et cap_net_admin sur le binaire"
  fi
  send_ctrl p
  wait_for_log "Capture démarrée : true" 20
  sleep "$live_capture_seconds"
  capture_window "04-capture-live"
  send_ctrl_shift p
  wait_for_log "Capture arrêtée : false" 20
  send_ctrl_shift r
  confirm_native_dialog
  sleep 1
  pass "capture live démarrée puis arrêtée"
else
  log "Capture live ignorée (ajouter --live-capture pour l'activer)."
fi

send_ctrl o
sleep 1
capture_window "04-import-pcap-dialog"
click_main "$center_x" "$((center_y - 40))"
fill_native_dialog "$pcap_fixture"
send_key Tab
send_key Tab
send_key Return
# Bilan par catégorie fine (#150) : l'équation complète est dans le log.
wait_for_log "10 paquets lus = 10 intégrés + 0 tronqués + 0 DLT non supportés + 0 malformés" 30
wait_for_log "FIN traitement liste PCAP" 30
sleep 1
capture_window "05-pcap-imported"
pass "import PCAP réel et rendu du graphe"

click_main 196 92
sleep 1
click_main 196 92
sleep 1
capture_window "06-forceatlas"
pass "worker ForceAtlas2 activé/désactivé"

png_base="$artifacts_dir/graph-export"
click_main 70 92
fill_native_dialog "$png_base"
png_export="$png_base.png"
wait_for_file "$png_export"
if ! file "$png_export" | grep -Fq "PNG image data"; then
  fail "l'export graphe n'est pas un PNG valide"
fi
read -r png_width png_height < <(identify -format '%w %h\n' "$png_export")
if ((png_width < main_width * 2 - 20 || png_height < 1000)); then
  fail "résolution PNG inattendue : ${png_width}x${png_height}"
fi
pass "export PNG valide (${png_width}x${png_height})"

click_toolbar 11 # importer un fichier de labels (icône CSV)
sleep 1
capture_window "07-import-labels-dialog"
click_main "$center_x" "$((center_y - 30))"
fill_native_dialog "$labels_fixture"
wait_for_log "import_label_file: $labels_fixture" 20
wait_for_log "label(s) de nœud mis à jour dans le graphe" 20
sleep 1
capture_window "08-labels-imported"
pass "import du fichier de labels"

click_toolbar 9 # gérer les labels
sleep 1
capture_window "09-labels-management"
pass "panneau de gestion des labels"

matrix_export_base="$artifacts_dir/matrix-export"
send_ctrl s
fill_native_dialog "$matrix_export_base"
matrix_export="$matrix_export_base.csv"
wait_for_file "$matrix_export"
if ! head -n 2 "$matrix_export" | grep -Fq "mac_source,mac_destination"; then
  fail "en-tête inattendu dans l'export matrice"
fi
pass "export CSV de la matrice"

labels_export_base="$artifacts_dir/labels-export"
click_toolbar 8 # exporter les labels
fill_native_dialog "$labels_export_base"
labels_export="$labels_export_base.csv"
wait_for_file "$labels_export"
if ! head -n 1 "$labels_export" | grep -Fxq "mac,ip,label"; then
  fail "en-tête inattendu dans l'export de labels"
fi
pass "export CSV des labels"

# Depuis #102 (v4.10.0), `export_logs` écrit une archive ZIP unique et non
# plus un dossier de fichiers `.log`.
logs_export_base="$artifacts_dir/logs-export"
send_ctrl l
fill_native_dialog "$logs_export_base"
logs_export="$logs_export_base.zip"
wait_for_file "$logs_export" 20
if ! file "$logs_export" | grep -Fq "Zip archive data"; then
  fail "l'export des logs n'est pas une archive ZIP : $logs_export"
fi
if ! unzip -Z1 "$logs_export" | grep -q '\.log$'; then
  fail "l'archive de logs ne contient aucun fichier .log"
fi
pass "export des logs en archive ZIP"

# Le relevé courant est modifié : le reset demande confirmation (#159).
# Que le reset ait bien vidé l'état est vérifié par le réimport ci-dessous,
# dont les compteurs attendus supposent une matrice repartie de zéro.
send_ctrl_shift r
confirm_native_dialog
sleep 1
send_ctrl o
sleep 1
click_main "$center_x" "$((center_y + 42))"
fill_native_dialog "$matrix_fixture"
send_key Tab
send_key Tab
send_key Return
wait_for_log "30 ligne(s) importée(s) -> 30 flux fusionné(s), 12 nœuds, 11 arêtes" 30
sleep 1
capture_window "10-matrix-imported"
pass "reset puis import d'une matrice CSV"

if grep -E -i -q \
  'content security policy|refused to (load|execute)|blocked by.*csp|csp violation' \
  "$runtime_log"; then
  fail "violation CSP détectée dans $runtime_log"
fi
if grep -E -i -q 'thread .* panicked|fatal runtime error' "$runtime_log"; then
  fail "panic détectée dans $runtime_log"
fi
pass "aucune violation CSP ni panic dans les logs runtime"

{
  printf 'SONAR X11 E2E: PASS\n'
  printf 'Tests réussis: %d\n' "$pass_count"
  printf 'Binaire: %s\n' "$binary_path"
  printf 'DISPLAY: %s\n' "$DISPLAY"
  printf 'PCAP: %s\n' "$pcap_fixture"
  printf 'Matrice: %s\n' "$matrix_fixture"
  printf 'Labels: %s\n' "$labels_fixture"
} >"$summary_file"

log "Scénario terminé : $pass_count contrôles réussis."
cat "$summary_file"
