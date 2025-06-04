use async_trait::async_trait;
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::env;

use crate::types::*;
use crate::engine::*;
use crate::errors::{StoryGenerationError, Result};

pub struct OpenAIStoryEngine {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
}

#[derive(Deserialize)]
struct OpenAIResponseMessage {
    content: String,
}

impl OpenAIStoryEngine {
    pub fn new() -> Result<Self> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| StoryGenerationError::ConfigurationError {
                message: "OPENAI_API_KEY environment variable not set".to_string(),
            })?;

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
            model: "gpt-3.5-turbo".to_string(),
        })
    }

    async fn call_openai(&self, prompt: &str) -> Result<String> {
        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![
                OpenAIMessage {
                    role: "system".to_string(),
                    content: "You are a creative storyteller for interactive adventures. Generate engaging, immersive story content with clear choices for the player.".to_string(),
                },
                OpenAIMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            max_tokens: 500,
            temperature: 0.8,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| StoryGenerationError::AiServiceError {
                message: format!("Failed to call OpenAI API: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(StoryGenerationError::AiServiceError {
                message: format!("OpenAI API error {}: {}", status, error_text),
            });
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| StoryGenerationError::AiServiceError {
                message: format!("Failed to parse OpenAI response: {}", e),
            })?;

        openai_response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| StoryGenerationError::AiServiceError {
                message: "No response from OpenAI".to_string(),
            })
    }

    fn parse_story_response(&self, response: &str) -> Result<(String, Vec<String>)> {
        // Simple parsing - look for "SITUATION:" and "CHOICES:" markers
        let lines: Vec<&str> = response.lines().collect();
        let mut situation = String::new();
        let mut choices = Vec::new();
        let mut current_section = "";

        for line in lines {
            let line = line.trim();
            if line.to_uppercase().starts_with("SITUATION:") {
                current_section = "situation";
                situation = line.strip_prefix("SITUATION:").unwrap_or(line).trim().to_string();
            } else if line.to_uppercase().starts_with("CHOICES:") {
                current_section = "choices";
            } else if line.starts_with("1.") || line.starts_with("2.") || line.starts_with("3.") {
                if current_section == "choices" {
                    let choice = line.splitn(2, '.').nth(1).unwrap_or(line).trim().to_string();
                    choices.push(choice);
                }
            } else if !line.is_empty() {
                if current_section == "situation" {
                    if !situation.is_empty() {
                        situation.push(' ');
                    }
                    situation.push_str(line);
                }
            }
        }

        // Fallback parsing if structured format isn't found
        if situation.is_empty() {
            situation = response.split('\n').take(3).collect::<Vec<_>>().join(" ");
        }

        if choices.is_empty() {
            choices = vec![
                "Continue the adventure".to_string(),
                "Look around carefully".to_string(),
                "Try a different approach".to_string(),
            ];
        }

        Ok((situation, choices))
    }
}

#[async_trait]
impl StoryGenerationEngine for OpenAIStoryEngine {
    async fn generate_initial_story(
        &self,
        prompt: &str,
        constraints: ProjectConstraints,
    ) -> Result<StoryGraph> {
        let enhanced_prompt = format!(
            "Create the beginning of an interactive adventure story based on: '{}'
            
            Experience level: {:?}
            
            Please format your response as:
            SITUATION: [A compelling opening situation in 2-3 sentences]
            CHOICES:
            1. [First choice]
            2. [Second choice] 
            3. [Third choice]",
            prompt, constraints.experience_level
        );

        let ai_response = self.call_openai(&enhanced_prompt).await?;
        let (situation, choice_descriptions) = self.parse_story_response(&ai_response)?;

        let mut story = StoryGraph::default();
        story.title = format!("Adventure: {}", prompt);
        story.description = Some("An AI-generated interactive adventure".to_string());

        let choices: Vec<Choice> = choice_descriptions.iter().map(|desc| {
            Choice {
                id: Uuid::new_v4(),
                description: desc.clone(),
                target_node_id: None,
                weight: 1.0,
                feasibility_score: Some(0.8),
                consequences: vec![],
                metadata: ChoiceMetadata {
                    confidence_score: Some(0.8),
                    risk_level: RiskLevel::Medium,
                    time_estimate: Some("5 minutes".to_string()),
                    dependencies: vec![],
                },
            }
        }).collect();

        let root_id = Uuid::new_v4();
        let root_node = StoryNode {
            id: root_id,
            node_type: NodeType::Start,
            situation,
            choices,
            state: NodeState::Current,
            complexity_score: Some(0.5),
            metadata: NodeMetadata {
                generated_by_ai: true,
                validation_status: ValidationStatus::Valid,
                user_annotations: vec![],
                technical_details: vec![],
            },
            created_at: Utc::now(),
        };

        story.nodes.insert(root_id, root_node);
        story.root_node_id = Some(root_id);
        story.current_node_id = Some(root_id);
        story.metadata.project_constraints = constraints;

        Ok(story)
    }

