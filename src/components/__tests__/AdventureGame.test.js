import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import AdventureGame from '../AdventureGame.vue'

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

// Create mock AI service for tests
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
          text: "Study Monolithic Design", 
          icon: "🏰",
          cost: 0,
          experience: 15,
          description: "Understand single-deployment architectures",
          requiresItem: null,
          risk: "low"
        }
      ],
      confidence: 0.85,
      reasoning: 'Mock generated choices'
    }
  },

  async generateStoryNode(choice, fromNode, context) {
    return {
      location: "Mock Location",
      situation: "<p>Mock situation generated for testing.</p>",
      confidence: 0.9,
      reasoning: 'Mock generated story node'
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

describe('AdventureGame', () => {
  let wrapper
  let mockFetch
  let mockAIService

  beforeEach(() => {
    mockFetch = vi.fn()
    global.fetch = mockFetch
    mockAIService = createMockAIService()
    
    wrapper = mount(AdventureGame, {
      props: {
        aiService: mockAIService
      },
      attachTo: document.body
    })
  })

  afterEach(() => {
    if (wrapper) {
      wrapper.unmount()
    }
  })

  describe('Initial Game State', () => {
    it('should initialize with correct default game state', () => {
      expect(wrapper.vm.gameState.currentNodeId).toBe('start')
      expect(wrapper.vm.gameState.level).toBe(1)
      expect(wrapper.vm.gameState.experience).toBe(0)
      expect(wrapper.vm.gameState.gold).toBe(50)
      expect(wrapper.vm.gameState.inventory).toEqual([])
      expect(wrapper.vm.gameState.isGameOver).toBe(false)
      expect(wrapper.vm.gameState.isWin).toBe(false)
    })

    it('should display game stats correctly', () => {
      const stats = wrapper.findAll('.stat-value')
      expect(stats[0].text()).toBe('1') // Level
      expect(stats[1].text()).toBe('0') // Experience
      expect(stats[2].text()).toBe('50💰') // Gold
    })

    it('should show the starting location', () => {
      expect(wrapper.find('.current-location').text()).toBe('📍 The Town Square of Architectura')
    })

    it('should display the starting story text', () => {
      const storyText = wrapper.find('.story-text')
      expect(storyText.html()).toContain('Architectura')
      expect(storyText.html()).toContain('software systems')
    })
  })

  describe('Story Graph Navigation', () => {
    it('should compute current node correctly', () => {
      expect(wrapper.vm.currentNode.id).toBe('start')
      expect(wrapper.vm.currentNode.location).toBe('The Town Square of Architectura')
    })

    it('should have no choices initially in procedural system', () => {
      // In procedural system, choices are not generated automatically
      const choices = wrapper.vm.currentChoices
      expect(choices.length).toBe(0)
    })

    it('should display choices as clickable buttons when available', async () => {
      // Generate choices first
      await wrapper.vm.generateChoicesForNode('start')
      await wrapper.vm.$nextTick()

      const choiceButtons = wrapper.findAll('.choice-btn')
      expect(choiceButtons.length).toBe(wrapper.vm.currentChoices.length)
      expect(choiceButtons.length).toBeGreaterThan(0)
    })
  })

  describe('Procedural Generation', () => {
    it('should generate choices for a new node', async () => {
      // Create a fresh component instance to test generation from scratch
      const freshWrapper = mount(AdventureGame, {
        props: {
          aiService: createMockAIService()
        },
        attachTo: document.body
      })
      
      // Clear any auto-generated choices
      freshWrapper.vm.storyGraph.edges = []
      
      expect(freshWrapper.vm.currentChoices.length).toBe(0)
      
      await freshWrapper.vm.generateChoicesForNode('start')
      
      expect(freshWrapper.vm.currentChoices.length).toBeGreaterThan(0)
      expect(freshWrapper.vm.currentChoices.every(choice => choice.fromId === 'start')).toBe(true)
      
      freshWrapper.unmount()
    })

    it('should not regenerate choices for a node that already has them', async () => {
      await wrapper.vm.generateChoicesForNode('start')
      const firstChoiceCount = wrapper.vm.currentChoices.length
      
      await wrapper.vm.generateChoicesForNode('start')
      const secondChoiceCount = wrapper.vm.currentChoices.length
      
      expect(firstChoiceCount).toBe(secondChoiceCount)
    })

    it('should generate new nodes when choices are made', async () => {
      await wrapper.vm.generateChoicesForNode('start')
      const choice = wrapper.vm.currentChoices[0]
      
      const initialNodeCount = wrapper.vm.storyGraph.nodes.length
      await wrapper.vm.makeChoice(choice)
      
      expect(wrapper.vm.storyGraph.nodes.length).toBe(initialNodeCount + 1)
    })

    it('should navigate to generated nodes', async () => {
      await wrapper.vm.generateChoicesForNode('start')
      const choice = wrapper.vm.currentChoices[0]
      
      await wrapper.vm.makeChoice(choice)
      
      expect(wrapper.vm.gameState.currentNodeId).toBe(choice.toId)
      expect(wrapper.vm.gameState.visitedNodes.has(choice.toId)).toBe(true)
    })

    it('should generate contextual content based on choices', async () => {
      // First generate choices
      await wrapper.vm.generateChoicesForNode('start')
      
      const choice = wrapper.vm.currentChoices.find(c => 
        c.text.toLowerCase().includes('microservice')
      ) || wrapper.vm.currentChoices[0]
      
      await wrapper.vm.makeChoice(choice)
      
      // New node should be created
      const newNode = wrapper.vm.storyGraph.nodes.find(n => n.id === choice.toId)
      expect(newNode).toBeDefined()
      expect(newNode.location).toBeDefined()
      expect(newNode.situation).toBeDefined()
    })

    it('should generate different scenarios for different choices', async () => {
      // First generate choices
      await wrapper.vm.generateChoicesForNode('start')
      const choices = wrapper.vm.currentChoices
      
      // Should have multiple different choice options
      expect(choices.length).toBeGreaterThan(1)
      
      // Choices should have different text and icons
      const texts = choices.map(c => c.text)
      const uniqueTexts = new Set(texts)
      expect(uniqueTexts.size).toBe(texts.length) // All unique
    })

    it('should log generation activities', async () => {
      const initialLogLength = wrapper.vm.gameState.actionLog.length
      
      // Clear choices to force regeneration
      wrapper.vm.storyGraph.edges = []
      await wrapper.vm.generateChoicesForNode('start')
      
      expect(wrapper.vm.gameState.actionLog.length).toBeGreaterThan(initialLogLength)
      
      const hasGenerationLog = wrapper.vm.gameState.actionLog.some(entry =>
        entry.message.includes('Generated') || entry.message.includes('AI')
      )
      expect(hasGenerationLog).toBe(true)
    })
  })

  describe('Choice Making Logic', () => {
    beforeEach(async () => {
      // Generate choices for testing
      await wrapper.vm.generateChoicesForNode('start')
    })

    it('should allow making a choice when requirements are met', () => {
      const choice = wrapper.vm.currentChoices[0]
      expect(wrapper.vm.canMakeChoice(choice)).toBe(true)
    })

    it('should prevent choice when insufficient gold', () => {
      const expensiveChoice = {
        cost: 100,
        requiresItem: null
      }
      
      expect(wrapper.vm.canMakeChoice(expensiveChoice)).toBe(false)
    })

    it('should prevent choice when required item is missing', () => {
      const itemChoice = {
        cost: 0,
        requiresItem: 'Architecture Compass'
      }
      
      expect(wrapper.vm.canMakeChoice(itemChoice)).toBe(false)
    })

    it('should allow choice when required item is in inventory', () => {
      const itemChoice = {
        cost: 0,
        requiresItem: 'Architecture Compass'
      }
      
      wrapper.vm.gameState.inventory = [{
        id: 'architecture_compass',
        name: 'Architecture Compass',
        consumable: false
      }]
      
      expect(wrapper.vm.canMakeChoice(itemChoice)).toBe(true)
    })

    it('should update game state when making a choice', async () => {
      const choice = wrapper.vm.currentChoices[0]
      
      const initialGold = wrapper.vm.gameState.gold
      const initialExp = wrapper.vm.gameState.experience
      
      await wrapper.vm.makeChoice(choice)
      
      expect(wrapper.vm.gameState.currentNodeId).toBe(choice.toId)
      expect(wrapper.vm.gameState.visitedNodes.has(choice.toId)).toBe(true)
      expect(wrapper.vm.gameState.experience).toBe(initialExp + choice.experience)
      expect(wrapper.vm.gameState.gold).toBe(initialGold - choice.cost)
    })

    it('should handle winning condition', async () => {
      const winningChoice = {
        id: 'win-choice',
        fromId: 'start',
        toId: 'victory-node',
        text: 'Win the game',
        icon: '🏆',
        isWinning: true,
        experience: 50,
        cost: 0
      }
      
      // Add the choice to the graph
      wrapper.vm.storyGraph.edges.push(winningChoice)
      wrapper.vm.storyGraph.nodes.push({
        id: 'victory-node',
        location: 'Victory Hall',
        situation: 'You won!'
      })
      
      await wrapper.vm.makeChoice(winningChoice)
      
      expect(wrapper.vm.gameState.isGameOver).toBe(true)
      expect(wrapper.vm.gameState.isWin).toBe(true)
      expect(wrapper.vm.gameState.endMessage).toBe('You have mastered software architecture!')
    })

    it('should handle special game mechanics', async () => {
      const testChoice = {
        id: 'test-choice',
        fromId: 'start',
        toId: 'test-node',
        text: 'Test choice',
        icon: '🧪',
        experience: 10,
        cost: 0
      }
      
      // Add the choice to the graph
      wrapper.vm.storyGraph.edges.push(testChoice)
      
      await wrapper.vm.makeChoice(testChoice)
      
      expect(wrapper.vm.gameState.currentNodeId).toBe('test-node')
      expect(wrapper.vm.gameState.isGameOver).toBe(false)
    })
  })

  describe('Level System', () => {
    beforeEach(async () => {
      await wrapper.vm.generateChoicesForNode('start')
    })

    it('should level up when experience threshold is reached', async () => {
      wrapper.vm.gameState.experience = 95
      wrapper.vm.gameState.level = 1
      
      const choice = {
        id: 'level-test',
        fromId: 'start',
        toId: 'test-node',
        text: 'Level up test',
        icon: '⬆️',
        experience: 10,
        cost: 0
      }
      
      wrapper.vm.storyGraph.edges.push(choice)
      
      const initialGold = wrapper.vm.gameState.gold
      await wrapper.vm.makeChoice(choice)
      
      expect(wrapper.vm.gameState.level).toBe(2)
      expect(wrapper.vm.gameState.gold).toBe(initialGold + 50) // Level up bonus
    })

    it('should not level up when experience threshold is not reached', async () => {
      wrapper.vm.gameState.experience = 50
      wrapper.vm.gameState.level = 1
      
      const choice = wrapper.vm.currentChoices[0]
      
      await wrapper.vm.makeChoice(choice)
      
      expect(wrapper.vm.gameState.level).toBe(1)
    })
  })

  describe('Inventory Management', () => {
    it('should add items to inventory', () => {
      const item = {
        id: 'test_item',
        name: 'Test Item',
        consumable: false
      }
      
      wrapper.vm.addToInventory(item)
      
      expect(wrapper.vm.gameState.inventory).toContainEqual({
        ...item,
        quantity: 1
      })
    })

    it('should stack consumable items', () => {
      const item = {
        id: 'potion',
        name: 'Health Potion',
        consumable: true,
        quantity: 1
      }
      
      wrapper.vm.addToInventory(item)
      wrapper.vm.addToInventory(item)
      
      expect(wrapper.vm.gameState.inventory.length).toBe(1)
      expect(wrapper.vm.gameState.inventory[0].quantity).toBe(2)
    })

    it('should remove items from inventory', () => {
      const item = {
        id: 'test_item',
        name: 'Test Item',
        consumable: false,
        quantity: 1
      }
      
      wrapper.vm.gameState.inventory = [item]
      wrapper.vm.removeFromInventory('test_item')
      
      expect(wrapper.vm.gameState.inventory).toEqual([])
    })

    it('should decrease quantity for consumable items', () => {
      const item = {
        id: 'potion',
        name: 'Health Potion',
        consumable: true,
        quantity: 3
      }
      
      wrapper.vm.gameState.inventory = [item]
      wrapper.vm.removeFromInventory('potion')
      
      expect(wrapper.vm.gameState.inventory[0].quantity).toBe(2)
    })
  })

  describe('Game Persistence', () => {
    it('should save game state to localStorage', () => {
      const setItemSpy = vi.spyOn(localStorage, 'setItem')
      
      wrapper.vm.saveGameState()
      
      expect(setItemSpy).toHaveBeenCalledWith(
        'adventure_game_state',
        expect.stringContaining('"currentNodeId":"start"')
      )
    })

    it('should load game state from localStorage and validate node references', () => {
      const savedState = {
        currentNodeId: 'microservices_path', // This node doesn't exist in current graph
        level: 2,
        experience: 150,
        gold: 75,
        inventory: [],
        visitedNodes: ['start', 'microservices_path'],
        isGameOver: false,
        isWin: false,
        endMessage: '',
        actionLog: []
      }
      
      vi.spyOn(localStorage, 'getItem').mockReturnValue(JSON.stringify(savedState))
      
      const loaded = wrapper.vm.loadGameState()
      
      expect(loaded).toBe(true)
      // Should reset to 'start' because 'microservices_path' doesn't exist
      expect(wrapper.vm.gameState.currentNodeId).toBe('start')
      expect(wrapper.vm.gameState.level).toBe(2) // Other state should be preserved
      // microservices_path should be removed from visitedNodes since it doesn't exist
      expect(wrapper.vm.gameState.visitedNodes.has('microservices_path')).toBe(false)
      expect(wrapper.vm.gameState.visitedNodes.has('start')).toBe(true)
    })

    it('should handle corrupted localStorage data gracefully', () => {
      vi.spyOn(localStorage, 'getItem').mockReturnValue('invalid json')
      
      const loaded = wrapper.vm.loadGameState()
      
      expect(loaded).toBe(false)
    })
  })

  describe('Action Logging', () => {
    it('should add entries to action log', () => {
      const initialLogLength = wrapper.vm.gameState.actionLog.length
      const message = 'Test action'
      wrapper.vm.addToLog(message)
      
      expect(wrapper.vm.gameState.actionLog.length).toBe(initialLogLength + 1)
      expect(wrapper.vm.gameState.actionLog[wrapper.vm.gameState.actionLog.length - 1].message).toBe(message)
      expect(wrapper.vm.gameState.actionLog[wrapper.vm.gameState.actionLog.length - 1].timestamp).toBeInstanceOf(Date)
    })

    it('should format timestamps correctly', () => {
      const testDate = new Date('2023-01-01T15:30:00')
      const formatted = wrapper.vm.formatTime(testDate)
      
      expect(formatted).toMatch(/\d{1,2}:\d{2}/)
    })
  })

  describe('Progress Tracking API', () => {
    it('should track progress successfully', async () => {
      const mockResponse = {
        ok: true,
        json: () => Promise.resolve({
          sessionId: 'test-session',
          totalChoices: 5
        })
      }
      mockFetch.mockResolvedValue(mockResponse)
      
      const choice = {
        id: 'test-choice',
        fromId: 'start',
        toId: 'microservices_path'
      }
      
      await wrapper.vm.trackProgress(choice)
      
      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/adventure/track',
        expect.objectContaining({
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: expect.stringContaining('"choiceId":"test-choice"')
        })
      )
    })

    it('should handle API failures gracefully', async () => {
      mockFetch.mockRejectedValue(new Error('Network error'))
      
      const choice = {
        id: 'test-choice',
        fromId: 'start',
        toId: 'microservices_path'
      }
      
      // Should not throw
      await wrapper.vm.trackProgress(choice)
      
      // Should log offline mode message
      expect(wrapper.vm.gameState.actionLog.some(
        entry => entry.message.includes('Offline mode')
      )).toBe(true)
    })
  })

  describe('Session Management', () => {
    it('should generate session ID if not exists', () => {
      vi.spyOn(localStorage, 'getItem').mockReturnValue(null)
      const setItemSpy = vi.spyOn(localStorage, 'setItem')
      
      const sessionId = wrapper.vm.getSessionId()
      
      expect(sessionId).toMatch(/^session_\d+_[a-z0-9]+$/)
      expect(setItemSpy).toHaveBeenCalledWith('adventure_session_id', sessionId)
    })

    it('should return existing session ID', () => {
      const existingId = 'existing-session-id'
      vi.spyOn(localStorage, 'getItem').mockReturnValue(existingId)
      
      const sessionId = wrapper.vm.getSessionId()
      
      expect(sessionId).toBe(existingId)
    })
  })

  describe('Game Reset', () => {
    it('should reset to initial state', () => {
      // Modify game state
      wrapper.vm.gameState.currentNodeId = 'microservices_path'
      wrapper.vm.gameState.level = 3
      wrapper.vm.gameState.experience = 200
      wrapper.vm.gameState.gold = 100
      wrapper.vm.gameState.isGameOver = true
      
      wrapper.vm.restartGame()
      
      expect(wrapper.vm.gameState.currentNodeId).toBe('start')
      expect(wrapper.vm.gameState.level).toBe(1)
      expect(wrapper.vm.gameState.experience).toBe(0)
      expect(wrapper.vm.gameState.gold).toBe(50)
      expect(wrapper.vm.gameState.isGameOver).toBe(false)
      expect(wrapper.vm.gameState.visitedNodes.has('start')).toBe(true)
      expect(wrapper.vm.gameState.visitedNodes.size).toBe(1)
    })

    it('should clear saved game data on restart', () => {
      const removeItemSpy = vi.spyOn(localStorage, 'removeItem')
      
      wrapper.vm.restartGame()
      
      expect(removeItemSpy).toHaveBeenCalledWith('adventure_game_state')
      expect(removeItemSpy).toHaveBeenCalledWith('adventure_session_id')
    })
  })

  describe('Component Rendering', () => {
    it('should render game over screen when game is over', async () => {
      wrapper.vm.gameState.isGameOver = true
      wrapper.vm.gameState.isWin = true
      wrapper.vm.gameState.endMessage = 'Victory!'
      
      await wrapper.vm.$nextTick()
      
      const gameOverSection = wrapper.find('.game-over')
      expect(gameOverSection.exists()).toBe(true)
      expect(gameOverSection.text()).toContain('Victory!')
    })

    it('should hide choices when game is over', async () => {
      wrapper.vm.gameState.isGameOver = true
      
      await wrapper.vm.$nextTick()
      
      const choicesSection = wrapper.find('.choices-section')
      expect(choicesSection.exists()).toBe(false)
    })

    it('should display empty inventory message when no items', () => {
      wrapper.vm.gameState.inventory = []
      
      const emptyMessage = wrapper.find('.empty-inventory')
      expect(emptyMessage.exists()).toBe(true)
      expect(emptyMessage.text()).toBe('Your inventory is empty')
    })

    it('should display inventory items when present', async () => {
      wrapper.vm.gameState.inventory = [{
        id: 'test_item',
        name: 'Test Item',
        icon: '🔧',
        description: 'A test item',
        quantity: 2
      }]
      
      await wrapper.vm.$nextTick()
      
      const inventoryItems = wrapper.findAll('.inventory-item')
      expect(inventoryItems.length).toBe(1)
      expect(inventoryItems[0].text()).toContain('Test Item')
    })
  })

  describe('Graph Visualization', () => {
    it('should build graph elements correctly', () => {
      const elements = wrapper.vm.buildGraphElements()
      
      const nodes = elements.filter(el => !el.data.source)
      const edges = elements.filter(el => el.data.source)
      
      expect(nodes.length).toBe(wrapper.vm.storyGraph.nodes.length)
      expect(edges.length).toBe(wrapper.vm.storyGraph.edges.length)
    })

    it('should handle layout changes', () => {
      wrapper.vm.layoutMode = 'circle'
      wrapper.vm.updateGraphLayout()
      
      // Graph layout should be called (mocked)
      expect(wrapper.vm.cy.layout).toHaveBeenCalled()
    })
  })
})