(ns record-keeping.llm-extractor
  (:require [cheshire.core :as json]
            [clj-http.client :as http]
            [clojure.string :as str]
            [clojure.tools.logging :as log]
            [record-keeping.extraction-prompt :as prompt]))

;; Claude API configuration
(def claude-api-config
  {:base-url "https://api.anthropic.com/v1/messages"
   :model "claude-3-5-sonnet-20241022"
   :max-tokens 4000
   :temperature 0.1})

;; File chunking configuration
(def chunking-config
  {:max-chunk-size 2000  ; characters
   :overlap-size 200     ; characters
   :min-chunk-size 100   ; characters
   :preserve-functions true
   :preserve-classes true})

(defn chunk-file-content
  "Split file content into overlapping chunks that preserve code structure"
  [file-content file-path language]
  (let [lines (str/split-lines file-content)
        total-lines (count lines)]
    (loop [start-line 0
           chunks []]
      (if (>= start-line total-lines)
        chunks
        (let [chunk-lines (take-while 
                           #(<= (count (str/join "\n" %)) 
                                (:max-chunk-size chunking-config))
                           (map #(take % lines)
                                (range (inc start-line) (inc total-lines))))
              chunk-end-line (+ start-line (count (last chunk-lines)))
              chunk-content (str/join "\n" (drop start-line (take chunk-end-line lines)))
              next-start (max (inc start-line) 
                             (- chunk-end-line 
                                (/ (:overlap-size chunking-config) 50)))] ; rough line estimate
          (recur (int next-start)
                 (conj chunks
                       {:file-path file-path
                        :language language
                        :start-line (inc start-line) ; 1-based line numbers
                        :end-line chunk-end-line
                        :code-content chunk-content
                        :chunk-size (count chunk-content)})))))))

(defn call-claude-api
  "Make API call to Claude for code analysis"
  [prompt api-key]
  (try
    (let [response (http/post (:base-url claude-api-config)
                              {:headers {"Content-Type" "application/json"
                                       "x-api-key" api-key
                                       "anthropic-version" "2023-06-01"}
                               :body (json/generate-string
                                      {:model (:model claude-api-config)
                                       :max_tokens (:max-tokens claude-api-config)
                                       :temperature (:temperature claude-api-config)
                                       :messages [{:role "user" :content prompt}]})
                               :throw-exceptions false})]
      (if (= 200 (:status response))
        (let [response-body (json/parse-string (:body response) true)
              content (get-in response-body [:content 0 :text])]
          {:success true :content content})
        (do
          (log/error "Claude API error:" (:status response) (:body response))
          {:success false :error (str "API error: " (:status response))})))
    (catch Exception e
      (log/error "Exception calling Claude API:" (.getMessage e))
      {:success false :error (.getMessage e)})))

(defn parse-extraction-result
  "Parse JSON response from Claude and validate structure"
  [response-content]
  (try
    (let [json-start (str/index-of response-content "{")
          json-end (str/last-index-of response-content "}")
          json-str (when (and json-start json-end)
                     (subs response-content json-start (inc json-end)))
          parsed (when json-str (json/parse-string json-str true))]
      (if (and parsed 
               (contains? parsed :entities)
               (contains? parsed :relationships))
        {:success true :data parsed}
        {:success false :error "Invalid JSON structure"}))
    (catch Exception e
      (log/error "Failed to parse extraction result:" (.getMessage e))
      {:success false :error (str "Parse error: " (.getMessage e))})))

(defn validate-extraction
  "Validate extracted data using a second Claude call"
  [extracted-data chunk-data api-key]
  (let [validation-prompt (prompt/build-validation-prompt 
                          {:extracted-data (json/generate-string extracted-data)
                           :code-content (:code-content chunk-data)
                           :language (:language chunk-data)})
        response (call-claude-api validation-prompt api-key)]
    
    (if (:success response)
      (let [validation-content (:content response)]
        ;; For now, just return original data with validation info
        ;; Could implement more sophisticated validation logic here
        {:success true 
         :data extracted-data
         :validation validation-content
         :chunk-info (select-keys chunk-data [:file-path :start-line :end-line])})
      {:success false :error "Validation failed"})))

(defn extract-from-chunk
  "Extract knowledge graph data from a single code chunk using Claude"
  [chunk-data api-key & {:keys [validate] :or {validate false}}]
  (let [extraction-prompt (prompt/build-extraction-prompt chunk-data)
        _ (log/info "Extracting from chunk:" (:file-path chunk-data) 
                   "lines" (:start-line chunk-data) "-" (:end-line chunk-data))
        response (call-claude-api extraction-prompt api-key)]
    
    (if (:success response)
      (let [parse-result (parse-extraction-result (:content response))]
        (if (:success parse-result)
          (let [extracted-data (:data parse-result)]
            (if validate
              ;; Optional validation step
              (validate-extraction extracted-data chunk-data api-key)
              {:success true 
               :data extracted-data
               :chunk-info (select-keys chunk-data [:file-path :start-line :end-line])}))
          parse-result))
      response)))

