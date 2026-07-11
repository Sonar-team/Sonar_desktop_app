//! Stores de labels : lignes `(mac, ip, label)` gérées par l'utilisateur
//! ([`LabelStore`]) et labels générés au démarrage pour la machine de capture
//! ([`PcInfoLabel`]).

use serde::Serialize;

/// Source de vérité unique des labels utilisateur (#157) : imports de
/// fichiers, éditions du panneau de gestion, arbitrages. Le miroir de
/// lecture de la matrice (`FlowMatrix::label`, chemin chaud du processing)
/// est resynchronisé depuis ce store à chaque mutation
/// (`setup::labels::resync_labels`).
#[derive(Serialize, Default, Debug)]
pub struct LabelStore {
    pub rows: Vec<(String, String, String)>,
}

impl LabelStore {
    pub fn new() -> Self {
        LabelStore { rows: Vec::new() }
    }

    /// Ajoute une ligne `(mac, ip, label)` (sans dédoublonnage : l'import
    /// déduplique en amont).
    pub fn add(&mut self, row: (String, String, String)) {
        self.rows.push(row)
    }

    /// Toutes les lignes du store, dans l'ordre d'import.
    pub fn get(&self) -> &Vec<(String, String, String)> {
        &self.rows
    }

    /// Fixe le label pour la clé `(mac, ip)` : met à jour la ligne existante,
    /// ou l'ajoute si absente. Utilisé par l'arbitrage des conflits.
    pub fn set(&mut self, mac: &str, ip: &str, label: &str) {
        if let Some(row) = self.rows.iter_mut().find(|r| r.0 == mac && r.1 == ip) {
            row.2 = label.to_string();
        } else {
            self.rows
                .push((mac.to_string(), ip.to_string(), label.to_string()));
        }
    }

    /// Supprime la ligne de clé `(mac, ip)`. Retourne vrai si elle existait.
    pub fn remove(&mut self, mac: &str, ip: &str) -> bool {
        let before = self.rows.len();
        self.rows.retain(|r| !(r.0 == mac && r.1 == ip));
        self.rows.len() != before
    }

    /// Modifie une ligne existante, clé comprise. Refuse si la ligne
    /// d'origine n'existe pas, ou si la nouvelle clé écraserait une autre
    /// ligne (l'UI doit arbitrer explicitement, pas d'écrasement silencieux).
    pub fn update(
        &mut self,
        old_mac: &str,
        old_ip: &str,
        mac: &str,
        ip: &str,
        label: &str,
    ) -> Result<(), String> {
        if !self.rows.iter().any(|r| r.0 == old_mac && r.1 == old_ip) {
            return Err(format!("label introuvable pour ({old_mac}, {old_ip})"));
        }
        let key_changes = old_mac != mac || old_ip != ip;
        if key_changes && self.rows.iter().any(|r| r.0 == mac && r.1 == ip) {
            return Err(format!(
                "un label existe déjà pour ({mac}, {ip}) : supprimez-le ou modifiez-le"
            ));
        }
        if let Some(row) = self
            .rows
            .iter_mut()
            .find(|r| r.0 == old_mac && r.1 == old_ip)
        {
            *row = (mac.to_string(), ip.to_string(), label.to_string());
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        self.rows.clear()
    }
}

/// Lignes de labels « pc sonar » générées au démarrage depuis les interfaces
/// réseau locales (voir `setup::labels`).
pub struct PcInfoLabel {
    pub label_lines: Vec<String>,
}

impl PcInfoLabel {
    pub fn new() -> Self {
        PcInfoLabel {
            label_lines: Vec::new(),
        }
    }

    pub fn get_label(&self) -> &Vec<String> {
        &self.label_lines
    }

    pub fn push(&mut self, label_line: String) {
        self.label_lines.push(label_line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store_with(rows: &[(&str, &str, &str)]) -> LabelStore {
        let mut store = LabelStore::new();
        for (mac, ip, label) in rows {
            store.add((mac.to_string(), ip.to_string(), label.to_string()));
        }
        store
    }

    #[test]
    fn remove_deletes_only_the_matching_key() {
        let mut store = store_with(&[
            ("aa:bb:cc:dd:ee:ff", "192.168.1.10", "Automate"),
            ("aa:bb:cc:dd:ee:00", "192.168.1.11", "Supervision"),
        ]);

        assert!(store.remove("aa:bb:cc:dd:ee:ff", "192.168.1.10"));
        assert!(
            !store.remove("aa:bb:cc:dd:ee:ff", "192.168.1.10"),
            "déjà supprimé"
        );
        assert_eq!(store.get().len(), 1);
        assert_eq!(store.get()[0].2, "Supervision");
    }

    #[test]
    fn update_edits_in_place_and_refuses_collisions() {
        let mut store = store_with(&[
            ("aa:bb:cc:dd:ee:ff", "192.168.1.10", "Automate"),
            ("aa:bb:cc:dd:ee:00", "192.168.1.11", "Supervision"),
        ]);

        // Édition simple (même clé).
        store
            .update(
                "aa:bb:cc:dd:ee:ff",
                "192.168.1.10",
                "aa:bb:cc:dd:ee:ff",
                "192.168.1.10",
                "Automate four",
            )
            .unwrap();
        assert_eq!(store.get()[0].2, "Automate four");

        // Changement de clé vers une clé libre.
        store
            .update(
                "aa:bb:cc:dd:ee:ff",
                "192.168.1.10",
                "aa:bb:cc:dd:ee:ff",
                "192.168.1.12",
                "Automate four",
            )
            .unwrap();
        assert_eq!(store.get()[0].1, "192.168.1.12");

        // Collision avec une autre ligne : refusée, rien n'est modifié.
        let err = store
            .update(
                "aa:bb:cc:dd:ee:ff",
                "192.168.1.12",
                "aa:bb:cc:dd:ee:00",
                "192.168.1.11",
                "Doublon",
            )
            .unwrap_err();
        assert!(err.contains("existe déjà"));
        assert_eq!(store.get()[0].1, "192.168.1.12", "ligne d'origine intacte");

        // Ligne d'origine inexistante : refusée.
        assert!(
            store
                .update(
                    "00:00:00:00:00:01",
                    "10.0.0.1",
                    "00:00:00:00:00:01",
                    "10.0.0.1",
                    "X"
                )
                .unwrap_err()
                .contains("introuvable")
        );
    }
}
