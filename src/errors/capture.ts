// Erreurs du backend (miroir de `CaptureStateErrorKind` côté Rust, forme
// discriminée `{ kind, message }`) et affichage utilisateur : chaque erreur
// est traduite en message lisible, montrée en dialogue et journalisée.
import { message } from "@tauri-apps/plugin-dialog";
import { error } from "@tauri-apps/plugin-log";

export type CaptureErrorKind =
  | { kind: "invalidConfig"; message: string }
  | { kind: "configPersistence"; message: string }
  | { kind: "interfaceNotFound"; message: string }
  | { kind: "deviceListError"; message: string }
  | { kind: "captureInitError"; message: string }
  | { kind: "channelSendError"; message: string }
  | { kind: "eventSendError"; message: string }
  | { kind: "unsupportedLinkType"; message: string }
  | { kind: "mixedLinkType"; message: string };

// Miroir exact de `PcapImportErrorKind` (Rust) : les variantes à deux champs
// sont sérialisées par serde en `message: [fichier, détail]` (tag/content),
// pas en champs nommés (#142).
export type ImportErrorKind =
  | { kind: "openFileError"; message: [string, string] }
  | { kind: "readPacketError"; message: [string, string] }
  | { kind: "unsupportedLinkType"; message: [string, string] };

export type InvalidLineValue = [number, string];
export type InvalidFieldValue = [number, string, string];
export type LabelConflictRow = [number, number, string, string, string, string, string];

export type LabelErrorKind =
  | { kind: "invalidMacIpFormat"; message: [InvalidFieldValue[], InvalidFieldValue[]] }
  | { kind: "labelLinesConflicts"; message: [LabelConflictRow[], LabelConflictRow[]] }
  | { kind: "invalidRowsFormat"; message: InvalidLineValue[] }
  | { kind: "editRejected"; message: string }

// Miroir exact de `ExportErrorKind` (Rust) : les variantes sans donnée
// (`emptyPath`, `logNotFound`) sont sérialisées sans clé `message`.
export type ExportErrorKind =
  | { kind: "emptyPath" }
  | { kind: "io"; message: string }
  | { kind: "csv"; message: string }
  | { kind: "poisonError"; message: string }
  | { kind: "logNotFound" };

export type CaptureStateErrorKind =
  | { kind: "io"; message: string }
  | { kind: "poisonError"; message: string }
  | { kind: "invalidTransition"; message: string }
  | { kind: "capture"; message: CaptureErrorKind }
  | { kind: "export"; message: ExportErrorKind }
  | { kind: "import"; message: ImportErrorKind }
  | { kind: "label"; message: LabelErrorKind }
  | { kind: "tauri"; message: string };

export async function displayCaptureError(err: unknown) {
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
      case "capture":
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

function handleExportError(exportError: ExportErrorKind): string {
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

function handleImportError(importError: ImportErrorKind): string {
  if (
    !importError || typeof importError !== "object" || !("kind" in importError)
  ) {
    return `Erreur d'import inconnue : ${JSON.stringify(importError)}`;
  }

  switch (importError.kind) {
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
function capList<T>(items: T[], format: (item: T) => string, max = 8): string {
  const shown = items.slice(0, max).map(format).join('\n');
  const rest = items.length - max;
  return rest > 0 ? `${shown}\n… et ${rest} autre${rest > 1 ? 's' : ''}` : shown;
}

function handleLabelerror(labelError: LabelErrorKind): string {
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
