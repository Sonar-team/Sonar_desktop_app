// cytoscape-cola déclare "types": "index.d.ts" dans son package.json
// mais ne publie pas ce fichier — on fournit la déclaration minimale ici.
declare module "cytoscape-cola" {
  import cytoscape from "cytoscape";
  const ext: cytoscape.Ext;
  export default ext;
}
