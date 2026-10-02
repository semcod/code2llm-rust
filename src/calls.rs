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
