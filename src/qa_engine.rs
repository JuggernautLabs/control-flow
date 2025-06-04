use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::*;
use crate::engine::*;
use crate::errors::{StoryGenerationError, Result};

/// Represents a planning question in our collaborative conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningQuestion {
    pub id: Uuid,
    pub question: String,
    pub context: String,
    pub question_type: PlanningQuestionType,
    pub priority: QuestionPriority,
    pub dependencies: Vec<Uuid>, // Other questions that should be answered first
    pub metadata: PlanningQuestionMetadata,
    pub created_at: DateTime<Utc>,
}

/// Represents an answer or exploration of a planning question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningAnswer {
    pub id: Uuid,
    pub question_id: Uuid,
    pub answer: String,
    pub exploration_notes: Vec<String>,
    pub follow_up_questions: Vec<Uuid>,
    pub confidence: f32,
    pub metadata: PlanningAnswerMetadata,
    pub created_at: DateTime<Utc>,
}

/// Types of planning questions we can ask
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum PlanningQuestionType {
    Requirements,    // "What are the key requirements?"
    Architecture,    // "How should we structure this?"
    Dependencies,    // "What depends on what?"
    Priorities,      // "What should we tackle first?"
    Constraints,     // "What limitations do we have?"
    Risks,          // "What could go wrong?"
    Implementation, // "How would we actually build this?"
    Validation,     // "How do we know this works?"
}

