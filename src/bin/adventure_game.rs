use std::io::{self, Write};
use std::env;
use story_generation_engine::{
    StoryGenerationEngine, MockStoryEngine, OpenAIStoryEngine, ClaudeStoryEngine,
    StoryGraph, StoryNode, Choice, StoryContext, ProjectConstraints, 
    ExperienceLevel, NodeState, StoryEdge, EdgeMetadata
};
use uuid::Uuid;
use chrono::Utc;

struct AdventureGame {
    story: StoryGraph,
    engine: Box<dyn StoryGenerationEngine + Send + Sync>,
    history: Vec<String>,
}

impl AdventureGame {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load environment variables
        dotenvy::dotenv().ok();
        
        println!("🎮 Welcome to the Adventure Game Generator!");
        
        // Check for default engine in environment
        let default_engine = env::var("DEFAULT_ENGINE").unwrap_or_else(|_| "mock".to_string());
        
        println!("Choose your story engine:");
        println!("1. Mock Engine (predefined stories)");
        println!("2. OpenAI Engine (requires OpenAI API key)");
        println!("3. Claude Engine (requires Anthropic API key) - ⭐ Recommended");
        println!("4. Auto-detect from environment (DEFAULT_ENGINE={})", default_engine);
        print!("Enter choice (1-4): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        let engine: Box<dyn StoryGenerationEngine + Send + Sync> = match choice {
            "2" => {
                println!("🔄 Initializing OpenAI Engine...");
                match OpenAIStoryEngine::new() {
                    Ok(ai_engine) => {
                        println!("✅ OpenAI Engine initialized!");
                        Box::new(ai_engine)
                    },
                    Err(e) => {
                        println!("❌ Failed to initialize OpenAI Engine: {}", e);
                        println!("📝 Falling back to Mock Engine...");
                        Box::new(MockStoryEngine::new())
                    }
                }
            },
            "3" => {
                println!("🔄 Initializing Claude Engine...");
                match ClaudeStoryEngine::new() {
                    Ok(claude_engine) => {
                        println!("✅ Claude Engine initialized with model: {}", 
                                env::var("CLAUDE_MODEL").unwrap_or_else(|_| "claude-3-5-haiku-20241022".to_string()));
                        Box::new(claude_engine)
                    },
                    Err(e) => {
                        println!("❌ Failed to initialize Claude Engine: {}", e);
                        println!("💡 Make sure ANTHROPIC_API_KEY is set in your .env file");
                        println!("📝 Falling back to Mock Engine...");
                        Box::new(MockStoryEngine::new())
                    }
                }
            },
            "4" => {
                println!("🔍 Auto-detecting engine from environment...");
                Self::create_engine_from_env()
            },
            _ => {
                println!("📚 Using Mock Engine with predefined stories");
                Box::new(MockStoryEngine::new())
            }
        };

