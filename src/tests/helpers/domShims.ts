// Shims DOM minimaux pour exécuter sous Deno des modules qui référencent des
// globals navigateur au chargement (sigma → WebGL). À importer AVANT tout
// module du graphe dans les fichiers de test.
(globalThis as any).WebGL2RenderingContext ??= class WebGL2RenderingContext {};
(globalThis as any).WebGLRenderingContext ??= class WebGLRenderingContext {};
(globalThis as any).window ??= globalThis;

// Le localStorage natif de Deno persiste sur disque et peut être indisponible
// dans un sandbox en lecture seule. Les tests frontend n'ont besoin que du
// comportement navigateur, donc un stockage mémoire les rend déterministes.
const localStorageEntries = new Map<string, string>();
const memoryLocalStorage: Storage = {
  get length() {
    return localStorageEntries.size;
  },
  clear() {
    localStorageEntries.clear();
  },
  getItem(key: string) {
    return localStorageEntries.get(key) ?? null;
  },
  key(index: number) {
    return Array.from(localStorageEntries.keys())[index] ?? null;
  },
  removeItem(key: string) {
    localStorageEntries.delete(key);
  },
  setItem(key: string, value: string) {
    localStorageEntries.set(key, value);
  },
};

Object.defineProperty(globalThis, "localStorage", {
  configurable: true,
  value: memoryLocalStorage,
});
