#!/usr/bin/env node
// Rend un graphe JSON (produit par `sonar-cli graph -o graphe.json`) en PNG
// avec sigma.js, sans interface : la page ./page/main.ts est empaquetée par
// Vite, chargée dans un navigateur headless (famille Chromium), et l'image
// exportée est récupérée dans le DOM sérialisé (`--dump-dom`).
//
// Aucune dépendance nouvelle : Vite et sigma.js sont déjà ceux du frontend, le
// navigateur est celui de la machine (variable SONAR_BROWSER pour l'imposer).
import { spawn } from "node:child_process"
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { dirname, join, resolve } from "node:path"
import { fileURLToPath, pathToFileURL } from "node:url"

const HERE = dirname(fileURLToPath(import.meta.url))
const REPO = resolve(HERE, "../..")

const DEFAULTS = {
  width: 1600,
  height: 1000,
  scale: 2,
  iterations: 600,
  background: "#ffffff",
  edgeLabels: "auto", // auto | on | off
  seed: 42,
  timeout: 120,
  profile: "", // vide = profil habituel du navigateur (cf. runBrowser)
}

const BROWSER_CANDIDATES = [
  "chromium",
  "chromium-browser",
  "google-chrome",
  "google-chrome-stable",
  "brave-browser",
  "microsoft-edge",
]

function usage(message) {
  if (message) console.error(`error: ${message}`)
  console.error(`usage: node script/graph/render-sigma.mjs --graph <graphe.json> --out <image.png>
                              [--width N] [--height N] [--scale N] [--iterations N]
                              [--background <couleur>] [--edge-labels auto|on|off]
                              [--seed N] [--timeout <secondes>] [--browser <chemin>]
                              [--profile <dossier>]`)
  process.exit(message ? 2 : 0)
}

function parseArgs(argv) {
  const options = { ...DEFAULTS, graph: null, out: null, browser: process.env.SONAR_BROWSER || null }
  const numbers = new Set(["width", "height", "scale", "iterations", "seed", "timeout"])
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i]
    if (arg === "-h" || arg === "--help") usage()
    if (!arg.startsWith("--")) usage(`argument inattendu : ${arg}`)
    const key = arg.slice(2).replace(/-([a-z])/g, (_, c) => c.toUpperCase())
    const value = argv[++i]
    if (value === undefined) usage(`valeur manquante pour ${arg}`)
    if (!(key in options)) usage(`option inconnue : ${arg}`)
    if (numbers.has(key)) {
      const parsed = Number(value)
      if (!Number.isFinite(parsed) || parsed <= 0) usage(`${arg} attend un nombre positif`)
      options[key] = parsed
    } else {
      options[key] = value
    }
  }
  if (!options.graph) usage("--graph est obligatoire")
  if (!options.out) usage("--out est obligatoire")
  if (!["auto", "on", "off"].includes(options.edgeLabels)) usage("--edge-labels attend auto, on ou off")
  return options
}

/** Navigateur de la famille Chromium : imposé, ou premier trouvé dans le PATH. */
function findBrowser(explicit) {
  if (explicit) {
    if (existsSync(explicit) || which(explicit)) return explicit
    throw new Error(`navigateur introuvable : ${explicit}`)
  }
  for (const candidate of BROWSER_CANDIDATES) {
    const found = which(candidate)
    if (found) return found
  }
  throw new Error(
    `aucun navigateur Chromium trouvé (essayés : ${BROWSER_CANDIDATES.join(", ")}).\n` +
      "Installez-en un, ou désignez-le avec SONAR_BROWSER=/chemin/vers/chromium."
  )
}

function which(command) {
  for (const dir of (process.env.PATH || "").split(":")) {
    if (!dir) continue
    const candidate = join(dir, command)
    if (existsSync(candidate)) return candidate
  }
  return null
}

/** Empaquette ./page/main.ts (et le code de graphe de l'app qu'il importe) en
 * un seul script IIFE, chargeable depuis un `file://` sans serveur ni CORS. */
async function bundlePage() {
  const { build } = await import(pathToFileURL(join(REPO, "node_modules/vite/dist/node/index.js")))
  const result = await build({
    configFile: false,
    root: REPO,
    logLevel: "error",
    build: {
      write: false,
      minify: true,
      target: "esnext",
      lib: {
        entry: join(HERE, "page/main.ts"),
        formats: ["iife"],
        name: "SonarGraphRender",
        fileName: () => "bundle.js",
      },
    },
  })
  const output = (Array.isArray(result) ? result[0] : result).output
  const chunk = output.find((item) => item.type === "chunk")
  if (!chunk) throw new Error("le bundle Vite n'a produit aucun script")
  return chunk.code
}

/** Neutralise `</script>` et les séparateurs de ligne Unicode dans une valeur
 * JSON injectée en ligne dans la page. */
function inlineJson(value) {
  return JSON.stringify(value)
    .replace(/</g, "\\u003c")
    .replace(/\u2028/g, "\\u2028")
    .replace(/\u2029/g, "\\u2029")
}