        Ok(Self {
            story: StoryGraph::default(),
            engine,
            history: Vec::new(),
        })
    }

    fn create_engine_from_env() -> Box<dyn StoryGenerationEngine + Send + Sync> {
        let default_engine = env::var("DEFAULT_ENGINE").unwrap_or_else(|_| "mock".to_string());
        
        match default_engine.to_lowercase().as_str() {
            "claude" => {
                println!("🔄 Environment specified Claude engine...");
                match ClaudeStoryEngine::new() {
                    Ok(engine) => {
                        println!("✅ Claude Engine initialized from environment!");
                        println!("📊 Model: {}", env::var("CLAUDE_MODEL").unwrap_or_else(|_| "claude-3-5-haiku-20241022".to_string()));
                        println!("🌡️  Temperature: {}", env::var("TEMPERATURE").unwrap_or_else(|_| "0.7".to_string()));
                        println!("🎯 Max Tokens: {}", env::var("MAX_TOKENS").unwrap_or_else(|_| "2048".to_string()));
                        Box::new(engine)
                    },
                    Err(e) => {
                        println!("❌ Failed to initialize Claude from environment: {}", e);
                        println!("📝 Falling back to Mock Engine...");
                        Box::new(MockStoryEngine::new())
                    }
                }
            },
            "openai" => {
                println!("🔄 Environment specified OpenAI engine...");
                match OpenAIStoryEngine::new() {
                    Ok(engine) => {
                        println!("✅ OpenAI Engine initialized from environment!");
                        Box::new(engine)
                    },
                    Err(e) => {
                        println!("❌ Failed to initialize OpenAI from environment: {}", e);
                        println!("📝 Falling back to Mock Engine...");
                        Box::new(MockStoryEngine::new())
                    }
                }
            },
            _ => {
                println!("📚 Using Mock Engine (default or specified in environment)");
                Box::new(MockStoryEngine::new())
            }
        }
    }

    async fn start_new_adventure(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🌟 Let's create your planning adventure!");
        println!("💡 This system helps you break down complex goals into manageable steps.");
        println!("Examples:");
        println!("  • 'Build a todo app with real-time collaboration'");
        println!("  • 'Plan a marketing campaign for a startup'");
        println!("  • 'Design a home renovation project'");
        println!("  • 'Organize a company retreat'");
        print!("\nWhat goal would you like to plan? ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let prompt = input.trim();

        if prompt.is_empty() {
            println!("Using default planning scenario...");
        }

        println!("\n🎯 Choose your experience level:");
        println!("1. Beginner (simpler planning approaches)");
        println!("2. Intermediate (balanced complexity)");
        println!("3. Advanced (detailed strategic planning)");
        println!("4. Expert (comprehensive enterprise-level planning)");
        print!("Enter choice (1-4): ");
        io::stdout().flush()?;

        let mut experience_input = String::new();
        io::stdin().read_line(&mut experience_input)?;
        
        let experience_level = match experience_input.trim() {
            "1" => ExperienceLevel::Beginner,
            "2" => ExperienceLevel::Intermediate,
            "3" => ExperienceLevel::Advanced,
            "4" => ExperienceLevel::Expert,
            _ => {
                println!("Invalid choice, using Intermediate");
                ExperienceLevel::Intermediate
            }
        };

        // Gather additional project context
        println!("\n📋 Project Details (optional - press Enter to skip):");
        
        print!("Timeline (e.g., '3 months', '1 year'): ");
        io::stdout().flush()?;
        let mut timeline_input = String::new();
        io::stdin().read_line(&mut timeline_input)?;
        let timeline = if timeline_input.trim().is_empty() { 
            None 
        } else { 
            Some(timeline_input.trim().to_string()) 
        };

        print!("Team size (number): ");
        io::stdout().flush()?;
        let mut team_input = String::new();
        io::stdin().read_line(&mut team_input)?;
        let team_size = team_input.trim().parse::<u32>().ok();

        print!("Budget (e.g., '$10k', 'unlimited'): ");
        io::stdout().flush()?;
        let mut budget_input = String::new();
        io::stdin().read_line(&mut budget_input)?;
        let budget = if budget_input.trim().is_empty() { 
            None 
        } else { 
            Some(budget_input.trim().to_string()) 
        };

        let constraints = ProjectConstraints {
            timeline,
            team_size,
            experience_level,
            budget,
            technical_constraints: vec![],
            business_constraints: vec![],
        };

        println!("\n🔮 Generating your planning journey...");
        self.story = self.engine.generate_initial_story(prompt, constraints).await?;
        println!("✅ Planning journey created: {}", self.story.title);

        Ok(())
    }

    async fn play_game(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // Get current node
            let current_node_id = match self.story.current_node_id {
                Some(id) => id,
                None => {
                    println!("\n🎉 Adventure complete! Thanks for playing!");
                    break;
                }
            };

            let current_node = match self.story.nodes.get(&current_node_id) {
                Some(node) => node.clone(),
                None => {
                    println!("❌ Error: Current node not found!");
                    break;
                }
            };

            // Display current situation
            self.display_current_situation(&current_node);

            // Handle choices
            if current_node.choices.is_empty() {
                println!("\n🎯 You've reached a planning milestone!");
                println!("This planning path is complete. Would you like to:");
                println!("1. Start a new planning journey");
                println!("2. Quit");
                print!("Enter choice (1-2): ");
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                
                match input.trim() {
                    "1" => {
                        self.start_new_adventure().await?;
                        continue;
                    },
                    _ => break,
                }
            }

            let choice_result = self.handle_player_choice(&current_node).await?;
            match choice_result {
                PlayerAction::MadeChoice(choice) => {
                    self.process_choice(&current_node, &choice).await?;
                },
                PlayerAction::Quit => break,
                PlayerAction::ShowHistory => {
                    self.show_history();
                    continue;
                },
                PlayerAction::NewAdventure => {
                    self.start_new_adventure().await?;
                    continue;
                },
            }
        }

        println!("\n👋 Thanks for using the planning system! Your journey ends here.");
        Ok(())
    }

    fn display_current_situation(&self, node: &StoryNode) {
        let separator = "=".repeat(60);
        println!("\n{}", separator);
        println!("📍 Current Planning Situation:");
        println!("{}", node.situation);
        if let Some(complexity) = node.complexity_score {
            println!("\n🎯 Complexity Level: {:.1}/1.0", complexity);
        }
        println!("{}", separator);
    }

    async fn handle_player_choice(&self, node: &StoryNode) -> Result<PlayerAction, Box<dyn std::error::Error>> {
        loop {
            println!("\n🎯 What do you choose?");
            
            for (i, choice) in node.choices.iter().enumerate() {
                println!("{}. {}", i + 1, choice.description);
            }
            
            println!("\n📋 Other options:");
            println!("h. Show planning history");
            println!("n. Start new planning journey");
            println!("q. Quit");
            
            print!("\nEnter your choice: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();

            match input.as_str() {
                "h" | "history" => return Ok(PlayerAction::ShowHistory),
                "n" | "new" => return Ok(PlayerAction::NewAdventure),
                "q" | "quit" => return Ok(PlayerAction::Quit),
                _ => {
                    if let Ok(choice_num) = input.parse::<usize>() {
                        if choice_num > 0 && choice_num <= node.choices.len() {
                            let choice = node.choices[choice_num - 1].clone();
                            return Ok(PlayerAction::MadeChoice(choice));
                        } else {
                            println!("❌ Invalid choice number! Please choose 1-{}", node.choices.len());
                            continue;
                        }
                    } else {
                        println!("❌ Invalid input! Please enter a number, 'h', 'n', or 'q'");
                        continue;
                    }
                }
            }
        }
    }

    async fn process_choice(&mut self, current_node: &StoryNode, choice: &Choice) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n⚡ You decided: {}", choice.description);
        self.history.push(format!("From '{}' → Decided: '{}'", 
            current_node.situation.chars().take(50).collect::<String>() + "...", 
            choice.description
        ));

        // Check if this choice leads to an existing node
        if let Some(target_node_id) = choice.target_node_id {
            if self.story.nodes.contains_key(&target_node_id) {
                self.story.current_node_id = Some(target_node_id);
                self.update_node_state(target_node_id);
                return Ok(());
            }
        }

        // Generate new content using the engine
        println!("🔄 Analyzing the consequences of your decision...");
        
        let context = StoryContext {
            previous_choices: self.history.iter().map(|_| Uuid::new_v4()).collect(),
            user_responses: vec![],
            current_constraints: self.story.metadata.project_constraints.clone(),
            session_metadata: story_generation_engine::SessionMetadata {
                session_id: Uuid::new_v4(),
                user_id: None,
                started_at: Utc::now(),
                last_activity: Utc::now(),
            },
        };

        let new_node = self.engine.expand_choice(choice, &context).await?;
        let new_node_id = new_node.id;

        // Add edge from current node to new node
        let edge = StoryEdge {
            id: Uuid::new_v4(),
            from_node_id: current_node.id,
            to_node_id: new_node_id,
            choice_id: choice.id,
            traversal_count: 1,
            metadata: EdgeMetadata {
                decision_timestamp: Some(Utc::now()),
                user_feedback: None,
                success_probability: Some(0.8),
            },
        };

        // Update story
        self.story.nodes.insert(new_node_id, new_node);
        self.story.edges.push(edge);
        self.story.current_node_id = Some(new_node_id);
        self.update_node_state(new_node_id);

        Ok(())
    }

    fn update_node_state(&mut self, node_id: Uuid) {
        for (id, node) in &mut self.story.nodes {
            if *id == node_id {
                node.state = NodeState::Current;
            } else {
                node.state = NodeState::Visited;
            }
        }
    }

    fn show_history(&self) {
        let separator = "=".repeat(50);
        println!("\n📜 Your Planning Journey History:");
        println!("{}", separator);
        if self.history.is_empty() {
            println!("No planning decisions made yet.");
        } else {
            for (i, entry) in self.history.iter().enumerate() {
                println!("{}. {}", i + 1, entry);
            }
        }
        println!("{}", separator);
    }
}

