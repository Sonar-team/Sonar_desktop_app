# 🧭 Vision SONAR

> Document de vision produit. Il répond au **pourquoi** et au **vers quoi**,
> là où la [`Roadmap.md`](./Roadmap.md) répond au **quoi** et au **quand**.
> À relire quand une décision de fonctionnalité doit être arbitrée : une idée
> qui ne sert aucun des piliers ci-dessous ne mérite probablement pas la dette
> qu'elle introduit.

---

## 🎯 North Star

**Faire de SONAR l'outil de confiance, simple et accessible, pour cartographier
et inventorier les flux d'un réseau sensible ou OT — en passif, hors-ligne,
de façon normalisée et auditable.**

SONAR n'est pas « un analyseur réseau de plus ». Il vise un créneau précis :
les environnements où les autres outils ne passent pas la porte — réseaux
industriels, isolés, classifiés — et où la question n'est pas seulement *« qui
parle à qui ? »* mais aussi *« puis-je faire confiance à l'outil et à son
résultat ? »*.

SONAR s'adresse aussi aux utilisateurs qui ne sont pas spécialistes de la
capture ou de la télémétrie réseau. Les notions indispensables doivent être
expliquées par l'interface, les réglages sûrs proposés par défaut et les
résultats immédiatement exploitables sans connaître un protocole d'export de
flux.

---

## 🧩 Ce que SONAR est aujourd'hui

Capture **passive** → **matrice de flux** (format SFMS) → **graphe + labels** →
import / export / fusion. Le trafic tunnelé (CAPWAP, …) est décapsulé et
restitué en lignes père/fils reliées par un identifiant de tunnel — modèle et
choix techniques dans [`TUNNELS.md`](./TUNNELS.md).

Les entrées du produit restent volontairement peu nombreuses et faciles à
expliquer : une interface alimentée par un port miroir, un TAP ou un agrégateur
de TAP ; un fichier PCAP/PCAPNG ; ou une matrice SFMS existante.

**Décision (27/07/2026) : SONAR n'a pas vocation à devenir un collecteur
IPFIX, NetFlow ou sFlow, ni une sonde de capture exhaustive à 100 Gb/s.** Ces
formats et ce niveau de débit relèvent d'outils spécialisés. Ils ne sont pas
des entrées ou sorties prévues pour SONAR ; un équipement tiers peut en revanche
préparer un PCAP ciblé que SONAR analysera hors ligne.

Cinq choix techniques trahissent l'identité réelle du produit :

- **Parsing applicatif industriel** (Modbus, Profinet, OPC UA, S7Comm, SNMP,
  EtherNet/IP) → cible **OT / ICS**, pas seulement l'IT de bureau.
- **Fonctionnement hors-ligne / air-gapped** (vendoring Rust, builds offline
  reproductibles) → déploiement en **réseau isolé**.
- **Posture supply-chain forte** (builds reproductibles, SBOM, signatures
  Sigstore/cosign, attestations de provenance) → l'outil est **vérifiable** là
  où l'on doit prouver qu'un binaire n'est pas piégé.
- **Simplicité du parcours** (sources limitées, vocabulaire guidé, réglages
  avancés non obligatoires) → l'utilisateur n'a pas à devenir expert de la
  capture réseau pour produire un relevé exploitable.
- **Philosophie « SONAR ne tire aucune conclusion »** → l'outil **restitue**,
  l'humain **interprète et arbitre**.

### Deux formes, un seul cœur

SONAR se livre sous **deux formes**, portées à terme par le même cœur métier
(`sonar-flows-core`, workspace [`sonar-rust/`](./sonar-rust/)) :

- **SONAR desktop** (Tauri) — le relevé **interactif** : capture, graphe,
  labels, arbitrages. Une application Tauri repose sur la WebView et dépend
  donc **par construction** de la pile graphique de l'OS ; elle vise les
  postes avec interface.
- **sonar-cli** — le traitement **batch et serveur** : conversion PCAP →
  matrice, fusion de matrices, exit codes et stderr scriptables. C'est la
  forme prévue pour les machines sans interface graphique (Ubuntu Server,
  CI).

