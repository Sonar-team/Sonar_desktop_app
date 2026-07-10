# Cahier de recette VAE — SONAR

Liste des fonctions à recetter, avec cas d'usage pas à pas, résultats
attendus et fichiers de test nécessaires. Version cible : binaire release
(`deno task tauri build`), pas le mode dev (CSP et assets embarqués ne
s'appliquent qu'au binaire).

## 1. Prérequis d'environnement

| Prérequis | Détail |
|---|---|
| Poste Linux avec interface réseau active | Interface filaire ou Wi-Fi avec trafic réel |
| Capabilities de capture | `sudo setcap cap_net_raw,cap_net_admin=eip <chemin>/sonar` |
| Une seule instance à la fois | Vérifier `pgrep -f "target/(debug\|release)/sonar"` avant chaque session |
| Trafic de fond généra­ble | `ping -f <passerelle>`, `curl`, navigation web |
| Logs accessibles | `~/.local/share/fr.sonar.ssf/logs/` (identifiant `fr.sonar.ssf`) |

## 2. Fichiers de test nécessaires

### 2.1 Fournis dans le dépôt (`src-tauri/test_files/`)

| Fichier | Contenu | Sert aux cas |
|---|---|---|
| `20260703_NP_Labels.csv` | 9 labels + ligne d'en-tête, notation CIDR, label vide (→ « Label? ») | F7 |
| `20260703_NP_Matrice.csv` | Matrice de 30 flux exportée par SONAR | F6, F10 |
| `20260703_NP_Matrice_1000.csv` | Matrice générée de 1 000 flux (232 nœuds attendus) | F6, F8 (charge) |
| `20260709_labels_test.csv`, `20260709_labels.csv` | Variantes de fichiers de labels | F7 |
| `Adresses_IP_Forge.csv` | Adresses forgées pour labellisation | F7 |

### 2.2 À préparer par le testeur

| Fichier | Génération | Sert aux cas |
|---|---|---|
| `vae_simple.pcap` (~500 paquets) | `sudo tcpdump -i <iface> -c 500 -w vae_simple.pcap` pendant une navigation | F5, F8, F10 |
| `vae_gros.pcap` (> 100 000 paquets) | `sudo tcpdump -i <iface> -c 100000 -w vae_gros.pcap` sous trafic soutenu | F5 (charge), F8 |
| `vae_tronque.pcap` | `head -c $(( $(stat -c%s vae_simple.pcap) / 2 )) vae_simple.pcap > vae_tronque.pcap` | F5 (erreur) |
| PCAP tunnelé CAPWAP | Capture sur infrastructure Wi-Fi avec contrôleur (équivalent du `LOC42.pcapng` interne, non versionné — données de mission) | F8 (tunnels) |
| `vae_labels_conflits.csv` | Contenu ci-dessous | F7 (arbitrage) |
| `vae_labels_invalides.csv` | Contenu ci-dessous | F7 (validation) |

`vae_labels_conflits.csv` — même clé (mac, ip) avec deux labels → arbitrage :

```csv
mac,ip,label
aa:bb:cc:dd:ee:01,192.168.1.50,poste-comptabilité
aa:bb:cc:dd:ee:01,192.168.1.50,poste-direction
aa:bb:cc:dd:ee:02,192.168.1.51,imprimante
```

`vae_labels_invalides.csv` — MAC malformée (ligne 2) et colonne manquante (ligne 3) :

```csv
mac,ip,label
aa:bb:cc:dd:e:01,192.168.1.50,mac-invalide
192.168.1.51,colonne-manquante
```

## 3. Cas d'usage par fonction

### F1 — Démarrage de l'application

| | |
|---|---|
| **Pas à pas** | Lancer le binaire release. |
| **Attendu** | Fenêtre « SONAR » 1800×950, bannière ASCII et snapshot d'environnement dans les logs, `[CPU.vue] Listener registered` (webview OK). Aucune ligne `[ERROR]`. |
| **Variante** | `./sonar --sonar-smoke-test` → `SONAR_STARTUP_VALIDATION=OK`, code de sortie 0. |

### F2 — Configuration de la capture (Ctrl+,)

| | |
|---|---|
| **Pas à pas** | Ouvrir le panneau config ; choisir l'interface ; modifier buffer, capacité de canal, timeout, snaplen ; valider ; redémarrer l'application. |
| **Attendu** | Valeurs bornées (buffer 64 Ko–512 Mo, canal 1–1 000 000, timeout 1–10 000 ms, snaplen 64–262 144) ; valeur hors bornes refusée avec message ; la configuration est **persistée** et rechargée au redémarrage (log « Configuration capture chargée depuis le disque »). |

### F3 — Capture live (Ctrl+P / Ctrl+Shift+P)

