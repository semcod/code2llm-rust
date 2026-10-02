//! Cyclomatic complexity and function boundary extraction.

use rayon::prelude::*;

/// Extract the body of a function between braces from pre-split lines (1-indexed start_line).
pub fn extract_function_body_from_lines(lines: &[&str], start_line: usize) -> String {
    if start_line < 1 || start_line > lines.len() {
        return String::new();
    }

    let mut depth: i32 = 0;
    let mut started = false;
    let mut body_lines: Vec<&str> = Vec::new();

    for &line in &lines[start_line - 1..] {
        for ch in line.chars() {
            if ch == '{' {
                depth += 1;
                started = true;
            } else if ch == '}' {
                depth -= 1;
            }
        }
        if started {
            body_lines.push(line);
        }
        if started && depth <= 0 {
            break;
        }
    }

    body_lines.join("\n")
}

/// Extract the body of a function between braces starting from start_line (1-indexed).
pub fn extract_function_body(content: &str, start_line: usize) -> String {
    let lines: Vec<&str> = content.split('\n').collect();
    extract_function_body_from_lines(&lines, start_line)
}

/// Compute McCabe cyclomatic complexity for a function body according to language rules.
pub fn calculate_complexity(body: &str, lang: &str) -> (u32, &'static str) {
    if body.is_empty() {
        return (1, "A");
    }

    let count = match lang {
        "go" => count_go_branches(body),
        "rust" => count_rust_branches(body),
        _ => count_c_family_branches(body),
    };

    let cc = 1 + count;
    let rank = if cc <= 5 {
        "A"
    } else if cc <= 10 {
        "B"
    } else if cc <= 20 {
        "C"
    } else {
        "D"
    };

    (cc, rank)
}

/// Batch compute cyclomatic complexity in parallel using Rayon.
pub fn batch_complexity(
    content: &str,
    lines: &[usize],
    lang: &str,
) -> Vec<(usize, u32, &'static str)> {
    let lines_vec: Vec<&str> = content.split('\n').collect();
    lines
        .par_iter()
        .map(|&start_line| {
            let body = extract_function_body_from_lines(&lines_vec, start_line);
            let (cc, rank) = calculate_complexity(&body, lang);
            (start_line, cc, rank)
        })
        .collect()
}

fn count_c_family_branches(body: &str) -> u32 {
    let bytes = body.as_bytes();
    let len = bytes.len();
    let mut count = 0;
    let mut i = 0;

    while i < len {
        if i + 1 < len {
            if bytes[i] == b'&' && bytes[i + 1] == b'&' {
                count += 1;
                i += 2;
                continue;
            }
            if bytes[i] == b'|' && bytes[i + 1] == b'|' {
                count += 1;
                i += 2;
                continue;
            }
            if bytes[i] == b'?' && bytes[i + 1] == b'?' {
                count += 1;
                i += 2;
                continue;
            }
            if bytes[i] == b'?' && bytes[i + 1] == b'.' {
                count += 1;
                i += 2;
                continue;
            }
        }

        // Ternary operator ? ... :
        if bytes[i] == b'?' {
            let mut j = i + 1;
            let mut found_colon = false;
            while j < len && bytes[j] != b'\n' && bytes[j] != b';' && bytes[j] != b'{' && bytes[j] != b'}' {
                if bytes[j] == b':' {
                    found_colon = true;
                    break;
                }
                j += 1;
            }
            if found_colon {
                count += 1;
                i = j + 1;
                continue;
            }
        }

        let prev_char = if i > 0 { Some(bytes[i - 1] as char) } else { None };
        let prev_word = prev_char.map_or(false, |c| c.is_alphanumeric() || c == '_');

        if !prev_word {
            // Check else if
            if body[i..].starts_with("else") {
                let rest = &body[i + 4..];
                let trimmed = rest.trim_start();
                if trimmed.starts_with("if") {
                    let after = trimmed[2..].chars().next();
                    let after_word = after.map_or(false, |c| c.is_alphanumeric() || c == '_');
                    if !after_word {
                        count += 1;
                        let skipped = (len - rest.len()) + (rest.len() - trimmed.len()) + 2;
                        i = skipped;
                        continue;
                    }
                }
            }

            let keywords = ["if", "for", "while", "do", "switch", "case", "catch"];
            let mut matched = false;
            for &kw in &keywords {
                if body[i..].starts_with(kw) {
                    let next_char = body[i + kw.len()..].chars().next();
                    let next_word = next_char.map_or(false, |c| c.is_alphanumeric() || c == '_');
                    if !next_word {
                        count += 1;
                        i += kw.len();
                        matched = true;
                        break;
                    }
                }
            }
            if matched {
                continue;
            }
        }

        i += 1;
    }

    count
}

