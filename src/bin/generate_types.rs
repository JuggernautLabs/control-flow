use story_generation_engine::types::*;
use ts_rs::TS;

fn main() {
    // Generate TypeScript types for all exported structs
    let export_path = std::path::Path::new("./typescript-types");
    
    // Create the directory if it doesn't exist
    if !export_path.exists() {
        std::fs::create_dir_all(export_path).expect("Failed to create typescript-types directory");
    }

    // Export all types
    StoryGraph::export().expect("Failed to export StoryGraph");
    StoryNode::export().expect("Failed to export StoryNode");
    Choice::export().expect("Failed to export Choice");
    StoryEdge::export().expect("Failed to export StoryEdge");
    NodeType::export().expect("Failed to export NodeType");
    NodeState::export().expect("Failed to export NodeState");
    StoryMetadata::export().expect("Failed to export StoryMetadata");
    NodeMetadata::export().expect("Failed to export NodeMetadata");
    ChoiceMetadata::export().expect("Failed to export ChoiceMetadata");
    EdgeMetadata::export().expect("Failed to export EdgeMetadata");
    ProjectConstraints::export().expect("Failed to export ProjectConstraints");
    StoryAnalytics::export().expect("Failed to export StoryAnalytics");
    TechnicalDetail::export().expect("Failed to export TechnicalDetail");
    ValidationStatus::export().expect("Failed to export ValidationStatus");
    RiskLevel::export().expect("Failed to export RiskLevel");
    ExperienceLevel::export().expect("Failed to export ExperienceLevel");
    ImportanceLevel::export().expect("Failed to export ImportanceLevel");
    StoryContext::export().expect("Failed to export StoryContext");
    UserResponse::export().expect("Failed to export UserResponse");
    SessionMetadata::export().expect("Failed to export SessionMetadata");
    CoherenceReport::export().expect("Failed to export CoherenceReport");
    CoherenceIssue::export().expect("Failed to export CoherenceIssue");
    CoherenceIssueType::export().expect("Failed to export CoherenceIssueType");
    IssueSeverity::export().expect("Failed to export IssueSeverity");
    Question::export().expect("Failed to export Question");
    QuestionOption::export().expect("Failed to export QuestionOption");
    ValidationRules::export().expect("Failed to export ValidationRules");
    QuestionContext::export().expect("Failed to export QuestionContext");
    QuestionType::export().expect("Failed to export QuestionType");
    UncertaintyContext::export().expect("Failed to export UncertaintyContext");

    println!("TypeScript types generated successfully in ./typescript-types/");
}