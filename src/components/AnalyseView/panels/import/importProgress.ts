// Calculs dérivés de la progression d'import (ImportProgress) affichée dans
// l'overlay de conversion : pourcentage global toutes-pièces et nom du
// fichier en cours.
import type { ImportProgress } from '../../../../types/capture';

export function progressPercentOf(progress: ImportProgress | null): number {
  if (!progress || progress.filesTotal <= 0) return 0;
  const fileRatio = progress.total > 0
    ? Math.min(1, Math.max(0, progress.current / progress.total))
    : 1;
  const globalRatio = (progress.fileIndex - 1 + fileRatio) / progress.filesTotal;
  return Math.min(100, Math.max(0, globalRatio * 100));
}

export function progressFileNameOf(progress: ImportProgress | null): string {
  if (!progress) return '';
  return progress.fileName.split(/[\\/]/).pop() ?? progress.fileName;
}
