import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
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

// Create mock AI service for UI tests
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
      reasoning: 'Mock generated choices for architectural learning'
    }
  },

  async generateStoryNode(choice, fromNode, context) {
    return {
      location: `Mock Location for ${choice.text}`,
      situation: `<p>You chose "${choice.text}" and now find yourself in a new scenario.</p>`,
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

describe('AdventureGame UI Interactions', () => {
  let wrapper
  let mockAIService

  beforeEach(() => {
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

  describe('Generate Choices Button', () => {
    it('should show generate button when no choices exist', () => {
      const generateBtn = wrapper.find('.generate-btn')
      expect(generateBtn.exists()).toBe(true)
      expect(generateBtn.text()).toContain('Generate Adventure Choices')
    })

    it('should generate choices when generate button is clicked', async () => {
      const generateBtn = wrapper.find('.generate-btn')
      expect(generateBtn.exists()).toBe(true)
      
      // Initially no choices
      expect(wrapper.vm.currentChoices.length).toBe(0)
      
      // Click generate button
      await generateBtn.trigger('click')
      await wrapper.vm.$nextTick()
      
      // Should have choices now
      expect(wrapper.vm.currentChoices.length).toBeGreaterThan(0)
    })

    it('should hide generate button after choices are generated', async () => {
      const generateBtn = wrapper.find('.generate-btn')
      
      // Click generate button
      await generateBtn.trigger('click')
      await wrapper.vm.$nextTick()
      
      // Generate button should be hidden, choices should be visible
      expect(wrapper.find('.generate-btn').exists()).toBe(false)
      expect(wrapper.find('.choices-grid').exists()).toBe(true)
    })

    it('should show loading state during generation', async () => {
      // Spy on the AI service to delay response
      const originalGenerateChoices = mockAIService.generateChoices
      let resolvePromise
      mockAIService.generateChoices = vi.fn(() => new Promise(resolve => {
        resolvePromise = resolve
      }))

      const generateBtn = wrapper.find('.generate-btn')
      
      // Click generate button (don't await yet)
      const clickPromise = generateBtn.trigger('click')
      await wrapper.vm.$nextTick()
      
      // Should show loading state
      expect(wrapper.find('.loading-bar').exists()).toBe(true)
      expect(wrapper.find('.loading-message').text()).toContain('Generating')
      
      // Resolve the promise and wait for completion
      resolvePromise({
        choices: [{ text: 'Test', icon: '🧪', cost: 0, experience: 10 }],
        confidence: 0.8,
        reasoning: 'test'
      })
      await clickPromise
      await wrapper.vm.$nextTick()
      
      // Loading should be gone
      expect(wrapper.find('.loading-bar').exists()).toBe(false)
      
      // Restore original function
      mockAIService.generateChoices = originalGenerateChoices
    })

    it('should show error if generation fails', async () => {
      // Mock AI service to throw error
      mockAIService.generateChoices = vi.fn().mockRejectedValue(new Error('Generation failed'))

      const generateBtn = wrapper.find('.generate-btn')
      
      // Click generate button
      await generateBtn.trigger('click')
      await wrapper.vm.$nextTick()
      
      // Should show error
      expect(wrapper.find('.error-bar').exists()).toBe(true)
      expect(wrapper.find('.error-message').text()).toContain('Failed to generate choices')
    })
  })

  describe('Choice Selection', () => {
    beforeEach(async () => {
      // Generate choices for testing
      await wrapper.vm.generateChoicesForNode('start')
      await wrapper.vm.$nextTick()
    })

    it('should display choice buttons after generation', () => {
      const choiceButtons = wrapper.findAll('.choice-btn')
      expect(choiceButtons.length).toBeGreaterThan(0)
      
      // Check button content
      const firstButton = choiceButtons[0]
      expect(firstButton.find('.choice-text').exists()).toBe(true)
      expect(firstButton.find('.choice-icon').exists()).toBe(true)
    })

    it('should navigate to new location when choice is clicked', async () => {
      const choiceButtons = wrapper.findAll('.choice-btn')
      const firstChoice = choiceButtons[0]
      
      const initialLocation = wrapper.find('.current-location').text()
      
      // Click the first choice
      await firstChoice.trigger('click')
      await wrapper.vm.$nextTick()
      
      // Should navigate to new location
      const newLocation = wrapper.find('.current-location').text()
      expect(newLocation).not.toBe(initialLocation)
    })

    it('should update game stats after making a choice', async () => {
      const choiceButtons = wrapper.findAll('.choice-btn')
      const firstChoice = choiceButtons[0]
      
      const initialExp = parseInt(wrapper.findAll('.stat-value')[1].text())
      
      // Click the first choice
      await firstChoice.trigger('click')
      await wrapper.vm.$nextTick()
      
      // Experience should increase
      const newExp = parseInt(wrapper.findAll('.stat-value')[1].text())
      expect(newExp).toBeGreaterThan(initialExp)
    })

    it('should disable choices that cannot be afforded', async () => {
      // Set low gold amount
      wrapper.vm.gameState.gold = 1
      await wrapper.vm.$nextTick()
      
      const choiceButtons = wrapper.findAll('.choice-btn')
      const expensiveChoice = choiceButtons.find(btn => 
        btn.find('.choice-requirements').exists() && 
        btn.find('.choice-requirements').text().includes('💰')
      )
      
      if (expensiveChoice) {
        expect(expensiveChoice.classes()).toContain('disabled')
        expect(expensiveChoice.attributes('disabled')).toBeDefined()
      }
    })

    it('should show choice requirements in UI', () => {
      const choiceButtons = wrapper.findAll('.choice-btn')
      const choiceWithCost = choiceButtons.find(btn => 
        btn.find('.choice-requirements').exists()
      )
      
      if (choiceWithCost) {
        const requirements = choiceWithCost.find('.choice-requirements')
        expect(requirements.text()).toMatch(/💰|🔑/)
      }
    })
  })

  describe('Action Log', () => {
    it('should display action log entries', () => {
      const logSection = wrapper.find('.action-log')
      expect(logSection.exists()).toBe(true)
      
      const logEntries = wrapper.findAll('.log-entry')
      expect(logEntries.length).toBeGreaterThan(0)
    })

    it('should add log entry when generating choices', async () => {
      const initialLogCount = wrapper.findAll('.log-entry').length
      
      // Generate choices
      await wrapper.vm.generateChoicesForNode('start')
      await wrapper.vm.$nextTick()
      
      const newLogCount = wrapper.findAll('.log-entry').length
      expect(newLogCount).toBeGreaterThan(initialLogCount)
      
      // Check for generation log entry
      const logTexts = wrapper.findAll('.log-text').map(el => el.text())
      expect(logTexts.some(text => text.includes('Generated'))).toBe(true)
    })

    it('should show timestamps in log entries', () => {
      const logEntries = wrapper.findAll('.log-entry')
      if (logEntries.length > 0) {
        const firstEntry = logEntries[0]
        const timeElement = firstEntry.find('.log-time')
        expect(timeElement.exists()).toBe(true)
        expect(timeElement.text()).toMatch(/\d{1,2}:\d{2}/)
      }
    })
  })

  describe('Error Handling UI', () => {
    it('should show error bar when error occurs', async () => {
      // Trigger an error
      wrapper.vm.showError('Test error message')
      await wrapper.vm.$nextTick()
      
      const errorBar = wrapper.find('.error-bar')
      expect(errorBar.exists()).toBe(true)
      expect(errorBar.find('.error-message').text()).toBe('Test error message')
    })

    it('should hide error bar when close button is clicked', async () => {
      // Show error
      wrapper.vm.showError('Test error')
      await wrapper.vm.$nextTick()
      
      // Click close button
      const closeBtn = wrapper.find('.error-close')
      await closeBtn.trigger('click')
      await wrapper.vm.$nextTick()
      
      // Error should be hidden
      expect(wrapper.find('.error-bar').exists()).toBe(false)
    })
  })

  describe('Inventory UI', () => {
    it('should show empty inventory message initially', () => {
      const emptyMessage = wrapper.find('.empty-inventory')
      expect(emptyMessage.exists()).toBe(true)
      expect(emptyMessage.text()).toBe('Your inventory is empty')
    })

    it('should display inventory items when present', async () => {
      // Add an item to inventory
      wrapper.vm.addToInventory({
        id: 'test_item',
        name: 'Test Item',
        icon: '🧪',
        description: 'A test item',
        consumable: false
      })
      await wrapper.vm.$nextTick()
      
      const inventoryItems = wrapper.findAll('.inventory-item')
      expect(inventoryItems.length).toBe(1)
      expect(inventoryItems[0].find('.item-name').text()).toBe('Test Item')
      expect(inventoryItems[0].find('.item-icon').text()).toBe('🧪')
    })

    it('should show item quantities for stackable items', async () => {
      // Add stackable items
      const stackableItem = {
        id: 'potion',
        name: 'Health Potion',
        icon: '🧪',
        consumable: true
      }
      
      wrapper.vm.addToInventory(stackableItem)
      wrapper.vm.addToInventory(stackableItem)
      await wrapper.vm.$nextTick()
      
      const inventoryItems = wrapper.findAll('.inventory-item')
      expect(inventoryItems.length).toBe(1)
      
      const quantityElement = inventoryItems[0].find('.item-quantity')
      expect(quantityElement.exists()).toBe(true)
      expect(quantityElement.text()).toBe('2')
    })
  })

  describe('Game Reset UI', () => {
    it('should show restart button when game is over', async () => {
      // Set game over state
      wrapper.vm.gameState.isGameOver = true
      wrapper.vm.gameState.isWin = true
      wrapper.vm.gameState.endMessage = 'You won!'
      await wrapper.vm.$nextTick()
      
      const restartBtn = wrapper.find('.restart-btn')
      expect(restartBtn.exists()).toBe(true)
      expect(restartBtn.text()).toContain('Start New Adventure')
    })

    it('should reset game when restart button is clicked', async () => {
      // Set modified game state
      wrapper.vm.gameState.level = 5
      wrapper.vm.gameState.experience = 500
      wrapper.vm.gameState.gold = 200
      wrapper.vm.gameState.isGameOver = true
      await wrapper.vm.$nextTick()
      
      // Click restart
      const restartBtn = wrapper.find('.restart-btn')
      await restartBtn.trigger('click')
      await wrapper.vm.$nextTick()
      
      // Should be reset to initial state
      expect(wrapper.vm.gameState.level).toBe(1)
      expect(wrapper.vm.gameState.experience).toBe(0)
      expect(wrapper.vm.gameState.gold).toBe(50)
      expect(wrapper.vm.gameState.isGameOver).toBe(false)
    })
  })

  describe('Graph Visualization UI', () => {
    it('should show graph controls', () => {
      const graphControls = wrapper.find('.graph-controls')
      expect(graphControls.exists()).toBe(true)
      
      const layoutSelect = wrapper.find('.layout-select')
      expect(layoutSelect.exists()).toBe(true)
      
      const resetBtn = wrapper.find('.graph-btn')
      expect(resetBtn.exists()).toBe(true)
    })

    it('should update layout when layout select changes', async () => {
      const layoutSelect = wrapper.find('.layout-select')
      
      // Change layout
      await layoutSelect.setValue('circle')
      await wrapper.vm.$nextTick()
      
      expect(wrapper.vm.layoutMode).toBe('circle')
    })
  })

  describe('Navigation State Management', () => {
    it('should handle navigation to nonexistent nodes gracefully', async () => {
      // Try to navigate to a node that doesn't exist
      const invalidChoice = {
        id: 'invalid-choice',
        fromId: 'start',
        toId: 'nonexistent_node',
        text: 'Invalid choice',
        icon: '❌',
        cost: 0,
        experience: 10
      }
      
      // Add the invalid choice to test error handling
      wrapper.vm.storyGraph.edges.push(invalidChoice)
      await wrapper.vm.$nextTick()
      
      // Try to make the choice - should handle gracefully
      await wrapper.vm.makeChoice(invalidChoice)
      
      // Should either generate the missing node or show an error
      const hasError = wrapper.find('.error-bar').exists()
      const hasNewNode = wrapper.vm.storyGraph.nodes.some(n => n.id === 'nonexistent_node')
      
      expect(hasError || hasNewNode).toBe(true)
    })
  })
})