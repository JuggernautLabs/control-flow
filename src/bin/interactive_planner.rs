use std::io::{self, Write};
use std::env;
use std::fs;
use std::path::Path;
use std::collections::HashMap;
use story_generation_engine::{
    StoryGenerationEngine, MockStoryEngine, OpenAIStoryEngine, ClaudeStoryEngine,
    StoryGraph, StoryNode, Choice, StoryContext, ProjectConstraints, 
    ExperienceLevel, StoryEdge, EdgeMetadata, ChoiceMetadata, RiskLevel
};
use uuid::Uuid;
use chrono::Utc;
use serde_json;

#[derive(Clone, Debug)]
enum PlannerMode {
    Planning,    // Iterative system planning mode
    Storytelling, // Traditional adventure storytelling
}

impl PlannerMode {
    fn description(&self) -> &str {
        match self {
            PlannerMode::Planning => "Iterative Planning - Break down complex goals into actionable steps",
            PlannerMode::Storytelling => "Adventure Storytelling - Traditional narrative exploration",
        }
    }

    fn perspective(&self) -> &str {
        match self {
            PlannerMode::Planning => "You are planning and designing a system",
            PlannerMode::Storytelling => "You are experiencing an adventure story",
        }
    }
}

struct InteractivePlanner {
    story: StoryGraph,
    engine: Box<dyn StoryGenerationEngine + Send + Sync>,
    current_node_id: Option<Uuid>,
    mode: PlannerMode,
    visited_nodes: HashMap<Uuid, bool>,
}

impl InteractivePlanner {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load environment variables
        dotenvy::dotenv().ok();
        
        println!("🎯 Welcome to the Interactive Planning System!");
        
        // Select mode first
        let mode = Self::select_mode()?;
        println!("🎭 Mode: {}", mode.description());
        
        // Select engine
        let engine = Self::select_engine().await?;

