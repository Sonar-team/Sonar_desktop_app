# Fuzzing PCAP de `sonar-flows-core`

La cible `pcap_reader` injecte des octets arbitraires dans la même chaîne que
le desktop : ouverture libpcap, validation du DLT, lecture exhaustive,
`packet_parser` et agrégation dans `FlowMatrix`. Une erreur de fichier est un
résultat normal ; un panic, un crash ou une erreur de sanitizer est un défaut.

Le corpus initial est dérivé des petites captures et de la frontière DLT du
corpus TShark de #168. Le script ajoute aussi des variantes tronquées pour le
corpus adversarial de #151 :

```shell
script/pcap/prepare-fuzz-corpus.sh
cd sonar-rust/crates/sonar-flows-core
cargo +nightly fuzz run pcap_reader -- -max_len=262144
```

`cargo-fuzz` et un compilateur C++ compatible C++11 doivent être installés.
Le corpus généré et les artefacts de crash restent ignorés ; seules les seeds
minimisées qui reproduisent une régression doivent être promues dans les
fixtures versionnées.
