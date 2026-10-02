# code2llm-rust

High-performance native Rust acceleration extension for `code2llm`.

## Overview

`code2llm-rust` is a Python C-extension built with PyO3 and Rayon, providing ultra-fast native implementations of core computational bottlenecks in `code2llm`:

- **Gitignore-Aware File Discovery**: Parallel repository traversal with `.gitignore` compliance and early subtree pruning (`node_modules`, `.git`, `target`, `dist`, `.venv`, `.cache`).
- **Cyclomatic Complexity**: Parallel McCabe cyclomatic complexity and zero-copy brace depth extraction (200x+ faster than pure Python regex).
- **Call Extraction**: Native token-level call scanner for C-family, JavaScript, TypeScript, Rust, Go languages.
- **Graph Centrality**: Native Brandes' betweenness centrality algorithm executing in parallel across CPU cores (42x – 100x+ faster than NetworkX).
- **Tarjan SCC Cycle Detection**: Linear-time $O(V + E)$ cycle detection replacing slow exponential Johnson's algorithm with no node limits.
- **In-Memory Reachability**: Graph reachability BFS/DFS directly on the in-memory call graph without re-reading files from disk.
- **Module Coupling Metrics**: Vectorized calculation of Afferent Coupling ($Ca$), Efferent Coupling ($Ce$), and Instability ($I$).
- **Code Smell Engine**: Vectorized detection of God functions and Data clumps.

## Installation

```bash
pip install code2llm-rust
```

Or build and install from source:

```bash
maturin develop --release
```

## Python API

```python
import code2llm_rust

# 1. Parallel file discovery
files = code2llm_rust.walk_project_files(
    root=".",
    extensions=[".py", ".ts", ".rs"],
    filenames=["Dockerfile"],
    filename_prefixes=["Makefile."],
    skip_dirs=["node_modules", "target", ".venv"],
    respect_gitignore=True,
)

# 2. Extract function body
body = code2llm_rust.extract_function_body(source_code, start_line=10)

# 3. Calculate complexity
cc, rank = code2llm_rust.calculate_complexity(body, lang="c_family")

# 4. Batch compute across lines in parallel
results = code2llm_rust.batch_complexity(source_code, [10, 25, 40], lang="c_family")

# 5. Extract calls
calls = code2llm_rust.extract_calls(body)

# 6. Betweenness centrality (Brandes' algorithm)
centrality = code2llm_rust.betweenness_centrality(nodes, edges, k=100, normalized=True)

# 7. Circular dependencies (Tarjan SCC)
cycles = code2llm_rust.detect_circular_dependencies(nodes, edges)

# 8. Graph reachability
reach = code2llm_rust.compute_reachability(nodes, edges, entry_points=["main"])

# 9. Module coupling
interactions, metrics = code2llm_rust.compute_module_coupling(func_modules, calls)
```

## License

Apache-2.0
