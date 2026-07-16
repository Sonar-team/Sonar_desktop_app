# TShark analysis: `pcap_to_matrice.pcapng`

- Generator: `script/pcap/analyze-tshark.sh`
- TShark: `TShark (Wireshark) 4.6.6.`
- Time zone: `UTC`
- SHA-256: `331608da836422ebe2105de075d70532bb7f7990ea41249b351464edd710eb3b`

## Aggregate verification

| Source | Rows/frames | Packet count | Total bytes | Result |
|---|---:|---:|---:|---|
| PCAP | 15 | 15 | 1008 | - |
| `pcap_to_matrice.csv` | 12 | 15 | 1008 | **PASS** |

## Capture metadata

```text
File name:           pcap_to_matrice.pcapng
File type:           Wireshark/... - pcapng
File encapsulation:  Ethernet
File timestamp precision:  nanoseconds (9)
Packet size limit:   file hdr: (not set)
Number of packets:   15
File size:           1848 bytes
Data size:           1008 bytes
Capture duration:    0.583263375 seconds
Earliest packet time: 2026-07-16 07:24:08.744995204
Latest packet time:   2026-07-16 07:24:09.328258579
Data byte rate:      1728 bytes/s
Data bit rate:       13 kbps
Average packet size: 67.20 bytes
Average packet rate: 25 packets/s
SHA256:              331608da836422ebe2105de075d70532bb7f7990ea41249b351464edd710eb3b
SHA1:                0604aa71381b8b2a9514a6b41f7a906c111a539e
Strict time order:   True
Capture hardware:    Intel(R) Core(TM) i7-10875H CPU @ 2.30GHz (with SSE4.2)
Capture oper-sys:    Linux 6.8.0-59-generic
Capture application: Dumpcap (Wireshark) 4.6.6
Number of interfaces in file: 1
Interface #0 info:
                     Name = wlp0s20f3
                     Encapsulation = Ethernet (1 - ether)
                     Capture length = 262144
                     Time precision = nanoseconds (9)
                     Time ticks per second = 1000000000
                     Time resolution = 0x09
                     Operating system = Linux 6.8.0-59-generic
                     Number of stat entries = 1
                     Number of packets = 15
```

## Protocol hierarchy

```text

===================================================================
Protocol Hierarchy Statistics
Filter:

frame                                    frames:15 bytes:1008
  eth                                    frames:15 bytes:1008
    ip                                   frames:12 bytes:828
      tcp                                frames:12 bytes:828
        tls                              frames:1 bytes:90
    arp                                  frames:3 bytes:180
===================================================================
```

## Packet inventory

