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

export type CaptureStateErrorKind =
  | { kind: "io"; message: string }
  | { kind: "poisonError"; message: string }
  | { kind: "capture"; message: CaptureErrorKind }
  | { kind: "import"; message: ImportErrorKind }
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
