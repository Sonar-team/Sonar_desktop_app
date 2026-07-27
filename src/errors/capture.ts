// Affichage utilisateur des erreurs backend : chaque erreur est traduite en
// message lisible, montrée en dialogue et journalisée. Les types sont le
// contrat IPC GÉNÉRÉ depuis les enums Rust (#142, `cargo test
// export_ipc_bindings`) — ne pas les redéclarer à la main ici : toute dérive
// doit être une erreur de compilation, pas un `undefined` en production.
import { message } from "@tauri-apps/plugin-dialog";
import { error } from "@tauri-apps/plugin-log";

import type { CaptureErrorKind } from "../types/generated/CaptureErrorKind";
import type { CaptureStateErrorKind } from "../types/generated/CaptureStateErrorKind";
import type { ExportErrorKind } from "../types/generated/ExportErrorKind";
import type { LabelErrorKind } from "../types/generated/LabelErrorKind";
import type { PcapImportErrorKind as ImportErrorKind } from "../types/generated/PcapImportErrorKind";

export type {
  CaptureErrorKind,
  CaptureStateErrorKind,
  ExportErrorKind,
  ImportErrorKind,
  LabelErrorKind,
};

// Alias lisibles des tuples du contrat labels (les types générés les
// inlinent) : consommés par `labelImport.ts` et le panneau d'import.
export type InvalidLineValue = [number, string];
export type InvalidFieldValue = [number, string, string];
export type LabelConflictRow = [number, number, string, string, string, string, string];

/** Rend lisible une valeur d'erreur hors contrat (chaîne, null, Error…)
 *  sans jamais lever : c'est le dernier filet avant l'affichage (#161). */
function stringifyUnknown(err: unknown): string {
  if (typeof err === "string") return err;
  if (err instanceof Error) return err.message;
  try {
    return JSON.stringify(err) ?? String(err);
  } catch {
    return String(err);
  }
}

export async function displayCaptureError(err: unknown) {
  // Garde objet/null : `"kind" in err` lève un TypeError sur une valeur non
  // objet et masquerait l'erreur d'origine par une exception secondaire (#161).
  if (typeof err !== "object" || err === null || !("kind" in err)) {
    const userFriendlyMessage = `Erreur inattendue : ${stringifyUnknown(err)}`;
    await message(userFriendlyMessage, {
      title: "Erreur Capture (inattendue)",
      kind: "error",
    });
    error(`Erreur Capture (inattendue) : ${userFriendlyMessage}`);
    return;
  }

  const captureError = err as CaptureStateErrorKind;
  let userFriendlyMessage = "Erreur inconnue";

  if ("kind" in captureError) {
    switch (captureError.kind) {
      case "io":
        userFriendlyMessage = `Erreur IO : ${captureError.message}`;
        break;
      case "poisonError":
        userFriendlyMessage = `Erreur verrou : ${captureError.message}`;
        break;
      case "invalidTransition":
        userFriendlyMessage =
          `Opération refusée : ${captureError.message}`;
        break;
      case "capture": {
        const captureKind = captureError.message as CaptureErrorKind;
        if ("kind" in captureKind) {
          switch (captureKind.kind) {
            case "invalidConfig":
              userFriendlyMessage =
                `Configuration invalide : ${captureKind.message}`;
              break;
            case "configPersistence":
              userFriendlyMessage =
                `Erreur persistance configuration : ${captureKind.message}`;
              break;
            case "interfaceNotFound":
              userFriendlyMessage =
                `Interface non trouvée : ${captureKind.message}`;
              break;
            case "deviceListError":
              userFriendlyMessage =
                `Erreur récupération device : ${captureKind.message}.\nEssayez : sudo setcap cap_net_raw,cap_net_admin=eip nom_du_binaire.`;
              break;
            case "captureInitError":
              userFriendlyMessage =
                `Erreur initialisation capture : ${captureKind.message}`;
              break;
            case "channelSendError":
              userFriendlyMessage =
                `Erreur envoi canal capture : ${captureKind.message}`;
              break;
            case "eventSendError":
              userFriendlyMessage =
                `Erreur envoi evenement capture : ${captureKind.message}`;
              break;
            case "unsupportedLinkType":
              userFriendlyMessage =
                `Type de liaison non supporté par cette version : ${captureKind.message}.\nLa capture n'a pas démarré.`;
              break;
            case "mixedLinkType":
              userFriendlyMessage =
                `${captureKind.message}\nExportez ou réinitialisez le relevé en cours avant de capturer sur cette interface.`;
              break;
          }
        }
        break;
      }
      case "export":
        userFriendlyMessage = handleExportError(captureError.message);
        break;

      case "import":
        userFriendlyMessage = handleImportError(captureError.message);
        break;

      case "label":
        userFriendlyMessage = handleLabelerror(captureError.message);
        break;

      case "tauri":
        userFriendlyMessage = `Erreur Tauri : ${captureError.message}`;
        break;
    }
  }

  await message(userFriendlyMessage, {
    title: `Erreur Capture (${captureError.kind})`,
    kind: "error",
  });
  error(
    `Erreur Capture (${captureError.kind}) : ${userFriendlyMessage}`,
  );
}

