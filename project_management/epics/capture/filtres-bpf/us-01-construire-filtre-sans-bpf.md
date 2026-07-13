# US-01 — Construire un filtre sans connaître la syntaxe BPF

En tant qu'analyste réseau, je veux cocher des critères simples pour générer
un filtre BPF valide, afin de limiter le bruit de capture sans mémoriser la
syntaxe exacte.

Critères d'acceptation :
- les options de couche, protocole, IP, réseau et ports mettent à jour
  l'aperçu automatiquement ;
- les erreurs de saisie bloquent l'application du filtre ;
- les presets génèrent un filtre cohérent et visible dans l'aperçu.
