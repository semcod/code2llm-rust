//! Fast code smell and anti-pattern detection algorithms.

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct GodFunctionCandidate {
    pub name: String,
    pub file: String,
    pub line: usize,
    pub fan_out: usize,
    pub mutations: usize,
    pub complexity: u32,
    pub severity: f64,
}

/// Detect god functions from metrics and complexity.
pub fn detect_god_functions_native(
    functions: &[(String, String, usize, usize, usize, u32)], // (name, file, line, fan_out, mutations, cc)
) -> Vec<GodFunctionCandidate> {
    let mut smells = Vec::new();

    for (name, file, line, fan_out, mutations, cc) in functions {
        let fo = *fan_out;
        let mut_cnt = *mutations;
        let complexity = *cc;

        if fo > 10 || mut_cnt > 6 || complexity > 12 {
            let severity = ((fo as f64 / 20.0) * 0.3
                + (mut_cnt as f64 / 15.0) * 0.3
                + (complexity as f64 / 30.0) * 0.4)
                .min(1.0);

            smells.push(GodFunctionCandidate {
                name: name.clone(),
                file: file.clone(),
                line: *line,
                fan_out: fo,
                mutations: mut_cnt,
                complexity,
                severity,
            });
        }
    }

    smells
}

/// Detect data clumps: groups of parameters (>= min_size) shared by multiple functions (>= min_occurrences).
pub fn detect_data_clumps_native(
    func_params: &[(String, Vec<String>)],
    min_size: usize,
    min_occurrences: usize,
) -> Vec<(Vec<String>, Vec<String>)> {
    let mut clumps = Vec::new();
    let n = func_params.len();
    if n < min_occurrences {
        return clumps;
    }

    let param_sets: Vec<(&str, HashSet<&str>)> = func_params
        .iter()
        .map(|(name, params)| (name.as_str(), params.iter().map(|s| s.as_str()).collect()))
        .collect();

    // Map common parameter signature tuples -> list of function names
    let mut clump_map: HashMap<Vec<String>, Vec<String>> = HashMap::new();

    for i in 0..n {
        for j in (i + 1)..n {
            let common: Vec<String> = param_sets[i]
                .1
                .intersection(&param_sets[j].1)
                .map(|s| s.to_string())
                .collect();

            if common.len() >= min_size {
                let mut sorted_common = common;
                sorted_common.sort();

                let entry = clump_map.entry(sorted_common).or_default();
                let f1 = param_sets[i].0.to_string();
                let f2 = param_sets[j].0.to_string();
                if !entry.contains(&f1) {
                    entry.push(f1);
                }
                if !entry.contains(&f2) {
                    entry.push(f2);
                }
            }
        }
    }

    for (params, funcs) in clump_map {
        if funcs.len() >= min_occurrences {
            clumps.push((params, funcs));
        }
    }

    clumps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_god_functions() {
        let funcs = vec![
            ("normal".to_string(), "a.py".to_string(), 1, 2, 1, 3),
            ("huge".to_string(), "b.py".to_string(), 10, 25, 10, 35),
        ];
        let smells = detect_god_functions_native(&funcs);
        assert_eq!(smells.len(), 1);
        assert_eq!(smells[0].name, "huge");
        assert!(smells[0].severity > 0.8);
    }

    #[test]
    fn test_detect_data_clumps() {
        let params = vec![
            ("f1".to_string(), vec!["a".to_string(), "b".to_string(), "c".to_string()]),
            ("f2".to_string(), vec!["a".to_string(), "b".to_string(), "c".to_string()]),
        ];
        let clumps = detect_data_clumps_native(&params, 3, 2);
        assert_eq!(clumps.len(), 1);
        assert_eq!(clumps[0].0, vec!["a", "b", "c"]);
        assert_eq!(clumps[0].1.len(), 2);
    }
}