fn count_go_branches(body: &str) -> u32 {
    let bytes = body.as_bytes();
    let len = bytes.len();
    let mut count = 0;
    let mut i = 0;

    while i < len {
        if i + 1 < len {
            if (bytes[i] == b'&' && bytes[i + 1] == b'&') || (bytes[i] == b'|' && bytes[i + 1] == b'|') {
                count += 1;
                i += 2;
                continue;
            }
        }

        let prev_char = if i > 0 { Some(bytes[i - 1] as char) } else { None };
        let prev_word = prev_char.map_or(false, |c| c.is_alphanumeric() || c == '_');

        if !prev_word {
            let keywords = ["if", "for", "switch", "case", "select", "go", "defer"];
            let mut matched = false;
            for &kw in &keywords {
                if body[i..].starts_with(kw) {
                    let next_char = body[i + kw.len()..].chars().next();
                    let next_word = next_char.map_or(false, |c| c.is_alphanumeric() || c == '_');
                    if !next_word {
                        count += 1;
                        i += kw.len();
                        matched = true;
                        break;
                    }
                }
            }
            if matched {
                continue;
            }
        }

        i += 1;
    }

    count
}

fn count_rust_branches(body: &str) -> u32 {
    let bytes = body.as_bytes();
    let len = bytes.len();
    let mut count = 0;
    let mut i = 0;

    while i < len {
        if i + 1 < len {
            if (bytes[i] == b'&' && bytes[i + 1] == b'&') || (bytes[i] == b'|' && bytes[i + 1] == b'|') {
                count += 1;
                i += 2;
                continue;
            }
        }

        if bytes[i] == b'?' {
            count += 1;
            i += 1;
            continue;
        }

        let prev_char = if i > 0 { Some(bytes[i - 1] as char) } else { None };
        let prev_word = prev_char.map_or(false, |c| c.is_alphanumeric() || c == '_');

        if !prev_word {
            if body[i..].starts_with("else") {
                let rest = &body[i + 4..];
                let trimmed = rest.trim_start();
                if trimmed.starts_with("if") {
                    let after = trimmed[2..].chars().next();
                    let after_word = after.map_or(false, |c| c.is_alphanumeric() || c == '_');
                    if !after_word {
                        count += 1;
                        let skipped = (len - rest.len()) + (rest.len() - trimmed.len()) + 2;
                        i = skipped;
                        continue;
                    }
                }
            }

            let keywords = ["if", "for", "while", "loop", "match"];
            let mut matched = false;
            for &kw in &keywords {
                if body[i..].starts_with(kw) {
                    let next_char = body[i + kw.len()..].chars().next();
                    let next_word = next_char.map_or(false, |c| c.is_alphanumeric() || c == '_');
                    if !next_word {
                        count += 1;
                        i += kw.len();
                        matched = true;
                        break;
                    }
                }
            }
            if matched {
                continue;
            }
        }

        i += 1;
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_function_body() {
        let code = "fn main() {\n    let x = 10;\n    if x > 5 {\n        println!(\"hello\");\n    }\n}\n\nfn foo() {}";
        let body = extract_function_body(code, 1);
        assert!(body.starts_with("fn main() {"));
        assert!(body.ends_with("}"));
        assert!(!body.contains("fn foo"));
    }

    #[test]
    fn test_c_family_complexity() {
        let body = "{\n  if (a && b) {\n    while (c) { doWork(); }\n  }\n  else if (d || e) {\n    switch (x) { case 1: break; }\n  }\n}";
        let (cc, rank) = calculate_complexity(body, "c_family");
        assert_eq!(cc, 8);
        assert_eq!(rank, "B");
    }

    #[test]
    fn test_batch_complexity() {
        let code = "fn a() { if (x) {} }\nfn b() { while (y) { if (z) {} } }";
        let res = batch_complexity(code, &[1, 2], "c_family");
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].1, 2);
        assert_eq!(res[1].1, 3);
    }
}
