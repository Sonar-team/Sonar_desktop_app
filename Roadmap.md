# 🎯 **Objectif général**
Améliorer l’utilisabilité, la précision, et l’efficacité de SONAR en introduisant des fonctions clés de manipulation et d’analyse de matrices de flux réseau.

---

## 📅 **Roadmap SONAR - 2025 à 2027**

### ✅ **2025 – Consolidation fonctionnelle**

**Q2–Q3 2025**
- [ ] **Ajout de la fonction `stop record`** :  
  Permettre d'arrêter manuellement une capture réseau en cours (grâce à une interface bouton ou signal externe).
  - Implémentation dans l'interface utilisateur
  - Synchronisation avec l’état de la machine à états Tauri/Rust
  - Logging de l’arrêt avec horodatage

**Q4 2025**
- [ ] **Ajout d’une fonction de tri dans la vue matrice**  
  - Tri par volume, par IP source, par IP destination, par VLAN, etc.
  - UI réactive et sortable sur toutes les colonnes pertinentes

---

### 🧪 **2026 – Interaction avancée avec les matrices**

**S1 2026**
- [ ] **Ajout de la fonction d’édition de matrice**  
  - Modifier manuellement une cellule, une ligne, ou fusionner/supprimer des entrées
  - Validation des modifications (limites, types de données)
  - Ajout d’un mode “édition sécurisée” avec rollback ou confirmation

**S2 2026**
- [ ] **Interface de fusion de matrices de flux**  
  Objectif : fusionner plusieurs relevés (ex : plusieurs PCAP) pour obtenir une vision agrégée.
  - UI de sélection et import de plusieurs matrices
  - Règles de fusion personnalisables (par IP, VLAN, protocole, etc.)
  - Visualisation des conflits / doublons

---

### 🚀 **2027 – Industrialisation et UX finale**

**S1 2027**
- [ ] **Refonte UX des modules de tri, édition, et fusion**
  - Interface graphique simplifiée, design cohérent, adaptatif
  - Ajout d’aides contextuelles
  - Tests utilisateurs (si possible avec marins non techniques)

**S2 2027**
- [ ] **Automatisation des tâches répétitives**
  - Pré-fusion automatique de matrices proches
  - Tri par défaut configurable
  - Templates de configuration d’édition