**Décision (11/07/2026) : il n'y a pas de « mode headless » de l'application
desktop.** Faire tourner un binaire Tauri sans GUI est un contresens
technique ; l'usage sans interface passe par `sonar-cli`. Le mode headless
résiduel du desktop est retiré ([#155](https://github.com/Sonar-team/Sonar_desktop_app/issues/155)),
et les traitements batch qui ne nécessitent pas d'interface (conversion PCAP,
fusion et export) appartiennent à `sonar-cli`, pas au desktop.

À ne pas confondre avec l'**interface en ligne de commande du binaire
desktop**, qui est conservée et assumée : lancer `sonar` avec des arguments
(smoke test de démarrage `--sonar-smoke-test`, options de session) sert
l'**orchestration** — Ansible ou équivalent — et l'automatisation des tests
de déploiement et d'intégration. La distinction : ces arguments **pilotent le
lancement** d'une application qui garde sa GUI (ou vérifient qu'elle sait
démarrer), ils ne prétendent pas la faire fonctionner sans.

---

## 🏛️ Les cinq piliers

### 1. Cartographie de confiance (cœur)
Restituer fidèlement les échanges observés sous forme de matrice de flux et de
graphe. Pas d'interprétation automatique : la donnée est brute, traçable,
exacte. C'est la fondation dont tout le reste dépend.

Cette fidélité doit rester accessible : le parcours principal emploie des
termes compréhensibles, explique les erreurs et ne demande pas de connaître les
mécanismes internes de capture pour obtenir un résultat correct.

### 2. Passif & hors-ligne par conception
En OT, le scan actif est souvent interdit ; en réseau classifié, la
connectivité externe l'est aussi. SONAR ne doit **rien émettre** sur le réseau
observé et doit fonctionner **sans dépendance en ligne**. C'est une contrainte,
mais surtout un argument différenciant à assumer et à démontrer.

### 3. Normalisation (SFMS)
Une matrice de flux n'a de valeur d'échange que si son format est stable et
documenté. Le **SONAR Flow Matrix Standard** est le format de référence de
SONAR : simple à exporter, réimporter, comparer et traiter avec des outils
courants, sans dépendre d'un collecteur ou d'un protocole constructeur.

### 4. Assurance vérifiable
La confiance ne se déclare pas, elle se prouve. Reproductibilité du build,
provenance, SBOM — et à terme la même exigence appliquée aux **livrables**
(matrices, rapports) pour qu'un tiers fasse confiance au résultat, pas seulement
au binaire.

### 5. Connaissance cumulée
Une capture est une photo ; l'audit et le SOC ont besoin d'un **film**. Labels
qui deviennent un référentiel d'actifs, comparaison de relevés dans le temps,
baseline : c'est ce qui transforme SONAR d'un outil de constat en outil de
suivi.

---

## 👥 Pour qui

- **Techniciens et équipes terrain** — obtenir un relevé exploitable avec un
  parcours guidé, sans expertise préalable des formats de télémétrie réseau.
- **Auditeurs réseau** — produire une cartographie défendable, comparable d'un
  audit à l'autre.
- **Architectes / analystes SOC & CSIRT** — établir une baseline du normal,
  détecter les écarts, alimenter les règles.
- **Responsables OT / ICS** — inventorier un réseau industriel **sans jamais
  l'émettre**, avec les protocoles métier reconnus.
- **Administrateurs réseau** — découvrir le shadow IT/OT, cartographier les
  angles morts.

---

## 🛣️ Axes d'évolution

Ordonnés par effet de levier sur la North Star. Ils complètent la
`Roadmap.md` (qui détaille les incréments) en donnant leur *raison d'être*.

### Axe A — Baseline & diff temporel *(pilier 5 — priorité haute)*
Passer de la photo au film : comparer deux relevés (« quel équipement / flux
est apparu, a disparu, a changé depuis le dernier audit »), notion de
`first_seen` / fenêtre temporelle (extension **SFMS-T** déjà anticipée).
Débloque simultanément l'audit (preuve de dérive) et le SOC (baseline).

### Axe B — Labels → référentiel d'actifs *(pilier 5)*
Les labels deviennent des **entités** (zone, VLAN, criticité, propriétaire),
importables depuis un référentiel existant (ex. base réseau Excel), groupables
dans le graphe. Étapes déjà engagées : import/export de labels, arbitrage des
conflits, puis **vue dédiée « Gestion des labels »**.

