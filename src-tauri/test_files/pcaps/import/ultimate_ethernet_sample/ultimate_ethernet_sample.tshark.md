# TShark analysis: `ultimate_ethernet_sample.pcap`

- Generator: `script/pcap/analyze-tshark.sh`
- TShark: `TShark (Wireshark) 4.6.6.`
- Time zone: `UTC`
- SHA-256: `91252b10568ae9faed0c7262aa16c7c824643d478a392689345e1f63a43f0dde`

## Aggregate verification

| Source | Rows/frames | Packet count | Total bytes | Result |
|---|---:|---:|---:|---|
| PCAP | 328 | 328 | 96622 | - |
| `ultimate_ethernet_sample.csv` | 216 | 328 | 96622 | **PASS** |

## Capture metadata

```text
File name:           ultimate_ethernet_sample.pcap
File type:           Wireshark/tcpdump/... - pcap
File encapsulation:  Ethernet
File timestamp precision:  microseconds (6)
Packet size limit:   file hdr: 262144 bytes
Number of packets:   328
File size:           101 kB
Data size:           96 kB
Capture duration:    538124659.987110 seconds
Earliest packet time: 2009-05-04 07:34:16.122475
Latest packet time:   2026-05-23 14:38:36.109585
Data byte rate:      0 bytes/s
Data bit rate:       0 bits/s
Average packet size: 294.58 bytes
Average packet rate: 0 packets/s
SHA256:              91252b10568ae9faed0c7262aa16c7c824643d478a392689345e1f63a43f0dde
SHA1:                498d46d6cb17e250756063e53cac1930d594e131
Strict time order:   True
Number of interfaces in file: 1
Interface #0 info:
                     Encapsulation = Ethernet (1 - ether)
                     Capture length = 262144
                     Time precision = microseconds (6)
                     Time ticks per second = 1000000
                     Number of stat entries = 0
                     Number of packets = 328
```

## Protocol hierarchy

```text

===================================================================
Protocol Hierarchy Statistics
Filter:

frame                                    frames:328 bytes:96622
  eth                                    frames:328 bytes:96622
    arp                                  frames:6 bytes:360
    ip                                   frames:123 bytes:32516
      udp                                frames:59 bytes:8717
        ldp                              frames:3 bytes:228
        capwap                           frames:3 bytes:630
        capwap.data                      frames:6 bytes:477
          wlan                           frames:3 bytes:261
            _ws.malformed                frames:2 bytes:160
        dhcp                             frames:3 bytes:1274
        dns                              frames:3 bytes:238
        ssdp                             frames:3 bytes:507
        mdns                             frames:2 bytes:573
        bfd                              frames:3 bytes:210
        bfd_echo                         frames:3 bytes:192
        daytime                          frames:2 bytes:128
        time                             frames:2 bytes:106
        rmcp                             frames:1 bytes:65
          ipmi_session                   frames:1 bytes:65
            ipmb                         frames:1 bytes:65
              data                       frames:1 bytes:65
        wol                              frames:1 bytes:144
        llmnr                            frames:1 bytes:77
        nbns                             frames:3 bytes:342
        portcontrol                      frames:3 bytes:318
        xml                              frames:1 bytes:702
        udpencap                         frames:3 bytes:168
        stun                             frames:3 bytes:310
        dtls                             frames:4 bytes:1650
        srtcp                            frames:3 bytes:218
        mbudp                            frames:3 bytes:160
          modbus                         frames:3 bytes:160
      ospf                               frames:3 bytes:282
      icmp                               frames:3 bytes:342
      tcp                                frames:37 bytes:16992
        http                             frames:3 bytes:850
        tls                              frames:7 bytes:5970
          tls                            frames:1 bytes:752
        smtp                             frames:4 bytes:3411
        data                             frames:6 bytes:5021
        lpd                              frames:3 bytes:197
        ftp                              frames:3 bytes:281
        zabbix                           frames:3 bytes:629
        daytime                          frames:1 bytes:80
        time                             frames:1 bytes:58
        tacplus                          frames:3 bytes:263
        mbtcp                            frames:3 bytes:232
          modbus                         frames:3 bytes:232
      igmp                               frames:6 bytes:370
      eigrp                              frames:3 bytes:352
      ipv6                               frames:3 bytes:414
        icmpv6                           frames:3 bytes:414
          hipercontracer                 frames:3 bytes:414
      data                               frames:3 bytes:4542
      gre                                frames:6 bytes:505
        ip                               frames:2 bytes:124
          gre                            frames:2 bytes:124
        ipv6                             frames:3 bytes:321
          tcp                            frames:3 bytes:321
            telnet                       frames:3 bytes:321
    mpls                                 frames:9 bytes:835
      ip                                 frames:9 bytes:835
        l2tp                             frames:3 bytes:390
          data                           frames:3 bytes:390
        tcp                              frames:6 bytes:445
          bgp                            frames:3 bytes:231
    loop                                 frames:3 bytes:180
      data                               frames:3 bytes:180
    llc                                  frames:7 bytes:1419
      cdp                                frames:3 bytes:1047
      udld                               frames:2 bytes:250
      dtp                                frames:2 bytes:122
    ipv6                                 frames:55 bytes:27640
      icmpv6                             frames:3 bytes:258
      udp                                frames:29 bytes:16737
        dhcpv6                           frames:2 bytes:255
        isakmp                           frames:3 bytes:834
        mdns                             frames:1 bytes:261
        cflow                            frames:3 bytes:4301
        radius                           frames:3 bytes:405
        rmcp                             frames:5 bytes:515
          ipmi_session                   frames:5 bytes:515
            ipmb                         frames:2 bytes:177
              data                       frames:2 bytes:177
            data                         frames:3 bytes:338
        llmnr                            frames:2 bytes:194
        xml                              frames:2 bytes:1444
        rdpudp                           frames:5 bytes:5806
        quic                             frames:3 bytes:2722
          quic                           frames:2 bytes:2524
      esp                                frames:3 bytes:618
      data                               frames:3 bytes:4514
      tcp                                frames:17 bytes:5513
        whois                            frames:2 bytes:164
        http                             frames:3 bytes:1534
          ocsp                           frames:3 bytes:1534
        rsh                              frames:3 bytes:776
        pop                              frames:5 bytes:2026
          imf                            frames:2 bytes:1680
        tpkt                             frames:3 bytes:335
          cotp                           frames:3 bytes:335
            rdp                          frames:3 bytes:335
    vlan                                 frames:113 bytes:31679
      pppoes                             frames:20 bytes:7952
        ppp                              frames:20 bytes:7952
          ipv6                           frames:2 bytes:1681
            udp                          frames:1 bytes:163
              dhcpv6                     frames:1 bytes:163
            ip                           frames:1 bytes:1518
              tcp                        frames:1 bytes:1518
                tls                      frames:1 bytes:1518
          lcp                            frames:3 bytes:200
          ipcp                           frames:3 bytes:196
          ipv6cp                         frames:3 bytes:200
          ip                             frames:9 bytes:5675
            udp                          frames:8 bytes:5084
              sip                        frames:5 bytes:4406
              rtp                        frames:3 bytes:678
            data                         frames:1 bytes:591
      ipv6                               frames:52 bytes:18647
        udp                              frames:10 bytes:1814
          ripng                          frames:3 bytes:390
          hsrp                           frames:1 bytes:138
          snmp                           frames:3 bytes:552
          cldap                          frames:3 bytes:734
        tcp                              frames:42 bytes:16833
          ssh                            frames:5 bytes:1035
          tls                            frames:2 bytes:1050
          imap                           frames:3 bytes:374
          smtp                           frames:1 bytes:260
          nbss                           frames:9 bytes:2405
            smb                          frames:3 bytes:453
            smb2                         frames:6 bytes:1952
              dcerpc                     frames:3 bytes:858
          ldap                           frames:2 bytes:586
          dcerpc                         frames:14 bytes:3968
            epm                          frames:3 bytes:838
          data                           frames:1 bytes:720
      llc                                frames:8 bytes:1112
        stp                              frames:3 bytes:204
        udld                             frames:1 bytes:129
        dtp                              frames:1 bytes:65
        vtp                              frames:3 bytes:714
      ip                                 frames:23 bytes:2874
        udp                              frames:20 bytes:1975
          rip                            frames:3 bytes:310
          hsrp                           frames:5 bytes:434
          syslog                         frames:3 bytes:467
          ntp                            frames:3 bytes:282
          data                           frames:3 bytes:266
          tftp                           frames:3 bytes:216
        tcp                              frames:3 bytes:899
          kerberos                       frames:3 bytes:899
      slow                               frames:2 bytes:256
        lacp                             frames:2 bytes:256
      data                               frames:2 bytes:162
      pppoed                             frames:3 bytes:198
      eapol                              frames:3 bytes:478
        mka                              frames:3 bytes:478
    lldp                                 frames:3 bytes:1158
    slow                                 frames:1 bytes:124
      lacp                               frames:1 bytes:124
    data                                 frames:1 bytes:81
    homeplug-av                          frames:3 bytes:180
    wol                                  frames:1 bytes:116
    macsec                               frames:3 bytes:334
      data                               frames:3 bytes:334
===================================================================
```

## Packet inventory