/// Priority levels for questions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestionPriority {
    Critical,    // Must answer before proceeding
    High,        // Should answer soon
    Medium,      // Good to explore
    Low,         // Nice to have clarity
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningQuestionMetadata {
    pub generated_by_ai: bool,
    pub user_initiated: bool,
    pub complexity_score: Option<f32>,
    pub estimated_time_to_answer: Option<String>,
    pub related_topics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningAnswerMetadata {
    pub answer_method: AnswerMethod,
    pub validation_status: AnswerValidation,
    pub expert_review_needed: bool,
    pub implementation_readiness: f32, // 0.0 = just concepts, 1.0 = ready to implement
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnswerMethod {
    Collaborative,   // Discussed together
    Research,        // Investigated existing solutions
    Prototyping,     // Built something to test
    ExpertConsult,   // Asked domain expert
    UserFeedback,    // Got input from users
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum AnswerValidation {
    Tentative,       // First thoughts, needs validation
    Discussed,       // Talked through, seems reasonable
    Researched,      // Backed by research/examples
    Validated,       // Tested or confirmed
}

/// A QA Session represents a complete planning conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QASession {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub questions: HashMap<Uuid, PlanningQuestion>,
    pub answers: HashMap<Uuid, PlanningAnswer>,
    pub current_question_id: Option<Uuid>,
    pub project_constraints: ProjectConstraints,
    pub session_metadata: QASessionMetadata,
    pub pending_requests: HashMap<Uuid, PendingRequest>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents a request that failed and can be retried
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingRequest {
    pub id: Uuid,
    pub request_type: RequestType,
    pub context: String,
    pub user_input: Option<String>,
    pub error_message: String,
    pub retry_count: u32,
    pub created_at: DateTime<Utc>,
    pub last_attempt: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestType {
    InitialQuestions,
    ExploreQuestion { question_id: Uuid },
    DeepenAnswer { answer_id: Uuid },
    GenerateFollowUps { answer_id: Uuid },
    CustomQuestion { question_text: String },
}

/// Result of retrying a failed request
#[derive(Debug)]
pub enum RetryResult {
    Success,
    StillFailing(String),
    PermanentFailure(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QASessionMetadata {
    pub session_type: SessionType,
    pub completion_percentage: f32,
    pub critical_questions_answered: u32,
    pub total_critical_questions: u32,
    pub readiness_for_implementation: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionType {
    InitialPlanning,
    DetailedDesign,
    RiskAssessment,
    ImplementationPlanning,
    Review,
}

/// The QA Engine wraps the story generation engine to provide structured Q&A
pub struct QAEngine {
    story_engine: Box<dyn StoryGenerationEngine + Send + Sync>,
}

impl QAEngine {
    pub fn new(story_engine: Box<dyn StoryGenerationEngine + Send + Sync>) -> Self {
        Self { story_engine }
    }

    /// Check if an error is an AI service error that can be retried
    pub fn is_retryable_error(error: &crate::errors::StoryGenerationError) -> bool {
        match error {
            crate::errors::StoryGenerationError::AiServiceError { message } => {
                // Check for specific retryable error patterns
                message.contains("overloaded") || 
                message.contains("timeout") ||
                message.contains("429") || // Rate limit
                message.contains("502") || // Bad gateway
                message.contains("503") || // Service unavailable  
                message.contains("529")    // Overloaded
            },
            crate::errors::StoryGenerationError::Timeout { .. } => true,
            _ => false,
        }
    }

    /// Add a pending request to the session for later retry
    pub fn add_pending_request(
        &self,
        session: &mut QASession,
        request_type: RequestType,
        context: String,
        user_input: Option<String>,
        error_message: String,
    ) -> Uuid {
        let request_id = Uuid::new_v4();
        let pending_request = PendingRequest {
            id: request_id,
            request_type,
            context,
            user_input,
            error_message,
            retry_count: 0,
            created_at: Utc::now(),
            last_attempt: Utc::now(),
        };
        
        session.pending_requests.insert(request_id, pending_request);
        session.updated_at = Utc::now();
        request_id
    }

    /// Retry a pending request
    pub async fn retry_pending_request(
        &self,
        session: &mut QASession,
        request_id: Uuid,
    ) -> Result<RetryResult> {
        // First, extract the request type and context to avoid borrowing issues
        let (request_type, context, project_constraints) = {
            let pending_request = session.pending_requests.get_mut(&request_id)
                .ok_or_else(|| crate::errors::StoryGenerationError::ValidationError {
                    message: "Pending request not found".to_string(),
                })?;

            pending_request.retry_count += 1;
            pending_request.last_attempt = Utc::now();

            (pending_request.request_type.clone(), pending_request.context.clone(), session.project_constraints.clone())
        };

        let result = match &request_type {
            RequestType::InitialQuestions => {
                match self.generate_initial_questions(&context, project_constraints).await {
                    Ok(new_session) => {
                        // Merge questions from new session
                        for (id, question) in new_session.questions {
                            session.questions.insert(id, question);
                        }
                        RetryResult::Success
                    },
                    Err(e) if Self::is_retryable_error(&e) => {
                        if let Some(pending_request) = session.pending_requests.get_mut(&request_id) {
                            pending_request.error_message = format!("{}", e);
                        }
                        RetryResult::StillFailing(format!("{}", e))
                    },
                    Err(e) => RetryResult::PermanentFailure(format!("{}", e)),
                }
            },
            RequestType::ExploreQuestion { question_id } => {
                match self.explore_question(session, *question_id).await {
                    Ok(answers) => {
                        for answer in answers {
                            session.answers.insert(answer.id, answer);
                        }
                        RetryResult::Success
                    },
                    Err(e) if Self::is_retryable_error(&e) => {
                        if let Some(pending_request) = session.pending_requests.get_mut(&request_id) {
                            pending_request.error_message = format!("{}", e);
                        }
                        RetryResult::StillFailing(format!("{}", e))
                    },
                    Err(e) => RetryResult::PermanentFailure(format!("{}", e)),
                }
            },
            RequestType::DeepenAnswer { answer_id } => {
                if let Some(answer) = session.answers.get(answer_id).cloned() {
                    match self.deepen_answer(session, &answer).await {
                        Ok(deepened_answer) => {
                            session.answers.insert(deepened_answer.id, deepened_answer);
                            RetryResult::Success
                        },
                        Err(e) if Self::is_retryable_error(&e) => {
                            if let Some(pending_request) = session.pending_requests.get_mut(&request_id) {
                                pending_request.error_message = format!("{}", e);
                            }
                            RetryResult::StillFailing(format!("{}", e))
                        },
                        Err(e) => RetryResult::PermanentFailure(format!("{}", e)),
                    }
                } else {
                    RetryResult::PermanentFailure("Answer not found".to_string())
                }
            },
            RequestType::GenerateFollowUps { answer_id } => {
                if let Some(answer) = session.answers.get(answer_id).cloned() {
                    match self.generate_follow_up_questions(session, &answer).await {
                        Ok(follow_ups) => {
                            for question in follow_ups {
                                session.questions.insert(question.id, question);
                            }
                            RetryResult::Success
                        },
                        Err(e) if Self::is_retryable_error(&e) => {
                            if let Some(pending_request) = session.pending_requests.get_mut(&request_id) {
                                pending_request.error_message = format!("{}", e);
                            }
                            RetryResult::StillFailing(format!("{}", e))
                        },
                        Err(e) => RetryResult::PermanentFailure(format!("{}", e)),
                    }
                } else {
                    RetryResult::PermanentFailure("Answer not found".to_string())
                }
            },
            RequestType::CustomQuestion { question_text } => {
                // For custom questions, just create the question directly - no AI needed
                let question_id = Uuid::new_v4();
                let question = PlanningQuestion {
                    id: question_id,
                    question: question_text.clone(),
                    context: "User-generated question".to_string(),
                    question_type: PlanningQuestionType::Requirements,
                    priority: QuestionPriority::Medium,
                    dependencies: vec![],
                    metadata: PlanningQuestionMetadata {
                        generated_by_ai: false,
                        user_initiated: true,
                        complexity_score: Some(0.5),
                        estimated_time_to_answer: Some("15-30 minutes".to_string()),
                        related_topics: vec![],
                    },
                    created_at: Utc::now(),
                };
                session.questions.insert(question_id, question);
                RetryResult::Success
            },
        };

        if matches!(result, RetryResult::Success) {
            session.pending_requests.remove(&request_id);
        }

        session.updated_at = Utc::now();
        Ok(result)
    }

    /// Get all pending requests that can be retried
    pub fn get_retryable_requests<'a>(&self, session: &'a QASession) -> Vec<&'a PendingRequest> {
        session.pending_requests.values()
            .filter(|req| req.retry_count < 3) // Max 3 retries
            .collect()
    }

    /// Generate initial planning questions for a project
    pub async fn generate_initial_questions(
        &self,
        project_description: &str,
        constraints: ProjectConstraints,
    ) -> Result<QASession> {
        // Use the story engine to generate questions instead of a story
        let prompt = format!(
            "I'm starting a planning session for: '{}'
            
            Project context:
            - Timeline: {}
            - Team size: {}
            - Experience level: {:?}
            - Budget: {}
            
            As my planning partner, what are the most important questions we should answer together? 
            Just give me 4-6 critical questions that will help us understand and plan this project effectively.
            
            Please respond naturally - I'll extract the questions from whatever format feels right to you.",
            project_description,
            constraints.timeline.as_deref().unwrap_or("Not specified"),
            constraints.team_size.unwrap_or(0),
            constraints.experience_level,
            constraints.budget.as_deref().unwrap_or("Not specified")
        );

        // Generate a "story" that we'll parse as questions
        let temp_story = self.story_engine.generate_initial_story(&prompt, constraints.clone()).await?;
        
        // Parse the story into questions
        let session = self.parse_story_into_qa_session(temp_story, project_description.to_string(), constraints)?;
        
        Ok(session)
    }

    /// Explore a specific question to generate potential answers
    pub async fn explore_question(
        &self,
        session: &QASession,
        question_id: Uuid,
    ) -> Result<Vec<PlanningAnswer>> {
        let question = session.questions.get(&question_id)
            .ok_or_else(|| StoryGenerationError::ValidationError {
                message: "Question not found in session".to_string(),
            })?;

        let prompt = format!(
            "Let's explore this planning question together: '{}'
            
            Context: {}
            
            As my collaborative planning partner, what are different ways we could approach answering this question? 
            Give me 3-4 different perspectives or approaches we could take. Just respond naturally with your ideas.",
            question.question,
            question.context
        );

        // Use story engine to generate answer approaches
        let context = StoryContext {
            previous_choices: vec![],
            user_responses: vec![],
            current_constraints: session.project_constraints.clone(),
            session_metadata: crate::types::SessionMetadata {
                session_id: session.id,
                user_id: None,
                started_at: session.created_at,
                last_activity: Utc::now(),
            },
        };

        // Create a temporary node to represent our question
        let question_node = StoryNode {
            id: Uuid::new_v4(),
            node_type: NodeType::Decision,
            situation: prompt,
            choices: vec![],
            state: NodeState::Current,
            complexity_score: question.metadata.complexity_score,
            metadata: NodeMetadata {
                generated_by_ai: true,
                validation_status: ValidationStatus::Valid,
                user_annotations: vec![],
                technical_details: vec![],
            },
            created_at: Utc::now(),
        };

        let choices = self.story_engine.generate_choices(&question_node, &context).await?;
        
        // Convert choices into planning answers
        let answers = choices.into_iter().map(|choice| {
            PlanningAnswer {
                id: Uuid::new_v4(),
                question_id,
                answer: choice.description,
                exploration_notes: vec![],
                follow_up_questions: vec![],
                confidence: choice.feasibility_score.unwrap_or(0.7),
                metadata: PlanningAnswerMetadata {
                    answer_method: AnswerMethod::Collaborative,
                    validation_status: AnswerValidation::Tentative,
                    expert_review_needed: false,
                    implementation_readiness: 0.3, // Just starting to explore
                },
                created_at: Utc::now(),
            }
        }).collect();

        Ok(answers)
    }

    /// Deepen exploration of a specific answer
    pub async fn deepen_answer(
        &self,
        session: &QASession,
        answer: &PlanningAnswer,
    ) -> Result<PlanningAnswer> {
        let question = session.questions.get(&answer.question_id)
            .ok_or_else(|| StoryGenerationError::ValidationError {
                message: "Parent question not found".to_string(),
            })?;

        let prompt = format!(
            "We're deepening our exploration of this approach: '{}'
            
            Original question: {}
            Our current thinking: {}
            
            Let's dig deeper into this. What specific details, considerations, or implications should we explore?
            What questions does this approach raise? What would we need to figure out to make this work?
            
            Share your detailed thoughts on this approach.",
            answer.answer,
            question.question,
            answer.exploration_notes.join(", ")
        );

        // Create a choice representing this answer
        let answer_choice = Choice {
            id: answer.id,
            description: prompt,
            target_node_id: None,
            weight: 1.0,
            feasibility_score: Some(answer.confidence),
            consequences: vec![],
            metadata: ChoiceMetadata {
                confidence_score: Some(answer.confidence),
                risk_level: RiskLevel::Medium,
                time_estimate: None,
                dependencies: vec![],
            },
        };

        let context = StoryContext {
            previous_choices: vec![],
            user_responses: vec![],
            current_constraints: session.project_constraints.clone(),
            session_metadata: crate::types::SessionMetadata {
                session_id: session.id,
                user_id: None,
                started_at: session.created_at,
                last_activity: Utc::now(),
            },
        };

        // Expand this choice to get deeper insights
        let expanded_node = self.story_engine.expand_choice(&answer_choice, &context).await?;

        // Create an enriched answer
        let mut deepened_answer = answer.clone();
        deepened_answer.exploration_notes.push(expanded_node.situation);
        deepened_answer.metadata.validation_status = AnswerValidation::Discussed;
        deepened_answer.metadata.implementation_readiness += 0.2;

        Ok(deepened_answer)
    }

    /// Generate follow-up questions based on current answers
    pub async fn generate_follow_up_questions(
        &self,
        session: &QASession,
        answer: &PlanningAnswer,
    ) -> Result<Vec<PlanningQuestion>> {
        let original_question = session.questions.get(&answer.question_id)
            .ok_or_else(|| StoryGenerationError::ValidationError {
                message: "Original question not found".to_string(),
            })?;

        let prompt = format!(
            "Based on our exploration of '{}'
            
            Original question: {}
            Our current answer: {}
            Exploration notes: {}
            
            What new questions does this raise? What do we need to investigate next to build on this understanding? 
            
            Just give me 2-4 follow-up questions that would help us go deeper.",
            answer.answer,
            original_question.question,
            answer.answer,
            answer.exploration_notes.join("\n")
        );

        // Use story engine to generate follow-up questions
        let context = StoryContext {
            previous_choices: vec![],
            user_responses: vec![],
            current_constraints: session.project_constraints.clone(),
            session_metadata: crate::types::SessionMetadata {
                session_id: session.id,
                user_id: None,
                started_at: session.created_at,
                last_activity: Utc::now(),
            },
        };

        let question_node = StoryNode {
            id: Uuid::new_v4(),
            node_type: NodeType::Decision,
            situation: prompt,
            choices: vec![],
            state: NodeState::Current,
            complexity_score: Some(0.6),
            metadata: NodeMetadata {
                generated_by_ai: true,
                validation_status: ValidationStatus::Valid,
                user_annotations: vec![],
                technical_details: vec![],
            },
            created_at: Utc::now(),
        };

        let choices = self.story_engine.generate_choices(&question_node, &context).await?;

        // Convert choices into follow-up questions
        let follow_up_questions = choices.into_iter().map(|choice| {
            let question_type = self.infer_question_type(&choice.description);
            PlanningQuestion {
                id: Uuid::new_v4(),
                question: choice.description,
                context: format!("Follow-up from exploring: {}", answer.answer),
                question_type,
                priority: QuestionPriority::Medium,
                dependencies: vec![answer.question_id],
                metadata: PlanningQuestionMetadata {
                    generated_by_ai: true,
                    user_initiated: false,
                    complexity_score: choice.feasibility_score,
                    estimated_time_to_answer: Some("10-20 minutes".to_string()),
                    related_topics: vec![original_question.question.clone()],
                },
                created_at: Utc::now(),
            }
        }).collect();

        Ok(follow_up_questions)
    }

    /// Parse a story into a QA session (internal utility)
    fn parse_story_into_qa_session(
        &self,
        story: StoryGraph,
        project_description: String,
        constraints: ProjectConstraints,
    ) -> Result<QASession> {
        let mut questions = HashMap::new();
        
        // Extract questions from the root node
        if let Some(root_id) = story.root_node_id {
            if let Some(root_node) = story.nodes.get(&root_id) {
                // Parse the situation as context and try to extract questions from it
                let context = root_node.situation.clone();
                
                // First try to extract from choices
                if !root_node.choices.is_empty() {
                    for choice in &root_node.choices {
                        let question = PlanningQuestion {
                            id: choice.id,
                            question: self.clean_question_text(&choice.description),
                            context: context.clone(),
                            question_type: self.infer_question_type(&choice.description),
                            priority: QuestionPriority::High,
                            dependencies: vec![],
                            metadata: PlanningQuestionMetadata {
                                generated_by_ai: true,
                                user_initiated: false,
                                complexity_score: choice.feasibility_score,
                                estimated_time_to_answer: Some("15-30 minutes".to_string()),
                                related_topics: vec![project_description.clone()],
                            },
                            created_at: Utc::now(),
                        };
                        
                        questions.insert(choice.id, question);
                    }
                } else {
                    // If no choices, try to extract questions from the situation text
                    let extracted_questions = self.extract_questions_from_text(&context);
                    for question_text in extracted_questions {
                        let question_id = Uuid::new_v4();
                        let question = PlanningQuestion {
                            id: question_id,
                            question: question_text.clone(),
                            context: context.clone(),
                            question_type: self.infer_question_type(&question_text),
                            priority: QuestionPriority::High,
                            dependencies: vec![],
                            metadata: PlanningQuestionMetadata {
                                generated_by_ai: true,
                                user_initiated: false,
                                complexity_score: Some(0.7),
                                estimated_time_to_answer: Some("15-30 minutes".to_string()),
                                related_topics: vec![project_description.clone()],
                            },
                            created_at: Utc::now(),
                        };
                        
                        questions.insert(question_id, question);
                    }
                }
            }
        }

        let session = QASession {
            id: story.id,
            title: format!("Planning Session: {}", project_description),
            description: Some("Collaborative planning conversation".to_string()),
            questions,
            answers: HashMap::new(),
            current_question_id: None,
            project_constraints: constraints,
            session_metadata: QASessionMetadata {
                session_type: SessionType::InitialPlanning,
                completion_percentage: 0.0,
                critical_questions_answered: 0,
                total_critical_questions: 0,
                readiness_for_implementation: 0.1,
            },
            pending_requests: HashMap::new(),
            created_at: story.created_at,
            updated_at: Utc::now(),
        };

        Ok(session)
    }

    /// Extract questions from natural text
    fn extract_questions_from_text(&self, text: &str) -> Vec<String> {
        let mut questions = Vec::new();
        
        // Split by lines and look for question patterns
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            // Remove numbering (1., 2., etc.)
            let cleaned_line = if let Some(stripped) = line.strip_prefix(char::is_numeric) {
                if stripped.starts_with('.') || stripped.starts_with(')') {
                    stripped[1..].trim()
                } else {
                    line
                }
            } else {
                line
            };
            
            // Look for question indicators or just treat substantial lines as questions
            if cleaned_line.ends_with('?') 
                || cleaned_line.to_lowercase().contains("what")
                || cleaned_line.to_lowercase().contains("how")
                || cleaned_line.to_lowercase().contains("should we")
                || cleaned_line.to_lowercase().contains("could we")
                || cleaned_line.to_lowercase().contains("might we")
                || cleaned_line.len() > 20 // Substantial content
            {
                questions.push(self.clean_question_text(cleaned_line));
            }
        }
        
        // If we didn't find good questions, split on periods and try again
        if questions.is_empty() {
            for sentence in text.split('.') {
                let sentence = sentence.trim();
                if sentence.len() > 20 {
                    questions.push(self.clean_question_text(sentence));
                }
            }
        }
        
        questions
    }

    /// Clean up question text by removing type annotations and formatting
    fn clean_question_text(&self, text: &str) -> String {
        // Remove type annotations like "| Requirements", "| Implementation"
        let cleaned = if let Some(pipe_pos) = text.rfind('|') {
            text[..pipe_pos].trim()
        } else {
            text
        };
        
        // Remove leading numbering
        let cleaned = if let Some(dot_pos) = cleaned.find('.') {
            if cleaned[..dot_pos].chars().all(|c| c.is_numeric() || c.is_whitespace()) {
                cleaned[dot_pos + 1..].trim()
            } else {
                cleaned
            }
        } else {
            cleaned
        };
        
        // Ensure it ends with a question mark if it's actually a question
        let cleaned = cleaned.trim();
        if (cleaned.to_lowercase().starts_with("what") 
            || cleaned.to_lowercase().starts_with("how")
            || cleaned.to_lowercase().starts_with("should")
            || cleaned.to_lowercase().starts_with("could")
            || cleaned.to_lowercase().starts_with("might"))
            && !cleaned.ends_with('?') {
            format!("{}?", cleaned)
        } else {
            cleaned.to_string()
        }
    }

    /// Infer question type from question text (simple heuristic)
    fn infer_question_type(&self, question: &str) -> PlanningQuestionType {
        let q_lower = question.to_lowercase();
        
        if q_lower.contains("requirement") || q_lower.contains("need") || q_lower.contains("should") {
            PlanningQuestionType::Requirements
        } else if q_lower.contains("architecture") || q_lower.contains("structure") || q_lower.contains("design") {
            PlanningQuestionType::Architecture
        } else if q_lower.contains("depend") || q_lower.contains("order") || q_lower.contains("before") {
            PlanningQuestionType::Dependencies
        } else if q_lower.contains("first") || q_lower.contains("priority") || q_lower.contains("important") {
            PlanningQuestionType::Priorities
        } else if q_lower.contains("constraint") || q_lower.contains("limit") || q_lower.contains("budget") {
            PlanningQuestionType::Constraints
        } else if q_lower.contains("risk") || q_lower.contains("problem") || q_lower.contains("wrong") {
            PlanningQuestionType::Risks
        } else if q_lower.contains("build") || q_lower.contains("implement") || q_lower.contains("create") {
            PlanningQuestionType::Implementation
        } else if q_lower.contains("test") || q_lower.contains("validate") || q_lower.contains("verify") {
            PlanningQuestionType::Validation
        } else {
            PlanningQuestionType::Requirements // Default
        }
    }
}

/// Trait for QA session management
#[async_trait]
pub trait QASessionManager {
    async fn save_session(&self, session: &QASession) -> Result<()>;
    async fn load_session(&self, session_id: Uuid) -> Result<QASession>;
    async fn list_sessions(&self) -> Result<Vec<QASession>>;
    async fn delete_session(&self, session_id: Uuid) -> Result<()>;
}