    async fn generate_choices(
        &self,
        current_node: &StoryNode,
        context: &StoryContext,
    ) -> Result<Vec<Choice>> {
        let prompt = format!(
            "Given this situation: '{}'
            
            Previous choices made: {}
            
            Generate 3 new choices for what the player can do next. Format as:
            CHOICES:
            1. [Choice 1]
            2. [Choice 2]
            3. [Choice 3]",
            current_node.situation,
            context.previous_choices.len()
        );

        let ai_response = self.call_openai(&prompt).await?;
        let (_, choice_descriptions) = self.parse_story_response(&ai_response)?;

        let choices: Vec<Choice> = choice_descriptions.iter().map(|desc| {
            Choice {
                id: Uuid::new_v4(),
                description: desc.clone(),
                target_node_id: None,
                weight: 1.0,
                feasibility_score: Some(0.8),
                consequences: vec![],
                metadata: ChoiceMetadata {
                    confidence_score: Some(0.8),
                    risk_level: RiskLevel::Medium,
                    time_estimate: Some("5 minutes".to_string()),
                    dependencies: vec![],
                },
            }
        }).collect();

        Ok(choices)
    }

    async fn expand_choice(
        &self,
        choice: &Choice,
        context: &StoryContext,
    ) -> Result<StoryNode> {
        let prompt = format!(
            "The player chose: '{}'
            
            Previous story context: {} previous choices made
            
            What happens next? Create a new situation and 3 choices. Format as:
            SITUATION: [New situation in 2-3 sentences]
            CHOICES:
            1. [Choice 1]
            2. [Choice 2]
            3. [Choice 3]",
            choice.description,
            context.previous_choices.len()
        );

        let ai_response = self.call_openai(&prompt).await?;
        let (situation, choice_descriptions) = self.parse_story_response(&ai_response)?;

        let new_choices: Vec<Choice> = choice_descriptions.iter().map(|desc| {
            Choice {
                id: Uuid::new_v4(),
                description: desc.clone(),
                target_node_id: None,
                weight: 1.0,
                feasibility_score: Some(0.8),
                consequences: vec![],
                metadata: ChoiceMetadata {
                    confidence_score: Some(0.8),
                    risk_level: RiskLevel::Medium,
                    time_estimate: Some("5 minutes".to_string()),
                    dependencies: vec![],
                },
            }
        }).collect();

        Ok(StoryNode {
            id: Uuid::new_v4(),
            node_type: NodeType::Decision,
            situation,
            choices: new_choices,
            state: NodeState::Current,
            complexity_score: Some(0.6),
            metadata: NodeMetadata {
                generated_by_ai: true,
                validation_status: ValidationStatus::Valid,
                user_annotations: vec![],
                technical_details: vec![],
            },
            created_at: Utc::now(),
        })
    }

    async fn validate_coherence(
        &self,
        proposed_node: &StoryNode,
        existing_graph: &StoryGraph,
    ) -> Result<CoherenceReport> {
        let prompt = format!(
            "Evaluate the coherence of this story continuation:
            
            Story title: {}
            New situation: '{}'
            
            Does this fit well with the existing story? Rate coherence from 0.0 to 1.0 and explain.",
            existing_graph.title,
            proposed_node.situation
        );

        let ai_response = self.call_openai(&prompt).await?;
        
        // Simple parsing for coherence score
        let confidence_score = if ai_response.to_lowercase().contains("good") || 
                                 ai_response.to_lowercase().contains("coherent") {
            0.8
        } else if ai_response.to_lowercase().contains("poor") || 
                  ai_response.to_lowercase().contains("inconsistent") {
            0.3
        } else {
            0.6
        };

        Ok(CoherenceReport {
            is_coherent: confidence_score > 0.5,
            confidence_score,
            issues: vec![],
            suggestions: vec![ai_response],
        })
    }