        Ok(Self {
            story: StoryGraph::default(),
            engine,
            current_node_id: None,
            mode,
            visited_nodes: HashMap::new(),
        })
    }

    fn select_mode() -> Result<PlannerMode, Box<dyn std::error::Error>> {
        println!("\n🎭 Choose your interaction mode:");
        println!("1. Planning Mode - Iterative system design and planning");
        println!("2. Storytelling Mode - Traditional adventure narratives");
        print!("Enter choice (1-2): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "2" => Ok(PlannerMode::Storytelling),
            _ => Ok(PlannerMode::Planning),
        }
    }

    async fn select_engine() -> Result<Box<dyn StoryGenerationEngine + Send + Sync>, Box<dyn std::error::Error>> {
        let default_engine = env::var("DEFAULT_ENGINE").unwrap_or_else(|_| "mock".to_string());
        
        println!("\n🚀 Choose your story engine:");
        println!("1. Mock Engine (predefined content)");
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
            "4" => Self::create_engine_from_env(),
            _ => {
                println!("📚 Using Mock Engine");
                Box::new(MockStoryEngine::new())
            }
        };

        Ok(engine)
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

    async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            println!("\n{}", "=".repeat(60));
            println!("🎯 Interactive Planning System");
            println!("Mode: {}", self.mode.description());
            if !self.story.title.is_empty() {
                println!("Current Story: {}", self.story.title);
                println!("Nodes: {} | Current: {}", 
                    self.story.nodes.len(),
                    if let Some(id) = self.current_node_id { 
                        format!("{}", id).chars().take(8).collect() 
                    } else { 
                        "None".to_string() 
                    }
                );
            }
            println!("{}", "=".repeat(60));

            println!("\n📋 Main Menu:");
            println!("1. Create new story/plan");
            println!("2. Load existing story");
            println!("3. Continue current story");
            println!("4. Navigate story tree");
            println!("5. Save current story");
            println!("6. View story tree");
            println!("7. Switch mode");
            println!("8. Quit");
            print!("\nEnter choice (1-8): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            match input.trim() {
                "1" => self.create_new_story().await?,
                "2" => self.load_story()?,
                "3" => self.continue_story().await?,
                "4" => self.navigate_tree().await?,
                "5" => self.save_story()?,
                "6" => self.view_tree(),
                "7" => self.switch_mode()?,
                "8" => break,
                _ => println!("❌ Invalid choice! Please enter 1-8."),
            }
        }

        println!("\n👋 Thanks for using the Interactive Planning System!");
        Ok(())
    }

    async fn create_new_story(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🌟 Let's start a new {}!", 
            match self.mode {
                PlannerMode::Planning => "planning conversation",
                PlannerMode::Storytelling => "adventure",
            }
        );
        
        let examples = match self.mode {
            PlannerMode::Planning => vec![
                "Build a todo app with real-time collaboration",
                "Design a microservices architecture",
                "Plan a marketing campaign for a startup",
                "Organize a company retreat",
            ],
            PlannerMode::Storytelling => vec![
                "A mysterious cave expedition",
                "Space station emergency",
                "Medieval castle intrigue",
                "Cyberpunk heist mission",
            ],
        };

        println!("💡 Examples:");
        for example in &examples {
            println!("  • {}", example);
        }

        print!("\nWhat would you like to {}? ",
            match self.mode {
                PlannerMode::Planning => "work on together",
                PlannerMode::Storytelling => "explore",
            }
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let prompt = input.trim();

        if prompt.is_empty() {
            println!("Using default scenario...");
        }

        // Get experience level
        let experience_level = self.get_experience_level()?;
        
        // Get additional constraints for planning mode
        let constraints = if matches!(self.mode, PlannerMode::Planning) {
            self.get_project_constraints(experience_level)?
        } else {
            ProjectConstraints {
                timeline: Some("Open-ended".to_string()),
                team_size: Some(1),
                experience_level,
                budget: None,
                technical_constraints: vec![],
                business_constraints: vec![],
            }
        };

        println!("\n🔮 Setting up our {}...", 
            match self.mode {
                PlannerMode::Planning => "collaborative planning conversation",
                PlannerMode::Storytelling => "adventure",
            }
        );
        
        self.story = self.engine.generate_initial_story(prompt, constraints).await?;
        self.current_node_id = self.story.root_node_id;
        self.visited_nodes.clear();
        
        if let Some(root_id) = self.story.root_node_id {
            self.visited_nodes.insert(root_id, true);
        }

        println!("✅ Created: {}", self.story.title);
        Ok(())
    }

    fn get_experience_level(&self) -> Result<ExperienceLevel, Box<dyn std::error::Error>> {
        println!("\n🎯 Choose your experience level:");
        match self.mode {
            PlannerMode::Planning => {
                println!("1. Beginner (simpler planning approaches)");
                println!("2. Intermediate (balanced complexity)");
                println!("3. Advanced (detailed strategic planning)");
                println!("4. Expert (comprehensive enterprise-level planning)");
            },
            PlannerMode::Storytelling => {
                println!("1. Beginner (easier choices)");
                println!("2. Intermediate (balanced difficulty)");
                println!("3. Advanced (complex scenarios)");
                println!("4. Expert (maximum challenge)");
            }
        }
        print!("Enter choice (1-4): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let experience_level = match input.trim() {
            "1" => ExperienceLevel::Beginner,
            "2" => ExperienceLevel::Intermediate,
            "3" => ExperienceLevel::Advanced,
            "4" => ExperienceLevel::Expert,
            _ => {
                println!("Invalid choice, using Intermediate");
                ExperienceLevel::Intermediate
            }
        };

        Ok(experience_level)
    }

    fn get_project_constraints(&self, experience_level: ExperienceLevel) -> Result<ProjectConstraints, Box<dyn std::error::Error>> {
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

        Ok(ProjectConstraints {
            timeline,
            team_size,
            experience_level,
            budget,
            technical_constraints: vec![],
            business_constraints: vec![],
        })
    }

    fn load_story(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let stories_dir = Path::new("stories");
        if !stories_dir.exists() {
            println!("❌ No stories directory found. Create a story first!");
            return Ok(());
        }

        let mut story_files = Vec::new();
        
        // Read all .json files in stories directory
        for entry in fs::read_dir(stories_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    story_files.push((filename.to_string(), path));
                }
            }
        }

        if story_files.is_empty() {
            println!("❌ No saved stories found in the stories directory!");
            return Ok(());
        }

        println!("\n📚 Saved Stories:");
        for (i, (filename, _)) in story_files.iter().enumerate() {
            println!("{}. {}", i + 1, filename.trim_end_matches(".json"));
        }
        print!("Enter choice (1-{}): ", story_files.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if let Ok(choice) = input.trim().parse::<usize>() {
            if choice > 0 && choice <= story_files.len() {
                let (_, path) = &story_files[choice - 1];
                let content = fs::read_to_string(path)?;
                self.story = serde_json::from_str(&content)?;
                self.current_node_id = self.story.root_node_id;
                self.visited_nodes.clear();
                
                // Mark all nodes as visited for loaded stories
                for node_id in self.story.nodes.keys() {
                    self.visited_nodes.insert(*node_id, true);
                }
                
                println!("✅ Loaded: {}", self.story.title);
            } else {
                println!("❌ Invalid choice!");
            }
        } else {
            println!("❌ Invalid input!");
        }

        Ok(())
    }

    fn save_story(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.story.title.is_empty() {
            println!("❌ No story to save! Create a story first.");
            return Ok(());
        }

        let stories_dir = Path::new("stories");
        if !stories_dir.exists() {
            fs::create_dir_all(stories_dir)?;
        }

        print!("💾 Enter filename (or press Enter for auto-generated): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let filename = input.trim();

        let filename = if filename.is_empty() {
            let safe_title = self.story.title.chars()
                .filter(|c| c.is_alphanumeric() || *c == ' ')
                .collect::<String>()
                .replace(' ', "_")
                .to_lowercase();
            
            // Truncate title to avoid filename length issues
            let safe_title = if safe_title.len() > 50 {
                safe_title.chars().take(50).collect::<String>()
            } else {
                safe_title
            };
            
            format!("{}-{}.json", 
                safe_title,
                Utc::now().format("%Y%m%d_%H%M%S")
            )
        } else {
            if filename.ends_with(".json") {
                filename.to_string()
            } else {
                format!("{}.json", filename)
            }
        };

        let filepath = stories_dir.join(&filename);
        let content = serde_json::to_string_pretty(&self.story)?;
        fs::write(&filepath, content)?;

        println!("✅ Story saved as: {}", filename);
        Ok(())
    }

    async fn continue_story(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.story.nodes.is_empty() {
            println!("❌ No active story! Create or load a story first.");
            return Ok(());
        }

        loop {
            if let Some(current_id) = self.current_node_id {
                if let Some(current_node) = self.story.nodes.get(&current_id).cloned() {
                    let should_continue = self.explore_node(current_node).await?;
                    if !should_continue {
                        break;
                    }
                } else {
                    println!("❌ Current node not found!");
                    break;
                }
            } else {
                println!("❌ No current node set!");
                break;
            }
        }

        Ok(())
    }

    async fn navigate_tree(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.story.nodes.is_empty() {
            println!("❌ No story loaded! Create or load a story first.");
            return Ok(());
        }

        loop {
            println!("\n🌳 Story Tree Navigation");
            println!("Current story: {}", self.story.title);
            println!("Total nodes: {}", self.story.nodes.len());
            
            // Show all nodes with their connections
            println!("\n📍 Available Nodes:");
            let mut node_list: Vec<_> = self.story.nodes.iter().collect();
            node_list.sort_by_key(|(id, _)| *id);
            
            for (i, (node_id, node)) in node_list.iter().enumerate() {
                let is_current = Some(**node_id) == self.current_node_id;
                let is_root = Some(**node_id) == self.story.root_node_id;
                let visited = *self.visited_nodes.get(node_id).unwrap_or(&false);
                
                let status = if is_current {
                    "👉 CURRENT"
                } else if is_root {
                    "🌱 ROOT"
                } else if visited {
                    "✅ VISITED"
                } else {
                    "⭕ UNVISITED"
                };
                
                println!("{}. {} | {} | {}", 
                    i + 1, 
                    status,
                    node_id.to_string().chars().take(8).collect::<String>(),
                    node.situation.chars().take(50).collect::<String>() + 
                    if node.situation.len() > 50 { "..." } else { "" }
                );
            }

            println!("\n🎯 Options:");
            println!("Enter node number (1-{}) to explore", node_list.len());
            println!("Type 'back' to return to main menu");
            print!("Choice: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input == "back" {
                break;
            }

            if let Ok(choice) = input.parse::<usize>() {
                if choice > 0 && choice <= node_list.len() {
                    let (node_id, node) = node_list[choice - 1];
                    self.current_node_id = Some(*node_id);
                    self.visited_nodes.insert(*node_id, true);
                    let _ = self.explore_node(node.clone()).await?;
                } else {
                    println!("❌ Invalid choice! Please enter 1-{}", node_list.len());
                }
            } else {
                println!("❌ Invalid input! Please enter a number or 'back'");
            }
        }

        Ok(())
    }

    async fn explore_node(&mut self, node: StoryNode) -> Result<bool, Box<dyn std::error::Error>> {
        println!("\n{}", "=".repeat(60));
        println!("💬 Your Planning Partner Says{}", 
            match self.mode {
                PlannerMode::Planning => ":",
                PlannerMode::Storytelling => " (Story Mode):",
            }
        );
        println!("{}", node.situation);
        
        if let Some(complexity) = node.complexity_score {
            println!("\n🎯 Complexity Level: {:.1}/1.0", complexity);
        }
        println!("{}", "=".repeat(60));

        loop {
            println!("\n🎯 Available Options:");
            
            // Show existing choices
            if !node.choices.is_empty() {
                println!("\n📋 Generated Options:");
                for (i, choice) in node.choices.iter().enumerate() {
                    println!("{}. {}", i + 1, choice.description);
                }
            }

            println!("\n🛠️  Or you can:");
            println!("c. Suggest your own direction");
            println!("r. Ask for different options");
            println!("h. Review our conversation history");
            println!("b. Navigate to different topic");
            print!("Enter choice: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            match input {
                "c" => {
                    let custom_choice = self.get_custom_option()?;
                    self.process_choice(&node, &custom_choice).await?;
                    return Ok(true); // Continue to next node
                },
                "r" => {
                    let new_choices = self.request_new_options(&node).await?;
                    self.display_and_select_choices(&node, &new_choices).await?;
                    return Ok(true); // Continue to next node
                },
                "h" => self.show_node_history(&node),
                "b" => return Ok(false), // Back to navigation/menu
                _ => {
                    if let Ok(choice_num) = input.parse::<usize>() {
                        if choice_num > 0 && choice_num <= node.choices.len() {
                            let choice = node.choices[choice_num - 1].clone();
                            self.process_choice(&node, &choice).await?;
                            return Ok(true); // Continue to next node
                        } else {
                            println!("❌ Invalid choice number!");
                        }
                    } else {
                        println!("❌ Invalid input!");
                    }
                }
            }
        }
    }

    fn get_custom_option(&self) -> Result<Choice, Box<dyn std::error::Error>> {
        print!("💭 What direction would you like to explore? ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let description = input.trim().to_string();

        Ok(Choice {
            id: Uuid::new_v4(),
            description,
            target_node_id: None,
            weight: 1.0,
            feasibility_score: Some(0.9), // User choices are highly feasible
            consequences: vec![],
            metadata: ChoiceMetadata {
                confidence_score: Some(1.0), // User is confident in their choice
                risk_level: RiskLevel::Low,
                time_estimate: None,
                dependencies: vec![],
            },
        })
    }

    async fn request_new_options(&mut self, node: &StoryNode) -> Result<Vec<Choice>, Box<dyn std::error::Error>> {
        println!("🔄 Let me think of some different approaches...");
        
        let context = StoryContext {
            previous_choices: vec![], // Simplified for now
            user_responses: vec![],
            current_constraints: self.story.metadata.project_constraints.clone(),
            session_metadata: story_generation_engine::SessionMetadata {
                session_id: Uuid::new_v4(),
                user_id: None,
                started_at: Utc::now(),
                last_activity: Utc::now(),
            },
        };

        let choices = self.engine.generate_choices(node, &context).await?;
        println!("✅ Here are {} fresh perspectives:", choices.len());
        Ok(choices)
    }

    async fn display_and_select_choices(&mut self, node: &StoryNode, choices: &[Choice]) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n💡 Here's what I'm thinking:");
        for (i, choice) in choices.iter().enumerate() {
            println!("{}. {}", i + 1, choice.description);
        }

        print!("Which direction interests you? (1-{}): ", choices.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if let Ok(choice_num) = input.trim().parse::<usize>() {
            if choice_num > 0 && choice_num <= choices.len() {
                let choice = choices[choice_num - 1].clone();
                self.process_choice(node, &choice).await?;
            } else {
                println!("❌ Invalid choice!");
            }
        } else {
            println!("❌ Invalid input!");
        }

        Ok(())
    }

    async fn process_choice(&mut self, current_node: &StoryNode, choice: &Choice) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n⚡ Great! You want to: {}", choice.description);

        // Check if choice leads to existing node
        if let Some(target_node_id) = choice.target_node_id {
            if self.story.nodes.contains_key(&target_node_id) {
                self.current_node_id = Some(target_node_id);
                self.visited_nodes.insert(target_node_id, true);
                println!("➡️  Navigated to existing node");
                return Ok(());
            }
        }

        // Generate new node
        println!("🔄 Let me think about what this opens up...");
        
        let context = StoryContext {
            previous_choices: vec![], // Simplified
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

        // Add edge
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
        self.current_node_id = Some(new_node_id);
        self.visited_nodes.insert(new_node_id, true);

        println!("✅ Interesting! This raises some new questions...");
        Ok(())
    }

    fn show_node_history(&self, node: &StoryNode) {
        println!("\n📜 Node History:");
        println!("ID: {}", node.id);
        println!("Type: {:?}", node.node_type);
        println!("State: {:?}", node.state);
        if let Some(complexity) = node.complexity_score {
            println!("Complexity: {:.2}", complexity);
        }
        println!("Created: {}", node.created_at.format("%Y-%m-%d %H:%M:%S"));
        println!("Generated by AI: {}", node.metadata.generated_by_ai);
        
        // Show incoming edges
        let incoming: Vec<_> = self.story.edges.iter()
            .filter(|edge| edge.to_node_id == node.id)
            .collect();
        
        if !incoming.is_empty() {
            println!("\n⬅️  Incoming connections: {}", incoming.len());
        }

        // Show outgoing edges  
        let outgoing: Vec<_> = self.story.edges.iter()
            .filter(|edge| edge.from_node_id == node.id)
            .collect();
            
        if !outgoing.is_empty() {
            println!("➡️  Outgoing connections: {}", outgoing.len());
        }
    }

    fn view_tree(&self) {
        if self.story.nodes.is_empty() {
            println!("❌ No story loaded!");
            return;
        }

        println!("\n🌳 Story Tree Overview");
        println!("Title: {}", self.story.title);
        println!("Nodes: {}", self.story.nodes.len());
        println!("Edges: {}", self.story.edges.len());
        
        if let Some(root_id) = self.story.root_node_id {
            println!("Root: {}", root_id.to_string().chars().take(8).collect::<String>());
        }

        println!("\n📊 Node Analysis:");
        let mut unvisited_count = 0;
        let mut visited_count = 0;
        
        for node_id in self.story.nodes.keys() {
            if *self.visited_nodes.get(node_id).unwrap_or(&false) {
                visited_count += 1;
            } else {
                unvisited_count += 1;
            }
        }
        
        println!("✅ Visited: {}", visited_count);
        println!("⭕ Unvisited: {}", unvisited_count);
        
        // Show tree structure (simplified)
        if let Some(root_id) = self.story.root_node_id {
            println!("\n🌳 Tree Structure (first level):");
            self.display_node_tree(root_id, 0, 2);
        }
    }

    fn display_node_tree(&self, node_id: Uuid, depth: usize, max_depth: usize) {
        if depth > max_depth {
            return;
        }

        let indent = "  ".repeat(depth);
        let is_current = Some(node_id) == self.current_node_id;
        let visited = *self.visited_nodes.get(&node_id).unwrap_or(&false);
        
        let status = if is_current {
            "👉"
        } else if visited {
            "✅"
        } else {
            "⭕"
        };

        if let Some(node) = self.story.nodes.get(&node_id) {
            println!("{}{} {}", 
                indent, 
                status,
                node.situation.chars().take(40).collect::<String>() +
                if node.situation.len() > 40 { "..." } else { "" }
            );

            // Show children
            let children: Vec<_> = self.story.edges.iter()
                .filter(|edge| edge.from_node_id == node_id)
                .map(|edge| edge.to_node_id)
                .collect();

            for child_id in children {
                self.display_node_tree(child_id, depth + 1, max_depth);
            }
        }
    }

    fn switch_mode(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let new_mode = Self::select_mode()?;
        self.mode = new_mode;
        println!("✅ Switched to: {}", self.mode.description());
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut planner = InteractivePlanner::new().await?;
    planner.run().await?;
    Ok(())
}