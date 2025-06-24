(ns example-usage
  (:require [record-keeping.core :as core]
            [record-keeping.llm-extractor :as llm]
            [record-keeping.schema :as schema]
            [record-keeping.datomic-adapter :as db-adapter]
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
  [extraction-result repository-path]
  (when extraction-result
    (println "\n🗄️ Datomic storage and query example:")
    
    ;; Store in Datomic using the core system
    (let [storage-result (core/store-in-datomic extraction-result repository-path {})
          conn (core/get-connection {})
          db (d/db conn)]
      
      (if (:success storage-result)
        (do
          (println "✅ Successfully stored in Datomic!")
          (println "Repository ID:" (:repository-id storage-result))
          (println "Entities stored:" (:entities-stored storage-result))
          (println "Relationships stored:" (:relationships-stored storage-result))
          
          ;; Run example queries
          (println "\n📊 Running example queries:")
          
          ;; Find all functions
          (let [functions (d/q '[:find [(pull ?e [:code-entity/name :code-entity/file-path]) ...]
                                 :where
                                 [?e :code-entity/type :function]]
                               db)]
            (println "• Total functions in database:" (count functions))
            (when (seq functions)
              (println "  Sample functions:")
              (doseq [func (take 3 functions)]
                (println "    -" (:code-entity/name func) 
                        "in" (:code-entity/file-path func)))))
          
          ;; Find all classes
          (let [classes (d/q '[:find [(pull ?e [:code-entity/name :code-entity/file-path]) ...]
                               :where
                               [?e :code-entity/type :class]]
                             db)]
            (println "• Total classes in database:" (count classes))
            (when (seq classes)
              (println "  Sample classes:")
              (doseq [cls (take 3 classes)]
                (println "    -" (:code-entity/name cls)))))
          
          ;; Repository statistics
          (let [stats (d/q '[:find (count ?e) .
                             :where [?e :code-entity/type]]
                           db)]
            (println "• Total entities:" stats))
          
          ;; Find relationships
          (let [relationships (d/q '[:find (count ?r) .
                                     :where [?r :relationship/type]]
                                   db)]
            (println "• Total relationships:" relationships))
          
          storage-result)
        
        (do
          (println "❌ Failed to store in Datomic:" (:error storage-result))
          nil)))))

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

;; Complete pipeline demo
(defn demo-complete-pipeline
  "Demonstrate the complete pipeline: extraction + Datomic storage + queries"
  [api-key]
  (println "\n🎯 Demo: Complete Knowledge Graph Pipeline")
  (println "=" (apply str (repeat 60 "=")))
  
  (let [temp-file (java.io.File/createTempFile "pipeline-demo" ".py")]
    (try
      ;; Write sample code to temp file
      (spit temp-file sample-python-code)
      (let [file-path (.getAbsolutePath temp-file)]
        
        (println "Step 1: Extracting entities and relationships...")
        (let [extraction-result (extract-single-file-example file-path api-key)]
          
          (if extraction-result
            (do
              (println "\nStep 2: Storing in Datomic...")
              (let [storage-result (datomic-storage-example extraction-result file-path)]
                
                (if storage-result
                  (do
                    (println "\n✅ Complete pipeline successful!")
                    (println "Knowledge graph created with:")
                    (println "  • Entities extracted and stored")
                    (println "  • Relationships mapped and stored") 
                    (println "  • Database queries working")
                    storage-result)
                  (do
                    (println "\n❌ Pipeline failed at storage step")
                    nil))))
            
            (do
              (println "\n❌ Pipeline failed at extraction step")
              nil))))
      
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
  
  ;; Demo 2: Complete pipeline with Datomic
  (demo-complete-pipeline api-key)
  
  (println "\n" (apply str (repeat 50 "-")))
  
  ;; Demo 3: Repository extraction (if current directory has source files)
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