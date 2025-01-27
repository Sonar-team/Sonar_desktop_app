# Procédure d'Exploitation de SONAR

## 1. Introduction
Cette procédure d'exploitation décrit les étapes nécessaires pour l'utilisation correcte du logiciel **SONAR**, un outil de relevé de matrice de flux réseau. Elle s'adresse aux opérateurs et analystes réseau chargés de réaliser des audits et des relevés sur l'infrastructure informatique.

## 2. Prérequis
Avant d'utiliser SONAR, assurez-vous de disposer des éléments suivants :
- Un poste de travail compatible (Windows 10,11/Linux/macOS)
- Une connexion réseau 
- Les interfaces reseaux activées
- Des droits d'accès suffisants sur le réseau
- Le logiciel SONAR installé
- Le le moteur de capture reseau Npcap soit installé sis votre pc est sous windows
- Un stockage suffisant pour les fichiers de relevés

## 3. Démarrage de SONAR
1. **Lancer l'application** :
   - Sous Windows : Double-cliquer sur l'exécutable `SONAR.exe`
   - Sous Linux/macOS : Exécuter `./sonar` depuis le terminal
2. **Se connecter** :
   - Si requis, entrez vos identifiants de connexion
3. **Vérifier la configuration** :
   - Accéder aux paramètres pour vérifier les interfaces réseau surveillées
   - Configurer les filtres si besoin

## 4. Réalisation d'un Relevé
1. **Sélectionner les interfaces réseau**
2. **Démarrer la capture des paquets**
3. **Laisser tourner la collecte selon le besoin**
4. **Arrêter la capture et sauvegarder les données**
5. **Exporter les résultats** sous le format souhaité (CSV, PCAP, JSON, etc.)

## 5. Analyse des Résultats
1. **Charger un relevé enregistré**
2. **Visualiser la matrice de flux**
3. **Filtrer les données selon les critères d'analyse** (source, destination, protocole, etc.)
4. **Générer un rapport** si nécessaire

## 6. Archivage et Gestion des Données
1. **Nommer et organiser les fichiers de relevés**
2. **Stocker les relevés sur un serveur ou une base de données**
3. **Assurer la protection des données sensibles** (chiffrement, accès restreint)

## 7. Maintenance et Mise à Jour
1. **Vérifier régulièrement les mises à jour de SONAR**
2. **Effectuer des tests de bon fonctionnement périodiques**
3. **Consulter la documentation en cas de problème**

## 8. Support et Contact
- Pour toute question ou problème technique, contactez l'équipe de support via [adresse email] ou [plateforme de support].

