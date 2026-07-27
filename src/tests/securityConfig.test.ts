// Contrat de durcissement du WebView Tauri : les exceptions CSP et les
// permissions frontend doivent rester minimales et explicites.
import { deepStrictEqual as assertEquals } from "node:assert/strict";

import tauriConfig from "../../src-tauri/tauri.conf.json" with { type: "json" };
import desktopCapability from "../../src-tauri/capabilities/default.json" with {
  type: "json",
};

Deno.test("CSP de production - aucun contenu distant ou style inline", () => {
  assertEquals(tauriConfig.app.security.csp, {
    "default-src": "'self'",
    "script-src": "'self'",
    "script-src-attr": "'none'",
    "connect-src": "ipc: http://ipc.localhost",
    "img-src": "'self'",
    "style-src": "'self'",
    "style-src-attr": "'none'",
    "worker-src": "'self' blob:",
    "font-src": "'self'",
    "object-src": "'none'",
    "base-uri": "'none'",
    "form-action": "'none'",
    "frame-src": "'none'",
    "frame-ancestors": "'none'",
    "media-src": "'none'",
  });
  assertEquals(tauriConfig.app.security.freezePrototype, true);
  assertEquals(tauriConfig.app.security.dangerousDisableAssetCspModification, false);
});

Deno.test("CSP de développement - les exceptions restent locales", () => {
  assertEquals(tauriConfig.app.security.devCsp, {
    "default-src": "'self'",
    "script-src": "'self'",
    "script-src-attr": "'none'",
    "connect-src": "'self' ipc: http://ipc.localhost ws://localhost:1420 ws://localhost:1421",
    "img-src": "'self'",
    "style-src": "'self' 'unsafe-inline'",
    "worker-src": "'self' blob:",
    "font-src": "'self'",
    "object-src": "'none'",
    "base-uri": "'none'",
    "form-action": "'none'",
    "frame-src": "'none'",
    "frame-ancestors": "'none'",
    "media-src": "'none'",
  });
});

Deno.test("capability principale - uniquement les commandes frontend utilisées", () => {
  assertEquals(desktopCapability.permissions, [
    "core:default",
    "dialog:default",
    "log:allow-log",
    "process:allow-exit",
    "fs:allow-write-file",
  ]);
  assertEquals("fs" in desktopCapability, false);
});
