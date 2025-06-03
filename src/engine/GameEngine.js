/**
 * Adventure Game Engine - Core State Machine
 * Handles all game logic independently of UI framework
 */

export class GameEngine {
  constructor(aiService, options = {}) {
    this.aiService = aiService
    this.options = {
      debugMode: false,
      logActions: true,
      ...options
    }
    
    // Core game state
    this.state = this.createInitialState()
    
    // State change listeners
    this.listeners = new Set()
    
    // Action history for debugging
    this.actionHistory = []
    
    // Internal validation
    this.validationRules = new Map()
    this.setupValidationRules()
    
    this.log('GameEngine initialized')
  }

  createInitialState() {
    return {
      // Game progression
      currentNodeId: 'start',
      phase: 'waiting_for_choices', // waiting_for_choices, generating_choices, choosing, generating_node, advancing
      
      // Player stats
      level: 1,
      experience: 0,
      gold: 50,
      inventory: [],
      
      // Game world
      visitedNodes: new Set(['start']),
      storyGraph: {
        nodes: [{
          id: 'start',
          location: 'The Town Square of Architectura',
          situation: `
            <p>Welcome to <strong>Architectura</strong>, where software systems come to life! You're a new architect arriving in the mystical town square.</p>
            <p>An old wizard approaches: <em>"Young architect, what kind of system challenges interest you most? Your choice will shape your adventure..."</em></p>
          `
        }],
        edges: []
      },
      
      // Game status
      isGameOver: false,
      isWin: false,
      endMessage: '',
      
      // Action log
      actionLog: [],
      
      // Generation tracking
      generationInProgress: false,
      currentGeneration: null,
      
      // Error tracking
      lastError: null,
      errorCount: 0
    }
  }

  setupValidationRules() {
    this.validationRules.set('currentNodeExists', (state) => {
      const nodeExists = state.storyGraph.nodes.some(n => n.id === state.currentNodeId)
      return {
        valid: nodeExists,
        message: nodeExists ? 'Current node exists' : `Current node '${state.currentNodeId}' not found in graph`
      }
    })

    this.validationRules.set('edgesValid', (state) => {
      const nodeIds = new Set(state.storyGraph.nodes.map(n => n.id))
      const invalidEdges = state.storyGraph.edges.filter(edge => {
        // Allow edges pointing to nodes that will be generated (marked with 'generated' flag)
        const fromExists = nodeIds.has(edge.fromId)
        const toExists = nodeIds.has(edge.toId) || edge.generated
        return !fromExists || !toExists
      })
      return {
        valid: invalidEdges.length === 0,
        message: invalidEdges.length === 0 ? 'All edges valid' : `${invalidEdges.length} invalid edges found`
      }
    })

    this.validationRules.set('phaseConsistent', (state) => {
      const validPhases = ['waiting_for_choices', 'generating_choices', 'choosing', 'generating_node', 'advancing']
      const phaseValid = validPhases.includes(state.phase)
      return {
        valid: phaseValid,
        message: phaseValid ? 'Phase is valid' : `Invalid phase: ${state.phase}`
      }
    })

    this.validationRules.set('visitedNodesExist', (state) => {
      const nodeIds = new Set(state.storyGraph.nodes.map(n => n.id))
      const invalidVisited = Array.from(state.visitedNodes).filter(id => !nodeIds.has(id))
      return {
        valid: invalidVisited.length === 0,
        message: invalidVisited.length === 0 ? 'All visited nodes exist' : `${invalidVisited.length} visited nodes don't exist`
      }
    })
  }

  // State validation
  validateState() {
    const results = {}
    let allValid = true

    for (const [ruleName, rule] of this.validationRules) {
      try {
        const result = rule(this.state)
        results[ruleName] = result
        if (!result.valid) {
          allValid = false
        }
      } catch (error) {
        results[ruleName] = {
          valid: false,
          message: `Validation error: ${error.message}`
        }
        allValid = false
      }
    }

    return {
      valid: allValid,
      rules: results,
      summary: allValid ? 'All validations passed' : 'Some validations failed'
    }
  }

