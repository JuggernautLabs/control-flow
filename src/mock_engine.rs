use async_trait::async_trait;
use uuid::Uuid;
use chrono::Utc;
use rand::Rng;
use std::collections::HashMap;

use crate::types::*;
use crate::engine::*;
use crate::errors::{StoryGenerationError, Result};

pub struct MockStoryEngine {
    pub predefined_stories: HashMap<String, PredefinedStory>,
}

#[derive(Debug, Clone)]
pub struct PredefinedStory {
    pub title: String,
    pub description: String,
    pub starting_situation: String,
    pub story_template: StoryTemplate,
}

#[derive(Debug, Clone)]
pub struct StoryTemplate {
    pub nodes: Vec<NodeTemplate>,
    pub connections: Vec<(usize, usize, String)>, // (from_index, to_index, choice_description)
}

#[derive(Debug, Clone)]
pub struct NodeTemplate {
    pub situation: String,
    pub node_type: NodeType,
    pub choices: Vec<String>,
}

impl MockStoryEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            predefined_stories: HashMap::new(),
        };
        engine.load_default_stories();
        engine
    }

    fn load_default_stories(&mut self) {
        // Adventure Story 1: The Mysterious Cave
        let cave_story = PredefinedStory {
            title: "The Mysterious Cave".to_string(),
            description: "A classic adventure where you explore a mysterious cave".to_string(),
            starting_situation: "You stand before the entrance of a dark, mysterious cave. Strange sounds echo from within, and you can see flickering light deep inside.".to_string(),
            story_template: StoryTemplate {
                nodes: vec![
                    NodeTemplate {
                        situation: "You stand before the entrance of a dark, mysterious cave. Strange sounds echo from within, and you can see flickering light deep inside.".to_string(),
                        node_type: NodeType::Start,
                        choices: vec![
                            "Enter the cave boldly".to_string(),
                            "Look for a torch first".to_string(),
                            "Call out to see if anyone responds".to_string(),
                        ],
                    },
                    NodeTemplate {
                        situation: "You stride confidently into the cave. The darkness swallows you, but your eyes gradually adjust. You see two tunnels ahead.".to_string(),
                        node_type: NodeType::Decision,
                        choices: vec![
                            "Take the left tunnel (you hear water dripping)".to_string(),
                            "Take the right tunnel (you feel a warm breeze)".to_string(),
                            "Go back to the entrance".to_string(),
                        ],
                    },
                    NodeTemplate {
                        situation: "You find some dry branches and create a makeshift torch. The cave is now illuminated, revealing ancient drawings on the walls.".to_string(),
                        node_type: NodeType::Action,
                        choices: vec![
                            "Examine the drawings closely".to_string(),
                            "Continue deeper into the cave".to_string(),
                            "Try to understand what the drawings mean".to_string(),
                        ],
                    },
                    NodeTemplate {
                        situation: "Your voice echoes through the cave. After a moment, you hear a faint response: 'Help... trapped...'".to_string(),
                        node_type: NodeType::Decision,
                        choices: vec![
                            "Rush in to help immediately".to_string(),
                            "Ask who they are first".to_string(),
                            "Go get help from the village".to_string(),
                        ],
                    },
                    NodeTemplate {
                        situation: "The left tunnel leads to an underground lake. The water is crystal clear, and you can see something glittering at the bottom.".to_string(),
                        node_type: NodeType::Outcome,
                        choices: vec![
                            "Dive down to retrieve the glittering object".to_string(),
                            "Look for another way around the lake".to_string(),
                            "Return to explore the other tunnel".to_string(),
                        ],
                    },
                    NodeTemplate {
                        situation: "The right tunnel grows warmer as you walk. You discover it leads to a chamber with a dragon sleeping on a pile of treasure!".to_string(),
                        node_type: NodeType::Outcome,
                        choices: vec![
                            "Try to sneak past and grab some treasure".to_string(),
                            "Slowly back away".to_string(),
                            "Try to wake the dragon and talk to it".to_string(),
                        ],
                    },
                    NodeTemplate {
                        situation: "The ancient drawings tell the story of a lost civilization that once lived in these caves. You realize this could be a major archaeological discovery!".to_string(),
                        node_type: NodeType::End,
                        choices: vec!["Return to the surface to report your discovery".to_string()],
                    },
                ],
                connections: vec![
                    (0, 1, "Enter the cave boldly".to_string()),
                    (0, 2, "Look for a torch first".to_string()),
                    (0, 3, "Call out to see if anyone responds".to_string()),
                    (1, 4, "Take the left tunnel (you hear water dripping)".to_string()),
                    (1, 5, "Take the right tunnel (you feel a warm breeze)".to_string()),
                    (2, 6, "Examine the drawings closely".to_string()),
                ],
            },
        };

        // Adventure Story 2: The Space Station
        let space_story = PredefinedStory {
            title: "Emergency on Station Alpha".to_string(),
            description: "A sci-fi adventure aboard a space station in crisis".to_string(),
            starting_situation: "Alarms are blaring throughout Space Station Alpha. The lights flicker ominously as you float through the corridor in zero gravity.".to_string(),
            story_template: StoryTemplate {
                nodes: vec![
                    NodeTemplate {
                        situation: "Alarms are blaring throughout Space Station Alpha. The lights flicker ominously as you float through the corridor in zero gravity.".to_string(),
                        node_type: NodeType::Start,
                        choices: vec![
                            "Head to the control room".to_string(),
                            "Check the life support systems".to_string(),
                            "Look for other crew members".to_string(),
                        ],
                    },
                    NodeTemplate {
                        situation: "The control room is in chaos. Multiple screens show system failures, and you can see Earth through the viewport, looking surprisingly close.".to_string(),
                        node_type: NodeType::Decision,
                        choices: vec![
                            "Try to stabilize the failing systems".to_string(),
                            "Send a distress signal to Earth".to_string(),
                            "Check the navigation computer".to_string(),
                        ],
                    },
                    NodeTemplate {
                        situation: "The life support bay is filling with smoke. You can see that one of the oxygen recyclers has malfunctioned and is sparking dangerously.".to_string(),
                        node_type: NodeType::Action,
                        choices: vec![
                            "Try to repair the recycler".to_string(),
                            "Shut down the malfunctioning unit".to_string(),
                            "Evacuate the area and seal it off".to_string(),
                        ],
                    },
                ],
                connections: vec![
                    (0, 1, "Head to the control room".to_string()),
                    (0, 2, "Check the life support systems".to_string()),
                ],
            },
        };

        self.predefined_stories.insert("cave".to_string(), cave_story);
        self.predefined_stories.insert("space".to_string(), space_story);
    }

    fn create_story_from_template(&self, template: &PredefinedStory) -> StoryGraph {
        let mut story = StoryGraph::default();
        story.title = template.title.clone();
        story.description = Some(template.description.clone());

        let mut node_ids = Vec::new();

        // Create nodes
        for (index, node_template) in template.story_template.nodes.iter().enumerate() {
            let node_id = Uuid::new_v4();
            node_ids.push(node_id);

            let choices: Vec<Choice> = node_template.choices.iter().map(|choice_desc| {
                Choice {
                    id: Uuid::new_v4(),
                    description: choice_desc.clone(),
                    target_node_id: None, // Will be set when we create edges
                    weight: 1.0,
                    feasibility_score: Some(0.8),
                    consequences: vec![],
                    metadata: ChoiceMetadata {
                        confidence_score: Some(0.9),
                        risk_level: RiskLevel::Low,
                        time_estimate: Some("5 minutes".to_string()),
                        dependencies: vec![],
                    },
                }
            }).collect();

            let node = StoryNode {
                id: node_id,
                node_type: node_template.node_type.clone(),
                situation: node_template.situation.clone(),
                choices,
                state: if index == 0 { NodeState::Current } else { NodeState::Unvisited },
                complexity_score: Some(0.5),
                metadata: NodeMetadata {
                    generated_by_ai: false,
                    validation_status: ValidationStatus::Valid,
                    user_annotations: vec![],
                    technical_details: vec![],
                },
                created_at: Utc::now(),
            };

            story.nodes.insert(node_id, node);
        }

        // Set root node
        if !node_ids.is_empty() {
            story.root_node_id = Some(node_ids[0]);
            story.current_node_id = Some(node_ids[0]);
        }

        // Create edges
        for (from_index, to_index, choice_desc) in &template.story_template.connections {
            if let (Some(&from_id), Some(&to_id)) = (node_ids.get(*from_index), node_ids.get(*to_index)) {
                let edge = StoryEdge {
                    id: Uuid::new_v4(),
                    from_node_id: from_id,
                    to_node_id: to_id,
                    choice_id: Uuid::new_v4(), // This should match a choice ID, but for simplicity...
                    traversal_count: 0,
                    metadata: EdgeMetadata {
                        decision_timestamp: None,
                        user_feedback: None,
                        success_probability: Some(0.8),
                    },
                };

                story.edges.push(edge);

                // Update the choice's target_node_id
                if let Some(from_node) = story.nodes.get_mut(&from_id) {
                    for choice in &mut from_node.choices {
                        if choice.description == *choice_desc {
                            choice.target_node_id = Some(to_id);
                            break;
                        }
                    }
                }
            }
        }

        story
    }
}

