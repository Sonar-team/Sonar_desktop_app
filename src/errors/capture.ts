import { message } from "@tauri-apps/plugin-dialog";
import { error } from "@tauri-apps/plugin-log";

export type CaptureErrorKind =
  | { kind: "interfaceNotFound"; message: string }
  | { kind: "deviceListError"; message: string }
  | { kind: "captureInitError"; message: string }
  | { kind: "channelSendError"; message: string };

export type CaptureStateErrorKind =
  | { kind: "io"; message: string }
  | { kind: "poisonError"; message: string }
  | { kind: "capture"; message: CaptureErrorKind };

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
