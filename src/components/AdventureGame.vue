<template>
  <div class="adventure-game">
    <!-- Global Error Bar -->
    <div v-if="globalError" class="error-bar">
      <div class="error-content">
        <span class="error-icon">⚠️</span>
        <span class="error-message">{{ globalError }}</span>
        <button @click="clearError" class="error-close">✕</button>
      </div>
    </div>

    <!-- Global Loading Bar -->
    <div v-if="isLoading" class="loading-bar">
      <div class="loading-content">
        <div class="spinner"></div>
        <span class="loading-message">{{ loadingMessage }}</span>
      </div>
    </div>

    <div class="game-header">
      <h3>🗡️ The Software Architecture Quest</h3>
      <div class="game-stats">
        <div class="stat">
          <span class="stat-label">Level:</span>
          <span class="stat-value">{{ gameState.level }}</span>
        </div>
        <div class="stat">
          <span class="stat-label">Experience:</span>
          <span class="stat-value">{{ gameState.experience }}</span>
        </div>
        <div class="stat">
          <span class="stat-label">Gold:</span>
          <span class="stat-value">{{ gameState.gold }}💰</span>
        </div>
      </div>
    </div>

    <div class="game-content">
      <!-- Left Panel: Story and Choices -->
      <div class="story-panel">
        <div class="story-section">
          <div class="current-location">
            📍 {{ currentNode.location }}
          </div>
          <div class="story-text" v-html="currentNode.situation"></div>
          
          <!-- Adventure Choices -->
          <div class="choices-section" v-if="!gameState.isGameOver">
            <h4>What do you do?</h4>
            
            <!-- Show choices if available -->
            <div v-if="currentChoices.length > 0" class="choices-grid">
              <button
                v-for="choice in currentChoices"
                :key="choice.id"
                @click="makeChoice(choice)"
                :disabled="!canMakeChoice(choice)"
                :class="['choice-btn', {
                  'disabled': !canMakeChoice(choice),
                  'premium': choice.requiresItem,
                  'risky': choice.risk === 'high'
                }]"
              >
                <div class="choice-icon">{{ choice.icon }}</div>
                <div class="choice-text">{{ choice.text }}</div>
                <div class="choice-requirements" v-if="choice.requiresItem || choice.cost">
                  <span v-if="choice.requiresItem">🔑 {{ choice.requiresItem }}</span>
                  <span v-if="choice.cost">💰 {{ choice.cost }}</span>
                </div>
              </button>
            </div>
            
            <!-- Show generation button if no choices -->
            <div v-else class="no-choices">
              <p>🎲 Ready to explore your options?</p>
              <button @click="generateChoicesForNode(gameState.currentNodeId)" class="generate-btn">
                ✨ Generate Adventure Choices
              </button>
            </div>
          </div>

          <!-- Game Over -->
          <div v-if="gameState.isGameOver" class="game-over">
            <h3>{{ gameState.isWin ? '🎉 Victory!' : '💀 Game Over' }}</h3>
            <p>{{ gameState.endMessage }}</p>
            <button @click="restartGame" class="restart-btn">🔄 Start New Adventure</button>
          </div>
        </div>

        <!-- Inventory -->
        <div class="inventory-section">
          <h4>🎒 Inventory</h4>
          <div class="inventory-grid">
            <div 
              v-for="item in gameState.inventory" 
              :key="item.id"
              :class="['inventory-item', { 'consumable': item.consumable }]"
              :title="item.description"
            >
              <div class="item-icon">{{ item.icon }}</div>
              <div class="item-name">{{ item.name }}</div>
              <div v-if="item.quantity > 1" class="item-quantity">{{ item.quantity }}</div>
            </div>
          </div>
          <div v-if="gameState.inventory.length === 0" class="empty-inventory">
            Your inventory is empty
          </div>
        </div>
      </div>

      <!-- Right Panel: Graph Visualization -->
      <div class="graph-panel">
        <div class="graph-header">
          <h4>🗺️ Adventure Map</h4>
          <div class="graph-controls">
            <button @click="resetGraphLayout" class="graph-btn">🔄 Reset</button>
            <select v-model="layoutMode" @change="updateGraphLayout" class="layout-select">
              <option value="breadthfirst">Tree</option>
              <option value="circle">Circle</option>
              <option value="grid">Grid</option>
              <option value="random">Random</option>
            </select>
          </div>
        </div>
        <div class="graph-container" ref="graphContainer"></div>
      </div>
    </div>

    <!-- Action Log -->
    <div class="action-log">
      <h4>📜 Adventure Log</h4>
      <div class="log-entries">
        <div 
          v-for="(entry, index) in gameState.actionLog.slice(-5)" 
          :key="index"
          class="log-entry"
        >
          <span class="log-time">{{ formatTime(entry.timestamp) }}</span>
          <span class="log-text">{{ entry.message }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import cytoscape from 'cytoscape'

export default {
  name: 'AdventureGame',
  props: {
    aiService: {
      type: Object,
      required: true
    }
  },
  data() {
    return {
      cy: null,
      layoutMode: 'breadthfirst',
      gameState: {
        currentNodeId: 'start',
        level: 1,
        experience: 0,
        gold: 50,
        inventory: [],
        visitedNodes: new Set(['start']),
        isGameOver: false,
        isWin: false,
        endMessage: '',
        actionLog: []
      },
      // UI state
      isLoading: false,
      loadingMessage: '',
      globalError: null,
      storyGraph: {
        nodes: [
          {
            id: 'start',
            location: 'The Town Square of Architectura',
            situation: `
              <p>Welcome to <strong>Architectura</strong>, where software systems come to life! You're a new architect arriving in the mystical town square.</p>
              <p>An old wizard approaches: <em>"Young architect, what kind of system challenges interest you most? Your choice will shape your adventure..."</em></p>
            `
          }
        ],
        edges: []
      },
      shopItems: [
        {
          id: 'architecture_compass',
          name: 'Architecture Compass',
          icon: '🧭',
          description: 'Helps navigate complex system design decisions',
          cost: 25,
          consumable: false
        },
        {
          id: 'system_design_scroll',
          name: 'System Design Scroll',
          icon: '📜',
          description: 'Contains ancient patterns for building scalable systems',
          cost: 35,
          consumable: false
        },
        {
          id: 'debugging_potion',
          name: 'Debugging Potion',
          icon: '🧪',
          description: 'Reveals hidden bugs and system issues',
          cost: 15,
          consumable: true,
          quantity: 1
        }
      ]
    }
  },
  computed: {
    currentNode() {
      return this.storyGraph.nodes.find(node => node.id === this.gameState.currentNodeId) || this.storyGraph.nodes[0]
    },
    currentChoices() {
      return this.storyGraph.edges.filter(edge => edge.fromId === this.gameState.currentNodeId)
    }
  },
  async mounted() {
    // Try to load saved game first
    const gameLoaded = this.loadGameState()
    
    this.initializeGraph()
    
    if (gameLoaded) {
      this.addToLog('🎮 Welcome back, adventurer!')
      this.updateGraph()
      // If loaded game has choices, keep them. Otherwise show generation button.
      if (this.currentChoices.length === 0) {
        this.addToLog('🎲 Click the button below to generate new choices!')
      }
    } else {
      this.addToLog('🎮 Adventure begins in the town square...')
      this.addToLog('🌟 Click "Generate Adventure Choices" to start exploring!')
    }
    
    // Debug: Log current state
    console.log('Adventure game mounted, current choices:', this.currentChoices.length)
    console.log('Story graph edges:', this.storyGraph.edges.length)
  },
  methods: {
    showError(message) {
      this.globalError = message
      this.addToLog(`❌ Error: ${message}`)
    },

    clearError() {
      this.globalError = null
    },

    setLoading(loading, message = '') {
      this.isLoading = loading
      this.loadingMessage = message
    },

    async generateChoicesForNode(nodeId) {
      // If already has choices, don't regenerate
      const existingChoices = this.storyGraph.edges.filter(edge => edge.fromId === nodeId)
      if (existingChoices.length > 0) {
        this.addToLog('ℹ️ Choices already exist for this location')
        return existingChoices
      }

      try {
        const currentNode = this.storyGraph.nodes.find(n => n.id === nodeId)
        if (!currentNode) {
          throw new Error(`Node not found: ${nodeId}`)
        }

        // Use AI service to generate choices
        const result = await this.aiService.generateChoices(currentNode, {
          gameState: this.gameState,
          visitedNodes: Array.from(this.gameState.visitedNodes),
          currentGraph: this.storyGraph
        })

        if (!result.choices || result.choices.length === 0) {
          throw new Error('No choices generated by AI service')
        }

        // Create edges for each generated choice
        result.choices.forEach((choice, index) => {
          const newNodeId = `${nodeId}_choice_${index + 1}_${Date.now()}`
          
          const edge = {
            id: `${nodeId}_to_${newNodeId}`,
            fromId: nodeId,
            toId: newNodeId,
            text: choice.text,
            icon: choice.icon,
            cost: choice.cost || 0,
            experience: choice.experience || 10,
            requiresItem: choice.requiresItem,
            risk: choice.risk || 'low'
          }
          
          this.storyGraph.edges.push(edge)
        })

        this.addToLog(`🎲 Generated ${result.choices.length} new choices (confidence: ${Math.round(result.confidence * 100)}%)`)
        this.updateGraph()
        
        return this.storyGraph.edges.filter(edge => edge.fromId === nodeId)
        
      } catch (error) {
        console.error('Error generating choices:', error)
        this.showError(`Failed to generate choices: ${error.message}`)
        return []
      }
    },

    async generateNodeFromChoice(choice) {
      try {
        const fromNode = this.storyGraph.nodes.find(n => n.id === choice.fromId)
        if (!fromNode) {
          throw new Error(`Source node not found: ${choice.fromId}`)
        }

        // Use AI service to generate new story node
        const result = await this.aiService.generateStoryNode(choice, fromNode, {
          gameState: this.gameState,
          visitedNodes: Array.from(this.gameState.visitedNodes),
          currentGraph: this.storyGraph
        })

        if (!result.location || !result.situation) {
          throw new Error('Invalid story node generated by AI service')
        }

        // Create the new node
        const newNode = {
          id: choice.toId,
          location: result.location,
          situation: result.situation
        }

        this.storyGraph.nodes.push(newNode)
        this.addToLog(`🏗️ Generated: ${newNode.location} (confidence: ${Math.round(result.confidence * 100)}%)`)
        
        return newNode
        
      } catch (error) {
        console.error('Error generating node:', error)
        this.showError(`Failed to generate new location: ${error.message}`)
        return null
      }
    },

    async makeChoice(choice) {
      if (!this.canMakeChoice(choice)) return

      try {
        // Check if target node exists, generate if not
        const targetNode = this.storyGraph.nodes.find(n => n.id === choice.toId)
        if (!targetNode) {
          const newNode = await this.generateNodeFromChoice(choice)
          if (!newNode) {
            this.showError('Failed to generate new scenario')
            return
          }
        }

      // Process costs and requirements (existing logic)
      if (choice.cost > 0) {
        this.gameState.gold -= choice.cost
      }

      if (choice.requiresItem) {
        const item = this.gameState.inventory.find(i => i.name === choice.requiresItem)
        if (item && item.consumable) {
          this.removeFromInventory(item.id)
        }
      }

      // Add experience and check level up
      this.gameState.experience += choice.experience || 0
      
      if (this.gameState.experience >= this.gameState.level * 100) {
        this.gameState.level++
        this.gameState.gold += 50
        this.addToLog(`🆙 Level up! You are now level ${this.gameState.level}`)
      }

      // Navigate to new node
      this.gameState.currentNodeId = choice.toId
      this.gameState.visitedNodes.add(choice.toId)

      // Generate new choices for the new node
      await this.generateChoicesForNode(choice.toId)

      // Check for special conditions
      if (choice.isWinning) {
        this.gameState.isGameOver = true
        this.gameState.isWin = true
        this.gameState.endMessage = 'You have mastered software architecture!'
        this.gameState.gold += 100
      }

      // Log the action
      this.addToLog(`${choice.icon} ${choice.text} (+${choice.experience} XP)`)

        // Update graph and save
        this.updateGraph()
        this.saveGameState()
        this.trackProgress(choice)
        
      } catch (error) {
        console.error('Error making choice:', error)
        this.showError(`Failed to process choice: ${error.message}`)
      }
    },
    initializeGraph() {
      const container = this.$refs.graphContainer
      if (!container) return

      this.cy = cytoscape({
        container: container,
        style: [
          {
            selector: 'node',
            style: {
              'background-color': (ele) => {
                if (ele.id() === this.gameState.currentNodeId) return '#e74c3c'
                if (this.gameState.visitedNodes.has(ele.id())) return '#3498db'
                return '#95a5a6'
              },
              'label': 'data(label)',
              'text-valign': 'center',
              'text-halign': 'center',
              'color': '#fff',
              'font-size': '12px',
              'font-weight': 'bold',
              'width': '60px',
              'height': '60px',
              'border-width': '2px',
              'border-color': '#34495e'
            }
          },
          {
            selector: 'edge',
            style: {
              'line-color': (ele) => {
                const edge = this.storyGraph.edges.find(e => e.id === ele.id())
                if (edge?.risk === 'high') return '#e74c3c'
                if (edge?.isWinning) return '#27ae60'
                return '#7f8c8d'
              },
              'target-arrow-color': '#7f8c8d',
              'target-arrow-shape': 'triangle',
              'curve-style': 'bezier',
              'label': 'data(label)',
              'font-size': '10px',
              'text-rotation': 'autorotate'
            }
          }
        ],
        elements: this.buildGraphElements(),
        layout: {
          name: this.layoutMode,
          directed: true,
          padding: 10
        }
      })

      this.cy.on('tap', 'node', (event) => {
        const nodeId = event.target.id()
        console.log('Clicked node:', nodeId)
      })
    },

    buildGraphElements() {
      const nodes = this.storyGraph.nodes.map(node => ({
        data: {
          id: node.id,
          label: node.location.split(' ').slice(-2).join(' ')
        }
      }))

      const edges = this.storyGraph.edges.map(edge => ({
        data: {
          id: edge.id,
          source: edge.fromId,
          target: edge.toId,
          label: edge.icon
        }
      }))

      return [...nodes, ...edges]
    },


    canMakeChoice(choice) {
      // Check gold requirement
      if (choice.cost > this.gameState.gold) return false

      // Check item requirement
      if (choice.requiresItem) {
        return this.gameState.inventory.some(item => item.name === choice.requiresItem)
      }

      return true
    },

    addToInventory(item) {
      const existing = this.gameState.inventory.find(i => i.id === item.id)
      if (existing && item.consumable) {
        existing.quantity = (existing.quantity || 1) + (item.quantity || 1)
      } else {
        this.gameState.inventory.push({ ...item, quantity: item.quantity || 1 })
      }
    },

    removeFromInventory(itemId) {
      const index = this.gameState.inventory.findIndex(i => i.id === itemId)
      if (index > -1) {
        const item = this.gameState.inventory[index]
        if (item.quantity > 1) {
          item.quantity--
        } else {
          this.gameState.inventory.splice(index, 1)
        }
      }
    },

    updateGraph() {
      if (!this.cy) return

      // Update node colors based on current position and visited status
      this.cy.nodes().forEach(node => {
        const nodeId = node.id()
        let color = '#95a5a6' // Default: unvisited
        
        if (nodeId === this.gameState.currentNodeId) {
          color = '#e74c3c' // Current position: red
        } else if (this.gameState.visitedNodes.has(nodeId)) {
          color = '#3498db' // Visited: blue
        }
        
        node.style('background-color', color)
      })
    },

    updateGraphLayout() {
      if (!this.cy) return
      this.cy.layout({ name: this.layoutMode, directed: true, padding: 10 }).run()
    },

    resetGraphLayout() {
      this.updateGraphLayout()
    },

    restartGame() {
      // Clear saved game data
      this.clearSavedGame()
      
      this.gameState = {
        currentNodeId: 'start',
        level: 1,
        experience: 0,
        gold: 50,
        inventory: [],
        visitedNodes: new Set(['start']),
        isGameOver: false,
        isWin: false,
        endMessage: '',
        actionLog: []
      }
      this.addToLog('🎮 New adventure begins!')
      this.updateGraph()
      this.saveGameState()
    },

    addToLog(message) {
      this.gameState.actionLog.push({
        message,
        timestamp: new Date()
      })
    },

    formatTime(timestamp) {
      // Ensure timestamp is a Date object
      const date = timestamp instanceof Date ? timestamp : new Date(timestamp)
      return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    },

    async trackProgress(choice) {
      // API call to track user progress and choices
      try {
        const response = await fetch('http://localhost:3001/api/adventure/track', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            choiceId: choice.id,
            fromNode: choice.fromId,
            toNode: choice.toId,
            sessionId: this.getSessionId(),
            gameState: {
              level: this.gameState.level,
              experience: this.gameState.experience,
              gold: this.gameState.gold,
              currentNodeId: this.gameState.currentNodeId,
              inventoryCount: this.gameState.inventory.length
            }
          })
        })
        
        if (response.ok) {
          const data = await response.json()
          this.setSessionId(data.sessionId)
          this.addToLog(`📡 Progress synced (Choice #${data.totalChoices})`)
          console.log('Progress tracked:', data)
        }
      } catch (error) {
        this.addToLog('📡 Offline mode - progress not synced')
        console.log('Tracking failed (offline mode):', error.message)
      }
    },

    getSessionId() {
      // Get or create session ID for this browser session
      let sessionId = localStorage.getItem('adventure_session_id')
      if (!sessionId) {
        sessionId = `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
        localStorage.setItem('adventure_session_id', sessionId)
      }
      return sessionId
    },

    setSessionId(sessionId) {
      localStorage.setItem('adventure_session_id', sessionId)
    },

    saveGameState() {
      // Save game state to localStorage for persistence
      localStorage.setItem('adventure_game_state', JSON.stringify({
        ...this.gameState,
        visitedNodes: Array.from(this.gameState.visitedNodes)
      }))
    },

    loadGameState() {
      // Load game state from localStorage
      try {
        const saved = localStorage.getItem('adventure_game_state')
        if (saved) {
          const parsedState = JSON.parse(saved)
          
          // Validate that currentNodeId exists in current graph
          const nodeIds = new Set(this.storyGraph.nodes.map(n => n.id))
          if (!nodeIds.has(parsedState.currentNodeId)) {
            console.warn(`Saved currentNodeId "${parsedState.currentNodeId}" not found in graph. Resetting to start.`)
            this.addToLog('⚠️ Saved location not found, returning to town square')
            parsedState.currentNodeId = 'start'
          }
          
          // Validate visitedNodes exist in current graph
          const validVisitedNodes = (parsedState.visitedNodes || ['start']).filter(nodeId => 
            nodeIds.has(nodeId)
          )
          if (validVisitedNodes.length !== (parsedState.visitedNodes || []).length) {
            console.warn('Some visited nodes not found in current graph, cleaning up')
            this.addToLog('🧹 Cleaned up invalid visited locations')
          }
          
          // Restore Date objects for action log
          const actionLog = (parsedState.actionLog || []).map(entry => ({
            ...entry,
            timestamp: new Date(entry.timestamp)
          }))
          
          this.gameState = {
            ...parsedState,
            currentNodeId: parsedState.currentNodeId,
            visitedNodes: new Set(validVisitedNodes),
            actionLog: actionLog
          }
          
          // In procedural system, we need to clear edges that reference non-existent nodes
          this.cleanupInvalidEdges()
          
          this.addToLog('💾 Game state restored and validated')
          return true
        }
      } catch (error) {
        console.log('Failed to load saved game:', error)
        this.addToLog('❌ Failed to restore saved game')
      }
      return false
    },

    cleanupInvalidEdges() {
      // Remove edges that reference nodes that don't exist
      const nodeIds = new Set(this.storyGraph.nodes.map(n => n.id))
      const validEdges = this.storyGraph.edges.filter(edge => 
        nodeIds.has(edge.fromId) && nodeIds.has(edge.toId)
      )
      
      if (validEdges.length !== this.storyGraph.edges.length) {
        const removedCount = this.storyGraph.edges.length - validEdges.length
        console.warn(`Removed ${removedCount} invalid edges`)
        this.addToLog(`🧹 Cleaned up ${removedCount} invalid navigation paths`)
        this.storyGraph.edges = validEdges
      }
    },

    clearSavedGame() {
      localStorage.removeItem('adventure_game_state')
      localStorage.removeItem('adventure_session_id')
    }
  }
}
</script>

<style scoped>
.adventure-game {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, #2c3e50 0%, #34495e 100%);
  border-radius: 12px;
  overflow: hidden;
  color: #ecf0f1;
}

.game-header {
  background: rgba(52, 73, 94, 0.9);
  padding: 15px 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 2px solid #3498db;
}

.game-header h3 {
  margin: 0;
  color: #3498db;
  font-size: 1.4em;
}

.game-stats {
  display: flex;
  gap: 20px;
}

.stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  min-width: 60px;
}

.stat-label {
  font-size: 0.8em;
  color: #bdc3c7;
  margin-bottom: 2px;
}

.stat-value {
  font-weight: bold;
  font-size: 1.1em;
  color: #f39c12;
}

.game-content {
  flex: 1;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0;
  min-height: 0;
}

.story-panel {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  background: rgba(44, 62, 80, 0.7);
  border-right: 1px solid #3498db;
}

.story-section {
  flex: 1;
}

.current-location {
  font-size: 1.1em;
  color: #3498db;
  margin-bottom: 15px;
  font-weight: bold;
}

.story-text {
  background: rgba(236, 240, 241, 0.1);
  padding: 15px;
  border-radius: 8px;
  line-height: 1.6;
  margin-bottom: 20px;
  border-left: 4px solid #3498db;
}

.story-text p {
  margin: 0 0 10px 0;
}

.story-text strong {
  color: #f39c12;
}

.story-text em {
  color: #e67e22;
  font-style: italic;
}

.choices-section h4 {
  color: #e74c3c;
  margin-bottom: 15px;
}

.choices-grid {
  display: grid;
  gap: 10px;
}

.choice-btn {
  background: rgba(52, 152, 219, 0.2);
  border: 2px solid #3498db;
  border-radius: 8px;
  padding: 12px;
  color: #ecf0f1;
  cursor: pointer;
  transition: all 0.3s ease;
  text-align: left;
  display: grid;
  grid-template-columns: auto 1fr auto;
  gap: 10px;
  align-items: center;
}

.choice-btn:hover:not(.disabled) {
  background: rgba(52, 152, 219, 0.4);
  border-color: #2980b9;
  transform: translateX(5px);
}

.choice-btn.disabled {
  opacity: 0.5;
  cursor: not-allowed;
  border-color: #7f8c8d;
}

.choice-btn.premium {
  border-color: #f39c12;
  background: rgba(243, 156, 18, 0.2);
}

.choice-btn.risky {
  border-color: #e74c3c;
  background: rgba(231, 76, 60, 0.2);
}

.choice-icon {
  font-size: 1.2em;
}

.choice-text {
  font-weight: 500;
}

.choice-requirements {
  font-size: 0.8em;
  color: #f39c12;
}

.inventory-section {
  background: rgba(44, 62, 80, 0.5);
  padding: 15px;
  border-radius: 8px;
  border: 1px solid #34495e;
}

.inventory-section h4 {
  color: #e67e22;
  margin-bottom: 10px;
}

.inventory-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
  gap: 8px;
}

.inventory-item {
  background: rgba(189, 195, 199, 0.1);
  border: 1px solid #7f8c8d;
  border-radius: 6px;
  padding: 8px;
  text-align: center;
  position: relative;
  transition: all 0.2s ease;
}

.inventory-item:hover {
  background: rgba(189, 195, 199, 0.2);
  border-color: #bdc3c7;
}

.item-icon {
  font-size: 1.5em;
  margin-bottom: 4px;
}

.item-name {
  font-size: 0.7em;
  color: #bdc3c7;
}

.item-quantity {
  position: absolute;
  top: -5px;
  right: -5px;
  background: #e74c3c;
  color: white;
  border-radius: 50%;
  width: 18px;
  height: 18px;
  font-size: 0.7em;
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-inventory {
  color: #7f8c8d;
  font-style: italic;
  text-align: center;
  padding: 20px;
}

.graph-panel {
  display: flex;
  flex-direction: column;
  background: rgba(44, 62, 80, 0.3);
}

.graph-header {
  padding: 15px 20px;
  background: rgba(52, 73, 94, 0.7);
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #34495e;
}

.graph-header h4 {
  margin: 0;
  color: #3498db;
}

.graph-controls {
  display: flex;
  gap: 10px;
  align-items: center;
}

.graph-btn {
  background: #3498db;
  color: white;
  border: none;
  padding: 5px 10px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.8em;
}

.layout-select {
  background: #34495e;
  color: #ecf0f1;
  border: 1px solid #3498db;
  padding: 5px;
  border-radius: 4px;
}

.graph-container {
  flex: 1;
  min-height: 300px;
  background: rgba(236, 240, 241, 0.05);
}

.action-log {
  background: rgba(44, 62, 80, 0.9);
  padding: 15px 20px;
  border-top: 1px solid #34495e;
  max-height: 120px;
}

.action-log h4 {
  margin: 0 0 10px 0;
  color: #e67e22;
  font-size: 1em;
}

.log-entries {
  max-height: 80px;
  overflow-y: auto;
}

.log-entry {
  display: flex;
  gap: 10px;
  margin-bottom: 5px;
  font-size: 0.85em;
}

.log-time {
  color: #7f8c8d;
  min-width: 60px;
}

.log-text {
  color: #bdc3c7;
}

.game-over {
  background: rgba(231, 76, 60, 0.2);
  border: 2px solid #e74c3c;
  border-radius: 8px;
  padding: 20px;
  text-align: center;
}

.game-over h3 {
  color: #e74c3c;
  margin-bottom: 10px;
}

.restart-btn {
  background: #27ae60;
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 1em;
  margin-top: 15px;
}

.restart-btn:hover {
  background: #229954;
}

.no-choices {
  text-align: center;
  padding: 20px;
  background: rgba(236, 240, 241, 0.1);
  border-radius: 8px;
  border: 2px dashed #3498db;
}

.no-choices p {
  margin-bottom: 15px;
  color: #3498db;
  font-style: italic;
}

.generate-btn {
  background: linear-gradient(45deg, #9b59b6, #3498db);
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 1em;
  font-weight: bold;
  transition: all 0.3s ease;
  box-shadow: 0 4px 8px rgba(0,0,0,0.2);
}

.generate-btn:hover {
  background: linear-gradient(45deg, #8e44ad, #2980b9);
  transform: translateY(-2px);
  box-shadow: 0 6px 12px rgba(0,0,0,0.3);
}

.generate-btn:active {
  transform: translateY(0);
}

/* Error Bar */
.error-bar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  background: linear-gradient(135deg, #e74c3c, #c0392b);
  color: white;
  z-index: 1000;
  box-shadow: 0 2px 8px rgba(0,0,0,0.3);
  animation: slideDown 0.3s ease-out;
}

.error-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.error-icon {
  font-size: 1.2em;
  margin-right: 10px;
}

.error-message {
  flex: 1;
  font-weight: 500;
}

.error-close {
  background: none;
  border: none;
  color: white;
  font-size: 1.2em;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: background 0.2s ease;
}

.error-close:hover {
  background: rgba(255,255,255,0.2);
}

/* Loading Bar */
.loading-bar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  background: linear-gradient(135deg, #3498db, #2980b9);
  color: white;
  z-index: 1000;
  box-shadow: 0 2px 8px rgba(0,0,0,0.3);
  animation: slideDown 0.3s ease-out;
}

.loading-content {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 12px 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.loading-message {
  margin-left: 12px;
  font-weight: 500;
}

/* Spinner */
.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid rgba(255,255,255,0.3);
  border-top: 2px solid white;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

@keyframes slideDown {
  from {
    transform: translateY(-100%);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}
</style>