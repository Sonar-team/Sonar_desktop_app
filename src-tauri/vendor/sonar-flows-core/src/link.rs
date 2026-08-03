//! Vue Sonar de la couche liaison multi-LINKTYPE de `packet_parser` 7.
//!
//! Depuis packet_parser 7, `PacketFlowOwned::data_link` est un
//! [`LinkLayerOwned`] qui représente fidèlement Ethernet, RAW IP, Linux
//! SLL/SLL2 et IEEE 802.11 — sans inventer de MAC ni d'EtherType. La matrice,
//! le graphe et l'export CSV de Sonar raisonnent en champs textuels ; ce
//! module centralise la projection typé → texte ([`LinkView`]) et son inverse
//! pour le réimport CSV ([`matrix_link_from_text`]), qui doit rester exact
//! sur toute identité produite par un export Sonar (#148, #150).

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::OnceLock;

use packet_parser::owned::{
    DataLinkOwned, LinkLayerOwned, LinkLayerOwnedKind, LinuxSll2LinkOwned, LinuxSllLinkOwned,
    PacketFlowOwned,
};
use packet_parser::parse::data_link::ethertype::Ethertype;
use packet_parser::parse::data_link::vlan_tag::VlanTag;
use packet_parser::{LinkType, LinuxArphrdType, LinuxCookedPacketType, MacAddress};

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

