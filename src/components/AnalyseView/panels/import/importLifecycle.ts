/**
 * Libère l'état occupé avant toute opération asynchrone de rafraîchissement.
 * Ainsi, même si la vérification backend échoue après un import refusé,
 * l'overlay et le verrou global ne peuvent pas rester actifs (#167).
 */
export async function finalizeImportState(
  releaseBusy: () => void,
  refreshHasData: () => Promise<void>,
  reportError: (error: unknown) => void | Promise<void>,
): Promise<void> {
  releaseBusy();
  try {
    await refreshHasData();
  } catch (error) {
    await reportError(error);
  }
}
