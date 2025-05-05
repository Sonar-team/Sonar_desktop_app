# 🎯 **Objectif général**
Améliorer l’utilisabilité, la précision, et l’efficacité de SONAR en introduisant des fonctions clés de manipulation et d’analyse de matrices de flux réseau.

---

## 📅 **Roadmap SONAR - 2025 à 2027**

### ✅ **2025 – Consolidation fonctionnelle**

**2.2**
- [ ] **Ajout de la fonction `stop record` `pause record`** :  
  Permettre d'arrêter manuellement une capture réseau en cours (grâce à une interface bouton ou signal externe).
  - Implémentation dans l'interface utilisateur
  - Synchronisation avec l’état de la machine à états Tauri/Rust avec pinia

**2.3**
- [ ] **Update d’une fonction de tri dans la vue matrice rm vuetify**  
  - Tri par volume, par IP source, par IP destination, par VLAN, etc.
  - UI réactive et sortable sur toutes les colonnes pertinentes

---

### 🧪 **2026 – Interaction avancée avec les matrices**

**2.4**
- [ ] **Update import Pcap/csv/excel**  
  - import générique de matices

**2.5**
- [ ] **Ajout de la fonction d’édition de matrice**  
  - Modifier manuellement une cellule, une ligne, ou fusionner/supprimer des entrées
  - Validation des modifications (limites, types de données)
  - Ajout d’un mode “édition sécurisée” avec rollback ou confirmation

**2.6**
- [ ] **Interface de fusion de matrices de flux**  
  Objectif : fusionner plusieurs relevés (ex : plusieurs PCAP) pour obtenir une vision agrégée.
  - UI de sélection et import de plusieurs matrices
  - Règles de fusion personnalisables (par IP, VLAN, protocole, etc.)
  - Visualisation des conflits / doublons

---