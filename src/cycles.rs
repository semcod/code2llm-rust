//! Cycle and circular dependency detection using Tarjan's Strongly Connected Components (SCC).

use std::collections::HashMap;

/// Result of circular dependency analysis.
#[derive(Debug, Clone)]
pub struct CycleGroup {
    pub members: Vec<String>,
}

/// Detect circular dependencies using Tarjan's SCC algorithm in linear time O(V + E).
///
/// Returns groups of mutually dependent nodes. Does not suffer from exponential slowdown
/// on large graphs, unlike NetworkX `simple_cycles`.
pub fn detect_circular_dependencies(
    nodes: &[String],
    edges: &[(String, String)],
) -> Vec<Vec<String>> {
    let n = nodes.len();
    if n < 2 {
        return Vec::new();
    }

    let mut node_to_idx: HashMap<&str, usize> = HashMap::with_capacity(n);
    for (i, name) in nodes.iter().enumerate() {
        node_to_idx.insert(name.as_str(), i);
    }

    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut self_loops: Vec<bool> = vec![false; n];

    for (u_name, v_name) in edges {
        if let (Some(&u), Some(&v)) = (node_to_idx.get(u_name.as_str()), node_to_idx.get(v_name.as_str())) {
            if u == v {
                self_loops[u] = true;
            } else {
                adj[u].push(v);
            }
        }
    }

    // Tarjan's SCC state
    let mut index = 0;
    let mut indices: Vec<Option<usize>> = vec![None; n];
    let mut lowlink: Vec<usize> = vec![0; n];
    let mut on_stack: Vec<bool> = vec![false; n];
    let mut stack: Vec<usize> = Vec::with_capacity(n);
    let mut cycles: Vec<Vec<String>> = Vec::new();

    for u in 0..n {
        if indices[u].is_none() {
            strongconnect(
                u,
                &adj,
                &self_loops,
                &mut index,
                &mut indices,
                &mut lowlink,
                &mut on_stack,
                &mut stack,
                nodes,
                &mut cycles,
            );
        }
    }

    cycles
}

fn strongconnect(
    u: usize,
    adj: &[Vec<usize>],
    self_loops: &[bool],
    index: &mut usize,
    indices: &mut [Option<usize>],
    lowlink: &mut [usize],
    on_stack: &mut [bool],
    stack: &mut Vec<usize>,
    nodes: &[String],
    cycles: &mut Vec<Vec<String>>,
) {
    indices[u] = Some(*index);
    lowlink[u] = *index;
    *index += 1;
    stack.push(u);
    on_stack[u] = true;

    for &v in &adj[u] {
        if indices[v].is_none() {
            strongconnect(
                v,
                adj,
                self_loops,
                index,
                indices,
                lowlink,
                on_stack,
                stack,
                nodes,
                cycles,
            );
            lowlink[u] = lowlink[u].min(lowlink[v]);
        } else if on_stack[v] {
            lowlink[u] = lowlink[u].min(indices[v].unwrap());
        }
    }

    // If u is a root node of an SCC, pop the stack and generate an SCC
    if lowlink[u] == indices[u].unwrap() {
        let mut scc: Vec<String> = Vec::new();
        loop {
            let v = stack.pop().unwrap();
            on_stack[v] = false;
            scc.push(nodes[v].clone());
            if v == u {
                break;
            }
        }

        // An SCC is a circular dependency if it contains > 1 node, or 1 node with a self-loop
        if scc.len() > 1 || (scc.len() == 1 && self_loops[u]) {
            scc.reverse();
            cycles.push(scc);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_cycle() {
        let nodes = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let edges = vec![
            ("A".to_string(), "B".to_string()),
            ("B".to_string(), "C".to_string()),
        ];
        let cycles = detect_circular_dependencies(&nodes, &edges);
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_single_cycle() {
        let nodes = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let edges = vec![
            ("A".to_string(), "B".to_string()),
            ("B".to_string(), "C".to_string()),
            ("C".to_string(), "A".to_string()),
        ];
        let cycles = detect_circular_dependencies(&nodes, &edges);
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 3);
    }
}