    async fn ask_clarification_question(
        &self,
        uncertainty: &UncertaintyContext,
    ) -> Result<Question> {
        Ok(Question {
            id: Uuid::new_v4(),
            question_type: QuestionType::MultipleChoice,
            prompt: format!("I need clarification about: {}", uncertainty.missing_context.join(", ")),
            options: Some(vec![
                QuestionOption {
                    id: "continue".to_string(),
                    label: "Continue as planned".to_string(),
                    value: serde_json::Value::String("continue".to_string()),
                    description: Some("Proceed with the current story direction".to_string()),
                },
                QuestionOption {
                    id: "clarify".to_string(),
                    label: "Provide more details".to_string(),
                    value: serde_json::Value::String("clarify".to_string()),
                    description: Some("Give more context about what you want".to_string()),
                },
            ]),
            validation_rules: ValidationRules::default(),
            context: QuestionContext {
                uncertainty_areas: uncertainty.missing_context.clone(),
                missing_context: vec![],
                conflicting_choices: vec![],
                priority_level: 1,
            },
            created_at: Utc::now(),
        })
    }
}

pub struct ClaudeStoryEngine {
    client: reqwest::Client,
    api_key: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    temperature: f32,
    messages: Vec<ClaudeMessage>,
    system: String,
}

#[derive(Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Deserialize)]
struct ClaudeContent {
    text: String,
}

impl ClaudeStoryEngine {
    pub fn new() -> Result<Self> {
        dotenvy::dotenv().ok();
        
        let api_key = env::var("ANTHROPIC_API_KEY")
            .map_err(|_| StoryGenerationError::ConfigurationError {
                message: "ANTHROPIC_API_KEY environment variable not set".to_string(),
            })?;

        let model = env::var("CLAUDE_MODEL")
            .unwrap_or_else(|_| "claude-3-5-haiku-20241022".to_string());

        let max_tokens = env::var("MAX_TOKENS")
            .unwrap_or_else(|_| "2048".to_string())
            .parse()
            .unwrap_or(2048);

        let temperature = env::var("TEMPERATURE")
            .unwrap_or_else(|_| "0.7".to_string())
            .parse()
            .unwrap_or(0.7);

        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
            model,
            max_tokens,
            temperature,
        })
    }

    async fn call_claude(&self, prompt: &str) -> Result<String> {
        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            messages: vec![
                ClaudeMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                }
            ],
            system: "You are a collaborative planning partner having a conversation about what work should be investigated next. Your role is to help excavate ideas from the user's mind through thoughtful questions and suggestions. Think like a skilled project manager or architect who asks insightful questions about priorities, dependencies, and next steps. When presenting options, frame them as collaborative suggestions: 'What if we...', 'Should we investigate...', 'It might be worth exploring...'. Never pretend implementation is happening - focus on planning, analysis, and decision-making. When something is fully specified, suggest 'This seems ready to implement - should we move forward with [specific item]?'".to_string(),
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("content-type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .map_err(|e| StoryGenerationError::AiServiceError {
                message: format!("Failed to call Claude API: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(StoryGenerationError::AiServiceError {
                message: format!("Claude API error {}: {}", status, error_text),
            });
        }

        let claude_response: ClaudeResponse = response
            .json()
            .await
            .map_err(|e| StoryGenerationError::AiServiceError {
                message: format!("Failed to parse Claude response: {}", e),
            })?;

        claude_response
            .content
            .first()
            .map(|content| content.text.clone())
            .ok_or_else(|| StoryGenerationError::AiServiceError {
                message: "No response content from Claude".to_string(),
            })
    }

    fn parse_story_response(&self, response: &str) -> Result<(String, Vec<String>)> {
        let lines: Vec<&str> = response.lines().collect();
        let mut situation = String::new();
        let mut choices = Vec::new();
        let mut current_section = "";

        for line in lines {
            let line = line.trim();
            if line.to_uppercase().starts_with("SITUATION:") {
                current_section = "situation";
                situation = line.strip_prefix("SITUATION:").unwrap_or(line).trim().to_string();
            } else if line.to_uppercase().starts_with("CHOICES:") {
                current_section = "choices";
            } else if line.starts_with("1.") || line.starts_with("2.") || line.starts_with("3.") || line.starts_with("4.") {
                if current_section == "choices" {
                    let choice = line.splitn(2, '.').nth(1).unwrap_or(line).trim().to_string();
                    choices.push(choice);
                }
            } else if !line.is_empty() {
                if current_section == "situation" {
                    if !situation.is_empty() {
                        situation.push(' ');
                    }
                    situation.push_str(line);
                }
            }
        }

        if situation.is_empty() {
            situation = response.split('\n').take(3).collect::<Vec<_>>().join(" ");
        }

        if choices.is_empty() {
            choices = vec![
                "What if we broke this into smaller, more manageable pieces?".to_string(),
                "Should we gather more requirements and context first?".to_string(),
                "It might be worth starting with a proof of concept".to_string(),
                "We could research what solutions already exist".to_string(),
            ];
        }

        Ok((situation, choices))
    }
}

