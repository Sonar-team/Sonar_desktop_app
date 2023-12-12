import socket
import random

def generate_random_ip():
    return ".".join(map(str, (random.randint(0, 255) for _ in range(4))))

def send_udp_packet(ip, port, message):
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    try:
        sock.sendto(message.encode(), (ip, port))
        print(f"Paquet envoyé à {ip}:{port}")
    except Exception as e:
        print(f"Erreur lors de l'envoi au {ip}:{port}: {e}")
    finally:
        sock.close()

# Paramètres
number_of_packets = 1000000
port = 12345
message = "Test UDP"

for _ in range(number_of_packets):
    ip_address = generate_random_ip()
    send_udp_packet(ip_address, port, message)