```text
frame.number|frame.time_relative|frame.len|_ws.col.protocol|eth.src|eth.dst|arp.opcode|arp.src.proto_ipv4|arp.dst.proto_ipv4|ip.src|ip.dst|tcp.srcport|tcp.dstport|tcp.flags.str|tcp.len|_ws.col.info
"1"|"0.000000000"|"60"|"RARP"|"00:04:00:83:76:2c"|"ff:ff:ff:ff:ff:ff"|"3"|"0.0.0.0"|"0.0.0.0"|||||||"Who is 00:04:00:83:76:2c? Tell 00:04:00:83:76:2c"
"2"|"0.000000000"|"60"|"RARP"|"00:04:00:83:76:2c"|"ff:ff:ff:ff:ff:ff"|"3"|"0.0.0.0"|"0.0.0.0"|||||||"Who is 00:04:00:83:76:2c? Tell 00:04:00:83:76:2c"
"3"|"302.486575000"|"60"|"RARP"|"00:04:00:83:76:2c"|"ff:ff:ff:ff:ff:ff"|"3"|"0.0.0.0"|"0.0.0.0"|||||||"Who is 00:04:00:83:76:2c? Tell 00:04:00:83:76:2c"
"4"|"113831506.536076000"|"76"|"LDP"|"c2:3d:19:6c:00:01"|"01:00:5e:00:00:02"||||"10.0.0.1"|"224.0.0.2"|||||"Hello Message "
"5"|"113831506.536076000"|"76"|"LDP"|"c2:3d:19:6c:00:01"|"01:00:5e:00:00:02"||||"10.0.0.1"|"224.0.0.2"|||||"Hello Message "
"6"|"113831506.723076000"|"76"|"LDP"|"c2:3c:19:6c:00:01"|"01:00:5e:00:00:02"||||"10.0.0.2"|"224.0.0.2"|||||"Hello Message "
"7"|"113831508.720076000"|"136"|"L2TPv3"|"c2:3c:19:6c:00:01"|"c2:3d:19:6c:00:01"||||"10.200.200.202"|"10.200.200.201"|||||"D[S:0x000056D2]"
"8"|"113831508.720076000"|"136"|"L2TPv3"|"c2:3c:19:6c:00:01"|"c2:3d:19:6c:00:01"||||"10.200.200.202"|"10.200.200.201"|||||"D[S:0x000056D2]"
"9"|"113831508.735076000"|"60"|"LOOP"|"c2:3d:19:6c:00:01"|"c2:3d:19:6c:00:01"||||||||||"Reply"
"10"|"113831508.735076000"|"60"|"LOOP"|"c2:3d:19:6c:00:01"|"c2:3d:19:6c:00:01"||||||||||"Reply"
"11"|"113831509.141076000"|"60"|"ARP"|"c2:3d:19:6c:00:01"|"c2:3c:19:6c:00:01"|"1"|"10.0.0.1"|"10.0.0.2"|||||||"Who has 10.0.0.2? Tell 10.0.0.1"
"12"|"113831509.141076000"|"60"|"ARP"|"c2:3d:19:6c:00:01"|"c2:3c:19:6c:00:01"|"1"|"10.0.0.1"|"10.0.0.2"|||||||"Who has 10.0.0.2? Tell 10.0.0.1"
"13"|"113831509.157076000"|"60"|"ARP"|"c2:3c:19:6c:00:01"|"c2:3d:19:6c:00:01"|"2"|"10.0.0.2"|"10.0.0.1"|||||||"10.0.0.2 is at c2:3c:19:6c:00:01"
"14"|"113831509.406076000"|"118"|"L2TPv3"|"c2:3d:19:6c:00:01"|"c2:3c:19:6c:00:01"||||"10.200.200.201"|"10.200.200.202"|||||"D[S:0x00001138]"
"15"|"113831510.763076000"|"94"|"OSPF"|"c2:3c:19:6c:00:01"|"01:00:5e:00:00:05"||||"10.0.0.2"|"224.0.0.5"|||||"Hello Packet"
"16"|"113831510.763076000"|"94"|"OSPF"|"c2:3c:19:6c:00:01"|"01:00:5e:00:00:05"||||"10.0.0.2"|"224.0.0.5"|||||"Hello Packet"
"17"|"113831511.980076000"|"94"|"OSPF"|"c2:3d:19:6c:00:01"|"01:00:5e:00:00:05"||||"10.0.0.1"|"224.0.0.5"|||||"Hello Packet"
"18"|"113831512.682076000"|"349"|"CDP"|"c2:3c:19:6c:00:01"|"01:00:0c:cc:cc:cc"||||||||||"Device ID: P2  Port ID: FastEthernet0/1  "
"19"|"113831512.682076000"|"349"|"CDP"|"c2:3c:19:6c:00:01"|"01:00:0c:cc:cc:cc"||||||||||"Device ID: P2  Port ID: FastEthernet0/1  "
"20"|"113831514.008076000"|"114"|"ICMP"|"c2:3c:19:6c:00:01"|"c2:3d:19:6c:00:01"||||"10.0.0.2"|"10.0.0.1"|||||"Echo (ping) request  id=0x0001, seq=0/0, ttl=255"
"21"|"113831514.008076000"|"114"|"ICMP"|"c2:3c:19:6c:00:01"|"c2:3d:19:6c:00:01"||||"10.0.0.2"|"10.0.0.1"|||||"Echo (ping) request  id=0x0001, seq=0/0, ttl=255"
"22"|"113831514.024076000"|"114"|"ICMP"|"c2:3d:19:6c:00:01"|"c2:3c:19:6c:00:01"||||"10.0.0.1"|"10.0.0.2"|||||"Echo (ping) reply    id=0x0001, seq=0/0, ttl=255 (request in 21)"
"23"|"113831515.974076000"|"60"|"LOOP"|"c2:3c:19:6c:00:01"|"c2:3c:19:6c:00:01"||||||||||"Reply"
"24"|"113831518.641076000"|"77"|"BGP"|"c2:3c:19:6c:00:01"|"c2:3d:19:6c:00:01"||||"10.200.200.202"|"10.200.200.201"|"179"|"23975"|"·······AP···"|"19"|"KEEPALIVE Message"
"25"|"113831518.641076000"|"77"|"TCP"|"c2:3c:19:6c:00:01"|"c2:3d:19:6c:00:01"||||"10.200.200.202"|"10.200.200.201"|"179"|"23975"|"·······AP···"|"19"|"[TCP Retransmission] 179 → 23975 [PSH, ACK] Seq=1 Ack=1 Win=16073 Len=19"
"26"|"113831518.704076000"|"77"|"BGP"|"c2:3d:19:6c:00:01"|"c2:3c:19:6c:00:01"||||"10.200.200.201"|"10.200.200.202"|"23975"|"179"|"·······AP···"|"19"|"KEEPALIVE Message"
"27"|"113831518.704076000"|"77"|"TCP"|"c2:3d:19:6c:00:01"|"c2:3c:19:6c:00:01"||||"10.200.200.201"|"10.200.200.202"|"23975"|"179"|"·······AP···"|"19"|"[TCP Retransmission] 23975 → 179 [PSH, ACK] Seq=1 Ack=20 Win=16054 Len=19"
"28"|"113831518.938076000"|"60"|"TCP"|"c2:3c:19:6c:00:01"|"c2:3d:19:6c:00:01"||||"10.200.200.202"|"10.200.200.201"|"179"|"23975"|"·······A····"|"0"|"179 → 23975 [ACK] Seq=20 Ack=20 Win=16054 Len=0"
"29"|"113831563.913076000"|"349"|"CDP"|"c2:3d:19:6c:00:01"|"01:00:0c:cc:cc:cc"||||||||||"Device ID: P1  Port ID: FastEthernet0/1  "
"30"|"113831578.655076000"|"77"|"BGP"|"c2:3c:19:6c:00:01"|"c2:3d:19:6c:00:01"||||"10.200.200.202"|"10.200.200.201"|"179"|"23975"|"·······AP···"|"19"|"KEEPALIVE Message"
"31"|"148881240.483381000"|"365"|"HTTP"|"00:0c:29:9d:c9:d6"|"00:19:e2:a1:f9:86"||||"192.168.110.10"|"80.237.133.136"|"1152"|"80"|"·······AP···"|"311"|"GET / HTTP/1.1 "
"32"|"148881240.752926000"|"97"|"HTTP"|"00:19:e2:a1:f9:86"|"00:0c:29:9d:c9:d6"||||"80.237.133.136"|"192.168.110.10"|"80"|"1152"|"·······AP···"|"43"|"Continuation"
"33"|"148881257.793036000"|"388"|"HTTP"|"00:0c:29:9d:c9:d6"|"00:19:e2:a1:f9:86"||||"192.168.110.10"|"212.144.254.123"|"1154"|"3128"|"·······AP···"|"334"|"GET http://ip.webernetz.net/ HTTP/1.1 "
"34"|"150640362.682969000"|"262"|"CAPWAP-Control"|"00:00:00:00:00:00"|"00:00:00:00:00:00"||||"127.0.0.1"|"127.0.0.1"|||||"CAPWAP-Control - Join Request"
"35"|"150640362.835307000"|"167"|"CAPWAP-Control"|"00:00:00:00:00:00"|"00:00:00:00:00:00"||||"127.0.0.1"|"127.0.0.1"|||||"CAPWAP-Control - Join Response"
"36"|"150640362.835666000"|"201"|"CAPWAP-Control"|"00:00:00:00:00:00"|"00:00:00:00:00:00"||||"127.0.0.1"|"127.0.0.1"|||||"CAPWAP-Control - Configuration Status Request"
"37"|"150640362.943814000"|"72"|"CAPWAP-Data"|"00:00:00:00:00:00"|"00:00:00:00:00:00"||||"127.0.0.1"|"127.0.0.1"|||||"CAPWAP-Data Keep-Alive"
"38"|"150640362.944157000"|"72"|"CAPWAP-Data"|"00:00:00:00:00:00"|"00:00:00:00:00:00"||||"127.0.0.1"|"127.0.0.1"|||||"CAPWAP-Data Keep-Alive"
"39"|"150640372.613529000"|"80"|"802.11"|"00:00:00:00:00:00"|"00:00:00:00:00:00"||||"127.0.0.1"|"127.0.0.1"|||||"Association Request, SN=411, FN=0, Flags=o.mP....[Malformed Packet]"
"40"|"150640372.614506000"|"101"|"802.11"|"00:00:00:00:00:00"|"00:00:00:00:00:00"||||"127.0.0.1"|"127.0.0.1"|||||"Association Request, SN=412, FN=0, Flags=........, SSID=""Prova"""
"41"|"156079728.540459000"|"72"|"CAPWAP-Data"|"00:00:00:00:00:00"|"00:00:00:00:00:00"||||"127.0.0.1"|"127.0.0.1"|||||"CAPWAP-Data Keep-Alive"
"42"|"156079748.023625000"|"80"|"802.11"|"00:00:00:00:00:00"|"00:00:00:00:00:00"||||"127.0.0.1"|"127.0.0.1"|||||"Association Request, SN=470, FN=0, Flags=o.mP....[Malformed Packet]"
"43"|"188826404.676330000"|"90"|"ICMPv6"|"00:21:6a:2d:3b:8e"|"33:33:00:00:00:16"||||||||||"Multicast Listener Report Message v2"
"44"|"188826405.206323000"|"90"|"ICMPv6"|"00:21:6a:2d:3b:8e"|"33:33:00:00:00:16"||||||||||"Multicast Listener Report Message v2"
"45"|"188826405.249667000"|"78"|"ICMPv6"|"00:21:6a:2d:3b:8e"|"33:33:ff:2d:3b:8e"||||||||||"Neighbor Solicitation for fe80::221:6aff:fe2d:3b8e"
"46"|"188826405.316543000"|"342"|"DHCP"|"00:21:6a:2d:3b:8e"|"ff:ff:ff:ff:ff:ff"||||"0.0.0.0"|"255.255.255.255"|||||"DHCP Discover - Transaction ID 0xecd8ce24"
"47"|"188826407.076245000"|"129"|"DHCPv6"|"00:21:6a:2d:3b:8e"|"33:33:00:01:00:02"||||||||||"Information-request XID: 0x85aa23 CID: 0004ac266ef2fd37a985610c570592591a4c "
"48"|"188826407.116877000"|"126"|"DHCPv6"|"d4:21:22:76:5b:78"|"00:21:6a:2d:3b:8e"||||||||||"Reply XID: 0x85aa23 CID: 0004ac266ef2fd37a985610c570592591a4c "
"49"|"188826407.333049000"|"590"|"DHCP"|"d4:21:22:76:5b:78"|"00:21:6a:2d:3b:8e"||||"192.168.2.1"|"192.168.2.102"|||||"DHCP Offer    - Transaction ID 0xecd8ce24"
"50"|"188826407.333322000"|"342"|"DHCP"|"00:21:6a:2d:3b:8e"|"ff:ff:ff:ff:ff:ff"||||"0.0.0.0"|"255.255.255.255"|||||"DHCP Request  - Transaction ID 0xecd8ce24"
"51"|"188826443.750444000"|"68"|"DNS"|"00:21:6a:2d:3b:8e"|"d4:21:22:76:5b:78"||||"192.168.2.102"|"192.168.2.1"|||||"Standard query 0xb89f A heise.de"
"52"|"188826443.760800000"|"84"|"DNS"|"d4:21:22:76:5b:78"|"00:21:6a:2d:3b:8e"||||"192.168.2.1"|"192.168.2.102"|||||"Standard query response 0xb89f A heise.de A 193.99.144.80"
"53"|"188826443.768485000"|"86"|"DNS"|"00:21:6a:2d:3b:8e"|"d4:21:22:76:5b:78"||||"192.168.2.102"|"192.168.2.1"|||||"Standard query 0xd7fa PTR 80.144.99.193.in-addr.arpa"
"54"|"188826445.856070000"|"50"|"IGMPv3"|"d4:21:22:76:5b:78"|"01:00:5e:00:00:01"||||"192.168.2.1"|"224.0.0.1"|||||"Membership Query, general"
"55"|"189080689.225967000"|"163"|"DHCPv6"|"d4:21:22:76:5b:79"|"44:2b:03:19:03:44"||||||||||"Solicit XID: 0xc92c99 CID: 000100011cd0bf26d42122765b79 "
"56"|"218860580.149318000"|"361"|"TLSv1"|"00:12:3f:0a:8a:96"|"00:19:e2:a1:f9:89"||||"192.168.110.9"|"80.154.108.235"|"50477"|"443"|"·······AP···"|"295"|"Client Hello"
"57"|"218860580.154349000"|"2962"|"TLSv1.2"|"00:19:e2:a1:f9:89"|"00:12:3f:0a:8a:96"||||"80.154.108.235"|"192.168.110.9"|"443"|"50477"|"·······A····"|"2896"|"Server Hello"
"58"|"218860580.154390000"|"752"|"TLSv1.2"|"00:19:e2:a1:f9:89"|"00:12:3f:0a:8a:96"||||"80.154.108.235"|"192.168.110.9"|"443"|"50477"|"·······AP···"|"686"|"Certificate, Server Key Exchange, Server Hello Done"
"59"|"218860580.166526000"|"192"|"TLSv1.2"|"00:12:3f:0a:8a:96"|"00:19:e2:a1:f9:89"||||"192.168.110.9"|"80.154.108.235"|"50477"|"443"|"·······AP···"|"126"|"Client Key Exchange, Change Cipher Spec, Encrypted Handshake Message"
"60"|"218860681.967007000"|"96"|"SMTP"|"00:19:e2:a1:f9:89"|"00:12:3f:0a:8a:96"||||"80.154.108.237"|"192.168.110.9"|"25"|"45271"|"·······AP···"|"30"|"S: 220 mail.webertest.net ESMTP"
"61"|"218860681.967171000"|"94"|"SMTP"|"00:12:3f:0a:8a:96"|"00:19:e2:a1:f9:89"||||"192.168.110.9"|"80.154.108.237"|"45271"|"25"|"·······AP···"|"28"|"C: HELO localhost.localdomain"
"62"|"218860681.968183000"|"90"|"SMTP"|"00:19:e2:a1:f9:89"|"00:12:3f:0a:8a:96"||||"80.154.108.237"|"192.168.110.9"|"25"|"45271"|"·······AP···"|"24"|"S: 250 mail.webertest.net"
"63"|"218860681.972697000"|"3131"|"SMTP"|"00:12:3f:0a:8a:96"|"00:19:e2:a1:f9:89"||||"192.168.110.9"|"80.154.108.237"|"45271"|"25"|"·······AP···"|"3065"|"[TCP ACKed unseen segment] [TCP Previous segment not captured] C: DATA fragment, 3065 bytes"
"64"|"247148574.771516000"|"130"|"RIPng"|"00:1a:6c:a1:2b:99"|"33:33:00:00:00:09"||||||||||" Command Response, Version 1"
"65"|"247148574.816390000"|"68"|"STP"|"00:0a:8a:a1:5a:9a"|"01:00:0c:cc:cc:cd"||||||||||"RST. Root = 24576/121/00:0a:8a:a1:5a:80  Cost = 0  Port = 0x8042"
"66"|"247148574.892027000"|"386"|"LLDP"|"00:21:1b:ae:31:99"|"01:80:c2:00:00:0e"||||||||||"MA/00:21:1b:ae:31:80 IN/Gi0/1 120 SysN=CCNP-LAB-S1.webernetz.net SysD=Cisco IOS Software, C2960 Software (C2960-LANBASEK9-M), Version 15.0(2)SE9, RELEASE SOFTWARE (fc1)\\nTechnical Support: http://www.cisco.com/techsupport\\nCopyright (c) 1986-2015 by Cisco Systems, Inc.\\nCompiled Tue 01-Dec-15 07:07 by prod_rel_team "
"67"|"247148575.183580000"|"138"|"HSRPv2"|"00:1a:6c:a1:2b:99"|"33:33:00:00:00:66"||||||||||"Hello (state Standby)"
"68"|"247148575.328359000"|"68"|"STP"|"00:21:1b:ae:31:99"|"01:00:0c:cc:cc:cd"||||||||||"RST. Root = 24576/80/00:21:1b:ae:31:80  Cost = 0  Port = 0x8048"
"69"|"247148575.370477000"|"130"|"RIPv2"|"00:1e:7a:79:3f:11"|"01:00:5e:00:00:09"||||"192.168.10.1"|"224.0.0.9"|||||"Response"
"70"|"247148575.383351000"|"125"|"UDLD"|"00:0a:8a:a1:5a:9a"|"01:00:0c:cc:cc:cc"||||||||||"Device ID: FOC0630Z3KZ  Port ID: Gi0/2  "
"71"|"247148575.711027000"|"68"|"STP"|"00:21:1b:ae:31:99"|"01:00:0c:cc:cc:cd"||||||||||"RST. Root = 24576/10/00:21:1b:ae:31:80  Cost = 0  Port = 0x8048"
"72"|"247148576.008451000"|"118"|"HSRPv2"|"00:00:0c:9f:f0:79"|"01:00:5e:00:00:66"||||"192.168.121.254"|"224.0.0.102"|||||"Hello (state Active)"
"73"|"247148577.135631000"|"118"|"HSRPv2"|"00:1a:6c:a1:2b:99"|"01:00:5e:00:00:66"||||"192.168.121.253"|"224.0.0.102"|||||"Hello (state Standby)"
"74"|"247148578.213801000"|"129"|"UDLD"|"00:21:1b:ae:31:99"|"01:00:0c:cc:cc:cc"||||||||||"Device ID: FOC1213Z3S4  Port ID: Gi0/1  "
"75"|"247148578.864152000"|"70"|"RIPv2"|"00:1a:6c:a1:2b:99"|"01:00:5e:00:00:09"||||"192.168.121.253"|"224.0.0.9"|||||"Response"
"76"|"247148581.466053000"|"110"|"RIPng"|"00:1e:7a:79:3f:11"|"33:33:00:00:00:09"||||||||||" Command Response, Version 1"
"77"|"247148581.466422000"|"150"|"RIPng"|"00:1e:7a:79:3f:11"|"33:33:00:00:00:09"||||||||||" Command Response, Version 1"
"78"|"247148582.258426000"|"161"|"Syslog"|"00:21:1b:ae:31:c1"|"00:00:0c:9f:f0:79"||||"192.168.121.10"|"192.168.120.10"|||||"LOCAL7.NOTICE: 72: Mar  3 19:57:17.371: %LINK-5-CHANGED: Interface GigabitEthernet0/2, changed state to administratively down"
"79"|"247148582.258931000"|"165"|"Syslog"|"00:21:1b:ae:31:c1"|"00:00:0c:9f:f0:79"||||"192.168.121.10"|"192.168.120.10"|||||"LOCAL7.NOTICE: 73: Mar  3 19:57:18.377: %LINEPROTO-5-UPDOWN: Line protocol on Interface GigabitEthernet0/2, changed state to down"
"80"|"247148582.842512000"|"110"|"RIPv2"|"00:1e:7a:79:3f:11"|"01:00:5e:00:00:09"||||"192.168.121.2"|"224.0.0.9"|||||"Response"
"81"|"247148585.864731000"|"61"|"DTP"|"00:0a:8a:a1:5a:9a"|"01:00:0c:cc:cc:cc"||||||||||"Dynamic Trunk Protocol"
"82"|"247148586.149400000"|"65"|"DTP"|"00:21:1b:ae:31:99"|"01:00:0c:cc:cc:cc"||||||||||"Dynamic Trunk Protocol"
"83"|"247148588.577903000"|"94"|"NTP"|"00:16:47:df:e7:c1"|"00:00:0c:9f:f0:79"||||"192.168.121.40"|"212.224.120.164"|||||"NTP Version 3, client"
"84"|"247148588.579910000"|"94"|"NTP"|"00:14:69:9e:11:41"|"00:16:47:df:e7:c1"||||"212.224.120.164"|"192.168.121.40"|||||"NTP Version 3, server"
"85"|"247148589.431785000"|"128"|"LACP"|"00:0a:8a:a1:5a:9a"|"01:80:c2:00:00:02"||||||||||"v1 ACTOR 00:0a:8a:a1:5a:80 P: 26 K: 2 **DCSG*A PARTNER 00:21:1b:ae:31:80 P: 282 K: 2 **DCSG*A"
"86"|"247148590.383684000"|"125"|"UDLD"|"00:0a:8a:a1:5a:9a"|"01:00:0c:cc:cc:cc"||||||||||"Device ID: FOC0630Z3KZ  Port ID: Gi0/2  "
"87"|"247148590.389689000"|"106"|"VTP"|"00:21:1b:ae:31:99"|"01:00:0c:cc:cc:cc"||||||||||"Summary Advertisement, Revision: 13, Followers: 1"
"88"|"247148590.389932000"|"502"|"VTP"|"00:21:1b:ae:31:99"|"01:00:0c:cc:cc:cc"||||||||||"Subset Advertisement, Revision: 13, Seq: 1"
"89"|"247148590.458945000"|"106"|"VTP"|"00:0a:8a:a1:5a:9a"|"01:00:0c:cc:cc:cc"||||||||||"Summary Advertisement, Revision: 13, Followers: 1"
"90"|"247148590.574460000"|"94"|"NTP"|"00:16:47:df:e7:c1"|"00:00:0c:9f:f0:79"||||"192.168.121.40"|"78.46.107.140"|||||"NTP Version 3, client"
"91"|"247148595.823276000"|"98"|"UDP"|"00:1e:7a:79:3f:11"|"00:14:69:9e:11:41"||||"192.168.121.2"|"192.168.121.254"|||||"64199 → 1967 Len=52"
"92"|"247148595.824292000"|"98"|"UDP"|"00:1e:7a:79:3f:11"|"00:1a:6c:a1:2b:99"||||"192.168.121.2"|"192.168.121.253"|||||"64091 → 1967 Len=52"
"93"|"247148595.825779000"|"70"|"UDP"|"00:14:69:9e:11:41"|"00:1e:7a:79:3f:11"||||"192.168.121.254"|"192.168.121.2"|||||"1967 → 64199 Len=24"
"94"|"247148599.655371000"|"124"|"LACP"|"00:21:1b:ae:31:99"|"01:80:c2:00:00:02"||||||||||"v1 ACTOR 00:21:1b:ae:31:80 P: 282 K: 2 **DCSG*A PARTNER 00:0a:8a:a1:5a:80 P: 26 K: 2 **DCSG*A"
"95"|"247148604.798673000"|"386"|"LLDP"|"00:21:1b:ae:31:99"|"01:80:c2:00:00:0e"||||||||||"MA/00:21:1b:ae:31:80 IN/Gi0/1 120 SysN=CCNP-LAB-S1.webernetz.net SysD=Cisco IOS Software, C2960 Software (C2960-LANBASEK9-M), Version 15.0(2)SE9, RELEASE SOFTWARE (fc1)\\nTechnical Support: http://www.cisco.com/techsupport\\nCopyright (c) 1986-2015 by Cisco Systems, Inc.\\nCompiled Tue 01-Dec-15 07:07 by prod_rel_team "
"96"|"247148615.868142000"|"61"|"DTP"|"00:0a:8a:a1:5a:9a"|"01:00:0c:cc:cc:cc"||||||||||"Dynamic Trunk Protocol"
"97"|"247148616.716524000"|"128"|"LACP"|"00:0a:8a:a1:5a:9a"|"01:80:c2:00:00:02"||||||||||"v1 ACTOR 00:0a:8a:a1:5a:80 P: 26 K: 2 **DCSG*A PARTNER 00:21:1b:ae:31:80 P: 282 K: 2 **DCSG*A"
"98"|"247148634.778828000"|"386"|"LLDP"|"00:21:1b:ae:31:99"|"01:80:c2:00:00:0e"||||||||||"MA/00:21:1b:ae:31:80 IN/Gi0/1 120 SysN=CCNP-LAB-S1.webernetz.net SysD=Cisco IOS Software, C2960 Software (C2960-LANBASEK9-M), Version 15.0(2)SE9, RELEASE SOFTWARE (fc1)\\nTechnical Support: http://www.cisco.com/techsupport\\nCopyright (c) 1986-2015 by Cisco Systems, Inc.\\nCompiled Tue 01-Dec-15 07:07 by prod_rel_team "
"99"|"247148696.491161000"|"81"|"0x6002"|"00:14:69:9e:11:41"|"ab:00:00:02:00:00"||||||||||"DEC DNA Remote Console"
"100"|"247148731.039778000"|"141"|"Syslog"|"00:21:1b:ae:31:c1"|"00:00:0c:9f:f0:79"||||"192.168.121.10"|"192.168.120.10"|||||"LOCAL7.ERR: 74: Mar  3 19:59:46.152: %LINK-3-UPDOWN: Interface GigabitEthernet0/2, changed state to up"
"101"|"247148765.652153000"|"177"|"SNMP"|"00:14:69:9e:11:41"|"00:1e:7a:79:3f:11"||||||||||"get-request 1.3.6.1.2.1.31.1.1.1.1.2 1.3.6.1.2.1.31.1.1.1.6.2 1.3.6.1.2.1.31.1.1.1.1.2 1.3.6.1.2.1.31.1.1.1.10.2"
"102"|"247148765.655153000"|"198"|"SNMP"|"00:1e:7a:79:3f:11"|"00:14:69:9e:11:41"||||||||||"get-response 1.3.6.1.2.1.31.1.1.1.1.2 1.3.6.1.2.1.31.1.1.1.6.2 1.3.6.1.2.1.31.1.1.1.1.2 1.3.6.1.2.1.31.1.1.1.10.2"
"103"|"247148765.660032000"|"177"|"SNMP"|"00:14:69:9e:11:41"|"00:1e:7a:79:3f:11"||||||||||"get-request 1.3.6.1.2.1.31.1.1.1.1.9 1.3.6.1.2.1.31.1.1.1.6.9 1.3.6.1.2.1.31.1.1.1.1.9 1.3.6.1.2.1.31.1.1.1.10.9"
"104"|"247148829.642848000"|"81"|"0x6002"|"00:1a:6c:a1:2b:99"|"ab:00:00:02:00:00"||||||||||"DEC DNA Remote Console"
"105"|"247148873.345503000"|"119"|"SSH"|"00:14:69:9e:11:41"|"00:1e:7a:79:3f:11"||||||"60892"|"22"|"·······AP···"|"41"|"Client: Protocol (SSH-2.0-OpenSSH_7.2p2 Ubuntu-4ubuntu2.1)"
"106"|"247148873.350631000"|"97"|"SSHv2"|"00:1e:7a:79:3f:11"|"00:1a:6c:a1:2b:99"||||||"22"|"60892"|"·······AP···"|"19"|"Server: Protocol (SSH-2.0-Cisco-1.25)"
"107"|"247148873.352882000"|"594"|"SSHv2"|"00:14:69:9e:11:41"|"00:1e:7a:79:3f:11"||||||"60892"|"22"|"·······A····"|"516"|""
"108"|"247148873.353136000"|"382"|"TCP"|"00:14:69:9e:11:41"|"00:1e:7a:79:3f:11"||||||"60892"|"22"|"·······AP···"|"304"|"[TCP Previous segment not captured] 60892 → 22 [PSH, ACK] Seq=1074 Ack=20 Win=28800 Len=304"
"109"|"247148902.627427000"|"88"|"TFTP"|"00:1e:7a:79:3f:11"|"00:14:69:9e:11:41"||||"192.168.121.2"|"192.168.110.10"|||||"Write Request, File: CCNP-LAB-R2-Mar--3-20-02-38.701-7, Transfer type: octet"
"110"|"247148902.645557000"|"64"|"TFTP"|"00:14:69:9e:11:41"|"00:1e:7a:79:3f:11"||||"192.168.110.10"|"192.168.121.2"|||||"Acknowledgement, Block: 0"
"111"|"247148902.649431000"|"64"|"TFTP"|"00:14:69:9e:11:41"|"00:1e:7a:79:3f:11"||||"192.168.110.10"|"192.168.121.2"|||||"Acknowledgement, Block: 1"
"112"|"257139411.884149000"|"278"|"ISAKMP"|"b4:0c:25:05:8e:10"|"08:5b:0e:3c:11:5d"||||||||||"IKE_SA_INIT MID=00 Initiator Request"
"113"|"257139443.892982000"|"278"|"ISAKMP"|"08:5b:0e:3c:11:5d"|"b4:0c:25:05:8e:10"||||||||||"IKE_SA_INIT MID=00 Initiator Request"
"114"|"257139443.913741000"|"278"|"ISAKMP"|"b4:0c:25:05:8e:10"|"08:5b:0e:3c:11:5d"||||||||||"IKE_SA_INIT MID=00 Responder Response"
"115"|"257139444.434568000"|"206"|"ESP"|"b4:0c:25:05:8e:10"|"08:5b:0e:3c:11:5d"||||||||||"ESP (SPI=0x3d713155)"
"116"|"257139444.444319000"|"206"|"ESP"|"08:5b:0e:3c:11:5d"|"b4:0c:25:05:8e:10"||||||||||"ESP (SPI=0xf918698d)"
"117"|"257139445.436227000"|"206"|"ESP"|"b4:0c:25:05:8e:10"|"08:5b:0e:3c:11:5d"||||||||||"ESP (SPI=0x3d713155)"
"118"|"268487977.681648000"|"169"|"SSDP"|"c8:0e:14:7e:33:9f"|"01:00:5e:7f:ff:fa"||||"192.168.7.1"|"239.255.255.250"|||||"M-SEARCH * HTTP/1.1 "
"119"|"268487979.182284000"|"64"|"IGMPv3"|"c8:0e:14:7e:33:9f"|"01:00:5e:00:00:01"||||"192.168.7.1"|"224.0.0.1"|||||"Membership Query, general"
"120"|"268487981.881813000"|"64"|"IGMPv1"|"00:a0:de:de:54:13"|"01:00:5e:7f:ff:fa"||||"192.168.7.12"|"239.255.255.250"|||||"Membership Report"
"121"|"268487982.182281000"|"64"|"IGMPv1"|"00:a0:de:de:54:13"|"01:00:5e:00:00:fb"||||"192.168.7.12"|"224.0.0.251"|||||"Membership Report"
"122"|"268487982.682593000"|"169"|"SSDP"|"c8:0e:14:7e:33:9f"|"01:00:5e:7f:ff:fa"||||"192.168.7.1"|"239.255.255.250"|||||"M-SEARCH * HTTP/1.1 "
"123"|"268487987.681575000"|"169"|"SSDP"|"c8:0e:14:7e:33:9f"|"01:00:5e:7f:ff:fa"||||"192.168.7.1"|"239.255.255.250"|||||"M-SEARCH * HTTP/1.1 "
"124"|"268487988.981966000"|"64"|"IGMPv3"|"b8:27:eb:c9:16:37"|"01:00:5e:00:00:16"||||"192.168.7.5"|"224.0.0.22"|||||"Membership Report / Join group 224.0.0.251 for any sources"
"125"|"268488042.884328000"|"241"|"MDNS"|"74:81:14:81:c2:d4"|"01:00:5e:00:00:fb"||||"192.168.7.26"|"224.0.0.251"|||||"Standard query 0x0000 PTR _homekit._tcp.local, ""QU"" question PTR _raop._tcp.local, ""QU"" question PTR _airplay._tcp.local, ""QU"" question PTR _sleep-proxy._udp.local, ""QU"" question ANY iTunes_Ctrl_73F532408528F239._dacp._tcp.local, ""QU"" question SRV 0 0 57221 Johannes-ei-Patt.local OPT"
"126"|"268488042.884433000"|"332"|"MDNS"|"b8:27:eb:c9:16:37"|"01:00:5e:00:00:fb"||||"192.168.7.5"|"224.0.0.251"|||||"Standard query response 0x0000 PTR 9F1D6780E10D@Küche._raop._tcp.local TXT, cache flush SRV, cache flush 0 0 5000 jw-pi01.local AAAA, cache flush 2003:50:aa1c:c600:ba27:ebff:fec9:1637 A, cache flush 192.168.7.5"
"127"|"268488042.884474000"|"261"|"MDNS"|"74:81:14:81:c2:d4"|"33:33:00:00:00:fb"||||||||||"Standard query 0x0000 PTR _homekit._tcp.local, ""QU"" question PTR _raop._tcp.local, ""QU"" question PTR _airplay._tcp.local, ""QU"" question PTR _sleep-proxy._udp.local, ""QU"" question ANY iTunes_Ctrl_73F532408528F239._dacp._tcp.local, ""QU"" question SRV 0 0 57221 Johannes-ei-Patt.local OPT"
"128"|"268488077.412209000"|"561"|"TCP"|"00:a0:de:de:54:13"|"c8:0e:14:7e:33:9f"||||"192.168.7.12"|"192.168.7.1"|"1226"|"51108"|"·······AP···"|"503"|"1226 → 51108 [PSH, ACK] Seq=1 Ack=1 Win=65535 Len=503"
"129"|"268488077.490175000"|"108"|"TCP"|"00:a0:de:de:54:13"|"c8:0e:14:7e:33:9f"||||"192.168.7.12"|"192.168.7.1"|"1227"|"51108"|"·······AP···"|"50"|"1227 → 51108 [PSH, ACK] Seq=1 Ack=1 Win=65535 Len=50"
"130"|"268488077.490244000"|"778"|"TCP"|"00:a0:de:de:54:13"|"c8:0e:14:7e:33:9f"||||"192.168.7.12"|"192.168.7.1"|"1228"|"51108"|"·······AP···"|"720"|"1228 → 51108 [PSH, ACK] Seq=1 Ack=1 Win=65535 Len=720"
"131"|"268488116.812265000"|"64"|"IGMPv1"|"00:a0:de:de:54:13"|"01:00:5e:7f:ff:fa"||||"192.168.7.12"|"239.255.255.250"|||||"Membership Report"
"132"|"272033355.067547000"|"66"|"PPPoED"|"bc:05:43:cc:c2:a9"|"ff:ff:ff:ff:ff:ff"||||||||||"Active Discovery Initiation (PADI)"
"133"|"272033355.570565000"|"66"|"PPPoED"|"bc:05:43:cc:c2:a9"|"ff:ff:ff:ff:ff:ff"||||||||||"Active Discovery Initiation (PADI)"
"134"|"272033356.558612000"|"66"|"PPPoED"|"bc:05:43:cc:c2:a9"|"ff:ff:ff:ff:ff:ff"||||||||||"Active Discovery Initiation (PADI)"
"135"|"272033366.971790000"|"68"|"PPP LCP"|"44:2b:03:19:03:44"|"bc:05:43:cc:c2:a9"||||||||||"Echo Request"
"136"|"272033368.570135000"|"64"|"PPP LCP"|"bc:05:43:cc:c2:a9"|"44:2b:03:19:03:44"||||||||||"Configuration Request"
"137"|"272033368.570136000"|"68"|"PPP LCP"|"44:2b:03:19:03:44"|"bc:05:43:cc:c2:a9"||||||||||"Configuration Request"
"138"|"272033368.663844000"|"68"|"PPP IPCP"|"44:2b:03:19:03:44"|"bc:05:43:cc:c2:a9"||||||||||"Configuration Request"
"139"|"272033368.663846000"|"64"|"PPP IPCP"|"bc:05:43:cc:c2:a9"|"44:2b:03:19:03:44"||||||||||"Configuration Request"
"140"|"272033368.663849000"|"64"|"PPP IPV6CP"|"bc:05:43:cc:c2:a9"|"44:2b:03:19:03:44"||||||||||"Configuration Request"
"141"|"272033368.663851000"|"64"|"PPP IPCP"|"bc:05:43:cc:c2:a9"|"44:2b:03:19:03:44"||||||||||"Configuration Ack"
"142"|"272033368.663855000"|"68"|"PPP IPV6CP"|"44:2b:03:19:03:44"|"bc:05:43:cc:c2:a9"||||||||||"Configuration Request"
"143"|"272033368.663857000"|"68"|"PPP IPV6CP"|"44:2b:03:19:03:44"|"bc:05:43:cc:c2:a9"||||||||||"Configuration Ack"
"144"|"277179164.641979000"|"118"|"EIGRP"|"00:1a:6c:a1:2b:99"|"01:00:5e:00:00:0a"||||"192.168.127.1"|"224.0.0.10"|||||"Hello"
"145"|"277179164.642009000"|"136"|"EIGRP"|"00:14:69:9e:11:40"|"01:00:5e:00:00:0a"||||"192.168.127.2"|"224.0.0.10"|||||"Hello"
"146"|"277179164.642012000"|"98"|"EIGRP"|"00:14:69:9e:11:40"|"00:1a:6c:a1:2b:99"||||"192.168.127.2"|"192.168.127.1"|||||"Update"
"147"|"283324490.367661000"|"271"|"TLSv1"|"00:0c:29:c1:34:dc"|"b4:0c:25:05:8e:13"||||||"7549"|"443"|"·······AP···"|"193"|"Client Hello (SNI=ip.webernetz.net)"
"148"|"283324490.373414000"|"779"|"SSL"|"b4:0c:25:05:8e:13"|"00:0c:29:c1:34:dc"||||||"443"|"7549"|"·······AP···"|"701"|"Continuation Data"
"149"|"283324496.489880000"|"192"|"IMAP"|"b4:0c:25:05:8e:13"|"00:0c:29:c1:34:dc"||||||"143"|"7552"|"·······AP···"|"114"|"Response: * OK [CAPABILITY IMAP4rev1 LITERAL+ SASL-IR LOGIN-REFERRALS ID ENABLE IDLE AUTH=PLAIN AUTH=LOGIN] Dovecot ready."
"150"|"283324497.849090000"|"100"|"IMAP"|"00:0c:29:c1:34:dc"|"b4:0c:25:05:8e:13"||||||"7552"|"143"|"·······AP···"|"22"|"Request: 1 authenticate PLAIN"
"151"|"283324497.850088000"|"82"|"IMAP"|"b4:0c:25:05:8e:13"|"00:0c:29:c1:34:dc"||||||"143"|"7552"|"·······AP···"|"4"|"Response: + "
"152"|"283324512.276106000"|"260"|"SMTP"|"00:0c:29:c1:34:dc"|"b4:0c:25:05:8e:13"||||||"7562"|"25"|"·······AP···"|"182"|"C: DATA fragment, 182 bytes"
"153"|"283324516.585284000"|"119"|"SSH"|"b4:0c:25:05:8e:13"|"00:0c:29:c1:34:dc"||||||"22"|"7563"|"·······AP···"|"41"|"Server: Protocol (SSH-2.0-OpenSSH_7.2p2 Ubuntu-4ubuntu2.4)"
"154"|"283324516.591037000"|"106"|"SSH"|"00:0c:29:c1:34:dc"|"b4:0c:25:05:8e:13"||||||"7563"|"22"|"·······AP···"|"28"|"Client: Protocol (SSH-2.0-PuTTY_Release_0.63)"
"155"|"315983834.940447000"|"138"|"ICMPv6, HiPerConTracer"|"00:10:db:ff:10:00"|"00:14:69:9e:11:40"||||"216.66.80.30"|"193.24.227.12"|||||"Echo (ping) request id=0x073d, seq=1, hop limit=62 (SendTTL=20, Round=21)"
"156"|"315983834.941473000"|"138"|"ICMPv6, HiPerConTracer"|"00:14:69:9e:11:40"|"00:10:db:ff:10:00"||||"193.24.227.12"|"216.66.80.30"|||||"Echo (ping) reply id=0x073d, seq=1, hop limit=62 (request in 155) (SendTTL=20, Round=21)"
"157"|"315983835.940660000"|"138"|"ICMPv6, HiPerConTracer"|"00:10:db:ff:10:00"|"00:14:69:9e:11:40"||||"216.66.80.30"|"193.24.227.12"|||||"Echo (ping) request id=0x073d, seq=2, hop limit=62 (SendTTL=20, Round=21)"
"158"|"317545551.899237000"|"1514"|"IPv4"|"00:0c:29:8a:5d:d7"|"00:86:9c:e7:55:14"||||"193.24.227.238"|"172.217.40.76"|||||"Fragmented IP protocol (proto=UDP 17, off=0, ID=d0fe)"
"159"|"317545554.111970000"|"1510"|"IPv6"|"00:0c:29:8a:5d:d7"|"00:86:9c:e7:55:14"||||||||||"IPv6 fragment (off=0 more=y ident=0x28403c0b nxt=17)"
"160"|"317545561.952703000"|"1510"|"IPv6"|"00:0c:29:8a:5d:d7"|"00:86:9c:e7:55:14"||||||||||"IPv6 fragment (off=0 more=y ident=0x247f0cb3 nxt=17)"
"161"|"317545562.947240000"|"1514"|"IPv4"|"00:0c:29:8a:5d:d7"|"00:86:9c:e7:55:14"||||"193.24.227.238"|"173.194.169.104"|||||"Fragmented IP protocol (proto=UDP 17, off=0, ID=e211)"
"162"|"317545564.904537000"|"1514"|"IPv4"|"00:0c:29:8a:5d:d7"|"00:86:9c:e7:55:14"||||"193.24.227.238"|"74.125.47.136"|||||"Fragmented IP protocol (proto=UDP 17, off=0, ID=893c)"
"163"|"319447449.110509000"|"1494"|"IPv6"|"08:5b:0e:a1:83:5e"|"00:0c:29:7c:a4:cb"||||||||||"IPv6 fragment (off=0 more=y ident=0x0000069a nxt=17)"
"164"|"332407522.889363000"|"1445"|"SIP/SDP"|"3c:61:04:50:d2:1a"|"c8:0e:14:7e:33:a0"||||"217.0.21.65"|"84.146.135.221"|||||"Request: INVITE sip:+4960339285361@84.146.135.221;user=phone;uniq=E04784589605A88765A939C2CA2A7 | "
"165"|"332407522.914943000"|"481"|"SIP"|"c8:0e:14:7e:33:a0"|"3c:61:04:50:d2:1a"||||"84.146.135.221"|"217.0.21.65"|||||"Status: 100 Trying | "
"166"|"332407522.930823000"|"645"|"SIP"|"c8:0e:14:7e:33:a0"|"3c:61:04:50:d2:1a"||||"84.146.135.221"|"217.0.21.65"|||||"Status: 180 Ringing | "
"167"|"332407553.250163000"|"1164"|"SIP/SDP"|"c8:0e:14:7e:33:a0"|"3c:61:04:50:d2:1a"||||"84.146.135.221"|"217.0.21.65"|||||"Status: 200 OK (INVITE) | "
"168"|"332407553.264834000"|"226"|"RTP"|"c8:0e:14:7e:33:a0"|"3c:61:04:50:d2:1a"||||"84.146.135.221"|"217.0.5.215"|||||"PT=ITU-T G.711 PCMA, SSRC=0x6CCC5511, Seq=1, Time=160, Mark"
"169"|"332407553.284830000"|"226"|"RTP"|"c8:0e:14:7e:33:a0"|"3c:61:04:50:d2:1a"||||"84.146.135.221"|"217.0.5.215"|||||"PT=ITU-T G.711 PCMA, SSRC=0x6CCC5511, Seq=2, Time=320"
"170"|"332407553.304903000"|"226"|"RTP"|"c8:0e:14:7e:33:a0"|"3c:61:04:50:d2:1a"||||"84.146.135.221"|"217.0.5.215"|||||"PT=ITU-T G.711 PCMA, SSRC=0x6CCC5511, Seq=3, Time=480"
"171"|"332407553.580008000"|"671"|"SIP"|"3c:61:04:50:d2:1a"|"c8:0e:14:7e:33:a0"||||"217.0.21.65"|"84.146.135.221"|||||"Request: ACK sip:+4960339285361@84.146.135.221;user=phone;uniq=E04784589605A88765A939C2CA2A7 | "
"172"|"332670826.330073000"|"591"|"IPv4"|"3c:61:04:50:d2:1a"|"c8:0e:14:7e:33:a0"||||"217.0.21.65"|"84.146.135.221"|||||"Fragmented IP protocol (proto=UDP 17, off=1472, ID=38fc)"
"173"|"332801714.808215000"|"981"|"TLSv1.3"|"b8:27:eb:ab:ae:c7"|"00:00:0c:9f:f1:c2"||||"141.41.241.70"|"141.41.39.187"|"443"|"40976"|"·······AP···"|"915"|"Server Hello, Change Cipher Spec, Application Data, Application Data, Application Data, Application Data"
"174"|"332801714.824316000"|"146"|"TLSv1.3"|"d8:67:d9:07:8e:c1"|"b8:27:eb:ab:ae:c7"||||"141.41.39.187"|"141.41.241.70"|"40976"|"443"|"·······AP···"|"80"|"Change Cipher Spec, Application Data"
"175"|"332801714.825842000"|"576"|"TLSv1.3"|"b8:27:eb:ab:ae:c7"|"00:00:0c:9f:f1:c2"||||"141.41.241.70"|"141.41.39.187"|"443"|"40976"|"·······AP···"|"510"|"Application Data, Application Data"
"176"|"335418362.555327000"|"90"|"WHOIS"|"d4:be:d9:4c:11:9e"|"00:86:9c:e7:55:14"||||||"52222"|"43"|"·······AP···"|"16"|"Query: 193.24.227.225"
"177"|"335418362.661317000"|"678"|"WHOIS"|"00:86:9c:e7:55:14"|"d4:be:d9:4c:11:9e"||||||"43"|"52222"|"·······AP···"|"604"|"Answer: 193.24.227.225"
"178"|"335418362.661358000"|"74"|"WHOIS"|"00:86:9c:e7:55:14"|"d4:be:d9:4c:11:9e"||||||"43"|"52222"|"·······A···F"|"0"|"Answer: 193.24.227.225"
"179"|"338266527.971690000"|"1430"|"CFLOW"|"00:86:9c:e7:55:14"|"d4:be:d9:4c:11:9e"||||||||||"total: 20 (v9) records Obs-Domain-ID=    1 [Data:256] [Data:256] [Data:256] [Data:256] [Data:256] [Data:258] [Data:258] [Data:258] [Data:258] [Data:256] [Data:256] [Data:256] [Data:256] [Data:256] [Data:256] [Data:258] [Data:258] [Data:258] [Data:256] [Data:256]"
"180"|"338266528.186991000"|"1417"|"CFLOW"|"00:86:9c:e7:55:14"|"d4:be:d9:4c:11:9e"||||||||||"total: 21 (v9) records Obs-Domain-ID=    1 [Data:256] [Data:256] [Data:258] [Data:256] [Data:256] [Data:256] [Data:256] [Data:256] [Data:256] [Data:258] [Data:256] [Data:258] [Data:256] [Data:256] [Data:256] [Data:256] [Data:256] [Data:256] [Data:256] [Data:256] [Data:258]"
"181"|"338266528.458396000"|"1454"|"CFLOW"|"00:86:9c:e7:55:14"|"d4:be:d9:4c:11:9e"||||||||||"total: 20 (v9) records Obs-Domain-ID=    1 [Data:256] [Data:258] [Data:258] [Data:258] [Data:258] [Data:256] [Data:258] [Data:258] [Data:256] [Data:256] [Data:256] [Data:256] [Data:256] [Data:258] [Data:256] [Data:256] [Data:256] [Data:258] [Data:256] [Data:256]"
"182"|"338277664.949614000"|"62"|"GRE"|"00:1e:7a:79:3f:10"|"00:25:45:60:17:c1"||||"172.16.23.2,192.168.47.1"|"192.168.47.1,172.16.23.2"|||||"Encapsulated Possible GRE keepalive packet"
"183"|"338277664.949615000"|"60"|"GRE"|"00:25:45:60:17:c1"|"00:1e:7a:79:3f:10"||||"192.168.47.1"|"172.16.23.2"|||||"Encapsulated Possible GRE keepalive packet"
"184"|"338277671.170512000"|"62"|"GRE"|"00:25:45:60:17:c1"|"00:1e:7a:79:3f:10"||||"192.168.47.1,172.16.23.2"|"172.16.23.2,192.168.47.1"|||||"Encapsulated Possible GRE keepalive packet"
"185"|"338279056.995257000"|"110"|"TELNET"|"00:1e:7a:79:3f:10"|"00:25:45:60:17:c1"||||"172.16.23.2"|"192.168.47.1"|"18716"|"23"|"·······AP···"|"12"|"Do Suppress Go Ahead, Will Terminal Speed, Will Negotiate About Window Size, Will Remote Flow Control"
"186"|"338279056.998106000"|"110"|"TELNET"|"00:25:45:60:17:c1"|"00:1e:7a:79:3f:10"||||"192.168.47.1"|"172.16.23.2"|"23"|"18716"|"·······AP···"|"12"|"Will Echo, Will Suppress Go Ahead, Do Terminal Type, Do Negotiate About Window Size"
"187"|"338279056.999975000"|"101"|"TELNET"|"00:1e:7a:79:3f:10"|"00:25:45:60:17:c1"||||"172.16.23.2"|"192.168.47.1"|"18716"|"23"|"·······AP···"|"3"|"Do Echo"
"188"|"343727186.438644000"|"276"|"OCSP"|"b8:27:eb:03:a0:ac"|"00:86:9c:e7:55:14"||||||"55996"|"80"|"·······AP···"|"202"|"Request"
"189"|"343727186.602255000"|"982"|"OCSP"|"00:86:9c:e7:55:14"|"b8:27:eb:03:a0:ac"||||||"80"|"55996"|"·······AP···"|"908"|"Response"
"190"|"343732136.336299000"|"276"|"OCSP"|"b8:27:eb:03:a0:ac"|"00:86:9c:e7:55:14"||||||"35638"|"80"|"·······AP···"|"202"|"Request"
"191"|"356411927.059378000"|"58"|"LPD"|"54:ee:75:ec:9a:f4"|"a8:d0:e5:d4:fe:cb"||||"10.82.185.11"|"192.168.11.10"|"57895"|"515"|"·······AP···"|"4"|"LPR: transfer a printer job / jobcmd: receive control file"
"192"|"356411927.060414000"|"60"|"LPD"|"a8:d0:e5:d4:fe:cb"|"54:ee:75:ec:9a:f4"||||"192.168.11.10"|"10.82.185.11"|"515"|"57895"|"·······AP···"|"1"|"LPD response"
"193"|"356411927.061267000"|"79"|"LPD"|"54:ee:75:ec:9a:f4"|"a8:d0:e5:d4:fe:cb"||||"10.82.185.11"|"192.168.11.10"|"57895"|"515"|"·······AP···"|"25"|"LPR: transfer a printer job / jobcmd: receive control file"
"194"|"357620708.871252000"|"1518"|"SSL"|"90:03:25:74:4e:06"|"cc:ce:1e:5b:c4:93"||||"52.109.32.27"|"192.168.7.35"|"443"|"63594"|"·······A····"|"1412"|""
"195"|"361182722.921897000"|"96"|"FTP"|"a8:d0:e5:d4:fe:cb"|"54:ee:75:ec:9a:f4"||||"5.35.226.136"|"10.82.185.11"|"21"|"51072"|"·······AP···"|"42"|"Response: 220 ::ffff:5.35.226.136 FTP server ready"
"196"|"361182724.284316000"|"82"|"FTP"|"54:ee:75:ec:9a:f4"|"a8:d0:e5:d4:fe:cb"||||"10.82.185.11"|"5.35.226.136"|"51072"|"21"|"·······AP···"|"28"|"Request: USER ftp1119456-nureintest"
"197"|"361182724.299889000"|"103"|"FTP"|"a8:d0:e5:d4:fe:cb"|"54:ee:75:ec:9a:f4"||||"5.35.226.136"|"10.82.185.11"|"21"|"51072"|"·······AP···"|"49"|"Response: 331 Password required for ftp1119456-nureintest"
"198"|"361182725.431779000"|"1506"|"TCP"|"a8:d0:e5:d4:fe:cb"|"54:ee:75:ec:9a:f4"||||"5.35.226.136"|"10.82.185.11"|"51652"|"51075"|"·······A····"|"1452"|"51652 → 51075 [ACK] Seq=1 Ack=1 Win=913 Len=1452"
"199"|"361182725.431779000"|"562"|"TCP"|"a8:d0:e5:d4:fe:cb"|"54:ee:75:ec:9a:f4"||||"5.35.226.136"|"10.82.185.11"|"51652"|"51075"|"·······AP··F"|"508"|"51652 → 51075 [FIN, PSH, ACK] Seq=1453 Ack=1 Win=913 Len=508"
"200"|"361182727.627065000"|"1506"|"TCP"|"a8:d0:e5:d4:fe:cb"|"54:ee:75:ec:9a:f4"||||"5.35.226.136"|"10.82.185.11"|"52833"|"51076"|"·······A····"|"1452"|"52833 → 51076 [ACK] Seq=1 Ack=1 Win=913 Len=1452"
"201"|"361949113.835989000"|"81"|"0x6002"|"00:1e:7a:79:3f:10"|"ab:00:00:02:00:00"||||||||||"DEC DNA Remote Console"
"202"|"361949125.378430000"|"70"|"BFD Control"|"00:1e:7a:79:3f:10"|"00:15:62:6a:fe:f0"||||"193.24.225.54"|"193.24.225.56"|||||"Diag: No Diagnostic, State: Down, Flags: 0x00"
"203"|"361949126.135798000"|"70"|"BFD Control"|"00:1e:7a:79:3f:10"|"00:15:62:6a:fe:f0"||||"193.24.225.54"|"193.24.225.56"|||||"Diag: No Diagnostic, State: Down, Flags: 0x00"
"204"|"361949126.135950000"|"70"|"BFD Control"|"00:15:62:6a:fe:f0"|"00:1e:7a:79:3f:10"||||"193.24.225.56"|"193.24.225.54"|||||"Diag: No Diagnostic, State: Init, Flags: 0x00"
"205"|"361949126.371756000"|"64"|"BFD Echo"|"00:1e:7a:79:3f:10"|"00:15:62:6a:fe:f0"||||"193.24.225.54"|"193.24.225.54"|||||"Originator specific content"
"206"|"361949126.371856000"|"64"|"BFD Echo"|"00:15:62:6a:fe:f0"|"00:1e:7a:79:3f:10"||||"193.24.225.54"|"193.24.225.54"|||||"Originator specific content"
"207"|"361949126.379015000"|"64"|"BFD Echo"|"00:15:62:6a:fe:f0"|"00:1e:7a:79:3f:10"||||"193.24.225.56"|"193.24.225.56"|||||"Originator specific content"
"208"|"375864018.449222000"|"60"|"HomePlug AV"|"c8:0e:14:7e:33:9f"|"00:b0:52:00:00:01"||||||||||"Qualcomm Atheros, OP_ATTR.REQ (Get Device Attributes Request)"
"209"|"375864018.449321000"|"60"|"HomePlug AV"|"c8:0e:14:7e:33:9f"|"ff:ff:ff:ff:ff:ff"||||||||||"Qualcomm Atheros, GET_SW.REQ (Get Device/SW Version Request)"
"210"|"375864020.450498000"|"60"|"HomePlug AV"|"c8:0e:14:7e:33:9f"|"00:b0:52:00:00:01"||||||||||"Qualcomm Atheros, OP_ATTR.REQ (Get Device Attributes Request)"
"211"|"380094905.578687000"|"152"|"RSH"|"b8:27:eb:03:a0:ac"|"00:21:70:b2:0e:6c"||||||"55600"|"514"|"·······AP···"|"66"|"Client -> Server data"
"212"|"380094905.579271000"|"185"|"RSH"|"b8:27:eb:03:a0:ac"|"00:21:70:b2:0e:6c"||||||"55600"|"514"|"·······AP···"|"99"|"Client -> Server data"
"213"|"380094905.586183000"|"439"|"RSH"|"b8:27:eb:03:a0:ac"|"00:21:70:b2:0e:6c"||||||"55600"|"514"|"·······AP···"|"353"|"Client -> Server data"
"214"|"411633454.736446000"|"66"|"HSRP"|"00:00:0c:07:ac:14"|"01:00:5e:00:00:02"||||"192.168.20.2"|"224.0.0.2"|||||"Hello (state Active)"
"215"|"411633455.572470000"|"66"|"HSRP"|"00:00:0c:07:ac:14"|"01:00:5e:00:00:02"||||"192.168.20.2"|"224.0.0.2"|||||"Hello (state Active)"
"216"|"411633457.763198000"|"66"|"HSRP"|"00:00:0c:07:ac:14"|"01:00:5e:00:00:02"||||"192.168.20.2"|"224.0.0.2"|||||"Hello (state Active)"
"217"|"416449302.153569000"|"299"|"Zabbix"|"00:0c:29:d5:b8:68"|"00:0c:29:af:1c:ec"||||"192.168.7.16"|"192.168.7.17"|"49404"|"10051"|"·······AP···"|"233"|"Zabbix Agent data from ""Zabbix6.2-client"", Len=220 (49404 → 10051)"
"218"|"416449302.153795000"|"169"|"Zabbix"|"00:0c:29:af:1c:ec"|"00:0c:29:d5:b8:68"||||"192.168.7.17"|"192.168.7.16"|"10051"|"49404"|"·······AP···"|"103"|"Zabbix Server/proxy response for agent data for ""Zabbix6.2-client"" (success), Len=90 (10051 → 49404)"
"219"|"416449332.158543000"|"161"|"Zabbix"|"00:0c:29:d5:b8:68"|"00:0c:29:af:1c:ec"||||"192.168.7.16"|"192.168.7.17"|"49406"|"10051"|"·······AP···"|"95"|"Zabbix Agent heartbeat from ""Zabbix6.2-client"", Len=82 (49406 → 10051)"
"220"|"427350545.701791000"|"80"|"DAYTIME"|"00:13:95:24:34:04"|"70:4c:a5:99:4a:b3"||||"194.247.5.12"|"85.215.94.29"|"13"|"43624"|"·······AP···"|"26"|"DAYTIME Response"
"221"|"427350559.837987000"|"60"|"DAYTIME"|"70:4c:a5:99:4a:b3"|"00:13:95:24:34:04"||||"85.215.94.29"|"194.247.5.12"|||||"DAYTIME Request"
"222"|"427350559.838344000"|"68"|"DAYTIME"|"00:13:95:24:34:04"|"b8:27:eb:03:a0:ac"||||"194.247.5.12"|"85.215.94.29"|||||"DAYTIME Response"
"223"|"427350600.190400000"|"58"|"TIME"|"00:13:95:24:34:04"|"70:4c:a5:99:4a:b3"||||"194.247.5.12"|"85.215.94.29"|"37"|"49510"|"·······AP···"|"4"|"TIME Response"
"224"|"427350611.674560000"|"60"|"TIME"|"70:4c:a5:99:4a:b3"|"00:13:95:24:34:04"||||"85.215.94.29"|"194.247.5.12"|||||"TIME Request"
"225"|"427350611.676192000"|"46"|"TIME"|"00:13:95:24:34:04"|"b8:27:eb:03:a0:ac"||||"194.247.5.12"|"85.215.94.29"|||||"TIME Response"
"226"|"429500811.305504000"|"106"|"TACACS+"|"3c:13:cc:ee:1f:09"|"00:1b:17:00:47:11"||||"192.168.0.1"|"192.0.2.49"|"39255"|"49"|"·······A····"|"52"|"Q: Authentication"
"227"|"429500811.308673000"|"82"|"TACACS+"|"00:00:0c:07:ac:fa"|"00:1b:17:00:23:11"||||"192.0.2.49"|"192.168.0.1"|"49"|"39255"|"·······AP···"|"28"|"R: Authentication"
"228"|"429500814.510313000"|"75"|"TACACS+"|"3c:13:cc:ee:1f:09"|"00:1b:17:00:47:11"||||"192.168.0.1"|"192.0.2.49"|"39255"|"49"|"·······A····"|"21"|"Q: Authentication"
"229"|"429602608.727746000"|"163"|"RADIUS"|"00:0c:29:b7:1d:68"|"00:0c:29:a8:26:f7"||||||||||"Access-Request id=238"
"230"|"429602608.728132000"|"94"|"RADIUS"|"00:0c:29:a8:26:f7"|"00:0c:29:b7:1d:68"||||||||||"Access-Accept id=238"
"231"|"429602615.318187000"|"148"|"RADIUS"|"00:0c:29:b7:1d:68"|"00:0c:29:a8:26:f7"||||||||||"Access-Request id=23"
"232"|"454399727.184859000"|"85"|"IPMB"|"1c:69:7a:0f:cc:5e"|"64:7c:e8:8a:79:12"||||||||||"Session ID 0x0"
"233"|"454399727.186819000"|"92"|"IPMB"|"64:7c:e8:8a:79:12"|"1c:69:7a:0f:cc:5e"||||||||||"Session ID 0x0"
"234"|"454399727.186860000"|"110"|"RMCP+"|"1c:69:7a:0f:cc:5e"|"64:7c:e8:8a:79:12"||||||||||"Session ID 0x0, payload type: RMCP+ Open Session Request"
"235"|"454399727.188114000"|"114"|"RMCP+"|"64:7c:e8:8a:79:12"|"1c:69:7a:0f:cc:5e"||||||||||"Session ID 0x0, payload type: RMCP+ Open Session Response"
"236"|"454399727.188175000"|"114"|"RMCP+"|"1c:69:7a:0f:cc:5e"|"64:7c:e8:8a:79:12"||||||||||"Session ID 0x0, payload type: RAKP Message 1"
"237"|"454399746.737632000"|"65"|"IPMB"|"1c:69:7a:0f:cc:5e"|"64:7c:e8:8a:79:12"||||"192.168.7.53"|"192.168.3.83"|||||"Session ID 0x0"
"238"|"454408036.187329000"|"116"|"WOL"|"1c:69:7a:0f:cc:5e"|"b8:27:eb:bc:cd:b4"||||||||||"MagicPacket for b8:27:eb:bc:cd:b4"
"239"|"454408044.615013000"|"144"|"WOL"|"1c:69:7a:0f:cc:5e"|"ff:ff:ff:ff:ff:ff"||||"192.168.7.53"|"255.255.255.255"|||||"MagicPacket for b8:27:eb:bc:cd:b4"
"240"|"454546685.195255000"|"97"|"LLMNR"|"00:e0:4c:68:66:c1"|"33:33:00:01:00:03"||||||||||"Standard query 0x374c ANY Johannes-Dell"
"241"|"454546685.195813000"|"77"|"LLMNR"|"00:e0:4c:68:66:c1"|"01:00:5e:00:00:fc"||||"169.254.140.132"|"224.0.0.252"|||||"Standard query 0x374c ANY Johannes-Dell"
"242"|"454546685.618080000"|"97"|"LLMNR"|"00:e0:4c:68:66:c1"|"33:33:00:01:00:03"||||||||||"Standard query 0x2b43 ANY Johannes-Dell"
"243"|"454546686.485549000"|"114"|"NBNS"|"00:e0:4c:68:66:c1"|"ff:ff:ff:ff:ff:ff"||||"169.254.140.132"|"169.254.255.255"|||||"Registration NB JOHANNES-DELL<00>"
"244"|"454546686.486023000"|"114"|"NBNS"|"00:e0:4c:68:66:c1"|"ff:ff:ff:ff:ff:ff"||||"169.254.140.132"|"169.254.255.255"|||||"Registration NB WORKGROUP<00>"
"245"|"454546686.486437000"|"114"|"NBNS"|"00:e0:4c:68:66:c1"|"ff:ff:ff:ff:ff:ff"||||"169.254.140.132"|"169.254.255.255"|||||"Registration NB JOHANNES-DELL<20>"
"246"|"454546689.669840000"|"106"|"PCP v2"|"00:e0:4c:68:66:c1"|"00:86:9c:e7:55:14"||||"10.0.1.97"|"10.0.1.1"|||||"Map Request: 6881 -> 6881 [TCP]"
"247"|"454546689.920294000"|"106"|"PCP v2"|"00:e0:4c:68:66:c1"|"00:86:9c:e7:55:14"||||"10.0.1.97"|"10.0.1.1"|||||"Map Request: 6881 -> 6881 [TCP]"
"248"|"454546690.421028000"|"106"|"PCP v2"|"00:e0:4c:68:66:c1"|"00:86:9c:e7:55:14"||||"10.0.1.97"|"10.0.1.1"|||||"Map Request: 6881 -> 6881 [TCP]"
"249"|"454546699.680527000"|"722"|"UDP/XML"|"00:e0:4c:68:66:c1"|"33:33:00:00:00:0c"||||||||||"49307 → 3702 Len=656"
"250"|"454546699.681031000"|"702"|"UDP/XML"|"00:e0:4c:68:66:c1"|"01:00:5e:7f:ff:fa"||||"10.0.1.97"|"239.255.255.250"|||||"49306 → 3702 Len=656"
"251"|"454546699.818955000"|"722"|"UDP/XML"|"00:e0:4c:68:66:c1"|"33:33:00:00:00:0c"||||||||||"49307 → 3702 Len=656"
"252"|"471763158.868748000"|"103"|"POP"|"70:4c:a5:99:4a:b3"|"00:0c:29:5f:2c:a1"||||||"110"|"53955"|"·······AP···"|"29"|"S: +OK Dovecot (Debian) ready."
"253"|"471763158.871509000"|"80"|"POP"|"00:0c:29:5f:2c:a1"|"70:4c:a5:99:4a:b3"||||||"53955"|"110"|"·······AP···"|"6"|"C: CAPA"
"254"|"471763158.890517000"|"163"|"POP"|"70:4c:a5:99:4a:b3"|"00:0c:29:5f:2c:a1"||||||"110"|"53955"|"·······AP···"|"89"|"S: +OK"
"255"|"471763236.207800000"|"501"|"POP/IMF"|"70:4c:a5:99:4a:b3"|"00:0c:29:5f:2c:a1"||||||"110"|"53958"|"·······AP···"|"427"|"courier,mo=  , nospace;"">&nbsp; blog: &nbsp;&nbsp;&nbsp;<a href=3D""https://weberblog.net?u=  , tm_source=3Dsignatur&amp;utm_medium=3Demail&amp;utm_campaign=3Dbusiness"">ht=  , tps://weberblog.net</a></span><br /><span style=3D""font-family: courier new=  , ,courier,monospace;"">&nbsp; twitter: <a href=3D""https://twitter.com/weberne=  , tz"">@webernetz</a></span></p>  , </div>  , </body></html>  ,   , --=_ed54ec42c4a7bd9efffe402e83d2e341--  ,   , .  "
"256"|"471764337.495506000"|"1179"|"POP/IMF"|"70:4c:a5:99:4a:b3"|"00:0c:29:5f:2c:a1"||||||"110"|"54004"|"·······AP···"|"1105"|"E*�\\a*������\\177�\\034�\\003��2����c7`f<�)'t���\\035HU�ٛ��T7@�\\017�23L��\\027�F\\036�%z|z�}'�F Ɂ��~�u��\\035\\027�ZÌ\\023H�Ʃh�R�db��z\\005����W�\\004,s�\\002����ӵ�{5?�[�\\031 i-�կ:��H��f&�\\177��\\035�ε�sd��>\\027�͢���7f0Yؓ�;e-�)\\027b�\\��\\0012i�\\026�r�3C��ˬU�3��bzW^��7\\005�L�:ȹ�\\006W��W�1-� , }\\005���1\\034��6�#�\\bQh�^ҷh�5�k���D\\016Vݙ~\\002?���\\004�\\005�w�\\022\\�j���.w�b�����*�\\v,���\\017�:�\\020�\\016�7�։ 㓋Iŀ���_\\033\\000x�\\003O\\017\\a�4\\037�wTg@�ħ`4�k*\\024ȴ|�ڀ\\001�q&6\\d\\027�� , \\177�:�OUN��\\036��\\016�D��\\0245\\027�7B�.""\\033�\\034c}�qw�B&lzzݠa��G\\030��>\\033���\\0359X , U�S�u\\033�\\v�}�e\\004\\017�װ���.7$�=�6���Zd�\\003t�^F��Jی\\021�˥\\020e��i�R�_��^I�\\022<?ެ\\032�d��tM���\\036Gi�9\\034+�-���t\\005C�\\016(��=�%-7�)�\\017V��㋼\\u0007<�Pp�� ,  ���q-�i\\v��\\!�N�w�\\\\b��͏r\\000�5��b*M�m�""mmI�ע�ei��h��\\032?��|� T���\\026䐙�{�5��!,������iʍ'��\\022 , \\000\\023�\\027.%�\\037\\b@�\\000��_�T+�领;��7l�>ʴV��f\\027�け�];\\v4�����8\\035�\\022$�{^��� ��Ia������\\030��\\036���}��\\006}����\\021Z������\\000c\\006�[\\000&K\\027\\003\\003\\001\\031�G\\aY\\006��\\002�J\\030W#0�u����4z��m�\\004�\\023FV�n�M����\\v�4�\\b�\\006Q��h0l\\026 , �Na������\\037Y���\\033�\\0002��&��\\034\\030:�o�ػpI��FS�p��y�j䫙��\\030r\\034\\033ʰ ���3\\035�$�i�f��\\036���v����]-�ٷM��f�MbW��J�p+Ҏ�ݷ\\000�\\016��νH�""X\\034��mZs\\031���A��`S�L鉩�!E�ќ��wr�\\030m�\\035��סB\\024j���ܧ�6�\\aΫ\\0329T��{��\\024��\\025\\eFk\\002��\\vpŽ� , ��g8:�`VAz\\016%{�3���VL4\\016���\\036�;o�\\027\\003\\003\\000E �H\\v�OW���c�K����g�\\030�K*�s]b��:��a����Y=�>�\\017᭶��'�.�1�����*��\\017"
"257"|"484474739.244638000"|"263"|"KRB5"|"00:0c:29:36:86:34"|"3c:fa:30:03:12:30"||||"192.168.3.53"|"172.16.80.10"|"52520"|"88"|"·······AP···"|"193"|"AS-REQ"
"258"|"484474739.245981000"|"293"|"KRB5"|"00:0c:29:a9:e4:e3"|"3c:fa:30:03:12:30"||||"172.16.80.10"|"192.168.3.53"|"88"|"52520"|"·······AP···"|"223"|"KRB Error: KRB5KDC_ERR_PREAUTH_REQUIRED"
"259"|"484474739.247014000"|"343"|"KRB5"|"00:0c:29:36:86:34"|"3c:fa:30:03:12:30"||||"192.168.3.53"|"172.16.80.10"|"52524"|"88"|"·······AP···"|"273"|"AS-REQ"
"260"|"484543807.661851000"|"241"|"CLDAP"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||||||"searchRequest(1) ""<ROOT>"" baseObject "
"261"|"484543807.662349000"|"252"|"CLDAP"|"00:0c:29:a9:e4:e3"|"3c:fa:30:03:12:30"||||||||||"searchResEntry(1) ""<ROOT>"" searchResDone(1) success  [1 result]"
"262"|"484544000.217961000"|"241"|"CLDAP"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||||||"searchRequest(1) ""<ROOT>"" baseObject "
"263"|"484544013.254439000"|"151"|"SMB"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49958"|"445"|"·······AP···"|"73"|"Negotiate Protocol Request"
"264"|"484544013.255264000"|"330"|"SMB2"|"00:0c:29:a9:e4:e3"|"3c:fa:30:03:12:30"||||||"445"|"49958"|"·······AP···"|"252"|"Negotiate Protocol Response"
"265"|"484544013.255546000"|"350"|"SMB2"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49958"|"445"|"·······AP···"|"272"|"Negotiate Protocol Request"
"266"|"484544013.255995000"|"414"|"SMB2"|"00:0c:29:a9:e4:e3"|"3c:fa:30:03:12:30"||||||"445"|"49958"|"·······AP···"|"336"|"Negotiate Protocol Response"
"267"|"484544013.886269000"|"428"|"LDAP"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49961"|"389"|"·······AP···"|"350"|"searchRequest(6) ""<ROOT>"" baseObject "
"268"|"484544013.886942000"|"1499"|"TCP"|"00:0c:29:a9:e4:e3"|"3c:fa:30:03:12:30"||||||"389"|"49961"|"·······AP···"|"1421"|"389 → 49961 [PSH, ACK] Seq=1441 Ack=351 Win=8194 Len=1421"
"269"|"484544013.911664000"|"158"|"LDAP"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49961"|"389"|"·······AP···"|"80"|"bindRequest(8) ""<ROOT>"" , NTLMSSP_NEGOTIATEsasl "
"270"|"484544015.538119000"|"151"|"SMB"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49963"|"445"|"·······AP···"|"73"|"Negotiate Protocol Request"
"271"|"484544038.721283000"|"151"|"SMB"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49975"|"445"|"·······AP···"|"73"|"Negotiate Protocol Request"
"272"|"484544038.824722000"|"1518"|"NBSS"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49975"|"445"|"·······A····"|"1440"|"[TCP Previous segment not captured] Session message"
"273"|"484544038.982020000"|"238"|"DCERPC"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49984"|"135"|"·······AP···"|"160"|"Bind: call_id: 2, Fragment: Single, 3 context items: EPMv4 V3.0 (32bit NDR), EPMv4 V3.0 (64bit NDR), EPMv4 V3.0 (6cb71c2c-9812-4540-0300-000000000000)"
"274"|"484544038.982535000"|"186"|"DCERPC"|"00:0c:29:a9:e4:e3"|"3c:fa:30:03:12:30"||||||"135"|"49984"|"·······AP···"|"108"|"Bind_ack: call_id: 2, Fragment: Single, max_xmit: 5840 max_recv: 5840, 3 results: Provider rejection, Acceptance, Negotiate ACK"
"275"|"484544038.983520000"|"246"|"EPM"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49984"|"135"|"·······AP···"|"168"|"Map request, DRSUAPI, 32bit NDR"
"276"|"484544038.984007000"|"346"|"EPM"|"00:0c:29:a9:e4:e3"|"3c:fa:30:03:12:30"||||||"135"|"49984"|"·······AP···"|"268"|"Map response, DRSUAPI, 32bit NDR, DRSUAPI, 32bit NDR"
"277"|"484544038.995335000"|"720"|"TCP"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49985"|"49667"|"·······AP···"|"642"|"49985 → 49667 [PSH, ACK] Seq=1 Ack=1 Win=1029 Len=642"
"278"|"484544039.001248000"|"346"|"DCERPC"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49985"|"49667"|"·······AP···"|"268"|"[TCP Previous segment not captured] Request: call_id: 2, Fragment: Single, opnum: 0, Ctx: 1"
"279"|"484544039.001696000"|"282"|"DCERPC"|"00:0c:29:a9:e4:e3"|"3c:fa:30:03:12:30"||||||"49667"|"49985"|"·······AP···"|"204"|"[TCP ACKed unseen segment] Response: call_id: 2, Fragment: Single, Ctx: 1"
"280"|"484544039.002154000"|"330"|"DCERPC"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49985"|"49667"|"·······AP···"|"252"|"Request: call_id: 3, Fragment: Single, opnum: 12, Ctx: 1"
"281"|"484544040.028453000"|"246"|"EPM"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49984"|"135"|"·······AP···"|"168"|"Map request, DRSUAPI, 32bit NDR"
"282"|"484544040.707178000"|"1518"|"NBSS"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49975"|"445"|"·······A····"|"1440"|"[TCP Previous segment not captured] Session message"
"283"|"484544041.139586000"|"270"|"DCERPC"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49998"|"61737"|"·······AP···"|"192"|"Request: call_id: 2, Fragment: Single, opnum: 4, Ctx: 1"
"284"|"484544041.139988000"|"114"|"DCERPC"|"00:0c:29:a9:e4:e3"|"3c:fa:30:03:12:30"||||||"61737"|"49998"|"·······AP···"|"36"|"Response: call_id: 2, Fragment: Single, Ctx: 1"
"285"|"484544041.140544000"|"338"|"DCERPC"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49998"|"61737"|"·······AP···"|"260"|"Request: call_id: 3, Fragment: Single, opnum: 26, Ctx: 1"
"286"|"484544063.641110000"|"326"|"DCERPC"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49999"|"49667"|"·······AP···"|"248"|"Request: call_id: 8, Fragment: Single, opnum: 76, Ctx: 3"
"287"|"484544063.641720000"|"422"|"DCERPC"|"00:0c:29:a9:e4:e3"|"3c:fa:30:03:12:30"||||||"49667"|"49999"|"·······AP···"|"344"|"Response: call_id: 8, Fragment: Single, Ctx: 3"
"288"|"484544063.644051000"|"278"|"DCERPC"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49999"|"49667"|"·······AP···"|"200"|"[TCP ACKed unseen segment] [TCP Previous segment not captured] Request: call_id: 9, Fragment: Single, opnum: 76, Ctx: 3"
"289"|"484544095.041421000"|"1518"|"NBSS"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49671"|"445"|"·······A····"|"1440"|"Session message"
"290"|"484552191.206919000"|"346"|"DCERPC"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49679"|"445"|"·······AP···"|"268"|"Request: call_id: 2, Fragment: Single, opnum: 64, Ctx: 1"
"291"|"484552191.207611000"|"258"|"DCERPC"|"00:0c:29:a9:e4:e3"|"3c:fa:30:03:12:30"||||||"445"|"49679"|"·······AP···"|"180"|"Response: call_id: 2, Fragment: Single, Ctx: 1"
"292"|"484552191.207945000"|"254"|"DCERPC"|"00:0c:29:c3:7f:eb"|"3c:fa:30:03:12:30"||||||"49679"|"445"|"·······AP···"|"176"|"Request: call_id: 3, Fragment: Single, opnum: 6, Ctx: 1"
"293"|"484553085.140167000"|"121"|"RDP"|"9c:2d:cd:3f:0e:c0"|"3c:fa:30:03:12:12"||||||"5403"|"3389"|"·······AP···"|"47"|"Cookie: mstshash=weberjoh-, Negotiate Request"
"294"|"484553085.150100000"|"93"|"RDP"|"3c:fa:30:03:12:12"|"9c:2d:cd:3f:0e:c0"||||||"3389"|"5403"|"·······AP···"|"19"|"Negotiate Response"
"295"|"484553095.244813000"|"121"|"RDP"|"9c:2d:cd:3f:0e:c0"|"3c:fa:30:03:12:12"||||||"5409"|"3389"|"·······AP···"|"47"|"Cookie: mstshash=weberjoh-, Negotiate Request"
"296"|"484553095.330926000"|"1294"|"RDPUDP"|"9c:2d:cd:3f:0e:c0"|"3c:fa:30:03:12:12"||||||||||"SYN,CORRELATIONID,SYNEX"
"297"|"484553095.332002000"|"1294"|"RDPUDP"|"3c:fa:30:03:12:12"|"9c:2d:cd:3f:0e:c0"||||||||||"SYN,SYNEX"
"298"|"484553095.338075000"|"1069"|"RDPUDP2"|"9c:2d:cd:3f:0e:c0"|"3c:fa:30:03:12:12"||||||||||"AOA,DUMMY"
"299"|"484553095.344854000"|"1080"|"RDPUDP2"|"3c:fa:30:03:12:12"|"9c:2d:cd:3f:0e:c0"||||||||||"ACK,OVERHEAD,DELAYACK,AOA,DUMMY"
"300"|"484553095.344854000"|"1069"|"RDPUDP2"|"3c:fa:30:03:12:12"|"9c:2d:cd:3f:0e:c0"||||||||||"AOA,DUMMY"
"301"|"484633303.877973000"|"56"|"UDPENCAP"|"3c:fa:30:03:12:10"|"00:70:76:69:66:00"||||"100.93.7.250"|"194.247.4.10"|||||"NAT-keepalive"
"302"|"484633323.877808000"|"56"|"UDPENCAP"|"3c:fa:30:03:12:10"|"00:70:76:69:66:00"||||"100.93.7.250"|"194.247.4.10"|||||"NAT-keepalive"
"303"|"484633343.877805000"|"56"|"UDPENCAP"|"3c:fa:30:03:12:10"|"00:70:76:69:66:00"||||"100.93.7.250"|"194.247.4.10"|||||"NAT-keepalive"
"304"|"484634967.586288000"|"62"|"STUN"|"9c:2d:cd:3f:0e:c0"|"3c:fa:30:03:12:12"||||"192.168.21.41"|"52.59.186.27"|||||"Binding Request"
"305"|"484634967.591048000"|"110"|"STUN"|"3c:fa:30:03:12:12"|"9c:2d:cd:3f:0e:c0"||||"52.59.186.27"|"192.168.21.41"|||||"Binding Success Response XOR-MAPPED-ADDRESS: 94.31.100.250:20117 MAPPED-ADDRESS: 94.31.100.250:20117"
"306"|"484634967.596567000"|"138"|"STUN"|"9c:2d:cd:3f:0e:c0"|"3c:fa:30:03:12:12"||||"192.168.21.41"|"192.168.11.1"|||||"Binding Request user: d4L6:0880ee37"
"307"|"484634967.648034000"|"199"|"DTLS"|"3c:fa:30:03:12:12"|"9c:2d:cd:3f:0e:c0"||||"192.168.11.1"|"192.168.21.41"|||||"Client Hello"
"308"|"484634967.648301000"|"747"|"DTLSv1.2"|"9c:2d:cd:3f:0e:c0"|"3c:fa:30:03:12:12"||||"192.168.21.41"|"192.168.11.1"|||||"Server Hello, Certificate, Server Key Exchange, Certificate Request, Server Hello Done"
"309"|"484634967.650474000"|"587"|"DTLSv1.2"|"3c:fa:30:03:12:12"|"9c:2d:cd:3f:0e:c0"||||"192.168.11.1"|"192.168.21.41"|||||"Certificate, Client Key Exchange, Certificate Verify, Change Cipher Spec, Encrypted Handshake Message"
"310"|"484634967.650837000"|"117"|"DTLSv1.2"|"9c:2d:cd:3f:0e:c0"|"3c:fa:30:03:12:12"||||"192.168.21.41"|"192.168.11.1"|||||"Change Cipher Spec, Encrypted Handshake Message"
"311"|"484634968.791945000"|"70"|"SRTCP"|"9c:2d:cd:3f:0e:c0"|"3c:fa:30:03:12:12"||||"192.168.21.41"|"192.168.11.1"|||||"Receiver Report   "
"312"|"484634968.826701000"|"74"|"SRTCP"|"9c:2d:cd:3f:0e:c0"|"3c:fa:30:03:12:12"||||"192.168.21.41"|"192.168.11.1"|||||"Payload-specific Feedback   PLI  "
"313"|"484634968.863348000"|"74"|"SRTCP"|"9c:2d:cd:3f:0e:c0"|"3c:fa:30:03:12:12"||||"192.168.21.41"|"192.168.11.1"|||||"Payload-specific Feedback   PLI  "
"314"|"520244834.114021000"|"1262"|"QUIC"|"1c:69:7a:0f:cc:5e"|"3c:fa:30:03:12:12"||||||||||"Initial, DCID=9ab19cb12ffe2349, SCID=b7ee840d86bde7ab, PKN: 0, CRYPTO"
"315"|"520244834.126501000"|"198"|"QUIC"|"3c:fa:30:03:12:12"|"1c:69:7a:0f:cc:5e"||||||||||"Retry, DCID=b7ee840d86bde7ab, SCID=49843e5a"
"316"|"520244834.129371000"|"1262"|"QUIC"|"1c:69:7a:0f:cc:5e"|"3c:fa:30:03:12:12"||||||||||"Initial, DCID=49843e5a, SCID=b7ee840d86bde7ab, PKN: 1, CRYPTO"
"317"|"527525175.633854000"|"102"|"MACSEC"|"00:0c:29:55:9b:4b"|"33:33:00:00:00:02"||||||||||"MACsec frame [UNVERIFIED]"
"318"|"527525186.777831000"|"102"|"MACSEC"|"00:0c:29:ef:7c:66"|"33:33:00:00:00:02"||||||||||"MACsec frame [UNVERIFIED]"
"319"|"527525210.807591000"|"130"|"MACSEC"|"00:0c:29:55:9b:4b"|"00:0c:29:ef:7c:66"||||||||||"MACsec frame [UNVERIFIED]"
"320"|"532614529.410582000"|"126"|"EAPOL-MKA"|"aa:c1:ab:1d:d3:cc"|"01:80:c2:00:00:03"||||||||||"Potential Peer List, ICV Indicator"
"321"|"532614529.487984000"|"126"|"EAPOL-MKA"|"aa:c1:ab:6e:91:a9"|"01:80:c2:00:00:03"||||||||||"Potential Peer List, ICV Indicator"
"322"|"532614531.432267000"|"226"|"EAPOL-MKA"|"aa:c1:ab:1d:d3:cc"|"01:80:c2:00:00:03"||||||||||"Key Server, Live Peer List, MACsec SAK Use, Distributed SAK, ICV Indicator"
"323"|"538124601.101827000"|"78"|"Modbus/TCP"|"bc:24:11:d6:d0:8f"|"bc:24:11:29:2b:31"||||"192.168.222.129"|"192.168.222.131"|"45286"|"502"|"·······AP···"|"12"|"   Query: Trans:     1; Unit:   1, Func:   1: Read Coils"
"324"|"538124601.103589000"|"76"|"Modbus/TCP"|"bc:24:11:29:2b:31"|"bc:24:11:d6:d0:8f"||||"192.168.222.131"|"192.168.222.129"|"502"|"45286"|"·······AP···"|"10"|"Response: Trans:     1; Unit:   1, Func:   1: Read Coils"
"325"|"538124601.103810000"|"78"|"Modbus/TCP"|"bc:24:11:d6:d0:8f"|"bc:24:11:29:2b:31"||||"192.168.222.129"|"192.168.222.131"|"45286"|"502"|"·······AP···"|"12"|"   Query: Trans:     2; Unit:   1, Func:   2: Read Discrete Inputs"
"326"|"538124659.985675000"|"54"|"Modbus/UDP"|"bc:24:11:d6:d0:8f"|"bc:24:11:29:2b:31"||||"192.168.222.129"|"192.168.222.131"|||||"   Query: Trans:     1; Unit:   1, Func:   1: Read Coils"
"327"|"538124659.986949000"|"52"|"Modbus/UDP"|"bc:24:11:29:2b:31"|"bc:24:11:d6:d0:8f"||||"192.168.222.131"|"192.168.222.129"|||||"Response: Trans:     1; Unit:   1, Func:   1: Read Coils"
"328"|"538124659.987110000"|"54"|"Modbus/UDP"|"bc:24:11:d6:d0:8f"|"bc:24:11:29:2b:31"||||"192.168.222.129"|"192.168.222.131"|||||"   Query: Trans:     2; Unit:   1, Func:   2: Read Discrete Inputs"
```

## Conversations

```text
================================================================================
TCP Conversations
Filter:<No Filter>
                                                           |       <-      | |       ->      | |     Total     |    Relative    |   Duration   |
                                                           | Frames  Bytes | | Frames  Bytes | | Frames  Bytes |      Start     |              |
