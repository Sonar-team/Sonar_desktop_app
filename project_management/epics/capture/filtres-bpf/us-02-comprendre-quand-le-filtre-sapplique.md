# US-02 — Comprendre quand le filtre s'applique

En tant qu'utilisateur en phase de capture, je veux voir si mon filtre est
actif, prêt ou en attente de redémarrage, afin de ne pas penser que
l'application a ignoré ma demande.

Critères d'acceptation :
- la barre haute affiche un badge de filtre quand un filtre est configuré ou
  actif ;
- si la capture tourne déjà, le statut indique que le nouveau filtre est en
  attente de redémarrage ;
- si la capture est arrêtée, le statut indique que le filtre est prêt pour
  la prochaine capture ;
- une capture démarrée avec un filtre configuré affiche ensuite un statut
  actif.

> Reste à faire (branche `filter-fix-ux`) : distinguer filtre *actif* de
> filtre *en attente* dans le badge du panel et de la status bar.
