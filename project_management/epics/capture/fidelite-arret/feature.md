# Feature : arrêt fidèle de la capture

> Epic : capture — Statut : livré le 14/07/2026 (P0 fermé)
> Issue : [#158](https://github.com/Sonar-team/Sonar_desktop_app/issues/158)

Aucun paquet accepté ne doit être perdu silencieusement à l'arrêt de la
capture ni lorsque le plafond de flux est atteint : producteur arrêté et
joint avant la fin du drainage, canal drainé jusqu'à déconnexion, compteurs
finaux cohérents (reçus/acceptés/traités/perdus).

## User stories

- [x] [US-01 — arrêter sans perdre de paquets](us-01-stop-sans-perte.md)
- [x] [US-02 — atteindre le plafond de flux sans perte invisible](us-02-plafond-flux.md)

Le risque d'interblocage start/stop découvert ensuite est distinct du drainage
et suivi dans [#166](https://github.com/Sonar-team/Sonar_desktop_app/issues/166).
