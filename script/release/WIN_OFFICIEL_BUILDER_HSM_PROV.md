# Build Windows officiel avec HSM et provenance

Cette procédure décrit l’utilisation de
`script/release/win_officiel_builder_hsm_prov.ps1` sur le poste Windows sécurisé
de release.

Le script construit uniquement l’installateur NSIS de SONAR, signe les
exécutables avec le certificat Authenticode de l’entreprise, génère une
provenance SLSA v1 et la signe avec une seconde clé gérée par Cosign. Il ne
cherche pas à produire un build reproductible : la preuve porte sur l’artefact
réellement construit et publié.

## 1. Résultat attendu

Une exécution réussie produit un kit hors du dépôt :

```text
<dossier-de-sortie>/
├── artifacts/
│   └── sonar_<version>_x64-setup.exe
├── provenance/
│   ├── sonar-windows-nsis-provenance.slsa-v1.json
│   └── <installateur>.provenance.sigstore.json
├── signatures/
│   ├── <installateur>.sigstore.json
│   └── SHA256SUMS.sigstore.json
├── trust/
│   ├── cosign-sonar.pub
│   └── cosign-tsa-chain.pem       # seulement si demandé
├── logs/
│   └── build.log
├── metadata.json
├── SHA256SUMS
└── VERIFY.txt
```

L’installateur final possède donc deux protections complémentaires :

- une signature native Authenticode, reconnue par Windows et horodatée ;
- une signature Cosign et une provenance SLSA liées à son SHA-256 exact.

Le manifeste `SHA256SUMS` est lui-même signé. Le journal `logs/build.log`, qui
continue d’évoluer jusqu’à la fin du script, est volontairement exclu du
manifeste signé et doit être traité comme une trace d’audit séparée.

## 2. Prérequis du poste Windows

Le poste de release doit disposer de :

- Windows x86_64 et PowerShell 7.2 ou plus récent ;
- Git, Node.js, Deno, Rust/Cargo, Tauri CLI et Cosign ;
- Windows SDK avec `signtool.exe` x64 ;
- 7-Zip ;
- le middleware du HSM Authenticode ;
- l’accès au HSM ou à la clé matérielle utilisée par Cosign ;
- l’accès à un serveur d’horodatage RFC 3161 ;
- les autorités de certification de l’entreprise et du serveur d’horodatage
  installées dans les magasins de confiance Windows.

Les versions Node, Deno, Rust, Tauri et Cosign doivent être exactement celles de
`config/build-versions.env`. Le script les vérifie avant de compiler.

Sur un poste sans accès à Internet, les dépendances Deno, Cargo, Tauri et NSIS
doivent déjà être présentes dans les caches du compte de build. Le HSM/KMS et le
serveur d’horodatage doivent être locaux ou accessibles sur le réseau interne.
Une URL de TSA publique ne fonctionnera pas depuis un poste totalement isolé.

## 3. Préparer les identités de signature

### 3.1 Certificat Authenticode

Le certificat de signature de code doit être installé dans l’un de ces
magasins :

- `Cert:\CurrentUser\My` ;
- `Cert:\LocalMachine\My`.

Il doit être valide, autoriser l’usage « signature de code » et être relié à la
clé privée conservée dans le HSM. Pour afficher les certificats disponibles :

```powershell
Get-ChildItem Cert:\LocalMachine\My |
  Select-Object Subject, Thumbprint, NotBefore, NotAfter, HasPrivateKey
```

Conserver l’empreinte SHA-1 sans espace :

```powershell
$authenticodeThumbprint = "0123456789ABCDEF0123456789ABCDEF01234567"
Get-Item "Cert:\LocalMachine\My\$authenticodeThumbprint" |
  Format-List Subject, Issuer, Thumbprint, NotBefore, NotAfter, HasPrivateKey
```

