# US-03 — Prouver l'intégrité d'un relevé via le manifest

En tant qu'auditeur, je veux qu'un projet embarque un manifest de preuve
(source, compteurs, pertes, versions), afin d'attester que le résultat
présenté correspond aux données réellement acceptées.

Critères d'acceptation :
- le manifest reprend le bilan lus/acceptés/traités/perdus par source ;
- il référence la version de SONAR et du parseur utilisées ;
- il est exportable avec le projet et lisible sans l'application.