enum PlayerAction {
    MadeChoice(Choice),
    Quit,
    ShowHistory,
    NewAdventure,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut game = AdventureGame::new().await?;
    game.start_new_adventure().await?;
    game.play_game().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_engine_basic_story() {
        let engine = MockStoryEngine::new();
        let constraints = ProjectConstraints {
            timeline: None,
            team_size: None,
            experience_level: ExperienceLevel::Beginner,
            budget: None,
            technical_constraints: vec![],
            business_constraints: vec![],
        };

        let result = engine.generate_initial_story("cave adventure", constraints).await;
        assert!(result.is_ok());
        
        let story = result.unwrap();
        assert!(!story.title.is_empty());
        assert!(story.root_node_id.is_some());
        assert!(!story.nodes.is_empty());
    }

    #[tokio::test]
    async fn test_choice_expansion() {
        let engine = MockStoryEngine::new();
        let choice = Choice {
            id: Uuid::new_v4(),
            description: "Enter the cave".to_string(),
            target_node_id: None,
            weight: 1.0,
            feasibility_score: Some(0.8),
            consequences: vec![],
            metadata: story_generation_engine::ChoiceMetadata {
                confidence_score: Some(0.9),
                risk_level: story_generation_engine::RiskLevel::Low,
                time_estimate: Some("5 minutes".to_string()),
                dependencies: vec![],
            },
        };

        let context = StoryContext {
            previous_choices: vec![],
            user_responses: vec![],
            current_constraints: ProjectConstraints::default(),
            session_metadata: story_generation_engine::SessionMetadata {
                session_id: Uuid::new_v4(),
                user_id: None,
                started_at: Utc::now(),
                last_activity: Utc::now(),
            },
        };

        let result = engine.expand_choice(&choice, &context).await;
        assert!(result.is_ok());
        
        let new_node = result.unwrap();
        assert!(!new_node.situation.is_empty());
        assert!(!new_node.choices.is_empty());
    }
}