  // Event system
  addListener(listener) {
    this.listeners.add(listener)
    return () => this.listeners.delete(listener)
  }

  emit(event, data = {}) {
    const eventData = {
      type: event,
      timestamp: Date.now(),
      state: this.getState(),
      ...data
    }
    
    this.listeners.forEach(listener => {
      try {
        listener(eventData)
      } catch (error) {
        console.error('Listener error:', error)
      }
    })
  }

  // Logging
  log(message, data = {}) {
    if (this.options.logActions) {
      console.log(`[GameEngine] ${message}`, data)
    }
    
    this.state.actionLog.push({
      message,
      timestamp: new Date(),
      data
    })

    this.emit('log', { message, data })
  }

  // Action recording
  recordAction(action, input, result) {
    const actionRecord = {
      action,
      input,
      result,
      timestamp: Date.now(),
      stateBefore: JSON.parse(JSON.stringify(this.state)),
      validation: this.validateState()
    }
    
    this.actionHistory.push(actionRecord)
    
    if (this.options.debugMode) {
      console.log('Action recorded:', actionRecord)
    }
  }

  // State access
  getState() {
    return JSON.parse(JSON.stringify(this.state))
  }

  getCurrentNode() {
    return this.state.storyGraph.nodes.find(n => n.id === this.state.currentNodeId)
  }

  getCurrentChoices() {
    return this.state.storyGraph.edges.filter(e => e.fromId === this.state.currentNodeId)
  }

  canAdvance() {
    const validation = this.validateState()
    
    // Auto-fix state if invalid and auto-fixing is enabled (default: enabled)
    if (!validation.valid && this.options.autoFix === true) {
      this.log('State validation failed, attempting auto-fix')
      const fixCount = this.autoFixState()
      if (fixCount > 0) {
        // Re-validate after auto-fix
        const reValidation = this.validateState()
        if (reValidation.valid) {
          this.log('Auto-fix successful, state is now valid')
        }
        return this.canAdvance() // Recursive call with fixed state
      }
    }
    
    const hasChoices = this.getCurrentChoices().length > 0
    const notGenerating = !this.state.generationInProgress
    const notGameOver = !this.state.isGameOver
    
    return {
      canAdvance: validation.valid && (hasChoices || this.state.phase === 'waiting_for_choices') && notGenerating && notGameOver,
      reasons: {
        validState: validation.valid,
        hasChoicesOrWaiting: hasChoices || this.state.phase === 'waiting_for_choices',
        notGenerating,
        notGameOver,
        validation
      }
    }
  }

