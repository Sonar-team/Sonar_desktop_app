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

function showError(title: string, kind: string, message: string, detail?: string) {
  document.dispatchEvent(new CustomEvent('show-error', {
    detail: { title, kind, message, detail }
  }));
}

export async function displayCaptureError(err: unknown) {
  const captureError = err as CaptureStateErrorKind;

  if (!("kind" in captureError)) {
    showError("Erreur Capture", "unknown", "Erreur inconnue.", JSON.stringify(err));
    error(`Erreur Capture inconnue : ${JSON.stringify(err)}`);
    return;
  }

  switch (captureError.kind) {
    case "io":
      showError("Erreur Capture", "io", "Erreur d'entrée/sortie.", captureError.message);
      break;
    case "poisonError":
      showError("Erreur Capture", "poisonError", "Erreur verrou interne.", captureError.message);
      break;
    case "capture": {
      const captureKind = captureError.message as CaptureErrorKind;
      if ("kind" in captureKind) {
        switch (captureKind.kind) {
          case "interfaceNotFound":
            showError("Erreur Capture", "interfaceNotFound", "Interface réseau introuvable.", captureKind.message);
            break;
          case "deviceListError":
            showError("Erreur Capture", "deviceListError", "Impossible de lister les interfaces réseau.", captureKind.message);
            break;
          case "captureInitError":
            showError("Erreur Capture", "captureInitError", "Échec de l'initialisation de la capture.", captureKind.message);
            break;
          case "channelSendError":
            showError("Erreur Capture", "channelSendError", "Erreur d'envoi sur le canal de capture.", captureKind.message);
            break;
        }
      }
      break;
    }
    case "import": {
      const importKind = captureError.message as ImportErrorKind;
      showError("Erreur Import", importKind?.kind ?? "import", handleImportError(importKind));
      break;
    }
    case "other":
      showError("Erreur", "other", "Erreur inattendue.", captureError.message);
      break;
  }

  error(`Erreur Capture (${captureError.kind})`);
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
