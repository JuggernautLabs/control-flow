use client_implementations::client::{LowLevelClient, QueryResolver, RetryConfig};
use client_implementations::error::QueryResolverError;
use crate::ticket::{TicketDecomposition, RefinementRequest, RefinementContext};
use tracing::{info, debug, instrument};

pub struct RefinementService<C: LowLevelClient> {
    resolver: QueryResolver<C>,
}

impl<C: LowLevelClient + Send + Sync> RefinementService<C> {
    pub fn new(client: C, config: RetryConfig) -> Self {
        info!("Creating new RefinementService");
        Self {
            resolver: QueryResolver::new(client, config),
        }
    }
    
    #[instrument(skip(self, refinement_request), fields(term = %refinement_request.term))]
    pub async fn refine_term(&self, refinement_request: &RefinementRequest, context: Option<&RefinementContext>) -> Result<TicketDecomposition, QueryResolverError> {
        info!(term = %refinement_request.term, "Starting term refinement");
        let prompt = self.build_refinement_prompt(refinement_request, context);
        debug!(prompt_len = prompt.len(), "Built refinement prompt");
        
        let result = self.resolver.query(prompt).await;
        match &result {
            Ok(_) => info!(term = %refinement_request.term, "Term refinement completed successfully"),
            Err(e) => info!(error = %e, term = %refinement_request.term, "Term refinement failed"),
        }
        result
    }
    
    fn build_refinement_prompt(&self, refinement_request: &RefinementRequest, context: Option<&RefinementContext>) -> String {
        debug!(term = %refinement_request.term, "Building refinement prompt");
        let character_instructions = include_str!("../character.md");
        
        let context_info = if let Some(ctx) = context {
            format!(
                "\n\nContext Information:\n- Parent Ticket ID: {}\n- Original Context: {}\n- Term Being Refined: {}",
                ctx.parent_ticket_id,
                ctx.original_context,
                ctx.term_being_refined
            )
        } else {
            String::new()
        };
        
        let prompt = format!(
            r#"{character_instructions}

## REFINEMENT REQUEST MODE

You are now in REFINEMENT MODE. Your task is to create a focused ticket that clarifies and refines a specific term that was identified as needing refinement in a previous ticket decomposition.

### Term to Refine:
**Term:** {term}
**Context:** {term_context}
**Reason for Refinement:** {reason}
**Priority:** {priority:?}{context_info}

### Instructions:
1. Create a NEW ticket specifically focused on clarifying this term
2. The ticket should explore all possible interpretations of the term
3. Provide specific questions to disambiguate the term
4. Include technical specification requirements
5. Consider platform, technology, scale, and implementation constraints
6. The resulting ticket should enable clear decision-making about this term

### Expected Output:
Create a complete TicketDecomposition JSON structure that focuses specifically on refining "{term}". The title should clearly indicate this is a refinement ticket for the specific term.

Respond with ONLY the JSON structure, no additional text or explanations."#,
            character_instructions = character_instructions,
            term = refinement_request.term,
            term_context = refinement_request.context,
            reason = refinement_request.reason,
            priority = refinement_request.priority,
            context_info = context_info
        );
        
        debug!(prompt_len = prompt.len(), "Built refinement prompt");
        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ticket::{RefinementPriority, OriginalTicket, DecomposedTicket, TicketMetadata, TicketStatus, Priority, Complexity, ValidationResults, TicketId};
    use std::collections::HashMap;

    fn create_test_refinement_request() -> RefinementRequest {
        RefinementRequest {
            term: "user-friendly".to_string(),
            context: "create a user-friendly interface".to_string(),
            reason: "ambiguous - could mean accessible, intuitive, minimalist, or mobile-responsive".to_string(),
            priority: RefinementPriority::High,
        }
    }

    #[test]
    fn test_refinement_prompt_building() {
        use client_implementations::claude::ClaudeClient;
        
        let client = ClaudeClient::new("test-key".to_string());
        let config = RetryConfig::default();
        let service = RefinementService::new(client, config);
        
        let refinement_request = create_test_refinement_request();
        let prompt = service.build_refinement_prompt(&refinement_request, None);
        
        assert!(prompt.contains("user-friendly"));
        assert!(prompt.contains("create a user-friendly interface"));
        assert!(prompt.contains("ambiguous - could mean accessible"));
        assert!(prompt.contains("REFINEMENT MODE"));
    }

    #[test]
    fn test_refinement_prompt_with_context() {
        use client_implementations::claude::ClaudeClient;
        
        let client = ClaudeClient::new("test-key".to_string());
        let config = RetryConfig::default();
        let service = RefinementService::new(client, config);
        
        let refinement_request = create_test_refinement_request();
        let parent_ticket = TicketDecomposition {
            original_ticket: OriginalTicket {
                title: "Parent Ticket".to_string(),
                raw_input: "Original input".to_string(),
            },
            decomposed_ticket: DecomposedTicket {
                terms: HashMap::new(),
                terms_needing_refinement: vec![],
                open_questions: vec![],
                engine_questions: vec![],
                validation_method: vec![],
                validation_results: ValidationResults {
                    mime: "text/plain".to_string(),
                    url: "placeholder".to_string(),
                },
                metadata: TicketMetadata {
                    status: TicketStatus::AwaitingRefinement,
                    priority: Priority::Medium,
                    estimated_complexity: Complexity::Low,
                    processed_at: "2024-01-01T00:00:00Z".to_string(),
                    engine_version: "1.0".to_string(),
                },
            },
        };
        
        let parent_id = TicketId::generate(&parent_ticket);
        let context = RefinementContext {
            parent_ticket_id: parent_id.clone(),
            term_being_refined: "user-friendly".to_string(),
            original_context: "UI design requirements".to_string(),
            additional_context: vec![],
        };
        
        let prompt = service.build_refinement_prompt(&refinement_request, Some(&context));
        
        assert!(prompt.contains("user-friendly"));
        assert!(prompt.contains("Parent Ticket ID"));
        assert!(prompt.contains("UI design requirements"));
    }
}