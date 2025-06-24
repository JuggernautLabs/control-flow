# Code Knowledge Graph Ingestion System Specification

## Overview

An ingestion system for constructing semantic knowledge graphs from software repositories, storing structured code understanding in Datomic with embedded representations for each node.

## System Architecture

### Core Components

1. **Parser & AST Generator**
   - Uses Tree-sitter for multiple language support
   - Generates concrete syntax trees (CST) and abstract syntax trees (AST)
   - Extracts structured information: functions, classes, modules, variables, imports

2. **Chunking Engine**
   - Semantic chunking based on syntactic boundaries (functions, classes, modules)
   - Context-aware chunking that preserves relationships
   - Configurable chunk size with overlap for context preservation

3. **Information Extractor**
   - Identifies key code elements:
     - Functions (signatures, parameters, return types, docstrings)
     - Classes/Traits (inheritance, methods, properties)
     - Modules (imports, exports, dependencies)
     - Variables (types, scopes, usage patterns)
     - Comments and documentation

4. **Semantic Analyzer**
   - Type inference and signature analysis
   - Dependency relationship extraction (calls, imports, inheritance)
   - Control flow and data flow analysis
   - Cross-reference resolution

5. **Embedding Generator**
   - Code embeddings using models like CodeBERT, GraphCodeBERT, or similar
   - Documentation embeddings for natural language descriptions
   - Contextual embeddings that consider surrounding code

6. **Graph Constructor**
   - Builds knowledge graph with nodes and relationships
   - Integrates static analysis results with semantic information
   - Creates multi-layered graph structure

7. **Datomic Adapter**
   - Translates graph structure to Datomic schema
   - Handles incremental updates and versioning
   - Optimizes for query performance

## Data Model

### Entity Types (Datomic Entities)

#### Core Code Entities
```clojure
;; Module/File
{:code-entity/type :module
 :code-entity/id "module-uuid"
 :module/path "src/main/example.py"
 :module/name "example"
 :module/language :python
 :module/embedding [vector-of-floats]
 :module/hash "sha256-hash"
 :module/size-loc 150}

;; Function
{:code-entity/type :function
 :code-entity/id "function-uuid"
 :function/name "calculate_metrics"
 :function/signature "(data: DataFrame, config: dict) -> dict"
 :function/docstring "Calculates various metrics from input data"
 :function/parameters [{:param/name "data" :param/type "DataFrame"}
                       {:param/name "config" :param/type "dict"}]
 :function/return-type "dict"
 :function/complexity 15
 :function/embedding [vector-of-floats]
 :function/source-lines [45 78]
 :function/parent-module "module-uuid"}

;; Class/Trait
{:code-entity/type :class
 :code-entity/id "class-uuid"
 :class/name "DataProcessor"
 :class/docstring "Main class for processing data pipelines"
 :class/methods [list-of-method-refs]
 :class/properties [list-of-property-refs]
 :class/inheritance [list-of-parent-classes]
 :class/embedding [vector-of-floats]
 :class/parent-module "module-uuid"}
```

#### Relationship Types
```clojure
;; Function calls
{:relationship/type :calls
 :relationship/source "function-uuid-1"
 :relationship/target "function-uuid-2"
 :relationship/context "line 23 in main function"
 :relationship/frequency 3}

;; Import relationships
{:relationship/type :imports
 :relationship/source "module-uuid-1"
 :relationship/target "module-uuid-2"
 :relationship/import-type :from-import
 :relationship/imported-names ["function1" "Class1"]}

;; Inheritance
{:relationship/type :inherits
 :relationship/source "class-uuid-1"
 :relationship/target "class-uuid-2"}

;; Type usage
{:relationship/type :uses-type
 :relationship/source "function-uuid"
 :relationship/target "type-uuid"
 :relationship/usage-context :parameter}
```

### Semantic Metadata
```clojure
;; Code embeddings
{:embedding/entity "entity-uuid"
 :embedding/type :code
 :embedding/model "graphcodebert-base"
 :embedding/vector [768-dimensional-vector]
 :embedding/timestamp #inst "2024-01-15"}

;; Documentation embeddings  
{:embedding/entity "entity-uuid"
 :embedding/type :documentation
 :embedding/model "sentence-transformers"
 :embedding/vector [384-dimensional-vector]
 :embedding/text "original documentation text"}
```

## Processing Pipeline