  // Core actions
  async generateChoices() {
    this.log('Starting choice generation')
    
    // Pre-flight validation and auto-fix
    const validation = this.validateState()
    if (!validation.valid && this.options.autoFix === true) {
      this.log('State validation failed during choice generation, attempting auto-fix')
      this.autoFixState()
    }
    
    // Validate pre-conditions
    if (this.state.phase !== 'waiting_for_choices' && this.state.phase !== 'choosing') {
      throw new Error(`Cannot generate choices in phase: ${this.state.phase}`)
    }

    if (this.state.generationInProgress) {
      throw new Error('Generation already in progress')
    }

    const existingChoices = this.getCurrentChoices()
    if (existingChoices.length > 0) {
      this.log('Choices already exist, skipping generation')
      // If we're not in choosing phase, update it
      if (this.state.phase !== 'choosing') {
        this.state.phase = 'choosing'
      }
      return existingChoices
    }

    const currentNode = this.getCurrentNode()
    if (!currentNode) {
      // Auto-fix: If current node is missing, reset to start
      this.log(`Current node '${this.state.currentNodeId}' not found, resetting to start`)
      this.state.currentNodeId = 'start'
      this.state.phase = 'waiting_for_choices'
      this.autoFixState()
      
      const fixedCurrentNode = this.getCurrentNode()
      if (!fixedCurrentNode) {
        throw new Error(`Unable to recover: start node not found in graph`)
      }
      return this.generateChoices() // Retry with fixed state
    }

    try {
      // Update state
      this.state.phase = 'generating_choices'
      this.state.generationInProgress = true
      this.state.currentGeneration = {
        type: 'choices',
        nodeId: this.state.currentNodeId,
        startTime: Date.now()
      }
      
      this.emit('generationStarted', { type: 'choices', nodeId: this.state.currentNodeId })
      this.log('Choice generation started', { nodeId: this.state.currentNodeId })

      // Generate choices using AI service
      const context = {
        gameState: this.getState(),
        visitedNodes: Array.from(this.state.visitedNodes),
        currentGraph: this.state.storyGraph
      }

      const result = await this.aiService.generateChoices(currentNode, context)
      
      if (!result.choices || result.choices.length === 0) {
        throw new Error('AI service returned no choices')
      }

      // Create edges for each choice
      const newEdges = result.choices.map((choice, index) => {
        const newNodeId = `${this.state.currentNodeId}_choice_${index + 1}_${Date.now()}`
        
        return {
          id: `${this.state.currentNodeId}_to_${newNodeId}`,
          fromId: this.state.currentNodeId,
          toId: newNodeId,
          text: choice.text,
          icon: choice.icon,
          cost: choice.cost || 0,
          experience: choice.experience || 10,
          requiresItem: choice.requiresItem,
          risk: choice.risk || 'low',
          generated: true,
          confidence: result.confidence || 0.8
        }
      })

      // Add edges to graph
      this.state.storyGraph.edges.push(...newEdges)
      
      // Update state
      this.state.phase = 'choosing'
      this.state.generationInProgress = false
      this.state.currentGeneration = null
      
      this.log(`Generated ${newEdges.length} choices`, { 
        choices: newEdges.length, 
        confidence: result.confidence 
      })
      
      this.emit('generationCompleted', { 
        type: 'choices', 
        count: newEdges.length, 
        choices: newEdges 
      })

      this.recordAction('generateChoices', { nodeId: this.state.currentNodeId }, { 
        success: true, 
        choiceCount: newEdges.length 
      })

      return newEdges

    } catch (error) {
      this.state.phase = 'waiting_for_choices'
      this.state.generationInProgress = false
      this.state.currentGeneration = null
      this.state.lastError = error.message
      this.state.errorCount++
      
      this.log(`Choice generation failed: ${error.message}`)
      this.emit('generationFailed', { type: 'choices', error: error.message })
      
      this.recordAction('generateChoices', { nodeId: this.state.currentNodeId }, { 
        success: false, 
        error: error.message 
      })

      throw error
    }
  }

  async makeChoice(choiceId) {
    this.log('Making choice', { choiceId })

    // Validate pre-conditions
    if (this.state.phase !== 'choosing') {
      throw new Error(`Cannot make choice in phase: ${this.state.phase}`)
    }

    if (this.state.generationInProgress) {
      throw new Error('Cannot make choice while generation in progress')
    }

    const choice = this.state.storyGraph.edges.find(e => e.id === choiceId)
    if (!choice) {
      throw new Error(`Choice '${choiceId}' not found`)
    }

    if (!this.canMakeChoice(choice)) {
      throw new Error(`Cannot afford choice: ${choice.text}`)
    }

    try {
      this.state.phase = 'advancing'
      this.emit('choiceStarted', { choice })

      // Check if target node exists
      let targetNode = this.state.storyGraph.nodes.find(n => n.id === choice.toId)
      
      if (!targetNode) {
        // Generate new node
        this.log('Generating new node for choice', { choiceId: choice.id, toId: choice.toId })
        targetNode = await this.generateNode(choice)
      }

      // Apply choice consequences
      this.applyChoiceConsequences(choice)

      // Navigate to new node
      this.state.currentNodeId = choice.toId
      this.state.visitedNodes.add(choice.toId)
      
      // Clear choices for new node (will need regeneration)  
      // Only clear edges FROM the new node, not TO it
      this.state.storyGraph.edges = this.state.storyGraph.edges.filter(e => e.fromId !== choice.toId)
      
      // Update phase
      this.state.phase = 'waiting_for_choices'
      
      this.log('Choice completed', { 
        choice: choice.text, 
        newLocation: targetNode.location,
        newNodeId: choice.toId
      })
      
      this.emit('choiceCompleted', { 
        choice, 
        targetNode, 
        newLocation: targetNode.location 
      })

      this.recordAction('makeChoice', { choiceId, choice }, { 
        success: true, 
        newNodeId: choice.toId,
        newLocation: targetNode.location
      })

      return {
        success: true,
        targetNode,
        newNodeId: choice.toId
      }

    } catch (error) {
      this.state.phase = 'choosing'
      this.state.lastError = error.message
      this.state.errorCount++
      
      this.log(`Choice failed: ${error.message}`)
      this.emit('choiceFailed', { choice, error: error.message })
      
      this.recordAction('makeChoice', { choiceId, choice }, { 
        success: false, 
        error: error.message 
      })

      throw error
    }
  }

