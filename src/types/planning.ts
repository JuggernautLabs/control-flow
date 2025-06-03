// Core Planning Primitives for Global Feature Queue System

/**
 * Work refinement states - represents the progression from high-level to implementable
 */
export type WorkRefinementState = 
  | 'feature' // High-level feature description
  | 'refined' // Broken down into implementable components  
  | 'implementable' // Ready for implementation planning
  | 'planned' // Has implementation plan with tests
  | 'in_progress' // Being actively worked on
  | 'completed' // Implementation finished
  | 'verified' // Implementation verified against semantics

/**
 * Semantic confidence scoring for AI-driven decisions
 */
export interface SemanticConfidence {
  confidence: number; // 0-1 score
  reasoning: string;
}

/**
 * Interface definition for implementable components
 */
export interface ComponentInterface {
  name: string;
  signature: string; // e.g., "addTodo(item: Todo): Promise<void>"
  purpose: string;
  inputs: Array<{
    name: string;
    type: string;
    description: string;
  }>;
  outputs: Array<{
    name: string;
    type: string;
    description: string;
  }>;
  preconditions?: string[];
  postconditions?: string[];
}

/**
 * Test specification for TDD implementation
 */
export interface TestSpecification {
  testName: string;
  description: string;
  setup: string; // Test setup code/description
  action: string; // What action is being tested
  expectedOutcome: string; // Expected result
  semanticCorrelation: SemanticConfidence; // How well test correlates with ticket semantics
}

/**
 * Implementation plan generated from ticket interfaces
 */
export interface ImplementationPlan {
  ticketId: string;
  interfaces: ComponentInterface[];
  testSpecs: TestSpecification[];
  dependencies: string[]; // Other ticket IDs this depends on
  estimatedComplexity: 'low' | 'medium' | 'high';
  createdAt: string;
  validatedAt?: string;
}

/**
 * Core ticket abstraction - represents any unit of work
 */
export interface Ticket {
  id: string;
  title: string;
  description: string;
  refinementState: WorkRefinementState;
  
  // Semantic analysis results
  semanticDescription: {
    complexity: SemanticConfidence;
    scope: SemanticConfidence;
    implementability: SemanticConfidence;
  };
  
  // Refinement tracking
  parentTicketId?: string; // If this was refined from a larger ticket
  childTicketIds?: string[]; // If this was refined into smaller tickets
  
  // Implementation readiness
  interfaces?: ComponentInterface[];
  implementationPlan?: ImplementationPlan;
  
  // Work tracking
  assignedTo?: string;
  pickupTimestamp?: string;
  completionTimestamp?: string;
  
  // Metadata
  createdAt: string;
  updatedAt: string;
  tags: string[];
  priority: 'low' | 'medium' | 'high' | 'critical';
}

/**
 * Feature - highest level work description (specialized ticket)
 */
export interface Feature extends Ticket {
  refinementState: 'feature' | 'refined';
  businessValue: string;
  stakeholders: string[];
  acceptanceCriteria: string[];
}

/**
 * Refinement analysis result
 */
export interface RefinementAnalysis {
  shouldRefine: SemanticConfidence;
  suggestedBreakdown?: Array<{
    title: string;
    description: string;
    interfaces: ComponentInterface[];
    estimatedComplexity: 'low' | 'medium' | 'high';
  }>;
  refinementReasoning: string;
}

/**
 * Global feature queue operations
 */
export interface GlobalFeatureQueue {
  // Queue state
  getAllTickets(): Promise<Ticket[]>;
  getAvailableWork(): Promise<Ticket[]>; // Tickets ready to be picked up
  getTicketById(id: string): Promise<Ticket | null>;
  
  // Work flow operations
  addFeature(feature: Omit<Feature, 'id' | 'createdAt' | 'updatedAt'>): Promise<Feature>;
  refineTicket(ticketId: string): Promise<RefinementAnalysis>;
  markImplementable(ticketId: string, interfaces: ComponentInterface[]): Promise<Ticket>;
  generateImplementationPlan(ticketId: string): Promise<ImplementationPlan>;
  pickupWork(ticketId: string, assignee: string): Promise<Ticket>;
  completeWork(ticketId: string): Promise<Ticket>;
  
  // Semantic operations
  analyzeSemantics(ticketId: string): Promise<Ticket['semanticDescription']>;
  validateImplementation(ticketId: string, implementation: string): Promise<SemanticConfidence>;
}

/**
 * Work delegation context for async processing
 */
export interface WorkDelegationContext {
  availableEngineers: Array<{
    id: string;
    name: string;
    skills: string[];
    currentWorkload: number; // 0-1 capacity utilization
  }>;
  queueMetrics: {
    totalTickets: number;
    availableWork: number;
    inProgress: number;
    avgCompletionTime: number; // in hours
  };
}

/**
 * Type guards for work refinement states
 */
export const isFeature = (ticket: Ticket): ticket is Feature => 
  ticket.refinementState === 'feature';

export const isImplementable = (ticket: Ticket): boolean =>
  ticket.refinementState === 'implementable' && !!ticket.interfaces;

export const hasImplementationPlan = (ticket: Ticket): boolean =>
  ticket.refinementState === 'planned' && !!ticket.implementationPlan;

/**
 * Validation schemas (using Zod for runtime validation)
 */
export const TicketStateTransitions: Record<WorkRefinementState, WorkRefinementState[]> = {
  'feature': ['refined'],
  'refined': ['implementable'],
  'implementable': ['planned'],
  'planned': ['in_progress'],
  'in_progress': ['completed'],
  'completed': ['verified'],
  'verified': []
};