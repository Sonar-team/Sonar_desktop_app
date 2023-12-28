#!/bin/bash

# Check if the script is run as root, which is often necessary for network operations
if [[ $EUID -ne 0 ]]; then
    echo "This script must be run as root" 
    exit 1
fi

# The IP address you want to send ARP request
TARGET_IP="192.168.1.1"

# The network interface to use, e.g., eth0, wlan0
INTERFACE="lo"

# Send ARP request
echo "Sending ARP request to $TARGET_IP on $INTERFACE"
arping -I $INTERFACE $TARGET_IP
