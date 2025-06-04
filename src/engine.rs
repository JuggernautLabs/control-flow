use async_trait::async_trait;
use uuid::Uuid;
use crate::types::*;
use crate::errors::{Result};

#[async_trait]
pub trait StoryGenerationEngine {
    async fn generate_initial_story(
        &self,
        prompt: &str,
        constraints: ProjectConstraints,
    ) -> Result<StoryGraph>;

    async fn generate_choices(
        &self,
        current_node: &StoryNode,
        context: &StoryContext,
    ) -> Result<Vec<Choice>>;

    async fn expand_choice(
        &self,
        choice: &Choice,
        context: &StoryContext,
    ) -> Result<StoryNode>;

    async fn validate_coherence(
        &self,
        proposed_node: &StoryNode,
        existing_graph: &StoryGraph,
    ) -> Result<CoherenceReport>;

    async fn ask_clarification_question(
        &self,
        uncertainty: &UncertaintyContext,
    ) -> Result<Question>;
}

#[async_trait]
pub trait StoryGraphManager {
    async fn create_story(&self, title: String, description: Option<String>) -> Result<StoryGraph>;
    
    async fn add_node(&self, graph: &mut StoryGraph, node: StoryNode) -> Result<()>;
    
    async fn add_edge(&self, graph: &mut StoryGraph, edge: StoryEdge) -> Result<()>;
    
    async fn traverse_to_node(&self, graph: &mut StoryGraph, node_id: Uuid) -> Result<()>;
    
    async fn calculate_analytics(&self, graph: &StoryGraph) -> Result<StoryAnalytics>;
    
    async fn validate_graph_integrity(&self, graph: &StoryGraph) -> Result<Vec<CoherenceIssue>>;
}

#[async_trait]
pub trait QuestionEngine {
    async fn generate_context_questions(
        &self,
        story_state: &StoryGraph,
        uncertainty_threshold: f32,
    ) -> Result<Vec<Question>>;

    async fn process_answer(
        &self,
        question: &Question,
        answer: &UserResponse,
    ) -> Result<StoryContext>;

    async fn assess_uncertainty(
        &self,
        story_state: &StoryGraph,
    ) -> Result<UncertaintyContext>;
}

#[derive(Debug, Clone)]
pub struct StoryGenerationConfig {
    pub max_choices_per_node: u32,
    pub max_graph_depth: u32,
    pub complexity_threshold: f32,
    pub coherence_threshold: f32,
    pub ai_model_config: AiModelConfig,
}

#[derive(Debug, Clone)]
pub struct AiModelConfig {
    pub model_name: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout_seconds: u64,
}

impl Default for StoryGenerationConfig {
    fn default() -> Self {
        Self {
            max_choices_per_node: 6,
            max_graph_depth: 10,
            complexity_threshold: 0.7,
            coherence_threshold: 0.8,
            ai_model_config: AiModelConfig::default(),
        }
    }
}

impl Default for AiModelConfig {
    fn default() -> Self {
        Self {
            model_name: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            timeout_seconds: 30,
        }
    }
}

pub struct StoryGraphUtils;

impl StoryGraphUtils {
    pub fn find_path_to_node(graph: &StoryGraph, target_node_id: Uuid) -> Option<Vec<Uuid>> {
        if let Some(root_id) = graph.root_node_id {
            Self::dfs_path(graph, root_id, target_node_id, &mut Vec::new())
        } else {
            None
        }
    }

    fn dfs_path(
        graph: &StoryGraph,
        current_id: Uuid,
        target_id: Uuid,
        path: &mut Vec<Uuid>,
    ) -> Option<Vec<Uuid>> {
        path.push(current_id);

        if current_id == target_id {
            return Some(path.clone());
        }

        for edge in &graph.edges {
            if edge.from_node_id == current_id {
                if let Some(result) = Self::dfs_path(graph, edge.to_node_id, target_id, path) {
                    return Some(result);
                }
            }
        }

        path.pop();
        None
    }

    pub fn get_leaf_nodes(graph: &StoryGraph) -> Vec<Uuid> {
        let mut leaf_nodes = Vec::new();
        let mut has_outgoing = std::collections::HashSet::new();

        for edge in &graph.edges {
            has_outgoing.insert(edge.from_node_id);
        }

        for node_id in graph.nodes.keys() {
            if !has_outgoing.contains(node_id) {
                leaf_nodes.push(*node_id);
            }
        }

        leaf_nodes
    }

    pub fn calculate_node_depth(graph: &StoryGraph, node_id: Uuid) -> Option<u32> {
        if let Some(_root_id) = graph.root_node_id {
            if let Some(path) = Self::find_path_to_node(graph, node_id) {
                return Some((path.len() - 1) as u32);
            }
        }
        None
    }

    pub fn get_node_children(graph: &StoryGraph, node_id: Uuid) -> Vec<Uuid> {
        graph
            .edges
            .iter()
            .filter(|edge| edge.from_node_id == node_id)
            .map(|edge| edge.to_node_id)
            .collect()
    }

    pub fn get_node_parents(graph: &StoryGraph, node_id: Uuid) -> Vec<Uuid> {
        graph
            .edges
            .iter()
            .filter(|edge| edge.to_node_id == node_id)
            .map(|edge| edge.from_node_id)
            .collect()
    }
}