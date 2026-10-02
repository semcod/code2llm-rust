//! Call graph pipeline path detection.

use std::collections::{HashMap, HashSet};

/// Detect candidate pipeline paths in a directed call graph.
pub fn find_pipeline_paths(
    nodes: &[String],
    edges: &[(String, String)],
    min_length: usize,
    max_pipelines: usize,
) -> Vec<Vec<String>> {
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::with_capacity(nodes.len());
    let mut in_degrees: HashMap<&str, usize> = HashMap::with_capacity(nodes.len());

    for node in nodes {
        adj.entry(node.as_str()).or_default();
        in_degrees.entry(node.as_str()).or_insert(0);
    }

    for (src, dst) in edges {
        adj.entry(src.as_str()).or_default().push(dst.as_str());
        *in_degrees.entry(dst.as_str()).or_insert(0) += 1;
    }

    // Sources: nodes with in-degree == 0
    let mut sources: Vec<&str> = nodes
        .iter()
        .map(|s| s.as_str())
        .filter(|&n| in_degrees.get(n).copied().unwrap_or(0) == 0)
        .collect();

    if sources.is_empty() {
        let mut sorted_nodes: Vec<&str> = nodes.iter().map(|s| s.as_str()).collect();
        sorted_nodes.sort_by_key(|&n| in_degrees.get(n).copied().unwrap_or(0));
        sources = sorted_nodes.into_iter().take(5).collect();
    }

    let mut paths: Vec<Vec<String>> = Vec::new();
    let mut used_nodes: HashSet<&str> = HashSet::new();

    // 1. Longest paths from sources
    for source in sources {
        let best_path = longest_path_from(&adj, source, 10);
        if best_path.len() >= min_length {
            used_nodes.extend(best_path.iter().copied());
            paths.push(best_path.into_iter().map(|s| s.to_string()).collect());
        }
    }

    // 2. Weakly connected components
    let components = find_weakly_connected_components(nodes, edges);
    for comp in components {
        if comp.len() < min_length {
            continue;
        }

        let used_overlap = comp.iter().filter(|&&n| used_nodes.contains(n)).count();
        if used_overlap > comp.len() / 2 {
            continue;
        }

        let comp_set: HashSet<&str> = comp.into_iter().collect();
        let mut comp_adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for &n in &comp_set {
            if let Some(succs) = adj.get(n) {
                let filtered: Vec<&str> = succs.iter().copied().filter(|s| comp_set.contains(s)).collect();
                comp_adj.insert(n, filtered);
            }
        }

        let mut comp_sources: Vec<&str> = comp_set
            .iter()
            .copied()
            .filter(|&n| in_degrees.get(n).copied().unwrap_or(0) == 0)
            .collect();
        if comp_sources.is_empty() {
            comp_sources = comp_set.iter().copied().take(3).collect();
        }

        let mut best: Vec<&str> = Vec::new();
        for src in comp_sources {
            let p = longest_path_from(&comp_adj, src, 10);
            if p.len() > best.len() {
                best = p;
            }
        }

        if best.len() >= min_length {
            let overlap = best.iter().filter(|&&n| used_nodes.contains(n)).count();
            if overlap <= best.len() / 2 {
                used_nodes.extend(best.iter().copied());
                paths.push(best.into_iter().map(|s| s.to_string()).collect());
            }
        }
    }

    // Sort by path length descending
    paths.sort_by(|a, b| b.len().cmp(&a.len()));
    if paths.len() > max_pipelines {
        paths.truncate(max_pipelines);
    }

    paths
}

fn longest_path_from<'a>(
    adj: &HashMap<&'a str, Vec<&'a str>>,
    source: &'a str,
    max_depth: usize,
) -> Vec<&'a str> {
    let mut best: Vec<&'a str> = vec![source];
    let mut stack: Vec<(&'a str, Vec<&'a str>)> = vec![(source, vec![source])];

    while let Some((curr, path)) = stack.pop() {
        if path.len() > best.len() {
            best = path.clone();
        }

        if path.len() >= max_depth {
            continue;
        }

        if let Some(succs) = adj.get(curr) {
            for &succ in succs {
                if !path.contains(&succ) {
                    let mut next_path = path.clone();
                    next_path.push(succ);
                    stack.push((succ, next_path));
                }
            }
        }
    }

    best
}

fn find_weakly_connected_components<'a>(
    nodes: &'a [String],
    edges: &'a [(String, String)],
) -> Vec<Vec<&'a str>> {
    let mut undirected_adj: HashMap<&str, Vec<&str>> = HashMap::with_capacity(nodes.len());
    for node in nodes {
        undirected_adj.entry(node.as_str()).or_default();
    }
    for (src, dst) in edges {
        undirected_adj.entry(src.as_str()).or_default().push(dst.as_str());
        undirected_adj.entry(dst.as_str()).or_default().push(src.as_str());
    }

    let mut visited: HashSet<&str> = HashSet::new();
    let mut components = Vec::new();

    for node in nodes {
        let node_str = node.as_str();
        if !visited.contains(node_str) {
            let mut component = Vec::new();
            let mut queue = vec![node_str];
            visited.insert(node_str);

            while let Some(curr) = queue.pop() {
                component.push(curr);
                if let Some(neighbors) = undirected_adj.get(curr) {
                    for &next in neighbors {
                        if !visited.contains(next) {
                            visited.insert(next);
                            queue.push(next);
                        }
                    }
                }
            }
            components.push(component);
        }
    }

    components
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_pipeline_paths() {
        let nodes = vec![
            "stage_a".to_string(),
            "stage_b".to_string(),
            "stage_c".to_string(),
            "stage_d".to_string(),
            "isolated".to_string(),
        ];
        let edges = vec![
            ("stage_a".to_string(), "stage_b".to_string()),
            ("stage_b".to_string(), "stage_c".to_string()),
            ("stage_c".to_string(), "stage_d".to_string()),
        ];

        let paths = find_pipeline_paths(&nodes, &edges, 3, 5);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec!["stage_a", "stage_b", "stage_c", "stage_d"]);
    }
}
