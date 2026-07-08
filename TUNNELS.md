# 🚇 Tunnels dans SONAR — vision, modèle et choix techniques

> Document d'architecture. Il fixe **comment SONAR restitue le trafic
> tunnelé** (CAPWAP aujourd'hui ; VXLAN, GRE, … demain) dans la matrice de
> flux, et **pourquoi** ces choix ont été faits. Le côté décodage (comment le
> parser pèle un tunnel) est suivi dans
> [`src-tauri/vendor/packet_parser/TODO_TUNNELS.md`](src-tauri/vendor/packet_parser/TODO_TUNNELS.md).

---

## 🎯 Vision

Un paquet tunnelé transporte **deux conversations réelles à la fois** : celle
du tunnel (AP ↔ contrôleur pour CAPWAP) et celle qui voyage dedans (le client
Wi-Fi qui parle à un serveur). Conformément au pilier « cartographie de
confiance » de la [vision SONAR](VISION.md), les deux doivent apparaître dans
la matrice, **sans interprétation ni perte** : l'analyste doit pouvoir
reconstituer quel trafic est passé par quel tunnel, et vérifier au paquet près
que rien ne manque.

D'où le modèle **père / fils** :

- la **ligne père** est le flux externe du tunnel ; son protocole applicatif
  est le nom du tunnel (ex. `CAPWAP`) ;
- les **lignes fils** sont les conversations internes décapsulées ;
- la colonne **`encap_id`** les relie.

## 📐 Les trois invariants

Toute évolution du traitement des tunnels doit préserver ces trois propriétés
(dans cet ordre de priorité) :

1. **Un identifiant par paire de tunnel.** L'`encap_id` est un hash des deux
   extrémités du flux externe (MAC, IP, port), **ordonnées avant hachage** :
   l'aller et le retour d'un même tunnel portent le **même** identifiant.
   → `tunnel_pair_id()` dans
   [`src-tauri/src/state/capture/capture_handle/messages/capture.rs`](src-tauri/src/state/capture/capture_handle/messages/capture.rs).
2. **Comptabilité exacte père/fils.** Pour chaque tunnel, la somme des paquets
   attribués aux lignes fils est **égale** au compteur de sa ligne père. Un
   paquet de tunnel sans contenu décapsulable (keepalive, contrôle) est compté
   **hors tunnel** (part `encap_id` vide de la ligne père), pour que le père ne
   compte que les paquets réellement porteurs de fils.
3. **Une ligne par flux.** La matrice reste compacte : un flux n'apparaît
   qu'une fois, même s'il traverse plusieurs tunnels. Pas de doublons.

## ⚖️ La tension et sa résolution

Les invariants 2 et 3 sont contradictoires si on ne stocke qu'**un** id par
ligne : un broadcast (ARP du contrôleur) est répliqué dans *tous* les tunnels
d'AP — sur la capture de référence `LOC42.pcapng`, 43 flux broadcast
traversent jusqu'à 81 tunnels chacun.

Deux approches ont été essayées puis **rejetées** :

- **v1 — « premier tunnel vu »** : une ligne par flux, un seul `encap_id`
  (celui du premier tunnel rencontré). Compact, mais 73 tunnels sur 84 se
  retrouvaient **orphelins** (aucune ligne fille) et les comptes par tunnel
  étaient faux. Invariant 2 violé.
- **v2 — une ligne par (flux, tunnel)** : comptes exacts, mais la matrice
  passait de 412 à 2090 lignes sur LOC42 (×5), le broadcast répliqué noyant la
  lecture. Invariant 3 violé.

**Choix retenu** : les compteurs sont tenus **par tunnel en interne**
(`FlowMatrix : HashMap<flux, Vec<(Option<encap_id>, FlowStats)>>`), et
**refusionnés à l'export** en une ligne par flux ; la ventilation par tunnel
est sérialisée dans la colonne `encap_id`. Les trois invariants tiennent.

## 📄 Format de la colonne `encap_id` (extension SFMS)

Trois formes, de la plus courante à la plus rare :

| Forme | Sens |
|---|---|
| *(vide)* | Flux jamais vu dans un tunnel. |
| `8568e7dc25cf8676` | Un seul tunnel, qui porte **tous** les paquets de la ligne. Jointure directe avec la ligne père. |
| `id1:39\|id2:12\|…` | Comptes par tunnel : `id:n` = *n* paquets de cette ligne sont passés par ce tunnel. |

Règles de lecture :

- l'id est le hash de paire sur 16 caractères hexadécimaux ;
- un `id` nu vaut `id:count` (tout le compteur de la ligne) ;
- si la somme des `n` est inférieure au `count` de la ligne, le reste est du
  trafic **hors tunnel** (même flux vu en clair sur le fil, ou keepalives pour
  une ligne père) ;
- les octets ne sont **pas** ventilés par tunnel (seuls les paquets le sont).

Vérification SOC type : pour un tunnel `T`, `somme des n mentionnant T dans
les lignes fils == count attribué à T sur la ligne père CAPWAP`.

Sérialisation et parsing : `format_encap_list()` / `parse_encap_list()` dans
[`src-tauri/src/state/flow_matrix/mod.rs`](src-tauri/src/state/flow_matrix/mod.rs)
— l'import CSV comprend les trois formes, l'aller-retour export → import est
sans perte.

## 🧪 Garde-fous

Deux tests d'intégration rejouent le pipeline complet sur
`src-tauri/test_files/LOC42.pcapng` (7 372 paquets, ~80 tunnels CAPWAP) :

- `import_pcap_tunnel_counts_are_balanced` — équilibre père/fils de chaque
  tunnel, aucun orphelin dans les deux sens ;
- `pcap_matrix_survives_csv_roundtrip` — export CSV → réimport → réexport à
  l'identique (compteurs et ventilation par tunnel).

S'y ajoutent les tests unitaires du hash de paire (indépendance au sens) et du
format de colonne (aller-retour des trois formes). **Tout changement sur les
tunnels doit laisser ces tests verts.**

## 🧭 Étendre à un nouveau tunnel

Côté application, il n'y a **rien à faire** : tout flux dont le parser remplit
`PacketFlow.inner` suit automatiquement ce modèle (hash de paire, ventilation,
export). Le branchement d'un nouveau protocole de tunnel (VXLAN, GRE, GTP-U,
IP-in-IP…) se fait uniquement dans le parser — mode d'emploi et pièges dans
[`TODO_TUNNELS.md`](src-tauri/vendor/packet_parser/TODO_TUNNELS.md).
