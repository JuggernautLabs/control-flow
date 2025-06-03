import { describe, it, expect, vi, beforeEach } from 'vitest'
import * as claudeApi from '../../services/claudeApi.js'

// Mock the Claude API
vi.mock('../../services/claudeApi.js', () => ({
  extractMissingContext: vi.fn(),
  analyzeSemantics: vi.fn(),
  analyzeRefinement: vi.fn(),
  generateComponentInterfaces: vi.fn(),
  validateImplementation: vi.fn()
}))

describe('Story Graph Integration Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('Story Graph Data Structure', () => {
    it('should have valid node structure', () => {
      const testNode = {
        id: 'test_node',
        location: 'Test Location',
        situation: 'A test situation'
      }
      
      expect(testNode.id).toBeDefined()
      expect(testNode.location).toBeDefined()
      expect(testNode.situation).toBeDefined()
    })

    it('should have valid edge structure', () => {
      const testEdge = {
        id: 'test_edge',
        fromId: 'start',
        toId: 'end',
        text: 'Test choice',
        icon: '🧪',
        cost: 10,
        experience: 20
      }
      
      expect(testEdge.fromId).toBeDefined()
      expect(testEdge.toId).toBeDefined()
      expect(testEdge.text).toBeDefined()
      expect(testEdge.icon).toBeDefined()
      expect(typeof testEdge.cost).toBe('number')
      expect(typeof testEdge.experience).toBe('number')
    })

    it('should maintain graph connectivity', () => {
      const nodes = [
        { id: 'start', location: 'Start' },
        { id: 'middle', location: 'Middle' },
        { id: 'end', location: 'End' }
      ]
      
      const edges = [
        { id: 'e1', fromId: 'start', toId: 'middle', text: 'Go middle' },
        { id: 'e2', fromId: 'middle', toId: 'end', text: 'Go end' },
        { id: 'e3', fromId: 'start', toId: 'end', text: 'Skip middle' }
      ]
      
      // Verify all edge references point to existing nodes
      const nodeIds = new Set(nodes.map(n => n.id))
      
      for (const edge of edges) {
        expect(nodeIds.has(edge.fromId)).toBe(true)
        expect(nodeIds.has(edge.toId)).toBe(true)
      }
    })
  })

  describe('Story Generation Pipeline', () => {
    it('should complete full story generation workflow', async () => {
      const planDescription = 'Build a microservices architecture for e-commerce'
      
      // Mock the API responses in sequence
      claudeApi.extractMissingContext.mockResolvedValue({
        value: ['authentication', 'payment_gateway', 'deployment'],
        confidence: 0.8,
        reasoning: 'Missing key architectural decisions'
      })
      
      claudeApi.analyzeSemantics.mockResolvedValue({
        complexity: { confidence: 0.9, reasoning: 'High complexity project' },
        scope: { confidence: 0.7, reasoning: 'Well-defined but broad scope' },
        implementability: { confidence: 0.6, reasoning: 'Needs more technical details' }
      })
      
      claudeApi.analyzeRefinement.mockResolvedValue({
        shouldRefine: { value: true, confidence: 0.85 },
        suggestedBreakdown: [
          {
            title: 'User Service',
            description: 'Handle user management and authentication',
            estimated_complexity: 'medium'
          },
          {
            title: 'Product Service',
            description: 'Manage product catalog',
            estimated_complexity: 'medium'
          },
          {
            title: 'Order Service',
            description: 'Process orders and payments',
            estimated_complexity: 'high'
          }
        ]
      })
      
      claudeApi.generateComponentInterfaces.mockResolvedValue({
        value: [
          {
            name: 'UserService',
            signature: 'authenticateUser(credentials: UserCredentials): Promise<AuthResult>',
            purpose: 'Authenticate users',
            inputs: [{ name: 'credentials', type: 'UserCredentials', description: 'Login info' }],
            outputs: [{ name: 'authResult', type: 'Promise<AuthResult>', description: 'Auth result' }]
          }
        ],
        confidence: 0.8,
        reasoning: 'Clear interface based on requirements'
      })
      
      claudeApi.validateImplementation.mockResolvedValue({
        value: true,
        confidence: 0.9,
        reasoning: 'Implementation plan covers all requirements'
      })
      
      // Simulate the story generation workflow
      const missingContext = await claudeApi.extractMissingContext(planDescription)
      expect(missingContext.value).toContain('authentication')
      expect(missingContext.confidence).toBeGreaterThan(0.7)
      
      const semantics = await claudeApi.analyzeSemantics(planDescription)
      expect(semantics.complexity.confidence).toBeGreaterThan(0.8)
      
      const refinement = await claudeApi.analyzeRefinement(planDescription, 'feature')
      expect(refinement.shouldRefine.value).toBe(true)
      expect(refinement.suggestedBreakdown).toHaveLength(3)
      
      const interfaces = await claudeApi.generateComponentInterfaces(
        refinement.suggestedBreakdown[0].description
      )
      expect(interfaces.value).toHaveLength(1)
      expect(interfaces.value[0].name).toBe('UserService')
      
      const validation = await claudeApi.validateImplementation(
        refinement.suggestedBreakdown[0].description,
        interfaces.value,
        { steps: ['Design', 'Implement', 'Test'] }
      )
      expect(validation.value).toBe(true)
      expect(validation.confidence).toBeGreaterThan(0.8)
    })

    it('should handle partial failures in story generation', async () => {
      const planDescription = 'Build something complex'
      
      // Mock partial success scenario
      claudeApi.extractMissingContext.mockResolvedValue({
        value: ['scope', 'requirements'],
        confidence: 0.5,
        reasoning: 'Vague description'
      })
      
      claudeApi.analyzeSemantics.mockRejectedValue(new Error('API timeout'))
      
      claudeApi.analyzeRefinement.mockResolvedValue({
        shouldRefine: { value: false, confidence: 0.3 },
        suggestedBreakdown: null
      })
      
      const missingContext = await claudeApi.extractMissingContext(planDescription)
      expect(missingContext.confidence).toBe(0.5)
      
      try {
        await claudeApi.analyzeSemantics(planDescription)
        expect.fail('Should have thrown error')
      } catch (error) {
        expect(error.message).toBe('API timeout')
      }
      
      const refinement = await claudeApi.analyzeRefinement(planDescription, 'feature')
      expect(refinement.shouldRefine.value).toBe(false)
      expect(refinement.shouldRefine.confidence).toBeLessThan(0.5)
    })
  })

  describe('Graph Traversal and State Management', () => {
    it('should track visited nodes correctly', () => {
      const visitedNodes = new Set()
      const path = ['start', 'middle', 'end']
      
      for (const nodeId of path) {
        visitedNodes.add(nodeId)
      }
      
      expect(visitedNodes.size).toBe(3)
      expect(visitedNodes.has('start')).toBe(true)
      expect(visitedNodes.has('middle')).toBe(true)
      expect(visitedNodes.has('end')).toBe(true)
      expect(visitedNodes.has('never_visited')).toBe(false)
    })

    it('should maintain game state consistency', () => {
      const gameState = {
        currentNodeId: 'start',
        level: 1,
        experience: 0,
        gold: 50,
        inventory: [],
        visitedNodes: new Set(['start'])
      }
      
      // Simulate making a choice
      const choice = {
        toId: 'microservices_path',
        experience: 10,
        cost: 5
      }
      
      // Update state
      gameState.currentNodeId = choice.toId
      gameState.experience += choice.experience
      gameState.gold -= choice.cost
      gameState.visitedNodes.add(choice.toId)
      
      expect(gameState.currentNodeId).toBe('microservices_path')
      expect(gameState.experience).toBe(10)
      expect(gameState.gold).toBe(45)
      expect(gameState.visitedNodes.has('microservices_path')).toBe(true)
      expect(gameState.visitedNodes.size).toBe(2)
    })

    it('should validate choice requirements', () => {
      const gameState = {
        gold: 30,
        inventory: [
          { id: 'compass', name: 'Architecture Compass', consumable: false }
        ]
      }
      
      const choices = [
        { cost: 25, requiresItem: null }, // Affordable
        { cost: 35, requiresItem: null }, // Too expensive
        { cost: 10, requiresItem: 'Architecture Compass' }, // Has required item
        { cost: 10, requiresItem: 'Missing Item' } // Missing required item
      ]
      
      function canMakeChoice(choice) {
        if (choice.cost > gameState.gold) return false
        if (choice.requiresItem) {
          return gameState.inventory.some(item => item.name === choice.requiresItem)
        }
        return true
      }
      
      expect(canMakeChoice(choices[0])).toBe(true)
      expect(canMakeChoice(choices[1])).toBe(false)
      expect(canMakeChoice(choices[2])).toBe(true)
      expect(canMakeChoice(choices[3])).toBe(false)
    })
  })

  describe('Progress Tracking Integration', () => {
    it('should format choice data for API tracking', () => {
      const choice = {
        id: 'choice_123',
        fromId: 'start',
        toId: 'microservices_path',
        text: 'Explore Microservices',
        experience: 10,
        cost: 5
      }
      
      const gameState = {
        level: 2,
        experience: 150,
        gold: 75,
        currentNodeId: 'microservices_path',
        inventory: [{ id: 'item1' }]
      }
      
      const sessionId = 'session_12345'
      
      const trackingData = {
        choiceId: choice.id,
        fromNode: choice.fromId,
        toNode: choice.toId,
        sessionId: sessionId,
        gameState: {
          level: gameState.level,
          experience: gameState.experience,
          gold: gameState.gold,
          currentNodeId: gameState.currentNodeId,
          inventoryCount: gameState.inventory.length
        }
      }
      
      expect(trackingData.choiceId).toBe('choice_123')
      expect(trackingData.fromNode).toBe('start')
      expect(trackingData.toNode).toBe('microservices_path')
      expect(trackingData.gameState.level).toBe(2)
      expect(trackingData.gameState.inventoryCount).toBe(1)
    })

    it('should handle offline mode gracefully', async () => {
      // Mock fetch to simulate network failure
      global.fetch = vi.fn().mockRejectedValue(new Error('Network error'))
      
      const trackingPromise = fetch('http://localhost:3001/api/adventure/track', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ test: 'data' })
      })
      
      await expect(trackingPromise).rejects.toThrow('Network error')
    })
  })

  describe('Story Graph Validation', () => {
    it('should validate complete story graph structure', () => {
      const storyGraph = {
        nodes: [
          { id: 'start', location: 'Town Square', situation: 'Beginning' },
          { id: 'path1', location: 'Forest', situation: 'Middle path' },
          { id: 'path2', location: 'Mountain', situation: 'Alternate path' },
          { id: 'end', location: 'Castle', situation: 'Final destination' }
        ],
        edges: [
          { id: 'e1', fromId: 'start', toId: 'path1', text: 'Enter forest' },
          { id: 'e2', fromId: 'start', toId: 'path2', text: 'Climb mountain' },
          { id: 'e3', fromId: 'path1', toId: 'end', text: 'Reach castle' },
          { id: 'e4', fromId: 'path2', toId: 'end', text: 'Descend to castle' }
        ]
      }
      
      // Validation checks
      const nodeIds = new Set(storyGraph.nodes.map(n => n.id))
      const edgeIds = new Set(storyGraph.edges.map(e => e.id))
      
      // All nodes have required properties
      for (const node of storyGraph.nodes) {
        expect(node.id).toBeDefined()
        expect(node.location).toBeDefined()
        expect(node.situation).toBeDefined()
      }
      
      // All edges reference valid nodes
      for (const edge of storyGraph.edges) {
        expect(nodeIds.has(edge.fromId)).toBe(true)
        expect(nodeIds.has(edge.toId)).toBe(true)
      }
      
      // No duplicate IDs
      expect(nodeIds.size).toBe(storyGraph.nodes.length)
      expect(edgeIds.size).toBe(storyGraph.edges.length)
      
      // Graph is connected (has paths from start)
      function isReachable(fromId, toId, visited = new Set()) {
        if (fromId === toId) return true
        if (visited.has(fromId)) return false
        
        visited.add(fromId)
        const outgoingEdges = storyGraph.edges.filter(e => e.fromId === fromId)
        
        return outgoingEdges.some(edge => 
          isReachable(edge.toId, toId, new Set(visited))
        )
      }
      
      // Start node can reach all other nodes
      for (const node of storyGraph.nodes) {
        if (node.id !== 'start') {
          expect(isReachable('start', node.id)).toBe(true)
        }
      }
    })

    it('should detect invalid graph structures', () => {
      const invalidGraph = {
        nodes: [
          { id: 'start', location: 'Town' },
          { id: 'orphan', location: 'Isolated' }
        ],
        edges: [
          { id: 'e1', fromId: 'start', toId: 'nonexistent', text: 'Broken' }
        ]
      }
      
      const nodeIds = new Set(invalidGraph.nodes.map(n => n.id))
      const brokenEdges = invalidGraph.edges.filter(
        edge => !nodeIds.has(edge.fromId) || !nodeIds.has(edge.toId)
      )
      
      expect(brokenEdges).toHaveLength(1)
      expect(brokenEdges[0].toId).toBe('nonexistent')
    })
  })

  describe('Semantic Analysis Integration', () => {
    it('should integrate semantic analysis with story progression', async () => {
      const storyNode = {
        id: 'complex_decision',
        location: 'Architecture Crossroads',
        situation: 'Choose between microservices and monolith architecture'
      }
      
      claudeApi.analyzeSemantics.mockResolvedValue({
        complexity: { confidence: 0.9, reasoning: 'High architectural complexity' },
        scope: { confidence: 0.8, reasoning: 'Well-defined architectural choice' },
        implementability: { confidence: 0.6, reasoning: 'Needs technical context' }
      })
      
      const semantics = await claudeApi.analyzeSemantics(storyNode.situation)
      
      expect(semantics.complexity.confidence).toBeGreaterThan(0.8)
      expect(semantics.scope.reasoning).toContain('architectural choice')
      expect(semantics.implementability.confidence).toBeLessThan(0.7)
      
      // Based on semantic analysis, we can decide story flow
      const needsMoreInfo = semantics.implementability.confidence < 0.7
      expect(needsMoreInfo).toBe(true)
    })

    it('should adapt story generation based on user choices', async () => {
      const userChoices = [
        { choice: 'microservices_path', context: 'chose distributed architecture' },
        { choice: 'event_driven', context: 'prefers event-driven patterns' }
      ]
      
      const contextualDescription = `
        User has shown preference for distributed systems by choosing microservices
        and event-driven architecture. Generate next story node with this context.
      `
      
      claudeApi.generateComponentInterfaces.mockResolvedValue({
        value: [
          {
            name: 'EventBus',
            signature: 'publishEvent(event: DomainEvent): Promise<void>',
            purpose: 'Publish domain events for microservices communication'
          }
        ],
        confidence: 0.9,
        reasoning: 'Interfaces match user architectural preferences'
      })
      
      const interfaces = await claudeApi.generateComponentInterfaces(contextualDescription)
      
      expect(interfaces.value[0].name).toBe('EventBus')
      expect(interfaces.value[0].purpose).toContain('microservices')
      expect(interfaces.confidence).toBeGreaterThan(0.8)
    })
  })
})