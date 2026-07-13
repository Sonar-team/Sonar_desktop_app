# US-01 — Arrêter sans perdre de paquets

En tant qu'analyste réseau, je veux que l'arrêt d'une capture draine tous
les paquets déjà acceptés, afin que le résultat affiché corresponde
exactement à ce qui a été capturé.

Critères d'acceptation :
- le thread producteur est arrêté et joint avant la fin du drainage ;
- le canal est drainé jusqu'à déconnexion, sans timeout silencieux ;
- les compteurs finaux reçus/acceptés/traités/perdus sont cohérents ;
- toute perte résiduelle est comptée avec une raison explicite.
