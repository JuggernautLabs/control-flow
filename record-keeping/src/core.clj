(ns record-keeping.core
  (:require [clojure.java.io :as io]
            [clojure.string :as str]
            [datomic.api :as d]
            [cheshire.core :as json]
            [record-keeping.llm-extractor :as llm]
            [record-keeping.schema :as schema]
            [record-keeping.datomic-adapter :as db-adapter]
            [aero.core :as aero]
            [clojure.tools.logging :as log]))

;; Core data structures
(defrecord CodeEntity [id type name source-location metadata embedding])
(defrecord Relationship [type source target metadata])
(defrecord Repository [path language-files entities relationships])

;; Load configuration
(defn load-config
  "Load configuration from resources/config.edn"
  []
  (aero/read-config (io/resource "config.edn")))

;; Datomic storage functions
(defn store-in-datomic
  "Store extraction result in Datomic"
  [extraction-result repo-path opts]
  (try
    (let [config (load-config)
          conn (db-adapter/get-connection config)]
      
      ;; Install schema if needed
      (db-adapter/install-schema! conn)
      
      ;; Store the extraction result
      (let [result (db-adapter/store-extraction-result! conn extraction-result repo-path)]
        (log/info "Successfully stored in Datomic:" result)
        result))
    
    (catch Exception e
      (log/error "Failed to store in Datomic:" (.getMessage e))
      {:success false :error (.getMessage e)})))

(defn get-connection
  "Get Datomic connection from options"
  [opts]
  (let [config (load-config)]
    (db-adapter/get-connection config)))

;; Main ingestion pipeline
(defn ingest-repository
  "Main entry point for ingesting a repository into the knowledge graph using LLM extraction"
  [repo-path opts]
  (let [api-key (:claude-api-key opts)
        _ (log/info "Starting LLM-based knowledge graph extraction")
        extraction-result (llm/extract-from-repository 
                          repo-path 
                          api-key 
                          :validate (:validate-extraction opts false)
                          :parallel (:parallel-processing opts false))
        
        _ (when-not (:success extraction-result)
            (log/error "Extraction failed with errors:" (:errors extraction-result)))
        
        ;; Store in Datomic
        storage-result (when (:success extraction-result)
                        (store-in-datomic extraction-result repo-path opts))]
    
    (if storage-result
      (merge extraction-result storage-result)
      {:success false :error "Failed to extract or store repository"})))

