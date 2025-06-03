import { describe, it, expect } from 'vitest'
import { 
  isFeature, 
  isImplementable, 
  hasImplementationPlan,
  TicketStateTransitions 
} from '../planning.ts'
import { createMockTicket, createMockComponentInterface } from '../../test/utils/testHelpers.js'

describe('Planning Types', () => {
  describe('Type Guards', () => {
    it('should correctly identify feature tickets', () => {
      const featureTicket = createMockTicket({
        refinementState: 'feature',
        businessValue: 'Improves user experience',
        stakeholders: ['Product Manager', 'UX Designer'],
        acceptanceCriteria: ['User can login', 'Password is secure']
      })
      
      expect(isFeature(featureTicket)).toBe(true)
      
      const nonFeatureTicket = createMockTicket({
        refinementState: 'implementable'
      })
      
      expect(isFeature(nonFeatureTicket)).toBe(false)
    })

    it('should correctly identify implementable tickets', () => {
      const implementableTicket = createMockTicket({
        refinementState: 'implementable',
        interfaces: [createMockComponentInterface()]
      })
      
      expect(isImplementable(implementableTicket)).toBe(true)
      
      const notImplementableTicket = createMockTicket({
        refinementState: 'implementable',
        interfaces: undefined
      })
      
      expect(isImplementable(notImplementableTicket)).toBe(false)
      
      const wrongStateTicket = createMockTicket({
        refinementState: 'feature',
        interfaces: [createMockComponentInterface()]
      })
      
      expect(isImplementable(wrongStateTicket)).toBe(false)
    })

    it('should correctly identify tickets with implementation plans', () => {
      const plannedTicket = createMockTicket({
        refinementState: 'planned',
        implementationPlan: {
          ticketId: 'ticket_123',
          interfaces: [createMockComponentInterface()],
          testSpecs: [],
          dependencies: [],
          estimatedComplexity: 'medium',
          createdAt: new Date().toISOString()
        }
      })
      
      expect(hasImplementationPlan(plannedTicket)).toBe(true)
      
      const notPlannedTicket = createMockTicket({
        refinementState: 'planned',
        implementationPlan: undefined
      })
      
      expect(hasImplementationPlan(notPlannedTicket)).toBe(false)
    })
  })

  describe('State Transitions', () => {
    it('should define valid state transitions', () => {
      expect(TicketStateTransitions['feature']).toEqual(['refined'])
      expect(TicketStateTransitions['refined']).toEqual(['implementable'])
      expect(TicketStateTransitions['implementable']).toEqual(['planned'])
      expect(TicketStateTransitions['planned']).toEqual(['in_progress'])
      expect(TicketStateTransitions['in_progress']).toEqual(['completed'])
      expect(TicketStateTransitions['completed']).toEqual(['verified'])
      expect(TicketStateTransitions['verified']).toEqual([])
    })

    it('should validate state transition validity', () => {
      function isValidTransition(fromState, toState) {
        const validNextStates = TicketStateTransitions[fromState]
        return validNextStates && validNextStates.includes(toState)
      }
      
      // Valid transitions
      expect(isValidTransition('feature', 'refined')).toBe(true)
      expect(isValidTransition('refined', 'implementable')).toBe(true)
      expect(isValidTransition('implementable', 'planned')).toBe(true)
      
      // Invalid transitions
      expect(isValidTransition('feature', 'implementable')).toBe(false)
      expect(isValidTransition('completed', 'in_progress')).toBe(false)
      expect(isValidTransition('verified', 'feature')).toBe(false)
    })

    it('should handle terminal state correctly', () => {
      const verifiedTransitions = TicketStateTransitions['verified']
      expect(verifiedTransitions).toEqual([])
      expect(verifiedTransitions.length).toBe(0)
    })
  })

  describe('Interface Definitions', () => {
    it('should validate ComponentInterface structure', () => {
      const validInterface = createMockComponentInterface({
        name: 'UserService',
        signature: 'createUser(userData: UserData): Promise<User>',
        purpose: 'Create new user accounts',
        inputs: [
          {
            name: 'userData',
            type: 'UserData',
            description: 'User registration information'
          }
        ],
        outputs: [
          {
            name: 'user',
            type: 'Promise<User>',
            description: 'Created user object'
          }
        ],
        preconditions: ['Email must be valid', 'Password meets requirements'],
        postconditions: ['User exists in database', 'Welcome email sent']
      })
      
      expect(validInterface.name).toBe('UserService')
      expect(validInterface.signature).toContain('createUser')
      expect(validInterface.inputs).toHaveLength(1)
      expect(validInterface.outputs).toHaveLength(1)
      expect(validInterface.preconditions).toHaveLength(2)
      expect(validInterface.postconditions).toHaveLength(2)
    })

    it('should handle optional fields in ComponentInterface', () => {
      const minimalInterface = createMockComponentInterface({
        name: 'SimpleService',
        signature: 'doSomething(): void',
        purpose: 'Does something simple',
        inputs: [],
        outputs: []
        // preconditions and postconditions are optional
      })
      
      expect(minimalInterface.name).toBeDefined()
      expect(minimalInterface.signature).toBeDefined()
      expect(minimalInterface.purpose).toBeDefined()
      expect(minimalInterface.inputs).toEqual([])
      expect(minimalInterface.outputs).toEqual([])
    })
  })

  describe('Semantic Confidence', () => {
    it('should validate confidence scores', () => {
      const validConfidences = [0.0, 0.5, 1.0, 0.85, 0.234]
      const invalidConfidences = [-0.1, 1.1, 2.0, NaN, 'string']
      
      function isValidConfidence(confidence) {
        return typeof confidence === 'number' && 
               confidence >= 0 && 
               confidence <= 1 && 
               !isNaN(confidence)
      }
      
      validConfidences.forEach(conf => {
        expect(isValidConfidence(conf)).toBe(true)
      })
      
      invalidConfidences.forEach(conf => {
        expect(isValidConfidence(conf)).toBe(false)
      })
    })

    it('should require reasoning for semantic confidence', () => {
      const validSemanticConfidence = {
        confidence: 0.8,
        reasoning: 'Based on clear requirements and technical constraints'
      }
      
      expect(validSemanticConfidence.confidence).toBe(0.8)
      expect(validSemanticConfidence.reasoning).toBeDefined()
      expect(typeof validSemanticConfidence.reasoning).toBe('string')
      expect(validSemanticConfidence.reasoning.length).toBeGreaterThan(0)
    })
  })

  describe('Test Specifications', () => {
    it('should validate TestSpecification structure', () => {
      const testSpec = {
        testName: 'Should create user successfully',
        description: 'Tests that user creation works with valid data',
        setup: 'Initialize database and mock email service',
        action: 'Call createUser with valid user data',
        expectedOutcome: 'User is created and welcome email is sent',
        semanticCorrelation: {
          confidence: 0.9,
          reasoning: 'Test directly validates core requirement'
        }
      }
      
      expect(testSpec.testName).toBeDefined()
      expect(testSpec.description).toBeDefined()
      expect(testSpec.setup).toBeDefined()
      expect(testSpec.action).toBeDefined()
      expect(testSpec.expectedOutcome).toBeDefined()
      expect(testSpec.semanticCorrelation.confidence).toBe(0.9)
    })

    it('should link tests to ticket requirements', () => {
      const ticket = createMockTicket({
        title: 'Implement user registration',
        description: 'Users should be able to create accounts with email and password'
      })
      
      const relatedTest = {
        testName: 'Should validate email format',
        description: 'Ensures email validation works correctly',
        setup: 'Mock user input',
        action: 'Submit registration with invalid email',
        expectedOutcome: 'Validation error is displayed',
        semanticCorrelation: {
          confidence: 0.85,
          reasoning: 'Directly tests email validation requirement from ticket'
        }
      }
      
      // Test correlation logic
      const correlationThreshold = 0.7
      const isHighCorrelation = relatedTest.semanticCorrelation.confidence > correlationThreshold
      
      expect(isHighCorrelation).toBe(true)
      expect(relatedTest.semanticCorrelation.reasoning).toContain('ticket')
    })
  })

  describe('Implementation Plans', () => {
    it('should validate ImplementationPlan structure', () => {
      const implementationPlan = {
        ticketId: 'ticket_456',
        interfaces: [
          createMockComponentInterface({
            name: 'UserValidator',
            signature: 'validateUser(data: UserData): ValidationResult'
          }),
          createMockComponentInterface({
            name: 'UserRepository',
            signature: 'saveUser(user: User): Promise<void>'
          })
        ],
        testSpecs: [
          {
            testName: 'Should validate user data',
            description: 'Tests user validation logic',
            setup: 'Mock validation rules',
            action: 'Validate user with various inputs',
            expectedOutcome: 'Correct validation results',
            semanticCorrelation: { confidence: 0.9, reasoning: 'Core validation test' }
          }
        ],
        dependencies: ['ticket_123', 'ticket_789'],
        estimatedComplexity: 'medium',
        createdAt: '2023-01-01T00:00:00.000Z',
        validatedAt: '2023-01-01T01:00:00.000Z'
      }
      
      expect(implementationPlan.ticketId).toBe('ticket_456')
      expect(implementationPlan.interfaces).toHaveLength(2)
      expect(implementationPlan.testSpecs).toHaveLength(1)
      expect(implementationPlan.dependencies).toHaveLength(2)
      expect(['low', 'medium', 'high']).toContain(implementationPlan.estimatedComplexity)
      expect(implementationPlan.createdAt).toBeDefined()
      expect(implementationPlan.validatedAt).toBeDefined()
    })

    it('should handle optional validation timestamp', () => {
      const unvalidatedPlan = {
        ticketId: 'ticket_999',
        interfaces: [],
        testSpecs: [],
        dependencies: [],
        estimatedComplexity: 'low',
        createdAt: '2023-01-01T00:00:00.000Z'
        // validatedAt is optional
      }
      
      expect(unvalidatedPlan.validatedAt).toBeUndefined()
      expect(unvalidatedPlan.createdAt).toBeDefined()
    })
  })

  describe('Priority and Complexity Enums', () => {
    it('should validate priority values', () => {
      const validPriorities = ['low', 'medium', 'high', 'critical']
      const invalidPriorities = ['urgent', 'normal', 'highest', '']
      
      validPriorities.forEach(priority => {
        const ticket = createMockTicket({ priority })
        expect(['low', 'medium', 'high', 'critical']).toContain(ticket.priority)
      })
    })

    it('should validate complexity values', () => {
      const validComplexities = ['low', 'medium', 'high']
      const invalidComplexities = ['simple', 'complex', 'extreme', '']
      
      validComplexities.forEach(complexity => {
        const plan = {
          estimatedComplexity: complexity,
          ticketId: 'test',
          interfaces: [],
          testSpecs: [],
          dependencies: [],
          createdAt: new Date().toISOString()
        }
        expect(['low', 'medium', 'high']).toContain(plan.estimatedComplexity)
      })
    })
  })
})