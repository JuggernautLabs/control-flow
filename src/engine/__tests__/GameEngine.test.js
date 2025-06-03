import { describe, it, expect, vi, beforeEach } from 'vitest'
import { GameEngine } from '../GameEngine.js'

// Mock AI service for testing
const createMockAIService = () => ({
  async generateChoices(currentNode, context) {
    return {
      choices: [
        {
          text: "Learn about Microservices",
          icon: "🏗️",
          cost: 0,
          experience: 15,
          description: "Explore distributed architecture patterns",
          requiresItem: null,
          risk: "low"
        },
        {
          text: "Study Database Design", 
          icon: "🗄️",
          cost: 10,
          experience: 20,
          description: "Master data persistence strategies",
          requiresItem: null,
          risk: "medium"
        },
        {
          text: "Visit the Knowledge Shop",
          icon: "🏪", 
          cost: 5,
          experience: 5,
          description: "Buy helpful architectural tools",
          requiresItem: null,
          risk: "low"
        }
      ],
      confidence: 0.85,
      reasoning: 'Generated architectural learning choices'
    }
  },

  async generateStoryNode(choice, fromNode, context) {
    const scenarios = {
      microservices: {
        location: "The Distributed District",
        situation: `<p>You enter the bustling <strong>Distributed District</strong> where microservices operate independently.</p>`
      },
      database: {
        location: "The Data Vault",
        situation: `<p>You descend into the <strong>Data Vault</strong> where information is carefully organized and stored.</p>`
      },
      shop: {
        location: "The Knowledge Shop",
        situation: `<p>Inside the mystical shop, scrolls and artifacts glow with architectural wisdom.</p>`
      }
    }
    
    let scenario = scenarios.microservices
    if (choice.text.toLowerCase().includes('database')) {
      scenario = scenarios.database
    } else if (choice.text.toLowerCase().includes('shop')) {
      scenario = scenarios.shop
    }

    return {
      location: scenario.location,
      situation: scenario.situation,
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

describe('GameEngine Core Functionality', () => {
  let engine
  let mockAIService
  let events

  beforeEach(() => {
    mockAIService = createMockAIService()
    engine = new GameEngine(mockAIService, { debugMode: true, autoFix: false }) // Disable auto-fix in tests
    events = []
    
    engine.addListener((event) => {
      events.push(event)
    })
  })

  describe('Initialization', () => {
    it('should initialize with correct default state', () => {
      const state = engine.getState()
      
      expect(state.currentNodeId).toBe('start')
      expect(state.phase).toBe('waiting_for_choices')
      expect(state.level).toBe(1)
      expect(state.experience).toBe(0)
      expect(state.gold).toBe(50)
      expect(state.inventory).toEqual([])
      expect(state.isGameOver).toBe(false)
      expect(state.generationInProgress).toBe(false)
      expect(state.storyGraph.nodes.length).toBe(1)
      expect(state.storyGraph.edges.length).toBe(0)
    })

    it('should validate initial state as correct', () => {
      const validation = engine.validateState()
      expect(validation.valid).toBe(true)
    })

    it('should have current node accessible', () => {
      const currentNode = engine.getCurrentNode()
      expect(currentNode).toBeDefined()
      expect(currentNode.id).toBe('start')
      expect(currentNode.location).toBe('The Town Square of Architectura')
    })

    it('should have no choices initially', () => {
      const choices = engine.getCurrentChoices()
      expect(choices).toEqual([])
    })
  })

  describe('State Validation', () => {
    it('should detect invalid current node', () => {
      engine.state.currentNodeId = 'nonexistent'
      const validation = engine.validateState()
      
      expect(validation.valid).toBe(false)
      expect(validation.rules.currentNodeExists.valid).toBe(false)
      expect(validation.rules.currentNodeExists.message).toContain('not found')
    })

    it('should detect invalid edges', () => {
      engine.state.storyGraph.edges.push({
        id: 'invalid',
        fromId: 'start',
        toId: 'nonexistent',
        text: 'Invalid choice'
      })
      
      const validation = engine.validateState()
      expect(validation.valid).toBe(false)
      expect(validation.rules.edgesValid.valid).toBe(false)
    })

    it('should detect invalid visited nodes', () => {
      engine.state.visitedNodes.add('nonexistent')
      const validation = engine.validateState()
      
      expect(validation.valid).toBe(false)
      expect(validation.rules.visitedNodesExist.valid).toBe(false)
    })

    it('should detect invalid phase', () => {
      engine.state.phase = 'invalid_phase'
      const validation = engine.validateState()
      
      expect(validation.valid).toBe(false)
      expect(validation.rules.phaseConsistent.valid).toBe(false)
    })
  })

  describe('Advancement Conditions', () => {
    it('should allow advancement when waiting for choices', () => {
      const canAdvance = engine.canAdvance()
      expect(canAdvance.canAdvance).toBe(true)
      expect(canAdvance.reasons.validState).toBe(true)
      expect(canAdvance.reasons.hasChoicesOrWaiting).toBe(true)
      expect(canAdvance.reasons.notGenerating).toBe(true)
      expect(canAdvance.reasons.notGameOver).toBe(true)
    })

    it('should not allow advancement during generation', () => {
      engine.state.generationInProgress = true
      const canAdvance = engine.canAdvance()
      
      expect(canAdvance.canAdvance).toBe(false)
      expect(canAdvance.reasons.notGenerating).toBe(false)
    })

    it('should not allow advancement when game is over', () => {
      engine.state.isGameOver = true
      const canAdvance = engine.canAdvance()
      
      expect(canAdvance.canAdvance).toBe(false)
      expect(canAdvance.reasons.notGameOver).toBe(false)
    })

    it('should not allow advancement with invalid state', () => {
      engine.state.currentNodeId = 'nonexistent'
      const canAdvance = engine.canAdvance()
      
      expect(canAdvance.canAdvance).toBe(false)
      expect(canAdvance.reasons.validState).toBe(false)
    })
  })

  describe('Choice Generation', () => {
    it('should generate choices successfully', async () => {
      expect(engine.state.phase).toBe('waiting_for_choices')
      expect(engine.getCurrentChoices().length).toBe(0)
      
      const choices = await engine.generateChoices()
      
      expect(choices.length).toBe(3)
      expect(engine.state.phase).toBe('choosing')
      expect(engine.state.generationInProgress).toBe(false)
      expect(engine.getCurrentChoices().length).toBe(3)
      
      // Validate generated choices
      choices.forEach(choice => {
        expect(choice.fromId).toBe('start')
        expect(choice.toId).toBeDefined()
        expect(choice.text).toBeDefined()
        expect(choice.icon).toBeDefined()
        expect(typeof choice.cost).toBe('number')
        expect(typeof choice.experience).toBe('number')
      })
    })

    it('should emit correct events during generation', async () => {
      // Ensure we start fresh
      expect(engine.getCurrentChoices().length).toBe(0)
      
      await engine.generateChoices()
      
      console.log('All events:', events.map(e => ({ type: e.type, nodeId: e.nodeId, count: e.count })))
      console.log('Generated choices:', engine.getCurrentChoices().length)
      
      const startEvent = events.find(e => e.type === 'generationStarted')
      const completeEvent = events.find(e => e.type === 'generationCompleted')
      
      expect(startEvent).toBeDefined()
      expect(startEvent.type).toBe('generationStarted')
      // Events have flattened structure, so nodeId is at top level  
      expect(startEvent.nodeId).toBe('start')
      
      expect(completeEvent).toBeDefined()
      expect(completeEvent.type).toBe('generationCompleted')
      expect(completeEvent.count).toBe(3)
    })

    it('should handle generation errors gracefully', async () => {
      mockAIService.generateChoices = vi.fn().mockRejectedValue(new Error('AI service failed'))
      
      await expect(engine.generateChoices()).rejects.toThrow('AI service failed')
      
      expect(engine.state.phase).toBe('waiting_for_choices')
      expect(engine.state.generationInProgress).toBe(false)
      expect(engine.state.lastError).toBe('AI service failed')
      expect(engine.state.errorCount).toBe(1)
      
      const failEvent = events.find(e => e.type === 'generationFailed')
      expect(failEvent).toBeDefined()
    })

    it('should not regenerate choices if they already exist', async () => {
      // Generate choices first time
      await engine.generateChoices()
      const firstChoices = engine.getCurrentChoices()
      expect(engine.state.phase).toBe('choosing')
      
      // Try to generate again - should return existing choices without error
      const secondChoices = await engine.generateChoices()
      
      expect(secondChoices).toEqual(firstChoices)
      expect(engine.getCurrentChoices().length).toBe(3)
      expect(engine.state.phase).toBe('choosing') // Phase should remain the same
    })

    it('should prevent generation in wrong phase', async () => {
      engine.state.phase = 'advancing' // Use a phase that actually should prevent generation
      
      await expect(engine.generateChoices()).rejects.toThrow('Cannot generate choices in phase: advancing')
    })

    it('should prevent concurrent generation', async () => {
      engine.state.generationInProgress = true
      
      await expect(engine.generateChoices()).rejects.toThrow('Generation already in progress')
    })
  })

  describe('Choice Making', () => {
    beforeEach(async () => {
      await engine.generateChoices()
    })

    it('should make choice successfully', async () => {
      const choices = engine.getCurrentChoices()
      const choice = choices[0]
      
      expect(engine.state.currentNodeId).toBe('start')
      expect(engine.state.phase).toBe('choosing')
      
      const result = await engine.makeChoice(choice.id)
      
      expect(result.success).toBe(true)
      expect(result.newNodeId).toBe(choice.toId)
      expect(result.targetNode).toBeDefined()
      expect(result.targetNode.id).toBe(choice.toId)
      
      expect(engine.state.currentNodeId).toBe(choice.toId)
      expect(engine.state.phase).toBe('waiting_for_choices')
      expect(engine.state.visitedNodes.has(choice.toId)).toBe(true)
      
      // Experience should be gained
      expect(engine.state.experience).toBe(choice.experience)
    })

    it('should generate new node when making choice', async () => {
      const initialNodeCount = engine.state.storyGraph.nodes.length
      const choices = engine.getCurrentChoices()
      const choice = choices[0]
      
      await engine.makeChoice(choice.id)
      
      expect(engine.state.storyGraph.nodes.length).toBe(initialNodeCount + 1)
      
      const newNode = engine.state.storyGraph.nodes.find(n => n.id === choice.toId)
      expect(newNode).toBeDefined()
      expect(newNode.location).toBeDefined()
      expect(newNode.situation).toBeDefined()
      expect(newNode.generated).toBe(true)
    })

    it('should apply choice consequences correctly', async () => {
      const choices = engine.getCurrentChoices()
      const expensiveChoice = choices.find(c => c.cost > 0)
      
      const initialGold = engine.state.gold
      const initialExp = engine.state.experience
      
      await engine.makeChoice(expensiveChoice.id)
      
      expect(engine.state.gold).toBe(initialGold - expensiveChoice.cost)
      expect(engine.state.experience).toBe(initialExp + expensiveChoice.experience)
    })

    it('should handle level up when enough experience gained', async () => {
      engine.state.experience = 95 // Close to level up
      
      const choices = engine.getCurrentChoices()
      const choice = choices[0]
      
      const initialGold = engine.state.gold
      await engine.makeChoice(choice.id)
      
      expect(engine.state.level).toBe(2)
      expect(engine.state.gold).toBe(initialGold - choice.cost + 50) // Level up bonus
      
      const levelUpEvent = events.find(e => e.type === 'levelUp')
      expect(levelUpEvent).toBeDefined()
      expect(levelUpEvent.newLevel).toBe(2)
    })

    it('should prevent choice when cannot afford', async () => {
      engine.state.gold = 0
      const choices = engine.getCurrentChoices()
      const expensiveChoice = choices.find(c => c.cost > 0)
      
      await expect(engine.makeChoice(expensiveChoice.id)).rejects.toThrow('Cannot afford choice')
    })

    it('should prevent choice in wrong phase', async () => {
      engine.state.phase = 'waiting_for_choices'
      const choices = engine.getCurrentChoices()
      
      await expect(engine.makeChoice(choices[0].id)).rejects.toThrow('Cannot make choice in phase: waiting_for_choices')
    })

    it('should prevent choice for non-existent choice', async () => {
      await expect(engine.makeChoice('nonexistent')).rejects.toThrow('Choice \'nonexistent\' not found')
    })

    it('should clear choices for new node after advancement', async () => {
      const choices = engine.getCurrentChoices()
      const choice = choices[0]
      
      await engine.makeChoice(choice.id)
      
      // Should have no choices for the new node initially
      const newChoices = engine.getCurrentChoices()
      expect(newChoices.length).toBe(0)
      expect(engine.state.phase).toBe('waiting_for_choices')
    })
  })

  describe('Complete Game Flow', () => {
    it('should complete a full advancement cycle', async () => {
      // Initial state
      expect(engine.state.phase).toBe('waiting_for_choices')
      expect(engine.getCurrentChoices().length).toBe(0)
      expect(engine.canAdvance().canAdvance).toBe(true)
      
      // Generate choices
      await engine.generateChoices()
      expect(engine.state.phase).toBe('choosing')
      expect(engine.getCurrentChoices().length).toBe(3)
      
      // Make choice
      const choice = engine.getCurrentChoices()[0]
      const result = await engine.makeChoice(choice.id)
      
      expect(result.success).toBe(true)
      expect(engine.state.phase).toBe('waiting_for_choices')
      expect(engine.state.currentNodeId).toBe(choice.toId)
      expect(engine.getCurrentChoices().length).toBe(0)
      
      const canAdvanceResult = engine.canAdvance()
      if (!canAdvanceResult.canAdvance) {
        console.log('Cannot advance after choice:', canAdvanceResult)
        console.log('Current state:', engine.getState())
        console.log('Validation:', engine.validateState())
      }
      expect(canAdvanceResult.canAdvance).toBe(true)
      
      // Should be able to generate choices for new node
      await engine.generateChoices()
      expect(engine.getCurrentChoices().length).toBe(3)
      expect(engine.state.phase).toBe('choosing')
    })

    it('should maintain state validity throughout advancement', async () => {
      const steps = 3
      
      for (let i = 0; i < steps; i++) {
        // Validate state before each step
        const validation = engine.validateState()
        expect(validation.valid).toBe(true)
        
        // Generate choices if needed
        if (engine.getCurrentChoices().length === 0) {
          await engine.generateChoices()
        }
        
        // Make a choice
        const choices = engine.getCurrentChoices()
        if (choices.length > 0) {
          const affordableChoice = choices.find(c => engine.canMakeChoice(c))
          if (affordableChoice) {
            await engine.makeChoice(affordableChoice.id)
          }
        }
        
        // Validate state after step
        const postValidation = engine.validateState()
        if (!postValidation.valid) {
          console.log('Validation failed at step', i, postValidation)
        }
        expect(postValidation.valid).toBe(true)
      }
    })

    it('should track action history correctly', async () => {
      await engine.generateChoices()
      const choice = engine.getCurrentChoices()[0]
      await engine.makeChoice(choice.id)
      
      expect(engine.actionHistory.length).toBe(2)
      
      const generateAction = engine.actionHistory.find(a => a.action === 'generateChoices')
      expect(generateAction).toBeDefined()
      expect(generateAction.result.success).toBe(true)
      
      const choiceAction = engine.actionHistory.find(a => a.action === 'makeChoice')
      expect(choiceAction).toBeDefined()
      expect(choiceAction.result.success).toBe(true)
    })
  })

  describe('Error Recovery', () => {
    it('should recover from choice generation failure', async () => {
      mockAIService.generateChoices = vi.fn().mockRejectedValue(new Error('Network error'))
      
      await expect(engine.generateChoices()).rejects.toThrow('Network error')
      expect(engine.state.phase).toBe('waiting_for_choices')
      
      // Fix the service and try again
      mockAIService.generateChoices = createMockAIService().generateChoices
      
      await engine.generateChoices()
      expect(engine.state.phase).toBe('choosing')
      expect(engine.getCurrentChoices().length).toBe(3)
    })

    it('should recover from node generation failure', async () => {
      await engine.generateChoices()
      const choice = engine.getCurrentChoices()[0]
      
      mockAIService.generateStoryNode = vi.fn().mockRejectedValue(new Error('Node generation failed'))
      
      await expect(engine.makeChoice(choice.id)).rejects.toThrow('Node generation failed')
      expect(engine.state.phase).toBe('choosing')
      expect(engine.state.currentNodeId).toBe('start') // Should stay at original node
      
      // Fix the service and try again
      mockAIService.generateStoryNode = createMockAIService().generateStoryNode
      
      await engine.makeChoice(choice.id)
      expect(engine.state.currentNodeId).toBe(choice.toId)
      expect(engine.state.phase).toBe('waiting_for_choices')
    })
  })

  describe('Game Reset', () => {
    it('should reset to initial state', async () => {
      // Advance the game
      await engine.generateChoices()
      const choice = engine.getCurrentChoices()[0]
      await engine.makeChoice(choice.id)
      
      expect(engine.state.currentNodeId).not.toBe('start')
      expect(engine.state.experience).toBeGreaterThan(0)
      
      // Reset
      engine.reset()
      
      expect(engine.state.currentNodeId).toBe('start')
      expect(engine.state.phase).toBe('waiting_for_choices')
      expect(engine.state.experience).toBe(0)
      expect(engine.state.gold).toBe(50)
      expect(engine.getCurrentChoices().length).toBe(0)
      expect(engine.actionHistory.length).toBe(0)
      
      const resetEvent = events.find(e => e.type === 'gameReset')
      expect(resetEvent).toBeDefined()
    })
  })

  describe('Debug Information', () => {
    it('should provide comprehensive debug info', async () => {
      await engine.generateChoices()
      const debugInfo = engine.getDebugInfo()
      
      expect(debugInfo.state).toBeDefined()
      expect(debugInfo.validation).toBeDefined()
      expect(debugInfo.validation.valid).toBe(true)
      expect(debugInfo.actionHistory).toBeDefined()
      expect(debugInfo.currentChoices).toBeDefined()
      expect(debugInfo.canAdvance).toBeDefined()
      expect(debugInfo.canAdvance.canAdvance).toBe(true)
    })

    it('should export complete state', async () => {
      await engine.generateChoices()
      const exportData = engine.exportState()
      
      expect(exportData.gameState).toBeDefined()
      expect(exportData.actionHistory).toBeDefined()
      expect(exportData.validation).toBeDefined()
      expect(exportData.validation.valid).toBe(true)
    })
  })
})