;; File discovery
(defn supported-file?
  "Check if file extension is supported"
  [file]
  (let [ext (-> file .getName (str/split #"\.") last str/lower-case)]
    (contains? #{"py" "js" "ts" "java" "clj" "rs" "go"} ext)))

(defn discover-source-files
  "Discover all source files in the repository"
  [repo-path]
  (->> (file-seq (io/file repo-path))
       (filter #(.isFile %))
       (filter supported-file?)
       (map #(.getAbsolutePath %))))


;; Utility functions
(defn infer-language
  "Infer programming language from file extension"
  [file-path]
  (let [ext (-> file-path (str/split #"\.") last str/lower-case)]
    (get {"py" :python
          "js" :javascript  
          "ts" :typescript
          "java" :java
          "clj" :clojure
          "rs" :rust
          "go" :go} ext :unknown)))

(defn call-tree-sitter
  "Call tree-sitter parser (placeholder - would use JNI or external process)"
  [file-path]
  ;; This would call the actual tree-sitter implementation
  ;; For now, return a placeholder structure
  {:type "program"
   :children []})

;; AST parsing using tree-sitter
(defn parse-file
  "Parse a single file using tree-sitter"
  [file-path opts]
  {:file-path file-path
   :language (infer-language file-path)
   :ast (call-tree-sitter file-path)
   :content (slurp file-path)})

(defn parse-files
  "Parse source files into ASTs"
  [files opts]
  (map #(parse-file % opts) files))

;; Entity extraction
(defn extract-functions
  "Extract function entities from AST"
  [ast file-path]
  ;; Placeholder implementation
  ;; Would traverse AST looking for function definitions
  [])

(defn extract-classes
  "Extract class entities from AST"
  [ast file-path]
  ;; Placeholder implementation
  [])

(defn extract-modules
  "Extract module entities from AST"
  [ast file-path]
  ;; Placeholder implementation
  [])

(defn extract-file-entities
  "Extract entities from a single parsed file"
  [parsed-file opts]
  (let [ast (:ast parsed-file)
        file-path (:file-path parsed-file)]
    (concat
      (extract-functions ast file-path)
      (extract-classes ast file-path)
      (extract-modules ast file-path))))

(defn extract-entities
  "Extract code entities from parsed files"
  [parsed-files opts]
  (mapcat #(extract-file-entities % opts) parsed-files))

;; Relationship extraction
(defn extract-call-relationships
  "Extract function call relationships"
  [entities]
  ;; Analyze function calls within code
  [])

(defn extract-import-relationships
  "Extract import/dependency relationships"
  [entities]
  ;; Analyze import statements and dependencies
  [])

(defn extract-inheritance-relationships
  "Extract class inheritance relationships"
  [entities]
  ;; Analyze class hierarchies
  [])

(defn extract-relationships
  "Extract relationships between entities"
  [entities opts]
  (concat
    (extract-call-relationships entities)
    (extract-import-relationships entities)
    (extract-inheritance-relationships entities)))

;; Embedding generation
(defn compute-embedding
  "Compute embedding for a single entity"
  [entity opts]
  ;; This would call an embedding model like CodeBERT
  ;; Placeholder: return random vector
  (vec (repeatedly 768 #(rand))))

(defn generate-embeddings
  "Generate embeddings for entities"
  [entities opts]
  (map #(assoc % :embedding (compute-embedding % opts)) entities))

;; Graph construction
(defn construct-graph
  "Construct knowledge graph from entities and relationships"
  [entities relationships embeddings]
  {:entities entities
   :relationships relationships
   :metadata {:created-at (java.util.Date.)
              :entity-count (count entities)
              :relationship-count (count relationships)}})

;; Legacy compatibility functions

(defn store-entities
  "Store entities in Datomic (legacy compatibility)"
  [conn entities]
  (db-adapter/store-entities! conn entities))

(defn store-relationships  
  "Store relationships in Datomic (legacy compatibility)"
  [conn relationships entity-id-map]
  (db-adapter/store-relationships! conn relationships entity-id-map))

;; Load configuration
(defn load-config
  "Load configuration from resources/config.edn"
  []
  (aero/read-config (io/resource "config.edn")))

;; Main CLI entry point
(defn -main
  "Main entry point for CLI"
  [& args]
  (let [repo-path (first args)
        api-key (or (System/getenv "CLAUDE_API_KEY")
                   (second args))
        config (load-config)
        opts (merge config
                   {:claude-api-key api-key
                    :validate-extraction false
                    :parallel-processing false})]
    
    (when (empty? api-key)
      (println "Error: Claude API key required. Set CLAUDE_API_KEY env var or pass as second argument.")
      (System/exit 1))
    
    (when (empty? repo-path)
      (println "Usage: lein run <repo-path> [claude-api-key]")
      (System/exit 1))
    
    (println "Starting LLM-based knowledge graph ingestion for:" repo-path)
    (println "Using configuration:" (dissoc opts :claude-api-key))
    
    (let [result (ingest-repository repo-path opts)]
      (if (:success result)
        (do
          (println "✅ Ingestion complete!")
          (println "📊 Statistics:")
          (println "  - Entities extracted:" (count (:entities result)))
          (println "  - Relationships found:" (count (:relationships result)))
          (println "  - Files processed:" (:files-processed result))
          (when (seq (:errors result))
            (println "⚠️  Errors encountered:" (count (:errors result)))))
        (do
          (println "❌ Ingestion failed:" (:error result))
          (System/exit 1)))))) 