# Sprint : chaîne DevSecOps frontend

> Statut : planifié
> Dernière revue : 17/07/2026
> Périmètre : frontend Vue/TypeScript, dépendances npm résolues par Deno et
> contrôles GitHub Actions.

## Objectif

Mettre en place une chaîne frontend reproductible et bloquante qui détecte
les défauts de qualité, les erreurs de typage, les vulnérabilités connues,
les secrets accidentellement committés et les patterns de code dangereux
avant intégration ou publication.

ESLint est le contrôle de qualité du code ; il ne remplace ni l'analyse SAST,
ni l'audit des dépendances, ni la protection runtime par CSP.

## État initial

- `vue-tsc --noEmit` est exécuté en CI.
- Les tests Deno frontend sont exécutés en CI.
- Le build Vite est utilisé par la chaîne Tauri.
- Les versions et le graphe de dépendances sont verrouillés par
  `package.json`, `deno.json` et `deno.lock`.
- ESLint, Prettier, SAST JavaScript/Vue et détection de secrets ne sont pas
  encore des gates frontend dédiées. L'intégration ESLint est maintenant
  amorcée avec `eslint.config.js` et `deno task lint` ; le traitement des
  violations existantes reste le travail FE-01. Prettier est intégré pour les
  nouveaux fichiers de configuration ; le reformatage progressif de `src/`
  reste le travail FE-02 afin d'éviter un diff massif.

## Périmètre

### Inclus

- ESLint avec support Vue et TypeScript, sans warning toléré.
- Prettier en mode vérification CI.
- Tests, typecheck et build frontend comme gates explicites.
- Scan SAST ciblé sur `src/` avec Semgrep.
- Audit des dépendances frontend depuis `deno.lock`.
- Détection de secrets avec Gitleaks.
- Publication SARIF dans l'onglet Security lorsque l'outil le permet.
- Documentation des exceptions et du processus de mise à jour des outils.

### Hors périmètre

- Réécriture fonctionnelle des composants Vue existants pour satisfaire le
  style ; les corrections seront traitées par lots ciblés.
- Remplacement de Deno ou de Vite.
- Analyse du backend Rust, déjà couverte par `cargo-audit`, `cargo-deny`,
  Clippy et les tests Rust.
- Assouplissement de la CSP Tauri pour faire passer un scan.

## Lots de travail

### FE-01 — Lint ESLint Vue/TypeScript

- Ajouter ESLint flat config, `typescript-eslint` et `eslint-plugin-vue`.
- Activer les règles recommandées et les règles TypeScript strictes utiles.
- Interdire au minimum `any` non justifié, `eval`, `new Function` et les
  imports inutilisés.
- Ajouter `deno task lint` avec `--max-warnings=0`.
- Traiter les violations existantes sans désactiver globalement les règles.

### FE-02 — Formatage et hygiène du diff

- Ajouter Prettier avec configuration versionnée.
- Ajouter `deno task format:check`.
- Ne pas mélanger un reformatage massif avec une correction fonctionnelle.

### FE-03 — Gates frontend CI

Le job frontend doit exécuter dans cet ordre :

1. `deno install --frozen`
2. `deno task lint`
3. `deno task format:check`
4. `deno task typecheck`
5. `deno task test`
6. `deno task build`

Chaque étape doit échouer la pull request. Les actions GitHub doivent rester
épinglées par SHA comme dans les workflows existants.

### FE-04 — SAST JavaScript/Vue

- Ajouter une analyse Semgrep limitée au code applicatif `src/`.
- Contrôler notamment XSS, HTML injecté, `eval`, appels réseau non prévus,
  secrets dans le code et usages dangereux des API Tauri.
- Produire un rapport SARIF ; les règles de sévérité haute bloquent la CI.
- Documenter les faux positifs avec une justification locale et datée.

### FE-05 — Dépendances et supply chain

- Conserver les versions exactes et `deno install --frozen`.
- Ajouter un audit périodique des dépendances frontend compatible avec
  `deno.lock`.
- Vérifier que le SBOM frontend contient bien les dépendances réellement
  résolues, pas uniquement un fichier vide ou un manifeste incomplet.
- Revoir toute modification de `deno.lock` comme une modification de code.

### FE-06 — Secrets et règles de dépôt

- Ajouter Gitleaks sur les commits et sur la pull request.
- Exclure uniquement les fixtures explicitement documentées.
- Ajouter une checklist de revue : pas de token, URL interne, clé privée ou
  donnée PCAP sensible dans `src/`, les tests ou les assets.

### FE-07 — Runtime et distribution

- Maintenir la CSP Tauri minimale et tester ses directives lors des builds.
- Vérifier que les workers nécessaires à Sigma/ForceAtlas2 restent autorisés
  sans autoriser de scripts distants.
- Conserver les SBOM, hashes, attestations et scans Trivy des artefacts de
  release.

## Critères d'acceptation

- Une pull request frontend échoue si ESLint, Prettier, `vue-tsc`, les tests ou
  le build échoue.
- Aucun warning ESLint n'est ignoré globalement.
- Une vulnérabilité SAST haute ou un secret détecté bloque la CI.
- Les exceptions sont limitées, commentées et traçables.
- Le lockfile reste obligatoire et l'audit frontend est reproductible.
- Le rapport SARIF est consultable dans GitHub Security.
- Le bundle produit par Vite reste compatible avec le build Tauri.
- La CSP et les Web Workers Sigma sont vérifiés après le durcissement.

## Ordre recommandé

1. FE-01 et FE-02 sur une branche dédiée.
2. FE-03 après stabilisation des commandes locales.
3. FE-05 et FE-06, qui peuvent bloquer des problèmes indépendants du code.
4. FE-04 avec un périmètre initial limité et revue des faux positifs.
5. FE-07 et validation sur un build Windows de release.

## Commandes locales cibles

```bash
deno install --frozen
deno task lint
deno task format:check
deno task typecheck
deno task test
deno task build
```

## Risques

- Un lint activé d'un seul coup peut produire trop de bruit ; commencer par
  les règles recommandées et corriger par lots.
- Un audit frontend peut détecter des CVE transitives sans correctif
  immédiat ; toute exception doit avoir une échéance et une justification.
- Un SAST trop large peut générer des faux positifs et être contourné ; le
  périmètre doit rester ciblé et les résultats doivent être revus.
- Une CSP trop permissive pour faire fonctionner une librairie annulerait le
  bénéfice du durcissement ; adapter la librairie ou la directive minimale.
