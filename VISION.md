# 🧭 Vision SONAR

> Document de vision produit. Il répond au **pourquoi** et au **vers quoi**,
> là où la [`Roadmap.md`](./Roadmap.md) répond au **quoi** et au **quand**.
> À relire quand une décision de fonctionnalité doit être arbitrée : une idée
> qui ne sert aucun des piliers ci-dessous ne mérite probablement pas la dette
> qu'elle introduit.

---

## 🎯 North Star

**Faire de SONAR la référence de confiance pour cartographier et inventorier
les flux d'un réseau sensible ou OT — en passif, hors-ligne, de façon
normalisée et auditable.**

SONAR n'est pas « un analyseur réseau de plus ». Il vise un créneau précis :
les environnements où les autres outils ne passent pas la porte — réseaux
industriels, isolés, classifiés — et où la question n'est pas seulement *« qui
parle à qui ? »* mais aussi *« puis-je faire confiance à l'outil et à son
résultat ? »*.

---

## 🧩 Ce que SONAR est aujourd'hui

Capture **passive** → **matrice de flux** (format SFMS) → **graphe + labels** →
import / export / fusion. Le trafic tunnelé (CAPWAP, …) est décapsulé et
restitué en lignes père/fils reliées par un identifiant de tunnel — modèle et
choix techniques dans [`TUNNELS.md`](./TUNNELS.md).

Quatre choix techniques trahissent l'identité réelle du produit :

- **Parsing applicatif industriel** (Modbus, Profinet, OPC UA, S7Comm, SNMP,
  EtherNet/IP) → cible **OT / ICS**, pas seulement l'IT de bureau.
- **Fonctionnement hors-ligne / air-gapped** (vendoring Rust, builds offline
  reproductibles) → déploiement en **réseau isolé**.
- **Posture supply-chain forte** (builds reproductibles, SBOM, signatures
  Sigstore/cosign, attestations de provenance) → l'outil est **vérifiable** là
  où l'on doit prouver qu'un binaire n'est pas piégé.
- **Philosophie « SONAR ne tire aucune conclusion »** → l'outil **restitue**,
  l'humain **interprète et arbitre**.

---

## 🏛️ Les cinq piliers

### 1. Cartographie de confiance (cœur)
Restituer fidèlement les échanges observés sous forme de matrice de flux et de
graphe. Pas d'interprétation automatique : la donnée est brute, traçable,
exacte. C'est la fondation dont tout le reste dépend.

### 2. Passif & hors-ligne par conception
En OT, le scan actif est souvent interdit ; en réseau classifié, la
connectivité externe l'est aussi. SONAR ne doit **rien émettre** sur le réseau
observé et doit fonctionner **sans dépendance en ligne**. C'est une contrainte,
mais surtout un argument différenciant à assumer et à démontrer.

### 3. Normalisation (SFMS)
Une matrice de flux n'a de valeur d'échange que si son format est stable et
interopérable. Le **SONAR Flow Matrix Standard** vise à devenir *le* format de
référence, mappable vers l'écosystème existant (IPFIX / NetFlow / NDR).

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
inattendu, protocole indésirable) et **exporter vers IDS/SIEM** via le mapping
SFMS → IPFIX. Reste **assistif** : SONAR propose, l'humain décide.

### Axe D — Standard SFMS *(pilier 3)*
Schéma JSON + validateur de conformité, mapping IPFIX documenté, publication
comme référence ouverte (voire Internet-Draft). Fort effet de levier sur
l'adoption et l'interopérabilité.

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
- **Passif et hors-ligne par défaut.** Toute fonctionnalité qui exigerait
  d'émettre sur le réseau observé ou une dépendance en ligne doit être
  explicitement justifiée et optionnelle.
- **Vérifiable, pas seulement fonctionnel.** Un livrable (binaire ou export)
  doit pouvoir être contrôlé par un tiers sans faire confiance à l'auteur.
- **Interopérable.** Le format prime sur l'outil : mieux vaut une matrice
  réutilisable ailleurs qu'un silo propriétaire.
- **La dette se justifie par un pilier.** Une fonctionnalité qui ne sert aucun
  des cinq piliers est probablement hors périmètre.

---

## 🚫 Ce que SONAR n'est pas (non-buts)

- Un IDS/IPS ou un SIEM — SONAR **alimente** ces outils, il ne les remplace pas.
- Un scanner actif de réseau — la découverte est **passive**.
- Un déchiffreur de trafic — l'analyse applicative reste au niveau
  **observable** (pas de déchiffrement TLS).
- Un outil de supervision temps réel permanent — l'usage cible est le
  **relevé / l'audit**, pas le monitoring 24/7.

---

## 🧭 Boussole de décision

Face à une nouvelle idée, se poser dans l'ordre :

1. Sert-elle un des **cinq piliers** ? Lequel ?
2. Respecte-t-elle le **passif / hors-ligne** ?
3. Laisse-t-elle la **décision finale à l'humain** ?
4. Le résultat reste-t-il **vérifiable et interopérable** ?

Si une réponse est « non » sans justification forte, l'idée attend.

---

> **Projet :** SONAR — Surveillance Optimisée des Nœuds pour Analyse Réseau
> **Organisation :** ERDT-CYBER / SSF Toulon
> **Licence :** AGPLv3
> **Voir aussi :** [`Roadmap.md`](./Roadmap.md) · [`README.md`](./README.md) ·
> [`security/`](./security/)
