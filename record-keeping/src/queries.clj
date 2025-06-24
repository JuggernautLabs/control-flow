(ns record-keeping.queries
  (:require [datomic.api :as d]))

;; Example queries for the code knowledge graph

;; Basic entity retrieval queries

(defn find-all-functions
  "Find all function entities in the database"
  [db]
  (d/q '[:find [(pull ?e [*]) ...]
         :where
         [?e :code-entity/type :function]]
       db))

(defn find-functions-by-name
  "Find functions by name pattern"
  [db name-pattern]
  (d/q '[:find [(pull ?e [*]) ...]
         :in $ ?pattern
         :where
         [?e :code-entity/type :function]
         [?e :code-entity/name ?name]
         [(re-find ?pattern ?name)]]
       db (re-pattern name-pattern)))

(defn find-entities-in-file
  "Find all entities in a specific file"
  [db file-path]
  (d/q '[:find [(pull ?e [*]) ...]
         :in $ ?file
         :where
         [?e :code-entity/file-path ?file]]
       db file-path))

;; Relationship queries

(defn find-function-calls
  "Find all functions called by a given function"
  [db function-id]
  (d/q '[:find [(pull ?target [:code-entity/name
                               :code-entity/file-path
                               :function/signature]) ...]
         :in $ ?fn-id
         :where
         [?source :code-entity/id ?fn-id]
         [?rel :relationship/source ?source]
         [?rel :relationship/type :calls]
         [?rel :relationship/target ?target]]
       db function-id))

(defn find-callers-of-function
  "Find all functions that call a given function"
  [db function-id]
  (d/q '[:find [(pull ?source [:code-entity/name
                               :code-entity/file-path
                               :function/signature]) ...]
         :in $ ?fn-id
         :where
         [?target :code-entity/id ?fn-id]
         [?rel :relationship/target ?target]
         [?rel :relationship/type :calls]
         [?rel :relationship/source ?source]]
       db function-id))

(defn find-class-hierarchy
  "Find inheritance hierarchy for a class"
  [db class-id]
  (d/q '[:find [(pull ?parent [:code-entity/name
                               :class/docstring]) ...]
         :in $ ?class-id
         :where
         [?child :code-entity/id ?class-id]
         [?rel :relationship/source ?child]
         [?rel :relationship/type :inherits]
         [?rel :relationship/target ?parent]]
       db class-id))

(defn find-module-dependencies
  "Find all modules imported by a given module"
  [db module-id]
  (d/q '[:find [(pull ?target [:code-entity/name
                               :module/path]) ...]
         :in $ ?mod-id
         :where
         [?source :code-entity/id ?mod-id]
         [?rel :relationship/source ?source]
         [?rel :relationship/type :imports]
         [?rel :relationship/target ?target]]
       db module-id))

;; Complex analytical queries

(defn find-most-called-functions
  "Find functions with the highest call frequency"
  [db limit]
  (d/q '[:find ?name ?file ?call-count
         :in $ ?limit
         :where
         [?fn :code-entity/type :function]
         [?fn :code-entity/name ?name]
         [?fn :code-entity/file-path ?file]
         [(count ?rel) ?call-count]
         [?rel :relationship/target ?fn]
         [?rel :relationship/type :calls]]
       db limit))

(defn find-complex-functions
  "Find functions above a complexity threshold"
  [db complexity-threshold]
  (d/q '[:find [(pull ?fn [:code-entity/name
                           :code-entity/file-path
                           :function/complexity
                           :function/signature]) ...]
         :in $ ?threshold
         :where
         [?fn :code-entity/type :function]
         [?fn :function/complexity ?complexity]
         [(>= ?complexity ?threshold)]]
       db complexity-threshold))

(defn find-undocumented-functions
  "Find functions without docstrings"
  [db]
  (d/q '[:find [(pull ?fn [:code-entity/name
                           :code-entity/file-path]) ...]
         :where
         [?fn :code-entity/type :function]
         (not [?fn :function/docstring])]
       db))

;; Similarity and embedding queries

(defn find-similar-functions
  "Find functions similar to a given function based on embeddings
   Note: This would require a custom similarity function"
  [db function-id similarity-threshold]
  (d/q '[:find [(pull ?similar [:code-entity/name
                                :code-entity/file-path
                                :function/signature]) ...]
         :in $ ?fn-id ?threshold
         :where
         [?source :code-entity/id ?fn-id]
         [?rel :relationship/source ?source]
         [?rel :relationship/type :similar]
         [?rel :relationship/weight ?weight]
         [(>= ?weight ?threshold)]
         [?rel :relationship/target ?similar]]
       db function-id similarity-threshold))

