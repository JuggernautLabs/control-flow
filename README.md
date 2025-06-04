# Story Generation Engine

A Rust-based story generation engine with TypeScript bindings for building interactive planning narratives.

## Features

- **Comprehensive Type System**: Full type definitions for story graphs, nodes, choices, and metadata
- **TypeScript Integration**: Automatic TypeScript type generation using `ts-rs`
- **Serialization Support**: JSON serialization/deserialization with `serde`
- **Async Traits**: Clean async interfaces for story generation engines
- **Validation Framework**: Built-in coherence checking and validation systems

## Core Types

### Story Graph Structure
- `StoryGraph`: Complete story with nodes, edges, and metadata
- `StoryNode`: Individual story points with choices and state
- `Choice`: Decision options with consequences and metadata
- `StoryEdge`: Connections between nodes with traversal tracking

### Engine Interfaces
- `StoryGenerationEngine`: AI-driven story creation and expansion
- `StoryGraphManager`: Graph manipulation and analytics
- `QuestionEngine`: Context gathering and uncertainty resolution

## Building

```bash
# Build the library
cargo build

# Generate TypeScript types
cargo run --bin generate-types

# Run tests
cargo test
```

## TypeScript Integration

Generated TypeScript types are available in the `bindings/` directory:

```typescript
import type { StoryGraph, StoryNode, Choice } from './bindings';

// Use the types in your frontend application
const story: StoryGraph = {
  id: "...",
  title: "My Story",
  // ... other fields
};
```

## Project Structure

```
src/
├── lib.rs           # Library entry point
├── types.rs         # Core type definitions
├── engine.rs        # Engine traits and utilities
├── errors.rs        # Error types
└── bin/
    └── generate_types.rs  # TypeScript generation script

bindings/            # Generated TypeScript types
├── index.ts         # Type exports
├── StoryGraph.ts    # Story graph interface
├── StoryNode.ts     # Story node interface
└── ...              # Other generated types
```

## Usage Example

```rust
use story_generation_engine::{StoryGraph, StoryNode, NodeType, NodeState};

// Create a new story
let mut story = StoryGraph::default();
story.title = "My Adventure".to_string();

// Add a starting node
let start_node = StoryNode {
    id: uuid::Uuid::new_v4(),
    node_type: NodeType::Start,
    situation: "You stand at the entrance of a mysterious cave...".to_string(),
    state: NodeState::Current,
    // ... other fields
};
```