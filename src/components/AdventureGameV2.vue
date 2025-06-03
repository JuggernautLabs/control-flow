<template>
  <div class="adventure-game-v2">
    <!-- Debug Panel (only in debug mode) -->
    <div v-if="debugMode" class="debug-panel">
      <h4>🔧 Debug Panel</h4>
      <div class="debug-info">
        <div><strong>Phase:</strong> {{ gameState.phase }}</div>
        <div><strong>Current Node:</strong> {{ gameState.currentNodeId }}</div>
        <div><strong>Can Advance:</strong> {{ canAdvance.canAdvance ? '✅' : '❌' }}</div>
        <div><strong>Validation:</strong> {{ validation.valid ? '✅' : '❌' }}</div>
        <div v-if="!validation.valid" class="validation-errors">
          <div class="validation-summary">
            <strong>Validation Issues:</strong>
            <button @click="fixGameState" class="fix-btn mini">🔧 Auto-fix All</button>
          </div>
          <div v-for="(rule, name) in validation.rules" :key="name">
            <span v-if="!rule.valid" class="error">{{ name }}: {{ rule.message }}</span>
          </div>
        </div>
      </div>
      <div class="debug-actions">
        <button @click="exportDebugData" class="debug-btn">Export Debug Data</button>
        <button @click="toggleDebugMode" class="debug-btn">Hide Debug</button>
      </div>
    </div>

    <!-- Error Display -->
    <div v-if="error" class="error-display">
      <span class="error-icon">⚠️</span>
      <span class="error-message">{{ error }}</span>
      <button @click="clearError" class="error-close">✕</button>
    </div>

    <!-- Loading Display -->
    <div v-if="loading" class="loading-display">
      <div class="spinner"></div>
      <span class="loading-message">{{ loadingMessage }}</span>
    </div>

    <!-- Game Header -->
    <div class="game-header">
      <h3>🗡️ The Software Architecture Quest v2</h3>
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
      <div class="debug-toggle">
        <button @click="toggleDebugMode" class="debug-toggle-btn">
          {{ debugMode ? '🔧' : '🔍' }}
        </button>
      </div>
    </div>

    <!-- Game Content -->
    <div class="game-content">
      <!-- Story Panel -->
      <div class="story-panel">
        <div class="current-location">
          📍 {{ currentNode?.location || 'Unknown Location' }}
        </div>
        
        <div class="story-text" v-html="currentNode?.situation || 'Loading...'"></div>
        
        <!-- Game Controls -->
        <div v-if="!gameState.isGameOver" class="game-controls">
          <!-- Phase: Waiting for Choices -->
          <div v-if="gameState.phase === 'waiting_for_choices'" class="waiting-phase">
            <div class="phase-description">
              <p>🎲 Ready to explore your options?</p>
            </div>
            <button 
              @click="generateChoices" 
              :disabled="!canAdvance.canAdvance"
              class="action-btn primary"
            >
              ✨ Generate Adventure Choices
            </button>
            <div v-if="!canAdvance.canAdvance" class="advancement-blocked">
              <p>⚠️ Cannot advance:</p>
              <ul>
                <li v-if="!canAdvance.reasons.validState">
                  Invalid game state
                  <button @click="fixGameState" class="fix-btn">🔧 Auto-fix</button>
                </li>
                <li v-if="!canAdvance.reasons.notGenerating">Generation in progress</li>
                <li v-if="!canAdvance.reasons.notGameOver">Game is over</li>
              </ul>
            </div>
          </div>

          <!-- Phase: Generating Choices -->
          <div v-if="gameState.phase === 'generating_choices'" class="generating-phase">
            <div class="generation-progress">
              <div class="spinner"></div>
              <p>🎲 Generating adventure choices...</p>
            </div>
          </div>

          <!-- Phase: Choosing -->
          <div v-if="gameState.phase === 'choosing'" class="choosing-phase">
            <h4>What do you do?</h4>
            <div class="choices-grid">
              <button
                v-for="choice in currentChoices"
                :key="choice.id"
                @click="makeChoice(choice.id)"
                :disabled="!canMakeChoice(choice)"
                :class="['choice-btn', {
                  'disabled': !canMakeChoice(choice),
                  'premium': choice.requiresItem,
                  'risky': choice.risk === 'high'
                }]"
              >
                <div class="choice-icon">{{ choice.icon }}</div>
                <div class="choice-text">{{ choice.text }}</div>
                <div class="choice-stats">
                  <span class="exp">+{{ choice.experience }} XP</span>
                  <span v-if="choice.cost > 0" class="cost">{{ choice.cost }}💰</span>
                </div>
                <div v-if="choice.requiresItem" class="choice-requirement">
                  🔑 {{ choice.requiresItem }}
                </div>
              </button>
            </div>
          </div>

          <!-- Phase: Advancing -->
          <div v-if="gameState.phase === 'advancing'" class="advancing-phase">
            <div class="advancement-progress">
              <div class="spinner"></div>
              <p>🚀 Advancing to new location...</p>
            </div>
          </div>

          <!-- Phase: Generating Node -->
          <div v-if="gameState.phase === 'generating_node'" class="generating-node-phase">
            <div class="generation-progress">
              <div class="spinner"></div>
              <p>🏗️ Creating new scenario...</p>
            </div>
          </div>
        </div>

        <!-- Game Over -->
        <div v-if="gameState.isGameOver" class="game-over">
          <h3>{{ gameState.isWin ? '🎉 Victory!' : '💀 Game Over' }}</h3>
          <p>{{ gameState.endMessage }}</p>
          <button @click="resetGame" class="action-btn primary">
            🔄 Start New Adventure
          </button>
        </div>

        <!-- Inventory -->
        <div class="inventory-section">
          <h4>🎒 Inventory</h4>
          <div v-if="!gameState.inventory || gameState.inventory.length === 0" class="empty-inventory">
            Your inventory is empty
          </div>
          <div v-else class="inventory-grid">
            <div 
              v-for="item in gameState.inventory" 
              :key="item.id"
              class="inventory-item"
              :title="item.description"
            >
              <div class="item-icon">{{ item.icon }}</div>
              <div class="item-name">{{ item.name }}</div>
              <div v-if="item.quantity > 1" class="item-quantity">{{ item.quantity }}</div>
            </div>
          </div>
        </div>
      </div>

      <!-- Graph Visualization -->
      <div class="graph-panel">
        <div class="graph-header">
          <h4>🗺️ Adventure Map</h4>
          <div class="graph-stats">
            <span>{{ gameState.storyGraph?.nodes?.length || 0 }} locations</span>
            <span>{{ gameState.storyGraph?.edges?.length || 0 }} paths</span>
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
          v-for="(entry, index) in recentLogs" 
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
import { GameEngine } from '../engine/GameEngine.js'
import cytoscape from 'cytoscape'

