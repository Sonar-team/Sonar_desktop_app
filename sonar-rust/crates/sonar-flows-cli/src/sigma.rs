//! Export du graphe relationnel Sonar au format JSON consommé par le rendu
//! sigma.js (`script/graph/`).
//!
//! La forme produite est celle du snapshot envoyé au frontend
//! (`GraphData` : `nodes` et `edges` indexés par identifiant), afin que le
//! script de rendu réutilise tel quel le code de graphe de l'application
//! (`src/components/AnalyseView/graph/`) et produise la même image que
//! l'export PNG du bureau. Les filtres de la sous-commande `graph`
//! (`--min-bytes`, `--max-nodes`, `--protocol`, `--labels`) sont appliqués
//! avant sérialisation.

use std::collections::HashSet;
use std::path::Path;

use serde_json::{Map, Value};
use sonar_flows_core::graph::GraphData;

use crate::graphviz::{GraphOptions, filtered_edges, node_label};

/// Écrit le graphe filtré en JSON. Retourne le nombre de relations conservées.
pub fn export_graph_json(
    graph: &GraphData,
    output: &Path,
    options: &GraphOptions,
) -> Result<usize, String> {
    if options.max_nodes == 0 {
        return Err("--max-nodes doit être supérieur à zéro".to_string());
    }

    let edges = filtered_edges(graph, options);
    let retained_nodes: HashSet<&str> = edges
        .iter()
        .flat_map(|edge| [edge.source.as_str(), edge.target.as_str()])
        .collect();

    let mut nodes_json = Map::new();
    for node in graph
        .nodes
        .values()
        .filter(|node| retained_nodes.contains(node.id.as_str()))
    {
        let mut value = serde_json::to_value(node)
            .map_err(|error| format!("sérialisation du nœud {}: {error}", node.id))?;
        // `--labels` choisit le texte affiché : le champ `label` du snapshot
        // est ce que le frontend rend au-dessus du nœud.
        if let Value::Object(fields) = &mut value {
            fields.insert(
                "label".to_string(),
                Value::String(node_label(node, options.labels).to_string()),
            );
        }
        nodes_json.insert(node.id.clone(), value);
    }

    let mut edges_json = Map::new();
    for edge in &edges {
        let value = serde_json::to_value(edge)
            .map_err(|error| format!("sérialisation de l'arête {}: {error}", edge.id))?;
        edges_json.insert(edge.id.clone(), value);
    }

    let document = Value::Object(Map::from_iter([
        ("nodes".to_string(), Value::Object(nodes_json)),
        ("edges".to_string(), Value::Object(edges_json)),
    ]));

    // Sérialisation stable (clés triées par serde_json) : deux exports de la
    // même matrice produisent le même fichier, comme le reste de SONAR.
    let text = serde_json::to_string_pretty(&document)
        .map_err(|error| format!("sérialisation du graphe: {error}"))?;
    std::fs::write(output, text).map_err(|error| format!("{}: {error}", output.display()))?;

    Ok(edges.len())
}