```text
frame.number|frame.time_relative|frame.len|_ws.col.protocol|eth.src|eth.dst|arp.opcode|arp.src.proto_ipv4|arp.dst.proto_ipv4|ip.src|ip.dst|tcp.srcport|tcp.dstport|tcp.flags.str|tcp.len|_ws.col.info
"1"|"0.000000000"|"90"|"TLSv1.2"|"64:6e:e0:ea:fa:83"|"e0:d3:62:6a:a2:5a"||||"192.168.1.15"|"160.79.104.10"|"41408"|"443"|"·······AP···"|"24"|"Application Data"
"2"|"0.000073631"|"66"|"TCP"|"64:6e:e0:ea:fa:83"|"e0:d3:62:6a:a2:5a"||||"192.168.1.15"|"160.79.104.10"|"41408"|"443"|"·······A·R··"|"0"|"41408 → 443 [RST, ACK] Seq=25 Ack=1 Win=453 Len=0 TSval=49288905 TSecr=1775178299"
"3"|"0.010758479"|"66"|"TCP"|"64:6e:e0:ea:fa:83"|"e0:d3:62:6a:a2:5a"||||"192.168.1.15"|"160.79.104.10"|"40998"|"443"|"·······A····"|"0"|"40998 → 443 [ACK] Seq=1 Ack=1 Win=3469 Len=0 TSval=49288916 TSecr=1775219879"
"4"|"0.025614682"|"66"|"TCP"|"e0:d3:62:6a:a2:5a"|"64:6e:e0:ea:fa:83"||||"160.79.104.10"|"192.168.1.15"|"443"|"41408"|"·······A····"|"0"|"443 → 41408 [ACK] Seq=1 Ack=25 Win=4280 Len=0 TSval=1775220914 TSecr=49288905"
"5"|"0.025742497"|"54"|"TCP"|"64:6e:e0:ea:fa:83"|"e0:d3:62:6a:a2:5a"||||"192.168.1.15"|"160.79.104.10"|"41408"|"443"|"·········R··"|"0"|"41408 → 443 [RST] Seq=25 Win=0 Len=0"
"6"|"0.030788210"|"66"|"TCP"|"e0:d3:62:6a:a2:5a"|"64:6e:e0:ea:fa:83"||||"160.79.104.10"|"192.168.1.15"|"443"|"41408"|"·······A···F"|"0"|"443 → 41408 [FIN, ACK] Seq=1 Ack=25 Win=4280 Len=0 TSval=1775220917 TSecr=49288905"
"7"|"0.030886147"|"54"|"TCP"|"64:6e:e0:ea:fa:83"|"e0:d3:62:6a:a2:5a"||||"192.168.1.15"|"160.79.104.10"|"41408"|"443"|"·········R··"|"0"|"41408 → 443 [RST] Seq=25 Win=0 Len=0"
"8"|"0.054586476"|"78"|"TCP"|"e0:d3:62:6a:a2:5a"|"64:6e:e0:ea:fa:83"||||"160.79.104.10"|"192.168.1.15"|"443"|"40998"|"·······A····"|"0"|"[TCP ACKed unseen segment] 443 → 40998 [ACK] Seq=1 Ack=2 Win=34133 Len=0 TSval=1775220934 TSecr=49239065 SLE=1 SRE=1"
"9"|"0.131732765"|"60"|"ARP"|"bc:24:11:65:8c:26"|"ff:ff:ff:ff:ff:ff"|"1"|"192.168.1.60"|"192.168.1.90"|||||||"Who has 192.168.1.90? Tell 192.168.1.60"
"10"|"0.150750772"|"66"|"TCP"|"64:6e:e0:ea:fa:83"|"e0:d3:62:6a:a2:5a"||||"192.168.1.15"|"34.149.66.165"|"48258"|"443"|"·······A····"|"0"|"48258 → 443 [ACK] Seq=1 Ack=1 Win=454 Len=0 TSval=3238213000 TSecr=1775220039"
"11"|"0.184602905"|"78"|"TCP"|"e0:d3:62:6a:a2:5a"|"64:6e:e0:ea:fa:83"||||"34.149.66.165"|"192.168.1.15"|"443"|"48258"|"·······A····"|"0"|"[TCP ACKed unseen segment] 443 → 48258 [ACK] Seq=1 Ack=2 Win=8067 Len=0 TSval=1775221074 TSecr=3238160142 SLE=1 SRE=1"
"12"|"0.234185992"|"60"|"ARP"|"bc:24:11:38:16:85"|"ff:ff:ff:ff:ff:ff"|"1"|"192.168.1.80"|"192.168.1.90"|||||||"Who has 192.168.1.90? Tell 192.168.1.80"
"13"|"0.437287691"|"60"|"ARP"|"bc:24:11:4b:19:47"|"ff:ff:ff:ff:ff:ff"|"1"|"192.168.1.152"|"192.168.1.90"|||||||"Who has 192.168.1.90? Tell 192.168.1.152"
"14"|"0.551751377"|"66"|"TCP"|"64:6e:e0:ea:fa:83"|"e0:d3:62:6a:a2:5a"||||"192.168.1.15"|"35.190.46.17"|"37142"|"443"|"·······A····"|"0"|"37142 → 443 [ACK] Seq=1 Ack=1 Win=10176 Len=0 TSval=1390389708 TSecr=1775220439"
"15"|"0.583263375"|"78"|"TCP"|"e0:d3:62:6a:a2:5a"|"64:6e:e0:ea:fa:83"||||"35.190.46.17"|"192.168.1.15"|"443"|"37142"|"·······A····"|"0"|"[TCP ACKed unseen segment] 443 → 37142 [ACK] Seq=1 Ack=2 Win=1914 Len=0 TSval=1775221464 TSecr=1390100721 SLE=1 SRE=1"
```

## Conversations