#[async_trait]
impl StoryGenerationEngine for ClaudeStoryEngine {
    async fn generate_initial_story(
        &self,
        prompt: &str,
        constraints: ProjectConstraints,
    ) -> Result<StoryGraph> {
        let enhanced_prompt = format!(
            "Let's start planning together for: '{}'
            
            Here's what I know about the project:
            - Timeline: {}
            - Team Size: {}
            - Experience Level: {:?}
            - Budget: {}
            - Technical Constraints: {}
            - Business Constraints: {}
            
            I'd like to have a collaborative conversation about how we should approach this. Think about what questions we need to answer and what areas we should investigate first.
            
            Please format your response as:
            SITUATION: [A conversational assessment of where we are with this project and what we need to figure out - speak directly to me as a collaborator]
            CHOICES:
            1. [What if we started by investigating...]
            2. [Should we first look into...]
            3. [It might be worth exploring...]
            4. [We could begin by analyzing...]",
            prompt, 
            constraints.timeline.as_deref().unwrap_or("Not specified"),
            constraints.team_size.unwrap_or(0),
            constraints.experience_level,
            constraints.budget.as_deref().unwrap_or("Not specified"),
            constraints.technical_constraints.join(", "),
            constraints.business_constraints.join(", ")
        );

        let ai_response = self.call_claude(&enhanced_prompt).await?;
        let (situation, choice_descriptions) = self.parse_story_response(&ai_response)?;

        let mut story = StoryGraph::default();
        story.title = format!("Planning Session: {}", prompt);
        story.description = Some("Collaborative planning conversation".to_string());

        let choices: Vec<Choice> = choice_descriptions.iter().map(|desc| {
            Choice {
                id: Uuid::new_v4(),
                description: desc.clone(),
                target_node_id: None,
                weight: 1.0,
                feasibility_score: Some(0.8),
                consequences: vec![],
                metadata: ChoiceMetadata {
                    confidence_score: Some(0.85),
                    risk_level: RiskLevel::Medium,
                    time_estimate: Some("1-2 weeks".to_string()),
                    dependencies: vec![],
                },
            }
        }).collect();

        let root_id = Uuid::new_v4();
        let root_node = StoryNode {
            id: root_id,
            node_type: NodeType::Start,
            situation,
            choices,
            state: NodeState::Current,
            complexity_score: Some(0.5),
            metadata: NodeMetadata {
                generated_by_ai: true,
                validation_status: ValidationStatus::Valid,
                user_annotations: vec![],
                technical_details: vec![],
            },
            created_at: Utc::now(),
        };

        story.nodes.insert(root_id, root_node);
        story.root_node_id = Some(root_id);
        story.current_node_id = Some(root_id);
        story.metadata.project_constraints = constraints;

        Ok(story)
    }

    async fn generate_choices(
        &self,
        current_node: &StoryNode,
        context: &StoryContext,
    ) -> Result<Vec<Choice>> {
        let prompt = format!(
            "We're continuing our planning conversation. Here's where we are: '{}'
            
            We've made {} planning decisions so far. What should we investigate next?
            
            Please suggest 4 different directions we could explore, framed as collaborative questions or suggestions.
            
            Format as:
            CHOICES:
            1. [What if we...]
            2. [Should we look into...]
            3. [It might be worth investigating...]
            4. [We could also consider...]",
            current_node.situation,
            context.previous_choices.len()
        );

        let ai_response = self.call_claude(&prompt).await?;
        let (_, choice_descriptions) = self.parse_story_response(&ai_response)?;

        let choices: Vec<Choice> = choice_descriptions.iter().map(|desc| {
            Choice {
                id: Uuid::new_v4(),
                description: desc.clone(),
                target_node_id: None,
                weight: 1.0,
                feasibility_score: Some(0.8),
                consequences: vec![],
                metadata: ChoiceMetadata {
                    confidence_score: Some(0.8),
                    risk_level: RiskLevel::Medium,
                    time_estimate: Some("1-2 weeks".to_string()),
                    dependencies: vec![],
                },
            }
        }).collect();

        Ok(choices)
    }

