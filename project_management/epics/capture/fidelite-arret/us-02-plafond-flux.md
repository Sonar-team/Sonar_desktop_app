# US-02 — Atteindre le plafond de flux sans perte invisible

En tant qu'analyste réseau, je veux que l'atteinte de la limite de flux se
comporte comme un arrêt propre, afin de ne jamais découvrir après coup que
des paquets ont été jetés sans trace.

Critères d'acceptation :
- le comportement au plafond est identique à celui d'un stop manuel ;
- l'UI signale clairement que la limite a été atteinte ;
- le bilan final distingue pertes noyau, interface et application.
