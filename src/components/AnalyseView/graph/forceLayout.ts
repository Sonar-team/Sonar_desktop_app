// Superviseur ForceAtlas2 (worker) : (re)création adaptée à la taille du
// graphe, démarrage/arrêt liés au bouton « Gravité ». Encapsule l'instance
// du layout et le dernier ordre du graphe vu par inferSettings, puisque
// slowDown/BarnesHut en dépendent.
import Graph from "graphology"
import forceAtlas2 from "graphology-layout-forceatlas2"
import FA2Layout from "graphology-layout-forceatlas2/worker"

export interface ForceLayoutController {
  start(): void
  stop(): void
  /** Reprend un layout existant, ou le recrée s'il n'y en a pas. */
  resume(): void
  /** Recrée le layout si le graphe vient de se peupler ou a beaucoup grandi
   * depuis le dernier inferSettings (slowDown/BarnesHut dépendent de la taille). */
  ensureSettings(): void
  kill(): void
}

export function createForceLayoutController(graph: Graph): ForceLayoutController {
  let layout: InstanceType<typeof FA2Layout> | null = null
  let lastOrder = 0

  function kill() {
    if (!layout) return
    try { layout.kill() } catch { /* already stopped */ }
    layout = null
    lastOrder = 0
  }

  function start() {
    // ⚠️ Jamais sur un graphe vide : inferSettings y produit
    // slowDown = 1 + log(0) = -Infinity et fige la simulation pour toujours.
    if (graph.order === 0) return
    kill()
    lastOrder = graph.order
    const settings = forceAtlas2.inferSettings(graph)
    layout = new FA2Layout(graph, { settings })
    layout.start()
  }

  return {
    start,
    stop() { layout?.stop() },
    resume() {
      if (layout && lastOrder > 0) layout.start()
      else start()
    },
    ensureSettings() {
      if (graph.order === 0) return
      if (!layout || lastOrder === 0 || graph.order >= lastOrder * 4) start()
    },
    kill,
  }
}
