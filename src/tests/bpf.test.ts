// Tests de la construction et de la validation d'expressions BPF
// (src/utils/bpf.ts) : logique pure, aucun mock nécessaire.
import { deepStrictEqual as assertEquals } from "node:assert/strict";

import {
  BPF_PRESETS,
  type BpfFormState,
  buildBpfExpression,
  emptyBpfFormState,
  validateIpFields,
  validatePortFields,
} from "../utils/bpf.ts";

/** Patch partiel de formulaire (mêmes champs que les presets). */
type FormPatch = {
  opt?: Partial<BpfFormState["opt"]>;
  proto?: Partial<BpfFormState["proto"]>;
  ip?: Partial<BpfFormState["ip"]>;
  ports?: Partial<BpfFormState["ports"]>;
  advancedRaw?: string;
};

/** Formulaire vierge avec un patch appliqué (même mécanique que les presets). */
function stateWith(patch: FormPatch): BpfFormState {
  const state = emptyBpfFormState();
  return {
    ...state,
    advancedRaw: patch.advancedRaw ?? state.advancedRaw,
    opt: { ...state.opt, ...patch.opt },
    proto: { ...state.proto, ...patch.proto },
    ip: { ...state.ip, ...patch.ip },
    ports: { ...state.ports, ...patch.ports },
  };
}

Deno.test("buildBpfExpression - formulaire vide -> expression vide", () => {
  assertEquals(buildBpfExpression(emptyBpfFormState()), "");
});

Deno.test("buildBpfExpression - options de couche jointes par and", () => {
  const expr = buildBpfExpression(stateWith({
    opt: { vlan: true, onlyIp4: true, excludeIpv6: true, excludeArp: true },
  }));
  assertEquals(expr, "vlan and ip and not ip6 and not arp");
});

Deno.test("buildBpfExpression - protocoles multiples groupés en or", () => {
  const expr = buildBpfExpression(stateWith({ proto: { tcp: true, udp: true } }));
  assertEquals(expr, "(tcp or udp)");
});

Deno.test("buildBpfExpression - un seul protocole sans parenthèses", () => {
  assertEquals(buildBpfExpression(stateWith({ proto: { icmp: true } })), "icmp");
});

Deno.test("buildBpfExpression - hôtes et réseaux avec direction", () => {
  const expr = buildBpfExpression(stateWith({
    ip: {
      includeHost: "192.168.1.42",
      excludeHost: "10.0.0.1",
      includeNet: "10.0.0.0/8",
      excludeNet: "",
      direction: "src",
    },
  }));
  assertEquals(
    expr,
    "src host 192.168.1.42 and not src host 10.0.0.1 and src net 10.0.0.0/8",
  );
});

Deno.test("buildBpfExpression - listes de ports incluses/exclues et plage", () => {
  const expr = buildBpfExpression(stateWith({
    ports: { include: "80,443", exclude: "25", range: "10000-20000", direction: "any" },
  }));
  assertEquals(
    expr,
    "(port 80 or port 443) and not port 25 and portrange 10000-20000",
  );
});

Deno.test("buildBpfExpression - expression brute concaténée à la fin", () => {
  const expr = buildBpfExpression(stateWith({
    proto: { tcp: true },
    advancedRaw: "tcp[13] & 0x02 != 0",
  }));
  assertEquals(expr, "tcp and tcp[13] & 0x02 != 0");
});

Deno.test("BPF_PRESETS - le preset web produit l'expression attendue", () => {
  const patch = BPF_PRESETS["web"];
  const expr = buildBpfExpression(stateWith({
    opt: patch.opt as BpfFormState["opt"],
    proto: patch.proto as BpfFormState["proto"],
    ports: patch.ports as BpfFormState["ports"],
  }));
  assertEquals(expr, "ip and tcp and (port 80 or port 443)");
});

Deno.test("BPF_PRESETS - tous les presets produisent une expression non vide", () => {
  for (const [name, patch] of Object.entries(BPF_PRESETS)) {
    const expr = buildBpfExpression(stateWith({
      opt: patch.opt as BpfFormState["opt"],
      proto: patch.proto as BpfFormState["proto"],
      ports: patch.ports as BpfFormState["ports"],
      advancedRaw: patch.advancedRaw ?? "",
    }));
    assertEquals(expr.length > 0, true, `preset ${name} vide`);
  }
});

Deno.test("validateIpFields - champs valides ou vides -> aucune erreur", () => {
  const state = stateWith({
    ip: {
      includeHost: "192.168.1.1",
      excludeHost: "",
      includeNet: "10.0.0.0/8",
      excludeNet: "",
      direction: "any",
    },
  });
  assertEquals(validateIpFields(state.ip), []);
});

Deno.test("validateIpFields - IP et CIDR invalides signalés", () => {
  const state = stateWith({
    ip: {
      includeHost: "192.168.1",
      excludeHost: "",
      includeNet: "10.0.0.0",
      excludeNet: "",
      direction: "any",
    },
  });
  const errors = validateIpFields(state.ip);
  assertEquals(errors.length, 2);
  assertEquals(errors[0].includes("192.168.1"), true);
  assertEquals(errors[1].includes("10.0.0.0"), true);
});

Deno.test("validatePortFields - listes et plage valides -> aucune erreur", () => {
  const state = stateWith({
    ports: { include: "80,443,22", exclude: "25", range: "10000-20000", direction: "any" },
  });
  assertEquals(validatePortFields(state.ports), []);
});

Deno.test("validatePortFields - port hors bornes et plage invalide signalés", () => {
  const state = stateWith({
    ports: { include: "99999", exclude: "", range: "20000-99999", direction: "any" },
  });
  const errors = validatePortFields(state.ports);
  assertEquals(errors.length, 2);
});
