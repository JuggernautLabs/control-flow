import { reactive } from 'vue'
import * as claudeApi from '../services/claudeApi.js'

// Reactive store for ticket management with Claude API integration
export const ticketStore = reactive({
  tickets: [],
  nextId: 1,

  // Add new ticket/feature
  addTicket(ticketData) {
    const newTicket = {
      id: `ticket-${this.nextId++}`,
      title: ticketData.title,
      description: ticketData.description,
      refinementState: ticketData.type === 'feature' ? 'feature' : 'refined',
      semanticDescription: {
        complexity: { confidence: 0, reasoning: 'Not analyzed yet' },
        scope: { confidence: 0, reasoning: 'Not analyzed yet' },
        implementability: { confidence: 0, reasoning: 'Not analyzed yet' }
      },
      parentTicketId: ticketData.parentTicketId || null,
      childTicketIds: [],
      interfaces: [],
      assignedTo: null,
      pickupTimestamp: null,
      completionTimestamp: null,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      tags: ticketData.tags || [],
      priority: ticketData.priority || 'medium',
      // Feature-specific fields
      ...(ticketData.type === 'feature' && {
        businessValue: ticketData.businessValue || '',
        stakeholders: ticketData.stakeholders || [],
        acceptanceCriteria: ticketData.acceptanceCriteria || []
      })
    }
    
    this.tickets.push(newTicket)
    return newTicket
  },

  // Get ticket by ID
  getTicketById(id) {
    return this.tickets.find(ticket => ticket.id === id)
  },

  // Update ticket
  updateTicket(id, updates) {
    const ticket = this.getTicketById(id)
    if (ticket) {
      Object.assign(ticket, updates, { updatedAt: new Date().toISOString() })
    }
    return ticket
  },

  // Transition ticket state
  transitionTicketState(id, newState) {
    const ticket = this.getTicketById(id)
    if (ticket) {
      ticket.refinementState = newState
      ticket.updatedAt = new Date().toISOString()
      
      // Auto-assign timestamps for certain states
      if (newState === 'in_progress' && !ticket.pickupTimestamp) {
        ticket.pickupTimestamp = new Date().toISOString()
      }
      if (newState === 'completed' && !ticket.completionTimestamp) {
        ticket.completionTimestamp = new Date().toISOString()
      }
    }
    return ticket
  },

  // Run semantic analysis using Claude API
  async runSemanticAnalysis(id) {
    const ticket = this.getTicketById(id)
    if (!ticket) return null

    try {
      // Set loading state
      ticket.semanticDescription = {
        complexity: { confidence: 0, reasoning: 'Analyzing...' },
        scope: { confidence: 0, reasoning: 'Analyzing...' },
        implementability: { confidence: 0, reasoning: 'Analyzing...' }
      }

      const analysis = await claudeApi.analyzeSemantics(
        ticket.description,
        {
          title: ticket.title,
          refinementState: ticket.refinementState,
          tags: ticket.tags,
          priority: ticket.priority
        }
      )

      ticket.semanticDescription = analysis
      ticket.updatedAt = new Date().toISOString()
      return analysis
    } catch (error) {
      const errorAnalysis = {
        complexity: { confidence: 0.1, reasoning: `Analysis failed: ${error.message}` },
        scope: { confidence: 0.1, reasoning: `Analysis failed: ${error.message}` },
        implementability: { confidence: 0.1, reasoning: `Analysis failed: ${error.message}` }
      }
      ticket.semanticDescription = errorAnalysis
      ticket.updatedAt = new Date().toISOString()
      return errorAnalysis
    }
  },

  // Run refinement analysis using Claude API
  async analyzeRefinement(id) {
    const ticket = this.getTicketById(id)
    if (!ticket) return null

    try {
      const analysis = await claudeApi.analyzeRefinement(
        ticket.description,
        ticket.refinementState
      )
      
      return analysis
    } catch (error) {
      return {
        shouldRefine: {
          value: false,
          confidence: 0.1,
          reasoning: `Refinement analysis failed: ${error.message}`
        },
        suggestedBreakdown: null,
        refinementReasoning: `Error: ${error.message}`
      }
    }
  },

  // Generate component interfaces for implementable tickets
  async generateInterfaces(id) {
    const ticket = this.getTicketById(id)
    if (!ticket) return null

    try {
      const interfaceResult = await claudeApi.generateComponentInterfaces(
        ticket.description,
        {
          title: ticket.title,
          refinementState: ticket.refinementState,
          tags: ticket.tags,
          priority: ticket.priority
        }
      )

      // Update ticket with generated interfaces
      ticket.interfaces = interfaceResult.value
      ticket.updatedAt = new Date().toISOString()

      return interfaceResult
    } catch (error) {
      return {
        value: [],
        confidence: 0.1,
        reasoning: `Interface generation failed: ${error.message}`
      }
    }
  },

  // Generate implementation plan from interfaces
  async generateImplementationPlan(id) {
    const ticket = this.getTicketById(id)
    if (!ticket || !ticket.interfaces) return null

    try {
      // Create a basic implementation plan structure
      const plan = {
        ticketId: ticket.id,
        interfaces: ticket.interfaces,
        testSpecs: [], // Would generate test specs using Claude
        dependencies: [], // Would analyze dependencies
        estimatedComplexity: this.estimateComplexity(ticket),
        createdAt: new Date().toISOString()
      }

      ticket.implementationPlan = plan
      ticket.updatedAt = new Date().toISOString()

      return plan
    } catch (error) {
      return null
    }
  },

  // Validate implementation against original ticket
  async validateImplementation(id) {
    const ticket = this.getTicketById(id)
    if (!ticket || !ticket.interfaces) return null

    try {
      const validation = await claudeApi.validateImplementation(
        ticket.description,
        ticket.interfaces,
        ticket.implementationPlan
      )

      return validation
    } catch (error) {
      return {
        value: false,
        confidence: 0.1,
        reasoning: `Validation failed: ${error.message}`
      }
    }
  },

  // Helper method to estimate complexity
  estimateComplexity(ticket) {
    const interfaceCount = ticket.interfaces?.length || 0
    const descriptionLength = ticket.description.length

    if (interfaceCount > 5 || descriptionLength > 500) return 'high'
    if (interfaceCount > 2 || descriptionLength > 200) return 'medium'
    return 'low'
  },

  // Extract missing context for new tickets
  async extractMissingContext(id) {
    const ticket = this.getTicketById(id)
    if (!ticket) return null

    try {
      const contextResult = await claudeApi.extractMissingContext(ticket.description)
      
      // Could store this as additional context on the ticket
      ticket.missingContextAnalysis = contextResult
      ticket.updatedAt = new Date().toISOString()

      return contextResult
    } catch (error) {
      return {
        value: ['Error extracting context'],
        confidence: 0.1,
        reasoning: `Context extraction failed: ${error.message}`
      }
    }
  },

  // Get tickets by state
  getTicketsByState(state) {
    return this.tickets.filter(ticket => ticket.refinementState === state)
  },

  // Get available work (tickets ready to be picked up)
  getAvailableWork() {
    return this.tickets.filter(ticket => 
      ['implementable', 'planned'].includes(ticket.refinementState) && 
      !ticket.assignedTo
    )
  },

  // Create child tickets from refinement analysis
  createChildTickets(parentId, suggestedBreakdown) {
    const parentTicket = this.getTicketById(parentId)
    if (!parentTicket || !suggestedBreakdown) return []

    const childTickets = suggestedBreakdown.map(suggestion => {
      const childTicket = {
        id: `ticket-${this.nextId++}`,
        title: suggestion.title,
        description: suggestion.description,
        refinementState: 'refined',
        semanticDescription: {
          complexity: { 
            confidence: suggestion.estimatedComplexity === 'high' ? 0.9 : 
                        suggestion.estimatedComplexity === 'medium' ? 0.6 : 0.3,
            reasoning: `Estimated ${suggestion.estimatedComplexity} complexity from refinement`
          },
          scope: { confidence: 0.8, reasoning: 'Refined from parent ticket' },
          implementability: { confidence: 0.7, reasoning: 'Needs further analysis' }
        },
        parentTicketId: parentId,
        childTicketIds: [],
        interfaces: [],
        assignedTo: null,
        pickupTimestamp: null,
        completionTimestamp: null,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        tags: parentTicket.tags || [],
        priority: parentTicket.priority || 'medium'
      }
      return childTicket
    })

    // Add child tickets to store
    childTickets.forEach(child => this.tickets.push(child))

    // Update parent ticket with child IDs
    parentTicket.childTicketIds = childTickets.map(child => child.id)
    parentTicket.refinementState = 'refined'
    parentTicket.updatedAt = new Date().toISOString()

    return childTickets
  },

  // Create sample hierarchical data for testing
  createSampleHierarchy() {
    // Clear existing tickets
    this.tickets = []
    this.nextId = 1

    // Create a feature ticket
    const feature = this.addTicket({
      type: 'feature',
      title: 'User Authentication System',
      description: 'Complete user authentication system with registration, login, password reset, and role-based access control for the web application.',
      priority: 'high',
      tags: ['auth', 'security', 'backend', 'frontend'],
      businessValue: 'Enable secure user access and personalized experiences',
      stakeholders: ['Product Manager', 'Security Team', 'Engineering Team'],
      acceptanceCriteria: [
        'Users can register with email and password',
        'Users can login and logout securely',
        'Password reset functionality works via email',
        'Role-based permissions are enforced'
      ]
    })

    // Create child components
    const authBackend = this.addTicket({
      type: 'ticket',
      title: 'Authentication Backend Service',
      description: 'Backend API service for user authentication, including JWT token management, password hashing, and session handling.',
      priority: 'high',
      tags: ['backend', 'api', 'auth'],
      parentTicketId: feature.id
    })

    const authFrontend = this.addTicket({
      type: 'ticket', 
      title: 'Authentication Frontend Components',
      description: 'React components for login, registration, and password reset forms with proper validation and error handling.',
      priority: 'high',
      tags: ['frontend', 'react', 'auth'],
      parentTicketId: feature.id
    })

    const userDatabase = this.addTicket({
      type: 'ticket',
      title: 'User Database Schema',
      description: 'Database tables and relationships for users, roles, permissions, and authentication tokens.',
      priority: 'medium',
      tags: ['database', 'schema'],
      parentTicketId: feature.id
    })

    // Update feature with child IDs
    feature.childTicketIds = [authBackend.id, authFrontend.id, userDatabase.id]

    // Create sub-components for backend service
    const jwtService = this.addTicket({
      type: 'ticket',
      title: 'JWT Token Service',
      description: 'Service for creating, validating, and refreshing JWT authentication tokens.',
      priority: 'high',
      tags: ['backend', 'jwt', 'security'],
      parentTicketId: authBackend.id
    })

    const passwordService = this.addTicket({
      type: 'ticket',
      title: 'Password Hashing Service', 
      description: 'Secure password hashing and validation using bcrypt with proper salt rounds.',
      priority: 'high',
      tags: ['backend', 'security', 'crypto'],
      parentTicketId: authBackend.id
    })

    // Update backend with child IDs
    authBackend.childTicketIds = [jwtService.id, passwordService.id]
    authBackend.refinementState = 'refined'

    // Add some interfaces to implementable tickets
    jwtService.refinementState = 'implementable'
    jwtService.interfaces = [
      {
        name: 'generateToken',
        signature: 'generateToken(userId: string, expiresIn?: string): Promise<string>',
        purpose: 'Generate a new JWT token for authenticated user',
        inputs: [
          { name: 'userId', type: 'string', description: 'Unique user identifier' },
          { name: 'expiresIn', type: 'string', description: 'Token expiration time (default: 24h)' }
        ],
        outputs: [
          { name: 'token', type: 'string', description: 'Signed JWT token' }
        ],
        preconditions: ['User must be authenticated', 'userId must be valid'],
        postconditions: ['Token is cryptographically signed', 'Token contains user claims']
      },
      {
        name: 'validateToken',
        signature: 'validateToken(token: string): Promise<UserClaims | null>',
        purpose: 'Validate and decode a JWT token',
        inputs: [
          { name: 'token', type: 'string', description: 'JWT token to validate' }
        ],
        outputs: [
          { name: 'claims', type: 'UserClaims | null', description: 'Decoded user claims or null if invalid' }
        ]
      }
    ]

    passwordService.refinementState = 'implementable'
    passwordService.interfaces = [
      {
        name: 'hashPassword',
        signature: 'hashPassword(plaintext: string): Promise<string>',
        purpose: 'Hash a plaintext password securely',
        inputs: [
          { name: 'plaintext', type: 'string', description: 'Raw password string' }
        ],
        outputs: [
          { name: 'hash', type: 'string', description: 'Bcrypt hashed password' }
        ]
      }
    ]

    // Add semantic analysis to some tickets
    feature.semanticDescription = {
      complexity: { confidence: 0.85, reasoning: 'High complexity due to security requirements and multiple components' },
      scope: { confidence: 0.9, reasoning: 'Well-defined scope with clear acceptance criteria' },
      implementability: { confidence: 0.3, reasoning: 'Requires decomposition into implementable components' }
    }

    jwtService.semanticDescription = {
      complexity: { confidence: 0.7, reasoning: 'Moderate complexity with standard JWT operations' },
      scope: { confidence: 0.95, reasoning: 'Clearly defined JWT service responsibilities' },
      implementability: { confidence: 0.9, reasoning: 'Ready for implementation with defined interfaces' }
    }
  }
})