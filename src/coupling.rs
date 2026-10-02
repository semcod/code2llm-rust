//! Fast module coupling analysis (Ca, Ce, Instability).

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct ModuleCouplingMetrics {
    pub module: String,
    pub afferent: usize, // Ca: external modules calling this module
    pub efferent: usize, // Ce: external modules this module calls
    pub instability: f64, // I = Ce / (Ca + Ce)
    pub dependencies: Vec<String>,
}

/// Compute module interactions and coupling metrics across modules.
pub fn compute_module_coupling(
    func_modules: &[(String, String)], // (func_qname, module_name)
    calls: &[(String, String)],        // (caller_qname, callee_qname)
) -> (HashMap<String, Vec<String>>, HashMap<String, (usize, usize, f64)>) {
    let mut func_to_mod: HashMap<&str, &str> = HashMap::with_capacity(func_modules.len());
    let mut all_modules: HashSet<&str> = HashSet::new();

    for (func, module) in func_modules {
        func_to_mod.insert(func.as_str(), module.as_str());
        all_modules.insert(module.as_str());
    }

    // Module interactions: caller_mod -> set of callee_mods
    let mut outgoing: HashMap<&str, HashSet<&str>> = HashMap::new();
    let mut incoming: HashMap<&str, HashSet<&str>> = HashMap::new();

    for m in &all_modules {
        outgoing.insert(m, HashSet::new());
        incoming.insert(m, HashSet::new());
    }

    for (caller, callee) in calls {
        if let (Some(&mod_a), Some(&mod_b)) = (func_to_mod.get(caller.as_str()), func_to_mod.get(callee.as_str())) {
            if mod_a != mod_b {
                outgoing.get_mut(mod_a).unwrap().insert(mod_b);
                incoming.get_mut(mod_b).unwrap().insert(mod_a);
            }
        }
    }

    let mut interactions: HashMap<String, Vec<String>> = HashMap::new();
    let mut metrics: HashMap<String, (usize, usize, f64)> = HashMap::new();

    for &m in &all_modules {
        let mut deps: Vec<String> = outgoing[m].iter().map(|s| s.to_string()).collect();
        deps.sort();

        let ce = deps.len();
        let ca = incoming[m].len();
        let total = (ca + ce) as f64;
        let inst = if total > 0.0 { (ce as f64) / total } else { 0.0 };

        interactions.insert(m.to_string(), deps);
        metrics.insert(m.to_string(), (ca, ce, inst));
    }

    (interactions, metrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coupling() {
        let funcs = vec![
            ("mod_a.f1".to_string(), "mod_a".to_string()),
            ("mod_b.f2".to_string(), "mod_b".to_string()),
        ];
        let calls = vec![("mod_a.f1".to_string(), "mod_b.f2".to_string())];

        let (inter, metrics) = compute_module_coupling(&funcs, &calls);
        assert_eq!(inter["mod_a"], vec!["mod_b"]);
        assert_eq!(inter["mod_b"], Vec::<String>::new());

        let (ca_a, ce_a, inst_a) = metrics["mod_a"];
        assert_eq!(ca_a, 0);
        assert_eq!(ce_a, 1);
        assert_eq!(inst_a, 1.0);

        let (ca_b, ce_b, inst_b) = metrics["mod_b"];
        assert_eq!(ca_b, 1);
        assert_eq!(ce_b, 0);
        assert_eq!(inst_b, 0.0);
    }
}
