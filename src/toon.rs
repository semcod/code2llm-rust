//! Fast TOON format rendering and buffer serialization.

/// Format header lines for analysis.toon.yaml.
pub fn format_toon_header(
    nfiles: usize,
    total_lines: usize,
    lang_label: &str,
    timestamp: &str,
    avg_cc: f64,
    critical_cc: usize,
    total_funcs: usize,
    dups: usize,
    cycles: usize,
) -> Vec<String> {
    vec![
        format!(
            "# code2llm | {}f {}L | {} | {}",
            nfiles, total_lines, lang_label, timestamp
        ),
        format!(
            "# CC\u{0305}={:.1} | critical:{}/{} | dups:{} | cycles:{}",
            avg_cc, critical_cc, total_funcs, dups, cycles
        ),
    ]
}

/// Format function rows for FUNCTIONS section (CC >= threshold).
pub fn format_function_rows(
    functions: &[(String, String, usize, u32, &str)], // (name, rel_file, line, cc, rank)
    limit: usize,
) -> Vec<String> {
    functions
        .iter()
        .take(limit)
        .map(|(name, file, line, cc, rank)| {
            format!("  {:<32} {:>3} [{}] {}:{}", name, cc, rank, file, line)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_toon_header() {
        let lines = format_toon_header(10, 500, "Python", "2026-10-02", 3.5, 2, 50, 1, 0);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("10f 500L"));
        assert!(lines[1].contains("critical:2/50"));
    }
}
