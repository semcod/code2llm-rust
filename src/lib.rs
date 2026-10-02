//! Python native extension bindings for code2llm acceleration.

use pyo3::prelude::*;
use std::collections::HashMap;

pub mod calls;
pub mod centrality;
pub mod complexity;
pub mod coupling;
pub mod cycles;
pub mod discovery;
pub mod reachability;
pub mod smells;
pub mod cache;
pub mod toon;

/// Extract the body of a function between braces starting at `start_line` (1-indexed).
#[pyfunction]
#[pyo3(signature = (content, start_line))]
fn extract_function_body(content: &str, start_line: usize) -> String {
    complexity::extract_function_body(content, start_line)
}

/// Compute McCabe cyclomatic complexity and rank for a function body.
#[pyfunction]
#[pyo3(signature = (body, lang = "c_family"))]
fn calculate_complexity(body: &str, lang: &str) -> (u32, &'static str) {
    complexity::calculate_complexity(body, lang)
}

/// Batch compute cyclomatic complexity in parallel across start lines.
#[pyfunction]
#[pyo3(signature = (content, lines, lang = "c_family"))]
fn batch_complexity(
    py: Python<'_>,
    content: &str,
    lines: Vec<usize>,
    lang: &str,
) -> Vec<(usize, u32, &'static str)> {
    py.allow_threads(|| complexity::batch_complexity(content, &lines, lang))
}

/// Extract function call identifiers from a function body string.
#[pyfunction]
#[pyo3(signature = (body))]
fn extract_calls(body: &str) -> Vec<String> {
    calls::extract_calls_from_body(body)
}

/// Batch extract function calls in parallel across multiple functions in a file.
#[pyfunction]
#[pyo3(signature = (content, lines))]
fn batch_calls(
    py: Python<'_>,
    content: &str,
    lines: Vec<usize>,
) -> Vec<(usize, Vec<String>)> {
    py.allow_threads(|| calls::batch_calls(content, &lines))
}

/// Compute betweenness centrality for a directed graph using Brandes' algorithm.
#[pyfunction]
#[pyo3(signature = (nodes, edges, k = None, normalized = true))]
fn betweenness_centrality(
    py: Python<'_>,
    nodes: Vec<String>,
    edges: Vec<(String, String)>,
    k: Option<usize>,
    normalized: bool,
) -> HashMap<String, f64> {
    py.allow_threads(|| centrality::betweenness_centrality(&nodes, &edges, k, normalized))
}

/// Detect circular dependencies using Tarjan's SCC algorithm in linear time O(V + E).
#[pyfunction]
#[pyo3(signature = (nodes, edges))]
fn detect_circular_dependencies(
    py: Python<'_>,
    nodes: Vec<String>,
    edges: Vec<(String, String)>,
) -> Vec<Vec<String>> {
    py.allow_threads(|| cycles::detect_circular_dependencies(&nodes, &edges))
}

/// Compute graph reachability from entry points in O(V + E) memory time.
#[pyfunction]
#[pyo3(signature = (nodes, edges, entry_points))]
fn compute_reachability(
    py: Python<'_>,
    nodes: Vec<String>,
    edges: Vec<(String, String)>,
    entry_points: Vec<String>,
) -> HashMap<String, String> {
    py.allow_threads(|| reachability::compute_reachability(&nodes, &edges, &entry_points))
}

/// Compute module interactions and coupling metrics across modules.
#[pyfunction]
#[pyo3(signature = (func_modules, calls))]
fn compute_module_coupling(
    py: Python<'_>,
    func_modules: Vec<(String, String)>,
    calls: Vec<(String, String)>,
) -> (HashMap<String, Vec<String>>, HashMap<String, (usize, usize, f64)>) {
    py.allow_threads(|| coupling::compute_module_coupling(&func_modules, &calls))
}