### Axe C — Assistance à la détection *(pilier 5 + philosophie)*
Depuis une baseline validée, signaler les écarts (équipement nouveau, flux
inattendu, protocole indésirable) avec trois catégories directement lisibles :
présent dans la référence et observé, nouvellement observé, attendu mais non
observé. Reste **assistif** : SONAR présente les faits, l'humain décide.

### Axe D — Standard SFMS *(pilier 3)*
Publier un schéma SFMS versionné, un exemple commenté et un validateur de
conformité. Le format doit rester lisible dans un tableur ou un outil courant ;
son évolution ne doit pas imposer de notions de télémétrie avancée à
l'utilisateur.

### Axe E — Assurance des livrables *(pilier 4)*
Signer les **exports** (matrice, rapport d'audit), mode « rapport de
conformité » lui-même attestable, extension progressive de la reproductibilité
aux installateurs natifs.

### Axe F — « Passif-only » démontrable *(pilier 2)*
Formaliser et **prouver** (tests, documentation) que SONAR n'émet rien sur le
réseau observé. Argument décisif en OT.

---

## 🧪 Principes directeurs

- **Fidélité avant tout.** Aucune donnée inventée, aucune conclusion
  automatique. En cas d'ambiguïté (ex. conflit de labels), on **rend la
  décision à l'utilisateur** plutôt que de trancher silencieusement.
- **Accessible avant d'être extensible.** Un protocole ou un réglage n'entre
  pas dans le produit uniquement parce qu'il existe : il doit répondre à un
  besoin utilisateur démontré et rester compréhensible sans expertise
  spécialisée.
- **Passif et hors-ligne par défaut.** Toute fonctionnalité qui exigerait
  d'émettre sur le réseau observé ou une dépendance en ligne doit être
  explicitement justifiée et optionnelle.
- **Vérifiable, pas seulement fonctionnel.** Un livrable (binaire ou export)
  doit pouvoir être contrôlé par un tiers sans faire confiance à l'auteur.
- **Interopérable par la simplicité.** Le format prime sur l'outil : mieux vaut
  une matrice SFMS documentée et réutilisable qu'une multiplication de
  connecteurs spécialisés.
- **La dette se justifie par un pilier.** Une fonctionnalité qui ne sert aucun
  des cinq piliers est probablement hors périmètre.

---

## 🚫 Ce que SONAR n'est pas (non-buts)

- Un IDS/IPS ou un SIEM — les exports de SONAR peuvent **alimenter** ces outils,
  mais SONAR ne les remplace pas.
- Un collecteur IPFIX, NetFlow ou sFlow — ces protocoles spécialisés ne sont
  pas des sources natives de SONAR et leur prise en charge n'est pas un objectif
  produit.
- Une sonde de capture exhaustive pour cœur de réseau à 100 Gb/s — ce besoin
  relève d'un équipement spécialisé ; SONAR peut analyser les PCAP ciblés
  produits par cet équipement.
- Un scanner actif de réseau — la découverte est **passive**.
- Un déchiffreur de trafic — l'analyse applicative reste au niveau
  **observable** (pas de déchiffrement TLS).
- Un outil de supervision temps réel permanent — l'usage cible est le
  **relevé / l'audit**, pas le monitoring 24/7.
- Un service headless dérivé du desktop — une app Tauri dépend de la pile
  graphique de l'OS ; **sans GUI, c'est `sonar-cli`**.

---

## 🧭 Boussole de décision

Face à une nouvelle idée, se poser dans l'ordre :

1. Sert-elle un des **cinq piliers** ? Lequel ?
2. Respecte-t-elle le **passif / hors-ligne** ?
3. Reste-t-elle **compréhensible par un utilisateur non spécialiste** ?
4. Laisse-t-elle la **décision finale à l'humain** ?
5. Le résultat reste-t-il **vérifiable et interopérable** ?

Si une réponse est « non » sans justification forte, l'idée attend.

---

> **Projet :** SONAR — Surveillance Optimisée des Nœuds pour Analyse Réseau
> **Organisation :** ERDT-CYBER / SSF Toulon
> **Licence :** AGPLv3
> **Voir aussi :** [`Roadmap.md`](./Roadmap.md) · [`README.md`](./README.md) ·
> [`security/`](./security/)
