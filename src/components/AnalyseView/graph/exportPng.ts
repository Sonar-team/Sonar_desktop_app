// Export du graphe affiché en PNG : capture Sigma à haute résolution (x2)
// et écriture sur disque via le plugin fs de Tauri.
import type Sigma from "sigma"
import { toBlob } from "@sigma/export-image"
import { writeFile } from "@tauri-apps/plugin-fs"

export async function exportGraphToPng(renderer: Sigma, filePath: string) {
  const { width, height } = renderer.getDimensions()
  const blob = await toBlob(renderer, {
    format: "png",
    backgroundColor: "#ffffff",
    width: width * 2,
    height: height * 2,
  })
  const ab = await blob.arrayBuffer()
  await writeFile(filePath, new Uint8Array(ab))
}