export default {
  name: 'AdventureGameV2',
  props: {
    aiService: {
      type: Object,
      required: true
    }
  },
  data() {
    return {
      engine: null,
      gameState: {
        // Initialize with default values to prevent undefined access
        currentNodeId: 'start',
        phase: 'waiting_for_choices',
        level: 1,
        experience: 0,
        gold: 50,
        inventory: [],
        visitedNodes: new Set(['start']),
        storyGraph: {
          nodes: [],
          edges: []
        },
        isGameOver: false,
        isWin: false,
        endMessage: '',
        actionLog: [],
        generationInProgress: false,
        currentGeneration: null,
        lastError: null,
        errorCount: 0
      },
      validation: { valid: true, rules: {} },
      canAdvance: { canAdvance: false, reasons: {} },
      currentChoices: [],
      currentNode: null,
      error: null,
      loading: false,
      loadingMessage: '',
      debugMode: false,
      cy: null
    }
  },
  computed: {
    recentLogs() {
      return this.gameState.actionLog?.slice(-10) || []
    }
  },
  async mounted() {
    await this.initializeEngine()
    this.initializeGraph()
  },
  beforeUnmount() {
    if (this.cy) {
      this.cy.destroy()
    }
  },
  methods: {
    async initializeEngine() {
      try {
        this.engine = new GameEngine(this.aiService, { 
          debugMode: this.debugMode,
          logActions: true,
          autoFix: true  // Enable auto-fix for better user experience
        })
        
        // Subscribe to engine events
        this.engine.addListener(this.handleEngineEvent)
        
        // Update UI state
        this.updateUIState()
        
        console.log('Game engine initialized successfully')
      } catch (error) {
        console.error('Failed to initialize game engine:', error)
        this.showError(`Failed to initialize game: ${error.message}`)
      }
    },

    handleEngineEvent(event) {
      console.log('Engine event:', event.type, event)
      
      switch (event.type) {
        case 'generationStarted':
          this.loading = true
          this.loadingMessage = event.type === 'choices' ? 'Generating choices...' : 'Creating new location...'
          break
          
        case 'generationCompleted':
          this.loading = false
          this.loadingMessage = ''
          break
          
        case 'generationFailed':
          this.loading = false
          this.loadingMessage = ''
          this.showError(`Generation failed: ${event.error}`)
          break
          
        case 'choiceStarted':
          this.loading = true
          this.loadingMessage = 'Processing choice...'
          break
          
        case 'choiceCompleted':
          this.loading = false
          this.loadingMessage = ''
          this.updateGraph()
          break
          
        case 'choiceFailed':
          this.loading = false
          this.loadingMessage = ''
          this.showError(`Choice failed: ${event.error}`)
          break
          
        case 'levelUp':
          // Show celebration or notification
          console.log('🎉 Level up!', event.newLevel)
          break
          
        case 'gameWon':
          // Show victory celebration
          console.log('🏆 Game won!', event.endMessage)
          break
          
        case 'gameReset':
          this.updateGraph()
          break
      }
      
      // Always update UI state after events
      this.updateUIState()
    },

    updateUIState() {
      if (!this.engine) return
      
      try {
        this.gameState = this.engine.getState()
        this.validation = this.engine.validateState()
        this.canAdvance = this.engine.canAdvance()
        this.currentChoices = this.engine.getCurrentChoices()
        this.currentNode = this.engine.getCurrentNode()
        
        // Clear error if state becomes valid
        if (this.validation.valid && this.error) {
          this.error = null
        }
      } catch (error) {
        console.error('Error updating UI state:', error)
        this.showError(`UI update failed: ${error.message}`)
      }
    },

    async generateChoices() {
      try {
        this.clearError()
        await this.engine.generateChoices()
        this.updateUIState()
      } catch (error) {
        console.error('Choice generation failed:', error)
        this.showError(error.message)
      }
    },

    async makeChoice(choiceId) {
      try {
        this.clearError()
        const result = await this.engine.makeChoice(choiceId)
        console.log('Choice result:', result)
        this.updateUIState()
      } catch (error) {
        console.error('Choice failed:', error)
        this.showError(error.message)
      }
    },

    canMakeChoice(choice) {
      return this.engine?.canMakeChoice(choice) || false
    },

    resetGame() {
      if (this.engine) {
        this.engine.reset()
        this.updateUIState()
        this.clearError()
      }
    },

    showError(message) {
      this.error = message
      console.error('Game error:', message)
    },

    clearError() {
      this.error = null
    },

    fixGameState() {
      if (this.engine) {
        try {
          this.clearError()
          const fixCount = this.engine.autoFixState()
          this.updateUIState()
          
          if (fixCount > 0) {
            this.log(`Fixed ${fixCount} state issues`)
            // Show success message briefly
            const originalError = this.error
            this.error = `✅ Fixed ${fixCount} state issue${fixCount > 1 ? 's' : ''}`
            setTimeout(() => {
              if (this.error === `✅ Fixed ${fixCount} state issue${fixCount > 1 ? 's' : ''}`) {
                this.error = originalError
              }
            }, 3000)
          } else {
            this.showError('No issues found to fix')
          }
        } catch (error) {
          console.error('State fix failed:', error)
          this.showError(`Fix failed: ${error.message}`)
        }
      }
    },

    toggleDebugMode() {
      this.debugMode = !this.debugMode
      if (this.engine) {
        this.engine.options.debugMode = this.debugMode
      }
    },

    exportDebugData() {
      if (this.engine) {
        const debugData = this.engine.getDebugInfo()
        console.log('Debug data:', debugData)
        
        // Create downloadable file
        const dataStr = JSON.stringify(debugData, null, 2)
        const dataBlob = new Blob([dataStr], { type: 'application/json' })
        const url = URL.createObjectURL(dataBlob)
        
        const link = document.createElement('a')
        link.href = url
        link.download = `adventure-debug-${Date.now()}.json`
        link.click()
        
        URL.revokeObjectURL(url)
      }
    },

    formatTime(timestamp) {
      const date = timestamp instanceof Date ? timestamp : new Date(timestamp)
      return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
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
                if (this.gameState.visitedNodes?.has(ele.id())) return '#3498db'
                return '#95a5a6'
              },
              'label': 'data(label)',
              'text-valign': 'center',
              'text-halign': 'center',
              'color': '#fff',
              'font-size': '12px',
              'font-weight': 'bold',
              'width': '60px',
              'height': '60px'
            }
          },
          {
            selector: 'edge',
            style: {
              'line-color': '#7f8c8d',
              'target-arrow-color': '#7f8c8d',
              'target-arrow-shape': 'triangle',
              'curve-style': 'bezier',
              'label': 'data(label)',
              'font-size': '10px'
            }
          }
        ],
        elements: this.buildGraphElements(),
        layout: {
          name: 'breadthfirst',
          directed: true,
          padding: 10
        }
      })
    },

    buildGraphElements() {
      if (!this.gameState.storyGraph || !this.gameState.storyGraph.nodes) return []
      
      const nodes = (this.gameState.storyGraph.nodes || []).map(node => ({
        data: {
          id: node.id,
          label: (node.location || 'Unknown').split(' ').slice(-2).join(' ')
        }
      }))

      const edges = (this.gameState.storyGraph.edges || []).map(edge => ({
        data: {
          id: edge.id,
          source: edge.fromId,
          target: edge.toId,
          label: edge.icon || '→'
        }
      }))

      return [...nodes, ...edges]
    },

    updateGraph() {
      if (!this.cy) return
      
      // Update elements
      const elements = this.buildGraphElements()
      this.cy.elements().remove()
      this.cy.add(elements)
      
      // Update layout
      this.cy.layout({ 
        name: 'breadthfirst', 
        directed: true, 
        padding: 10 
      }).run()
      
      // Update node colors
      this.cy.nodes().forEach(node => {
        const nodeId = node.id()
        let color = '#95a5a6' // Default: unvisited
        
        if (nodeId === this.gameState.currentNodeId) {
          color = '#e74c3c' // Current position: red
        } else if (this.gameState.visitedNodes?.has(nodeId)) {
          color = '#3498db' // Visited: blue
        }
        
        node.style('background-color', color)
      })
    },

    // Public API methods for external components to call
    setLoading(loading, message = '') {
      this.loading = loading
      this.loadingMessage = message
    }
  }
}
</script>

