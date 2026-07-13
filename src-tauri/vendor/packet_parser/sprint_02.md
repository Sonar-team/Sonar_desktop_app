# Sprint 02 - Architecture de parsing multi-LINKTYPE extensible

> Statut : actif
>
> Date de cadrage : 2026-07-13
>
> Consommateur de reference : Sonar, fidelite du parsing et detection du DLT
> ([issue #150](https://github.com/Sonar-team/Sonar_desktop_app/issues/150)).

## Objectif

Faire evoluer `packet_parser` pour parser un paquet a partir de son type de
liaison explicite, sans supposer Ethernet et sans fabriquer de fausses donnees
de couche 2. L'architecture doit permettre d'ajouter un futur LINKTYPE en
ajoutant son decodeur, ses tests et sa documentation, sans modifier le pipeline
L3/L4/L7 ni les consommateurs.

Le resultat attendu de ce sprint n'est pas seulement un dispatcher. Le modele
retourne par la crate doit representer correctement Ethernet, RAW IP, Linux SLL
et Linux SLL2, y compris lorsque la couche de liaison ne possede pas deux
adresses MAC ou d'EtherType Ethernet.

## Frontiere de responsabilite

`packet_parser` recoit un paquet brut et un LINKTYPE canonique. La crate :

- selectionne le decodeur de liaison ;
- valide et decode l'en-tete de liaison ;
- expose les metadonnees de liaison reellement presentes ;
- transmet le protocole reseau et son payload au pipeline L3/L4/L7 commun ;
- retourne un `PacketFlow` ou une erreur structuree ;
- conserve un chemin zero-copy pour les payloads.

Le lecteur ou moteur de capture reste responsable de :

- lire les conteneurs PCAP/PCAPNG ;
- associer chaque paquet a son interface et a son LINKTYPE ;
- normaliser les valeurs `DLT_*` d'une capture live vers l'espace canonique
  `LINKTYPE_*` lorsque leurs valeurs different ;
- fournir `caplen`, `wirelen`, timestamps, snaplen et statistiques de pertes ;
- garantir l'atomicite d'un import et produire la comptabilite exhaustive.

`packet_parser` ne doit donc dependre ni de libpcap ni d'une crate de lecture
PCAP/PCAPNG.

## Decisions d'architecture

### Identifiant ouvert

`LinkType` reste un newtype numerique ouvert. Une valeur inconnue doit etre
preservee telle quelle et ne doit jamais etre convertie implicitement en
Ethernet.

L'espace numerique public de `LinkType` est celui des `LINKTYPE_*` stockes dans
les fichiers de capture. Les adaptateurs de capture live portent la
responsabilite de la normalisation depuis `DLT_*`.

### Sortie normalisee du decodeur

Un decodeur de liaison ne doit pas construire directement un `PacketFlow`
Ethernet. Il retourne une vue normalisee equivalente a :

```rust
pub(crate) struct DecodedLink<'a> {
    layer: LinkLayer<'a>,
    network_protocol: NetworkProtocol,
    network_payload: &'a [u8],
}
```

Le pipeline commun construit ensuite `PacketFlow` et execute le parsing
L3/L4/L7. Les champs restent prives et sont derives de `LinkLayer` par un
constructeur unique afin qu'un futur decodeur ne puisse pas annoncer un
protocole ou un payload incoherent. Les noms exacts peuvent evoluer, mais cette
separation est un invariant du sprint.

### Modele public de liaison

Le modele emprunte et son equivalent owned doivent :

- exposer le `LinkType` effectivement utilise ;
- representer les adresses de liaison comme optionnelles et typees ;
- conserver les details propres au LINKTYPE sans inventer de MAC ;
- exposer le protocole reseau suivant sans imposer un EtherType Ethernet ;
- conserver le payload sous forme de slice dans le chemin emprunte ;
- utiliser des enums extensibles `#[non_exhaustive]` lorsque de futurs variants
  publics sont attendus ;
- definir et tester un schema de serialisation non ambigu.

Le type Ethernet existant peut rester disponible comme parseur specialise et
comme raccourci de compatibilite. Il ne doit plus etre le seul modele possible
dans `PacketFlow` et `PacketFlowOwned`.

### Catalogue des decodeurs

Un catalogue interne suffit pour ce sprint. Une ABI de plugins dynamiques est
hors perimetre. La crate expose toutefois une capacite de preflight equivalente
a :

```rust
pub fn is_supported(link_type: LinkType) -> bool;
pub fn decoder_info(link_type: LinkType) -> Option<DecoderInfo>;
```

Un consommateur doit pouvoir refuser un LINKTYPE non supporte meme si une
interface PCAPNG ne contient aucun paquet.

## Matrice de support cible

| LINKTYPE | Valeur | Attente de ce sprint |
| --- | ---: | --- |
| Ethernet | 1 | Support complet, sans regression de l'API historique |
| RAW IP | 101 | Support IPv4 et IPv6, sans adresses MAC inventees |
| Linux SLL v1 | 113 | Support complet des champs SLL utiles et du payload reseau |
| Linux SLL v2 | 276 | Support complet des champs SLL2 utiles et du payload reseau |
| IEEE 802.11 | 105 | Modele fidele pour l'inner CAPWAP ; decodeur top-level hors perimetre |
| Bluetooth H4 avec pseudo-header | 201 | Identifie mais refuse explicitement tant qu'aucun decodeur n'existe |
| Toute autre valeur | valeur brute | Preservee puis `UnsupportedLinkType` |

Les futurs decodeurs top-level loopback/NULL, radiotap/802.11 et Bluetooth sont
prepares par l'architecture, mais leur implementation n'est pas requise dans
ce sprint.

## API attendue

- `parse(LinkType, &[u8])` est l'entree explicite principale.
- `PacketFlow::try_from(&[u8])` reste un raccourci Ethernet documente.
- Avec la feature `parse_timing`, une entree
  `parse_timed(LinkType, &[u8], &mut ParseTiming)` offre le meme dispatch et le
  meme resultat fonctionnel que `parse`.
- Un LINKTYPE non supporte est rejete avant toute tentative de decodage des
  octets.
- Les chemins normal et instrumente utilisent les memes decodeurs et le meme
  pipeline de couches.

## Contrat d'erreur

- `UnsupportedLinkType` conserve la valeur numerique recue.
- Une liaison trop courte indique au minimum le LINKTYPE, la taille necessaire
  et la taille recue.
- Une liaison malformee est distinguee d'une liaison tronquee.
- Une corruption L3/L4 reconnue conserve la degradation gracieuse actuelle via
  `PacketFlow::corrupted` ; elle n'est pas transformee arbitrairement en erreur
  fatale de liaison.
- La condition de capture `caplen < wirelen` n'est pas deductible par cette
  crate et reste hors de son contrat.
- Les enums d'erreur destines a evoluer sont `#[non_exhaustive]`.

## Compatibilite et version

La generalisation de `PacketFlow`, `PacketFlowOwned`, de leur serialisation et
des erreurs publiques est une rupture de contrat. La cible de publication est
donc une version majeure `7.0.0`, sauf si l'ancien modele et l'ancien enum
d'erreur restent integralement inchanges derriere une API additive distincte.

Avant publication :

- le choix de migration doit etre explicite dans `CHANGELOG.md` ;
- les README anglais et francais doivent utiliser une version coherent avec
  l'API documentee ;
- les constructions par litteral, `match` exhaustifs et schemas JSON impactes
  doivent etre listes ;
- aucune API publique Ethernet-only ne doit etre presentee comme multi-DLT.

## Plan de travail

### Phase 1 - Stabiliser la frontiere de liaison

- [x] Introduire la sortie normalisee du decodeur.
- [x] Faire passer Ethernet par cette sortie sans changer son resultat.
- [x] Generaliser le modele de liaison emprunte et owned.
- [x] Fixer le schema de serialisation et la strategie de migration majeure.
- [ ] Remplacer les erreurs de liaison provisoires par le contrat final.

### Phase 2 - Fermer la compatibilite Ethernet

- [x] Prouver `parse(LinkType::ETHERNET, bytes) == PacketFlow::try_from(bytes)`
  sur succes et erreur.
- [x] Couvrir IPv4/UDP, IPv6, VLAN valide et tronque, EtherType inconnu,
  corruption L3/L4 et tunnels CAPWAP.
- [x] Tester toutes les longueurs Ethernet de 0 a 13 octets.
- [x] Prouver la parite du chemin `parse_timing`.

### Phase 3 - Ajouter RAW IP

- [x] Ajouter les constantes et le decodeur LINKTYPE_RAW.
- [x] Detecter IPv4/IPv6 depuis la version IP sans fabriquer d'EtherType.
- [x] Tester IPv4, IPv6, paquet vide, version invalide et header tronque.
- [x] Utiliser un paquet extrait de la fixture RAW fournie ; le lecteur
  du conteneur PCAPNG reste un outil de test ou un test d'integration externe.
- [x] Prouver que les slices retournees restent zero-copy.

### Phase 4 - Ajouter Linux SLL et SLL2

- [x] Implementer Linux SLL v1 dans un module distinct.
- [x] Implementer Linux SLL2 dans un module distinct.
- [x] Exposer les champs utiles de SLL v1 sans les convertir en faux Ethernet.
- [x] Exposer les champs utiles de SLL2 sans les convertir en faux Ethernet.
- [x] Router SLL v1 vers IPv4, IPv6, ARP ou un protocole inconnu.
- [x] Router SLL2 vers IPv4, IPv6, ARP ou un protocole inconnu.
- [x] Couvrir les tailles minimales, valeurs futures et payloads tronques de
  SLL v1.
- [x] Couvrir les tailles minimales, champs reserves et payloads tronques de
  SLL2.
- [x] Ajouter des vecteurs SLL v1 minimaux, synthetiques et anonymises.
- [x] Ajouter des vecteurs SLL2 representatifs et anonymises.

### Phase 5 - Capacites, documentation et durcissement

- [x] Exposer le preflight des LINKTYPE supportes.
- [x] Documenter la distinction DLT live / LINKTYPE fichier.
- [x] Ajouter la matrice de support dans les deux README.
- [ ] Completer `METHODE_AJOUT_PROTOCOLE.md` avec la methode d'ajout d'un
  decodeur de liaison.
- [ ] Completer `CHANGELOG.md` et preparer la version majeure.
- [x] Ajouter les cas multi-decodeur aux cibles de fuzzing.

## Etat courant apres MW-05

- [x] `LinkType(u32)` ouvert introduit dans le worktree.
- [x] Entree explicite `parse(LinkType, &[u8])` introduite.
- [x] Decodeur Ethernet isole derriere un dispatcher interne.
- [x] `TryFrom<&[u8]>` redirige vers Ethernet.
- [x] Tests publics minimaux pour l'equivalence Ethernet et la preservation
  d'un LINKTYPE inconnu.
- [x] Chemin Ethernet instrumente redirige vers le dispatcher interne.
- [x] Constantes canoniques ajoutees pour Ethernet, RAW, SLL, Bluetooth H4
  avec pseudo-en-tete et SLL2.
- [x] Preflight `is_supported` et entree `parse_timed` exposes depuis le meme
  catalogue de decodeurs que le chemin normal.
- [x] Refus ferme prouve : SLL2, Bluetooth H4 ou une valeur inconnue ne
  retombent jamais sur le decodeur Ethernet.
- [x] Equivalence des erreurs Ethernet prouvee pour les longueurs 0 a 13.
- [x] Modele de liaison generalise emprunte/owned, avec schema JSON commun.
- [x] Ethernet utilise la sortie normalisee et le pipeline L3/L4/L7 commun.
- [x] L'inner CAPWAP est represente comme IEEE 802.11 (`LINKTYPE 105`) et non
  comme un faux Ethernet.
- [x] RAW IPv4/IPv6 est decode sans faux en-tete L2, faux EtherType ni fausse
  adresse MAC ; le paquet entier devient le payload reseau zero-copy.
- [x] Les vues `RawIpLink` empruntee et owned partagent le meme schema JSON et
  conservent explicitement la version IP.
- [x] Une liaison RAW vide ou avec une version IP invalide produit une erreur
  L2 structuree ; un header IP reconnu puis tronque reste un flux partiel
  marque corrompu au niveau Internet.
- [x] Le chemin RAW normal et le chemin `parse_timing` sont fonctionnellement
  identiques sur les succes, corruptions L3 et erreurs fatales L2.
- [x] SLL v1 preserve son en-tete cooked, son adresse brute optionnelle et son
  protocole, puis reutilise le pipeline reseau commun sans faux Ethernet.
- [x] Les valeurs SLL v1 futures restent acceptables ; une longueur d'adresse
  superieure a huit est preservee et explicitement signalee comme tronquee.
- [x] Le chemin SLL v1 normal et le chemin `parse_timing` sont
  fonctionnellement identiques sur les succes, corruptions L3 et erreurs L2.
- [x] SLL2 preserve son protocole, son champ reserve MBZ, son index
  d'interface, son ARPHRD, son type de paquet et son adresse brute sans faux
  Ethernet.
- [x] Les valeurs SLL2 futures ou inhabituelles restent comptables : reserve
  non nul, index zero ou maximal, ARPHRD/type de paquet inconnus et adresse
  declaree superieure au slot de huit octets.
- [x] Le chemin SLL2 normal et le chemin `parse_timing` sont
  fonctionnellement identiques sur les succes, corruptions L3 et erreurs L2.

Ces cases ne remplacent pas les validations de la Definition de termine.

## Journal des micro-wins

### MW-01 - Frontiere LINKTYPE explicite fermee par defaut (2026-07-13)

Statut : valide localement ; ce journal et le code forment le commit dedie de
ce micro-win sur la branche `agent/multi-linktype-parser`.

- [x] `LinkType` est numerique, serialisable et preserve les valeurs futures.
- [x] `parse`, `parse_timed` et `is_supported` partagent un catalogue unique.
- [x] Ethernet conserve les chemins historique et instrumente.
- [x] Les LINKTYPE identifies sans decodeur restent explicitement refuses.
- [x] `ParseError` est prepare pour evoluer en 7.0 via `#[non_exhaustive]` ;
  l'alias `ParsedPacketError` est conserve.
- [x] README anglais/francais et `CHANGELOG.md` distinguent clairement la 6.0
  publiee de l'API 7.0 en cours.

Validations du micro-win :

- `cargo test` : 637 tests unitaires, 5 tests d'integration et 13 doctests ;
- `cargo test --features parse_timing` : 646 tests unitaires, 7 tests
  d'integration et 13 doctests ;
- `cargo clippy --all-targets --all-features -- -D warnings` : aucune alerte.

Limite volontaire du jalon : le modele public reste Ethernet-only. RAW, SLL et
SLL2 sont connus du catalogue public mais `is_supported` renvoie `false` et le
parsing retourne `UnsupportedLinkType`. La regle de livraison reste donc
active.

### MW-02 - Sortie normalisee et modele LinkLayer extensible (2026-07-13)

Statut : valide localement ; ce journal et le code forment le commit dedie de
ce micro-win sur la branche `agent/multi-linktype-parser`.

- [x] Les decodeurs produisent un `DecodedLink` coherent dont les champs sont
  prives ; un seul pipeline construit ensuite L3/L4/L7 et `PacketFlow`.
- [x] `PacketFlow` et `PacketFlowOwned` utilisent des modeles `LinkLayer`
  extensibles au lieu d'imposer Ethernet.
- [x] `NetworkProtocol` transporte la semantique IPv4, IPv6, ARP, Profinet ou
  une valeur inconnue sans fabriquer d'EtherType.
- [x] Le schema JSON imbrique et tagge est identique pour les vues empruntee et
  owned ; les payloads restent exclus de la serialisation, de l'egalite et du
  hash.
- [x] L'inner CAPWAP expose une liaison IEEE 802.11 canonique (`LINKTYPE 105`)
  avec ses adresses resolues et son vrai protocole LLC/SNAP.
- [x] Les structures et enums publics destines a gagner des champs ou variants
  sont `#[non_exhaustive]`.
- [x] README anglais/francais et `CHANGELOG.md` documentent la rupture de
  modele et de schema reservee a la 7.0.

Validations du micro-win :

- `cargo test` : 641 tests unitaires, 9 tests d'API publique et 13 doctests ;
- `cargo test --features parse_timing` : 650 tests unitaires, 11 tests d'API
  publique et 13 doctests ;
- `cargo test --workspace --all-targets --all-features` : les 650 tests
  unitaires, 11 tests d'API publique, binaires et exemples du workspace sont
  valides ;
- `cargo test --doc --all-features` : 13 doctests valides ;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` :
  aucune alerte ;
- `cargo check --manifest-path fuzz/Cargo.toml --all-targets` : les trois
  cibles fuzz existantes compilent avec le nouveau modele ;
- `cargo fmt --all -- --check` et `git diff --check` : valides.

Limite volontaire du jalon : seul Ethernet dispose encore d'un decodeur
top-level. Le modele IEEE 802.11 est utilise pour le paquet interne CAPWAP,
mais `parse(LinkType::IEEE802_11, ...)` reste refuse. RAW, SLL et SLL2 restent
`UnsupportedLinkType` jusqu'aux micro-wins suivants.

### MW-03 - Decodage LINKTYPE_RAW IPv4/IPv6 (2026-07-13)

Statut : valide localement dans une copie propre de `ea21dc6` contenant
uniquement ce micro-win ; le fichier `src/parse/mod.rs`, modifie en parallele
par une autre session, est explicitement exclu de ce jalon et de son commit.

- [x] Le catalogue active `LINKTYPE_RAW` (`101`) et route IPv4 ou IPv6 depuis
  le premier nibble, sans fabriquer de MAC ni d'EtherType.
- [x] `RawIpLink` conserve le paquet complet par emprunt zero-copy ;
  `RawIpLinkOwned` et la vue empruntee produisent le meme JSON tagge.
- [x] `LinkLayerError` distingue la troncature L2 (paquet vide) d'une version
  IP invalide et conserve le LINKTYPE, les tailles ou le nibble observes.
- [x] Une version 4 ou 6 reconnue est transmise une seule fois au pipeline
  commun : un header IP tronque retourne un `PacketFlow` marque corrompu au
  niveau Internet plutot qu'une erreur fatale de liaison.
- [x] Le vecteur IPv4/ICMP provient de `raw_ip.pcapng` fourni par Sonar ; le
  vecteur IPv6/UDP est synthetique et reutilise une fixture deja eprouvee.
- [x] Les tests prouvent l'absence de fallback Ethernet, la preservation
  zero-copy, la parite borrowed/owned et la parite normale/instrumentee.

Validations du micro-win :

- `cargo test` : 648 tests unitaires, 14 tests d'API publique et 13 doctests ;
- `cargo test --features parse_timing` : 657 tests unitaires, 18 tests d'API
  publique et 13 doctests ;
- `cargo test --workspace --all-targets --all-features` : les 657 tests
  unitaires, 18 tests d'API publique, binaires et exemples du workspace sont
  valides ;
- `cargo test --doc --all-features` : 13 doctests valides ;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` :
  aucune alerte ;
- `cargo check --manifest-path fuzz/Cargo.toml --all-targets` : les trois
  cibles fuzz existantes compilent avec le decodeur RAW ;
- `cargo fmt --all -- --check` et `git diff --check` : valides.

Limites volontaires du jalon : SLL et SLL2 restent non supportes, les erreurs
Ethernet historiques ne sont pas encore migrees vers le contrat L2 final, et
aucune vraie fixture RAW IPv6 n'est encore disponible. Les checksums de la
fixture IPv4 fournie sont invalides selon Tshark ; elle sert donc uniquement a
valider le parsing structurel, pas une future verification des checksums. Les
cibles fuzz compilent, mais leur corpus multi-decodeur sera complete en phase
5.

### MW-04 - Decodage LINKTYPE_LINUX_SLL v1 (2026-07-13)

Statut : valide localement dans une copie propre de `2f2b72c` contenant
uniquement ce micro-win. Les modifications concurrentes de
`src/parse/mod.rs` et `src/checks/application/quic.rs` sont explicitement
exclues de ce jalon et de son commit.

- [x] Le catalogue active `LINKTYPE_LINUX_SLL` (`113`) et decode exactement
  son en-tete fixe de 16 octets en big-endian.
- [x] `LinuxSllLink` preserve les valeurs numeriques de type de paquet,
  ARPHRD, longueur d'adresse et protocole sans fabriquer de MAC, de
  destination ou d'EtherType.
- [x] L'adresse est optionnelle et empruntee directement depuis l'en-tete. Une
  longueur declaree superieure aux huit octets disponibles reste preservee,
  expose les huit octets capturables et active `address_is_truncated()`.
- [x] IPv4, IPv6, ARP et Profinet reutilisent le pipeline reseau commun ; un
  protocole inconnu reste un flux L2 propre avec sa valeur brute.
- [x] Un en-tete de moins de 16 octets produit une erreur L2 structuree. Un
  protocole reseau reconnu avec un payload tronque conserve SLL et produit un
  flux partiel marque corrompu au niveau Internet.
- [x] Les vues empruntee et owned partagent le meme schema JSON ; adresse et
  payload prouvent le comportement zero-copy attendu, et `parse_timing` reste
  fonctionnellement equivalent.
- [x] Un paquet loopback reel, non sensible, est extrait de la fixture Sonar.
  Les vecteurs IPv6, ARP, protocole inconnu et valeurs futures sont
  synthetiques ou anonymises ; le lecteur PCAPNG reste hors de la crate.

Validations du micro-win dans la copie propre :

- `cargo test` : 654 tests unitaires, 20 tests d'API publique et 13 doctests ;
- `cargo test --features parse_timing` : 663 tests unitaires, 26 tests d'API
  publique et 13 doctests ;
- `cargo test --workspace --all-targets --all-features` : les 663 tests
  unitaires, 26 tests d'API publique, binaires et exemples du workspace sont
  valides ;
- `cargo test --doc --all-features` : 13 doctests valides ;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` :
  aucune alerte ;
- `cargo check --manifest-path fuzz/Cargo.toml --all-targets` : les trois
  cibles fuzz existantes compilent avec le decodeur SLL v1 ;
- `cargo fmt --all -- --check` et `git diff --check` : valides.

Limites volontaires du jalon : SLL2 reste non supporte et aucune fixture SLL2
n'est encore disponible. Les variantes VLAN, Netlink et les dissections
dependant d'un ARPHRD precis restent hors perimetre ; leurs valeurs sont
cependant preservees. Les troncatures SLL v1 sont couvertes par des vecteurs
synthetiques, car la fixture fournie ne contient aucun paquet tronque. Le `25`
affiche par les outils Wireshark est leur identifiant d'encapsulation interne ;
l'API de la crate conserve le LINKTYPE canonique `113`. Le contrat d'erreur
Ethernet final et le corpus fuzz multi-decodeur restent a terminer.

### MW-05 - Decodage LINKTYPE_LINUX_SLL2 (2026-07-13)

Statut : valide localement dans une copie propre de `d050a60` contenant
uniquement le diff de ce micro-win. Les quatre commits concurrents QUIC, IPv4,
fixtures et fuzz (`09537df` a `d050a60`) font partie de la base integree, mais
pas du diff ni du commit MW-05.

- [x] Le catalogue active `LINKTYPE_LINUX_SLL2` (`276`) dans un decodeur
  distinct de SLL v1 et decode exactement son en-tete fixe de 20 octets.
- [x] Les champs multi-octets sont lus en big-endian aux offsets officiels ;
  le type de paquet et la longueur d'adresse restent les octets SLL2 des
  offsets 10 et 11.
- [x] `LinuxSll2Link` preserve le protocole, `reserved_mbz`, l'index
  d'interface `u32`, l'ARPHRD, le type de paquet, la longueur declaree et les
  octets d'adresse disponibles. Le payload commence a l'offset 20 et reste
  zero-copy.
- [x] Un champ reserve non nul reste visible et `reserved_is_zero()` permet de
  le comptabiliser sans perdre un paquet que Tshark sait encore dissecter.
  L'index d'interface reste numerique et n'est pas resolu sur la machine
  d'analyse.
- [x] Une adresse declaree au-dela du slot fixe de huit octets est bornee mais
  sa longueur est preservee et `address_is_truncated()` signale l'ecart.
- [x] IPv4, IPv6, ARP et Profinet reutilisent le pipeline commun ; un protocole
  inconnu reste un flux L2 propre avec sa valeur brute.
- [x] Toute taille de 0 a 19 octets produit une troncature L2 structuree. Un
  protocole reseau reconnu avec un payload tronque conserve SLL2 et produit un
  flux partiel marque corrompu au niveau Internet.
- [x] Les modeles emprunte et owned partagent le meme schema JSON ;
  `interface_index` participe a l'identite du flux, tandis que payload et
  padding en sont exclus.
- [x] Les 63 captures disponibles ont ete auditees sans trouver de SLL2. Le
  vecteur IPv4 anonymise utilise des adresses TEST-NET et a ete valide avec
  Tshark 4.6.6 ; IPv6, ARP, inconnu, valeurs futures et troncatures sont
  synthetiques.

Validations du micro-win dans la copie propre :

- `cargo test` : 664 tests unitaires, 26 tests d'API publique et 13 doctests ;
- `cargo test --features parse_timing` : 673 tests unitaires, 34 tests d'API
  publique et 13 doctests ;
- `cargo test --workspace --all-targets --all-features` : les 673 tests
  unitaires, 34 tests d'API publique, binaires et exemples du workspace sont
  valides ;
- `cargo test --doc --all-features` : 13 doctests valides ;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` :
  aucune alerte ;
- `cargo check --manifest-path fuzz/Cargo.toml --all-targets` : les cinq
  cibles fuzz, dont `parse_linktype` couvrant Ethernet, RAW, SLL, SLL2 et les
  valeurs refusees, compilent avec le decodeur SLL2 ;
- `rustfmt --check` limite aux fichiers Rust MW-05 et
  `git diff --cached --check` : valides. Le `cargo fmt --all -- --check`
  global signale deux ecarts preexistants dans les commits concurrents, dans
  `src/parse/mod.rs` et `src/checks/application/quic.rs`.

Limites volontaires du jalon : aucune vraie fixture SLL2 n'est disponible et
le vecteur de regression reste donc synthetique. L'encapsulation WTAP interne
affichee par Tshark 4.6.6 vaut `210`, tandis que l'API conserve le LINKTYPE
canonique `276`. Les dissections dependantes de VLAN, Netlink, GRE, radiotap,
LLC ou d'un ARPHRD particulier restent hors perimetre ; leurs valeurs sont
preservees sans faux decodage. La crate ne resout pas les noms d'interface de
la machine de capture. Le contrat d'erreur Ethernet final reste a terminer ;
la cible fuzz multi-decodeur existe desormais, mais son corpus de regression
doit encore etre execute et stabilise avant publication. Les deux ecarts de
formatage des commits concurrents doivent egalement etre corriges hors MW-05.

## Definition de termine

- [x] Aucun chemin de parsing ne suppose Ethernet sans le demander
  explicitement.
- [x] Ethernet, RAW, SLL et SLL2 produisent des representations fideles, sans
  MAC, EtherType ou metadonnees inventes.
- [x] Un LINKTYPE inconnu ou non supporte echoue de facon deterministe en
  conservant son identifiant.
- [x] Ajouter un futur LINKTYPE ne demande pas de modifier le pipeline
  L3/L4/L7 ni les consommateurs de `PacketFlow`.
- [x] Les modeles emprunte et owned ainsi que leur serialisation representent
  les memes metadonnees de liaison ; les payloads exclus restent empruntes.
- [x] Les chemins normal et `parse_timing` sont fonctionnellement identiques.
- [x] Les tests de compatibilite Ethernet, RAW, SLL, SLL2, erreurs et zero-copy
  sont presents.
- [x] La matrice de support et la migration sont documentees en anglais et en
  francais.
- [ ] `cargo fmt --check` passe sur le HEAD integre.
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passe.
- [x] `cargo test` passe.
- [x] `cargo test --features parse_timing` passe.
- [ ] Les cibles de fuzzing du parsing paquet et des nouveaux decodeurs ne
  paniquent pas sur le corpus de regression.

## Hors perimetre

- Lecture ou ecriture des conteneurs PCAP/PCAPNG.
- Comptage des paquets, pertes noyau/application, timestamps et metadonnees
  d'interface.
- Atomicite de l'etat Sonar et presentation CLI/desktop du rapport.
- Parite exhaustive avec tous les dissecteurs Tshark.
- ABI stable pour charger des decodeurs tiers dynamiquement.
- Decodage complet Bluetooth, radiotap/802.11, NULL/loopback ou autres
  LINKTYPE non listes dans la matrice cible.

## Regle de livraison

Ne pas publier la nouvelle API tant que le contrat d'erreur L2 final n'est pas
applique de facon coherente et que le corpus de regression fuzz n'est pas
execute puis stabilise. Ethernet, RAW, SLL v1 et SLL2 ferment desormais la
matrice requise par Sonar et la cible `parse_linktype` couvre leur dispatch ;
les micro-wins suivants doivent durcir ces quatre chemins sans regression.
