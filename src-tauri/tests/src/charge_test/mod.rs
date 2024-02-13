use rand::{distributions::Uniform, Rng};
use std::net::UdpSocket;

#[test]
fn charge_test() {
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Ne peut pas lier au socket local");
    socket
        .set_nonblocking(true)
        .expect("Impossible de passer en mode non-bloquant");

    let mut rng = rand::thread_rng();
    let range = Uniform::new(0, 256);

    // Réduire à un nombre plus gérable, comme 1000
    for _ in 0..1000 {
        let ip = format!(
            "{}.{}.{}.{}",
            rng.sample(&range),
            rng.sample(&range),
            rng.sample(&range),
            rng.sample(&range)
        );
        let addr = format!("{}:12345", ip);

        match socket.send_to(b"Test UDP", &addr) {
            Ok(_) => (), // Considérer d'enlever le println pour éviter de surcharger la sortie
            Err(e) => eprintln!("Erreur lors de l'envoi au {}: {}", addr, e),
        }
    }
}

