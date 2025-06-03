import { vi } from 'vitest'

/**
 * Test utilities for adventure game and story graph testing
 */

export const mockClaudeResponses = {
  extractMissingContext: {
    success: {
      missing_context: ['platform', 'scale', 'timeline'],
      confidence: 0.85,
      reasoning: 'Project lacks technical and scope details'
    },
    vague: {
      missing_context: ['everything'],
      confidence: 0.1,
      reasoning: 'Description too vague'
    },
    error: new Error('API Error')
  },
  
  analyzeSemantics: {
    highComplexity: {
      complexity: { confidence: 0.9, reasoning: 'High complexity project' },
      scope: { confidence: 0.7, reasoning: 'Well-defined but broad scope' },
      implementability: { confidence: 0.6, reasoning: 'Needs more technical details' }
    },
    lowComplexity: {
      complexity: { confidence: 0.3, reasoning: 'Simple task' },
      scope: { confidence: 0.9, reasoning: 'Clear, limited scope' },
      implementability: { confidence: 0.9, reasoning: 'Ready to implement' }
    }
  },
  
  analyzeRefinement: {
    shouldRefine: {
      should_refine: true,
      confidence: 0.85,
      reasoning: 'Task too complex for single implementation',
      suggested_breakdown: [
        {
          title: 'Component A',
          description: 'First component',
          estimated_complexity: 'medium'
        },
        {
          title: 'Component B', 
          description: 'Second component',
          estimated_complexity: 'low'
        }
      ]
    },
    shouldNotRefine: {
      should_refine: false,
      confidence: 0.8,
      reasoning: 'Appropriately scoped for implementation',
      suggested_breakdown: []
    }
  }
}

export const createMockGameState = (overrides = {}) => ({
  currentNodeId: 'start',
  level: 1,
  experience: 0,
  gold: 50,
  inventory: [],
  visitedNodes: new Set(['start']),
  isGameOver: false,
  isWin: false,
  endMessage: '',
  actionLog: [],
  ...overrides
})

export const createMockStoryNode = (overrides = {}) => ({
  id: 'test_node',
  location: 'Test Location',
  situation: 'A test situation with <strong>formatting</strong>',
  ...overrides
})

export const createMockStoryEdge = (overrides = {}) => ({
  id: 'test_edge',
  fromId: 'start',
  toId: 'end',
  text: 'Test choice',
  icon: '🧪',
  cost: 0,
  experience: 10,
  ...overrides
})

export const createMockChoice = (overrides = {}) => ({
  id: 'choice_1',
  fromId: 'start',
  toId: 'destination',
  text: 'Make a choice',
  icon: '⚡',
  cost: 10,
  experience: 20,
  ...overrides
})

export const createMockInventoryItem = (overrides = {}) => ({
  id: 'test_item',
  name: 'Test Item',
  icon: '🔧',
  description: 'A test item for testing',
  cost: 25,
  consumable: false,
  quantity: 1,
  ...overrides
})

export const createMockApiResponse = (data, isError = false) => {
  if (isError) {
    return Promise.reject(new Error(data))
  }
  
  return Promise.resolve({
    ok: true,
    json: () => Promise.resolve(data)
  })
}

export const setupMockFetch = (responses = []) => {
  const mockFetch = vi.fn()
  
  responses.forEach((response, index) => {
    if (response instanceof Error) {
      mockFetch.mockRejectedValueOnce(response)
    } else {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(response)
      })
    }
  })
  
  global.fetch = mockFetch
  return mockFetch
}

export const setupMockLocalStorage = () => {
  const storage = new Map()
  
  const mockLocalStorage = {
    getItem: vi.fn((key) => storage.get(key) || null),
    setItem: vi.fn((key, value) => storage.set(key, value)),
    removeItem: vi.fn((key) => storage.delete(key)),
    clear: vi.fn(() => storage.clear())
  }
  
  global.localStorage = mockLocalStorage
  return mockLocalStorage
}

export const waitForAsync = () => new Promise(resolve => setTimeout(resolve, 0))

export const createMockCytoscape = () => {
  const mockNodes = {
    forEach: vi.fn(),
    style: vi.fn(),
    filter: vi.fn(() => mockNodes)
  }
  
  const mockLayout = {
    run: vi.fn()
  }
  
  const mockCy = {
    on: vi.fn(),
    nodes: vi.fn(() => mockNodes),
    layout: vi.fn(() => mockLayout),
    style: vi.fn(),
    destroy: vi.fn(),
    add: vi.fn(),
    remove: vi.fn(),
    elements: vi.fn(() => []),
    fit: vi.fn(),
    center: vi.fn()
  }
  
  return mockCy
}

export const assertGameStateTransition = (beforeState, afterState, expectedChanges) => {
  Object.keys(expectedChanges).forEach(key => {
    if (key === 'visitedNodes') {
      expectedChanges[key].forEach(nodeId => {
        expect(afterState.visitedNodes.has(nodeId)).toBe(true)
      })
    } else {
      expect(afterState[key]).toBe(expectedChanges[key])
    }
  })
}

export const createMockTicket = (overrides = {}) => ({
  id: 'ticket_123',
  title: 'Test Ticket',
  description: 'A test ticket for development',
  refinementState: 'feature',
  semanticDescription: {
    complexity: { confidence: 0.7, reasoning: 'Medium complexity' },
    scope: { confidence: 0.8, reasoning: 'Well-defined scope' },
    implementability: { confidence: 0.6, reasoning: 'Needs refinement' }
  },
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
  tags: ['test'],
  priority: 'medium',
  ...overrides
})

export const createMockComponentInterface = (overrides = {}) => ({
  name: 'TestService',
  signature: 'testMethod(input: string): Promise<string>',
  purpose: 'Test service for testing',
  inputs: [
    {
      name: 'input',
      type: 'string',
      description: 'Test input parameter'
    }
  ],
  outputs: [
    {
      name: 'result',
      type: 'Promise<string>',
      description: 'Test output result'
    }
  ],
  preconditions: ['Input must not be empty'],
  postconditions: ['Returns processed string'],
  ...overrides
})

export const simulateUserInteraction = async (wrapper, selector, event = 'click') => {
  const element = wrapper.find(selector)
  expect(element.exists()).toBe(true)
  await element.trigger(event)
  await wrapper.vm.$nextTick()
}

export const expectLogEntry = (gameState, messageFragment) => {
  const hasEntry = gameState.actionLog.some(entry => 
    entry.message.includes(messageFragment)
  )
  expect(hasEntry).toBe(true)
}

export const expectInventoryItem = (gameState, itemName) => {
  const hasItem = gameState.inventory.some(item => 
    item.name === itemName
  )
  expect(hasItem).toBe(true)
}