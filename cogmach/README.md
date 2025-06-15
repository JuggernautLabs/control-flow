# Fundamental Primitives Cognition Machine

A Rust implementation of the fundamental observe/generate primitives for automated software development.

## Core Concept

All software development reduces to two operations:
- **Observe(Lens)**: Discover what IS
- **Generate(Specification)**: Create what SHOULD BE

## Quick Start

```bash
# Copy environment variables
cp .env.example .env
# Edit .env with your API keys

# Run the experiment demo
cargo run --bin cogmach-experiment

# Run interactive CLI
cargo run --bin cogmach-cli experiment "create a tictactoe api"

# Show system info
cargo run --bin cogmach-cli info

# Enable debug logging
cargo run --bin cogmach-cli --debug experiment "your goal"

# Enable trace logging
cargo run --bin cogmach-cli --trace experiment "your goal"
```

## Architecture

- **Mechanical Observation**: Direct system interrogation (file existence, code execution)
- **Semantic Observation**: AI-powered analysis (code structure, relationships)
- **Generation**: AI-powered content creation based on deltas
- **Execution**: Real code execution with pytest integration

## Tracing

The system uses structured tracing throughout:
- All major operations are instrumented
- Detailed logging of AI interactions
- Performance metrics and execution results
- Error tracking and debugging information

## Development

```bash
# Run tests
cargo test

# Check formatting
cargo fmt

# Run clippy
cargo clippy

# Build release
cargo build --release
```
