// Tests de la coloration des protocoles dans le graphe réseau
// (src/utils/protocolColors.ts) : correspondance exacte, normalisation de
// clé, niveaux de repli, et fallback stable par hachage.
import { deepStrictEqual as assertEquals } from "node:assert/strict";

import { PROTOCOL_COLOR_LEGEND, colorForProtocol } from "../utils/protocolColors.ts";

const UNKNOWN_PROTOCOL_COLOR = "#94A3B8";

Deno.test("colorForProtocol - correspondance exacte, insensible à la casse", () => {
  assertEquals(colorForProtocol("tcp"), colorForProtocol("TCP"));
  assertEquals(colorForProtocol("TCP"), "#2563EB");
});

Deno.test("colorForProtocol - normalise ponctuation/espaces (EtherNet/IP, OPC UA)", () => {
  assertEquals(colorForProtocol("EtherNet/IP"), colorForProtocol("ethernetip"));
  assertEquals(colorForProtocol("OPC UA"), colorForProtocol("opcua"));
});

Deno.test("colorForProtocol - vide ou absent -> couleur inconnue", () => {
  assertEquals(colorForProtocol(""), UNKNOWN_PROTOCOL_COLOR);
  assertEquals(colorForProtocol(null), UNKNOWN_PROTOCOL_COLOR);
  assertEquals(colorForProtocol(undefined), UNKNOWN_PROTOCOL_COLOR);
  assertEquals(colorForProtocol("   "), UNKNOWN_PROTOCOL_COLOR);
});

Deno.test("colorForProtocol - 'unknown' et variantes -> couleur inconnue", () => {
  assertEquals(colorForProtocol("Unknown"), UNKNOWN_PROTOCOL_COLOR);
  assertEquals(colorForProtocol("UnknownProtocol"), UNKNOWN_PROTOCOL_COLOR);
});

Deno.test("colorForProtocol - extensions IPv6 retombent sur la couleur IPv6", () => {
  const ipv6Color = colorForProtocol("IPv6");
  assertEquals(colorForProtocol("IPv6HopByHop"), ipv6Color);
  assertEquals(colorForProtocol("HopOpt"), ipv6Color);
  assertEquals(colorForProtocol("ipv6-quelquechose-inconnu"), ipv6Color);
});

Deno.test("colorForProtocol - protocoles de routage connus", () => {
  assertEquals(colorForProtocol("DSR"), "#84CC16");
  assertEquals(colorForProtocol("PNNI"), "#84CC16");
});

Deno.test("colorForProtocol - protocoles de sécurité connus", () => {
  assertEquals(colorForProtocol("HIP"), "#8B5CF6");
  assertEquals(colorForProtocol("SKIP"), "#8B5CF6");
});

Deno.test("colorForProtocol - heuristique MPLS", () => {
  assertEquals(colorForProtocol("MplsInIp"), "#60A5FA");
});

Deno.test("colorForProtocol - heuristique encapsulation IP-in-IP / over", () => {
  assertEquals(colorForProtocol("SomethingOverIp"), "#7DD3FC");
});

Deno.test("colorForProtocol - protocole totalement inconnu -> fallback par hachage stable", () => {
  const a = colorForProtocol("ProtocoleMaisonXYZ");
  const b = colorForProtocol("ProtocoleMaisonXYZ");
  assertEquals(a, b, "même entrée -> même couleur (déterministe)");

  const c = colorForProtocol("AutreProtocoleInconnu123");
  // Pas de garantie de distinction (collisions de hash possibles), mais la
  // couleur doit rester dans la palette de repli, jamais la couleur "inconnu".
  assertEquals(a !== UNKNOWN_PROTOCOL_COLOR, true);
  assertEquals(c !== UNKNOWN_PROTOCOL_COLOR, true);
});

Deno.test("PROTOCOL_COLOR_LEGEND - toutes les entrées ont une couleur définie", () => {
  assertEquals(PROTOCOL_COLOR_LEGEND.length > 0, true);
  for (const item of PROTOCOL_COLOR_LEGEND) {
    assertEquals(typeof item.color, "string");
    assertEquals(item.color.startsWith("#"), true, `${item.label}: ${item.color}`);
  }
});
