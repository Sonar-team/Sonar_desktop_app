// Types des rapports d'import de labels (miroir des structs Rust de
// `commandes::import::labels`).

/** Un choix de label en compétition sur une même clé : (n° de ligne, label, ligne brute). */
export type LabelChoice = [number, string, string];

/**
 * Conflit résolu automatiquement à l'import : une même clé (mac, ip) portait
 * plusieurs labels. Le premier est gardé, les autres attendent l'arbitrage.
 */
export type LabelConflictReport = {
  mac: string;
  ip: string;
  kept: LabelChoice;
  dropped: LabelChoice[];
};

/** Résultat d'un import de labels : labels appliqués et conflits résolus. */
export type LabelImportReport = {
  applied: number;
  conflicts: LabelConflictReport[];
};
