//! Fast in-memory graph reachability and orphan/dead-code detection.

use std::collections::{HashMap, VecDeque};

/// Compute reachability for all functions/nodes from declared entry points.
///
/// Returns a map of node_name -> "reachable" | "unreachable".
/// Runs in O(V + E) time with zero disk I/O, completely replacing slow external scanners.
pub fn compute_reachability(
    nodes: &[String],
    edges: &[(String, String)],
    entry_points: &[String],
) -> HashMap<String, String> {
    let n = nodes.len();
    let mut node_to_idx: HashMap<&str, usize> = HashMap::with_capacity(n);
    for (i, name) in nodes.iter().enumerate() {
        node_to_idx.insert(name.as_str(), i);
    }

    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (u_name, v_name) in edges {
        if let (Some(&u), Some(&v)) = (node_to_idx.get(u_name.as_str()), node_to_idx.get(v_name.as_str())) {
            adj[u].push(v);
        }
    }

    let mut visited: Vec<bool> = vec![false; n];
    let mut queue: VecDeque<usize> = VecDeque::new();

    for ep in entry_points {
        if let Some(&idx) = node_to_idx.get(ep.as_str()) {
            if !visited[idx] {
                visited[idx] = true;
                queue.push_back(idx);
            }
        }
    }

    // BFS along call edges
    while let Some(u) = queue.pop_front() {
        for &v in &adj[u] {
            if !visited[v] {
                visited[v] = true;
                queue.push_back(v);
            }
        }
    }

    let mut result: HashMap<String, String> = HashMap::with_capacity(n);
    for (i, name) in nodes.iter().enumerate() {
        let status = if visited[i] {
            "reachable".to_string()
        } else {
            "unreachable".to_string()
        };
        result.insert(name.clone(), status);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reachability() {
        let nodes = vec![
            "main".to_string(),
            "helper".to_string(),
            "unused".to_string(),
        ];
        let edges = vec![("main".to_string(), "helper".to_string())];
        let entry_points = vec!["main".to_string()];

        let res = compute_reachability(&nodes, &edges, &entry_points);
        assert_eq!(res["main"], "reachable");
        assert_eq!(res["helper"], "reachable");
        assert_eq!(res["unused"], "unreachable");
    }
}
