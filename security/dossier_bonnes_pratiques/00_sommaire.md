# Sécurisation de la chaîne d'approvisionnement logicielle d'une application open source

## Bonnes pratiques DevSecOps appliquées à une application Tauri développée sur GitHub

**Dossier de présentation — retour d'expérience du projet SONAR**

---

## Sommaire du dossier

| Fichier | Contenu |
|---------|---------|
| `00_sommaire.md` | Le présent sommaire et l'objet du dossier |
| `01_introduction.md` | Contexte, menace supply chain, études de cas SUNBURST (SolarWinds) et XZ Utils |
| `02_analyse_risques.md` | Analyse de risques d'une application Tauri open source non sécurisée |
| `03_scenarios_attaque.md` | Scénarios d'attaque opérationnels détaillés |
| `04_solutions.md` | Contre-mesures : bonnes pratiques mises en œuvre et preuves de faisabilité |
| `05_synthese_recommandations.md` | Synthèse, trajectoire de mise en conformité, checklists et glossaire |

---

## Objet du dossier

Ce dossier présente une démarche complète de sécurisation de la chaîne
d'approvisionnement logicielle (*software supply chain*) d'une application de
bureau open source, développée publiquement sur GitHub avec le framework
Tauri (frontend web, backend Rust).

Il s'appuie sur deux compromissions majeures documentées — **SUNBURST**
(SolarWinds, 2020) et **XZ Utils** (2024) — pour établir une analyse de
risques d'une application non sécurisée, dérouler les scénarios d'attaque
correspondants, et présenter pour chacun les contre-mesures effectivement
déployées sur le projet SONAR (analyseur de trafic réseau open source),
qui sert ici de démonstrateur.

La démarche vise trois objectifs :

1. **Comprendre** la menace supply chain telle qu'elle s'exerce sur un projet
   open source hébergé sur une forge publique ;
2. **Évaluer** les risques propres à une application Tauri, qui cumule trois
   écosystèmes de dépendances (npm/Deno, crates.io, paquets système) ;
3. **Démontrer** que des contre-mesures d'un niveau comparable aux exigences
   des référentiels publics (recommandations ANSSI, cadre SLSA, NIST SSDF)
   sont réalisables sur un projet open source à effectifs réduits, avec des
   outils libres et gratuits.

## Périmètre

- **Inclus** : dépendances applicatives, environnement et outillage de
  build, intégration continue (GitHub Actions), production et distribution
  des artefacts de release, vérifiabilité par un tiers.
- **Exclus** : sécurité du poste utilisateur final, sécurité réseau de
  l'infrastructure d'hébergement GitHub, analyse du code métier de
  l'application.
