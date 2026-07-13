//! Vue Sonar de la couche liaison multi-LINKTYPE de `packet_parser` 7.
//!
//! Depuis packet_parser 7, `PacketFlowOwned::data_link` est un
//! [`LinkLayerOwned`] qui représente fidèlement Ethernet, RAW IP, Linux
//! SLL/SLL2 et IEEE 802.11 — sans inventer de MAC ni d'EtherType. La matrice,
//! le graphe et l'export CSV de Sonar raisonnent en champs textuels ; ce
//! module centralise la projection typé → texte ([`LinkView`]) et son inverse
//! pour le réimport CSV ([`ethernet_link_from_text`]), qui doit rester exact
//! sur toute valeur produite par un export Sonar (#148).

use std::collections::HashMap;
use std::sync::OnceLock;

use packet_parser::MacAddress;
use packet_parser::owned::{DataLinkOwned, LinkLayerOwned, LinkLayerOwnedKind};
use packet_parser::parse::data_link::ethertype::Ethertype;
use packet_parser::parse::data_link::vlan_tag::VlanTag;

/// Projection textuelle de la couche liaison, telle que la matrice, le graphe
/// et le CSV la consomment. Pour Ethernet, chaque champ est identique octet
/// pour octet à l'ancien `DataLinkOwned` textuel de packet_parser 6.
#[derive(Debug, Clone)]
pub struct LinkView {
    pub source_mac: String,
    pub destination_mac: String,
    /// Nom du protocole de liaison (ex-`ethertype` : « IPv4 », « ARP »,
    /// « Unknown (0xABCD) »…).
    pub protocol: String,
    pub vlan_id: Option<u16>,
}

impl LinkView {
    /// Projette la couche liaison en champs textuels. Les LINKTYPE sans
    /// adresse (RAW IP) ou à adresse source unique (SLL/SLL2) produisent des
    /// MAC vides plutôt qu'inventées : les consommateurs traitent déjà la MAC
    /// vide (labels, endpoints observés).
    pub fn of(link: &LinkLayerOwned) -> Self {
        match link.kind() {
            LinkLayerOwnedKind::Ethernet(frame) => Self {
                source_mac: frame.source_mac.to_string(),
                destination_mac: frame.destination_mac.to_string(),
                protocol: frame.ethertype.name(),
                vlan_id: frame.vlan.as_ref().map(|v| v.id),
            },
            LinkLayerOwnedKind::Ieee80211(frame) => Self {
                source_mac: frame.source_mac.to_string(),
                destination_mac: frame.destination_mac.to_string(),
                protocol: frame.snap_protocol.name(),
                vlan_id: None,
            },
            LinkLayerOwnedKind::LinuxSll(details) => Self {
                source_mac: details
                    .source_address
                    .as_deref()
                    .map(format_hw_address)
                    .unwrap_or_default(),
                destination_mac: String::new(),
                protocol: link.network_protocol().to_string(),
                vlan_id: None,
            },
            LinkLayerOwnedKind::LinuxSll2(details) => Self {
                source_mac: details
                    .source_address
                    .as_deref()
                    .map(format_hw_address)
                    .unwrap_or_default(),
                destination_mac: String::new(),
                protocol: link.network_protocol().to_string(),
                vlan_id: None,
            },
            // RAW IP et variants futurs (`#[non_exhaustive]`) : pas d'adresse
            // de liaison réelle.
            _ => Self {
                source_mac: String::new(),
                destination_mac: String::new(),
                protocol: link.network_protocol().to_string(),
                vlan_id: None,
            },
        }
    }
}

/// Adresse matérielle en hexadécimal séparé par `:` (même forme que l'ancien
/// affichage MAC), quelle que soit sa longueur (SLL n'impose pas 6 octets).
fn format_hw_address(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(":")
}

/// Parse une MAC textuelle `aa:bb:cc:dd:ee:ff`. `None` si la forme est
/// invalide — la validation d'import s'appuie dessus pour rejeter la ligne
/// avec un message précis au lieu de dégrader silencieusement (#148).
pub fn mac_from_text(text: &str) -> Option<MacAddress> {
    MacAddress::try_from(text.to_string()).ok()
}

