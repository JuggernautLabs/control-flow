use client_implementations::client::{FlexibleClient, QueryResolver};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ffww::error::BuildError;

#[derive(Debug, Deserialize, JsonSchema)]
struct DomainAnalysis {
    /// Primary software category identified from the description
    #[schemars(description = "Main software type: 'command_line_tool', 'web_application', 'api_service', 'data_processor', 'gui_application', 'library', 'system_service'")]
    primary_domain: String,

    /// Secondary characteristics that influence requirements
    #[schemars(description = "Additional domain aspects like 'real_time', 'distributed', 'security_critical', 'performance_critical', 'user_facing'")]
    domain_characteristics: Vec<String>,

    /// Key entities or concepts mentioned in the description
    #[schemars(description = "Important nouns and concepts that will need to be modeled in the requirements")]
    key_entities: Vec<String>,

    /// Actions or capabilities the software must provide
    #[schemars(description = "Verbs and actions that indicate required functionality")]
    required_capabilities: Vec<String>,

    /// Technical constraints mentioned or implied
    #[schemars(description = "Platform, language, framework, or other technical constraints")]
    technical_constraints: Vec<String>,

    /// Confidence in the domain classification
    #[schemars(range(min = 0.0, max = 1.0), description = "How confident is this domain analysis?")]
    classification_confidence: f64,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RequirementField {
    /// Name of the requirement field in snake_case
    #[schemars(description = "Field name for the requirement struct")]
    field_name: String,

    /// Human-readable description of what this field captures
    #[schemars(description = "Clear description for the field's purpose")]
    field_description: String,

    /// Rust type for the field
    #[schemars(description = "Rust type: 'String', 'Vec<String>', 'bool', 'f64', 'u32', etc.")]
    field_type: String,

    /// Whether this field is required or optional
    #[schemars(description = "Is this field mandatory for this domain?")]
    required: bool,

    /// Validation constraints for the field
    #[schemars(description = "Additional validation like ranges, enums, or patterns")]
    validation_constraints: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct DynamicRequirementSchema {
    /// Name for the generated requirements struct
    #[schemars(description = "Descriptive name for the requirements type (e.g., 'WebApplicationRequirements')")]
    schema_name: String,

    /// Overall description of what these requirements capture
    #[schemars(description = "High-level description of the requirement schema's purpose")]
    schema_description: String,

    /// All fields that should be included in the requirements struct
    #[schemars(description = "Complete list of fields for the requirements structure")]
    fields: Vec<RequirementField>,

    /// Domain-specific validation rules that apply to this schema
    #[schemars(description = "Special validation logic specific to this domain")]
    validation_rules: Vec<String>,

    /// Confidence that this schema captures all necessary requirements
    #[schemars(range(min = 0.0, max = 1.0), description = "How complete is this requirement schema?")]
    schema_completeness: f64,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ImplementationGuidance {
    /// Suggested programming languages for this domain
    #[schemars(description = "Languages well-suited for this type of software")]
    recommended_languages: Vec<String>,

    /// Common architectural patterns for this domain
    #[schemars(description = "Architectural approaches that work well for this software type")]
    architectural_patterns: Vec<String>,

    /// Testing strategies appropriate for this domain
    #[schemars(description = "Testing approaches that should be emphasized")]
    testing_strategies: Vec<String>,

    /// Key quality attributes to focus on
    #[schemars(description = "Quality aspects most important for this domain (performance, security, usability, etc.)")]
    quality_priorities: Vec<String>,

    /// Common pitfalls to avoid in this domain
    #[schemars(description = "Known issues or antipatterns to watch for")]
    domain_pitfalls: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SchemaGenerationResult {
    /// Analysis of the problem domain
    domain_analysis: DomainAnalysis,

    /// Generated requirements schema structure
    requirement_schema: DynamicRequirementSchema,

    /// Implementation guidance for this domain
    implementation_guidance: ImplementationGuidance,

    /// Overall confidence in the schema generation
    #[schemars(range(min = 0.0, max = 1.0), description = "Overall confidence in this schema generation result")]
    generation_confidence: f64,
}

impl SchemaGenerationResult {
    /// Generate the actual Rust code for the requirements struct
    pub fn generate_rust_struct(&self) -> String {
        let mut struct_code = String::new();
        
        struct_code.push_str(&format!(
            "#[derive(Debug, Deserialize, JsonSchema)]\n/// {}\nstruct {} {{\n",
            self.requirement_schema.schema_description,
            self.requirement_schema.schema_name
        ));

        for field in &self.requirement_schema.fields {
            struct_code.push_str(&format!(
                "    /// {}\n    #[schemars(description = \"{}\")]\n    {}: {},\n\n",
                field.field_description,
                field.field_description,
                field.field_name,
                if field.required { 
                    field.field_type.clone() 
                } else { 
                    format!("Option<{}>", field.field_type) 
                }
            ));
        }

        struct_code.push_str("}\n");
        struct_code
    }

    /// Generate a default instance for testing the schema
    pub fn generate_default_instance(&self) -> String {
        let mut default_code = String::new();
        
        default_code.push_str(&format!(
            "impl Default for {} {{\n    fn default() -> Self {{\n        Self {{\n",
            self.requirement_schema.schema_name
        ));

        for field in &self.requirement_schema.fields {
            let default_value = match field.field_type.as_str() {
                "String" => "String::new()".to_string(),
                "Vec<String>" => "Vec::new()".to_string(),
                "bool" => "false".to_string(),
                "f64" => "0.0".to_string(),
                "u32" | "i32" => "0".to_string(),
                _ => "Default::default()".to_string(),
            };

            default_code.push_str(&format!(
                "            {}: {},\n",
                field.field_name,
                if field.required { 
                    default_value 
                } else { 
                    "None".to_string() 
                }
            ));
        }

        default_code.push_str("        }\n    }\n}\n");
        default_code
    }
}

// Usage function that integrates with the existing framework
async fn generate_dynamic_schema(
    resolver: &QueryResolver<FlexibleClient>,
    problem_description: &str,
) -> Result<SchemaGenerationResult, BuildError> {
    let schema_prompt = format!(
        "Analyze this software development request and generate a comprehensive requirements schema:

        Problem Description: '{}'

        Perform the following analysis:
        1. Identify the primary software domain and key characteristics
        2. Extract entities, capabilities, and constraints
        3. Design a complete requirements structure with appropriate fields
        4. Provide implementation guidance specific to this domain
        
        Focus on creating a schema that captures all aspects necessary for reliable software generation in this domain.",
        problem_description
    );

    let schema_result: SchemaGenerationResult = resolver
        .query_with_schema(schema_prompt)
        .await?;

    Ok(schema_result)
}

    fn main() {
        let schema_result = SchemaGenerationResult {
            domain_analysis: DomainAnalysis {
                primary_domain: "web_application".to_string(),
                domain_characteristics: vec!["user_facing".to_string()],
                key_entities: vec!["user".to_string(), "post".to_string()],
                required_capabilities: vec!["authenticate".to_string(), "create_post".to_string()],
                technical_constraints: vec!["web_browser".to_string()],
                classification_confidence: 0.9,
            },
            requirement_schema: DynamicRequirementSchema {
                schema_name: "BlogApplicationRequirements".to_string(),
                schema_description: "Requirements for a blog application".to_string(),
                fields: vec![
                    RequirementField {
                        field_name: "authentication_method".to_string(),
                        field_description: "How users authenticate".to_string(),
                        field_type: "String".to_string(),
                        required: true,
                        validation_constraints: vec![],
                    }
                ],
                validation_rules: vec![],
                schema_completeness: 0.8,
            },
            implementation_guidance: ImplementationGuidance {
                recommended_languages: vec!["Python".to_string(), "JavaScript".to_string()],
                architectural_patterns: vec!["MVC".to_string()],
                testing_strategies: vec!["integration_tests".to_string()],
                quality_priorities: vec!["security".to_string(), "usability".to_string()],
                domain_pitfalls: vec!["XSS vulnerabilities".to_string()],
            },
            generation_confidence: 0.85,
        };

        let generated_struct = schema_result.generate_rust_struct();
        assert!(generated_struct.contains("BlogApplicationRequirements"));
        assert!(generated_struct.contains("authentication_method"));
}