| | |
|---|---|
| **Préconditions** | F2 réalisée, trafic de fond actif. |
| **Pas à pas** | Démarrer ; observer 60 s ; arrêter. |
| **Attendu** | Barre de statut : compteurs 📥 (reçus) et 📊 (flux) croissants, timer qui tourne, CPU affiché ; graphe et tableau des trames alimentés en continu ; à l'arrêt : threads terminés dans les logs, **aucun** faux « paquets perdus (canal plein) », ligne « N paquet(s) drainés » si trafic soutenu au moment du stop ; les paquets drainés sont comptés dans la matrice exportée. |
| **Cas limite** | Arrêter sous `ping -f` : le drainage doit apparaître dans les logs. |
| **Robustesse** | Après un arrêt backend autonome (ex. interface désactivée pendant la capture : `sudo ip link set <iface> down`), l'UI repasse à l'arrêt (événement `stopped`) et un **redémarrage de capture fonctionne** sans « déjà en cours ». |

### F4 — Filtre BPF (Ctrl+F)

| | |
|---|---|
| **Pas à pas** | Ouvrir le panneau ; tester un preset (ex. Web 80/443) ; vérifier l'aperçu généré ; saisir une IP invalide (`192.168.1`) ; corriger ; appliquer ; démarrer une capture. |
| **Attendu** | Preset → expression correcte (`ip and tcp and (port 80 or port 443)`) ; IP invalide signalée en rouge et bouton Appliquer désactivé ; seul le trafic filtré alimente la matrice. |
| **Variante** | Appliquer un filtre **pendant** une capture → bandeau « Prochain démarrage » ; le filtre ne s'applique qu'au redémarrage. |

### F5 — Import PCAP (Ctrl+O ou vue d'accueil)

| | |
|---|---|
| **Fixtures** | `vae_simple.pcap`, `vae_gros.pcap`, `vae_tronque.pcap`. |
| **Pas à pas** | Ajouter `vae_simple.pcap` ; Ouvrir ; puis recommencer avec plusieurs fichiers à la fois (drag & drop accepté). |
| **Attendu** | Overlay de blocage pendant TOUTE la conversion (y compris multi-fichiers : il ne disparaît pas après le premier fichier) ; à la fin, graphe + matrice peuplés, un compteur `Finished` par fichier dans la barre de statut. |
| **Cas d'erreur (clé)** | Importer d'abord `vae_simple.pcap`, puis tenter `vae_tronque.pcap` : message d'erreur explicite (« Read error in pcap file … ») et **la matrice précédente est intacte** (import transactionnel — vérifier que le graphe affiche toujours les données du premier import). |

### F6 — Import de matrices CSV (fusion multi-sites)

| | |
|---|---|
| **Fixtures** | `20260703_NP_Matrice.csv` (2 copies renommées `site-a.csv` / `site-b.csv`). |
| **Pas à pas** | Importer les deux copies ensemble via le panneau import (section matrices). |
| **Attendu** | 30 flux (fusionnés, pas 60), compteurs cumulés ; export CSV → colonne `origin` = `site-a.csv\|site-b.csv` sur chaque ligne ; un réimport de cette matrice fusionnée **préserve** les origines (pas remplacées par le nouveau nom de fichier). |
| **Cas d'erreur** | CSV non-matrice (ex. un fichier de labels) → erreur explicite, matrice courante intacte. |

### F7 — Import de labels CSV et arbitrage

