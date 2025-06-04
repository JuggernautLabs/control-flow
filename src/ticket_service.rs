use client_implementations::client::{LowLevelClient, QueryResolver, RetryConfig};
use client_implementations::error::QueryResolverError;
use crate::ticket::TicketDecomposition;
use tracing::{info, debug, instrument};

pub struct TicketService<C: LowLevelClient> {
    resolver: QueryResolver<C>,
}

impl<C: LowLevelClient + Send + Sync> TicketService<C> {
    pub fn new(client: C, config: RetryConfig) -> Self {
        info!("Creating new TicketService");
        Self {
            resolver: QueryResolver::new(client, config),
        }
    }
    
    #[instrument(skip(self, user_input), fields(input_len = user_input.len()))]
    pub async fn decompose_ticket(&self, user_input: String) -> Result<TicketDecomposition, QueryResolverError> {
        info!(input_len = user_input.len(), "Starting ticket decomposition");
        let prompt = self.build_decomposition_prompt(&user_input);
        debug!(prompt_len = prompt.len(), "Built decomposition prompt");
        
        let result = self.resolver.query(prompt).await;
        match &result {
            Ok(_) => info!("Ticket decomposition completed successfully"),
            Err(e) => info!(error = %e, "Ticket decomposition failed"),
        }
        result
    }
    
    fn build_decomposition_prompt(&self, user_input: &str) -> String {
        debug!(input_len = user_input.len(), "Building decomposition prompt for user input");
        let character_instructions = include_str!("../character.md");
        
        let prompt = format!(
            r#"{character_instructions}

Please decompose the following user input into a structured ticket using the exact JSON format specified in the instructions:

User Input: "{user_input}"

Respond with ONLY the JSON structure, no additional text or explanations."#,
            character_instructions = character_instructions,
            user_input = user_input
        );
        
        debug!(prompt_len = prompt.len(), "Built prompt");
        prompt
    }
}