  async generateNode(choice) {
    this.log('Generating new node', { choiceId: choice.id, toId: choice.toId })

    try {
      this.state.generationInProgress = true
      this.state.currentGeneration = {
        type: 'node',
        choiceId: choice.id,
        toId: choice.toId,
        startTime: Date.now()
      }

      this.emit('generationStarted', { type: 'node', choiceId: choice.id })

      const fromNode = this.getCurrentNode()
      if (!fromNode) {
        throw new Error(`Source node '${this.state.currentNodeId}' not found`)
      }

      const context = {
        gameState: this.getState(),
        visitedNodes: Array.from(this.state.visitedNodes),
        currentGraph: this.state.storyGraph
      }

      const result = await this.aiService.generateStoryNode(choice, fromNode, context)
      
      if (!result.location || !result.situation) {
        throw new Error('AI service returned invalid node data')
      }

      const newNode = {
        id: choice.toId,
        location: result.location,
        situation: result.situation,
        generated: true,
        confidence: result.confidence || 0.8,
        generatedFrom: choice.id
      }

      this.state.storyGraph.nodes.push(newNode)
      this.state.generationInProgress = false
      this.state.currentGeneration = null

      this.log('Node generated successfully', { 
        nodeId: newNode.id, 
        location: newNode.location,
        confidence: result.confidence
      })

      this.emit('generationCompleted', { type: 'node', node: newNode })

      return newNode

    } catch (error) {
      this.state.generationInProgress = false
      this.state.currentGeneration = null
      
      this.log(`Node generation failed: ${error.message}`)
      this.emit('generationFailed', { type: 'node', error: error.message })

      throw error
    }
  }

  canMakeChoice(choice) {
    // Check gold requirement
    if (choice.cost > this.state.gold) {
      return false
    }

    // Check item requirement
    if (choice.requiresItem) {
      return this.state.inventory.some(item => item.name === choice.requiresItem)
    }

    return true
  }

  applyChoiceConsequences(choice) {
    this.log('Applying choice consequences', { choice: choice.text })

    // Deduct cost
    if (choice.cost > 0) {
      this.state.gold -= choice.cost
      this.log(`Spent ${choice.cost} gold`)
    }

    // Consume required item
    if (choice.requiresItem) {
      const itemIndex = this.state.inventory.findIndex(item => item.name === choice.requiresItem)
      if (itemIndex >= 0) {
        const item = this.state.inventory[itemIndex]
        if (item.consumable) {
          if (item.quantity > 1) {
            item.quantity--
          } else {
            this.state.inventory.splice(itemIndex, 1)
          }
          this.log(`Consumed ${choice.requiresItem}`)
        }
      }
    }

    // Add experience
    if (choice.experience > 0) {
      this.state.experience += choice.experience
      this.log(`Gained ${choice.experience} experience`)

      // Check for level up
      const requiredExp = this.state.level * 100
      if (this.state.experience >= requiredExp) {
        this.state.level++
        this.state.gold += 50 // Level up bonus
        this.log(`Level up! Now level ${this.state.level}`)
        this.emit('levelUp', { newLevel: this.state.level })
      }
    }

    // Check for winning condition
    if (choice.isWinning) {
      this.state.isGameOver = true
      this.state.isWin = true
      this.state.endMessage = 'You have mastered software architecture!'
      this.state.gold += 100 // Victory bonus
      this.log('Game won!')
      this.emit('gameWon', { endMessage: this.state.endMessage })
    }
  }