(defn infer-language
  "Infer programming language from file extension"
  [file-path]
  (let [ext (-> file-path
                (str/split #"\.")
                last
                str/lower-case)]
    (get {"py" "python"
          "js" "javascript"
          "ts" "typescript"
          "java" "java"
          "clj" "clojure"
          "cljs" "clojure"
          "rs" "rust"
          "go" "go"
          "cpp" "cpp"
          "c" "c"
          "rb" "ruby"
          "php" "php"} ext "unknown")))

(defn extract-from-file
  "Extract knowledge graph data from an entire file"
  [file-path api-key & {:keys [validate] :or {validate false}}]
  (try
    (let [file-content (slurp file-path)
          language (infer-language file-path)
          chunks (chunk-file-content file-content file-path language)
          _ (log/info "Processing file:" file-path "with" (count chunks) "chunks")]
      
      (loop [remaining-chunks chunks
             all-entities []
             all-relationships []
             errors []]
        
        (if (empty? remaining-chunks)
          {:success (empty? errors)
           :entities all-entities
           :relationships all-relationships
           :errors errors
           :file-path file-path
           :chunks-processed (count chunks)}
          
          (let [chunk (first remaining-chunks)
                result (extract-from-chunk chunk api-key :validate validate)]
            
            (if (:success result)
              (let [data (:data result)
                    entities (or (:entities data) [])
                    relationships (or (:relationships data) [])]
                (recur (rest remaining-chunks)
                       (concat all-entities entities)
                       (concat all-relationships relationships)
                       errors))
              (do
                (log/error "Failed to extract from chunk:" (:error result))
                (recur (rest remaining-chunks)
                       all-entities
                       all-relationships
                       (conj errors {:chunk chunk :error (:error result)}))))))))
    
    (catch Exception e
      (log/error "Failed to process file" file-path ":" (.getMessage e))
      {:success false :error (.getMessage e)})))

(defn merge-entity-results
  "Merge entities from multiple chunks, handling duplicates"
  [entity-lists]
  (let [all-entities (apply concat entity-lists)
        entity-map (group-by (juxt :name :type :source_location) all-entities)]
    (map (fn [[_ entities]]
           ;; Take the first entity but merge metadata if needed
           (first entities))
         entity-map)))

(defn merge-relationship-results
  "Merge relationships from multiple chunks, handling duplicates"
  [relationship-lists]
  (let [all-relationships (apply concat relationship-lists)
        unique-relationships (distinct all-relationships)]
    unique-relationships))

(defn discover-source-files
  "Discover source files in repository matching patterns"
  [repo-path file-patterns]
  (->> (file-seq (clojure.java.io/file repo-path))
       (filter #(.isFile %))
       (map #(.getAbsolutePath %))
       (filter (fn [path]
                 (some #(re-find % path) file-patterns)))
       (remove (fn [path]
                 (some #(str/includes? path %)
                       [".git" "node_modules" "__pycache__" "target" ".venv"])))))

(defn combine-extraction-results
  "Combine results from multiple file extractions"
  [results repo-path]
  (let [successful-results (filter :success results)
        failed-results (remove :success results)
        all-entities (merge-entity-results (map :entities successful-results))
        all-relationships (merge-relationship-results (map :relationships successful-results))
        all-errors (apply concat (map :errors successful-results))]
    
    {:success (empty? failed-results)
     :repository-path repo-path
     :entities all-entities
     :relationships all-relationships
     :files-processed (count successful-results)
     :total-files (count results)
     :errors (concat all-errors 
                    (map #(select-keys % [:file :error]) failed-results))}))

(defn extract-from-repository
  "Extract knowledge graph data from an entire repository"
  [repo-path api-key & {:keys [file-patterns validate parallel] 
                        :or {file-patterns [#"\.py$" #"\.js$" #"\.ts$" #"\.java$" #"\.clj$"]
                             validate false
                             parallel false}}]
  (let [files (discover-source-files repo-path file-patterns)]
    (log/info "Starting repository extraction from:" repo-path)
    (log/info "Found" (count files) "files to process")
    
    (if parallel
      ;; Parallel processing (would need pmap with rate limiting)
      (let [results (pmap #(extract-from-file % api-key :validate validate) files)]
        (combine-extraction-results results repo-path))
      ;; Sequential processing
      (loop [remaining-files files
             all-entities []
             all-relationships []
             errors []
             processed 0]
        
        (if (empty? remaining-files)
          {:success (empty? errors)
           :repository-path repo-path
           :entities all-entities
           :relationships all-relationships
           :files-processed processed
           :total-files (count files)
           :errors errors}
          
          (let [file (first remaining-files)
                result (extract-from-file file api-key :validate validate)]
            
            (if (:success result)
              (recur (rest remaining-files)
                     (concat all-entities (:entities result))
                     (concat all-relationships (:relationships result))
                     (concat errors (:errors result))
                     (inc processed))
              (do
                (log/error "Failed to process file:" file)
                (recur (rest remaining-files)
                       all-entities
                       all-relationships
                       (conj errors {:file file :error (:error result)})
                       (inc processed)))))))))) 