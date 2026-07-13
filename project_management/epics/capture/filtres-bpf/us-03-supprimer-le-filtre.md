# US-03 — Supprimer le filtre configuré

En tant qu'analyste, je veux effacer le filtre actif depuis le panneau, afin
de revenir à une capture non filtrée sans redémarrer l'application.

Critères d'acceptation :
- le bouton d'effacement appelle le backend avec un filtre vide ;
- le backend convertit un filtre vide en `None` ;
- si une capture filtrée est déjà en cours, l'UI indique que la suppression
  prendra effet au prochain redémarrage ;
- si aucune capture n'est en cours, le badge de filtre disparaît.
