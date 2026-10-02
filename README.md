# code2llm-rust

High-performance native Rust acceleration extension for `code2llm`.

## Overview

`code2llm-rust` is a Python C-extension built with PyO3 and Rayon, providing ultra-fast native implementations of core computational bottlenecks in `code2llm`:

- **Cyclomatic Complexity**: Parallel McCable complexity and brace depth extraction (200x+ faster than pure Python).
- **Call Extraction**: Native token-level call scanner for C-family, JavaScript, TypeScript, Rust, Go languages.
- **Graph Centrality**: Native Brandes' betweenness centrality algorithm executing in parallel across CPU cores (100x+ faster than NetworkX).
- **Code Smell Engine**: Vectorized detection of God functions and Data clumps.

## Installation

```bash
pip install code2llm-rust
```

Or build from source:

```bash
cd packages/code2llm-rust
maturin develop --release
```

## Python API

```python
import code2llm_rust

# Extract function body
body = code2llm_rust.extract_function_body(source_code, start_line=10)

# Calculate complexity
cc, rank = code2llm_rust.calculate_complexity(body, lang="c_family")

# Batch compute across lines in parallel
results = code2llm_rust.batch_complexity(source_code, [10, 25, 40], lang="c_family")

# Extract calls
calls = code2llm_rust.extract_calls(body)

# Betweenness centrality (Brandes' algorithm)
centrality = code2llm_rust.betweenness_centrality(nodes, edges, k=100, normalized=True)
```

## License

Apache-2.0
