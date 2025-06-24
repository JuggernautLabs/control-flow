(ns record-keeping.extraction-prompt)

;; LLM Prompt for Code Knowledge Graph Extraction

(def extraction-prompt-template
  "You are an expert code analyst tasked with extracting structured information from source code to build a knowledge graph. 

## TASK
Analyze the provided code chunk and extract:
1. **Code Entities**: Functions, classes, modules, variables, interfaces, etc.
2. **Relationships**: Function calls, imports, inheritance, type usage, etc.
3. **Metadata**: Types, signatures, documentation, complexity indicators, etc.

## OUTPUT FORMAT
Return a JSON object with this exact structure:

```json
{
  \"entities\": [
    {
      \"id\": \"unique-id\",
      \"type\": \"function|class|module|variable|interface|trait\",
      \"name\": \"entity-name\",
      \"full_name\": \"fully.qualified.name\",
      \"source_location\": {
        \"start_line\": 10,
        \"end_line\": 25,
        \"start_col\": 0,
        \"end_col\": 1
      },
      \"source_code\": \"actual source code\",
      \"language\": \"python|javascript|typescript|java|clojure|rust|go\",
      \"metadata\": {
        // Entity-specific metadata based on type
      }
    }
  ],
  \"relationships\": [
    {
      \"type\": \"calls|imports|inherits|uses_type|contains|similar\",
      \"source_id\": \"source-entity-id\",
      \"target_id\": \"target-entity-id\",
      \"context\": \"description of relationship context\",
      \"metadata\": {
        // Relationship-specific metadata
      }
    }
  ]
}
```

## ENTITY TYPES & METADATA

### Functions
```json
{
  \"type\": \"function\",
  \"metadata\": {
    \"signature\": \"function_name(param1: type, param2: type) -> return_type\",
    \"parameters\": [
      {\"name\": \"param1\", \"type\": \"string\", \"default_value\": null, \"is_optional\": false}
    ],
    \"return_type\": \"return_type\",
    \"docstring\": \"function documentation\",
    \"is_async\": false,
    \"is_generator\": false,
    \"complexity_indicators\": [\"loops\", \"conditionals\", \"nested_calls\"],
    \"decorators\": [\"@decorator_name\"],
    \"visibility\": \"public|private|protected\"
  }
}
```

### Classes
```json
{
  \"type\": \"class\",
  \"metadata\": {
    \"docstring\": \"class documentation\",
    \"methods\": [\"method_ids\"],
    \"properties\": [\"property_ids\"],
    \"is_abstract\": false,
    \"is_interface\": false,
    \"decorators\": [\"@decorator_name\"],
    \"visibility\": \"public|private|protected\"
  }
}
```

### Modules
```json
{
  \"type\": \"module\",
  \"metadata\": {
    \"exports\": [\"exported_entity_ids\"],
    \"docstring\": \"module documentation\",
    \"size_loc\": 150
  }
}
```

### Variables
```json
{
  \"type\": \"variable\",
  \"metadata\": {
    \"data_type\": \"inferred_type\",
    \"is_constant\": false,
    \"scope\": \"global|local|class\",
    \"initial_value\": \"value if simple\"
  }
}
```

## RELATIONSHIP TYPES

### Function Calls
```json
{
  \"type\": \"calls\",
  \"metadata\": {
    \"line_number\": 15,
    \"arguments\": [\"arg1\", \"arg2\"],
    \"call_type\": \"direct|method|constructor\"
  }
}
```

### Imports
```json
{
  \"type\": \"imports\",
  \"metadata\": {
    \"import_type\": \"import|from_import|import_as\",
    \"alias\": \"alias_name\",
    \"imported_names\": [\"specific_imports\"]
  }
}
```

### Inheritance
```json
{
  \"type\": \"inherits\",
  \"metadata\": {
    \"inheritance_type\": \"extends|implements|mixin\"
  }
}
```

## EXTRACTION RULES

1. **Entity IDs**: Generate unique, descriptive IDs like \"function_calculate_metrics_line_45\" or \"class_DataProcessor_line_12\"

2. **Relationships**: Only create relationships between entities you can clearly identify in the code

3. **Source Location**: Provide accurate line/column numbers based on the code chunk

4. **Type Inference**: Make best effort to infer types from context, annotations, usage patterns

5. **Documentation**: Extract all docstrings, comments, and inline documentation

6. **Complexity**: Note indicators like nested loops, multiple conditionals, exception handling

7. **Context**: For relationships, provide meaningful context about where/how they occur

## SPECIAL HANDLING

- **Anonymous Functions**: Create entities with descriptive names like \"lambda_line_25\" 
- **Nested Entities**: Include parent-child containment relationships
- **External References**: Only create relationships to entities defined in the current chunk
- **Partial Definitions**: Handle incomplete code gracefully
- **Comments**: Extract meaningful comments as documentation

## EXAMPLE

Input code chunk:
```python
def calculate_metrics(data: pd.DataFrame, config: dict) -> dict:
    \"\"\"Calculate various metrics from input data.\"\"\"
    result = {}
    if config.get('include_mean'):
        result['mean'] = data.mean()
    return result

class DataProcessor:
    \"\"\"Main processor for data analysis.\"\"\"
    
    def __init__(self, config: dict):
        self.config = config
    
    def process(self, data):
        return calculate_metrics(data, self.config)
```

Expected output:
```json
{
  \"entities\": [
    {
      \"id\": \"function_calculate_metrics_line_1\",
      \"type\": \"function\",
      \"name\": \"calculate_metrics\",
      \"full_name\": \"calculate_metrics\",
      \"source_location\": {\"start_line\": 1, \"end_line\": 6, \"start_col\": 0, \"end_col\": 16},
      \"source_code\": \"def calculate_metrics(data: pd.DataFrame, config: dict) -> dict:\\n    \\\"\\\"\\\"Calculate various metrics from input data.\\\"\\\"\\\"\\n    result = {}\\n    if config.get('include_mean'):\\n        result['mean'] = data.mean()\\n    return result\",
      \"language\": \"python\",
      \"metadata\": {
        \"signature\": \"calculate_metrics(data: pd.DataFrame, config: dict) -> dict\",
        \"parameters\": [
          {\"name\": \"data\", \"type\": \"pd.DataFrame\", \"default_value\": null, \"is_optional\": false},
          {\"name\": \"config\", \"type\": \"dict\", \"default_value\": null, \"is_optional\": false}
        ],
        \"return_type\": \"dict\",
        \"docstring\": \"Calculate various metrics from input data.\",
        \"is_async\": false,
        \"complexity_indicators\": [\"conditionals\", \"method_calls\"]
      }
    },
    {
      \"id\": \"class_DataProcessor_line_8\",
      \"type\": \"class\",
      \"name\": \"DataProcessor\",
      \"full_name\": \"DataProcessor\",
      \"source_location\": {\"start_line\": 8, \"end_line\": 16, \"start_col\": 0, \"end_col\": 43},
      \"source_code\": \"class DataProcessor:\\n    \\\"\\\"\\\"Main processor for data analysis.\\\"\\\"\\\"\\n    \\n    def __init__(self, config: dict):\\n        self.config = config\\n    \\n    def process(self, data):\\n        return calculate_metrics(data, self.config)\",
      \"language\": \"python\",
      \"metadata\": {
        \"docstring\": \"Main processor for data analysis.\",
        \"methods\": [\"method___init___line_11\", \"method_process_line_14\"],
        \"is_abstract\": false
      }
    }
  ],
  \"relationships\": [
    {
      \"type\": \"calls\",
      \"source_id\": \"method_process_line_14\",
      \"target_id\": \"function_calculate_metrics_line_1\",
      \"context\": \"Method calls function with data and config parameters\",
      \"metadata\": {
        \"line_number\": 15,
        \"arguments\": [\"data\", \"self.config\"]
      }
    },
    {
      \"type\": \"contains\",
      \"source_id\": \"class_DataProcessor_line_8\",
      \"target_id\": \"method_process_line_14\",
      \"context\": \"Class contains method definition\"
    }
  ]
}
```

## IMPORTANT NOTES

- Be thorough but accurate - only extract what you can clearly identify
- Maintain consistency in naming and ID generation
- Handle edge cases gracefully
- Provide meaningful context for relationships
- Extract semantic information that goes beyond pure syntax

Now analyze the following code chunk:")

(def code-chunk-prompt
  "
## CODE TO ANALYZE

File: {{file-path}}
Language: {{language}}
Lines {{start-line}}-{{end-line}}:

```{{language}}
{{code-content}}
```

Please extract the structured information following the format above.")

;; Helper function to build the complete prompt
(defn build-extraction-prompt
  "Build complete extraction prompt with code chunk"
  [{:keys [file-path language start-line end-line code-content]}]
  (str extraction-prompt-template
       "\n\n"
       (-> code-chunk-prompt
           (clojure.string/replace "{{file-path}}" (str file-path))
           (clojure.string/replace "{{language}}" (str language))
           (clojure.string/replace "{{start-line}}" (str start-line))
           (clojure.string/replace "{{end-line}}" (str end-line))
           (clojure.string/replace "{{code-content}}" (str code-content)))))

;; Validation prompt for extracted data
(def validation-prompt
  "Review the extracted code knowledge graph data for accuracy and completeness:

## VALIDATION CHECKLIST

1. **Entity Completeness**: Are all significant code entities captured?
2. **Relationship Accuracy**: Do the relationships actually exist in the code?
3. **Metadata Quality**: Is the metadata accurate and useful?
4. **ID Consistency**: Are entity IDs unique and descriptive?
5. **Source Locations**: Are line numbers and positions correct?

## INPUT DATA
```json
{{extracted-data}}
```

## ORIGINAL CODE
```{{language}}
{{code-content}}
```

Please provide:
1. A corrected version of the JSON if needed
2. A brief explanation of any corrections made
3. A confidence score (1-10) for the extraction quality")

(defn build-validation-prompt
  "Build validation prompt for extracted data"
  [{:keys [extracted-data code-content language]}]
  (-> validation-prompt
      (clojure.string/replace "{{extracted-data}}" (str extracted-data))
      (clojure.string/replace "{{code-content}}" (str code-content))
      (clojure.string/replace "{{language}}" (str language)))) 