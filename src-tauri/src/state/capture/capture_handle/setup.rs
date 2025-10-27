//! Configuration et ouverture d’une capture `pcap`.
//!
//! Ce module expose une fonction unique [`setup_capture`] qui ouvre une
//! capture sur une interface réseau avec les paramètres essentiels
//! (taille du buffer kernel, délai de capture, snaplen, etc.).
//!
//! # Vue d’ensemble
//! - **Interface** : nom de l’interface, tel que renvoyé par `pcap::Device::list()`
//!   (ex. `"eth0"`, `"enp3s0"`).
//! - **Buffer kernel** : mémoire allouée côté kernel pour tamponner les paquets.
//! - **Timeout** : délai (en millisecondes) pour la lecture des paquets (mode non immédiat).
//! - **Snaplen** : nombre maximal d’octets capturés par paquet (troncature au-delà).
//!
//! > Remarque : `immediate_mode(false)` laisse le kernel regrouper les paquets
//! > avant de les livrer (latence plus faible si `true`, mais plus de surcoût CPU).

use pcap::{Capture, Error};

/// Ouvre une capture `pcap` configurée sur l’interface demandée.
///
/// # Paramètres
/// * `config.0` – **Nom de l’interface** (`String`)  
///   Exemple : `"eth0"`, `"wlan0"`. Doit correspondre à un `pcap::Device` existant.
///
/// * `config.1` – **Taille du buffer kernel** (`i32`, en octets)  
///   Capacité du tampon d’acquisition côté kernel (ex. `64 * 1024 * 1024` pour 64 MiB).
///   Une valeur trop faible augmente le risque de pertes sous forte charge.
///
/// * `config.2` – **Capacité du canal applicatif** (`i32`)  
///   Ce champ est transmis par convention à l’application (ex. pour dimensionner un
///   `channel::bounded`). **Non** utilisé par `pcap` lui-même, mais conservé ici
///   pour centraliser la configuration de capture.
///
/// * `config.3` – **Timeout de capture** (`i32`, en millisecondes)  
///   Délai d’attente lors de la lecture : avec `immediate_mode(false)`, `pcap` peut
///   regrouper les paquets jusqu’à ce délai avant de les remettre à l’application.
///   Mettre `1–10 ms` pour une latence basse, `100+ ms` pour moins de réveils.
///
/// * `config.4` – **Snaplen** (`i32`, en octets)  
///   Taille maximale capturée par paquet (le reste est tronqué).  
///   Exemples : `65535` pour capturer “tout”, `262144` pour jumbo frames.
///
/// # Retour
/// Un [`Capture<pcap::Active>`] déjà ouvert et prêt à l’emploi.
///
/// # Erreurs
/// Retourne un [`pcap::Error`] si :
/// - l’interface n’existe pas ou n’est pas accessible ;
/// - l’ouverture échoue (droits insuffisants, permission, OS/driver) ;
/// - un paramètre est refusé par la pile `pcap` (buffer trop grand, etc.).
///
/// # Conseils
/// - **Performance** : augmentez `buffer_size` (config.1) sous forte charge
///   (ex. 64–256 MiB).  
/// - **Perte vs latence** : `immediate_mode(true)` diminue la latence mais
///   peut accroître le coût CPU ; laissez `false` pour laisser le kernel batcher.  
/// - **Snaplen** : mettez `65535` si vous devez décoder complètement les paquets.
///   Réduisez-le si seules les en-têtes vous intéressent (gain CPU/mémoire).
///
/// # Exemples
/// ```no_run
/// use pcap::Device;
/// // Votre configuration applicative :
/// let iface = "eth0".to_string();
/// let buffer_size = 64 * 1024 * 1024; // 64 MiB
/// let channel_capacity = 1024;        // pour vos channels internes (non utilisé par pcap)
/// let timeout_ms = 10;                // faible latence
/// let snaplen = 65_535;               // capture complète
///
/// let cap = setup_capture((iface, buffer_size, channel_capacity, timeout_ms, snaplen))
///     .expect("Impossible d'ouvrir la capture");
///
/// // Exemple : itérer les paquets
/// // while let Ok(packet) = cap.next_packet() {
/// //     // ... traitement ...
/// // }
/// ```
///
/// # Voir aussi
/// - [`pcap::Device::list`] pour découvrir les interfaces disponibles.
/// - [`Capture::setnonblock`] si vous devez basculer en mode non bloquant.
pub fn setup_capture(config: (String, i32, i32, i32, i32)) -> Result<Capture<pcap::Active>, Error> {
    Capture::from_device(config.0.as_str())?
        .promisc(true)
        .snaplen(config.4)
        .immediate_mode(false)
        .timeout(config.3)
        .buffer_size(config.1)
        .open()
}