10.200.200.202:179         <-> 10.200.200.201:23975             2 154 bytes       4 291 bytes       6 445 bytes  113831518.641075999        60.0140
2a00:6020:ad0b:8381:fd70:e2f9:f031:9dc4:49984 <-> 2a00:6020:ad0b:8380::10:135       2 532 bytes       3 730 bytes       5 1262 bytes  484544038.982020020         1.0464
192.168.110.9:50477        <-> 80.154.108.235:443               2 3714 bytes       2 553 bytes       4 4267 bytes  218860580.149318010         0.0172
80.154.108.237:25          <-> 192.168.110.9:45271              2 3225 bytes       2 186 bytes       4 3411 bytes  218860681.967007011         0.0057
2003:51:6012:110::b15:22:60892 <-> 2003:51:6012:121::2:22           1 97 bytes        3 1095 bytes       4 1192 bytes  247148873.345503002         0.0076
2a00:6020:ad0b:8381:fd70:e2f9:f031:9dc4:49958 <-> 2a00:6020:ad0b:8380::10:445       2 744 bytes       2 501 bytes       4 1245 bytes  484544013.254438996         0.0016
2a00:6020:ad0b:8381:fd70:e2f9:f031:9dc4:49985 <-> 2a00:6020:ad0b:8380::10:49667       1 282 bytes       3 1396 bytes       4 1678 bytes  484544038.995334983         0.0068
2003:de:2016:120::a08:53:143 <-> 2003:de:2016:125:fc36:8317:4e86:cb72:7552       1 100 bytes       2 274 bytes       3 374 bytes  283324496.489880025         1.3602
141.41.241.70:443          <-> 141.41.39.187:40976              1 146 bytes       2 1557 bytes       3 1703 bytes  332801714.808215022         0.0176
2001:470:765b::b15:22:52222 <-> 2001:48a8:6880::18:43            2 752 bytes       1 90 bytes        3 842 bytes  335418362.555326998         0.1060
2001:db8::2:18716          <-> 2001:db8::1:23                   1 110 bytes       2 211 bytes       3 321 bytes  338279056.995257020         0.0047
10.82.185.11:57895         <-> 192.168.11.10:515                1 60 bytes        2 137 bytes       3 197 bytes  356411927.059378028         0.0019
5.35.226.136:21            <-> 10.82.185.11:51072               1 82 bytes        2 199 bytes       3 281 bytes  361182722.921896994         1.3780
2001:470:1f0b:16b0:6986:b8d4:3649:9cbe:55600 <-> 2001:470:1f0b:16b0:221:70ff:feb2:e6c:514       0 0 bytes         3 776 bytes       3 776 bytes  380094905.578687012         0.0075
192.168.0.1:39255          <-> 192.0.2.49:49                    1 82 bytes        2 181 bytes       3 263 bytes  429500811.305504024         3.2048
2a01:488:42:1000:50ed:8588:8a:c570:110 <-> 2001:470:1f0b:16b0:f83f:53c1:be1:eca1:53955       1 80 bytes        2 266 bytes       3 346 bytes  471763158.868748009         0.0218
2a00:6020:ad0b:8381:fd70:e2f9:f031:9dc4:49961 <-> 2a00:6020:ad0b:8380::10:389       1 1499 bytes       2 586 bytes       3 2085 bytes  484544013.886268973         0.0254
2a00:6020:ad0b:8381:fd70:e2f9:f031:9dc4:49975 <-> 2a00:6020:ad0b:8380::10:445       0 0 bytes         3 3187 bytes       3 3187 bytes  484544038.721283019         1.9859
2a00:6020:ad0b:8381:fd70:e2f9:f031:9dc4:49998 <-> 2a00:6020:ad0b:8380::10:61737       1 114 bytes       2 608 bytes       3 722 bytes  484544041.139585972         0.0010
2a00:6020:ad0b:8381:fd70:e2f9:f031:9dc4:49999 <-> 2a00:6020:ad0b:8380::10:49667       1 422 bytes       2 604 bytes       3 1026 bytes  484544063.641110003         0.0029
2a00:6020:ad0b:8381:15f6:aeec:61d9:205f:49679 <-> 2a00:6020:ad0b:8380::10:445       1 258 bytes       2 600 bytes       3 858 bytes  484552191.206919014         0.0010
192.168.222.129:45286      <-> 192.168.222.131:502              1 76 bytes        2 156 bytes       3 232 bytes  538124601.101827025         0.0020
192.168.110.10:1152        <-> 80.237.133.136:80                1 97 bytes        1 365 bytes       2 462 bytes  148881240.483381003         0.2695
2003:de:2016:125:fc36:8317:4e86:cb72:7549 <-> 2003:de:2016:110::a12:443:443       1 779 bytes       1 271 bytes       2 1050 bytes  283324490.367660999         0.0058
2003:de:2016:110::b15:22:22 <-> 2003:de:2016:125:fc36:8317:4e86:cb72:7563       1 106 bytes       1 119 bytes       2 225 bytes  283324516.585283995         0.0058
2001:470:765b:0:1c6e:18ae:ddb4:3bc1:55996 <-> 2a02:26f0:6c00::210:ba61:80       1 982 bytes       1 276 bytes       2 1258 bytes  343727186.438643992         0.1636
5.35.226.136:51652         <-> 10.82.185.11:51075               0 0 bytes         2 2068 bytes       2 2068 bytes  361182725.431779027         0.0000
192.168.7.16:49404         <-> 192.168.7.17:10051               1 169 bytes       1 299 bytes       2 468 bytes  416449302.153568983         0.0002
192.168.3.53:52520         <-> 172.16.80.10:88                  1 293 bytes       1 263 bytes       2 556 bytes  484474739.244638026         0.0013
2a00:6020:ad0b:8321:34fb:d0ff:9ee8:abf2:5403 <-> 2a00:6020:ad0b:8380::10:3389       1 93 bytes        1 121 bytes       2 214 bytes  484553085.140166998         0.0099
192.168.110.10:1154        <-> 212.144.254.123:3128             0 0 bytes         1 388 bytes       1 388 bytes  148881257.793036014         0.0000
192.168.7.12:1226          <-> 192.168.7.1:51108                0 0 bytes         1 561 bytes       1 561 bytes  268488077.412208974         0.0000
192.168.7.12:1227          <-> 192.168.7.1:51108                0 0 bytes         1 108 bytes       1 108 bytes  268488077.490175009         0.0000
192.168.7.12:1228          <-> 192.168.7.1:51108                0 0 bytes         1 778 bytes       1 778 bytes  268488077.490243971         0.0000
2003:de:2016:125:fc36:8317:4e86:cb72:7562 <-> 2003:de:2016:120::a08:53:25       0 0 bytes         1 260 bytes       1 260 bytes  283324512.276106000         0.0000
2001:470:765b:0:1c6e:18ae:ddb4:3bc1:35638 <-> 2a02:26f0:6c00::210:ba60:80       0 0 bytes         1 276 bytes       1 276 bytes  343732136.336299002         0.0000
52.109.32.27:443           <-> 192.168.7.35:63594               0 0 bytes         1 1518 bytes       1 1518 bytes  357620708.871252000         0.0000
5.35.226.136:52833         <-> 10.82.185.11:51076               0 0 bytes         1 1506 bytes       1 1506 bytes  361182727.627065003         0.0000
192.168.7.16:49406         <-> 192.168.7.17:10051               0 0 bytes         1 161 bytes       1 161 bytes  416449332.158542991         0.0000
194.247.5.12:13            <-> 85.215.94.29:43624               0 0 bytes         1 80 bytes        1 80 bytes   427350545.701790988         0.0000
194.247.5.12:37            <-> 85.215.94.29:49510               0 0 bytes         1 58 bytes        1 58 bytes   427350600.190400004         0.0000
2a01:488:42:1000:50ed:8588:8a:c570:110 <-> 2001:470:1f0b:16b0:f83f:53c1:be1:eca1:53958       0 0 bytes         1 501 bytes       1 501 bytes  471763236.207799971         0.0000
2a01:488:42:1000:50ed:8588:8a:c570:110 <-> 2001:470:1f0b:16b0:f83f:53c1:be1:eca1:54004       0 0 bytes         1 1179 bytes       1 1179 bytes  471764337.495505989         0.0000
192.168.3.53:52524         <-> 172.16.80.10:88                  0 0 bytes         1 343 bytes       1 343 bytes  484474739.247013986         0.0000
2a00:6020:ad0b:8381:fd70:e2f9:f031:9dc4:49963 <-> 2a00:6020:ad0b:8380::10:445       0 0 bytes         1 151 bytes       1 151 bytes  484544015.538119018         0.0000
2a00:6020:ad0b:8381:c8c2:18f1:4b9d:7524:49671 <-> 2a00:6020:ad0b:8380::10:445       0 0 bytes         1 1518 bytes       1 1518 bytes  484544095.041420996         0.0000
2a00:6020:ad0b:8321:34fb:d0ff:9ee8:abf2:5409 <-> 2a00:6020:ad0b:8380::10:3389       0 0 bytes         1 121 bytes       1 121 bytes  484553095.244813025         0.0000
================================================================================
================================================================================
IPv4 Conversations
Filter:<No Filter>
                                               |       <-      | |       ->      | |     Total     |    Relative    |   Duration   |
                                               | Frames  Bytes | | Frames  Bytes | | Frames  Bytes |      Start     |              |
