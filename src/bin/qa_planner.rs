use std::io::{self, Write};
use std::env;
use std::fs;
use std::path::Path;
use std::collections::HashMap;
use story_generation_engine::{
    StoryGenerationEngine, MockStoryEngine, OpenAIStoryEngine, ClaudeStoryEngine,
    QAEngine, QASession, PlanningQuestion, PlanningAnswer, ProjectConstraints, 
    ExperienceLevel, QuestionPriority, PlanningQuestionType, QASessionMetadata,
    SessionType, RequestType, RetryResult
};
use uuid::Uuid;
use chrono::Utc;
use serde_json;

struct QAPlanner {
    qa_engine: QAEngine,
    current_session: Option<QASession>,
}

impl QAPlanner {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load environment variables
        dotenvy::dotenv().ok();
        
        println!("🎯 Welcome to the Collaborative Planning System!");
        println!("💡 This system uses a Question-Answer approach to help you plan projects.");
        
        // Select engine
        let story_engine = Self::select_engine().await?;
        let qa_engine = QAEngine::new(story_engine);

        Ok(Self {
            qa_engine,
            current_session: None,
        })
    }

    async fn select_engine() -> Result<Box<dyn StoryGenerationEngine + Send + Sync>, Box<dyn std::error::Error>> {
        let default_engine = env::var("DEFAULT_ENGINE").unwrap_or_else(|_| "mock".to_string());
        
        println!("\n🚀 Choose your planning engine:");
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
            println!("🎯 Collaborative Planning System (Q&A Mode)");
            if let Some(session) = &self.current_session {
                println!("Current Session: {}", session.title);
                println!("Questions: {} | Answers: {} | Completion: {:.1}%", 
                    session.questions.len(),
                    session.answers.len(),
                    session.session_metadata.completion_percentage * 100.0
                );
            }
            println!("{}", "=".repeat(60));

            println!("\n📋 Main Menu:");
            println!("1. Start new planning session");
            println!("2. Load existing session");
            println!("3. Continue current session");
            println!("4. Browse questions");
            println!("5. Save current session");
            println!("6. View session overview");
            println!("7. Handle pending requests");
            println!("8. Quit");
            
            // Show pending requests indicator
            if let Some(session) = &self.current_session {
                let pending_count = session.pending_requests.len();
                if pending_count > 0 {
                    println!("⚠️  {} pending requests need attention", pending_count);
                }
            }
            print!("\nEnter choice (1-8): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            match input.trim() {
                "1" => self.start_new_session().await?,
                "2" => self.load_session()?,
                "3" => self.continue_session().await?,
                "4" => self.browse_questions().await?,
                "5" => self.save_session()?,
                "6" => self.view_session_overview(),
                "7" => self.handle_pending_requests().await?,
                "8" => break,
                _ => println!("❌ Invalid choice! Please enter 1-8."),
            }
        }

        println!("\n👋 Thanks for using the Collaborative Planning System!");
        Ok(())
    }

    async fn start_new_session(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🌟 Let's start a new planning conversation!");
        
        let examples = vec![
            "Build a todo app with real-time collaboration",
            "Design a microservices architecture for e-commerce",
            "Plan a marketing campaign for a startup",
            "Create a mobile app for fitness tracking",
            "Organize a company retreat for 50 people",
        ];

        println!("💡 Examples:");
        for example in &examples {
            println!("  • {}", example);
        }

        print!("\nWhat would you like to plan together? ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let project_description = input.trim();

        if project_description.is_empty() {
            println!("Using default project...");
        }

        // Get project constraints
        let constraints = self.get_project_constraints().await?;

        println!("\n🔮 Generating initial planning questions...");
        
        match self.qa_engine.generate_initial_questions(project_description, constraints.clone()).await {
            Ok(session) => {
                println!("✅ Generated {} questions to explore together!", session.questions.len());
                self.current_session = Some(session);
            },
            Err(e) if QAEngine::is_retryable_error(&e) => {
                println!("⚠️  AI service temporarily unavailable: {}", e);
                println!("💾 Creating session and saving request for later retry...");
                
                let mut session = QASession {
                    id: Uuid::new_v4(),
                    title: format!("Planning Session: {}", project_description),
                    description: Some("Collaborative planning conversation".to_string()),
                    questions: HashMap::new(),
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
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                
                self.qa_engine.add_pending_request(
                    &mut session,
                    RequestType::InitialQuestions,
                    project_description.to_string(),
                    None,
                    format!("{}", e),
                );
                
                println!("📝 Session created. You can retry the request later or continue with manual questions.");
                self.current_session = Some(session);
            },
            Err(e) => {
                println!("❌ Failed to generate questions: {}", e);
                println!("💡 You can still create a session and add questions manually.");
                return Ok(());
            }
        }

        Ok(())
    }

    async fn get_project_constraints(&self) -> Result<ProjectConstraints, Box<dyn std::error::Error>> {
        println!("\n🎯 Choose your experience level:");
        println!("1. Beginner (simpler planning approaches)");
        println!("2. Intermediate (balanced complexity)");
        println!("3. Advanced (detailed strategic planning)");
        println!("4. Expert (comprehensive enterprise-level planning)");
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

    async fn continue_session(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.current_session.is_none() {
            println!("❌ No active session! Start a new session or load an existing one.");
            return Ok(());
        }

        loop {
            let should_continue = self.explore_questions().await?;
            if !should_continue {
                break;
            }
        }

        Ok(())
    }

    async fn explore_questions(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        let session = self.current_session.as_ref().unwrap();
        
        println!("\n💭 Questions to Explore:");
        
        // Group questions by priority
        let mut critical_questions = Vec::new();
        let mut high_questions = Vec::new();
        let mut other_questions = Vec::new();
        
        for (id, question) in &session.questions {
            let has_answer = session.answers.values().any(|a| a.question_id == *id);
            if !has_answer {
                match question.priority {
                    QuestionPriority::Critical => critical_questions.push((id, question)),
                    QuestionPriority::High => high_questions.push((id, question)),
                    _ => other_questions.push((id, question)),
                }
            }
        }

        let mut question_list = Vec::new();
        let mut index = 1;

        if !critical_questions.is_empty() {
            println!("\n🔥 Critical Questions:");
            for (id, question) in critical_questions {
                println!("{}. {} | {}", index, question.question, 
                    self.format_question_type(&question.question_type));
                question_list.push(*id);
                index += 1;
            }
        }

        if !high_questions.is_empty() {
            println!("\n⭐ Important Questions:");
            for (id, question) in high_questions {
                println!("{}. {} | {}", index, question.question,
                    self.format_question_type(&question.question_type));
                question_list.push(*id);
                index += 1;
            }
        }

        if !other_questions.is_empty() {
            println!("\n💡 Other Questions:");
            for (id, question) in other_questions {
                println!("{}. {} | {}", index, question.question,
                    self.format_question_type(&question.question_type));
                question_list.push(*id);
                index += 1;
            }
        }

        if question_list.is_empty() {
            println!("\n🎉 All questions have been explored!");
            println!("You can review answers or generate follow-up questions.");
            return Ok(false);
        }

        println!("\n🛠️  Options:");
        println!("a. Add custom question");
        println!("r. Review existing answers");
        println!("b. Back to main menu");
        print!("Enter question number (1-{}) or option (a/r/b): ", question_list.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "a" => {
                self.add_custom_question().await?;
                Ok(true)
            },
            "r" => {
                self.review_answers();
                Ok(true)
            },
            "b" => Ok(false),
            _ => {
                if let Ok(choice) = input.parse::<usize>() {
                    if choice > 0 && choice <= question_list.len() {
                        let question_id = question_list[choice - 1];
                        self.explore_specific_question(question_id).await?;
                        Ok(true)
                    } else {
                        println!("❌ Invalid choice!");
                        Ok(true)
                    }
                } else {
                    println!("❌ Invalid input!");
                    Ok(true)
                }
            }
        }
    }

    async fn explore_specific_question(&mut self, question_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        let session = self.current_session.as_ref().unwrap().clone();
        let question = session.questions.get(&question_id).unwrap();

        println!("\n{}", "=".repeat(60));
        println!("❓ Exploring Question:");
        println!("{}", question.question);
        println!("Context: {}", question.context);
        println!("Type: {} | Priority: {:?}", 
            self.format_question_type(&question.question_type),
            question.priority
        );
        println!("{}", "=".repeat(60));

        // Check if we already have answers for this question
        let existing_answers: Vec<_> = session.answers.values()
            .filter(|a| a.question_id == question_id)
            .collect();

        if !existing_answers.is_empty() {
            println!("\n📋 Existing Answers:");
            for (i, answer) in existing_answers.iter().enumerate() {
                println!("{}. {} | Confidence: {:.1} | Status: {:?}", 
                    i + 1, answer.answer, answer.confidence, answer.metadata.validation_status);
            }
            
            println!("\n🛠️  Options:");
            println!("1. Explore new approaches to this question");
            println!("2. Deepen an existing answer");
            println!("3. Generate follow-up questions");
            println!("4. Back to question list");
            print!("Enter choice (1-4): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            match input.trim() {
                "1" => self.generate_new_approaches(question_id).await?,
                "2" => self.deepen_existing_answer(question_id, &existing_answers).await?,
                "3" => self.generate_follow_ups(question_id, &existing_answers).await?,
                _ => return Ok(()),
            }
        } else {
            // No existing answers, generate new approaches
            self.generate_new_approaches(question_id).await?;
        }

        Ok(())
    }

    async fn generate_new_approaches(&mut self, question_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        let session = self.current_session.as_ref().unwrap();
        
        println!("\n🔄 Let me think of different approaches to this question...");
        
        let answers = self.qa_engine.explore_question(session, question_id).await?;
        
        println!("\n💡 Here are {} different approaches we could take:", answers.len());
        for (i, answer) in answers.iter().enumerate() {
            println!("{}. {}", i + 1, answer.answer);
        }

        print!("\nWhich approach interests you most? (1-{}) or 'all' to explore all: ", answers.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        let session = self.current_session.as_mut().unwrap();

        if input == "all" {
            // Add all answers to the session
            for answer in answers {
                session.answers.insert(answer.id, answer);
            }
            println!("✅ Added all approaches to explore further!");
        } else if let Ok(choice) = input.parse::<usize>() {
            if choice > 0 && choice <= answers.len() {
                let selected_answer = answers[choice - 1].clone();
                println!("🎯 Great choice! You want to explore: {}", selected_answer.answer);
                
                // Deepen this answer immediately
                let deepened = self.qa_engine.deepen_answer(session, &selected_answer).await?;
                session.answers.insert(deepened.id, deepened);
                
                println!("✅ Added this approach and explored it deeper!");
            }
        }

        Ok(())
    }

    async fn deepen_existing_answer(&mut self, question_id: Uuid, existing_answers: &[&PlanningAnswer]) -> Result<(), Box<dyn std::error::Error>> {
        if existing_answers.len() == 1 {
            let answer = existing_answers[0];
            println!("\n🔍 Deepening exploration of: {}", answer.answer);
            
            let session = self.current_session.as_mut().unwrap();
            let deepened = self.qa_engine.deepen_answer(session, answer).await?;
            session.answers.insert(deepened.id, deepened);
            
            println!("✅ Gained deeper insights into this approach!");
        } else {
            println!("\n🔍 Which answer would you like to explore deeper?");
            for (i, answer) in existing_answers.iter().enumerate() {
                println!("{}. {}", i + 1, answer.answer);
            }
            
            print!("Enter choice (1-{}): ", existing_answers.len());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            if let Ok(choice) = input.trim().parse::<usize>() {
                if choice > 0 && choice <= existing_answers.len() {
                    let answer = existing_answers[choice - 1];
                    println!("🔍 Deepening exploration of: {}", answer.answer);
                    
                    let session = self.current_session.as_mut().unwrap();
                    let deepened = self.qa_engine.deepen_answer(session, answer).await?;
                    session.answers.insert(deepened.id, deepened);
                    
                    println!("✅ Gained deeper insights!");
                }
            }
        }

        Ok(())
    }

    async fn generate_follow_ups(&mut self, question_id: Uuid, existing_answers: &[&PlanningAnswer]) -> Result<(), Box<dyn std::error::Error>> {
        if existing_answers.is_empty() {
            println!("❌ No answers to generate follow-ups from!");
            return Ok(());
        }

        let session = self.current_session.as_ref().unwrap();
        
        // Use the first answer to generate follow-ups (could be enhanced to let user choose)
        let answer = existing_answers[0];
        
        println!("\n🔄 Generating follow-up questions based on: {}", answer.answer);
        
        let follow_ups = self.qa_engine.generate_follow_up_questions(session, answer).await?;
        
        println!("\n🤔 This exploration raises these new questions:");
        for (i, question) in follow_ups.iter().enumerate() {
            println!("{}. {}", i + 1, question.question);
        }

        print!("\nAdd these questions to explore later? (y/n): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase().starts_with('y') {
            let session = self.current_session.as_mut().unwrap();
            for question in follow_ups {
                session.questions.insert(question.id, question);
            }
            println!("✅ Added follow-up questions to your session!");
        }

        Ok(())
    }

    async fn add_custom_question(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        print!("💭 What question would you like to explore? ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let question_text = input.trim().to_string();

        print!("📝 Any context or background for this question? ");
        io::stdout().flush()?;
        
        let mut context_input = String::new();
        io::stdin().read_line(&mut context_input)?;
        let context = context_input.trim().to_string();

        let custom_question = PlanningQuestion {
            id: Uuid::new_v4(),
            question: question_text,
            context: if context.is_empty() { "User-generated question".to_string() } else { context },
            question_type: PlanningQuestionType::Requirements, // Default, could be smarter
            priority: QuestionPriority::Medium,
            dependencies: vec![],
            metadata: story_generation_engine::PlanningQuestionMetadata {
                generated_by_ai: false,
                user_initiated: true,
                complexity_score: Some(0.5),
                estimated_time_to_answer: Some("15-30 minutes".to_string()),
                related_topics: vec![],
            },
            created_at: Utc::now(),
        };

        let session = self.current_session.as_mut().unwrap();
        session.questions.insert(custom_question.id, custom_question);
        
        println!("✅ Added your custom question!");
        Ok(())
    }

    fn review_answers(&self) {
        let session = self.current_session.as_ref().unwrap();
        
        println!("\n📚 Answer Review:");
        
        if session.answers.is_empty() {
            println!("No answers yet! Start exploring questions to build up your knowledge base.");
            return;
        }

        for (i, (_, answer)) in session.answers.iter().enumerate() {
            let question = session.questions.get(&answer.question_id).unwrap();
            
            println!("\n{}. Question: {}", i + 1, question.question);
            println!("   Answer: {}", answer.answer);
            println!("   Status: {:?} | Confidence: {:.1}", 
                answer.metadata.validation_status, answer.confidence);
            
            if !answer.exploration_notes.is_empty() {
                println!("   Notes: {}", answer.exploration_notes.join("; "));
            }
        }
    }

    async fn browse_questions(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.current_session.is_none() {
            println!("❌ No active session! Start a new session first.");
            return Ok(());
        }

        let session = self.current_session.as_ref().unwrap();
        
        println!("\n📖 Question Browser:");
        
        let mut questions: Vec<_> = session.questions.values().collect();
        questions.sort_by_key(|q| &q.created_at);
        
        for (i, question) in questions.iter().enumerate() {
            let has_answer = session.answers.values().any(|a| a.question_id == question.id);
            let status = if has_answer { "✅" } else { "❓" };
            
            println!("{}. {} {} | {} | {:?}", 
                i + 1, status, question.question,
                self.format_question_type(&question.question_type),
                question.priority
            );
        }

        println!("\n🛠️  Options:");
        print!("Enter question number to explore, or 'back' to return: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input != "back" {
            if let Ok(choice) = input.parse::<usize>() {
                if choice > 0 && choice <= questions.len() {
                    let question_id = questions[choice - 1].id;
                    self.explore_specific_question(question_id).await?;
                }
            }
        }

        Ok(())
    }

    fn save_session(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.current_session.is_none() {
            println!("❌ No session to save!");
            return Ok(());
        }

        let session = self.current_session.as_ref().unwrap();
        
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
            let safe_title = session.title.chars()
                .filter(|c| c.is_alphanumeric() || *c == ' ')
                .collect::<String>()
                .replace(' ', "_")
                .to_lowercase();
            
            let safe_title = if safe_title.len() > 50 {
                safe_title.chars().take(50).collect::<String>()
            } else {
                safe_title
            };
            
            format!("qa_{}-{}.json", 
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
        let content = serde_json::to_string_pretty(&session)?;
        fs::write(&filepath, content)?;

        println!("✅ Session saved as: {}", filename);
        Ok(())
    }

    fn load_session(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let stories_dir = Path::new("stories");
        if !stories_dir.exists() {
            println!("❌ No stories directory found!");
            return Ok(());
        }

        let mut session_files = Vec::new();
        
        for entry in fs::read_dir(stories_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if filename.starts_with("qa_") {
                        session_files.push((filename.to_string(), path));
                    }
                }
            }
        }

        if session_files.is_empty() {
            println!("❌ No QA sessions found!");
            return Ok(());
        }

        println!("\n📚 Saved QA Sessions:");
        for (i, (filename, _)) in session_files.iter().enumerate() {
            println!("{}. {}", i + 1, filename.trim_start_matches("qa_").trim_end_matches(".json"));
        }
        print!("Enter choice (1-{}): ", session_files.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if let Ok(choice) = input.trim().parse::<usize>() {
            if choice > 0 && choice <= session_files.len() {
                let (_, path) = &session_files[choice - 1];
                let content = fs::read_to_string(path)?;
                let session: QASession = serde_json::from_str(&content)?;
                
                println!("✅ Loaded: {}", session.title);
                self.current_session = Some(session);
            }
        }

        Ok(())
    }

    fn view_session_overview(&self) {
        if let Some(session) = &self.current_session {
            println!("\n📊 Session Overview:");
            println!("Title: {}", session.title);
            if let Some(desc) = &session.description {
                println!("Description: {}", desc);
            }
            
            println!("\n📈 Progress:");
            println!("Questions: {}", session.questions.len());
            println!("Answers: {}", session.answers.len());
            println!("Completion: {:.1}%", session.session_metadata.completion_percentage * 100.0);
            println!("Implementation Readiness: {:.1}%", session.session_metadata.readiness_for_implementation * 100.0);
            
            // Question type breakdown
            let mut type_counts = HashMap::new();
            for question in session.questions.values() {
                *type_counts.entry(&question.question_type).or_insert(0) += 1;
            }
            
            println!("\n🏷️  Question Types:");
            for (qtype, count) in type_counts {
                println!("  {}: {}", self.format_question_type(qtype), count);
            }
            
            // Answer validation status
            let mut validation_counts = HashMap::new();
            for answer in session.answers.values() {
                *validation_counts.entry(&answer.metadata.validation_status).or_insert(0) += 1;
            }
            
            if !validation_counts.is_empty() {
                println!("\n✅ Answer Status:");
                for (status, count) in validation_counts {
                    println!("  {:?}: {}", status, count);
                }
            }
        } else {
            println!("❌ No active session!");
        }
    }

    fn format_question_type(&self, qtype: &PlanningQuestionType) -> String {
        match qtype {
            PlanningQuestionType::Requirements => "Requirements".to_string(),
            PlanningQuestionType::Architecture => "Architecture".to_string(),
            PlanningQuestionType::Dependencies => "Dependencies".to_string(),
            PlanningQuestionType::Priorities => "Priorities".to_string(),
            PlanningQuestionType::Constraints => "Constraints".to_string(),
            PlanningQuestionType::Risks => "Risks".to_string(),
            PlanningQuestionType::Implementation => "Implementation".to_string(),
            PlanningQuestionType::Validation => "Validation".to_string(),
        }
    }

    async fn handle_pending_requests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.current_session.is_none() {
            println!("❌ No active session! Start a new session first.");
            return Ok(());
        }

        // Clone request data to avoid borrowing issues
        let (retryable_requests, request_data): (Vec<_>, Vec<_>) = {
            let session = self.current_session.as_ref().unwrap();
            let retryable_requests = self.qa_engine.get_retryable_requests(session);

            if retryable_requests.is_empty() {
                println!("✅ No pending requests to handle!");
                return Ok(());
            }

            let request_data: Vec<_> = retryable_requests.iter().map(|req| {
                (req.id, req.request_type.clone(), req.context.clone(), req.error_message.clone(), req.created_at, req.retry_count)
            }).collect();

            (retryable_requests.into_iter().map(|req| req.id).collect(), request_data)
        };

        println!("\n🔄 Pending Requests:\n");
        for (i, (_id, request_type, context, error_message, created_at, retry_count)) in request_data.iter().enumerate() {
            println!("{}. {} | Retries: {}/{}", 
                i + 1,
                self.format_request_type(request_type),
                retry_count,
                3
            );
            println!("   Context: {}", context);
            println!("   Error: {}", error_message);
            println!("   Created: {}", created_at.format("%Y-%m-%d %H:%M:%S"));
            println!();
        }

        println!("🛠️  Options:");
        println!("1. Retry all pending requests");
        println!("2. Retry specific request");
        println!("3. Remove failed request");
        println!("4. Back to main menu");
        print!("Enter choice (1-4): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "1" => self.retry_all_pending_requests().await?,
            "2" => self.retry_specific_request(&retryable_requests).await?,
            "3" => self.remove_failed_request(&retryable_requests)?,
            _ => return Ok(()),
        }

        Ok(())
    }

    async fn retry_all_pending_requests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let session = self.current_session.as_ref().unwrap();
        let retryable_requests: Vec<Uuid> = session.pending_requests.keys().cloned().collect();

        println!("\n🔄 Retrying {} pending requests...", retryable_requests.len());

        let mut success_count = 0;
        let mut still_failing_count = 0;
        let mut permanent_failure_count = 0;

        for request_id in retryable_requests {
            let session = self.current_session.as_mut().unwrap();
            match self.qa_engine.retry_pending_request(session, request_id).await {
                Ok(RetryResult::Success) => {
                    println!("✅ Request {} succeeded!", request_id);
                    success_count += 1;
                },
                Ok(RetryResult::StillFailing(error)) => {
                    println!("⚠️  Request {} still failing: {}", request_id, error);
                    still_failing_count += 1;
                },
                Ok(RetryResult::PermanentFailure(error)) => {
                    println!("❌ Request {} permanently failed: {}", request_id, error);
                    permanent_failure_count += 1;
                },
                Err(e) => {
                    println!("❌ Error retrying request {}: {}", request_id, e);
                    permanent_failure_count += 1;
                }
            }
        }

        println!("\n📊 Retry Summary:");
        println!("  ✅ Succeeded: {}", success_count);
        println!("  ⚠️  Still failing: {}", still_failing_count);
        println!("  ❌ Permanent failures: {}", permanent_failure_count);

        Ok(())
    }

    async fn retry_specific_request(&mut self, retryable_requests: &[Uuid]) -> Result<(), Box<dyn std::error::Error>> {
        print!("Enter request number to retry (1-{}): ", retryable_requests.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if let Ok(choice) = input.trim().parse::<usize>() {
            if choice > 0 && choice <= retryable_requests.len() {
                let request_id = retryable_requests[choice - 1];
                let session = self.current_session.as_mut().unwrap();
                
                println!("🔄 Retrying request...");
                match self.qa_engine.retry_pending_request(session, request_id).await {
                    Ok(RetryResult::Success) => {
                        println!("✅ Request succeeded!");
                    },
                    Ok(RetryResult::StillFailing(error)) => {
                        println!("⚠️  Request still failing: {}", error);
                        println!("💡 You can try again later or remove this request.");
                    },
                    Ok(RetryResult::PermanentFailure(error)) => {
                        println!("❌ Request permanently failed: {}", error);
                        println!("💡 This request has been removed from pending list.");
                    },
                    Err(e) => {
                        println!("❌ Error retrying request: {}", e);
                    }
                }
            } else {
                println!("❌ Invalid choice!");
            }
        } else {
            println!("❌ Invalid input!");
        }

        Ok(())
    }

    fn remove_failed_request(&mut self, retryable_requests: &[Uuid]) -> Result<(), Box<dyn std::error::Error>> {
        print!("Enter request number to remove (1-{}): ", retryable_requests.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if let Ok(choice) = input.trim().parse::<usize>() {
            if choice > 0 && choice <= retryable_requests.len() {
                let request_id = retryable_requests[choice - 1];
                let session = self.current_session.as_mut().unwrap();
                
                if session.pending_requests.remove(&request_id).is_some() {
                    println!("✅ Removed failed request from pending list.");
                    session.updated_at = Utc::now();
                } else {
                    println!("❌ Request not found!");
                }
            } else {
                println!("❌ Invalid choice!");
            }
        } else {
            println!("❌ Invalid input!");
        }

        Ok(())
    }

    fn format_request_type(&self, request_type: &RequestType) -> String {
        match request_type {
            RequestType::InitialQuestions => "Generate Initial Questions".to_string(),
            RequestType::ExploreQuestion { question_id } => format!("Explore Question {}", question_id),
            RequestType::DeepenAnswer { answer_id } => format!("Deepen Answer {}", answer_id),
            RequestType::GenerateFollowUps { answer_id } => format!("Generate Follow-ups for {}", answer_id),
            RequestType::CustomQuestion { question_text } => format!("Custom Question: {}", question_text),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut planner = QAPlanner::new().await?;
    planner.run().await?;
    Ok(())
}