Lorsque le middleware relie correctement le certificat à sa clé, l’empreinte et
le magasin suffisent. Certains HSM imposent aussi le nom du CSP/KSP et celui du
conteneur de clé. Dans ce cas, ajouter ensemble :

```powershell
-SignToolCsp "Nom du fournisseur HSM" `
-SignToolKeyContainer "Nom du conteneur"
```

Ces deux valeurs viennent de la configuration du middleware HSM. Elles ne
doivent pas être inventées ou remplacées par un chemin de clé privée.

### 3.2 Clé de provenance Cosign

La provenance utilise une clé distincte. Deux modes sont possibles.

Mode HSM/KMS par référence :

```powershell
-CosignKeyReference "azurekms://coffre/cle-sonar-provenance"
```

Cosign accepte notamment les références `azurekms://`, `awskms://`,
`gcpkms://`, `hashivault://` et, avec le fournisseur adapté, `pkcs11:`. Le
script refuse un chemin vers une clé privée locale.

Mode clé matérielle PIV compatible Cosign :

```powershell
-CosignSecurityKey
```

Dans les deux cas, exporter uniquement la clé publique dans un emplacement de
confiance :

```powershell
-CosignPublicKey "C:\SONAR-TRUST\cosign-sonar.pub"
```

Cette clé publique doit aussi être remise aux vérificateurs par un canal séparé
du kit de release. Copier une clé publique uniquement depuis le kit ne permet
pas d’établir sa confiance initiale.

## 4. Préparer la source approuvée

Le script exige :

- un tag au format `vX.Y.Z` ;
- la même version dans le tag, `package.json` et
  `src-tauri/tauri.conf.json` ;
- un `HEAD` exactement égal au commit approuvé ;
- un tag signé et vérifiable avec `git tag -v` ;
- aucun fichier modifié ou non suivi.

Depuis une copie fraîche du dépôt :

```powershell
Set-Location C:\SONAR-BUILD\Sonar_desktop_app

$tag = "v4.8.3"
$expectedCommit = "0123456789abcdef0123456789abcdef01234567"

git tag -v $tag
git switch --detach $tag
git rev-parse HEAD
git status --short
```

La valeur de `$expectedCommit` doit venir de la validation de release ou d’un
canal indépendant, pas uniquement du dépôt présent sur la machine. Les sorties
de `git rev-parse HEAD` et `git status --short` doivent respectivement être le
commit approuvé et une sortie vide.

Par défaut, un tag non signé arrête le build. `-AllowUnsignedTag` existe
uniquement comme dérogation documentée ; il ne doit pas être utilisé pour une
release officielle normale.

## 5. Lancer la commande

### 5.1 Avec une référence HSM/KMS Cosign

```powershell
Set-Location C:\SONAR-BUILD\Sonar_desktop_app

$tag = "v4.8.3"
$expectedCommit = "0123456789abcdef0123456789abcdef01234567"
$authenticodeThumbprint = "89ABCDEF0123456789ABCDEF0123456789ABCDEF"
$releaseDirectory = "C:\SONAR-RELEASES\$tag"

.\script\release\win_officiel_builder_hsm_prov.ps1 `
  -ReleaseTag $tag `
  -ExpectedCommit $expectedCommit `
  -AuthenticodeThumbprint $authenticodeThumbprint `
  -CertificateStore LocalMachine `
  -TimestampUrl "https://tsa.entreprise.example/rfc3161" `
  -CosignKeyReference "azurekms://coffre/cle-sonar-provenance" `
  -CosignPublicKey "C:\SONAR-TRUST\cosign-sonar.pub" `
  -OutputDirectory $releaseDirectory
```

### 5.2 Avec une clé matérielle Cosign

Remplacer `-CosignKeyReference` par `-CosignSecurityKey` :

