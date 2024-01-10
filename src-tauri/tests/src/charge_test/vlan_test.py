from scapy.all import Ether, PPPoED , sendp, Dot1Q, IP

from dotenv import load_dotenv
import os

# Load environment variables
load_dotenv()

# Access variables
src_ip = os.getenv('SRC_IP')
dst_ip = os.getenv('DST_IP')

def send_vlan_packet(interface, vlan_id, src_mac, dst_mac, src_ip, dst_ip):
    packet = Ether(src=src_mac, dst=dst_mac)
    packet = packet / Dot1Q(vlan=vlan_id)
    packet = packet / IP(src=src_ip, dst=dst_ip)  # You can add more layers as needed

    sendp(packet, iface=interface)

# Example usage
send_vlan_packet(
    interface="enx0a87c76ee9f1",   # Replace with your network interface
    vlan_id=100,        # VLAN ID
    src_mac="00:01:02:03:04:05",  # Replace with source MAC address
    dst_mac="ff:ff:ff:ff:ff:ff",  # Replace with destination MAC address or broadcast
    src_ip=src_ip,        # Using variable from .env file
    dst_ip=dst_ip         # Using variable from .env file
)
# https://www.youtube.com/watch?v=Nlyx5lFQR34   

def send_qinq_packet(interface, outer_vlan, inner_vlan, src_mac, dst_mac):
    # Create an Ethernet frame
    packet = Ether(src=src_mac, dst=dst_mac)
    
    # Add the outer VLAN tag (S-VLAN)
    packet = packet / Dot1Q(vlan=outer_vlan, type=0x88a8)  # 0x88a8 is the EtherType for S-VLAN

    # Add the inner VLAN tag (C-VLAN)
    packet = packet / Dot1Q(vlan=inner_vlan)
    
    # You can add more layers after the VLAN tags if needed
    # For example, packet = packet / IP(src="1.2.3.4", dst="5.6.7.8") / TCP()

    # Send the packet
    sendp(packet, iface=interface)

# Example usage
send_qinq_packet(
    interface="enx0a87c76ee9f1",          # Replace with your network interface
    outer_vlan=100,            # Outer VLAN ID (S-VLAN)
    inner_vlan=200,            # Inner VLAN ID (C-VLAN)
    src_mac="00:01:02:03:04:06",  # Replace with source MAC address
    dst_mac="ff:ff:ff:ff:ff:ff"   # Replace with destination MAC address or broadcast
)

def send_pppoed_packet(interface, src_mac, dst_mac):
    # Create an Ethernet frame
    packet = Ether(src=src_mac, dst=dst_mac)

    # Add a PPPoE Discovery layer
    packet = packet / PPPoED()

    # Send the packet
    sendp(packet, iface=interface)

# Example usage
send_pppoed_packet(
    interface="enx0a87c76ee9f1",             # Replace with your network interface
    src_mac="00:01:02:03:04:05",  # Replace with source MAC address
    dst_mac="ff:ff:ff:ff:ff:ff"   # Replace with destination MAC address or broadcast
)

import socket
import ssl

def ssl_client(host, port, message):
    context = ssl.create_default_context()

    with socket.create_connection((host, port)) as sock:
        with context.wrap_socket(sock, server_hostname=host) as sslsock:
            sslsock.send(message.encode())

# Example usage
ssl_client('www.example.com', 443, 'Hello, SSL!')



