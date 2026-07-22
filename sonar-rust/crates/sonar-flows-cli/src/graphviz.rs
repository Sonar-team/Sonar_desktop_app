//! Export batch du graphe relationnel Sonar au format Graphviz.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use clap::ValueEnum;
use sonar_flows_core::graph::{Edge, GraphData, Node};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum GraphEngine {
    Dot,
    Neato,
    Fdp,
    Sfdp,
    Circo,
}

impl GraphEngine {
    fn executable(self) -> &'static str {
        match self {
            Self::Dot => "dot",
            Self::Neato => "neato",
            Self::Fdp => "fdp",
            Self::Sfdp => "sfdp",
            Self::Circo => "circo",
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum NodeLabels {
    Ip,
    Mac,
    Label,
}

pub struct GraphOptions {
    pub min_bytes: u64,
    pub max_nodes: usize,
    pub labels: NodeLabels,
    pub engine: GraphEngine,
}

/// Filtre le graphe, produit le DOT puis, selon l'extension, appelle Graphviz.
/// Retourne le nombre de relations conservées dans l'image.
pub fn export_graph(
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
    let dot = render_dot(graph, &edges, &retained_nodes, options.labels);

    let format = output
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| "la sortie doit avoir une extension .dot, .svg ou .png".to_string())?;

    match format.as_str() {
        "dot" => {
            std::fs::write(output, dot).map_err(|error| format!("{}: {error}", output.display()))?
        }
        "svg" | "png" => render_with_graphviz(&dot, output, &format, options.engine)?,
        _ => {
            return Err("format de sortie non supporté (attendu : .dot, .svg ou .png)".to_string());
        }
    }

    Ok(edges.len())
}

fn filtered_edges<'a>(graph: &'a GraphData, options: &GraphOptions) -> Vec<&'a Edge> {
    let mut candidates: Vec<&Edge> = graph
        .edges
        .values()
        .filter(|edge| edge.total_bytes >= options.min_bytes)
        .collect();

    // Les nœuds les plus actifs sont retenus, puis on garde le sous-graphe
    // induit. La clé textuelle tranche les égalités de façon stable.
    let mut traffic: HashMap<&str, u64> = HashMap::new();
    for edge in &candidates {
        for node_id in [&edge.source, &edge.target] {
            let total = traffic.entry(node_id).or_default();
            *total = total.saturating_add(edge.total_bytes);
        }
    }
    let mut ranked_nodes: Vec<(&str, u64)> = traffic.into_iter().collect();
    ranked_nodes.sort_by(|(id_a, bytes_a), (id_b, bytes_b)| {
        bytes_b.cmp(bytes_a).then_with(|| id_a.cmp(id_b))
    });
    let retained: HashSet<&str> = ranked_nodes
        .into_iter()
        .take(options.max_nodes)
        .map(|(id, _)| id)
        .collect();

    candidates.retain(|edge| {
        retained.contains(edge.source.as_str()) && retained.contains(edge.target.as_str())
    });
    candidates.sort_by(|a, b| {
        (&a.source, &a.target, &a.label, &a.id).cmp(&(&b.source, &b.target, &b.label, &b.id))
    });
    candidates
}

fn render_dot(
    graph: &GraphData,
    edges: &[&Edge],
    retained_nodes: &HashSet<&str>,
    labels: NodeLabels,
) -> String {
    let mut output = String::from(
        "digraph sonar {\n  graph [overlap=false, splines=true, outputorder=edgesfirst, bgcolor=\"white\", pad=0.35];\n  node [shape=circle, style=filled, fontname=\"sans-serif\", fontsize=10, fontcolor=\"#111827\", color=\"#334155\"];\n  edge [fontname=\"sans-serif\", fontsize=8, color=\"#64748b\", fontcolor=\"#334155\"];\n",
    );

    let nodes_by_id: HashMap<&str, &Node> = graph
        .nodes
        .values()
        .map(|node| (node.id.as_str(), node))
        .collect();

    let mut nodes: Vec<&Node> = graph
        .nodes
        .values()
        .filter(|node| retained_nodes.contains(node.id.as_str()))
        .collect();
    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    for node in nodes {
        let label = node_label(node, labels);
        let _ = writeln!(
            output,
            "  \"{}\" [label=\"{}\", fillcolor=\"{}\"];",
            dot_escape(stable_node_id(node)),
            dot_escape(label),
            dot_escape(&node.color)
        );
    }

    for edge in edges {
        let Some(source) = nodes_by_id.get(edge.source.as_str()) else {
            continue;
        };
        let Some(target) = nodes_by_id.get(edge.target.as_str()) else {
            continue;
        };
        let ports = edge
            .ports
            .iter()
            .map(u16::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let mut label = edge.label.clone();
        if !ports.is_empty() {
            let _ = write!(label, "\n{ports}");
        }
        let _ = write!(label, "\n{}", format_bytes(edge.total_bytes));
        let pen_width = (1.0 + (edge.total_bytes as f64 + 1.0).log2() * 0.22).min(7.0);
        let direction = if edge.bidir { "both" } else { "forward" };
        let _ = writeln!(
            output,
            "  \"{}\" -> \"{}\" [label=\"{}\", dir={}, penwidth={pen_width:.2}];",
            dot_escape(stable_node_id(source)),
            dot_escape(stable_node_id(target)),
            dot_escape(&label),
            direction
        );
    }
    output.push_str("}\n");
    output
}

fn stable_node_id(node: &Node) -> &str {
    non_empty(&node.ip)
        .or_else(|| non_empty(&node.mac))
        .unwrap_or(&node.name)
}

fn node_label(node: &Node, labels: NodeLabels) -> &str {
    match labels {
        NodeLabels::Ip => non_empty(&node.ip).unwrap_or(&node.name),
        NodeLabels::Mac => non_empty(&node.mac).unwrap_or(&node.name),
        NodeLabels::Label => node
            .label
            .as_deref()
            .and_then(non_empty)
            .or_else(|| non_empty(&node.ip))
            .unwrap_or(&node.name),
    }
}

fn non_empty(value: &str) -> Option<&str> {
    (!value.trim().is_empty()).then_some(value)
}

fn dot_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "")
}

fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;
    match bytes {
        0..=1023 => format!("{bytes} o"),
        1024..=1_048_575 => format!("{:.1} Kio", bytes as f64 / KIB),
        1_048_576..=1_073_741_823 => format!("{:.1} Mio", bytes as f64 / MIB),
        _ => format!("{:.2} Gio", bytes as f64 / GIB),
    }
}

fn render_with_graphviz(
    dot: &str,
    output: &Path,
    format: &str,
    engine: GraphEngine,
) -> Result<(), String> {
    let executable = engine.executable();
    let mut child = Command::new(executable)
        .arg(format!("-T{format}"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            format!(
                "impossible de lancer Graphviz ({executable}): {error}; installez Graphviz ou utilisez une sortie .dot"
            )
        })?;

    child
        .stdin
        .take()
        .ok_or_else(|| "entrée standard Graphviz indisponible".to_string())?
        .write_all(dot.as_bytes())
        .map_err(|error| format!("impossible d'envoyer le graphe à Graphviz: {error}"))?;

    let rendered = child
        .wait_with_output()
        .map_err(|error| format!("erreur pendant l'exécution de Graphviz: {error}"))?;
    if !rendered.status.success() {
        return Err(format!(
            "Graphviz a échoué: {}",
            String::from_utf8_lossy(&rendered.stderr).trim()
        ));
    }
    std::fs::write(output, rendered.stdout)
        .map_err(|error| format!("{}: {error}", output.display()))
}