    async fn expand_choice(
        &self,
        choice: &Choice,
        context: &StoryContext,
    ) -> Result<StoryNode> {
        let prompt = format!(
            "Great choice! You decided: '{}'
            
            Now that we've made this decision (and {} others before it), let's think about what this reveals and what we should tackle next.
            
            Please respond as a collaborative partner - what questions does this decision raise? What should we investigate now?
            
            Format as:
            SITUATION: [A conversational response about what this decision means and what we need to figure out next - speak directly to me]
            CHOICES:
            1. [What if we now looked at...]
            2. [Should we investigate...]
            3. [This makes me wonder about...]
            4. [We should probably explore...]",
            choice.description,
            context.previous_choices.len()
        );

        let ai_response = self.call_claude(&prompt).await?;
        let (situation, choice_descriptions) = self.parse_story_response(&ai_response)?;

        let new_choices: Vec<Choice> = choice_descriptions.iter().map(|desc| {
            Choice {
                id: Uuid::new_v4(),
                description: desc.clone(),
                target_node_id: None,
                weight: 1.0,
                feasibility_score: Some(0.8),
                consequences: vec![],
                metadata: ChoiceMetadata {
                    confidence_score: Some(0.8),
                    risk_level: RiskLevel::Medium,
                    time_estimate: Some("1-2 weeks".to_string()),
                    dependencies: vec![],
                },
            }
        }).collect();

        Ok(StoryNode {
            id: Uuid::new_v4(),
            node_type: NodeType::Decision,
            situation,
            choices: new_choices,
            state: NodeState::Current,
            complexity_score: Some(0.6),
            metadata: NodeMetadata {
                generated_by_ai: true,
                validation_status: ValidationStatus::Valid,
                user_annotations: vec![],
                technical_details: vec![],
            },
            created_at: Utc::now(),
        })
    }

    async fn validate_coherence(
        &self,
        proposed_node: &StoryNode,
        existing_graph: &StoryGraph,
    ) -> Result<CoherenceReport> {
        let prompt = format!(
            "As my planning collaborator, does this next step make sense in our conversation?
            
            Project: {}
            We've explored {} different planning areas so far
            New topic: '{}'
            
            From a collaborative planning perspective:
            - Does this follow naturally from our previous discussions?
            - Are we maintaining focus on the right priorities?
            - Does this seem like a logical next step in our planning process?
            
            Give me your honest assessment - should we continue down this path or refocus?",
            existing_graph.title,
            existing_graph.nodes.len(),
            proposed_node.situation
        );

        let ai_response = self.call_claude(&prompt).await?;
        
        let confidence_score = if ai_response.to_lowercase().contains("coherent") || 
                                 ai_response.to_lowercase().contains("logical") ||
                                 ai_response.to_lowercase().contains("makes sense") {
            0.85
        } else if ai_response.to_lowercase().contains("inconsistent") || 
                  ai_response.to_lowercase().contains("unrealistic") {
            0.3
        } else {
            0.6
        };

        Ok(CoherenceReport {
            is_coherent: confidence_score > 0.5,
            confidence_score,
            issues: vec![],
            suggestions: vec![ai_response],
        })
    }

    async fn ask_clarification_question(
        &self,
        uncertainty: &UncertaintyContext,
    ) -> Result<Question> {
        let prompt = format!(
            "I'm a bit unclear on some aspects of our planning discussion. Can you help me understand?
            
            I'm missing context on: {}
            I see some conflicting ideas around: {}
            These requirements seem ambiguous: {}
            
            As my planning partner, what question should I ask you to get clarity on the most important uncertainty?",
            uncertainty.missing_context.join(", "),
            uncertainty.conflicting_choices.iter().map(|c| c.description.as_str()).collect::<Vec<_>>().join(", "),
            uncertainty.ambiguous_requirements.join(", ")
        );

        let ai_response = self.call_claude(&prompt).await?;

        Ok(Question {
            id: Uuid::new_v4(),
            question_type: QuestionType::MultipleChoice,
            prompt: ai_response,
            options: Some(vec![
                QuestionOption {
                    id: "detailed".to_string(),
                    label: "Provide detailed requirements".to_string(),
                    value: serde_json::Value::String("detailed".to_string()),
                    description: Some("Give more specific technical and business requirements".to_string()),
                },
                QuestionOption {
                    id: "simple".to_string(),
                    label: "Keep it simple for now".to_string(),
                    value: serde_json::Value::String("simple".to_string()),
                    description: Some("Proceed with minimal viable approach".to_string()),
                },
                QuestionOption {
                    id: "research".to_string(),
                    label: "Research options first".to_string(),
                    value: serde_json::Value::String("research".to_string()),
                    description: Some("Investigate existing solutions and best practices".to_string()),
                },
            ]),
            validation_rules: ValidationRules::default(),
            context: QuestionContext {
                uncertainty_areas: uncertainty.missing_context.clone(),
                missing_context: vec![],
                conflicting_choices: vec![],
                priority_level: 1,
            },
            created_at: Utc::now(),
        })
    }
}