/// Inverse exact de [`Ethertype::name`] : nom connu (« IPv4 », « LLDP »…) ou
/// forme « Unknown (0xABCD) ». `None` pour toute autre chaîne.
pub fn ethertype_from_name(name: &str) -> Option<Ethertype> {
    if let Some(hex) = name
        .strip_prefix("Unknown (0x")
        .and_then(|s| s.strip_suffix(')'))
    {
        return u16::from_str_radix(hex, 16).ok().map(Ethertype);
    }

    // Table inverse construite en balayant `static_name` sur tout l'espace
    // u16 : aucune duplication de la table amont, donc aucune dérive possible.
    static NAMES: OnceLock<HashMap<&'static str, u16>> = OnceLock::new();
    let names = NAMES.get_or_init(|| {
        let mut map = HashMap::new();
        for code in 0..=u16::MAX {
            if let Some(name) = Ethertype(code).static_name() {
                map.entry(name).or_insert(code);
            }
        }
        map
    });
    names.get(name).map(|&code| Ethertype(code))
}

/// Reconstruit une couche liaison depuis les champs textuels d'une ligne CSV
/// (chemin inverse de [`LinkView::of`]).
///
/// - deux MAC vides avec un protocole IPv4/IPv6 et sans VLAN redeviennent un
///   lien RAW IP (export d'une capture RAW réimporté à l'identique) ;
/// - sinon la ligne redevient Ethernet. Une MAC vide devient l'adresse nulle
///   et un protocole inconnu `Ethertype(0)` : ces cas sont rejetés en amont
///   par [`crate::matrix::FlowMatrixRow::validate`], les replis n'existent
///   que pour rester infaillible ici.
///
/// Limite connue : une capture SLL/SLL2 exportée ne se réimporte pas sous sa
/// forme SLL (packet_parser n'expose pas de constructeur owned SLL) ; la
/// ligne redevient Ethernet et ne fusionnera pas avec une capture SLL live.
pub fn ethernet_link_from_text(
    source_mac: &str,
    destination_mac: &str,
    protocol: &str,
    vlan: Option<VlanTag>,
) -> LinkLayerOwned {
    if source_mac.is_empty() && destination_mac.is_empty() && vlan.is_none() {
        match protocol {
            "IPv4" => return LinkLayerOwned::raw_ipv4(),
            "IPv6" => return LinkLayerOwned::raw_ipv6(),
            _ => {}
        }
    }

    LinkLayerOwned::ethernet(DataLinkOwned {
        destination_mac: mac_from_text(destination_mac).unwrap_or(MacAddress([0; 6])),
        source_mac: mac_from_text(source_mac).unwrap_or(MacAddress([0; 6])),
        ethertype: ethertype_from_name(protocol).unwrap_or(Ethertype(0)),
        vlan,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    /// L'aller-retour texte → lien → texte est exact pour une ligne Ethernet.
    #[test]
    fn ethernet_round_trip_is_exact() {
        let link = ethernet_link_from_text("aa:bb:cc:dd:ee:ff", "11:22:33:44:55:66", "IPv4", None);
        let view = LinkView::of(&link);
        assert_eq!(view.source_mac, "aa:bb:cc:dd:ee:ff");
        assert_eq!(view.destination_mac, "11:22:33:44:55:66");
        assert_eq!(view.protocol, "IPv4");
        assert_eq!(view.vlan_id, None);
    }

    /// Tout nom produit par `Ethertype::name` doit être inversible, y compris
    /// la forme « Unknown (0x…) » : c'est la garantie de réimport CSV (#148).
    #[test]
    fn every_ethertype_name_is_invertible() {
        for code in [0x0000u16, 0x0800, 0x86DD, 0x0806, 0x88CC, 0x8892, 0xABCD] {
            let name = Ethertype(code).name();
            assert_eq!(
                ethertype_from_name(&name),
                Some(Ethertype(code)),
                "nom non inversible : {name}"
            );
        }
    }

    /// Une ligne RAW IP (MACs vides, protocole IPv4/IPv6) se réimporte sous
    /// sa forme RAW, identique à une capture RAW live.
    #[test]
    fn raw_ip_round_trip_keeps_raw_kind() {
        let link = ethernet_link_from_text("", "", "IPv4", None);
        assert_eq!(link, LinkLayerOwned::raw_ipv4());
        let view = LinkView::of(&link);
        assert_eq!(view.source_mac, "");
        assert_eq!(view.destination_mac, "");
        assert_eq!(view.protocol, "IPv4");
    }

    #[test]
    fn invalid_mac_is_rejected() {
        assert!(mac_from_text("pas-une-mac").is_none());
        assert!(mac_from_text("").is_none());
        assert!(mac_from_text("aa:bb:cc:dd:ee:ff").is_some());
    }
}
