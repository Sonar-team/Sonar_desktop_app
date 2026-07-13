# TODO — Décapsulation des tunnels

Suivi du chantier « un paquet encapsulé → plusieurs niveaux de flux ».
Tout passe par le module [`src/parse/tunnel/mod.rs`](src/parse/tunnel/mod.rs).

> Suivi GitHub : [issue #15](https://github.com/Akmot9/Packet-parser/issues/15).

## ✅ Fait

- [x] Champ récursif `PacketFlow.inner: Option<Box<PacketFlow>>` + `flatten()`.
- [x] Refactor `parse_impl` → `parse_layers(data_link, depth)` (réutilisable).
- [x] Garde-fou profondeur (`MAX_TUNNEL_DEPTH = 4`) + dégradation gracieuse
      (jamais d'erreur : `None` → seule la ligne externe est produite).
- [x] **CAPWAP-Data** (UDP 5247) → **IEEE 802.11** (ToDS/FromDS/WDS/QoS, gestion
      du byte-swap Frame Control Cisco) → **LLC/SNAP** → L3.
- [x] Le tunnel est reporté comme protocole applicatif de la ligne externe
      (ex. `"CAPWAP"`).
- [x] 2 tests de référence sur trames réelles (ToDS/HLEN 16 et FromDS/HLEN 8).

---

## 🧭 Rappel : comment brancher un nouveau tunnel

Dans `detect_inner()` : reconnaître le tunnel, peler ses en-têtes, obtenir la
trame interne, puis appeler `PacketFlow::parse_layers(inner_dl, depth + 1)` et
renvoyer `Some(("NOM", inner))`.

Deux formes de trame interne :

- **Interne = Ethernet** (VXLAN, NVGRE, Geneve-transparent) : la trame interne
  commence par un en-tête Ethernet complet →
  `PacketFlow::parse_layers(DataLink::try_from(inner_bytes)?, depth + 1)`.
- **Interne = IP brute** (IP-in-IP, GTP-U, GRE-IP) : pas de L2 interne →
  synthétiser un `DataLink` à MAC vides (`MacAddress([0u8; 6])`), `ethertype`
  = IPv4 (`0x0800`) ou IPv6 (`0x86DD`), `payload` = octets IP internes, puis
  `parse_layers`.

### ⚠️ Point d'architecture à ne pas rater

`detect_inner()` n'est appelé aujourd'hui **que si `transport` est `Some`**
(donc pour les tunnels au-dessus d'UDP/TCP). Les tunnels **niveau IP**
(GRE = proto 47, IP-in-IP = 4/41) ne produisent **pas** de couche `Transport`
→ il faudra ajouter un point de détection dans `parse_layers` quand
`transport` est `None` mais que `internet.payload_protocol` vaut Gre/Ipip.
Prévoir de passer `internet` (ou son `payload` + `payload_protocol`) à la
détection, pas seulement `transport`.

---

## 📋 À faire (un test de référence par tunnel = obligatoire)

> Ne rien merger sans une trame réelle en fixture (golden test), comme pour
> CAPWAP. Fournir le hex complet depuis l'Ethernet.

### Tunnels UDP (hook `transport` existant — le plus simple)

- [ ] **VXLAN** — UDP 4789. En-tête 8 octets (flags 1, réservé 3, VNI 3,
      réservé 1). Interne = **Ethernet**. Fixture requise.
- [ ] **GTP-U** — UDP 2152. En-tête variable (min 8 octets ; +4 si un des flags
      E/S/PN est posé ; puis extension headers). Interne = **IP** (pas de L2).
      Attention au champ « message type » = 255 (G-PDU) pour ne peler que les
      données. Fixture requise.
- [ ] **Geneve** — UDP 6081. En-tête 8 octets + options variables (Opt Len en
      mots de 4). `protocol type` indique Ethernet (0x6558) ou IP. Fixture
      requise.

### Tunnels niveau IP (nécessite le nouveau hook, cf. point d'archi)

- [ ] **IP-in-IP** — IP proto 4 (IPv4) / 41 (IPv6). Interne = **IP** directe.
      Fixture requise.
- [ ] **GRE** — IP proto 47. En-tête 4 octets min (+ champs optionnels selon
      les bits C/K/S) ; le champ `protocol type` (EtherType) dit si l'interne
      est IP (0x0800/0x86DD) ou **Ethernet** (0x6558, NVGRE/transparent
      bridging). Fixture requise.

### Cas particuliers / hors périmètre immédiat

- [ ] **CAPWAP-DTLS** (préambule type 1) : chiffré → **non décapsulable**.
      Déjà géré : on renvoie `None` (pas de récursion), la ligne externe reste.
      À documenter comme limite, rien à coder.
- [ ] **ESP** (proto 50) : chiffré → idem, ne pas récurser.
- [ ] **MPLS** (EtherType 0x8847/0x8848) : encapsulation niveau 2.5, à traiter
      au niveau `DataLink` si besoin un jour.

---

## 🔗 Intégration Sonar (fait côté Sonar par Cyprien)

- [ ] Re-vendorer la crate dans Sonar (`cargo vendor`) après bump de version.
- [ ] Dans le pipeline de capture, itérer `flatten()` → **N lignes de flux par
      paquet**.
- [ ] **Attribution des octets par niveau** pour ne pas doubler le volume :
      ligne externe = taille de la trame complète ; ligne interne = taille du
      segment interne (`inner.data_link.payload.len()` pour la couche L3
      interne). Le `count` (paquets) à +1 par niveau est correct.

---

## 🧪 Rappel tests

- Décoder la trame en amont (Python/Wireshark) pour figer les offsets et les
  adresses attendues, puis asserter externe **et** interne + `flatten().len()`.
- Couvrir les variantes qui changent le pelage (longueurs d'en-tête, sens du
  flux, présence d'options), comme les deux tests CAPWAP ToDS/FromDS.
