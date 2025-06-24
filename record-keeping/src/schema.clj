(ns record-keeping.schema
  (:require [datomic.api :as d]))

;; Datomic schema for code knowledge graph

(def code-entity-schema
  [;; Core entity identification
   {:db/ident :code-entity/id
    :db/valueType :db.type/uuid
    :db/cardinality :db.cardinality/one
    :db/unique :db.unique/identity
    :db/doc "Unique identifier for code entity"}

   {:db/ident :code-entity/type
    :db/valueType :db.type/keyword
    :db/cardinality :db.cardinality/one
    :db/doc "Type of code entity: :function, :class, :module, :variable"}

   {:db/ident :code-entity/name
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "Name of the code entity"}

   {:db/ident :code-entity/full-name
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "Fully qualified name including namespace/module"}

   ;; Source location
   {:db/ident :code-entity/file-path
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "File path containing this entity"}

   {:db/ident :code-entity/start-line
    :db/valueType :db.type/long
    :db/cardinality :db.cardinality/one
    :db/doc "Starting line number"}

   {:db/ident :code-entity/end-line
    :db/valueType :db.type/long
    :db/cardinality :db.cardinality/one
    :db/doc "Ending line number"}

   {:db/ident :code-entity/start-col
    :db/valueType :db.type/long
    :db/cardinality :db.cardinality/one
    :db/doc "Starting column number"}

   {:db/ident :code-entity/end-col
    :db/valueType :db.type/long
    :db/cardinality :db.cardinality/one
    :db/doc "Ending column number"}

   ;; Content and metadata
   {:db/ident :code-entity/source-code
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "Raw source code of the entity"}

   {:db/ident :code-entity/language
    :db/valueType :db.type/keyword
    :db/cardinality :db.cardinality/one
    :db/doc "Programming language"}

   {:db/ident :code-entity/hash
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "Hash of the source code for change detection"}

   ;; Embeddings
   {:db/ident :code-entity/embedding-vector
    :db/valueType :db.type/bytes
    :db/cardinality :db.cardinality/one
    :db/doc "Serialized embedding vector"}

   {:db/ident :code-entity/embedding-model
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "Model used to generate embedding"}

   {:db/ident :code-entity/embedding-dimension
    :db/valueType :db.type/long
    :db/cardinality :db.cardinality/one
    :db/doc "Dimension of embedding vector"}])

;; Function-specific attributes
(def function-schema
  [{:db/ident :function/signature
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "Function signature with parameters and return type"}

   {:db/ident :function/parameters
    :db/valueType :db.type/ref
    :db/cardinality :db.cardinality/many
    :db/doc "Function parameters"}

   {:db/ident :function/return-type
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "Return type of function"}

   {:db/ident :function/docstring
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "Function documentation string"}

   {:db/ident :function/complexity
    :db/valueType :db.type/long
    :db/cardinality :db.cardinality/one
    :db/doc "Cyclomatic complexity score"}

   {:db/ident :function/is-async
    :db/valueType :db.type/boolean
    :db/cardinality :db.cardinality/one
    :db/doc "Whether function is asynchronous"}

   {:db/ident :function/is-generator
    :db/valueType :db.type/boolean
    :db/cardinality :db.cardinality/one
    :db/doc "Whether function is a generator"}])

;; Parameter schema
(def parameter-schema
  [{:db/ident :parameter/name
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "Parameter name"}

   {:db/ident :parameter/type
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "Parameter type"}

   {:db/ident :parameter/default-value
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "Default value if any"}

   {:db/ident :parameter/is-optional
    :db/valueType :db.type/boolean
    :db/cardinality :db.cardinality/one
    :db/doc "Whether parameter is optional"}])

;; Class-specific attributes
(def class-schema
  [{:db/ident :class/docstring
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "Class documentation string"}

   {:db/ident :class/methods
    :db/valueType :db.type/ref
    :db/cardinality :db.cardinality/many
    :db/doc "Methods defined in this class"}

   {:db/ident :class/properties
    :db/valueType :db.type/ref
    :db/cardinality :db.cardinality/many
    :db/doc "Properties/fields of this class"}

   {:db/ident :class/is-abstract
    :db/valueType :db.type/boolean
    :db/cardinality :db.cardinality/one
    :db/doc "Whether class is abstract"}

   {:db/ident :class/is-interface
    :db/valueType :db.type/boolean
    :db/cardinality :db.cardinality/one
    :db/doc "Whether class is an interface"}])