```powershell
.\script\release\win_officiel_builder_hsm_prov.ps1 `
  -ReleaseTag $tag `
  -ExpectedCommit $expectedCommit `
  -AuthenticodeThumbprint $authenticodeThumbprint `
  -CertificateStore LocalMachine `
  -TimestampUrl "https://tsa.entreprise.example/rfc3161" `
  -CosignSecurityKey `
  -CosignPublicKey "C:\SONAR-TRUST\cosign-sonar.pub" `
  -OutputDirectory $releaseDirectory
```

Le dossier de sortie doit être absolu, extérieur au dépôt et vide ou inexistant.
En cas d’échec, le kit partiel est conservé pour l’analyse. Utiliser un nouveau
dossier vide pour la tentative suivante après avoir archivé ou supprimé le kit
partiel selon la procédure interne.

Le HSM peut demander son PIN plusieurs fois : lors du préflight Cosign, de la
signature de l’exécutable, de la création de l’installateur, de la provenance et
du manifeste. Saisir le PIN uniquement dans l’interface sécurisée du middleware
ou du token. Ne jamais le passer en paramètre, en variable d’environnement ou
dans un fichier du dépôt.

### 5.3 Paramètres complémentaires

| Paramètre | Usage |
| --- | --- |
| `-CertificateStore CurrentUser` | Sélectionne le magasin du compte de build ; c’est la valeur par défaut |
| `-CertificateStore LocalMachine` | Sélectionne le magasin de la machine et ajoute `/sm` à SignTool |
| `-SignToolCsp` et `-SignToolKeyContainer` | Indiquent explicitement le fournisseur et le conteneur HSM ; toujours les fournir ensemble |
| `-SourceRepositoryUri` | Fixe l’URI inscrite dans la provenance quand elle ne doit pas être déduite du remote Git |
| `-BuilderId` | Remplace l’identité SLSA par défaut `urn:sonar:builder:windows-hsm:v1` |
| `-UseLegacyAuthenticodeTimestamp` | Utilise l’ancien protocole Authenticode `/t` au lieu de RFC 3161 ; uniquement pour une TSA interne ancienne |
| `-AllowUnsignedTag` | Dérogation permettant un tag non signé ; interdite dans le processus officiel normal |

Pour afficher l’aide intégrée et tous les paramètres :

```powershell
Get-Help .\script\release\win_officiel_builder_hsm_prov.ps1 -Full
```

## 6. Horodatage Cosign facultatif

La signature Authenticode est toujours horodatée. Pour horodater aussi les
preuves Cosign, fournir ensemble l’URL RFC 3161 et sa chaîne de certificats :

```powershell
-CosignTimestampUrl "https://tsa.entreprise.example/rfc3161" `
-CosignTimestampCertificateChain "C:\SONAR-TRUST\tsa-chain.pem"
```

Le script copie la chaîne dans le kit puis exige que l’horodatage Cosign soit
validé. Ne fournir qu’un seul de ces deux paramètres provoque un arrêt.

## 7. Contrôles réalisés automatiquement

Avant de produire le kit, le script :

1. vérifie le système Windows, les outils et leurs versions ;
2. vérifie le tag, le commit, les versions applicatives et la propreté Git ;
3. vérifie le certificat Authenticode et son usage de signature de code ;
4. signe puis vérifie un petit fichier de préflight avec la clé Cosign ;
5. exécute `deno install --frozen`, le typecheck, le lint et les tests ;
6. exécute les tests Rust avec `cargo test --locked` ;
7. construit seulement l’installateur NSIS dans un répertoire temporaire ;
8. vérifie à nouveau le commit et la propreté Git après le build ;
9. contrôle la structure PE, l’import de `wpcap.dll` et l’absence d’un
   installateur Npcap/WinPcap embarqué ;
10. vérifie la signature Authenticode, l’empreinte du signataire et
    l’horodatage de l’exécutable et de l’installateur ;
11. signe l’installateur, la provenance SLSA et le manifeste avec Cosign ;
12. vérifie immédiatement toutes les preuves avec la clé publique fournie.

Une étape en erreur arrête le processus. Aucune provenance n’est signée si le
build a modifié la source approuvée.

## 8. Vérifier le kit avant publication

Depuis la racine du kit, suivre `VERIFY.txt`. L’ordre important est :

1. comparer `trust/cosign-sonar.pub` avec la clé publique de référence obtenue
   par un canal indépendant ;
2. vérifier la signature Cosign de `SHA256SUMS` ;
3. recalculer les empreintes des fichiers listés ;
4. vérifier Authenticode avec SignTool ;
5. vérifier la signature Cosign de l’installateur ;
6. vérifier l’attestation SLSA et son sujet.

Exemples principaux :

```powershell
Set-Location C:\SONAR-RELEASES\v4.8.3

