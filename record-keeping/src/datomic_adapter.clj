(ns record-keeping.datomic-adapter
  (:require [datomic.api :as d]
            [record-keeping.schema :as schema]
            [clojure.tools.logging :as log]
            [clojure.string :as str]))

;; Database connection management

(defn create-database!
  "Create Datomic database if it doesn't exist"
  [uri]
  (log/info "Creating database:" uri)
  (d/create-database uri))

(defn get-connection
  "Get connection to Datomic database, creating if necessary"
  [config]
  (let [uri (get-in config [:datomic :uri])]
    (when (get-in config [:datomic :create-if-not-exists])
      (create-database! uri))
    (let [conn (d/connect uri)]
      (log/info "Connected to Datomic database:" uri)
      conn)))

(defn install-schema!
  "Install schema if not already present"
  [conn]
  (log/info "Installing Datomic schema...")
  (try
    @(d/transact conn schema/full-schema)
    (log/info "Schema installation completed")
    (catch Exception e
      (log/warn "Schema installation failed (may already exist):" (.getMessage e)))))

;; Entity transformation functions

(defn transform-function-attrs
  "Transform function-specific attributes"
  [entity]
  (let [attrs {}
        metadata (:metadata entity)]
    (cond-> attrs
      (:signature metadata) (assoc :function/signature (:signature metadata))
      (:docstring metadata) (assoc :function/docstring (:docstring metadata))
      (:return_type metadata) (assoc :function/return-type (:return_type metadata))
      (:complexity metadata) (assoc :function/complexity (:complexity metadata))
      (some? (:is_async metadata)) (assoc :function/is-async (:is_async metadata))
      (some? (:is_generator metadata)) (assoc :function/is-generator (:is_generator metadata)))))

(defn transform-class-attrs
  "Transform class-specific attributes"
  [entity]
  (let [attrs {}
        metadata (:metadata entity)]
    (cond-> attrs
      (:docstring metadata) (assoc :class/docstring (:docstring metadata))
      (some? (:is_abstract metadata)) (assoc :class/is-abstract (:is_abstract metadata))
      (some? (:is_interface metadata)) (assoc :class/is-interface (:is_interface metadata)))))

(defn transform-module-attrs
  "Transform module-specific attributes"
  [entity]
  (let [attrs {}
        metadata (:metadata entity)]
    (cond-> attrs
      (:path metadata) (assoc :module/path (:path metadata))
      (:size_loc metadata) (assoc :module/size-loc (:size_loc metadata)))))

(defn transform-entity-to-datomic
  "Transform extracted entity to Datomic transaction format"
  [entity repository-id]
  (let [entity-id (java.util.UUID/randomUUID)
        base-attrs {:db/id (d/tempid :db.part/user)
                    :code-entity/id entity-id
                    :code-entity/type (keyword (:type entity))
                    :code-entity/name (:name entity)
                    :code-entity/language (keyword (or (:language entity) "unknown"))}
        
        ;; Add source location if available
        location-attrs (when-let [loc (:source_location entity)]
                        (cond-> {}
                          (:file_path loc) (assoc :code-entity/file-path (:file_path loc))
                          (:start_line loc) (assoc :code-entity/start-line (:start_line loc))
                          (:end_line loc) (assoc :code-entity/end-line (:end_line loc))
                          (:start_col loc) (assoc :code-entity/start-col (:start_col loc))
                          (:end_col loc) (assoc :code-entity/end-col (:end_col loc))))
        
        ;; Add source code if available
        source-attrs (when (:source_code entity)
                       {:code-entity/source-code (:source_code entity)})
        
        ;; Add type-specific attributes
        type-specific-attrs (case (keyword (:type entity))
                             :function (transform-function-attrs entity)
                             :class (transform-class-attrs entity)
                             :module (transform-module-attrs entity)
                             :variable {}
                             {})]
    
    (merge base-attrs location-attrs source-attrs type-specific-attrs)))



(defn transform-relationship-to-datomic
  "Transform extracted relationship to Datomic transaction format"
  [relationship entity-id-map]
  (let [source-id (get entity-id-map (:from relationship))
        target-id (get entity-id-map (:to relationship))]
    (when (and source-id target-id)
      {:db/id (d/tempid :db.part/user)
       :relationship/id (java.util.UUID/randomUUID)
       :relationship/type (keyword (:type relationship))
       :relationship/source [:code-entity/id source-id]
       :relationship/target [:code-entity/id target-id]
       :relationship/weight (or (:weight relationship) 1.0)
       :relationship/context (or (:context relationship) "")
       :relationship/frequency (or (:frequency relationship) 1)})))