export function handleExportError(exportError: ExportErrorKind): string {
  if (
    !exportError || typeof exportError !== "object" || !("kind" in exportError)
  ) {
    return `Erreur d'export inconnue : ${JSON.stringify(exportError)}`;
  }

  switch (exportError.kind) {
    case "emptyPath":
      return "Aucun chemin de fichier fourni pour l'export.";
    case "io":
      return `Erreur d'écriture du fichier exporté : ${exportError.message}`;
    case "csv":
      return `Erreur d'écriture CSV : ${exportError.message}`;
    case "poisonError":
      return `Erreur verrou pendant l'export : ${exportError.message}`;
    case "logNotFound":
      return "Le dossier de logs est introuvable.";
    default:
      return `Erreur d'export inconnue : ${JSON.stringify(exportError)}`;
  }
}

export function handleImportError(importError: ImportErrorKind): string {
  if (
    !importError || typeof importError !== "object" || !("kind" in importError)
  ) {
    return `Erreur d'import inconnue : ${JSON.stringify(importError)}`;
  }

  switch (importError.kind) {
    case "missingInput":
      return `Aucun fichier sélectionné pour ${importError.message}. Le relevé courant est inchangé.`;
    case "openFileError": {
      const [file, message] = importError.message;
      return `Impossible d'ouvrir le fichier ${file} : ${message}`;
    }
    case "readPacketError": {
      const [file, message] = importError.message;
      return `Erreur de lecture dans ${file} : ${message}.\nL'import a été annulé, la matrice courante est inchangée.`;
    }
    case "unsupportedLinkType": {
      const [file, label] = importError.message;
      return `Type de liaison non supporté dans ${file} : ${label}.\nL'import a été annulé, la matrice courante est inchangée.`;
    }
    default:
      return `Erreur d'import inconnue : ${JSON.stringify(importError)}`;
  }
}

/// Formate une liste d'erreurs en la plafonnant : au-delà de `max` entrées
/// (mauvais fichier, encodage…), le détail complet noie le diagnostic.
export function capList<T>(items: T[], format: (item: T) => string, max = 8): string {
  const shown = items.slice(0, max).map(format).join('\n');
  const rest = items.length - max;
  return rest > 0 ? `${shown}\n… et ${rest} autre${rest > 1 ? 's' : ''}` : shown;
}

export function handleLabelerror(labelError: LabelErrorKind): string {
  if (
    !labelError || typeof labelError !== "object" || !("kind" in labelError)
  ) {
    return `Erreur de label inconnue : ${JSON.stringify(labelError)}`;
  }

  switch(labelError.kind) {
    case "invalidMacIpFormat": {
      const [invalidMac, invalidIp] = labelError.message;
      const parts = [];
      if (invalidMac.length > 0) {
        parts.push(`MAC invalides (${invalidMac.length}) :\n${capList(invalidMac, ([line, mac, row]) => `ligne ${line}: ${mac} | ${row}`)}`);
      }
      if (invalidIp.length > 0) {
        parts.push(`IP invalides (${invalidIp.length}) :\n${capList(invalidIp, ([line, ip, row]) => `ligne ${line}: ${ip} | ${row}`)}`);
      }
      return `Formats invalides.\n${parts.join('\n\n')}`;
    }
    case "labelLinesConflicts": {
      const [sameIpDiffMac, sameIpDiffLabel] = labelError.message;
      const parts = [];
      if (sameIpDiffMac.length > 0) {
        parts.push(`même IP, MAC différent (${sameIpDiffMac.length}) :\n${capList(sameIpDiffMac, ([lineA, lineB, ip, ref_mac, mac]) => `lignes ${lineA}/${lineB} - ${ip} : ${ref_mac} <-> ${mac}`)}`);
      }
      if (sameIpDiffLabel.length > 0) {
        parts.push(`même IP, label différent (${sameIpDiffLabel.length}) :\n${capList(sameIpDiffLabel, ([lineA, lineB, ip, ref_label, label]) => `lignes ${lineA}/${lineB} - ${ip} : ${ref_label} <-> ${label}`)}`);
      }
      return `Conflits dans les lignes de labels.\n${parts.join('\n\n')}\n<Importation impossible>`;
    }
    case "invalidRowsFormat":
      return `Format de fichier invalide. Attendu au moins "mac, ip, label" (colonnes suivantes ajoutées au label).\n${capList(labelError.message, ([line, value]) => `ligne ${line}: ${value}`)}`
    case "editRejected":
      return `Édition refusée : ${labelError.message}`;
    default:
      return `Erreur de label inconnue : ${JSON.stringify(labelError)}`;
  }

}