  // Utility methods
  addToInventory(item) {
    const existing = this.state.inventory.find(i => i.id === item.id)
    if (existing && item.consumable) {
      existing.quantity = (existing.quantity || 1) + (item.quantity || 1)
    } else {
      this.state.inventory.push({ ...item, quantity: item.quantity || 1 })
    }
    this.log(`Added to inventory: ${item.name}`)
  }

  reset() {
    this.log('Resetting game engine')
    this.state = this.createInitialState()
    this.actionHistory = []
    this.emit('gameReset')
  }

  // Debug methods
  getDebugInfo() {
    return {
      state: this.getState(),
      validation: this.validateState(),
      actionHistory: this.actionHistory,
      currentChoices: this.getCurrentChoices(),
      canAdvance: this.canAdvance()
    }
  }

  exportState() {
    return {
      gameState: this.getState(),
      actionHistory: this.actionHistory,
      validation: this.validateState()
    }
  }

  // Clean up invalid edges to fix state consistency
  cleanupInvalidEdges() {
    this.log('Cleaning up invalid edges')
    const nodeIds = new Set(this.state.storyGraph.nodes.map(n => n.id))
    const initialEdgeCount = this.state.storyGraph.edges.length
    
    this.state.storyGraph.edges = this.state.storyGraph.edges.filter(edge => {
      // Keep edges if both nodes exist, or if it's a generated edge pointing to a future node
      const fromExists = nodeIds.has(edge.fromId)
      const toExists = nodeIds.has(edge.toId) || edge.generated
      const isValid = fromExists && toExists
      
      if (!isValid) {
        this.log(`Removing invalid edge: ${edge.id} (${edge.fromId} -> ${edge.toId})`)
      }
      return isValid
    })
    
    const removedCount = initialEdgeCount - this.state.storyGraph.edges.length
    if (removedCount > 0) {
      this.log(`Cleaned up ${removedCount} invalid edges`)
      this.emit('edgesCleanedUp', { removedCount })
    }
    
    return removedCount
  }

  // Auto-fix state validation issues
  autoFixState() {
    this.log('Running auto-fix for state validation')
    let fixCount = 0
    
    // 1. Clean up invalid edges
    fixCount += this.cleanupInvalidEdges()
    
    // 2. Fix invalid current node - reset to start if current node doesn't exist
    if (!this.state.storyGraph.nodes.some(n => n.id === this.state.currentNodeId)) {
      this.log(`Fixing invalid current node: ${this.state.currentNodeId} -> start`)
      this.state.currentNodeId = 'start'
      this.state.phase = 'waiting_for_choices'
      fixCount++
    }
    
    // 3. Clean up invalid visited nodes
    const nodeIds = new Set(this.state.storyGraph.nodes.map(n => n.id))
    const invalidVisited = Array.from(this.state.visitedNodes).filter(id => !nodeIds.has(id))
    if (invalidVisited.length > 0) {
      this.log(`Removing ${invalidVisited.length} invalid visited nodes`)
      invalidVisited.forEach(id => this.state.visitedNodes.delete(id))
      fixCount += invalidVisited.length
    }
    
    // 4. Ensure start node is always visited
    if (!this.state.visitedNodes.has('start')) {
      this.state.visitedNodes.add('start')
      this.log('Added start node to visited nodes')
      fixCount++
    }
    
    if (fixCount > 0) {
      this.log(`Auto-fix completed: ${fixCount} issues resolved`)
      this.emit('stateAutoFixed', { fixCount })
    }
    
    return fixCount
  }
}