### Phase 1: Repository Analysis
1. **Repository Scanning**
   - Discover all source files matching supported languages
   - Extract repository metadata (git info, structure, dependencies)
   - Prioritize files based on importance/centrality

2. **File Processing**
   - Parse each file using tree-sitter
   - Extract AST and perform static analysis
   - Identify all code entities and their boundaries

### Phase 2: Semantic Extraction
1. **Entity Extraction**
   - Extract functions, classes, modules with full metadata
   - Resolve type information where possible
   - Extract documentation and comments

2. **Relationship Discovery**
   - Analyze function calls and method invocations
   - Track import/export relationships
   - Identify inheritance and composition patterns
   - Detect data flow relationships

### Phase 3: Embedding Generation
1. **Code Embedding**
   - Generate embeddings for each code entity
   - Use contextual information (surrounding code, imports)
   - Create embeddings at multiple granularities

2. **Documentation Embedding**
   - Process docstrings, comments, and README files
   - Link documentation to corresponding code entities
   - Generate semantic embeddings for natural language content

### Phase 4: Graph Construction
1. **Node Creation**
   - Create Datomic entities for all discovered code elements
   - Attach metadata, embeddings, and descriptors
   - Establish entity relationships

2. **Relationship Mapping**
   - Create explicit relationship entities in Datomic
   - Weight relationships by importance/frequency
   - Establish bidirectional references where appropriate

### Phase 5: Graph Enrichment
1. **Similarity Computation**
   - Compute semantic similarity between entities using embeddings
   - Create similarity relationships above threshold
   - Identify potential code clones and similar patterns

2. **Community Detection**
   - Identify modules and subsystems
   - Detect architectural patterns
   - Group related functionality

## Datomic Schema Design

### Core Attributes
```clojure
;; Entity identification
{:db/ident :code-entity/id
 :db/valueType :db.type/uuid
 :db/cardinality :db.cardinality/one
 :db/unique :db.unique/identity}

{:db/ident :code-entity/type
 :db/valueType :db.type/keyword
 :db/cardinality :db.cardinality/one}

;; Embeddings
{:db/ident :code-entity/embedding
 :db/valueType :db.type/tuple
 :db/cardinality :db.cardinality/many}

;; Relationships
{:db/ident :relationship/source
 :db/valueType :db.type/ref
 :db/cardinality :db.cardinality/one}

{:db/ident :relationship/target
 :db/valueType :db.type/ref
 :db/cardinality :db.cardinality/one}
```

## Query Interface

### Example Queries

1. **Find similar functions**
```clojure
;; Find functions similar to a given function based on embeddings
[:find ?similar-func ?similarity-score
 :in $ ?target-func
 :where
 [?target-func :function/embedding ?target-embedding]
 [?similar-func :function/embedding ?similar-embedding]
 [(similarity-fn ?target-embedding ?similar-embedding) ?similarity-score]
 [(> ?similarity-score 0.8)]]
```

2. **Trace function dependencies**
```clojure
;; Find all functions that a given function depends on (transitively)
[:find ?dependency
 :in $ ?root-func
 :where
 (call-chain ?root-func ?dependency)]
```

## Technical Considerations

### Scalability
- Incremental processing for large repositories
- Parallel processing of independent files
- Embedding caching and reuse
- Efficient similarity search using vector databases

### Language Support
- Primary: Python, JavaScript, TypeScript, Java, Rust
- Extensible architecture for adding new languages
- Language-specific analysis rules and patterns

### Integration Points
- Git integration for tracking changes
- CI/CD pipeline integration
- IDE plugin compatibility
- API for external tool integration

## Performance Requirements

- **Ingestion Rate**: Process 1000+ files per hour
- **Storage**: Efficient storage in Datomic with compression
- **Query Performance**: Sub-second response for common queries
- **Memory Usage**: Configurable memory limits for large repositories
- **Incremental Updates**: Process only changed files in subsequent runs

## Quality Assurance

### Accuracy Metrics
- AST parsing accuracy (>99%)
- Type inference precision
- Relationship extraction recall
- Embedding quality validation

### Testing Strategy
- Unit tests for each component
- Integration tests with real repositories
- Performance benchmarks
- Regression testing for updates

This specification provides a comprehensive framework that builds on existing tools like tree-sitter-graph and GraphGen4Code while incorporating your specific requirements for Datomic storage and semantic embeddings. 