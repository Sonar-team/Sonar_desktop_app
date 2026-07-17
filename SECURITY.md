# Politique de sécurité

## Versions prises en charge

Les correctifs de sécurité sont fournis pour la dernière version stable publiée de chaque
composant maintenu dans ce dépôt.

| Cible                                                              | Prise en charge                                          |
| ------------------------------------------------------------------ | -------------------------------------------------------- |
| Dernière version stable de SONAR Desktop App                       | Oui                                                      |
| Dernière version stable de `sonar-flows-core` et `sonar-flows-cli` | Oui                                                      |
| Branche `main` et préversions                                      | Signalements acceptés, sans garantie de stabilité        |
| Versions stables antérieures                                       | Non, sauf indication contraire dans les notes de version |

Lorsqu'une nouvelle version stable est publiée, la précédente cesse donc d'être prise en charge.
Une vulnérabilité découverte sur une ancienne version reste pertinente si elle est également
reproductible sur la version prise en charge. Les versions disponibles sont publiées dans les
[releases GitHub](https://github.com/Sonar-team/Sonar_desktop_app/releases).

## Signaler une vulnérabilité

N'ouvrez pas d'issue, de discussion ou de pull request publique pour une vulnérabilité non
corrigée.

Utilisez le
[signalement privé de vulnérabilité GitHub](https://github.com/Sonar-team/Sonar_desktop_app/security/advisories/new).
Ce canal permet d'échanger avec les mainteneurs sans exposer prématurément les détails du
problème. Les signalements en français ou en anglais sont acceptés.

Les bugs sans conséquence de sécurité peuvent être signalés dans les
[issues publiques](https://github.com/Sonar-team/Sonar_desktop_app/issues).

## Informations à fournir

Dans la mesure du possible, indiquez :

- un résumé de la vulnérabilité et de son impact potentiel ;
- le composant, la version ou le commit, le système d'exploitation et l'architecture concernés ;
- la provenance de l'installation : artefact officiel, paquet, crate ou compilation locale ;
- les prérequis, les privilèges nécessaires et les interactions utilisateur éventuelles ;
- des étapes de reproduction minimales et reproductibles ;
- une preuve de concept limitée au strict nécessaire ;
- les journaux et messages d'erreur pertinents, après suppression des données sensibles ;
- une mesure d'atténuation ou une proposition de correction, si vous en avez une ;
- vos contraintes de divulgation et votre préférence concernant un éventuel crédit public.

Les captures réseau, matrices et journaux peuvent révéler des adresses IP ou MAC, des noms
d'hôtes, des chemins locaux, des filtres de capture et d'autres informations sensibles. Utilisez
de préférence un fichier PCAP, PCAPNG ou CSV synthétique et minimal. Ne transmettez jamais de
capture réelle, de secret, de jeton, de clé, de donnée personnelle ou de trafic tiers sans accord
préalable. Anonymisez également les journaux et les chemins de fichiers.

## Périmètre

Sont notamment dans le périmètre :

- le frontend Vue et le backend Rust de SONAR Desktop App ;
- les commandes IPC, les capacités Tauri, les accès aux fichiers et les appels système ;
- la capture réseau, les filtres BPF et la gestion des privilèges de capture ;
- le traitement de fichiers PCAP, PCAPNG et CSV non fiables ;
- `sonar-flows-core` et `sonar-flows-cli` ;
- l'exposition, l'altération ou la perte inattendue de données traitées par SONAR ;
- les installateurs, artefacts officiels, scripts de publication, signatures, attestations,
  nomenclatures logicielles (SBOM) et autres éléments de la chaîne de construction maintenus dans
  ce dépôt ;
- une dépendance tierce lorsqu'elle est exploitable dans le contexte de SONAR.

Ne sont généralement pas dans le périmètre :

- les bugs sans impact démontrable sur la confidentialité, l'intégrité ou la disponibilité ;
- les versions qui ne sont plus prises en charge, sauf si la version courante reste affectée ;
- les builds non officiels et les forks modifiés ;
- les vulnérabilités génériques du système d'exploitation, de GitHub, de Npcap, de libpcap ou
  d'une dépendance tierce sans impact propre à SONAR ;
- les résultats bruts d'un scanner sans analyse de l'exploitabilité ni de l'impact ;
- les attaques nécessitant déjà un contrôle administrateur complet du poste et n'apportant aucun
  impact supplémentaire ;
- l'ingénierie sociale, le hameçonnage, les attaques physiques et les attaques contre des systèmes
  tiers.

Une faiblesse amont exploitable par SONAR peut néanmoins être signalée en privé afin de nous
permettre d'évaluer l'exposition et de coordonner sa transmission au projet concerné.

## Recherche responsable

Lors de vos tests :

- utilisez uniquement des systèmes, comptes, réseaux et données qui vous appartiennent ou pour
  lesquels vous avez une autorisation explicite ;
- limitez l'accès aux données et l'exploitation au minimum nécessaire pour démontrer le problème ;
- n'interrompez aucun service, et ne détruisez ni ne modifiez de données ;
- n'installez aucun mécanisme de persistance ;
- ne réalisez pas d'exfiltration, d'ingénierie sociale ou de déni de service ;
- arrêtez vos tests si vous accédez de manière inattendue à des données tierces ;
- conservez les détails techniques dans le signalement privé jusqu'à la divulgation coordonnée.

## Traitement et divulgation coordonnée

Les délais ci-dessous sont des objectifs indicatifs, et non des engagements contractuels :

- accusé de réception sous cinq jours ouvrés ;
- première qualification sous dix jours ouvrés ;
- point d'avancement au moins tous les quatorze jours tant que le signalement reste ouvert.

Le délai de correction dépend de la gravité, de la complexité, des plateformes concernées et des
éventuelles dépendances amont. Une exploitation active ou un risque immédiat pour les utilisateurs
est traité en priorité.

Nous informerons le déclarant si le rapport est accepté, rejeté, déjà connu ou transmis à un projet
amont. Nous chercherons ensuite à convenir d'une date de divulgation laissant aux utilisateurs le
temps d'installer une version corrigée. Lorsque cela est pertinent, la publication pourra inclure
un GitHub Security Advisory, un identifiant CVE et le crédit du déclarant s'il le souhaite.

## English summary

Please report suspected vulnerabilities through
[GitHub's private vulnerability reporting](https://github.com/Sonar-team/Sonar_desktop_app/security/advisories/new).
Do not open a public issue, discussion, or pull request for an unpatched vulnerability. Include the
affected version and platform, security impact, minimal reproduction steps, and a sanitized proof
of concept. Never submit real network captures, unredacted logs, credentials, secrets, or third-party
data without prior agreement.
