# LLM-Powered Code Knowledge Graph

A sophisticated system that uses Large Language Models (Claude) to extract structured information from source code and build queryable knowledge graphs stored in Datomic.

## 🎯 Overview

This system analyzes source code repositories and extracts:
- **Code Entities**: Functions, classes, modules, variables, interfaces
- **Relationships**: Function calls, imports, inheritance, type usage
- **Metadata**: Types, signatures, documentation, complexity indicators

The extracted information is stored in a Datomic database as a knowledge graph that can be queried for code analysis, documentation generation, and software understanding.

## 🏗️ Architecture

```
Repository → Tree-sitter Parser → LLM Extraction → Entity Processing → Datomic Storage → Knowledge Graph
```

### Key Components

- **LLM Extractor**: Uses Claude API for semantic code analysis
- **Datomic Adapter**: Handles database operations and schema management
- **Core Pipeline**: Orchestrates the entire extraction and storage process
- **Query Interface**: Provides rich querying capabilities for the knowledge graph

## 🚀 Quick Start

### Prerequisites

- **Java 11+** (for Clojure and Datomic)
- **Clojure CLI tools** 
- **Anthropic API Key** (for Claude access)

### Installation

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd record-keeping
   ```

2. **Install Clojure CLI tools** (if not already installed):
   ```bash
   # On Linux/macOS
   curl -O https://download.clojure.org/install/linux-install-1.11.1.1413.sh
   chmod +x linux-install-1.11.1.1413.sh
   sudo ./linux-install-1.11.1.1413.sh
   ```

3. **Set up environment variables**:
   ```bash
   # Create .env file
   echo "ANTHROPIC_API_KEY=your-claude-api-key-here" > .env
   ```

4. **Install dependencies**:
   ```bash
   clojure -P  # Downloads all dependencies
   ```

## 🧪 Running Tests

### Basic System Test

Test the complete pipeline with a sample Python file:

```bash
# Load environment and run basic test
source .env && clojure -M:dev -e '
(load-file "src/extraction_prompt.clj")
(load-file "src/llm_extractor.clj")
(load-file "src/schema.clj")
(load-file "src/datomic_adapter.clj")
(load-file "src/core.clj")
(load-file "example_usage.clj")

(let [api-key (System/getenv "ANTHROPIC_API_KEY")]
  (if api-key
    (example-usage/demo-complete-pipeline api-key)
    (println "❌ ANTHROPIC_API_KEY not found")))
'
```

### Individual Component Tests

**Test LLM Extraction Only**:
```bash
source .env && clojure -M:dev -e '
(load-file "src/llm_extractor.clj")
(example-usage/demo-with-sample-code (System/getenv "ANTHROPIC_API_KEY"))
'
```

**Test Datomic Schema**:
```bash
clojure -M:dev -e '
(load-file "src/schema.clj")
(load-file "src/datomic_adapter.clj")
(println "✅ Schema loaded successfully")
'
```

### Expected Test Output

A successful test should show:
```
🎯 Demo: Complete Knowledge Graph Pipeline
= ============================================================
Step 1: Extracting entities and relationships...
✅ Extraction successful!
Entities found: 6
Relationships found: 5

Step 2: Storing in Datomic...
✅ Successfully stored in Datomic!
Repository ID: #uuid "..."
Entities stored: 6

📊 Running example queries:
• Total functions in database: 5
• Total classes in database: 1
• Total entities: 6

✅ Complete pipeline successful!
```

## 📁 Using on Local Directories

### Command Line Usage

**Analyze a single file**:
```bash
source .env && clojure -M:dev -e '
(load-file "src/core.clj")
(record-keeping.core/ingest-repository "/path/to/your/file.py" 
  {:claude-api-key (System/getenv "ANTHROPIC_API_KEY")})
'
```

**Analyze an entire repository**:
```bash
source .env && clojure -M:dev -e '
(load-file "src/core.clj")
(record-keeping.core/ingest-repository "/path/to/your/repository" 
  {:claude-api-key (System/getenv "ANTHROPIC_API_KEY")
   :parallel-processing true
   :validate-extraction false})
'
```

### Interactive Usage

Start a Clojure REPL for interactive analysis:

```bash
source .env && clojure -M:dev
```

Then in the REPL:

```clojure
;; Load the system
(load-file "src/core.clj")
(load-file "src/queries.clj")

