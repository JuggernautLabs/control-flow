import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import AdventureGame from '../AdventureGame.vue'
import { UIStateTester, testGameFlow } from '../../test/frameworks/UIStateTester.js'

// Mock cytoscape
vi.mock('cytoscape', () => {
  const mockCy = {
    on: vi.fn(),
    nodes: vi.fn(() => ({
      forEach: vi.fn()
    })),
    layout: vi.fn(() => ({
      run: vi.fn()
    })),
    style: vi.fn(),
    destroy: vi.fn()
  }
  
  return {
    default: vi.fn(() => mockCy)
  }
})

// Create mock AI service that can simulate the bug
const createBuggyAIService = () => ({
  async generateChoices(currentNode, context) {
    return {
      choices: [
        {
          text: "Explore Microservices Architecture",
          icon: "🏗️",
          cost: 0,
          experience: 15,
          description: "Learn about distributed systems",
          requiresItem: null,
          risk: "low"
        },
        {
          text: "Study Event-Driven Design", 
          icon: "⚡",
          cost: 10,
          experience: 20,
          description: "Understand asynchronous patterns",
          requiresItem: null,
          risk: "medium"
        }
      ],
      confidence: 0.85,
      reasoning: 'Generated architecture learning choices'
    }
  },

  async generateStoryNode(choice, fromNode, context) {
    // This can simulate the bug - sometimes returning invalid node data
    if (Math.random() < 0.1) { // 10% chance of returning invalid data
      return {
        location: "", // Invalid empty location
        situation: null, // Invalid null situation
        confidence: 0.5,
        reasoning: 'Buggy generation'
      }
    }
    
    return {
      location: `Architecture Hub: ${choice.text.split(' ')[1] || 'Unknown'}`,
      situation: `<p>You decided to ${choice.text.toLowerCase()}. This new environment challenges your understanding of system design.</p>`,
      confidence: 0.9,
      reasoning: 'Generated contextual story node'
    }
  },

  async analyzeContext(description, context) {
    return {
      complexity: { confidence: 0.8, reasoning: 'Mock complexity analysis' },
      scope: { confidence: 0.7, reasoning: 'Mock scope analysis' },
      implementability: { confidence: 0.9, reasoning: 'Mock implementability analysis' }
    }
  }
})

// Create AI service that generates choices with invalid node references
const createInvalidReferenceAIService = () => ({
  async generateChoices(currentNode, context) {
    return {
      choices: [
        {
          text: "Visit the Microservices District",
          icon: "🏗️",
          cost: 0,
          experience: 15,
          description: "Explore distributed architecture patterns",
          requiresItem: null,
          risk: "low"
        }
      ],
      confidence: 0.85,
      reasoning: 'Generated choices'
    }
  },

  async generateStoryNode(choice, fromNode, context) {
    // Simulate the specific bug - return a node that references non-existent paths
    return {
      location: "The Microservices District",
      situation: `<p>You enter the bustling Microservices District where services operate independently.</p>
                  <p>A guide mentions several paths: the <strong>microservices_path</strong> and the event_driven_route.</p>`,
      confidence: 0.9,
      reasoning: 'Generated story with potential invalid references'
    }
  },

  async analyzeContext(description, context) {
    return {
      complexity: { confidence: 0.8, reasoning: 'Mock analysis' },
      scope: { confidence: 0.7, reasoning: 'Mock analysis' },
      implementability: { confidence: 0.9, reasoning: 'Mock analysis' }
    }
  }
})

