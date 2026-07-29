// Contrat IPC partagé par le parcours d'import PCAP officiel. Garder le nom
// de commande et les clés dans un module pur permet de les tester sans WebView.
export const PCAP_IMPORT_COMMAND = 'convert_from_pcap_list';

export type InvokeCommand = (command: string, args?: Record<string, unknown>) => Promise<unknown>;

export function pcapImportArguments<TChannel>(pcapPaths: string[], onEvent: TChannel) {
  return { pcapPaths, onEvent };
}

export function invokePcapImport<TChannel>(
  invokeCommand: InvokeCommand,
  pcapPaths: string[],
  onEvent: TChannel,
): Promise<unknown> {
  return invokeCommand(PCAP_IMPORT_COMMAND, pcapImportArguments(pcapPaths, onEvent));
}