10.200.200.202       <-> 10.200.200.201             3 272 bytes       6 563 bytes       9 835 bytes  113831508.720075995        69.9350
127.0.0.1            <-> 127.0.0.1                  0 0 bytes         9 1107 bytes       9 1107 bytes  150640362.682969004   5439385.3407
172.16.23.2          <-> 192.168.47.1               4 294 bytes       4 335 bytes       8 629 bytes  338277664.949613988      1392.0504
192.168.21.41        <-> 192.168.11.1               2 786 bytes       6 1220 bytes       8 2006 bytes  484634967.596566975         1.2668
217.0.21.65          <-> 84.146.135.221             3 2290 bytes       3 2707 bytes       6 4997 bytes  332407522.889362991    263303.4407
5.35.226.136         <-> 10.82.185.11               1 82 bytes        5 3773 bytes       6 3855 bytes  361182722.921896994         4.7052
194.247.5.12         <-> 85.215.94.29               2 120 bytes       4 252 bytes       6 372 bytes  427350545.701790988        65.9744
192.168.222.129      <-> 192.168.222.131            2 128 bytes       4 264 bytes       6 392 bytes  538124601.101827025        58.8853
192.168.2.1          <-> 192.168.2.102              2 154 bytes       2 674 bytes       4 828 bytes  188826407.333048999        36.4354
192.168.110.9        <-> 80.154.108.235             2 3714 bytes       2 553 bytes       4 4267 bytes  218860580.149318010         0.0172
80.154.108.237       <-> 192.168.110.9              2 3225 bytes       2 186 bytes       4 3411 bytes  218860681.967007011         0.0057
10.0.0.2             <-> 10.0.0.1                   1 114 bytes       2 228 bytes       3 342 bytes  113831514.008075997         0.0160
192.168.121.10       <-> 192.168.120.10             0 0 bytes         3 467 bytes       3 467 bytes  247148582.258426011       148.7814
192.168.121.2        <-> 192.168.110.10             2 128 bytes       1 88 bytes        3 216 bytes  247148902.627427012         0.0220
192.168.7.1          <-> 239.255.255.250            0 0 bytes         3 507 bytes       3 507 bytes  268487977.681648016         9.9999
192.168.7.12         <-> 192.168.7.1                0 0 bytes         3 1447 bytes       3 1447 bytes  268488077.412208974         0.0780
216.66.80.30         <-> 193.24.227.12              1 138 bytes       2 276 bytes       3 414 bytes  315983834.940446973         1.0002
84.146.135.221       <-> 217.0.5.215                0 0 bytes         3 678 bytes       3 678 bytes  332407553.264833987         0.0401
141.41.241.70        <-> 141.41.39.187              1 146 bytes       2 1557 bytes       3 1703 bytes  332801714.808215022         0.0176
10.82.185.11         <-> 192.168.11.10              1 60 bytes        2 137 bytes       3 197 bytes  356411927.059378028         0.0019
193.24.225.54        <-> 193.24.225.56              1 70 bytes        2 140 bytes       3 210 bytes  361949125.378430009         0.7575
192.168.20.2         <-> 224.0.0.2                  0 0 bytes         3 198 bytes       3 198 bytes  411633454.736446023         3.0268
192.168.7.16         <-> 192.168.7.17               1 169 bytes       2 460 bytes       3 629 bytes  416449302.153568983        30.0050
192.168.0.1          <-> 192.0.2.49                 1 82 bytes        2 181 bytes       3 263 bytes  429500811.305504024         3.2048
169.254.140.132      <-> 169.254.255.255            0 0 bytes         3 342 bytes       3 342 bytes  454546686.485548973         0.0009
10.0.1.97            <-> 10.0.1.1                   0 0 bytes         3 318 bytes       3 318 bytes  454546689.669839978         0.7512
192.168.3.53         <-> 172.16.80.10               1 293 bytes       2 606 bytes       3 899 bytes  484474739.244638026         0.0024
100.93.7.250         <-> 194.247.4.10               0 0 bytes         3 168 bytes       3 168 bytes  484633303.877973020        39.9998
10.0.0.1             <-> 224.0.0.2                  0 0 bytes         2 152 bytes       2 152 bytes  113831506.536075994         0.0000
10.0.0.2             <-> 224.0.0.5                  0 0 bytes         2 188 bytes       2 188 bytes  113831510.763076007         0.0000
192.168.110.10       <-> 80.237.133.136             1 97 bytes        1 365 bytes       2 462 bytes  148881240.483381003         0.2695
0.0.0.0              <-> 255.255.255.255            0 0 bytes         2 684 bytes       2 684 bytes  188826405.316543013         2.0168
192.168.121.40       <-> 212.224.120.164            1 94 bytes        1 94 bytes        2 188 bytes  247148588.577903003         0.0020
192.168.121.2        <-> 192.168.121.254            1 70 bytes        1 98 bytes        2 168 bytes  247148595.823276013         0.0025
192.168.7.12         <-> 239.255.255.250            0 0 bytes         2 128 bytes       2 128 bytes  268487981.881812990       134.9305
193.24.225.54        <-> 193.24.225.54              0 0 bytes         2 128 bytes       2 128 bytes  361949126.371756017         0.0001
192.168.21.41        <-> 52.59.186.27               1 110 bytes       1 62 bytes        2 172 bytes  484634967.586287975         0.0048
10.0.0.2             <-> 224.0.0.2                  0 0 bytes         1 76 bytes        1 76 bytes   113831506.723076001         0.0000
10.0.0.1             <-> 224.0.0.5                  0 0 bytes         1 94 bytes        1 94 bytes   113831511.980076000         0.0000
192.168.110.10       <-> 212.144.254.123            0 0 bytes         1 388 bytes       1 388 bytes  148881257.793036014         0.0000
192.168.2.1          <-> 224.0.0.1                  0 0 bytes         1 50 bytes        1 50 bytes   188826445.856070012         0.0000
192.168.10.1         <-> 224.0.0.9                  0 0 bytes         1 130 bytes       1 130 bytes  247148575.370476991         0.0000
192.168.121.254      <-> 224.0.0.102                0 0 bytes         1 118 bytes       1 118 bytes  247148576.008451015         0.0000
192.168.121.253      <-> 224.0.0.102                0 0 bytes         1 118 bytes       1 118 bytes  247148577.135630995         0.0000
192.168.121.253      <-> 224.0.0.9                  0 0 bytes         1 70 bytes        1 70 bytes   247148578.864152014         0.0000
192.168.121.2        <-> 224.0.0.9                  0 0 bytes         1 110 bytes       1 110 bytes  247148582.842512012         0.0000
192.168.121.40       <-> 78.46.107.140              0 0 bytes         1 94 bytes        1 94 bytes   247148590.574460000         0.0000
192.168.121.2        <-> 192.168.121.253            0 0 bytes         1 98 bytes        1 98 bytes   247148595.824292004         0.0000
192.168.7.1          <-> 224.0.0.1                  0 0 bytes         1 64 bytes        1 64 bytes   268487979.182283998         0.0000
192.168.7.12         <-> 224.0.0.251                0 0 bytes         1 64 bytes        1 64 bytes   268487982.182281017         0.0000
192.168.7.5          <-> 224.0.0.22                 0 0 bytes         1 64 bytes        1 64 bytes   268487988.981966019         0.0000
192.168.7.26         <-> 224.0.0.251                0 0 bytes         1 241 bytes       1 241 bytes  268488042.884328008         0.0000
192.168.7.5          <-> 224.0.0.251                0 0 bytes         1 332 bytes       1 332 bytes  268488042.884432971         0.0000
192.168.127.1        <-> 224.0.0.10                 0 0 bytes         1 118 bytes       1 118 bytes  277179164.641978979         0.0000
192.168.127.2        <-> 224.0.0.10                 0 0 bytes         1 136 bytes       1 136 bytes  277179164.642009020         0.0000
192.168.127.2        <-> 192.168.127.1              0 0 bytes         1 98 bytes        1 98 bytes   277179164.642012000         0.0000
193.24.227.238       <-> 172.217.40.76              0 0 bytes         1 1514 bytes       1 1514 bytes  317545551.899236977         0.0000
193.24.227.238       <-> 173.194.169.104            0 0 bytes         1 1514 bytes       1 1514 bytes  317545562.947239995         0.0000
193.24.227.238       <-> 74.125.47.136              0 0 bytes         1 1514 bytes       1 1514 bytes  317545564.904537022         0.0000
52.109.32.27         <-> 192.168.7.35               0 0 bytes         1 1518 bytes       1 1518 bytes  357620708.871252000         0.0000
193.24.225.56        <-> 193.24.225.56              0 0 bytes         1 64 bytes        1 64 bytes   361949126.379015028         0.0000
192.168.7.53         <-> 192.168.3.83               0 0 bytes         1 65 bytes        1 65 bytes   454399746.737631977         0.0000
192.168.7.53         <-> 255.255.255.255            0 0 bytes         1 144 bytes       1 144 bytes  454408044.615013003         0.0000
169.254.140.132      <-> 224.0.0.252                0 0 bytes         1 77 bytes        1 77 bytes   454546685.195813000         0.0000
10.0.1.97            <-> 239.255.255.250            0 0 bytes         1 702 bytes       1 702 bytes  454546699.681030989         0.0000
================================================================================
================================================================================
Ethernet Conversations
Filter:<No Filter>
                                               |       <-      | |       ->      | |     Total     |    Relative    |   Duration   |
                                               | Frames  Bytes | | Frames  Bytes | | Frames  Bytes |      Start     |              |
