//! Call extraction from function bodies.

use rayon::prelude::*;
use crate::complexity::extract_function_body_from_lines;

pub const CALL_KEYWORDS: &[&str] = &[
    "if", "for", "while", "switch", "catch", "return", "throw", "new",
    "typeof", "instanceof", "import", "export", "require", "console",
    "super", "class", "function", "async", "await", "delete", "void",
    "case", "default",
];

/// Extract function call identifiers from a function body string.
pub fn extract_calls_from_body(body: &str) -> Vec<String> {
    let bytes = body.as_bytes();
    let len = bytes.len();
    let mut calls = Vec::new();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'(' {
            let mut end = i;
            while end > 0 && bytes[end - 1].is_ascii_whitespace() {
                end -= 1;
            }
            let mut start = end;
            while start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
                start -= 1;
            }

            if start < end {
                let ident = &body[start..end];
                if !CALL_KEYWORDS.contains(&ident) {
                    let mut prev_word_end = start;
                    while prev_word_end > 0 && bytes[prev_word_end - 1].is_ascii_whitespace() {
                        prev_word_end -= 1;
                    }
                    let mut prev_word_start = prev_word_end;
                    while prev_word_start > 0 && (bytes[prev_word_start - 1].is_ascii_alphabetic() || bytes[prev_word_start - 1] == b'_') {
                        prev_word_start -= 1;
                    }
                    let prev_word = if prev_word_start < prev_word_end {
                        &body[prev_word_start..prev_word_end]
                    } else {
                        ""
                    };

                    if prev_word != "function" && prev_word != "class" {
                        calls.push(ident.to_string());
                    }
                }
            }
        }
        i += 1;
    }

    calls
}

/// Batch extract calls across lines in parallel.
pub fn batch_calls(
    content: &str,
    lines: &[usize],
) -> Vec<(usize, Vec<String>)> {
    let lines_vec: Vec<&str> = content.split('\n').collect();
    lines
        .par_iter()
        .map(|&start_line| {
            let body = extract_function_body_from_lines(&lines_vec, start_line);
            let calls = extract_calls_from_body(&body);
            (start_line, calls)
        })
        .collect()
}

use std::collections::{HashMap, HashSet};

/// Result of full call graph resolution.
pub struct CallGraphResolution {
    pub resolved_calls: HashMap<String, Vec<String>>,
    pub called_by: HashMap<String, Vec<String>>,
    pub entry_points: Vec<String>,
}

/// Fast native call graph edge resolution matching ProjectAnalyzer._build_call_graph.
pub fn resolve_call_graph(
    functions_with_calls: &[(String, Vec<String>)],
) -> CallGraphResolution {
    let known_functions: HashSet<&str> = functions_with_calls
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();

    let mut simple_to_full: HashMap<&str, Vec<&str>> = HashMap::new();
    for (name, _) in functions_with_calls {
        let simple = match name.rfind('.') {
            Some(idx) => &name[idx + 1..],
            None => name.as_str(),
        };
        simple_to_full.entry(simple).or_default().push(name.as_str());
    }

    let mut resolved_calls: HashMap<String, Vec<String>> = HashMap::new();
    let mut called_by: HashMap<String, Vec<String>> = HashMap::new();

    for (name, _) in functions_with_calls {
        called_by.entry(name.clone()).or_default();
    }

    for (func_name, calls) in functions_with_calls {
        let func_module = match func_name.rfind('.') {
            Some(idx) => &func_name[..idx],
            None => "",
        };

        let mut res_calls = Vec::with_capacity(calls.len());
        for called in calls {
            let called_str = called.as_str();
            let resolved_opt = if known_functions.contains(called_str) {
                Some(called_str)
            } else if let Some(candidates) = simple_to_full.get(called_str) {
                let mut best = candidates[0];
                for &cand in candidates {
                    let cand_module = match cand.rfind('.') {
                        Some(idx) => &cand[..idx],
                        None => "",
                    };
                    if cand_module == func_module {
                        best = cand;
                        break;
                    }
                }
                Some(best)
            } else {
                None
            };

            if let Some(resolved) = resolved_opt {
                res_calls.push(resolved.to_string());
                called_by
                    .entry(resolved.to_string())
                    .or_default()
                    .push(func_name.clone());
            }
        }
        resolved_calls.insert(func_name.clone(), res_calls);
    }

    let mut entry_points: Vec<String> = Vec::new();
    for (name, callers) in &called_by {
        if callers.is_empty() {
            entry_points.push(name.clone());
        }
    }
    entry_points.sort();

    CallGraphResolution {
        resolved_calls,
        called_by,
        entry_points,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_call_graph() {
        let input = vec![
            ("app.main".to_string(), vec!["helper".to_string()]),
            ("app.helper".to_string(), vec!["db.query".to_string()]),
            ("db.query".to_string(), vec![]),
        ];

        let res = resolve_call_graph(&input);
        assert_eq!(res.resolved_calls["app.main"], vec!["app.helper"]);
        assert_eq!(res.resolved_calls["app.helper"], vec!["db.query"]);
        assert_eq!(res.called_by["app.helper"], vec!["app.main"]);
        assert_eq!(res.called_by["db.query"], vec!["app.helper"]);
        assert_eq!(res.entry_points, vec!["app.main"]);
    }
}

