# US-01 — Ne pas fusionner deux actifs de VLAN différents

En tant qu'auditeur ICS, je veux que deux équipements portant la même IP sur
deux VLAN ou sites différents restent deux actifs distincts, afin que la
matrice de flux reflète la réalité du réseau segmenté.

Critères d'acceptation :
- la clé d'actif intègre projet/site, capteur, interface et VLAN ;
- aucune fusion implicite de deux IP identiques dans des contextes
  différents ;
- labels, graphe et CSV restent cohérents après le changement d'identité.
