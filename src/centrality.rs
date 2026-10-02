//! Fast betweenness centrality computation using Brandes' algorithm.

use std::collections::{HashMap, VecDeque};
use rayon::prelude::*;

/// Compute betweenness centrality for a directed graph using Brandes' algorithm.
///
/// Supports optional node sampling `k` (similar to NetworkX betweenness_centrality(G, k=k))
/// and optional normalization. Runs in parallel across source nodes using Rayon.
pub fn betweenness_centrality(
    nodes: &[String],
    edges: &[(String, String)],
    k: Option<usize>,
    normalized: bool,
) -> HashMap<String, f64> {
    let n = nodes.len();
    if n == 0 {
        return HashMap::new();
    }
    if n == 1 {
        let mut res = HashMap::new();
        res.insert(nodes[0].clone(), 0.0);
        return res;
    }

    // Map node names to integer indices
    let mut node_to_idx: HashMap<&str, usize> = HashMap::with_capacity(n);
    for (i, name) in nodes.iter().enumerate() {
        node_to_idx.insert(name.as_str(), i);
    }

    // Build adjacency list
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (u_name, v_name) in edges {
        if let (Some(&u), Some(&v)) = (node_to_idx.get(u_name.as_str()), node_to_idx.get(v_name.as_str())) {
            adj[u].push(v);
        }
    }

    // Select source nodes (all or sampled subset)
    let sources: Vec<usize> = match k {
        Some(limit) if limit < n => {
            // Evenly spaced sampling across nodes
            let step = (n as f64) / (limit as f64);
            (0..limit).map(|i| (i as f64 * step) as usize).collect()
        }
        _ => (0..n).collect(),
    };

    let sample_count = sources.len();

    // Parallel Brandes accumulation across sources
    let total_betweenness: Vec<f64> = sources
        .par_iter()
        .fold(
            || vec![0.0f64; n],
            |mut local_cb, &s| {
                let mut stack: Vec<usize> = Vec::with_capacity(n);
                let mut pred: Vec<Vec<usize>> = vec![Vec::new(); n];
                let mut sigma: Vec<u64> = vec![0; n];
                sigma[s] = 1;

                let mut dist: Vec<i32> = vec![-1; n];
                dist[s] = 0;

                let mut queue: VecDeque<usize> = VecDeque::with_capacity(n);
                queue.push_back(s);

                // BFS to find shortest paths
                while let Some(v) = queue.pop_front() {
                    stack.push(v);
                    let d_v = dist[v];
                    let sigma_v = sigma[v];

                    for &w in &adj[v] {
                        if dist[w] < 0 {
                            dist[w] = d_v + 1;
                            queue.push_back(w);
                        }
                        if dist[w] == d_v + 1 {
                            sigma[w] = sigma[w].saturating_add(sigma_v);
                            pred[w].push(v);
                        }
                    }
                }

                // Accumulate dependencies
                let mut delta: Vec<f64> = vec![0.0f64; n];
                while let Some(w) = stack.pop() {
                    let sigma_w = sigma[w] as f64;
                    if sigma_w > 0.0 {
                        let coeff = (1.0 + delta[w]) / sigma_w;
                        for &v in &pred[w] {
                            delta[v] += (sigma[v] as f64) * coeff;
                        }
                    }
                    if w != s {
                        local_cb[w] += delta[w];
                    }
                }

                local_cb
            },
        )
        .reduce(
            || vec![0.0f64; n],
            |mut a, b| {
                for (x, y) in a.iter_mut().zip(b.iter()) {
                    *x += y;
                }
                a
            },
        );

    // Scaling factor for sampling and normalization
    let scale = if sample_count < n {
        (n as f64) / (sample_count as f64)
    } else {
        1.0
    };

    let norm_factor = if normalized && n > 2 {
        1.0 / ((n - 1) as f64 * (n - 2) as f64)
    } else {
        1.0
    };

    let mut result: HashMap<String, f64> = HashMap::with_capacity(n);
    for (i, node_name) in nodes.iter().enumerate() {
        let val = total_betweenness[i] * scale * norm_factor;
        result.insert(node_name.clone(), val);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_graph_centrality() {
        let nodes = vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()];
        let edges = vec![
            ("A".to_string(), "B".to_string()),
            ("B".to_string(), "C".to_string()),
            ("C".to_string(), "D".to_string()),
        ];
        let cb = betweenness_centrality(&nodes, &edges, None, true);
        assert!((cb["B"] - 1.0 / 3.0).abs() < 1e-6);
        assert!((cb["C"] - 1.0 / 3.0).abs() < 1e-6);
        assert_eq!(cb["A"], 0.0);
        assert_eq!(cb["D"], 0.0);
    }
}
