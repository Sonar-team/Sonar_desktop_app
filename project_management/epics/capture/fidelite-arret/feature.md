# Feature : arrêt fidèle de la capture

> Epic : capture — Statut : sprint actif (P0)
> Issue : [#158](https://github.com/Sonar-team/Sonar_desktop_app/issues/158)

Aucun paquet accepté ne doit être perdu silencieusement à l'arrêt de la
capture ni lorsque le plafond de flux est atteint : producteur arrêté et
joint avant la fin du drainage, canal drainé jusqu'à déconnexion, compteurs
finaux cohérents (reçus/acceptés/traités/perdus).

## User stories

- [ ] [US-01 — arrêter sans perdre de paquets](us-01-stop-sans-perte.md)
- [ ] [US-02 — atteindre le plafond de flux sans perte invisible](us-02-plafond-flux.md)
