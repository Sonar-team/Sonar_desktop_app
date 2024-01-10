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

    for _ in 0..10_000_000 {
        let ip = format!(
            "{}.{}.{}.{}",
            rng.sample(&range),
            rng.sample(&range),
            rng.sample(&range),
            rng.sample(&range)
        );
        let addr = format!("{}:12345", ip);

        match socket.send_to(b"Test UDP", &addr) {
            Ok(_) => println!("Paquet envoyé à {}", addr),
            Err(e) => eprintln!("Erreur lors de l'envoi au {}: {}", addr, e),
        }
    }
}