```text
================================================================================
TCP Conversations
Filter:<No Filter>
                                                           |       <-      | |       ->      | |     Total     |    Relative    |   Duration   |
                                                           | Frames  Bytes | | Frames  Bytes | | Frames  Bytes |      Start     |              |
192.168.1.15:41408         <-> 160.79.104.10:443                2 132 bytes       4 264 bytes       6 396 bytes     0.000000000         0.0309
192.168.1.15:40998         <-> 160.79.104.10:443                1 78 bytes        1 66 bytes        2 144 bytes     0.010758479         0.0438
192.168.1.15:48258         <-> 34.149.66.165:443                1 78 bytes        1 66 bytes        2 144 bytes     0.150750772         0.0339
192.168.1.15:37142         <-> 35.190.46.17:443                 1 78 bytes        1 66 bytes        2 144 bytes     0.551751377         0.0315
================================================================================
================================================================================
IPv4 Conversations
Filter:<No Filter>
                                               |       <-      | |       ->      | |     Total     |    Relative    |   Duration   |
                                               | Frames  Bytes | | Frames  Bytes | | Frames  Bytes |      Start     |              |
192.168.1.15         <-> 160.79.104.10              3 210 bytes       5 330 bytes       8 540 bytes     0.000000000         0.0546
192.168.1.15         <-> 34.149.66.165              1 78 bytes        1 66 bytes        2 144 bytes     0.150750772         0.0339
192.168.1.15         <-> 35.190.46.17               1 78 bytes        1 66 bytes        2 144 bytes     0.551751377         0.0315
================================================================================
================================================================================
Ethernet Conversations
Filter:<No Filter>
                                               |       <-      | |       ->      | |     Total     |    Relative    |   Duration   |
                                               | Frames  Bytes | | Frames  Bytes | | Frames  Bytes |      Start     |              |
64:6e:e0:ea:fa:83    <-> e0:d3:62:6a:a2:5a          5 366 bytes       7 462 bytes      12 828 bytes     0.000000000         0.5833
bc:24:11:65:8c:26    <-> ff:ff:ff:ff:ff:ff          0 0 bytes         1 60 bytes        1 60 bytes      0.131732765         0.0000
bc:24:11:38:16:85    <-> ff:ff:ff:ff:ff:ff          0 0 bytes         1 60 bytes        1 60 bytes      0.234185992         0.0000
bc:24:11:4b:19:47    <-> ff:ff:ff:ff:ff:ff          0 0 bytes         1 60 bytes        1 60 bytes      0.437287691         0.0000
================================================================================
```

## Endpoints

```text
================================================================================
TCP Endpoints
Filter:<No Filter>
                       |  Port  | | Packets | |  Bytes  | | Tx Packets | | Tx Bytes | | Rx Packets | | Rx Bytes |
160.79.104.10               443            8   540 bytes           3      210 bytes            5      330 bytes
192.168.1.15              41408            6   396 bytes           4      264 bytes            2      132 bytes
192.168.1.15              40998            2   144 bytes           1      66 bytes             1      78 bytes
192.168.1.15              48258            2   144 bytes           1      66 bytes             1      78 bytes
34.149.66.165               443            2   144 bytes           1      78 bytes             1      66 bytes
192.168.1.15              37142            2   144 bytes           1      66 bytes             1      78 bytes
35.190.46.17                443            2   144 bytes           1      78 bytes             1      66 bytes
================================================================================
================================================================================
IPv4 Endpoints
Filter:<No Filter>
                       | Packets | |  Bytes  | | Tx Packets | | Tx Bytes | | Rx Packets | | Rx Bytes |
192.168.1.15                   12   828 bytes           7      462 bytes            5      366 bytes
160.79.104.10                   8   540 bytes           3      210 bytes            5      330 bytes
34.149.66.165                   2   144 bytes           1      78 bytes             1      66 bytes
35.190.46.17                    2   144 bytes           1      78 bytes             1      66 bytes
================================================================================
================================================================================
Ethernet Endpoints
Filter:<No Filter>
                       | Packets | |  Bytes  | | Tx Packets | | Tx Bytes | | Rx Packets | | Rx Bytes |
64:6e:e0:ea:fa:83              12   828 bytes           7      462 bytes            5      366 bytes
e0:d3:62:6a:a2:5a              12   828 bytes           5      366 bytes            7      462 bytes
ff:ff:ff:ff:ff:ff               3   180 bytes           0      0 bytes              3      180 bytes
bc:24:11:65:8c:26               1   60 bytes            1      60 bytes             0      0 bytes
bc:24:11:38:16:85               1   60 bytes            1      60 bytes             0      0 bytes
bc:24:11:4b:19:47               1   60 bytes            1      60 bytes             0      0 bytes
================================================================================
```

## TLS records

```text
frame.number|ip.src|tcp.srcport|ip.dst|tcp.dstport|tls.record.content_type|tls.record.version|tls.record.length|_ws.col.info
"1"|"192.168.1.15"|"41408"|"160.79.104.10"|"443"|"23"|"0x0303"|"19"|"Application Data"
```

## Expert information

```text

Warns (9)
=============
   Frequency      Group           Protocol  Summary
           3   Sequence                TCP  Connection reset (RST)
           3   Sequence                TCP  D-SACK Sequence
           3   Sequence                TCP  ACKed segment that wasn't captured (common at capture start)

Notes (1)
=============
   Frequency      Group           Protocol  Summary
           1   Sequence                TCP  This frame initiates the connection closing

Chats (1)
=============
   Frequency      Group           Protocol  Summary
           1   Sequence                TCP  Connection finish (FIN)
```
