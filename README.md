# code2llm-rust


## AI Cost Tracking

![PyPI](https://img.shields.io/badge/pypi-costs-blue) ![Version](https://img.shields.io/badge/version-0.1.0-blue) ![Python](https://img.shields.io/badge/python-3.9+-blue) ![License](https://img.shields.io/badge/license-Apache--2.0-green)
![AI Cost](https://img.shields.io/badge/AI%20Cost-$0.07-orange) ![Human Time](https://img.shields.io/badge/Human%20Time-3.1h-blue) ![Model](https://img.shields.io/badge/Model-openrouter%2Fqwen%2Fqwen3--coder--next-lightgrey)

- 🤖 **LLM usage:** $0.0680 (4 commits)
- 👤 **Human dev:** ~$313 (3.1h @ $100/h, 30min dedup)

Generated on 2026-10-04 using [openrouter/qwen/qwen3-coder-next](https://openrouter.ai/qwen/qwen3-coder-next)

---



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

Licensed under Apache-2.0.