function buildHtml(bundle, graph, renderOptions) {
  return `<!doctype html>
<html lang="fr">
<head><meta charset="utf-8"><title>SONAR</title>
<style>
  html, body { margin: 0; padding: 0; background: #000; overflow: hidden; }
  #sigma { position: absolute; top: 0; left: 0; }
  #sonar-png, #sonar-error { display: none; }
</style>
</head>
<body>
<div id="sigma"></div>
<div id="sonar-png"></div>
<div id="sonar-error"></div>
<script>
window.__SONAR_GRAPH__ = ${inlineJson(graph)};
window.__SONAR_OPTIONS__ = ${inlineJson(renderOptions)};
</script>
<script>
${bundle}
</script>
</body>
</html>
`
}

function runBrowser(browser, pageUrl, profileDir, timeoutSeconds) {
  const args = [
    "--headless=new",
    // Profil jetable seulement si demandé : certains navigateurs de la
    // famille Chromium (Brave, vérifié le 07/08/2026) restent bloqués à
    // l'initialisation d'un profil neuf et ne rendent jamais la page. Sans
    // cette option, le navigateur réutilise son profil habituel — ce qui
    // fonctionne même quand une fenêtre est déjà ouverte, le mode headless
    // étant isolé.
    ...(profileDir ? [`--user-data-dir=${profileDir}`] : []),
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-extensions",
    "--disable-sync",
    "--hide-scrollbars",
    // WebGL sans carte graphique : sigma.js dessine ses nœuds et arêtes en
    // WebGL, SwiftShader le rend sur CPU (indispensable en CI et sur serveur).
    "--use-gl=angle",
    "--use-angle=swiftshader",
    "--enable-unsafe-swiftshader",
    `--virtual-time-budget=${Math.round(timeoutSeconds * 1000)}`,
    "--dump-dom",
    pageUrl,
  ]
  return new Promise((resolvePromise, rejectPromise) => {
    const child = spawn(browser, args, { stdio: ["ignore", "pipe", "pipe"] })
    let stdout = ""
    let stderr = ""
    const timer = setTimeout(() => {
      child.kill("SIGKILL")
      rejectPromise(new Error(`le navigateur n'a pas rendu l'image en ${timeoutSeconds} s`))
    }, timeoutSeconds * 1000 + 15000)
    child.stdout.on("data", (chunk) => (stdout += chunk))
    child.stderr.on("data", (chunk) => (stderr += chunk))
    child.on("error", (error) => {
      clearTimeout(timer)
      rejectPromise(new Error(`lancement de ${browser} impossible : ${error.message}`))
    })
    child.on("close", (code) => {
      clearTimeout(timer)
      if (code !== 0) {
        rejectPromise(new Error(`${browser} a échoué (code ${code})\n${stderr.trim()}`))
        return
      }
      resolvePromise({ stdout, stderr })
    })
  })
}

function extract(dom, kind) {
  const begin = `[SONAR:${kind}:begin]`
  const end = `[SONAR:${kind}:end]`
  const start = dom.indexOf(begin)
  if (start === -1) return null
  const stop = dom.indexOf(end, start)
  if (stop === -1) return null
  return dom.slice(start + begin.length, stop)
}

async function main() {
  const options = parseArgs(process.argv.slice(2))
  const graphPath = resolve(options.graph)
  const outPath = resolve(options.out)
  const graph = JSON.parse(readFileSync(graphPath, "utf8"))
  const edgeCount = Object.keys(graph.edges ?? {}).length
  const nodeCount = Object.keys(graph.nodes ?? {}).length
  if (nodeCount === 0) throw new Error(`${graphPath} : graphe vide, rien à rendre`)

  const browser = findBrowser(options.browser)
  const renderOptions = {
    width: Math.round(options.width),
    height: Math.round(options.height),
    scale: options.scale,
    iterations: Math.round(options.iterations),
    background: options.background,
    // Au-delà de ~80 relations, les labels de protocole se chevauchent et
    // rendent l'image illisible : ils sont masqués sauf demande explicite.
    edgeLabels: options.edgeLabels === "on" || (options.edgeLabels === "auto" && edgeCount <= 80),
    seed: Math.round(options.seed),
  }

  const bundle = await bundlePage()
  const workDir = mkdtempSync(join(tmpdir(), "sonar-sigma-"))
  try {
    const pagePath = join(workDir, "page.html")
    writeFileSync(pagePath, buildHtml(bundle, graph, renderOptions))
    const { stdout, stderr } = await runBrowser(
      browser,
      pathToFileURL(pagePath).href,
      options.profile ? resolve(options.profile) : "",
      options.timeout
    )

    const failure = extract(stdout, "error")
    if (failure) throw new Error(`rendu sigma.js en échec :\n${failure}`)
    const base64 = extract(stdout, "png")
    if (!base64) {
      throw new Error(
        `le navigateur n'a produit aucune image (WebGL indisponible ?).\n${stderr.trim().slice(-2000)}`
      )
    }
    writeFileSync(outPath, Buffer.from(base64, "base64"))
    console.error(
      `${nodeCount} nœud(s), ${edgeCount} relation(s) rendus dans ${outPath} ` +
        `(${renderOptions.width * renderOptions.scale}×${renderOptions.height * renderOptions.scale})`
    )
  } finally {
    rmSync(workDir, { recursive: true, force: true })
  }
}

main().catch((error) => {
  console.error(`error: ${error.message}`)
  process.exit(1)
})
