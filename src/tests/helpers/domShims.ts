// Shims DOM minimaux pour exécuter sous Deno des modules qui référencent des
// globals navigateur au chargement (sigma → WebGL). À importer AVANT tout
// module du graphe dans les fichiers de test.
(globalThis as any).WebGL2RenderingContext ??= class WebGL2RenderingContext {};
(globalThis as any).WebGLRenderingContext ??= class WebGLRenderingContext {};
(globalThis as any).window ??= globalThis;
