## Project Overview

Bosing is a high-performance waveform generator for superconducting quantum computing experiments. It's a mixed Rust/Python project that generates microwave pulse sequences with significant performance optimizations over naive implementations.

## Development Commands

This project uses `uv` for Python package management and Cargo for Rust:

```bash
# Setup development environment
uv sync

# Common development tasks
uv run task format      # Format both Rust and Python code
uv run task lint        # Lint both Rust and Python code
uv run task test        # Run both Rust and Python tests
uv run task makedocs    # Build documentation
uv run task stubtest    # Validate Python stubs

# Individual commands
cargo fmt --all         # Format Rust code
cargo clippy --workspace --all-targets  # Lint Rust code
cargo test --workspace  # Run Rust tests
ruff format             # Format Python code
ruff check              # Lint Python code
basedpyright            # Type check Python code
pytest                  # Run Python tests
```

## Architecture

### Core Structure

The project follows a hybrid architecture:

- **Rust Core** (`src/`): High-performance waveform generation logic
- **Python Bindings** (`bosing-py/`): PyO3 bindings exposing Rust functionality to Python
- **Python Package** (`python/src/bosing/`): Pure Python utilities and type stubs

### Key Components

1. **Domain Types** (`src/quant.rs`): Type-safe physical quantities

   - `Time`, `Frequency`, `Phase`, `Amplitude` - Physical quantities with NaN safety
   - `ChannelId`, `ShapeId`, `Label` - Identifier types
   - `AlignedIndex` - Sampling alignment with configurable precision levels

2. **Schedule Elements** (`src/schedule/`): Define timing and structure of pulse sequences

   - `Stack`: Sequential execution of elements
   - `Grid`: Grid-based timing with alignment
   - `Absolute`: Precise timing control
   - `Repeat`: Loop structures

3. **Waveform Generation** (`src/`): Core signal processing

   - `executor.rs`: Converts schedules to waveforms
   - `pulse.rs`: Pulse and oscillation handling
   - `shape.rs`: Envelope shapes (Hann, Interp, etc.)
   - `util.rs`: Utility functions and helpers

4. **Python Interface** (`bosing-py/src/`): PyO3 bindings
   - `elements.rs`: Python wrapper for schedule elements
   - `shapes.rs`: Python wrapper for shape functions
   - `wavegen.rs`: Main waveform generation entry point

### Build System

- **Workspace Structure**: Root Cargo.toml defines workspace with `bosing-py` member
- **Python Packaging**: Uses Maturin to build Rust extension and distribute as Python package
- **Dependency Management**: uv for Python, Cargo for Rust, with shared workspace dependencies

### Testing

- Rust tests in respective modules (`cargo test`)
- Python tests in `python/tests/` (`pytest`)
- Performance benchmarks in `python/examples/schedule_stress.py`

### Code Style

- **Rust**: Follows clippy pedantic and nursery rules, with custom lint configuration
- **Python**: Uses ruff for formatting and linting, basedpyright for type checking
- **Pre-commit**: Enforces formatting, linting, and dependency checks

## Important Notes

- Phase units are in cycles, not radians (0.5 = π radians)
- Performance critical code is implemented in Rust with parallelization using Rayon
- The project uses a hybrid ownership model where Python references are managed by PyO3
- All public Python APIs have corresponding type stubs in `python/src/bosing/_bosing.pyi`
- Domain types in `quant.rs` provide type safety for physical quantities and prevent NaN values
