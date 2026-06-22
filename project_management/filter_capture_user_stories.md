# Feature: filtres de capture BPF

## Contexte produit

Le panneau `Filter.vue` permet de préparer un filtre BPF pour limiter les paquets capturés par `pcap`. Le filtre est construit à partir d'options simples: couches, protocoles, adresses IP, réseaux, ports, plages de ports, presets et saisie manuelle.

Point important: ce filtre agit au niveau capture. Il ne filtre pas rétroactivement les paquets déjà affichés dans la table ou le graphe. Si une capture est déjà en cours, le filtre enregistré est appliqué au prochain démarrage de capture.

## Parcours utilisateur

1. L'utilisateur ouvre le panneau depuis le bouton filtre de la barre haute.
2. Il sélectionne des critères ou choisit un preset.
3. L'aperçu génère une expression BPF.
4. L'utilisateur applique le filtre.
5. Le backend stocke le filtre dans `CaptureState.filter`.
6. Au prochain `start_capture`, `setup_filter` appelle `cap.filter(...)`.

## Problème UX identifié

L'ancienne interface fermait le panneau après l'application sans afficher d'état persistant. Elle ne distinguait pas non plus un filtre actif d'un filtre seulement préparé pour le prochain démarrage. En capture déjà lancée, cela donnait l'impression que le filtre ne fonctionnait pas.

Autre problème: le bouton `Effacer` remettait seulement le formulaire à zéro. Il ne supprimait pas vraiment le filtre stocké côté backend.

## User Stories

### US-01 - Construire un filtre sans connaître la syntaxe BPF

En tant qu'analyste réseau, je veux cocher des critères simples pour générer un filtre BPF valide, afin de limiter le bruit de capture sans mémoriser la syntaxe exacte.

Critères d'acceptation:
- Les options de couche, protocole, IP, réseau et ports mettent à jour l'aperçu automatiquement.
- Les erreurs de saisie bloquent l'application du filtre.
- Les presets génèrent un filtre cohérent et visible dans l'aperçu.

### US-02 - Comprendre quand le filtre s'applique

En tant qu'utilisateur en phase de capture, je veux voir si mon filtre est actif, prêt ou en attente de redémarrage, afin de ne pas penser que l'application a ignoré ma demande.

Critères d'acceptation:
- La barre haute affiche un badge de filtre quand un filtre est configuré ou actif.
- Si la capture tourne déjà, le statut indique que le nouveau filtre est en attente de redémarrage.
- Si la capture est arrêtée, le statut indique que le filtre est prêt pour la prochaine capture.
- Une capture démarrée avec un filtre configuré affiche ensuite un statut actif.

### US-03 - Supprimer le filtre configuré

En tant qu'analyste, je veux effacer le filtre actif depuis le panneau, afin de revenir à une capture non filtrée sans redémarrer l'application.

Critères d'acceptation:
- Le bouton d'effacement appelle le backend avec un filtre vide.
- Le backend convertit un filtre vide en `None`.
- Si une capture filtrée est déjà en cours, l'UI indique que la suppression prendra effet au prochain redémarrage.
- Si aucune capture n'est en cours, le badge de filtre disparaît.

### US-04 - Modifier manuellement le filtre

En tant qu'utilisateur avancé, je veux pouvoir écrire directement une expression BPF, afin de couvrir des cas non exposés par les contrôles rapides.

Critères d'acceptation:
- La saisie manuelle est conservée.
- L'interface signale clairement que le mode manuel est actif.
- Un bouton permet de reprendre la génération automatique depuis les contrôles.

### US-05 - Eviter les faux positifs visuels

En tant qu'utilisateur, je veux que les options sélectionnées soient visuellement distinctes, afin de comprendre rapidement quels critères participent au filtre final.

Critères d'acceptation:
- Les options cochées sont mises en évidence.
- L'aperçu est lisible et contrasté.
- Les messages d'erreur ou de confirmation sont visibles dans le panneau.

## Implémentation en cours — branche `filter-fix-ux` (non commité)

### `src/store/capture.ts`
- Ajout de `activeFilter: string` dans le state Pinia.
- Ajout de l'action `setActiveFilter(filter)` pour mettre à jour la valeur globalement.

### `src/components/AnalyseView/panels/Filter.vue`
- Conversion Options API → `<script setup>` (Composition API).
- Refonte UI : header avec titre et bouton ✕ de fermeture, presets déplacés en haut du panneau, section "Expression BPF brute" séparée des champs structurés.
- `apply()` et `resetAll()` appellent désormais `captureStore.setActiveFilter()` pour synchroniser l'état global.
- Ajout du bouton "↺ Sync auto" pour reprendre la génération automatique après une édition manuelle du textarea (couvre US-04).
- Badge "édition manuelle" visible dans le header de l'aperçu lorsque le mode manuel est actif.
- Styles dark theme refaits : `.filter-overlay`, `.filter-panel`, `.field`, `.btn`, `.chip`, `.raw-input`, etc.

### `src/components/NavBar/status-bar/StatusBar.vue`
- Ajout d'un badge filtre actif dans la barre de statut : affiche le texte du filtre tronqué (max 320 px), avec un bouton ✕ pour le supprimer.
- Le ✕ appelle `invoke('set_filter', { filter: '' })` + `captureStore.setActiveFilter('')` (couvre US-03).
- Couvre partiellement US-02 : le badge apparaît/disparaît selon `captureStore.activeFilter`.

### `src/components/NavBar/TopBar.vue`
- Ajout de `toggle-graph` dans les emits déclarés.
- Suppression de la méthode `toggleConfig()` inutilisée.

### `src/components/AnalyseView/panels/Filter.vue` — bandeau filtre actif
- Ajout d'un bandeau "Filtre actif" entre le header et les presets, visible uniquement quand `captureStore.activeFilter` est non vide.
- Affiche l'expression BPF courante (tronquée si longue) avec un bouton "Supprimer" qui appelle `resetAll()`.
- Couvre US-02 côté panel : l'utilisateur voit immédiatement le filtre en vigueur en ouvrant le panel.

### `src/components/AnalyseView/panels/Filter.vue` — backdrop et comportement modal
- Correction d'un bug visuel : l'overlay sans fond faisait paraître l'app plus sombre à la fermeture du panel (le fond `#1e1e2e` du panel était plus clair que le contenu en dessous).
- Ajout de `background: rgba(0,0,0,0.5)` sur `.filter-overlay` : le fond de l'app est assombri quand le panel est ouvert, comme un modal standard.
- Suppression de `pointer-events: none` sur l'overlay ; l'overlay capture maintenant les clics.
- Clic en dehors du panel ferme le panel (`@click` sur l'overlay + `@click.stop` sur le panel).

### Ce qui reste à faire (US-02 partiel)
- Distinguer filtre *actif* (capture en cours avec ce filtre) de filtre *en attente* (configuré mais capture pas encore redémarrée) — le badge panel et le badge status bar ne font pas encore cette distinction.

---

## Risques et suite

- Le filtre n'est pas encore appliqué en live sur une capture déjà ouverte, car le handle `pcap` est possédé par le thread de capture.
- Pour du filtrage live, il faudrait ajouter un canal de contrôle vers le thread de capture ou redémarrer explicitement la capture.
- Le filtre actuel agit à l'entrée de capture; un filtre d'affichage local serait une feature différente.
