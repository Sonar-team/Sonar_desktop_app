import { message } from "@tauri-apps/plugin-dialog";
import { error } from "@tauri-apps/plugin-log";

export type CaptureErrorKind =
  | { kind: "interfaceNotFound"; message: string }
  | { kind: "deviceListError"; message: string }
  | { kind: "captureInitError"; message: string }
  | { kind: "channelSendError"; message: string };

export type ImportErrorKind =
  | { kind: "openFileError"; file: string; message: string }
  | { kind: "invalidPacket"; message: string }
  | { kind: "parseError"; message: string }
  | { kind: "other"; message: string };

export type LabelErrorKind =
  | {kind: "fileNameConflicts"; message: { files_names: [string][] }}
  | { kind: "invalidFormats"; message: { invalid_mac: [string, string, string, string, string][], invalid_ip: [string, string, string, string, string][] } }
  | { kind: "labelLinesConflicts"; message: { same_ip_diff_mac: [string, string, string, string, string][], same_ip_diff_label: [string, string, string, string, string][] } }

export type CaptureStateErrorKind =
  | { kind: "io"; message: string }
  | { kind: "poisonError"; message: string }
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
      case "capture":
        const captureKind = captureError.message as CaptureErrorKind;
        if ("kind" in captureKind) {
          switch (captureKind.kind) {
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
    case "invalidFormats":
      return `Formats invalides : MAC - ${labelError.message.invalid_mac.map(([file, mac]) => `${file} : ${mac}`).join('\n')}, IP - ${labelError.message.invalid_ip.map(([file, ip]) => `${file} : ${ip}`).join('\n')}`;
    case "fileNameConflicts":
      return `Ce(s) fichier(s) existe(nt) déjà : \n ${labelError.message} \n <Importation impossible>`
    case "labelLinesConflicts":
      return `Conflits dans les lignes de labels : même IP, MAC différent - ${labelError.message.same_ip_diff_mac.map(([ip, ref_mac,name_1, mac, name_2]) => `${ip} : ${ref_mac} (${name_1}) <-> ${mac} (${name_2})`).join('\n')}, même IP, label différent - ${labelError.message.same_ip_diff_label.map(([ip, ref_label, name_1, label, name_2]) => `${ip} : ${ref_label} (${name_1}) <-> ${label} (${name_2})`).join('\n')} \n <Importation impossible>`;
    default:
      return `Erreur de label inconnue : ${JSON.stringify(labelError)}`;
  }

}
