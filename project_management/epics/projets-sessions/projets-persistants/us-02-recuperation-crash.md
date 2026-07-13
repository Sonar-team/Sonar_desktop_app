# US-02 — Récupérer une session après un crash

En tant qu'analyste, je veux retrouver ma session au redémarrage après un
crash, afin de ne pas refaire une capture qui n'est peut-être plus
reproductible.

Critères d'acceptation :
- au démarrage, une session interrompue est détectée et proposée à la
  récupération ;
- l'état récupéré est cohérent (pas de données partielles silencieuses) ;
- l'utilisateur peut refuser la récupération et repartir de zéro.
