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

La cible `matrix_reader` fuzze le lecteur de matrice SFMS (préambule
percent-encodé, lignes CSV validées champ par champ) ; son corpus est dérivé
des fixtures versionnées par le même script.

`cargo-fuzz` et un compilateur C++ compatible C++11 doivent être installés.
Le corpus généré et les artefacts de crash restent ignorés ; seules les seeds
qui reproduisent une régression sont promues, dans `regressions/<cible>/`
(versionné, recopié dans le corpus par le script). Première entrée :
`timestamp_overflow_negative_tv_sec.pcap`, un timestamp pcapng négatif qui
faisait paniquer `timeval_to_systemtime` (trouvé par `pcap_reader`, corrigé
dans sonar-flows-core 0.8.0).

La CI exécute les deux cibles avec un budget borné à chaque PR (job
`fuzz_smoke` de rust-ci) ; un crash uploade son artefact.