/// Parse l'adresse matérielle variable d'un en-tête Linux cooked. Le slot
/// SLL/SLL2 contient au plus huit octets ; une adresse vide signifie que
/// l'en-tête n'en exposait aucune. Cette forme est volontairement distincte
/// de [`mac_from_text`], qui impose les six octets d'une MAC Ethernet.
pub fn cooked_address_from_text(text: &str) -> Result<Option<Vec<u8>>, String> {
    if text.is_empty() {
        return Ok(None);
    }

    let octets: Vec<&str> = text.split(':').collect();
    if !(1..=8).contains(&octets.len()) {
        return Err(format!(
            "adresse cooked invalide : « {text} » (attendu : 1 à 8 octets hexadécimaux)"
        ));
    }

    octets
        .into_iter()
        .map(|octet| {
            if octet.len() != 2 {
                return Err(format!(
                    "adresse cooked invalide : « {text} » (chaque octet doit avoir deux chiffres)"
                ));
            }
            u8::from_str_radix(octet, 16)
                .map_err(|_| format!("adresse cooked invalide : « {text} »"))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
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

/// Inverse de l'affichage du protocole SLL/SLL2. Les protocoles reconnus ont
/// un nom (`IPv4`, `ARP`…) ; les autres sont affichés par `packet_parser`
/// sous la forme numérique `0xABCD`. La forme Ethernet historique
/// `Unknown (0xABCD)` reste aussi acceptée pour la compatibilité.
pub fn cooked_protocol_from_text(name: &str) -> Option<u16> {
    if let Some(hex) = name.strip_prefix("0x") {
        return (hex.len() == 4)
            .then(|| u16::from_str_radix(hex, 16).ok())
            .flatten();
    }
    ethertype_from_name(name).map(|protocol| protocol.0)
}

/// Construit la représentation canonique de l'identité SFMS cooked. Les
/// détails propres au point d'observation (direction, ARPHRD, longueur
/// déclarée, index d'interface et champ réservé) reçoivent des sentinelles
/// stables et ne segmentent donc pas la matrice. Seuls l'adresse réellement
/// exposée, le protocole et le LINKTYPE restent identifiants.
fn canonical_cooked_link(
    link_type: LinkType,
    source_address: Option<Vec<u8>>,
    protocol: u16,
) -> LinkLayerOwned {
    let address_length = source_address.as_ref().map_or(0, Vec::len);
    if link_type == LinkType::LINUX_SLL {
        LinkLayerOwned::linux_sll(LinuxSllLinkOwned::new(
            LinuxCookedPacketType::HOST,
            LinuxArphrdType(0),
            address_length as u16,
            source_address,
            protocol,
        ))
    } else {
        debug_assert_eq!(link_type, LinkType::LINUX_SLL2);
        LinkLayerOwned::linux_sll2(LinuxSll2LinkOwned::new(
            protocol,
            0,
            0,
            LinuxArphrdType(0),
            LinuxCookedPacketType::HOST,
            address_length as u8,
            source_address,
        ))
    }
}

/// Retourne une clé de matrice normalisée sans modifier le paquet source.
/// Ethernet et RAW restent empruntés sans allocation ; un flux SLL/SLL2 est
/// cloné seulement si ses métadonnées d'observation diffèrent de la forme
/// canonique. Le paquet IPC garde ainsi l'en-tête cooked fidèle reçu du
/// parseur, tandis que la matrice fusionne les observations multi-sondes.
pub fn matrix_flow_identity(flow: &PacketFlowOwned) -> Cow<'_, PacketFlowOwned> {
    let canonical_link = match flow.data_link.kind() {
        LinkLayerOwnedKind::LinuxSll(details) => canonical_cooked_link(
            LinkType::LINUX_SLL,
            details.source_address.clone(),
            details.protocol,
        ),
        LinkLayerOwnedKind::LinuxSll2(details) => canonical_cooked_link(
            LinkType::LINUX_SLL2,
            details.source_address.clone(),
            details.protocol,
        ),
        _ => return Cow::Borrowed(flow),
    };

    if canonical_link == flow.data_link {
        Cow::Borrowed(flow)
    } else {
        let mut canonical = flow.clone();
        canonical.data_link = canonical_link;
        Cow::Owned(canonical)
    }
}

/// Variante possédée de [`matrix_flow_identity`], utilisée par l'import CSV
/// qui possède déjà son flux et peut donc le normaliser sans clone complet.
pub fn into_matrix_flow_identity(mut flow: PacketFlowOwned) -> PacketFlowOwned {
    let canonical_link = match flow.data_link.kind() {
        LinkLayerOwnedKind::LinuxSll(details) => Some(canonical_cooked_link(
            LinkType::LINUX_SLL,
            details.source_address.clone(),
            details.protocol,
        )),
        LinkLayerOwnedKind::LinuxSll2(details) => Some(canonical_cooked_link(
            LinkType::LINUX_SLL2,
            details.source_address.clone(),
            details.protocol,
        )),
        _ => None,
    };
    if let Some(canonical_link) = canonical_link {
        flow.data_link = canonical_link;
    }
    flow
}

/// Reconstruit une couche Ethernet depuis les champs textuels d'une ligne CSV
/// (chemin historique, conservé pour les appelants et tests Ethernet).
///
/// - deux MAC vides avec un protocole IPv4/IPv6 et sans VLAN redeviennent un
///   lien RAW IP (export d'une capture RAW réimporté à l'identique) ;
/// - sinon la ligne redevient Ethernet. Une MAC vide devient l'adresse nulle
///   et un protocole inconnu `Ethertype(0)` : ces cas sont rejetés en amont
///   par [`crate::matrix::FlowMatrixRow::validate`], les replis n'existent
///   que pour rester infaillible ici.
///
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
/// Inverse DLT-aware de [`LinkView::of`] pour une ligne de matrice SFMS.
/// L'identité Ethernet/RAW est reconstruite comme auparavant ; SLL/SLL2
/// utilisent la forme canonique de [`matrix_flow_identity`], sans inventer
/// ni sérialiser les métadonnées locales du point d'observation.
pub fn matrix_link_from_text(
    link_type: LinkType,
    source_address: &str,
    destination_address: &str,
    protocol: &str,
    vlan: Option<VlanTag>,
) -> Result<LinkLayerOwned, String> {
    match link_type {
        LinkType::ETHERNET => Ok(LinkLayerOwned::ethernet(DataLinkOwned {
            destination_mac: mac_from_text(destination_address).unwrap_or(MacAddress([0; 6])),
            source_mac: mac_from_text(source_address).unwrap_or(MacAddress([0; 6])),
            ethertype: ethertype_from_name(protocol).unwrap_or(Ethertype(0)),
            vlan,
        })),
        LinkType::RAW => match protocol {
            "IPv4" => Ok(LinkLayerOwned::raw_ipv4()),
            "IPv6" => Ok(LinkLayerOwned::raw_ipv6()),
            _ => Err(format!(
                "protocole RAW invalide : « {protocol} » (attendu : IPv4 ou IPv6)"
            )),
        },
        LinkType::LINUX_SLL | LinkType::LINUX_SLL2 => {
            let source_address = cooked_address_from_text(source_address)?;
            let protocol = cooked_protocol_from_text(protocol).ok_or_else(|| {
                format!(
                    "protocole cooked invalide : « {protocol} » (attendu : nom connu ou 0xNNNN)"
                )
            })?;
            Ok(canonical_cooked_link(link_type, source_address, protocol))
        }
        _ => Err(format!(
            "type de liaison non reconstructible : {}",
            link_type.0
        )),
    }
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

    /// Une adresse cooked n'est pas forcément une MAC Ethernet : sa longueur
    /// variable et un protocole inconnu doivent traverser le CSV à l'identique.
    #[test]
    fn cooked_text_round_trip_accepts_variable_address_and_unknown_protocol() {
        assert_eq!(
            cooked_address_from_text("01:02:03:04").unwrap(),
            Some(vec![1, 2, 3, 4])
        );
        assert!(cooked_address_from_text("00:01:02:03:04:05:06:07:08").is_err());
        assert_eq!(cooked_protocol_from_text("0xABCD"), Some(0xabcd));

        for link_type in [LinkType::LINUX_SLL, LinkType::LINUX_SLL2] {
            let link = matrix_link_from_text(link_type, "01:02:03:04", "", "0xABCD", None).unwrap();
            let view = LinkView::of(&link);
            assert_eq!(link.link_type(), link_type);
            assert_eq!(view.source_mac, "01:02:03:04");
            assert_eq!(view.destination_mac, "");
            assert_eq!(view.protocol, "0xABCD");
            assert_eq!(view.vlan_id, None);
        }
    }
    #[test]
    fn invalid_mac_is_rejected() {
        assert!(mac_from_text("pas-une-mac").is_none());
        assert!(mac_from_text("").is_none());
        assert!(mac_from_text("aa:bb:cc:dd:ee:ff").is_some());
    }
}
