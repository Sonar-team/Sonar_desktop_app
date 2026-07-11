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
  | { kind: "eventSendError"; message: string };

export type ImportErrorKind =
  | { kind: "openFileError"; file: string; message: string }
  | { kind: "invalidPacket"; message: string }
  | { kind: "parseError"; message: string }
  | { kind: "other"; message: string };

export type InvalidLineValue = [number, string];
export type InvalidFieldValue = [number, string, string];
export type LabelConflictRow = [number, number, string, string, string, string, string];

export type LabelErrorKind =
  | { kind: "invalidMacIpFormat"; message: [InvalidFieldValue[], InvalidFieldValue[]] }
  | { kind: "labelLinesConflicts"; message: [LabelConflictRow[], LabelConflictRow[]] }
  | { kind: "invalidRowsFormat"; message: InvalidLineValue[] }

export type CaptureStateErrorKind =
  | { kind: "io"; message: string }
  | { kind: "poisonError"; message: string }
  | { kind: "invalidTransition"; message: string }
  | { kind: "capture"; message: CaptureErrorKind }
  | { kind: "import"; message: ImportErrorKind }
  | { kind: "label"; message: LabelErrorKind}
  | { kind: "other"; message: string };

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
          }
        }
        break;
      case "import":
        userFriendlyMessage = handleImportError(captureError.message);
        break;

      case "label":
        userFriendlyMessage = handleLabelerror(captureError.message);
        break;

      case "other":
        userFriendlyMessage = `Erreur inattendue : ${captureError.message}`;
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

function handleImportError(importError: ImportErrorKind): string {
  if (
    !importError || typeof importError !== "object" || !("kind" in importError)
  ) {
    return `Erreur d'import inconnue : ${JSON.stringify(importError)}`;
  }

  switch (importError.kind) {
    case "openFileError":
      return `Impossible d'ouvrir le fichier ${importError.file} : ${importError.message}`;
    case "invalidPacket":
      return `Paquet invalide : ${importError.message}`;
    case "parseError":
      return `Erreur d'analyse : ${importError.message}`;
    case "other":
      return `Erreur d'import : ${importError.message}`;
    default:
      return `Erreur d'import inconnue : ${JSON.stringify(importError)}`;
  }
}

function handleLabelerror(labelError: LabelErrorKind): string {
  if (
    !labelError || typeof labelError !== "object" || !("kind" in labelError)
  ) {
    return `Erreur de label inconnue : ${JSON.stringify(labelError)}`;
  }

  switch(labelError.kind) {
    case "invalidMacIpFormat":
      const [invalidMac, invalidIp] = labelError.message;
      return `Formats invalides : MAC - ${invalidMac.map(([line, mac, row]) => `ligne ${line}: ${mac} | ${row}`).join('\n')}, IP - ${invalidIp.map(([line, ip, row]) => `ligne ${line}: ${ip} | ${row}`).join('\n')}`;
    case "labelLinesConflicts":
      const [sameIpDiffMac, sameIpDiffLabel] = labelError.message;
      return `Conflits dans les lignes de labels : même IP, MAC différent - ${sameIpDiffMac.map(([lineA, lineB, ip, ref_mac, mac, rowA, rowB]) => `lignes ${lineA}/${lineB} - ${ip} : ${ref_mac} <-> ${mac}\nligne ${lineA}: ${rowA}\nligne ${lineB}: ${rowB}`).join('\n')}, même IP, label différent - ${sameIpDiffLabel.map(([lineA, lineB, ip, ref_label, label, rowA, rowB]) => `lignes ${lineA}/${lineB} - ${ip} : ${ref_label} <-> ${label}\nligne ${lineA}: ${rowA}\nligne ${lineB}: ${rowB}`).join('\n')} \n <Importation impossible>`;
    case "invalidRowsFormat":
      return `Format de ligne invalide. Attendu au moins "mac, ip, label"; les colonnes suivantes sont ajoutées au label. Trouvé ${labelError.message.map(([line, value]) => `ligne ${line}: ${value}`).join('\n')}`
    default:
      return `Erreur de label inconnue : ${JSON.stringify(labelError)}`;
  }

}