00:0c:29:c3:7f:eb    <-> 3c:fa:30:03:12:30          0 0 bytes        23 10 kB          23 10 kB      484543807.661850989      8383.5461
9c:2d:cd:3f:0e:c0    <-> 3c:fa:30:03:12:12          7 4432 bytes      11 3887 bytes      18 8319 bytes  484553085.140166998     81883.7232
c2:3c:19:6c:00:01    <-> c2:3d:19:6c:00:01          6 506 bytes       9 851 bytes      15 1357 bytes  113831508.720075995        69.9350
00:1e:7a:79:3f:11    <-> 00:14:69:9e:11:41          8 1647 bytes       3 384 bytes      11 2031 bytes  247148595.823276013       306.8262
00:0c:29:a9:e4:e3    <-> 3c:fa:30:03:12:30          0 0 bytes        11 4396 bytes      11 4396 bytes  484474739.245980978     77451.9616
00:00:00:00:00:00    <-> 00:00:00:00:00:00          0 0 bytes         9 1107 bytes       9 1107 bytes  150640362.682969004   5439385.3407
44:2b:03:19:03:44    <-> bc:05:43:cc:c2:a9          4 256 bytes       5 340 bytes       9 596 bytes  272033366.971790016         1.6921
3c:61:04:50:d2:1a    <-> c8:0e:14:7e:33:a0          6 2968 bytes       3 2707 bytes       9 5675 bytes  332407522.889362991    263303.4407
54:ee:75:ec:9a:f4    <-> a8:d0:e5:d4:fe:cb          6 3833 bytes       3 219 bytes       9 4052 bytes  356411927.059378028   4770800.5677
00:12:3f:0a:8a:96    <-> 00:19:e2:a1:f9:89          4 3900 bytes       4 3778 bytes       8 7678 bytes  218860580.149318010       101.8234
00:0c:29:c1:34:dc    <-> b4:0c:25:05:8e:13          4 1172 bytes       4 737 bytes       8 1909 bytes  283324490.367660999        26.2234
b4:0c:25:05:8e:10    <-> 08:5b:0e:3c:11:5d          2 484 bytes       4 968 bytes       6 1452 bytes  257139411.884148985        33.5521
d4:be:d9:4c:11:9e    <-> 00:86:9c:e7:55:14          5 5053 bytes       1 90 bytes        6 5143 bytes  335418362.555326998   2848165.9031
00:1e:7a:79:3f:10    <-> 00:25:45:60:17:c1          3 232 bytes       3 273 bytes       6 505 bytes  338277664.949613988      1392.0504
00:1e:7a:79:3f:10    <-> 00:15:62:6a:fe:f0          3 198 bytes       3 204 bytes       6 402 bytes  361949125.378430009         1.0006
1c:69:7a:0f:cc:5e    <-> 64:7c:e8:8a:79:12          2 206 bytes       4 374 bytes       6 580 bytes  454399727.184858978        19.5528
bc:24:11:d6:d0:8f    <-> bc:24:11:29:2b:31          2 128 bytes       4 264 bytes       6 392 bytes  538124601.101827025        58.8853
d4:21:22:76:5b:78    <-> 00:21:6a:2d:3b:8e          2 154 bytes       3 800 bytes       5 954 bytes  188826407.116876990        36.6516
00:0a:8a:a1:5a:9a    <-> 01:00:0c:cc:cc:cc          0 0 bytes         5 478 bytes       5 478 bytes  247148575.383350998        40.4848
00:0c:29:8a:5d:d7    <-> 00:86:9c:e7:55:14          0 0 bytes         5 7562 bytes       5 7562 bytes  317545551.899236977        13.0053
70:4c:a5:99:4a:b3    <-> 00:0c:29:5f:2c:a1          1 80 bytes        4 1946 bytes       5 2026 bytes  471763158.868748009      1178.6268
00:21:1b:ae:31:99    <-> 01:00:0c:cc:cc:cc          0 0 bytes         4 802 bytes       4 802 bytes  247148578.213800997        12.1761
00:13:95:24:34:04    <-> 70:4c:a5:99:4a:b3          2 120 bytes       2 138 bytes       4 258 bytes  427350545.701790988        65.9728
00:04:00:83:76:2c    <-> ff:ff:ff:ff:ff:ff          0 0 bytes         3 180 bytes       3 180 bytes     0.000000000       302.4866
00:0c:29:9d:c9:d6    <-> 00:19:e2:a1:f9:86          1 97 bytes        2 753 bytes       3 850 bytes  148881240.483381003        17.3097
00:21:1b:ae:31:99    <-> 01:80:c2:00:00:0e          0 0 bytes         3 1158 bytes       3 1158 bytes  247148574.892026991        59.8868
00:21:1b:ae:31:c1    <-> 00:00:0c:9f:f0:79          0 0 bytes         3 467 bytes       3 467 bytes  247148582.258426011       148.7814
c8:0e:14:7e:33:9f    <-> 01:00:5e:7f:ff:fa          0 0 bytes         3 507 bytes       3 507 bytes  268487977.681648016         9.9999
00:a0:de:de:54:13    <-> c8:0e:14:7e:33:9f          0 0 bytes         3 1447 bytes       3 1447 bytes  268488077.412208974         0.0780
bc:05:43:cc:c2:a9    <-> ff:ff:ff:ff:ff:ff          0 0 bytes         3 198 bytes       3 198 bytes  272033355.067547023         1.4911
00:10:db:ff:10:00    <-> 00:14:69:9e:11:40          1 138 bytes       2 276 bytes       3 414 bytes  315983834.940446973         1.0002
b8:27:eb:03:a0:ac    <-> 00:86:9c:e7:55:14          1 982 bytes       2 552 bytes       3 1534 bytes  343727186.438643992      4949.8977
b8:27:eb:03:a0:ac    <-> 00:21:70:b2:0e:6c          0 0 bytes         3 776 bytes       3 776 bytes  380094905.578687012         0.0075
00:00:0c:07:ac:14    <-> 01:00:5e:00:00:02          0 0 bytes         3 198 bytes       3 198 bytes  411633454.736446023         3.0268
00:0c:29:d5:b8:68    <-> 00:0c:29:af:1c:ec          1 169 bytes       2 460 bytes       3 629 bytes  416449302.153568983        30.0050
00:0c:29:b7:1d:68    <-> 00:0c:29:a8:26:f7          1 94 bytes        2 311 bytes       3 405 bytes  429602608.727746010         6.5904
00:e0:4c:68:66:c1    <-> ff:ff:ff:ff:ff:ff          0 0 bytes         3 342 bytes       3 342 bytes  454546686.485548973         0.0009
00:e0:4c:68:66:c1    <-> 00:86:9c:e7:55:14          0 0 bytes         3 318 bytes       3 318 bytes  454546689.669839978         0.7512
3c:fa:30:03:12:10    <-> 00:70:76:69:66:00          0 0 bytes         3 168 bytes       3 168 bytes  484633303.877973020        39.9998
1c:69:7a:0f:cc:5e    <-> 3c:fa:30:03:12:12          1 198 bytes       2 2524 bytes       3 2722 bytes  520244834.114021003         0.0153
c2:3d:19:6c:00:01    <-> 01:00:5e:00:00:02          0 0 bytes         2 152 bytes       2 152 bytes  113831506.536075994         0.0000
c2:3d:19:6c:00:01    <-> c2:3d:19:6c:00:01          0 0 bytes         2 120 bytes       2 120 bytes  113831508.735075995         0.0000
c2:3c:19:6c:00:01    <-> 01:00:5e:00:00:05          0 0 bytes         2 188 bytes       2 188 bytes  113831510.763076007         0.0000
c2:3c:19:6c:00:01    <-> 01:00:0c:cc:cc:cc          0 0 bytes         2 698 bytes       2 698 bytes  113831512.682076007         0.0000
00:21:6a:2d:3b:8e    <-> 33:33:00:00:00:16          0 0 bytes         2 180 bytes       2 180 bytes  188826404.676330000         0.5300
00:21:6a:2d:3b:8e    <-> ff:ff:ff:ff:ff:ff          0 0 bytes         2 684 bytes       2 684 bytes  188826405.316543013         2.0168
00:21:1b:ae:31:99    <-> 01:00:0c:cc:cc:cd          0 0 bytes         2 136 bytes       2 136 bytes  247148575.328359008         0.3827
00:1e:7a:79:3f:11    <-> 01:00:5e:00:00:09          0 0 bytes         2 240 bytes       2 240 bytes  247148575.370476991         7.4720
00:1e:7a:79:3f:11    <-> 33:33:00:00:00:09          0 0 bytes         2 260 bytes       2 260 bytes  247148581.466053009         0.0004
00:16:47:df:e7:c1    <-> 00:00:0c:9f:f0:79          0 0 bytes         2 188 bytes       2 188 bytes  247148588.577903003         1.9966
00:0a:8a:a1:5a:9a    <-> 01:80:c2:00:00:02          0 0 bytes         2 256 bytes       2 256 bytes  247148589.431784987        27.2847
00:1e:7a:79:3f:11    <-> 00:1a:6c:a1:2b:99          0 0 bytes         2 195 bytes       2 195 bytes  247148595.824292004       277.5263
00:a0:de:de:54:13    <-> 01:00:5e:7f:ff:fa          0 0 bytes         2 128 bytes       2 128 bytes  268487981.881812990       134.9305
b8:27:eb:ab:ae:c7    <-> 00:00:0c:9f:f1:c2          0 0 bytes         2 1557 bytes       2 1557 bytes  332801714.808215022         0.0176
c8:0e:14:7e:33:9f    <-> 00:b0:52:00:00:01          0 0 bytes         2 120 bytes       2 120 bytes  375864018.449222028         2.0013
00:13:95:24:34:04    <-> b8:27:eb:03:a0:ac          0 0 bytes         2 114 bytes       2 114 bytes  427350559.838343978        51.8378
3c:13:cc:ee:1f:09    <-> 00:1b:17:00:47:11          0 0 bytes         2 181 bytes       2 181 bytes  429500811.305504024         3.2048
00:e0:4c:68:66:c1    <-> 33:33:00:01:00:03          0 0 bytes         2 194 bytes       2 194 bytes  454546685.195254982         0.4228
00:e0:4c:68:66:c1    <-> 33:33:00:00:00:0c          0 0 bytes         2 1444 bytes       2 1444 bytes  454546699.680526972         0.1384
00:0c:29:36:86:34    <-> 3c:fa:30:03:12:30          0 0 bytes         2 606 bytes       2 606 bytes  484474739.244638026         0.0024
aa:c1:ab:1d:d3:cc    <-> 01:80:c2:00:00:03          0 0 bytes         2 352 bytes       2 352 bytes  532614529.410582006         2.0217
c2:3c:19:6c:00:01    <-> 01:00:5e:00:00:02          0 0 bytes         1 76 bytes        1 76 bytes   113831506.723076001         0.0000
c2:3d:19:6c:00:01    <-> 01:00:5e:00:00:05          0 0 bytes         1 94 bytes        1 94 bytes   113831511.980076000         0.0000
c2:3c:19:6c:00:01    <-> c2:3c:19:6c:00:01          0 0 bytes         1 60 bytes        1 60 bytes   113831515.974076003         0.0000
c2:3d:19:6c:00:01    <-> 01:00:0c:cc:cc:cc          0 0 bytes         1 349 bytes       1 349 bytes  113831563.913075998         0.0000
00:21:6a:2d:3b:8e    <-> 33:33:ff:2d:3b:8e          0 0 bytes         1 78 bytes        1 78 bytes   188826405.249666989         0.0000
00:21:6a:2d:3b:8e    <-> 33:33:00:01:00:02          0 0 bytes         1 129 bytes       1 129 bytes  188826407.076245010         0.0000
d4:21:22:76:5b:78    <-> 01:00:5e:00:00:01          0 0 bytes         1 50 bytes        1 50 bytes   188826445.856070012         0.0000
d4:21:22:76:5b:79    <-> 44:2b:03:19:03:44          0 0 bytes         1 163 bytes       1 163 bytes  189080689.225966990         0.0000
00:1a:6c:a1:2b:99    <-> 33:33:00:00:00:09          0 0 bytes         1 130 bytes       1 130 bytes  247148574.771515995         0.0000
00:0a:8a:a1:5a:9a    <-> 01:00:0c:cc:cc:cd          0 0 bytes         1 68 bytes        1 68 bytes   247148574.816390008         0.0000
00:1a:6c:a1:2b:99    <-> 33:33:00:00:00:66          0 0 bytes         1 138 bytes       1 138 bytes  247148575.183580011         0.0000
00:00:0c:9f:f0:79    <-> 01:00:5e:00:00:66          0 0 bytes         1 118 bytes       1 118 bytes  247148576.008451015         0.0000
00:1a:6c:a1:2b:99    <-> 01:00:5e:00:00:66          0 0 bytes         1 118 bytes       1 118 bytes  247148577.135630995         0.0000
00:1a:6c:a1:2b:99    <-> 01:00:5e:00:00:09          0 0 bytes         1 70 bytes        1 70 bytes   247148578.864152014         0.0000
00:14:69:9e:11:41    <-> 00:16:47:df:e7:c1          0 0 bytes         1 94 bytes        1 94 bytes   247148588.579910010         0.0000
00:21:1b:ae:31:99    <-> 01:80:c2:00:00:02          0 0 bytes         1 124 bytes       1 124 bytes  247148599.655371010         0.0000
00:14:69:9e:11:41    <-> ab:00:00:02:00:00          0 0 bytes         1 81 bytes        1 81 bytes   247148696.491160989         0.0000
00:1a:6c:a1:2b:99    <-> ab:00:00:02:00:00          0 0 bytes         1 81 bytes        1 81 bytes   247148829.642848015         0.0000
c8:0e:14:7e:33:9f    <-> 01:00:5e:00:00:01          0 0 bytes         1 64 bytes        1 64 bytes   268487979.182283998         0.0000
00:a0:de:de:54:13    <-> 01:00:5e:00:00:fb          0 0 bytes         1 64 bytes        1 64 bytes   268487982.182281017         0.0000
b8:27:eb:c9:16:37    <-> 01:00:5e:00:00:16          0 0 bytes         1 64 bytes        1 64 bytes   268487988.981966019         0.0000
74:81:14:81:c2:d4    <-> 01:00:5e:00:00:fb          0 0 bytes         1 241 bytes       1 241 bytes  268488042.884328008         0.0000
b8:27:eb:c9:16:37    <-> 01:00:5e:00:00:fb          0 0 bytes         1 332 bytes       1 332 bytes  268488042.884432971         0.0000
74:81:14:81:c2:d4    <-> 33:33:00:00:00:fb          0 0 bytes         1 261 bytes       1 261 bytes  268488042.884473979         0.0000
00:1a:6c:a1:2b:99    <-> 01:00:5e:00:00:0a          0 0 bytes         1 118 bytes       1 118 bytes  277179164.641978979         0.0000
00:14:69:9e:11:40    <-> 01:00:5e:00:00:0a          0 0 bytes         1 136 bytes       1 136 bytes  277179164.642009020         0.0000
00:14:69:9e:11:40    <-> 00:1a:6c:a1:2b:99          0 0 bytes         1 98 bytes        1 98 bytes   277179164.642012000         0.0000
08:5b:0e:a1:83:5e    <-> 00:0c:29:7c:a4:cb          0 0 bytes         1 1494 bytes       1 1494 bytes  319447449.110508978         0.0000
d8:67:d9:07:8e:c1    <-> b8:27:eb:ab:ae:c7          0 0 bytes         1 146 bytes       1 146 bytes  332801714.824316025         0.0000
90:03:25:74:4e:06    <-> cc:ce:1e:5b:c4:93          0 0 bytes         1 1518 bytes       1 1518 bytes  357620708.871252000         0.0000
00:1e:7a:79:3f:10    <-> ab:00:00:02:00:00          0 0 bytes         1 81 bytes        1 81 bytes   361949113.835988998         0.0000
c8:0e:14:7e:33:9f    <-> ff:ff:ff:ff:ff:ff          0 0 bytes         1 60 bytes        1 60 bytes   375864018.449320972         0.0000
00:00:0c:07:ac:fa    <-> 00:1b:17:00:23:11          0 0 bytes         1 82 bytes        1 82 bytes   429500811.308673024         0.0000
1c:69:7a:0f:cc:5e    <-> b8:27:eb:bc:cd:b4          0 0 bytes         1 116 bytes       1 116 bytes  454408036.187328994         0.0000
1c:69:7a:0f:cc:5e    <-> ff:ff:ff:ff:ff:ff          0 0 bytes         1 144 bytes       1 144 bytes  454408044.615013003         0.0000
00:e0:4c:68:66:c1    <-> 01:00:5e:00:00:fc          0 0 bytes         1 77 bytes        1 77 bytes   454546685.195813000         0.0000
00:e0:4c:68:66:c1    <-> 01:00:5e:7f:ff:fa          0 0 bytes         1 702 bytes       1 702 bytes  454546699.681030989         0.0000
00:0c:29:55:9b:4b    <-> 33:33:00:00:00:02          0 0 bytes         1 102 bytes       1 102 bytes  527525175.633853972         0.0000
00:0c:29:ef:7c:66    <-> 33:33:00:00:00:02          0 0 bytes         1 102 bytes       1 102 bytes  527525186.777831018         0.0000
00:0c:29:55:9b:4b    <-> 00:0c:29:ef:7c:66          0 0 bytes         1 130 bytes       1 130 bytes  527525210.807591021         0.0000
aa:c1:ab:6e:91:a9    <-> 01:80:c2:00:00:03          0 0 bytes         1 126 bytes       1 126 bytes  532614529.487984002         0.0000
================================================================================
```

## Endpoints

```text
================================================================================
TCP Endpoints
Filter:<No Filter>
                       |  Port  | | Packets | |  Bytes  | | Tx Packets | | Tx Bytes | | Rx Packets | | Rx Bytes |
