# User story — Annulation d'un import PCAP en cours

Reliquat de l'issue #161 (« Intégrité du cycle frontend ») : les imports
longs affichent leur progression mais ne peuvent pas être interrompus.

## User story

> **En tant qu'** opérateur SONAR,
> **je veux** pouvoir annuler un import PCAP en cours lorsque je constate
> qu'il est trop long (fichier plus volumineux que prévu, mauvaise
> sélection, liste de fichiers trop grande),
> **afin de** reprendre immédiatement la main sur l'application sans la
> fermer de force et sans perdre le relevé sur lequel je travaillais.

## Critères d'acceptation

1. **Bouton visible** : pendant un import PCAP, l'overlay de progression
   (fichier N/M, nom, compteur de paquets) propose un bouton
   « Annuler l'import », accessible au clavier et nommé pour les lecteurs
   d'écran. Après le clic, le bouton passe à « Annulation… » et ne peut
   pas être re-cliqué.
2. **Effet rapide** : l'annulation est coopérative. Le fichier en cours
   cesse d'être analysé (les paquets restants sont drainés sans parsing ni
   mise à jour de matrice/graphe) et les fichiers suivants de la liste ne
   sont pas ouverts. L'UI est libérée en quelques secondes au plus, même
   sur un fichier volumineux.
3. **Session préservée** : l'import étant transactionnel, une annulation
   ne modifie ni la matrice, ni le graphe, ni les labels, ni la
   configuration de capture — mêmes garanties qu'un import rejeté (#167).
4. **Issue normale, pas une erreur** : l'annulation est signalée par une
   notification claire (« Import annulé. Le relevé courant est
   inchangé. »), distincte visuellement d'une erreur, et journalisée dans
   les logs de production. Côté IPC, elle voyage comme erreur typée
   `import/cancelled` du contrat généré (#142), jamais comme chaîne libre.
5. **État rendu** : après annulation, la phase revient à `Idle` : un
   nouvel import ou une capture peuvent démarrer aussitôt. Une annulation
   demandée alors qu'aucun import ne tourne (course avec la fin d'import)
   est sans effet et sans erreur affichée.
6. **Tests** : tests Rust (annulation entre fichiers et en cours de
   fichier, préservation de l'état, remise à zéro du flag par la
   réservation suivante) et tests frontend (classification de l'erreur
   `cancelled`, messages).

## Hors périmètre (suivi ultérieur)

- Annulation des imports de matrices CSV/XLSX et de labels : courts en
  pratique ; le flag posé dans `ImportGuard` leur permettra d'adopter le
  même mécanisme si besoin.
- Interruption sans drainage du fichier en cours (arrêt de la lecture
  elle-même) : nécessite une boucle interruptible dans `sonar-flows-core`
  (modification upstream + re-vendoring, jamais d'édition locale du
  vendor), à traiter avec la crate.
- La phase de comptage initial des paquets d'un fichier n'est pas
  interruptible (même contrainte upstream) ; l'annulation prend effet
  entre fichiers et pendant la phase d'analyse.
