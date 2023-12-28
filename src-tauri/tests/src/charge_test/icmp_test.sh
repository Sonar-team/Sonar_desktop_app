#!/bin/bash

# The IP address or hostname you want to send ICMP packets to
TARGET="192.168.1.1"

# Number of ICMP packets to send
COUNT=4

# Send ICMP packets
echo "Sending $COUNT ICMP packets to $TARGET"
ping -c $COUNT $TARGET