<style scoped>
.adventure-game-v2 {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 12px;
  overflow: hidden;
  color: #fff;
}

/* Debug Panel */
.debug-panel {
  background: rgba(0, 0, 0, 0.8);
  border-bottom: 2px solid #f39c12;
  padding: 10px 15px;
  font-family: monospace;
  font-size: 0.85em;
}

.debug-info {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 10px;
  margin-bottom: 10px;
}

.debug-actions {
  display: flex;
  gap: 10px;
}

.debug-btn {
  background: #f39c12;
  color: white;
  border: none;
  padding: 5px 10px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.8em;
}

.validation-errors .error {
  color: #e74c3c;
  font-size: 0.8em;
}

/* Error Display */
.error-display {
  background: linear-gradient(135deg, #e74c3c, #c0392b);
  color: white;
  padding: 12px 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  animation: slideDown 0.3s ease-out;
}

.error-close {
  background: none;
  border: none;
  color: white;
  font-size: 1.2em;
  cursor: pointer;
}

/* Loading Display */
.loading-display {
  background: linear-gradient(135deg, #3498db, #2980b9);
  color: white;
  padding: 12px 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  animation: slideDown 0.3s ease-out;
}

/* Game Header */
.game-header {
  background: rgba(0, 0, 0, 0.3);
  padding: 15px 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 2px solid rgba(255, 255, 255, 0.2);
}

.game-header h3 {
  margin: 0;
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
  opacity: 0.8;
}

.stat-value {
  font-weight: bold;
  font-size: 1.1em;
  color: #f39c12;
}

.debug-toggle-btn {
  background: rgba(255, 255, 255, 0.2);
  border: none;
  color: white;
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 1.2em;
}

/* Game Content */
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
  gap: 15px;
  background: rgba(0, 0, 0, 0.2);
  border-right: 1px solid rgba(255, 255, 255, 0.2);
  overflow-y: auto;
}

.current-location {
  font-size: 1.1em;
  font-weight: bold;
  color: #f39c12;
}

.story-text {
  background: rgba(255, 255, 255, 0.1);
  padding: 15px;
  border-radius: 8px;
  line-height: 1.6;
  border-left: 4px solid #f39c12;
}

/* Game Controls */
.game-controls {
  flex: 1;
}

.phase-description {
  text-align: center;
  margin-bottom: 15px;
  font-style: italic;
  opacity: 0.9;
}

.action-btn {
  background: linear-gradient(45deg, #667eea, #764ba2);
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 8px;
  cursor: pointer;
  font-size: 1em;
  font-weight: bold;
  transition: all 0.3s ease;
  width: 100%;
  box-shadow: 0 4px 8px rgba(0,0,0,0.2);
}

.action-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 12px rgba(0,0,0,0.3);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.advancement-blocked {
  margin-top: 15px;
  padding: 10px;
  background: rgba(231, 76, 60, 0.2);
  border-radius: 6px;
  border-left: 4px solid #e74c3c;
}

.advancement-blocked ul {
  margin: 5px 0 0 0;
  padding-left: 20px;
}

.fix-btn {
  background: #f39c12;
  color: white;
  border: none;
  padding: 4px 8px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.8em;
  margin-left: 8px;
  transition: background 0.2s ease;
}

.fix-btn:hover {
  background: #e67e22;
}

.fix-btn.mini {
  padding: 2px 6px;
  font-size: 0.7em;
  margin-left: 10px;
}

.validation-summary {
  display: flex;
  align-items: center;
  margin-bottom: 5px;
  color: #f39c12;
}

/* Choices */
.choices-grid {
  display: grid;
  gap: 10px;
}

.choice-btn {
  background: rgba(255, 255, 255, 0.1);
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-radius: 8px;
  padding: 12px;
  color: white;
  cursor: pointer;
  transition: all 0.3s ease;
  text-align: left;
  display: grid;
  grid-template-columns: auto 1fr auto;
  gap: 10px;
  align-items: center;
}

.choice-btn:hover:not(.disabled) {
  background: rgba(255, 255, 255, 0.2);
  border-color: rgba(255, 255, 255, 0.5);
  transform: translateX(5px);
}

.choice-btn.disabled {
  opacity: 0.5;
  cursor: not-allowed;
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
  font-size: 1.5em;
}

.choice-text {
  font-weight: 500;
}

.choice-stats {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 2px;
  font-size: 0.8em;
}

.choice-stats .exp {
  color: #2ecc71;
}

.choice-stats .cost {
  color: #f39c12;
}

.choice-requirement {
  grid-column: 1 / -1;
  font-size: 0.8em;
  color: #f39c12;
  text-align: center;
  margin-top: 5px;
}

/* Progress indicators */
.generation-progress,
.advancement-progress {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 15px;
  padding: 30px;
  text-align: center;
}

/* Inventory */
.inventory-section {
  background: rgba(0, 0, 0, 0.2);
  padding: 15px;
  border-radius: 8px;
}

.inventory-section h4 {
  margin: 0 0 10px 0;
  color: #f39c12;
}

.empty-inventory {
  text-align: center;
  opacity: 0.7;
  font-style: italic;
  padding: 20px;
}

.inventory-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(80px, 1fr));
  gap: 8px;
}

