(ns example-usage
  (:require [record-keeping.core :as core]
            [record-keeping.llm-extractor :as llm]
            [record-keeping.queries :as queries]
            [record-keeping.schema :as schema]
            [datomic.api :as d]))

;; Example: Extract knowledge graph from a single file
(defn extract-single-file-example
  "Example of extracting knowledge graph data from a single Python file"
  [file-path api-key]
  (println "Extracting from file:" file-path)
  
  (let [result (llm/extract-from-file file-path api-key)]
    (if (:success result)
      (do
        (println "✅ Extraction successful!")
        (println "Entities found:" (count (:entities result)))
        (println "Relationships found:" (count (:relationships result)))
        
        ;; Pretty print first few entities
        (println "\n📋 Sample entities:")
        (doseq [entity (take 3 (:entities result))]
          (println (format "  - %s (%s) at line %d"
                          (:name entity)
                          (:type entity)
                          (get-in entity [:source_location :start_line]))))
        
        result)
      (do
        (println "❌ Extraction failed:" (:error result))
        nil))))

;; Example: Extract from entire repository
(defn extract-repository-example
  "Example of extracting knowledge graph data from an entire repository"
  [repo-path api-key]
  (println "Extracting from repository:" repo-path)
  
  (let [result (llm/extract-from-repository repo-path api-key)]
    (if (:success result)
      (do
        (println "✅ Repository extraction successful!")
        (println "Files processed:" (:files-processed result) "/" (:total-files result))
        (println "Total entities:" (count (:entities result)))
        (println "Total relationships:" (count (:relationships result)))
        
        ;; Show breakdown by entity type
        (let [entity-counts (->> (:entities result)
                                (group-by :type)
                                (map (fn [[type entities]] [type (count entities)]))
                                (into {}))]
          (println "\n📊 Entity breakdown:")
          (doseq [[type count] entity-counts]
            (println (format "  - %s: %d" (name type) count))))
        
        result)
      (do
        (println "❌ Repository extraction failed:" (:error result))
        nil))))

;; Example: Store in Datomic and query
(defn datomic-storage-example
  "Example of storing extracted data in Datomic and running queries"
  [extraction-result]
  (when extraction-result
    (let [uri "datomic:mem://example-kg"
          _ (d/create-database uri)
          conn (d/connect uri)
          
          ;; Install schema
          _ (schema/install-schema! conn)
          
          ;; Convert extracted data to Datomic format and store
          ;; (This would need implementation in core.clj)
          
          db (d/db conn)]
      
      (println "\n🗄️ Datomic storage example:")
      (println "Database created and schema installed")
      
      ;; Example queries (would work once data is stored)
      (comment
        ;; Find all functions
        (let [functions (queries/find-all-functions db)]
          (println "Functions in database:" (count functions)))
        
        ;; Find function calls
        (let [calls (queries/find-function-calls db some-function-id)]
          (println "Functions called:" (count calls)))
        
        ;; Repository statistics
        (let [stats (queries/repository-statistics db nil)]
          (println "Repository stats:" stats))))))

;; Sample test data for demonstration
(def sample-python-code
  "def calculate_average(numbers):
    \"\"\"Calculate the average of a list of numbers.\"\"\"
    if not numbers:
        return 0
    return sum(numbers) / len(numbers)

class Calculator:
    \"\"\"A simple calculator class.\"\"\"
    
    def __init__(self):
        self.history = []
    
    def add(self, a, b):
        \"\"\"Add two numbers.\"\"\"
        result = a + b
        self.history.append(f'{a} + {b} = {result}')
        return result
    
    def get_average(self, numbers):
        \"\"\"Get average using the calculate_average function.\"\"\"
        return calculate_average(numbers)")

;; Demo function that creates a temp file and processes it
(defn demo-with-sample-code
  "Demo the extraction using sample Python code"
  [api-key]
  (let [temp-file (java.io.File/createTempFile "sample" ".py")]
    (try
      ;; Write sample code to temp file
      (spit temp-file sample-python-code)
      
      (println "🔍 Demo: Analyzing sample Python code")
      (println "Code preview:")
      (println (subs sample-python-code 0 200) "...")
      
      ;; Extract and display results
      (extract-single-file-example (.getAbsolutePath temp-file) api-key)
      
      (finally
        ;; Clean up temp file
        (.delete temp-file)))))

;; Main demo function
(defn run-demo
  "Run a complete demonstration of the system"
  [api-key]
  (println "🚀 Knowledge Graph Extraction Demo")
  (println "=====================================")
  
  ;; Demo 1: Single file with sample code
  (demo-with-sample-code api-key)
  
  (println "\n" (apply str (repeat 50 "-")))
  
  ;; Demo 2: Repository extraction (if current directory has source files)
  (let [current-dir (System/getProperty "user.dir")]
    (println "\n🗂️ Demo: Repository analysis of current directory")
    (extract-repository-example current-dir api-key))
  
  (println "\n✨ Demo complete!"))

;; Usage examples:
(comment
  ;; To run the demo:
  (run-demo "your-claude-api-key")
  
  ;; To extract from a specific file:
  (extract-single-file-example "/path/to/your/file.py" "your-api-key")
  
  ;; To extract from a repository:
  (extract-repository-example "/path/to/your/repo" "your-api-key")
  
  ;; To run via command line:
  ;; lein run /path/to/repo your-api-key
  
  ;; Or with environment variable:
  ;; export CLAUDE_API_KEY=your-key
  ;; lein run /path/to/repo
  ) 