cosign verify-blob `
  --private-infrastructure `
  --key "trust\cosign-sonar.pub" `
  --bundle "signatures\SHA256SUMS.sigstore.json" `
  "SHA256SUMS"

signtool verify /pa /all /v /tw "artifacts\sonar_4.8.3_x64-setup.exe"

cosign verify-blob-attestation `
  --private-infrastructure `
  --key "trust\cosign-sonar.pub" `
  --bundle "provenance\sonar_4.8.3_x64-setup.exe.provenance.sigstore.json" `
  --type slsaprovenance1 `
  "artifacts\sonar_4.8.3_x64-setup.exe"
```

Utiliser les noms réels indiqués dans `VERIFY.txt` et `metadata.json`. Si
l’horodatage Cosign a été activé, `VERIFY.txt` ajoute automatiquement les
paramètres nécessaires à sa vérification.

## 9. Diagnostic des erreurs courantes

| Erreur | Cause probable | Action |
| --- | --- | --- |
| Le script exige Windows | Commande lancée sur Linux ou macOS | Utiliser le poste Windows officiel |
| Dépôt non propre | Fichier modifié ou non suivi | Identifier la différence ; ne pas la masquer avant validation |
| Échec de `git tag -v` | Tag non signé ou clé du signataire absente | Importer la clé de confiance ou refaire le tag selon la procédure |
| Version d’outil incorrecte | Poste non aligné sur `build-versions.env` | Installer la version exacte attendue |
| Certificat introuvable | Mauvais magasin ou mauvaise empreinte | Contrôler `CertificateStore` et l’empreinte sans espace |
| Aucune clé privée | Middleware HSM absent ou association certificat/clé incorrecte | Corriger le middleware ; utiliser CSP/conteneur si requis |
| Échec SignTool | HSM verrouillé, PIN refusé, TSA inaccessible ou chaîne non fiable | Contrôler le token, le réseau interne, les CA et la révocation |
| Échec du préflight Cosign | Mauvaise référence, token absent ou clé publique non correspondante | Corriger la référence et comparer la clé publique de confiance |
| Source modifiée après build | Lockfile ou fichier suivi réécrit pendant la compilation | Examiner le diff ; ne pas publier le kit partiel |
| Dossier de sortie non vide | Ancienne tentative présente | Archiver la tentative puis choisir un nouveau dossier vide |

## 10. Checklist de publication

- [ ] Le tag signé et le commit ont été approuvés par une seconde personne.
- [ ] Le dépôt était propre avant et après le build.
- [ ] Les tests du script sont tous réussis.
- [ ] L’empreinte Authenticode correspond au certificat officiel de
      l’entreprise.
- [ ] La clé publique Cosign correspond à la copie de confiance indépendante.
- [ ] La signature du manifeste, les empreintes, Authenticode et la provenance
      ont été vérifiés depuis le kit final.
- [ ] `metadata.json` contient le tag et le commit attendus.
- [ ] Le kit complet est archivé et publié ; l’installateur n’est pas diffusé
      seul sans ses preuves.
- [ ] Le journal de build est conservé selon la politique interne et son accès
      est limité, car il peut contenir des chemins locaux et l’identité du
      certificat.