;; Storage functions

(defn store-extraction-result!
  "Store complete extraction result in Datomic"
  [conn extraction-result repository-path]
  (let [repository-id (java.util.UUID/randomUUID)
        
        ;; Store repository metadata
        repo-tx {:db/id (d/tempid :db.part/user)
                 :repository/id repository-id
                 :repository/path repository-path
                 :repository/name (last (str/split repository-path #"/"))
                 :repository/ingestion-date (java.util.Date.)
                 :repository/languages (->> (:entities extraction-result)
                                           (map :language)
                                           (remove nil?)
                                           (map keyword)
                                           set
                                           vec)}
        
        ;; Transform entities
        entity-txs (map #(transform-entity-to-datomic % repository-id) 
                       (:entities extraction-result))
        
        ;; Create entity name -> ID mapping for relationships
        entity-id-map (->> (map vector 
                               (map :name (:entities extraction-result))
                               (map :code-entity/id entity-txs))
                          (into {}))
        
        ;; Transform relationships
        relationship-txs (->> (:relationships extraction-result)
                             (map #(transform-relationship-to-datomic % entity-id-map))
                             (remove nil?))]
    
    (log/info "Storing" (count entity-txs) "entities and" 
              (count relationship-txs) "relationships")
    
    ;; Store in batches to avoid transaction size limits
    (let [all-txs (concat [repo-tx] entity-txs relationship-txs)
          batch-size 100
          batches (partition-all batch-size all-txs)]
      
      (doseq [batch batches]
        (log/debug "Storing batch of" (count batch) "entities")
        @(d/transact conn batch)))
    
    (log/info "Successfully stored extraction result for repository:" repository-path)
    {:success true
     :repository-id repository-id
     :entities-stored (count entity-txs)
     :relationships-stored (count relationship-txs)}))

(defn store-entities!
  "Store entities in Datomic (legacy function for compatibility)"
  [conn entities]
  (let [entity-txs (map #(transform-entity-to-datomic % nil) entities)]
    @(d/transact conn entity-txs)
    (log/info "Stored" (count entity-txs) "entities")))

(defn store-relationships!
  "Store relationships in Datomic (legacy function for compatibility)"
  [conn relationships entity-id-map]
  (let [relationship-txs (->> relationships
                             (map #(transform-relationship-to-datomic % entity-id-map))
                             (remove nil?))]
    @(d/transact conn relationship-txs)
    (log/info "Stored" (count relationship-txs) "relationships")))

;; Query helpers

(defn get-database
  "Get current database value"
  [conn]
  (d/db conn))

(defn find-entity-by-name
  "Find entity by name"
  [db entity-name]
  (d/q '[:find (pull ?e [*]) .
         :in $ ?name
         :where
         [?e :code-entity/name ?name]]
       db entity-name))

(defn get-repository-stats
  "Get statistics for a repository"
  [db repository-id]
  (let [entities (d/q '[:find (count ?e) .
                        :where
                        [?e :code-entity/type]]
                      db)
        functions (d/q '[:find (count ?e) .
                         :where
                         [?e :code-entity/type :function]]
                       db)
        classes (d/q '[:find (count ?e) .
                       :where
                       [?e :code-entity/type :class]]
                     db)
        relationships (d/q '[:find (count ?r) .
                             :where
                             [?r :relationship/type]]
                           db)]
    {:total-entities entities
     :functions functions
     :classes classes
     :relationships relationships}))

;; Utility functions

(defn cleanup-database!
  "Remove all data from database (for testing)"
  [conn]
  (log/warn "Cleaning up database - removing all data")
  (let [db (d/db conn)
        all-entities (d/q '[:find [?e ...]
                            :where
                            [?e :code-entity/id]]
                          db)
        all-relationships (d/q '[:find [?r ...]
                                 :where
                                 [?r :relationship/id]]
                               db)
        all-repos (d/q '[:find [?repo ...]
                         :where
                         [?repo :repository/id]]
                       db)]
    
    ;; Retract all entities
    (when (seq all-entities)
      @(d/transact conn (map (fn [e] [:db/retractEntity e]) all-entities)))
    (when (seq all-relationships)
      @(d/transact conn (map (fn [r] [:db/retractEntity r]) all-relationships)))
    (when (seq all-repos)
      @(d/transact conn (map (fn [repo] [:db/retractEntity repo]) all-repos)))
    
    (log/info "Database cleanup completed"))) 