;; Set up API key
(def api-key (System/getenv "ANTHROPIC_API_KEY"))

;; Analyze a repository
(def result (record-keeping.core/ingest-repository 
              "/path/to/your/code"
              {:claude-api-key api-key}))

;; Check results
(println "Entities found:" (count (:entities result)))
(println "Relationships found:" (count (:relationships result)))

;; Query the knowledge graph (if stored in Datomic)
(def conn (record-keeping.datomic-adapter/get-connection 
            (record-keeping.core/load-config)))
(def db (datomic.api/db conn))

;; Find all functions
(record-keeping.queries/find-all-functions db)

;; Find functions by name pattern
(record-keeping.queries/find-functions-by-name db #".*calculate.*")

;; Get repository statistics
(record-keeping.queries/repository-statistics db nil)
```

### Supported Languages

Currently supports:
- **Python** (.py)
- **JavaScript** (.js)
- **TypeScript** (.ts)
- **Java** (.java)
- **Clojure** (.clj)
- **Rust** (.rs)
- **Go** (.go)

## ⚙️ Configuration

Edit `resources/config.edn` to customize:

```clojure
{:embeddings {:provider :openai
              :model "text-embedding-ada-002"}
 
 :tree-sitter {:enabled false}  ; Currently using LLM extraction
 
 :processing {:chunk-size 2000
              :chunk-overlap 200
              :max-files 1000
              :parallel-workers 4}
              
 :datomic {:uri "datomic:mem://code-knowledge-graph"
           :create-if-not-exists true}}
```

## 📊 Querying the Knowledge Graph

### Basic Queries

```clojure
;; Load query functions
(load-file "src/queries.clj")

;; Get database connection
(def conn (record-keeping.datomic-adapter/get-connection config))
(def db (datomic.api/db conn))

;; Find all functions
(record-keeping.queries/find-all-functions db)

;; Find classes
(d/q '[:find [(pull ?e [:code-entity/name :code-entity/file-path]) ...]
       :where [?e :code-entity/type :class]] db)

;; Find function calls
(record-keeping.queries/find-function-calls db function-id)

;; Find complex functions (if complexity data available)
(record-keeping.queries/find-complex-functions db 10)
```

### Advanced Analysis

```clojure
;; Find most called functions
(record-keeping.queries/find-most-called-functions db 10)

;; Find potential code clones
(record-keeping.queries/find-potential-code-clones db 0.8)

;; Find unused functions (potential dead code)
(record-helping.queries/find-unused-functions db)

;; Repository statistics
(record-keeping.queries/repository-statistics db repo-id)
```

## 🔧 Troubleshooting

### Common Issues

**"ANTHROPIC_API_KEY not found"**:
- Ensure your `.env` file contains the API key
- Run `source .env` before executing commands

**"Unable to resolve symbol" errors**:
- Load files in the correct order (see test examples)
- Ensure all dependencies are installed with `clojure -P`

**Datomic connection errors**:
- Check that Java 11+ is installed
- Verify Datomic dependencies in `deps.edn`

**Large repository timeouts**:
- Enable parallel processing: `:parallel-processing true`
- Reduce chunk size in configuration
- Process repositories in smaller batches

### Performance Tips

- **For large repositories**: Enable parallel processing
- **For rate limiting**: Reduce chunk size or add delays
- **For memory issues**: Process files in smaller batches
- **For accuracy**: Enable validation (slower but more reliable)

## 📚 Project Structure

```
├── src/
│   ├── core.clj              # Main ingestion pipeline
│   ├── llm_extractor.clj     # Claude API integration
│   ├── datomic_adapter.clj   # Database operations
│   ├── schema.clj            # Datomic schema definition
│   ├── queries.clj           # Query interface
│   └── extraction_prompt.clj # LLM prompts
├── resources/
│   └── config.edn           # System configuration
├── deps.edn                 # Clojure dependencies
├── example_usage.clj        # Usage examples and tests
└── README.md               # This file
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## 📄 License

[Add your license information here]

## 🙋 Support

For questions or issues:
1. Check the troubleshooting section
2. Review the example usage
3. Open an issue with detailed error information

---

**Built with**: Clojure, Datomic, Claude API, and ❤️ 