2a00:6020:ad0b:8380::10        445           12   6959 bytes          3      1002 bytes           9      5957 bytes
2a00:6020:ad0b:8380::10      49667            7   2704 bytes          2      704 bytes            5      2000 bytes
10.200.200.202              179            6   445 bytes           4      291 bytes            2      154 bytes
10.200.200.201            23975            6   445 bytes           2      154 bytes            4      291 bytes
2a01:488:42:1000:50ed:8588:8a:c570        110            5   2026 bytes          4      1946 bytes           1      80 bytes
2a00:6020:ad0b:8381:fd70:e2f9:f031:9dc4      49984            5   1262 bytes          3      730 bytes            2      532 bytes
2a00:6020:ad0b:8380::10        135            5   1262 bytes          2      532 bytes            3      730 bytes
192.168.110.9             50477            4   4267 bytes          2      553 bytes            2      3714 bytes
80.154.108.235              443            4   4267 bytes          2      3714 bytes           2      553 bytes
80.154.108.237               25            4   3411 bytes          2      186 bytes            2      3225 bytes
192.168.110.9             45271            4   3411 bytes          2      3225 bytes           2      186 bytes
2003:51:6012:110::b15:22      60892            4   1192 bytes          3      1095 bytes           1      97 bytes
2003:51:6012:121::2          22            4   1192 bytes          1      97 bytes             3      1095 bytes
2a00:6020:ad0b:8381:fd70:e2f9:f031:9dc4      49958            4   1245 bytes          2      501 bytes            2      744 bytes
2a00:6020:ad0b:8381:fd70:e2f9:f031:9dc4      49985            4   1678 bytes          3      1396 bytes           1      282 bytes
192.168.7.1               51108            3   1447 bytes          0      0 bytes              3      1447 bytes
2003:de:2016:120::a08:53        143            3   374 bytes           2      274 bytes            1      100 bytes
2003:de:2016:125:fc36:8317:4e86:cb72       7552            3   374 bytes           1      100 bytes            2      274 bytes
141.41.241.70               443            3   1703 bytes          2      1557 bytes           1      146 bytes
141.41.39.187             40976            3   1703 bytes          1      146 bytes            2      1557 bytes
2001:470:765b::b15:22      52222            3   842 bytes           1      90 bytes             2      752 bytes
2001:48a8:6880::18           43            3   842 bytes           2      752 bytes            1      90 bytes
2001:db8::2               18716            3   321 bytes           2      211 bytes            1      110 bytes
2001:db8::1                  23            3   321 bytes           1      110 bytes            2      211 bytes
10.82.185.11              57895            3   197 bytes           2      137 bytes            1      60 bytes
192.168.11.10               515            3   197 bytes           1      60 bytes             2      137 bytes
5.35.226.136                 21            3   281 bytes           2      199 bytes            1      82 bytes
10.82.185.11              51072            3   281 bytes           1      82 bytes             2      199 bytes
2001:470:1f0b:16b0:6986:b8d4:3649:9cbe      55600            3   776 bytes           3      776 bytes            0      0 bytes
2001:470:1f0b:16b0:221:70ff:feb2:e6c        514            3   776 bytes           0      0 bytes              3      776 bytes
192.168.7.17              10051            3   629 bytes           1      169 bytes            2      460 bytes
192.168.0.1               39255            3   263 bytes           2      181 bytes            1      82 bytes
192.0.2.49                   49            3   263 bytes           1      82 bytes             2      181 bytes
2001:470:1f0b:16b0:f83f:53c1:be1:eca1      53955            3   346 bytes           1      80 bytes             2      266 bytes
172.16.80.10                 88            3   899 bytes           1      293 bytes            2      606 bytes
2a00:6020:ad0b:8381:fd70:e2f9:f031:9dc4      49961            3   2085 bytes          2      586 bytes            1      1499 bytes
2a00:6020:ad0b:8380::10        389            3   2085 bytes          1      1499 bytes           2      586 bytes
2a00:6020:ad0b:8381:fd70:e2f9:f031:9dc4      49975            3   3187 bytes          3      3187 bytes           0      0 bytes
2a00:6020:ad0b:8381:fd70:e2f9:f031:9dc4      49998            3   722 bytes           2      608 bytes            1      114 bytes
2a00:6020:ad0b:8380::10      61737            3   722 bytes           1      114 bytes            2      608 bytes
2a00:6020:ad0b:8381:fd70:e2f9:f031:9dc4      49999            3   1026 bytes          2      604 bytes            1      422 bytes
2a00:6020:ad0b:8381:15f6:aeec:61d9:205f      49679            3   858 bytes           2      600 bytes            1      258 bytes
2a00:6020:ad0b:8380::10       3389            3   335 bytes           1      93 bytes             2      242 bytes
192.168.222.129           45286            3   232 bytes           2      156 bytes            1      76 bytes
192.168.222.131             502            3   232 bytes           1      76 bytes             2      156 bytes
192.168.110.10             1152            2   462 bytes           1      365 bytes            1      97 bytes
80.237.133.136               80            2   462 bytes           1      97 bytes             1      365 bytes
2003:de:2016:125:fc36:8317:4e86:cb72       7549            2   1050 bytes          1      271 bytes            1      779 bytes
2003:de:2016:110::a12:443        443            2   1050 bytes          1      779 bytes            1      271 bytes
2003:de:2016:110::b15:22         22            2   225 bytes           1      119 bytes            1      106 bytes
2003:de:2016:125:fc36:8317:4e86:cb72       7563            2   225 bytes           1      106 bytes            1      119 bytes
2001:470:765b:0:1c6e:18ae:ddb4:3bc1      55996            2   1258 bytes          1      276 bytes            1      982 bytes
2a02:26f0:6c00::210:ba61         80            2   1258 bytes          1      982 bytes            1      276 bytes
5.35.226.136              51652            2   2068 bytes          2      2068 bytes           0      0 bytes
10.82.185.11              51075            2   2068 bytes          0      0 bytes              2      2068 bytes
192.168.7.16              49404            2   468 bytes           1      299 bytes            1      169 bytes
192.168.3.53              52520            2   556 bytes           1      263 bytes            1      293 bytes
2a00:6020:ad0b:8321:34fb:d0ff:9ee8:abf2       5403            2   214 bytes           1      121 bytes            1      93 bytes
192.168.110.10             1154            1   388 bytes           1      388 bytes            0      0 bytes
212.144.254.123            3128            1   388 bytes           0      0 bytes              1      388 bytes
192.168.7.12               1226            1   561 bytes           1      561 bytes            0      0 bytes
192.168.7.12               1227            1   108 bytes           1      108 bytes            0      0 bytes
192.168.7.12               1228            1   778 bytes           1      778 bytes            0      0 bytes
2003:de:2016:125:fc36:8317:4e86:cb72       7562            1   260 bytes           1      260 bytes            0      0 bytes
2003:de:2016:120::a08:53         25            1   260 bytes           0      0 bytes              1      260 bytes
2001:470:765b:0:1c6e:18ae:ddb4:3bc1      35638            1   276 bytes           1      276 bytes            0      0 bytes
2a02:26f0:6c00::210:ba60         80            1   276 bytes           0      0 bytes              1      276 bytes
52.109.32.27                443            1   1518 bytes          1      1518 bytes           0      0 bytes
192.168.7.35              63594            1   1518 bytes          0      0 bytes              1      1518 bytes
5.35.226.136              52833            1   1506 bytes          1      1506 bytes           0      0 bytes
10.82.185.11              51076            1   1506 bytes          0      0 bytes              1      1506 bytes
192.168.7.16              49406            1   161 bytes           1      161 bytes            0      0 bytes
194.247.5.12                 13            1   80 bytes            1      80 bytes             0      0 bytes
85.215.94.29              43624            1   80 bytes            0      0 bytes              1      80 bytes
194.247.5.12                 37            1   58 bytes            1      58 bytes             0      0 bytes
85.215.94.29              49510            1   58 bytes            0      0 bytes              1      58 bytes
2001:470:1f0b:16b0:f83f:53c1:be1:eca1      53958            1   501 bytes           0      0 bytes              1      501 bytes
2001:470:1f0b:16b0:f83f:53c1:be1:eca1      54004            1   1179 bytes          0      0 bytes              1      1179 bytes
192.168.3.53              52524            1   343 bytes           1      343 bytes            0      0 bytes
2a00:6020:ad0b:8381:fd70:e2f9:f031:9dc4      49963            1   151 bytes           1      151 bytes            0      0 bytes
2a00:6020:ad0b:8381:c8c2:18f1:4b9d:7524      49671            1   1518 bytes          1      1518 bytes           0      0 bytes
2a00:6020:ad0b:8321:34fb:d0ff:9ee8:abf2       5409            1   121 bytes           1      121 bytes            0      0 bytes
================================================================================
================================================================================
IPv4 Endpoints
Filter:<No Filter>
                       | Packets | |  Bytes  | | Tx Packets | | Tx Bytes | | Rx Packets | | Rx Bytes |