#[async_trait]
impl StoryGenerationEngine for MockStoryEngine {
    async fn generate_initial_story(
        &self,
        prompt: &str,
        _constraints: ProjectConstraints,
    ) -> Result<StoryGraph> {
        // Simple keyword matching to select a story
        let story_key = if prompt.to_lowercase().contains("cave") || prompt.to_lowercase().contains("adventure") {
            "cave"
        } else if prompt.to_lowercase().contains("space") || prompt.to_lowercase().contains("station") {
            "space"
        } else {
            // Default to cave story
            "cave"
        };

        if let Some(template) = self.predefined_stories.get(story_key) {
            Ok(self.create_story_from_template(template))
        } else {
            Err(StoryGenerationError::InvalidInput {
                message: format!("No story template found for prompt: {}", prompt),
            })
        }
    }

    async fn generate_choices(
        &self,
        current_node: &StoryNode,
        _context: &StoryContext,
    ) -> Result<Vec<Choice>> {
        // For the mock engine, we just return the existing choices
        // In a real implementation, this might generate new choices based on context
        Ok(current_node.choices.clone())
    }

    async fn expand_choice(
        &self,
        choice: &Choice,
        _context: &StoryContext,
    ) -> Result<StoryNode> {
        // For the mock engine, we generate a simple continuation
        let mut rng = rand::thread_rng();
        let situation_endings = vec![
            "You find yourself in a new situation that requires careful consideration.",
            "The path ahead is unclear, but you must make a decision.",
            "Something unexpected happens, changing everything.",
            "You discover something that wasn't there before.",
        ];

        let situation = format!("After choosing to '{}', {}", 
            choice.description, 
            situation_endings[rng.gen_range(0..situation_endings.len())]
        );

        let new_choices = vec![
            Choice {
                id: Uuid::new_v4(),
                description: "Continue forward".to_string(),
                target_node_id: None,
                weight: 1.0,
                feasibility_score: Some(0.7),
                consequences: vec![],
                metadata: ChoiceMetadata {
                    confidence_score: Some(0.8),
                    risk_level: RiskLevel::Medium,
                    time_estimate: Some("10 minutes".to_string()),
                    dependencies: vec![],
                },
            },
            Choice {
                id: Uuid::new_v4(),
                description: "Look around carefully".to_string(),
                target_node_id: None,
                weight: 1.0,
                feasibility_score: Some(0.9),
                consequences: vec![],
                metadata: ChoiceMetadata {
                    confidence_score: Some(0.9),
                    risk_level: RiskLevel::Low,
                    time_estimate: Some("5 minutes".to_string()),
                    dependencies: vec![],
                },
            },
            Choice {
                id: Uuid::new_v4(),
                description: "Go back".to_string(),
                target_node_id: None,
                weight: 1.0,
                feasibility_score: Some(0.8),
                consequences: vec![],
                metadata: ChoiceMetadata {
                    confidence_score: Some(0.7),
                    risk_level: RiskLevel::Low,
                    time_estimate: Some("5 minutes".to_string()),
                    dependencies: vec![],
                },
            },
        ];

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
        _proposed_node: &StoryNode,
        _existing_graph: &StoryGraph,
    ) -> Result<CoherenceReport> {
        // Mock validation - always passes
        Ok(CoherenceReport {
            is_coherent: true,
            confidence_score: 0.9,
            issues: vec![],
            suggestions: vec!["Story flow looks good!".to_string()],
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
                    id: "yes".to_string(),
                    label: "Yes, continue".to_string(),
                    value: serde_json::Value::Bool(true),
                    description: Some("Proceed with the current approach".to_string()),
                },
                QuestionOption {
                    id: "no".to_string(),
                    label: "No, try something else".to_string(),
                    value: serde_json::Value::Bool(false),
                    description: Some("Take a different approach".to_string()),
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