/// Detect god functions natively.
#[pyfunction]
#[pyo3(signature = (functions))]
fn detect_god_functions(
    py: Python<'_>,
    functions: Vec<(String, String, usize, usize, usize, u32)>,
) -> Vec<(String, String, usize, usize, usize, u32, f64)> {
    py.allow_threads(|| {
        let candidates = smells::detect_god_functions_native(&functions);
        candidates
            .into_iter()
            .map(|c| {
                (
                    c.name,
                    c.file,
                    c.line,
                    c.fan_out,
                    c.mutations,
                    c.complexity,
                    c.severity,
                )
            })
            .collect()
    })
}

/// Detect data clumps across functions.
#[pyfunction]
#[pyo3(signature = (func_params, min_size = 3, min_occurrences = 2))]
fn detect_data_clumps(
    py: Python<'_>,
    func_params: Vec<(String, Vec<String>)>,
    min_size: usize,
    min_occurrences: usize,
) -> Vec<(Vec<String>, Vec<String>)> {
    py.allow_threads(|| smells::detect_data_clumps_native(&func_params, min_size, min_occurrences))
}

/// Walk project files with gitignore and fast directory pruning.
#[pyfunction]
#[pyo3(signature = (root, extensions, filenames, filename_prefixes, skip_dirs, respect_gitignore = true))]
fn walk_project_files(
    py: Python<'_>,
    root: &str,
    extensions: Vec<String>,
    filenames: Vec<String>,
    filename_prefixes: Vec<String>,
    skip_dirs: Vec<String>,
    respect_gitignore: bool,
) -> Vec<(String, String)> {
    py.allow_threads(|| {
        discovery::walk_project_files(
            root,
            &extensions,
            &filenames,
            &filename_prefixes,
            &skip_dirs,
            respect_gitignore,
        )
    })
}

/// Compute content hash for a file.
#[pyfunction]
#[pyo3(signature = (filepath, analyzer_version))]
fn compute_file_content_hash(filepath: &str, analyzer_version: &str) -> Option<String> {
    cache::compute_file_content_hash(filepath, analyzer_version)
}

/// Check changed files in parallel using Rayon.
#[pyfunction]
#[pyo3(signature = (project_dir, filepaths, manifest_entries, analyzer_version))]
fn check_changed_files(
    py: Python<'_>,
    project_dir: &str,
    filepaths: Vec<String>,
    manifest_entries: HashMap<String, (String, f64, u64)>,
    analyzer_version: &str,
) -> (Vec<String>, Vec<String>, Vec<(String, f64)>) {
    py.allow_threads(|| {
        let res = cache::check_changed_files_parallel(
            project_dir,
            &filepaths,
            &manifest_entries,
            analyzer_version,
        );
        (res.changed, res.cached, res.refreshed)
    })
}

/// Format header lines for analysis.toon.yaml.
#[pyfunction]
#[pyo3(signature = (nfiles, total_lines, lang_label, timestamp, avg_cc, critical_cc, total_funcs, dups, cycles))]
fn format_toon_header(
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
    toon::format_toon_header(
        nfiles,
        total_lines,
        lang_label,
        timestamp,
        avg_cc,
        critical_cc,
        total_funcs,
        dups,
        cycles,
    )
}

/// Module initialization
#[pymodule]
fn code2llm_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(extract_function_body, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_complexity, m)?)?;
    m.add_function(wrap_pyfunction!(batch_complexity, m)?)?;
    m.add_function(wrap_pyfunction!(extract_calls, m)?)?;
    m.add_function(wrap_pyfunction!(batch_calls, m)?)?;
    m.add_function(wrap_pyfunction!(betweenness_centrality, m)?)?;
    m.add_function(wrap_pyfunction!(detect_circular_dependencies, m)?)?;
    m.add_function(wrap_pyfunction!(compute_reachability, m)?)?;
    m.add_function(wrap_pyfunction!(compute_module_coupling, m)?)?;
    m.add_function(wrap_pyfunction!(detect_god_functions, m)?)?;
    m.add_function(wrap_pyfunction!(detect_data_clumps, m)?)?;
    m.add_function(wrap_pyfunction!(walk_project_files, m)?)?;
    m.add_function(wrap_pyfunction!(compute_file_content_hash, m)?)?;
    m.add_function(wrap_pyfunction!(check_changed_files, m)?)?;
    m.add_function(wrap_pyfunction!(format_toon_header, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}