.inventory-item {
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 6px;
  padding: 8px;
  text-align: center;
  position: relative;
}

.item-icon {
  font-size: 1.5em;
  margin-bottom: 4px;
}

.item-name {
  font-size: 0.7em;
  opacity: 0.8;
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

/* Graph Panel */
.graph-panel {
  display: flex;
  flex-direction: column;
  background: rgba(0, 0, 0, 0.1);
}

.graph-header {
  padding: 15px 20px;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid rgba(255, 255, 255, 0.2);
}

.graph-header h4 {
  margin: 0;
  color: #f39c12;
}

.graph-stats {
  display: flex;
  gap: 15px;
  font-size: 0.8em;
  opacity: 0.8;
}

.graph-container {
  flex: 1;
  min-height: 300px;
  background: rgba(255, 255, 255, 0.05);
}

/* Action Log */
.action-log {
  background: rgba(0, 0, 0, 0.4);
  padding: 15px 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.2);
  max-height: 120px;
}

.action-log h4 {
  margin: 0 0 10px 0;
  color: #f39c12;
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
  color: rgba(255, 255, 255, 0.6);
  min-width: 60px;
}

.log-text {
  color: rgba(255, 255, 255, 0.9);
}

/* Game Over */
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

/* Animations */
.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid rgba(255, 255, 255, 0.3);
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