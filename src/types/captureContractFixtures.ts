// Vérification statique du contrat IPC (#142) : chaque constante importée
// est le JSON RÉELLEMENT produit par `CaptureEvent` (jamais son miroir),
// généré par `cargo test export_ipc_fixtures`
// (src-tauri/src/events/contract.rs), chacune vérifiée à sa définition par
// `satisfies CaptureEvent` (tags/champs excédentaires ou manquants détectés
// à l'assignation, type littéral précis conservé — voir la doc du test
// Rust `export_ipc_fixtures`). L'assignation `: CaptureEvent[]` ci-dessous
// fait que `vue-tsc`/`deno task typecheck` — qui tourne en CI — rejette ce
// fichier si le JSON réel ne satisfait plus le contrat généré : c'est le
// test « Rust → JSON → TypeScript » que #142 réclame, sur des valeurs qui
// viennent de Rust, pas retapées à la main.
//
// Fichier volontairement hors de `src/tests/` (exclu du typecheck, cf.
// `tsconfig.json`) : sa seule raison d'être est d'être compilé, jamais
// exécuté — aucun `Deno.test` ici.
import type { CaptureEvent } from "./capture";
import {
  channelCapacityPayloadFixture,
  finishedFixture,
  graphBatchFixture,
  graphBatchWithoutLabelOrPortsFixture,
  graphNodeAddedFixture,
  graphNodeUpdatedFixture,
  graphSnapshotFixture,
  importProgressFixture,
  packetBatchAllIpTypesFixture,
  packetBatchAllNetworkProtocolsFixture,
  packetBatchCorruptedFixture,
  packetBatchEthernetWithVlanFixture,
  packetBatchIeee80211Fixture,
  packetBatchInternetAndTransportPresentWithoutAddressesFixture,
  packetBatchMinimalFixture,
  packetBatchRawIpFixture,
  packetBatchSll2Fixture,
  packetBatchSll2WithoutAddressFixture,
  packetBatchSllFixture,
  packetBatchSllWithoutAddressFixture,
  packetBatchTunneledFixture,
  startedFixture,
  statsFixture,
  stoppedFixture,
} from "./generated/captureEventFixtures";

// Un littéral par variante (et par forme de paquet — VLAN/pas de VLAN,
// tunnelé, corrompu, chaque DLT) assigné à `CaptureEvent` : toute forme que
// Rust produit réellement et que le contrat généré ne décrit plus fait
// échouer la compilation ici, pas seulement un test Rust isolé.
export const captureContractFixtures: CaptureEvent[] = [
  startedFixture,
  statsFixture,
  channelCapacityPayloadFixture,
  stoppedFixture,
  finishedFixture,
  importProgressFixture,
  graphNodeAddedFixture,
  graphNodeUpdatedFixture,
  graphBatchFixture,
  graphBatchWithoutLabelOrPortsFixture,
  graphSnapshotFixture,
  packetBatchEthernetWithVlanFixture,
  packetBatchMinimalFixture,
  packetBatchTunneledFixture,
  packetBatchCorruptedFixture,
  packetBatchSllFixture,
  packetBatchSll2Fixture,
  packetBatchIeee80211Fixture,
  packetBatchRawIpFixture,
  packetBatchAllIpTypesFixture,
  packetBatchAllNetworkProtocolsFixture,
  packetBatchInternetAndTransportPresentWithoutAddressesFixture,
  packetBatchSllWithoutAddressFixture,
  packetBatchSll2WithoutAddressFixture,
];