127.0.0.1                      18   2214 bytes          9      1107 bytes           9      1107 bytes
192.168.21.41                  10   2178 bytes          7      1282 bytes           3      896 bytes
10.200.200.202                  9   835 bytes           6      563 bytes            3      272 bytes
10.200.200.201                  9   835 bytes           3      272 bytes            6      563 bytes
84.146.135.221                  9   5675 bytes          6      2968 bytes           3      2707 bytes
10.82.185.11                    9   4052 bytes          3      219 bytes            6      3833 bytes
192.168.110.9                   8   7678 bytes          4      3778 bytes           4      3900 bytes
172.16.23.2                     8   629 bytes           4      335 bytes            4      294 bytes
192.168.47.1                    8   629 bytes           4      294 bytes            4      335 bytes
192.168.11.1                    8   2006 bytes          2      786 bytes            6      1220 bytes
192.168.121.2                   7   592 bytes           4      394 bytes            3      198 bytes
192.168.7.1                     7   2018 bytes          4      571 bytes            3      1447 bytes
193.24.225.54                   7   466 bytes           4      268 bytes            3      198 bytes
10.0.0.1                        6   588 bytes           4      360 bytes            2      228 bytes
224.0.0.2                       6   426 bytes           0      0 bytes              6      426 bytes
10.0.0.2                        6   606 bytes           5      492 bytes            1      114 bytes
192.168.110.10                  6   1066 bytes          4      881 bytes            2      185 bytes
239.255.255.250                 6   1337 bytes          0      0 bytes              6      1337 bytes
192.168.7.12                    6   1639 bytes          6      1639 bytes           0      0 bytes
217.0.21.65                     6   4997 bytes          3      2707 bytes           3      2290 bytes
5.35.226.136                    6   3855 bytes          5      3773 bytes           1      82 bytes
194.247.5.12                    6   372 bytes           4      252 bytes            2      120 bytes
85.215.94.29                    6   372 bytes           2      120 bytes            4      252 bytes
192.168.222.129                 6   392 bytes           4      264 bytes            2      128 bytes
192.168.222.131                 6   392 bytes           2      128 bytes            4      264 bytes
192.168.2.1                     5   878 bytes           3      724 bytes            2      154 bytes
193.24.225.56                   5   338 bytes           2      134 bytes            3      204 bytes
192.168.2.102                   4   828 bytes           2      154 bytes            2      674 bytes
80.154.108.235                  4   4267 bytes          2      3714 bytes           2      553 bytes
80.154.108.237                  4   3411 bytes          2      186 bytes            2      3225 bytes
169.254.140.132                 4   419 bytes           4      419 bytes            0      0 bytes
10.0.1.97                       4   1020 bytes          4      1020 bytes           0      0 bytes
224.0.0.5                       3   282 bytes           0      0 bytes              3      282 bytes
255.255.255.255                 3   828 bytes           0      0 bytes              3      828 bytes
224.0.0.9                       3   310 bytes           0      0 bytes              3      310 bytes
192.168.121.254                 3   286 bytes           2      188 bytes            1      98 bytes
192.168.121.253                 3   286 bytes           2      188 bytes            1      98 bytes
192.168.121.10                  3   467 bytes           3      467 bytes            0      0 bytes
192.168.120.10                  3   467 bytes           0      0 bytes              3      467 bytes
192.168.121.40                  3   282 bytes           2      188 bytes            1      94 bytes
224.0.0.251                     3   637 bytes           0      0 bytes              3      637 bytes
216.66.80.30                    3   414 bytes           2      276 bytes            1      138 bytes
193.24.227.12                   3   414 bytes           1      138 bytes            2      276 bytes
193.24.227.238                  3   4542 bytes          3      4542 bytes           0      0 bytes
217.0.5.215                     3   678 bytes           0      0 bytes              3      678 bytes
141.41.241.70                   3   1703 bytes          2      1557 bytes           1      146 bytes
141.41.39.187                   3   1703 bytes          1      146 bytes            2      1557 bytes
192.168.11.10                   3   197 bytes           1      60 bytes             2      137 bytes
192.168.20.2                    3   198 bytes           3      198 bytes            0      0 bytes
192.168.7.16                    3   629 bytes           2      460 bytes            1      169 bytes
192.168.7.17                    3   629 bytes           1      169 bytes            2      460 bytes
192.168.0.1                     3   263 bytes           2      181 bytes            1      82 bytes
192.0.2.49                      3   263 bytes           1      82 bytes             2      181 bytes
169.254.255.255                 3   342 bytes           0      0 bytes              3      342 bytes
10.0.1.1                        3   318 bytes           0      0 bytes              3      318 bytes
192.168.3.53                    3   899 bytes           2      606 bytes            1      293 bytes
172.16.80.10                    3   899 bytes           1      293 bytes            2      606 bytes
100.93.7.250                    3   168 bytes           3      168 bytes            0      0 bytes
194.247.4.10                    3   168 bytes           0      0 bytes              3      168 bytes
80.237.133.136                  2   462 bytes           1      97 bytes             1      365 bytes
0.0.0.0                         2   684 bytes           2      684 bytes            0      0 bytes
224.0.0.1                       2   114 bytes           0      0 bytes              2      114 bytes
224.0.0.102                     2   236 bytes           0      0 bytes              2      236 bytes
212.224.120.164                 2   188 bytes           1      94 bytes             1      94 bytes
192.168.7.5                     2   396 bytes           2      396 bytes            0      0 bytes
192.168.127.1                   2   216 bytes           1      118 bytes            1      98 bytes
224.0.0.10                      2   254 bytes           0      0 bytes              2      254 bytes
192.168.127.2                   2   234 bytes           2      234 bytes            0      0 bytes
192.168.7.53                    2   209 bytes           2      209 bytes            0      0 bytes
52.59.186.27                    2   172 bytes           1      110 bytes            1      62 bytes
212.144.254.123                 1   388 bytes           0      0 bytes              1      388 bytes
192.168.10.1                    1   130 bytes           1      130 bytes            0      0 bytes
78.46.107.140                   1   94 bytes            0      0 bytes              1      94 bytes
224.0.0.22                      1   64 bytes            0      0 bytes              1      64 bytes
192.168.7.26                    1   241 bytes           1      241 bytes            0      0 bytes
172.217.40.76                   1   1514 bytes          0      0 bytes              1      1514 bytes
173.194.169.104                 1   1514 bytes          0      0 bytes              1      1514 bytes
74.125.47.136                   1   1514 bytes          0      0 bytes              1      1514 bytes
52.109.32.27                    1   1518 bytes          1      1518 bytes           0      0 bytes
192.168.7.35                    1   1518 bytes          0      0 bytes              1      1518 bytes
192.168.3.83                    1   65 bytes            0      0 bytes              1      65 bytes
224.0.0.252                     1   77 bytes            0      0 bytes              1      77 bytes
================================================================================
================================================================================
Ethernet Endpoints
Filter:<No Filter>
                       | Packets | |  Bytes  | | Tx Packets | | Tx Bytes | | Rx Packets | | Rx Bytes |
