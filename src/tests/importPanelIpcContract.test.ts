import './helpers/domShims.ts';

import {
  deepStrictEqual as assertEquals,
  match as assertMatch,
  strictEqual as assertStrictEquals,
} from 'node:assert/strict';
import { Channel } from '@tauri-apps/api/core';

import {
  PCAP_IMPORT_COMMAND,
  invokePcapImport,
  pcapImportArguments,
} from '../utils/pcapImportContract.ts';
import importPanelSource from '../components/AnalyseView/panels/ImportPanel.vue' with { type: 'text' };

Deno.test('import PCAP - invoke reçoit les clés backend et un vrai Channel', async () => {
  let callbackId = 40;
  Object.assign(window, {
    __TAURI_INTERNALS__: {
      transformCallback: () => ++callbackId,
      unregisterCallback: () => undefined,
    },
  });

  const onEvent = new Channel<unknown>();
  let receivedCommand: string | undefined;
  let receivedArguments: Record<string, unknown> | undefined;

  await invokePcapImport(
    async (command, args) => {
      receivedCommand = command;
      receivedArguments = args;
    },
    ['capture.pcap'],
    onEvent,
  );

  assertStrictEquals(receivedCommand, PCAP_IMPORT_COMMAND);
  assertEquals(pcapImportArguments(['capture.pcap'], onEvent), {
    pcapPaths: ['capture.pcap'],
    onEvent,
  });
  assertEquals(receivedArguments?.pcapPaths, ['capture.pcap']);
  assertStrictEquals(receivedArguments?.onEvent, onEvent);
  assertStrictEquals(receivedArguments?.onEvent instanceof Channel, true);
  assertStrictEquals(JSON.stringify(onEvent), '"__CHANNEL__:41"');
});

Deno.test('ImportPanel utilise le parcours IPC testé et construit le Channel', () => {
  assertMatch(importPanelSource, /new Channel<CaptureEvent>\(\)/);
  assertMatch(importPanelSource, /await invokePcapImport\(invoke, this\.packetFiles, onEvent\)/);
});
