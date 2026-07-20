# Sprint: Fiabilisation CI et release

> Statut : archivé pour #135/#137 ; #136 rouvert en P0
> Dernière revue : 20/07/2026
> #138 reste ouverte en P2 : Npcap est un prérequis externe détecté par NSIS,
> avec redirection officielle ; la discussion Nmap viendra ensuite.
> Les P0 de release ouverts sont #94, #136, #146 et #162.
>
> Suivi GitHub: [#135](https://github.com/Sonar-team/Sonar_desktop_app/issues/135),
> [#136](https://github.com/Sonar-team/Sonar_desktop_app/issues/136),
> [#137](https://github.com/Sonar-team/Sonar_desktop_app/issues/137),
> [#138](https://github.com/Sonar-team/Sonar_desktop_app/issues/138)

## Objectif

Rendre la chaîne CI/release digne de confiance : chaque PR est bloquée par
les vérifications réelles (elles ne le sont pas aujourd'hui), et une release
publiée est complète, validée et correctement nommée — ou n'existe pas.

Issu de l'audit de code du 10/07/2026. Ce document décrit son périmètre CI et
release initial ; il ne vaut pas validation globale de la fidélité runtime,
désormais suivie dans le sprint actif #165.

## Livrables

1. **Gates de PR effectifs** (#135)
   - `rust-clippy.yml` réparé (répertoires `src-tauri` et `sonar-rust`,
     `continue-on-error` retiré) ;
   - `sonar-rust` intégré à `rust-ci.yml` (fmt, clippy `-D warnings`, tests) ;
   - job frontend : `deno task typecheck` + `deno task test` ;
   - `cargo fmt` corrigé dans les deux workspaces puis gate `--check` ;
   - artefact macOS renommé selon la cible réelle (`x86_64-apple-darwin`).
2. **Release atomique** (#136)
   - livré : release créée en draft et publiée après les validations
     principales ;
   - rouvert le 20/07 : construire une seule fois puis tester, scanner,
     signer, attester et publier les mêmes octets, avec permissions minimales
     et sans remplacement `--clobber`.
3. **SBOM frontend complet** (#137)
   - contournement de la non-prise en charge de `deno.lock` par Syft
     (catalogage de `node_modules/` ou lockfile dérivé) ;
   - SBOM frontend contenant Vue/Vite/Pinia/Sigma/plugins Tauri JS, sans
     éléments Rust ou Npcap.
4. **Décision Npcap documentée** (#138) : aucun installeur dans le dépôt ou
   les bundles, détection pendant l'installation et redirection vers le site
   officiel ; l'issue reste ouverte pour demander à Nmap une solution adaptée
   au projet open source.

## Critères d'acceptation

- Une PR qui casse clippy, fmt, le typecheck ou un test (Rust, sonar-rust ou
  frontend) est rouge.
- Un tag de release dont un build échoue ne laisse aucune release publique
  visible.
- Les artefacts testés sont exactement ceux publiés et un rerun ne remplace
  jamais une release déjà publique (#136).
- Le SBOM frontend d'une release candidate liste les dépendances JS réelles.
- Aucun installeur Npcap n'est présent dans le dépôt ou les bundles ; le NSIS
  détecte le prérequis et redirige vers la page officielle.

## Hors périmètre (backlog, voir todo.md)

Exclusion mutuelle capture/import/export (#139), pool jumbo frames (#140),
télémétrie backpressure (#141), typage IPC (#142), capacités Tauri (#143),
accessibilité (#144), vues mortes (#145), stratégie E2E (#146).