3c:fa:30:03:12:30              36   15 kB               0      0 bytes             36      15 kB
c2:3d:19:6c:00:01              23   2192 bytes         12      1221 bytes          11      971 bytes
00:0c:29:c3:7f:eb              23   10 kB              23      10 kB                0      0 bytes
c2:3c:19:6c:00:01              22   2439 bytes         15      1873 bytes           7      566 bytes
3c:fa:30:03:12:12              21   11 kB               8      4630 bytes          13      6411 bytes
00:00:00:00:00:00              18   2214 bytes          9      1107 bytes           9      1107 bytes
9c:2d:cd:3f:0e:c0              18   8319 bytes         11      3887 bytes           7      4432 bytes
00:1e:7a:79:3f:11              17   2726 bytes          9      1079 bytes           8      1647 bytes
00:86:9c:e7:55:14              17   14 kB               6      6035 bytes          11      8522 bytes
ff:ff:ff:ff:ff:ff              13   1608 bytes          0      0 bytes             13      1608 bytes
00:14:69:9e:11:41              13   2206 bytes         10      1822 bytes           3      384 bytes
00:1e:7a:79:3f:10              13   988 bytes           7      558 bytes            6      430 bytes
01:00:0c:cc:cc:cc              12   2327 bytes          0      0 bytes             12      2327 bytes
bc:05:43:cc:c2:a9              12   794 bytes           7      454 bytes            5      340 bytes
00:e0:4c:68:66:c1              12   3077 bytes         12      3077 bytes           0      0 bytes
00:21:6a:2d:3b:8e              11   2025 bytes          8      1225 bytes           3      800 bytes
1c:69:7a:0f:cc:5e              11   3562 bytes          8      3158 bytes           3      404 bytes
00:0c:29:a9:e4:e3              11   4396 bytes         11      4396 bytes           0      0 bytes
44:2b:03:19:03:44              10   759 bytes           5      340 bytes            5      419 bytes
00:21:1b:ae:31:99              10   2220 bytes         10      2220 bytes           0      0 bytes
c8:0e:14:7e:33:9f              10   2198 bytes          7      751 bytes            3      1447 bytes
00:1a:6c:a1:2b:99               9   948 bytes           6      655 bytes            3      293 bytes
3c:61:04:50:d2:1a               9   5675 bytes          3      2707 bytes           6      2968 bytes
c8:0e:14:7e:33:a0               9   5675 bytes          6      2968 bytes           3      2707 bytes
54:ee:75:ec:9a:f4               9   4052 bytes          3      219 bytes            6      3833 bytes
a8:d0:e5:d4:fe:cb               9   4052 bytes          6      3833 bytes           3      219 bytes
70:4c:a5:99:4a:b3               9   2284 bytes          6      2066 bytes           3      218 bytes
00:12:3f:0a:8a:96               8   7678 bytes          4      3778 bytes           4      3900 bytes
00:19:e2:a1:f9:89               8   7678 bytes          4      3900 bytes           4      3778 bytes
00:0a:8a:a1:5a:9a               8   802 bytes           8      802 bytes            0      0 bytes
00:0c:29:c1:34:dc               8   1909 bytes          4      737 bytes            4      1172 bytes
b4:0c:25:05:8e:13               8   1909 bytes          4      1172 bytes           4      737 bytes
b8:27:eb:03:a0:ac               8   2424 bytes          5      1328 bytes           3      1096 bytes
01:00:5e:00:00:02               6   426 bytes           0      0 bytes              6      426 bytes
d4:21:22:76:5b:78               6   1004 bytes          4      850 bytes            2      154 bytes
00:00:0c:9f:f0:79               6   773 bytes           1      118 bytes            5      655 bytes
b4:0c:25:05:8e:10               6   1452 bytes          4      968 bytes            2      484 bytes
08:5b:0e:3c:11:5d               6   1452 bytes          2      484 bytes            4      968 bytes
01:00:5e:7f:ff:fa               6   1337 bytes          0      0 bytes              6      1337 bytes
00:a0:de:de:54:13               6   1639 bytes          6      1639 bytes           0      0 bytes
d4:be:d9:4c:11:9e               6   5143 bytes          1      90 bytes             5      5053 bytes
00:25:45:60:17:c1               6   505 bytes           3      232 bytes            3      273 bytes
00:15:62:6a:fe:f0               6   402 bytes           3      198 bytes            3      204 bytes
00:13:95:24:34:04               6   372 bytes           4      252 bytes            2      120 bytes
64:7c:e8:8a:79:12               6   580 bytes           2      206 bytes            4      374 bytes
bc:24:11:d6:d0:8f               6   392 bytes           4      264 bytes            2      128 bytes
bc:24:11:29:2b:31               6   392 bytes           2      128 bytes            4      264 bytes
00:14:69:9e:11:40               5   648 bytes           3      372 bytes            2      276 bytes
00:0c:29:8a:5d:d7               5   7562 bytes          5      7562 bytes           0      0 bytes
00:0c:29:5f:2c:a1               5   2026 bytes          1      80 bytes             4      1946 bytes
00:04:00:83:76:2c               3   180 bytes           3      180 bytes            0      0 bytes
01:00:5e:00:00:05               3   282 bytes           0      0 bytes              3      282 bytes
00:0c:29:9d:c9:d6               3   850 bytes           2      753 bytes            1      97 bytes
00:19:e2:a1:f9:86               3   850 bytes           1      97 bytes             2      753 bytes
33:33:00:00:00:09               3   390 bytes           0      0 bytes              3      390 bytes
01:00:0c:cc:cc:cd               3   204 bytes           0      0 bytes              3      204 bytes
01:80:c2:00:00:0e               3   1158 bytes          0      0 bytes              3      1158 bytes
01:00:5e:00:00:09               3   310 bytes           0      0 bytes              3      310 bytes
00:21:1b:ae:31:c1               3   467 bytes           3      467 bytes            0      0 bytes
00:16:47:df:e7:c1               3   282 bytes           2      188 bytes            1      94 bytes
01:80:c2:00:00:02               3   380 bytes           0      0 bytes              3      380 bytes
ab:00:00:02:00:00               3   243 bytes           0      0 bytes              3      243 bytes
01:00:5e:00:00:fb               3   637 bytes           0      0 bytes              3      637 bytes
00:10:db:ff:10:00               3   414 bytes           2      276 bytes            1      138 bytes
b8:27:eb:ab:ae:c7               3   1703 bytes          2      1557 bytes           1      146 bytes
00:21:70:b2:0e:6c               3   776 bytes           0      0 bytes              3      776 bytes
00:00:0c:07:ac:14               3   198 bytes           3      198 bytes            0      0 bytes
00:0c:29:d5:b8:68               3   629 bytes           2      460 bytes            1      169 bytes
00:0c:29:af:1c:ec               3   629 bytes           1      169 bytes            2      460 bytes
00:0c:29:b7:1d:68               3   405 bytes           2      311 bytes            1      94 bytes
00:0c:29:a8:26:f7               3   405 bytes           1      94 bytes             2      311 bytes
3c:fa:30:03:12:10               3   168 bytes           3      168 bytes            0      0 bytes
00:70:76:69:66:00               3   168 bytes           0      0 bytes              3      168 bytes
01:80:c2:00:00:03               3   478 bytes           0      0 bytes              3      478 bytes
33:33:00:00:00:16               2   180 bytes           0      0 bytes              2      180 bytes
01:00:5e:00:00:01               2   114 bytes           0      0 bytes              2      114 bytes
01:00:5e:00:00:66               2   236 bytes           0      0 bytes              2      236 bytes
b8:27:eb:c9:16:37               2   396 bytes           2      396 bytes            0      0 bytes
74:81:14:81:c2:d4               2   502 bytes           2      502 bytes            0      0 bytes
01:00:5e:00:00:0a               2   254 bytes           0      0 bytes              2      254 bytes
00:00:0c:9f:f1:c2               2   1557 bytes          0      0 bytes              2      1557 bytes
00:b0:52:00:00:01               2   120 bytes           0      0 bytes              2      120 bytes
3c:13:cc:ee:1f:09               2   181 bytes           2      181 bytes            0      0 bytes
00:1b:17:00:47:11               2   181 bytes           0      0 bytes              2      181 bytes
33:33:00:01:00:03               2   194 bytes           0      0 bytes              2      194 bytes
33:33:00:00:00:0c               2   1444 bytes          0      0 bytes              2      1444 bytes
00:0c:29:36:86:34               2   606 bytes           2      606 bytes            0      0 bytes
00:0c:29:55:9b:4b               2   232 bytes           2      232 bytes            0      0 bytes
33:33:00:00:00:02               2   204 bytes           0      0 bytes              2      204 bytes
00:0c:29:ef:7c:66               2   232 bytes           1      102 bytes            1      130 bytes
aa:c1:ab:1d:d3:cc               2   352 bytes           2      352 bytes            0      0 bytes
33:33:ff:2d:3b:8e               1   78 bytes            0      0 bytes              1      78 bytes
33:33:00:01:00:02               1   129 bytes           0      0 bytes              1      129 bytes
d4:21:22:76:5b:79               1   163 bytes           1      163 bytes            0      0 bytes
33:33:00:00:00:66               1   138 bytes           0      0 bytes              1      138 bytes
01:00:5e:00:00:16               1   64 bytes            0      0 bytes              1      64 bytes
33:33:00:00:00:fb               1   261 bytes           0      0 bytes              1      261 bytes
08:5b:0e:a1:83:5e               1   1494 bytes          1      1494 bytes           0      0 bytes
00:0c:29:7c:a4:cb               1   1494 bytes          0      0 bytes              1      1494 bytes
d8:67:d9:07:8e:c1               1   146 bytes           1      146 bytes            0      0 bytes
90:03:25:74:4e:06               1   1518 bytes          1      1518 bytes           0      0 bytes
cc:ce:1e:5b:c4:93               1   1518 bytes          0      0 bytes              1      1518 bytes
00:00:0c:07:ac:fa               1   82 bytes            1      82 bytes             0      0 bytes
00:1b:17:00:23:11               1   82 bytes            0      0 bytes              1      82 bytes
b8:27:eb:bc:cd:b4               1   116 bytes           0      0 bytes              1      116 bytes
01:00:5e:00:00:fc               1   77 bytes            0      0 bytes              1      77 bytes
aa:c1:ab:6e:91:a9               1   126 bytes           1      126 bytes            0      0 bytes
================================================================================
```

## TLS records

```text
frame.number|ip.src|tcp.srcport|ip.dst|tcp.dstport|tls.record.content_type|tls.record.version|tls.record.length|_ws.col.info
"56"|"192.168.110.9"|"50477"|"80.154.108.235"|"443"|"22"|"0x0301"|"290"|"Client Hello"
"57"|"80.154.108.235"|"443"|"192.168.110.9"|"50477"|"22"|"0x0303"|"66"|"Server Hello"
"58"|"80.154.108.235"|"443"|"192.168.110.9"|"50477"|"22,22,22"|"0x0303,0x0303,0x0303"|"3159,333,4"|"Certificate, Server Key Exchange, Server Hello Done"
"59"|"192.168.110.9"|"50477"|"80.154.108.235"|"443"|"22,20,22"|"0x0303,0x0303,0x0303"|"70,1,40"|"Client Key Exchange, Change Cipher Spec, Encrypted Handshake Message"
"147"||"7549"||"443"|"22"|"0x0301"|"188"|"Client Hello (SNI=ip.webernetz.net)"
"148"||"443"||"7549"||||"Continuation Data"
"173"|"141.41.241.70"|"443"|"141.41.39.187"|"40976"|"22,20"|"0x0303,0x0303,0x0303,0x0303,0x0303,0x0303"|"122,1,37,561,95,69"|"Server Hello, Change Cipher Spec, Application Data, Application Data, Application Data, Application Data"
"174"|"141.41.39.187"|"40976"|"141.41.241.70"|"443"|"20"|"0x0303,0x0303"|"1,69"|"Change Cipher Spec, Application Data"
"175"|"141.41.241.70"|"443"|"141.41.39.187"|"40976"||"0x0303,0x0303"|"250,250"|"Application Data, Application Data"
"194"|"52.109.32.27"|"443"|"192.168.7.35"|"63594"||||""
"314"||||||||"Initial, DCID=9ab19cb12ffe2349, SCID=b7ee840d86bde7ab, PKN: 0, CRYPTO"
```

## Expert information

```text

Errors (3)
=============
   Frequency      Group           Protocol  Summary
           2  Malformed        IEEE 802.11  Malformed Packet (Exception occurred)
           1   Protocol             DHCPv6  This message type is not permitted to use OPTION_CLIENT_FQDN

Warns (9)
=============
   Frequency      Group           Protocol  Summary
           6   Sequence                TCP  Previous segment(s) not captured (common at capture start)
           3   Sequence                TCP  ACKed segment that wasn't captured (common at capture start)

Notes (44)
=============
   Frequency      Group           Protocol  Summary
           2   Sequence                TCP  This frame is a (suspected) retransmission
           2   Sequence                TCP  Ambiguous ACK following Karn's definition
          12   Protocol                TCP  This packet's length exceeds MSS (common with TSO or incomplete conversations)
           5   Sequence               IPv4  "Time To Live" != 1 for a packet sent to the Local Network Control Block (see RFC 3171)
           2   Sequence               IPv4  "Time To Live" != 255 for a packet sent to the Local Network Control Block (see RFC 3171)
           3   Protocol             Syslog  Message conforms to neither RFC 5424 nor RFC 3164; trailing data appended
           1   Sequence               IPv4  "Time To Live" only 2
           2   Sequence                TCP  This frame initiates the connection closing
           9  Undecoded             DCERPC  No bind info for interface Context ID 1 - capture start too late?
           3  Undecoded             DCERPC  No bind info for interface Context ID 3 - capture start too late?
           2   Protocol               QUIC  (Random) padding data appended to the datagram
           1   Sequence               QUIC  This QUIC frame has a reused stream offset (retransmission?)

Chats (4)
=============
   Frequency      Group           Protocol  Summary
           2 Deprecated                TLS  This legacy_version field MUST be ignored. The supported_versions extension is present and MUST be used instead.
           2   Sequence                TCP  Connection finish (FIN)
```
