# Feature : filtres de capture BPF

> Epic : capture — Statut : backlog P1 (implémentation en cours, branche `filter-fix-ux`)
> Issue : [#90](https://github.com/Sonar-team/Sonar_desktop_app/issues/90)

Construire un filtre BPF depuis des critères simples (couches, protocoles,
IP, réseaux, ports, presets, saisie manuelle) et rendre son état visible :
prêt, en attente de redémarrage ou actif. Le filtre agit au niveau capture,
pas rétroactivement sur l'affichage.

Contexte détaillé, parcours UX et état d'implémentation :
[`../../../filter_capture_user_stories.md`](../../../filter_capture_user_stories.md).

## User stories

- [ ] [US-01 — construire un filtre sans connaître la syntaxe BPF](us-01-construire-filtre-sans-bpf.md)
- [ ] [US-02 — comprendre quand le filtre s'applique](us-02-comprendre-quand-le-filtre-sapplique.md)
- [ ] [US-03 — supprimer le filtre configuré](us-03-supprimer-le-filtre.md)
- [ ] [US-04 — modifier manuellement le filtre](us-04-edition-manuelle.md)
- [ ] [US-05 — éviter les faux positifs visuels](us-05-lisibilite-options.md)
