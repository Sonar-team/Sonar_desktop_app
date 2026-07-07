# 3. Scénarios d'attaque opérationnels

Chaque scénario décrit un enchaînement réaliste, du point d'entrée à
l'impact, pour une application Tauri open source non protégée. Ils
correspondent aux risques R1 à R9 du chapitre 2 et servent de référence aux
contre-mesures du chapitre 4.

## Scénario A — La dépendance transitive piégée (R1, R3)

**Étape 1.** L'attaquant identifie une dépendance Rust ou npm peu surveillée
mais largement transitive du projet cible.

**Étape 2.** Il en prend le contrôle : vol des identifiants d'un mainteneur
amont (hameçonnage, jeton fuité), ou publication d'un paquet au nom proche
(`packet-parser` au lieu de `packet_parser`).

**Étape 3.** Il publie une version mineure piégée. Comme le projet cible
déclare ses dépendances avec des plages de versions flottantes (`^1.2.0`) et
ne fige pas ses installations, **le prochain build récupère automatiquement
la version piégée**.

**Étape 4.** La charge s'exécute au moment du build (`build.rs` en Rust,
script `postinstall` en npm) ou au runtime chez l'utilisateur.

**Impact.** Exécution de code arbitraire sur les postes de build et
d'utilisateurs. C'est le mécanisme des campagnes event-stream (2018) et
ua-parser-js (2021).

## Scénario B — Le lockfile empoisonné (R2)

**Étape 1.** L'attaquant, contributeur occasionnel du projet, ouvre une pull
request d'apparence anodine intitulée « chore: update dependencies ».

**Étape 2.** Le diff modifie plusieurs centaines de lignes de fichier de
verrouillage. Une seule ligne redirige une dépendance vers une source ou une
version compromise.

**Étape 3.** Le relecteur, submergé par la taille du diff, approuve la PR
sans vérifier chaque entrée — le lockfile est réputé « généré
automatiquement ».

**Impact.** La dépendance compromise entre durablement dans le produit, dans
le bruit d'une opération de maintenance de routine.

## Scénario C — L'injection au build, façon SUNBURST (R4)

**Étape 1.** L'attaquant compromet l'environnement de compilation : runner
CI mal isolé, image Docker de base empoisonnée, ou outil de build téléchargé
sans vérification d'intégrité.

**Étape 2.** Au moment de la compilation de release, il substitue un fichier
ou injecte du code dans le binaire produit, puis efface toute trace.

**Étape 3.** Le binaire troyanisé suit le processus normal : il est signé,
publié, et rien dans le dépôt git ne trahit l'attaque.

**Impact.** Distribution d'un implant à tous les utilisateurs, avec la
signature officielle. **Aucune revue de code ne peut le détecter** : c'est
exactement SolarWinds. Seule une reconstruction indépendante comparant les
binaires révélerait l'écart.

## Scénario D — La release fabriquée à la main, façon XZ (R5, R8)

**Étape 1.** Un contributeur patient obtient, après des mois de
contributions légitimes, le droit de publier les releases.

**Étape 2.** Il génère les artefacts de release **sur son poste**, hors CI,
et y ajoute une charge absente du dépôt — dissimulée dans un fichier binaire
embarqué ou un script de packaging.

**Étape 3.** Il téléverse manuellement les binaires sur la page des releases
GitHub. Les utilisateurs téléchargent un artefact qui ne correspond pas au
code public, sans aucun moyen de le savoir.

**Impact.** Porte dérobée distribuée officiellement, invisible à qui compare
le seul code source. C'est le cœur du mécanisme XZ Utils.

## Scénario E — L'action CI détournée (R6)

**Étape 1.** Le projet utilise une action GitHub tierce référencée par tag
flottant : `uses: some-org/some-action@v4`.

**Étape 2.** L'attaquant compromet le compte de l'auteur de l'action et
**republie le tag `v4`** en le faisant pointer vers du code malveillant.

**Étape 3.** Au prochain déclenchement de la CI du projet cible, l'action
piégée s'exécute avec les permissions du job : elle lit les secrets
(`GITHUB_TOKEN`, clés de signature) et les exfiltre, ou modifie les
artefacts avant publication.

**Impact.** Prise de contrôle de la chaîne de publication. Réalisé à grande
échelle en mars 2025 via l'action `tj-actions/changed-files`, exposant les
secrets de milliers de dépôts.

## Scénario F — La vulnérabilité dormante (R7)

**Étape 1.** Une CVE critique est publiée sur une bibliothèque de
compression, de parsing ou de traitement d'image embarquée transitivement.

**Étape 2.** Le projet n'a **aucun inventaire** de ses composants (pas de
SBOM) et **aucun scan continu** : ni les mainteneurs ni les utilisateurs ne
savent que le composant vulnérable est présent.

**Étape 3.** L'attaquant exploite la vulnérabilité sur les postes où
l'application est installée.

**Impact.** Compromission via une faille pourtant connue et corrigée en
amont, faute de visibilité sur la composition du logiciel.

## Scénario G — L'évasion depuis le webview (R9)

**Étape 1.** Une dépendance frontend compromise (scénario A côté npm) ou une
faille XSS introduit du code hostile dans le webview de l'application.

**Étape 2.** Ce code tente de charger une charge secondaire depuis un serveur
distant, puis d'invoquer des commandes natives Tauri pour accéder au système
de fichiers ou au réseau.

**Impact.** Exfiltration des données de l'utilisateur (dans le cas d'un
analyseur réseau : captures, cartographie du SI) et potentielle exécution de
commandes système, si aucune politique de sécurité de contenu ni limitation
des permissions n'est en place.

---

Ces sept scénarios couvrent l'ensemble du cycle de vie : approvisionnement
(A, B), fabrication (C), distribution (D, E), et exposition résiduelle
(F, G). Le chapitre 4 leur oppose les contre-mesures effectivement en place.