;; Code quality queries

(defn find-potential-code-clones
  "Find potential code clones based on high similarity"
  [db similarity-threshold]
  (d/q '[:find [(pull ?source [:code-entity/name :code-entity/file-path])
                (pull ?target [:code-entity/name :code-entity/file-path])
                ?weight ...]
         :in $ ?threshold
         :where
         [?rel :relationship/type :similar]
         [?rel :relationship/weight ?weight]
         [(>= ?weight ?threshold)]
         [?rel :relationship/source ?source]
         [?rel :relationship/target ?target]]
       db similarity-threshold))

(defn find-unused-functions
  "Find functions that are never called (potential dead code)"
  [db]
  (d/q '[:find [(pull ?fn [:code-entity/name
                           :code-entity/file-path]) ...]
         :where
         [?fn :code-entity/type :function]
         (not [?rel :relationship/target ?fn]
              [?rel :relationship/type :calls])]
       db))

;; Repository analysis queries

(defn repository-statistics
  "Get basic statistics about the repository"
  [db repo-id]
  (let [function-count (d/q '[:find (count ?fn) .
                              :where [?fn :code-entity/type :function]]
                            db)
        class-count (d/q '[:find (count ?cls) .
                           :where [?cls :code-entity/type :class]]
                         db)
        module-count (d/q '[:find (count ?mod) .
                            :where [?mod :code-entity/type :module]]
                          db)
        relationship-count (d/q '[:find (count ?rel) .
                                  :where [?rel :relationship/type]]
                                db)]
    {:functions function-count
     :classes class-count
     :modules module-count
     :relationships relationship-count}))

(defn find-central-functions
  "Find functions that are most connected (high in-degree + out-degree)"
  [db limit]
  (d/q '[:find ?name ?file ?total-connections
         :in $ ?limit
         :where
         [?fn :code-entity/type :function]
         [?fn :code-entity/name ?name]
         [?fn :code-entity/file-path ?file]
         [(+ ?in-degree ?out-degree) ?total-connections]
         [(count ?in-rel) ?in-degree]
         [(count ?out-rel) ?out-degree]
         [?in-rel :relationship/target ?fn]
         [?out-rel :relationship/source ?fn]]
       db limit))

;; Language-specific queries

(defn find-async-functions
  "Find asynchronous functions"
  [db]
  (d/q '[:find [(pull ?fn [:code-entity/name
                           :code-entity/file-path
                           :function/signature]) ...]
         :where
         [?fn :code-entity/type :function]
         [?fn :function/is-async true]]
       db))

(defn find-functions-by-language
  "Find functions in a specific programming language"
  [db language]
  (d/q '[:find [(pull ?fn [:code-entity/name
                           :code-entity/file-path]) ...]
         :in $ ?lang
         :where
         [?fn :code-entity/type :function]
         [?fn :code-entity/language ?lang]]
       db language))

;; Advanced graph traversal queries

(defn find-call-chain
  "Find call chain from source to target function (transitive)"
  [db source-id target-id]
  ;; This would require a recursive rule in Datomic
  ;; For now, simplified version finding direct path
  (d/q '[:find [(pull ?intermediate [:code-entity/name]) ...]
         :in $ ?src ?tgt
         :where
         [?source :code-entity/id ?src]
         [?target :code-entity/id ?tgt]
         [?rel1 :relationship/source ?source]
         [?rel1 :relationship/type :calls]
         [?rel1 :relationship/target ?intermediate]
         [?rel2 :relationship/source ?intermediate]
         [?rel2 :relationship/type :calls]
         [?rel2 :relationship/target ?target]]
       db source-id target-id))

;; Query helper functions

(defn entity-by-id
  "Get full entity by ID"
  [db entity-id]
  (d/pull db '[*] [:code-entity/id entity-id]))

(defn search-entities
  "Free-text search across entity names and docstrings"
  [db search-term]
  (d/q '[:find [(pull ?e [:code-entity/name
                          :code-entity/type
                          :code-entity/file-path]) ...]
         :in $ ?term
         :where
         (or [?e :code-entity/name ?name]
             [?e :function/docstring ?name]
             [?e :class/docstring ?name])
         [(re-find ?term ?name)]]
       db (re-pattern (str "(?i)" search-term)))) 