| | |
|---|---|
| **Fixtures** | `20260703_NP_Labels.csv`, `vae_labels_conflits.csv`, `vae_labels_invalides.csv`. |
| **Pas à pas** | Panneau import (mode CSV) : importer `20260703_NP_Labels.csv`. |
| **Attendu** | Table « contenu importé » remplie (9 lignes, l'en-tête écarté), recherche fonctionnelle, labels appliqués aux nœuds du graphe sans casser sa disposition ; CIDR ramené à l'adresse ; label vide → « Label? ». |
| **Conflits** | Importer `vae_labels_conflits.csv` → import NON bloqué (premier label gagné), bouton « ⚖️ Arbitrer les conflits (1) » ; l'arbitrage propose les deux labels, le choix s'applique au graphe immédiatement. |
| **Validation** | Importer `vae_labels_invalides.csv` → dialogue de conflits détaillant ligne par ligne la MAC malformée et la ligne incomplète ; rien n'est appliqué. |
| **Sémantique assumée** | Réimporter un fichier différent ne retire PAS les labels déjà appliqués à la matrice/graphe (fusion) ; « 🔄 » ne vide que la table du store. |

### F8 — Graphe réseau

| | |
|---|---|
| **Fixtures** | `vae_simple.pcap` ou capture live ; PCAP CAPWAP pour les tunnels ; `20260703_NP_Matrice_1000.csv` pour la charge. |
| **Sélection/labels** | Clic sur un nœud → bandeau bas (ID, nom, label, MAC, IP, degré, trafic, protocoles) ; éditer le label + Entrée → appliqué (visible aussi dans « Afficher les labels ») ; Échap annule. |
| **Sens des flux** | La flèche d'une arête part du **premier émetteur observé** ; une arête ne devient bidirectionnelle que si du trafic passe réellement dans les deux sens. |
| **Alerte multi-MAC** | Si une IP est vue avec deux MAC unicast (test possible en modifiant la MAC d'une VM/du poste : `sudo ip link set <iface> address …` puis re-trafic) : bordure **rouge** du nœud et « ⚠ MACs multiples (2): … » dans le bandeau. |
| **Tunnels** | Sur PCAP CAPWAP : survol d'une arête CAPWAP → famille du tunnel en surbrillance, reste estompé, bandeau « 🚇 tunnel … — N flux liés » ; clic = épinglage (📌), re-clic ou clic sur le fond = libération. |
| **Zoom** | Labels d'arêtes visibles à partir du zoom 1.2, ports à partir de 1.8. |
| **Gravité** | Bouton ON/OFF fige/relance la disposition ForceAtlas2. |
| **Export PNG** | Bouton « ⬇️ Export PNG » → fichier `AAAAMMJJ_network_graph_DR_Matrice.png` à résolution double. |
| **Charge** | Import de la matrice 1 000 lignes → 232 nœuds, rendu fluide, zoom/survol réactifs. |

### F9 — Matrice (tableau) et bascule de vue

| | |
|---|---|
| **Pas à pas** | Basculer entre graphe et matrice ; vérifier les colonnes (MAC/IP/ports/protocoles/compteurs/labels) pendant une capture. |
| **Attendu** | Lignes cohérentes avec la barre de statut 📊 ; labels visibles après import F7. |

### F10 — Exports

| | |
|---|---|
| **Matrice (Ctrl+S)** | Export CSV `AAAAMMJJ_DR_Matrice.csv` ; réimport de ce fichier (F6) → mêmes flux, mêmes compteurs (aller-retour sans perte). |
| **Labels** | Export du fichier de labels → format `mac,ip,label`, champs manquants complétés depuis la matrice ; réimportable (F7). |
| **Logs (Ctrl+L)** | Choisir une destination → les fichiers `DR_SONAR_*.log` de `~/.local/share/fr.sonar.ssf/logs/` y sont copiés (cas historiquement en échec : vérifier qu'il n'y a PAS d'erreur « dossier introuvable »). |

### F11 — Labels appliqués à la matrice

| | |
|---|---|
| **Pas à pas** | Bouton « Afficher les labels » (bandeau bas du graphe). |
| **Attendu** | Modal listant (MAC, IP, label) réellement appliqués, recherche plein texte, fermeture par ✕ ou clic hors du modal. |

### F12 — Reset (Ctrl+Shift+R)

| | |
|---|---|
| **Attendu** | Graphe et matrice vidés, compteurs à zéro, surbrillance/épinglage tunnel libérés ; les labels du store restent disponibles pour la prochaine session. |

### F13 — Mode headless

| | |
|---|---|
| **Pas à pas** | `./sonar --headless` sur une interface configurée ; laisser capturer ; Ctrl+C (raccourci global). |
| **Attendu** | Aucune fenêtre ; capture démarrée automatiquement (logs) ; Ctrl+C arrête proprement. |

### F14 — Divers UI

| | |
|---|---|
| **Raccourcis** | Ctrl+S export CSV, Ctrl+Shift+R reset, Ctrl+P start, Ctrl+Shift+P stop, Ctrl+O import PCAP, Ctrl+, config, Ctrl+F filtre, Ctrl+L logs, Ctrl+Q quitter (avec confirmation). |
| **Menu** | « A propos » → Version (versions de build) et Changelog (dernière release). |
| **Instance unique** | Lancer une seconde instance pendant qu'une tourne → comportement contrôlé (pas de cohabitation silencieuse). |

## 4. Matrice de couverture rapide

| Cas | Nominal | Erreur | Charge | Robustesse |
|---|---|---|---|---|
| Capture (F3) | ✔ | interface coupée | ping -f | drainage, redémarrage |
| Import PCAP (F5) | ✔ | fichier tronqué | vae_gros.pcap | état préservé |
| Import matrice (F6) | ✔ | mauvais format | 1 000 lignes | origin préservée |
| Labels (F7) | ✔ | formats invalides | — | conflits arbitrés |
| Exports (F10) | ✔ | — | — | aller-retour sans perte |
