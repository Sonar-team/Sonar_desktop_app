// Filtrage plein-texte de la table des labels importés : recherche
// insensible à la casse sur les champs mac/ip/label de chaque ligne.
export function filterLabelRows(
  rows: [string, string, string][],
  search: string
): [string, string, string][] {
  const needle = search.toLowerCase();
  return rows.filter((row) => row.some((field) => field.toLowerCase().includes(needle)));
}