;; Module-specific attributes
(def module-schema
  [{:db/ident :module/path
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "Module file path"}

   {:db/ident :module/name
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "Module name"}

   {:db/ident :module/exports
    :db/valueType :db.type/ref
    :db/cardinality :db.cardinality/many
    :db/doc "Entities exported by this module"}

   {:db/ident :module/size-loc
    :db/valueType :db.type/long
    :db/cardinality :db.cardinality/one
    :db/doc "Lines of code in module"}])

;; Relationship schema
(def relationship-schema
  [{:db/ident :relationship/id
    :db/valueType :db.type/uuid
    :db/cardinality :db.cardinality/one
    :db/unique :db.unique/identity
    :db/doc "Unique identifier for relationship"}

   {:db/ident :relationship/type
    :db/valueType :db.type/keyword
    :db/cardinality :db.cardinality/one
    :db/doc "Type of relationship: :calls, :imports, :inherits, :uses-type, :similar"}

   {:db/ident :relationship/source
    :db/valueType :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/doc "Source entity of relationship"}

   {:db/ident :relationship/target
    :db/valueType :db.type/ref
    :db/cardinality :db.cardinality/one
    :db/doc "Target entity of relationship"}

   {:db/ident :relationship/weight
    :db/valueType :db.type/double
    :db/cardinality :db.cardinality/one
    :db/doc "Weight/strength of relationship"}

   {:db/ident :relationship/context
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "Context information about relationship"}

   {:db/ident :relationship/frequency
    :db/valueType :db.type/long
    :db/cardinality :db.cardinality/one
    :db/doc "How often this relationship occurs"}])

;; Call relationship specific
(def call-relationship-schema
  [{:db/ident :call/line-number
    :db/valueType :db.type/long
    :db/cardinality :db.cardinality/one
    :db/doc "Line number where call occurs"}

   {:db/ident :call/arguments
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/many
    :db/doc "Arguments passed to function call"}])

;; Import relationship specific
(def import-relationship-schema
  [{:db/ident :import/import-type
    :db/valueType :db.type/keyword
    :db/cardinality :db.cardinality/one
    :db/doc "Type of import: :import, :from-import, :import-as"}

   {:db/ident :import/alias
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "Alias used for import"}

   {:db/ident :import/imported-names
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/many
    :db/doc "Specific names imported"}])

;; Repository metadata
(def repository-schema
  [{:db/ident :repository/id
    :db/valueType :db.type/uuid
    :db/cardinality :db.cardinality/one
    :db/unique :db.unique/identity
    :db/doc "Repository unique identifier"}

   {:db/ident :repository/path
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "Repository root path"}

   {:db/ident :repository/name
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "Repository name"}

   {:db/ident :repository/url
    :db/valueType :db.type/string
    :db/cardinality :db.cardinality/one
    :db/doc "Repository URL if available"}

   {:db/ident :repository/ingestion-date
    :db/valueType :db.type/instant
    :db/cardinality :db.cardinality/one
    :db/doc "When repository was ingested"}

   {:db/ident :repository/languages
    :db/valueType :db.type/keyword
    :db/cardinality :db.cardinality/many
    :db/doc "Programming languages found in repository"}])

;; Combine all schemas
(def full-schema
  (concat code-entity-schema
          function-schema
          parameter-schema
          class-schema
          module-schema
          relationship-schema
          call-relationship-schema
          import-relationship-schema
          repository-schema))

;; Schema installation function
(defn install-schema!
  "Install the schema in Datomic database"
  [conn]
  @(d/transact conn full-schema))

;; Enum values for common attributes
(def entity-types #{:function :class :module :variable :interface :trait})
(def relationship-types #{:calls :imports :inherits :uses-type :similar :contains})
(def programming-languages #{:python :javascript :typescript :java :clojure :rust :go})
(def import-types #{:import :from-import :import-as :require}) 