// Classification des erreurs bloquantes d'import de labels : transforme
// l'erreur brute du backend en un cas typé que le panneau d'import peut
// afficher dans son dialogue de conflits, ou `null` si l'erreur n'est pas
// spécifique aux labels (à afficher via `displayCaptureError`).
import {
  CaptureStateErrorKind,
  InvalidFieldValue,
  InvalidLineValue,
  LabelConflictRow,
  LabelErrorKind,
} from '../errors/capture';

export type LabelFileIssues =
  | { kind: 'invalidMacIpFormat'; invalidMac: InvalidFieldValue[]; invalidIp: InvalidFieldValue[] }
  | { kind: 'labelLinesConflicts'; sameIpDiffMac: LabelConflictRow[]; sameIpDiffLabel: LabelConflictRow[] }
  | { kind: 'invalidRowsFormat'; invalidLines: InvalidLineValue[] };

export function classifyLabelImportError(err: unknown): LabelFileIssues | null {
  const error = err as CaptureStateErrorKind;
  if (!error || typeof error !== 'object' || error.kind !== 'label') return null;

  const labelError = error.message as LabelErrorKind;
  switch (labelError.kind) {
    case 'invalidMacIpFormat': {
      const [invalidMac, invalidIp] = labelError.message;
      return { kind: 'invalidMacIpFormat', invalidMac, invalidIp };
    }
    case 'labelLinesConflicts': {
      const [sameIpDiffMac, sameIpDiffLabel] = labelError.message;
      return { kind: 'labelLinesConflicts', sameIpDiffMac, sameIpDiffLabel };
    }
    case 'invalidRowsFormat':
      return { kind: 'invalidRowsFormat', invalidLines: labelError.message };
    default:
      return null;
  }
}