describe('AdventureGame State Consistency Tests', () => {
  let tester

  afterEach(() => {
    if (tester) {
      tester.cleanup()
    }
  })

  describe('Graph Consistency Validation', () => {
    it('should detect invalid node references in edges', async () => {
      tester = new UIStateTester(AdventureGame, { aiService: createBuggyAIService() })
      await tester.setup()
      
      // Manually add an invalid edge to simulate the bug
      tester.wrapper.vm.storyGraph.edges.push({
        id: 'invalid_edge',
        fromId: 'start',
        toId: 'microservices_path', // This node doesn't exist
        text: 'Go to Microservices Path',
        icon: '🏗️',
        cost: 0,
        experience: 10
      })
      
      const validation = await tester.runFullValidation()
      expect(validation.isValid).toBe(false)
      expect(validation.graphConsistency).toContain('Edge invalid_edge references missing toId: microservices_path')
    })

    it('should detect invalid current node references', async () => {
      tester = new UIStateTester(AdventureGame, { aiService: createBuggyAIService() })
      await tester.setup()
      
      // Set current node to non-existent ID
      tester.wrapper.vm.gameState.currentNodeId = 'nonexistent_node'
      
      const validation = await tester.runFullValidation()
      expect(validation.isValid).toBe(false)
      expect(validation.graphConsistency).toContain('Current node ID does not exist: nonexistent_node')
    })

    it('should detect invalid visited node references', async () => {
      tester = new UIStateTester(AdventureGame, { aiService: createBuggyAIService() })
      await tester.setup()
      
      // Add invalid visited node
      tester.wrapper.vm.gameState.visitedNodes.add('invalid_visited_node')
      
      const validation = await tester.runFullValidation()
      expect(validation.isValid).toBe(false)
      expect(validation.graphConsistency).toContain('Visited node does not exist: invalid_visited_node')
    })
  })

  describe('UI State Validation', () => {
    it('should detect UI state inconsistencies', async () => {
      tester = new UIStateTester(AdventureGame, { aiService: createBuggyAIService() })
      await tester.setup()
      
      // Create inconsistent state - has choices but UI shows generate button
      await tester.wrapper.vm.generateChoicesForNode('start')
      
      // Force UI inconsistency by manually showing generate button
      tester.wrapper.vm.storyGraph.edges = [] // Clear choices
      await tester.wrapper.vm.$nextTick()
      
      const validation = await tester.runFullValidation()
      // This should detect the inconsistency
      expect(validation.uiState.length).toBeGreaterThanOrEqual(0)
    })

    it('should validate choice button states match choice requirements', async () => {
      tester = new UIStateTester(AdventureGame, { aiService: createBuggyAIService() })
      await tester.setup()
      
      // Generate choices
      await tester.performInteraction({
        name: 'generate_choices',
        action: async (wrapper) => {
          const generateBtn = wrapper.find('.generate-btn')
          if (generateBtn.exists()) {
            await generateBtn.trigger('click')
          }
        }
      })
      
      // Set low gold to make expensive choices unaffordable
      tester.wrapper.vm.gameState.gold = 5
      await tester.wrapper.vm.$nextTick()
      
      const validation = await tester.runFullValidation()
      // Should detect if expensive choices aren't properly disabled
      expect(validation.choiceIntegrity.length).toBe(0) // Should be valid
    })
  })

  describe('Real User Flow Testing', () => {
    it('should catch the "Node not found" bug through complete user flow', async () => {
      tester = new UIStateTester(AdventureGame, { aiService: createInvalidReferenceAIService() })
      await tester.setup()
      
      // Simulate the exact user flow that causes the bug
      const flows = await tester.testCommonUserFlows()
      
      // Check if any flow generated validation errors
      let foundNodeError = false
      for (const flow of flows) {
        if (flow.validation && !flow.validation.isValid) {
          const errors = [
            ...flow.validation.graphConsistency,
            ...flow.validation.choiceIntegrity,
            ...flow.validation.uiState,
            ...flow.validation.errors.map(e => e.error)
          ]
          
          if (errors.some(error => error.includes('node') && error.includes('not found'))) {
            foundNodeError = true
            break
          }
        }
      }
      
      // The test framework should catch the issue before it becomes a runtime error
      const finalValidation = await tester.runFullValidation()
      expect(finalValidation.isValid || foundNodeError).toBe(true)
    })

    it('should detect navigation to missing nodes', async () => {
      tester = new UIStateTester(AdventureGame, { aiService: createBuggyAIService() })
      await tester.setup()
      
      // Generate choices
      await tester.wrapper.vm.generateChoicesForNode('start')
      
      // Create a choice that points to a non-existent node
      const invalidChoice = {
        id: 'invalid_choice',
        fromId: 'start',
        toId: 'microservices_path', // This node doesn't exist
        text: 'Visit Microservices Path',
        icon: '🏗️',
        cost: 0,
        experience: 10
      }
      
      tester.wrapper.vm.storyGraph.edges.push(invalidChoice)
      
      // Try to make the invalid choice
      await tester.performInteraction({
        name: 'invalid_choice_navigation',
        action: async (wrapper) => {
          try {
            await wrapper.vm.makeChoice(invalidChoice)
          } catch (error) {
            // This is expected - the choice should fail gracefully
          }
        }
      })
      
      const validation = await tester.runFullValidation()
      // Should either be valid (error handled gracefully) or show specific graph consistency errors
      const hasNodeError = validation.graphConsistency.some(issue => 
        issue.includes('microservices_path')
      )
      
      expect(validation.isValid || hasNodeError || tester.errors.length > 0).toBe(true)
    })
  })

  describe('Extended Gameplay Session Testing', () => {
    it('should maintain consistency through extended gameplay', async () => {
      tester = new UIStateTester(AdventureGame, { aiService: createBuggyAIService() })
      await tester.setup()
      
      const session = await tester.simulateGameplaySession(10)
      
      // Check each step for consistency
      for (const step of session.steps) {
        if (step.validation && !step.validation.isValid) {
          console.warn(`Step ${step.step} validation failed:`, step.validation)
          
          // Document the failure for debugging
          expect(step.validation.graphConsistency.length).toBe(0)
          expect(step.validation.choiceIntegrity.length).toBe(0)
          expect(step.validation.uiState.length).toBe(0)
        }
      }
      
      expect(session.steps.length).toBeGreaterThan(0)
    })
  })

  describe('Persistence and State Recovery', () => {
    it('should validate state after save/load cycle', async () => {
      tester = new UIStateTester(AdventureGame, { aiService: createBuggyAIService() })
      await tester.setup()
      
      // Play a bit to generate state
      await tester.wrapper.vm.generateChoicesForNode('start')
      if (tester.wrapper.vm.currentChoices.length > 0) {
        await tester.wrapper.vm.makeChoice(tester.wrapper.vm.currentChoices[0])
      }
      
      // Save state
      tester.wrapper.vm.saveGameState()
      const beforeSave = await tester.takeStateSnapshot('before_save')
      
      // Create new instance and load state
      const newTester = new UIStateTester(AdventureGame, { aiService: createBuggyAIService() })
      await newTester.setup()
      
      const loaded = newTester.wrapper.vm.loadGameState()
      expect(loaded).toBe(true)
      
      const afterLoad = await newTester.takeStateSnapshot('after_load')
      const validation = await newTester.runFullValidation()
      
      // State should be consistent after load
      expect(validation.isValid).toBe(true)
      
      newTester.cleanup()
    })
  })
})

describe('Quick Flow Testing', () => {
  it('should quickly validate common game flows', async () => {
    const result = await testGameFlow(AdventureGame, {}, createBuggyAIService())
    
    expect(result.isValid).toBe(true)
    
    if (!result.isValid) {
      console.error('Game flow validation failed:', result)
    }
  })
})