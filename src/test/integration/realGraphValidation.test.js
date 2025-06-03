import { describe, it, expect, beforeEach } from 'vitest'
import AdventureGame from '../../components/AdventureGame.vue'

// Create mock AI service for validation tests
const createMockAIService = () => ({
  async generateChoices() {
    return { choices: [], confidence: 0.8, reasoning: 'test' }
  },
  async generateStoryNode() {
    return { location: 'Test', situation: '<p>Test</p>', confidence: 0.8, reasoning: 'test' }
  },
  async analyzeContext() {
    return { complexity: { confidence: 0.8, reasoning: 'test' }, scope: { confidence: 0.8, reasoning: 'test' }, implementability: { confidence: 0.8, reasoning: 'test' } }
  }
})

// This test file validates the raw data structure to catch graph errors
describe('Real Graph Validation', () => {
  let componentData
  
  beforeEach(() => {
    // Get the raw component data without mounting (avoids Cytoscape canvas issues)
    // Note: We need to call the data function with a mock this context
    componentData = AdventureGame.data.call({
      $props: { aiService: createMockAIService() }
    })
  })

  it('should validate all edges reference existing nodes', () => {
    const { nodes, edges } = componentData.storyGraph
    const nodeIds = new Set(nodes.map(n => n.id))
    
    // Check every edge references valid nodes
    const invalidEdges = edges.filter(edge => 
      !nodeIds.has(edge.fromId) || !nodeIds.has(edge.toId)
    )
    
    if (invalidEdges.length > 0) {
      const errorDetails = invalidEdges.map(edge => 
        `Edge ${edge.id}: ${edge.fromId} -> ${edge.toId}`
      ).join(', ')
      
      expect.fail(`Invalid edges found: ${errorDetails}`)
    }
    
    expect(invalidEdges).toEqual([])
  })

  it('should have a start node in procedural system', () => {
    const { nodes, edges } = componentData.storyGraph
    
    // In procedural system, we should have just the start node initially
    expect(nodes.length).toBe(1)
    expect(nodes[0].id).toBe('start')
    expect(edges.length).toBe(0) // No edges until generated
  })

  it('should validate all shop items have required properties', () => {
    const { shopItems } = componentData
    
    for (const item of shopItems) {
      expect(item.id).toBeDefined()
      expect(item.name).toBeDefined()
      expect(item.icon).toBeDefined()
      expect(item.description).toBeDefined()
      expect(typeof item.cost).toBe('number')
      expect(typeof item.consumable).toBe('boolean')
    }
  })

  it('should validate initial structure is consistent', () => {
    const { nodes, edges } = componentData.storyGraph
    const nodeIds = new Set(nodes.map(n => n.id))
    
    // In procedural system, edges are generated, so we just check node consistency
    expect(nodeIds.has('start')).toBe(true)
    
    // Validate start node structure
    const startNode = nodes.find(n => n.id === 'start')
    expect(startNode.location).toBeDefined()
    expect(startNode.situation).toBeDefined()
  })
})