<template>
  <div class="dependency-graph-container">
    <div class="graph-header">
      <h2>🕸️ Ticket Dependency Graph</h2>
      <div class="graph-controls">
        <button @click="resetLayout" class="btn btn-sm">🔄 Reset Layout</button>
        <button @click="togglePhysics" class="btn btn-sm">
          {{ physicsEnabled ? '⏸️ Pause' : '▶️ Play' }} Physics
        </button>
        <select v-model="layoutMode" @change="updateLayout" class="layout-select">
          <option value="hierarchical">Hierarchical</option>
          <option value="force">Force Directed</option>
          <option value="circular">Circular</option>
        </select>
      </div>
    </div>

    <div class="graph-viewport" ref="viewport">
      <svg 
        :width="viewportWidth" 
        :height="viewportHeight"
        @click="handleCanvasClick"
        class="graph-svg"
      >
        <!-- Background grid and arrow markers -->
        <defs>
          <pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">
            <path d="M 20 0 L 0 0 0 20" fill="none" stroke="#f0f0f0" stroke-width="1"/>
          </pattern>
          
          <!-- Arrow markers for different edge types -->
          <marker id="arrowhead-hierarchy" markerWidth="10" markerHeight="7" 
                  refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#667eea" />
          </marker>
          
          <marker id="arrowhead-dependency" markerWidth="10" markerHeight="7" 
                  refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#f39c12" />
          </marker>
        </defs>
        <rect width="100%" height="100%" fill="url(#grid)" />

        <!-- Dependency edges -->
        <g class="edges">
          <line
            v-for="edge in edges"
            :key="edge.id"
            :x1="edge.source.x"
            :y1="edge.source.y"
            :x2="edge.target.x"
            :y2="edge.target.y"
            :class="['edge', edge.type]"
            :stroke-width="edge.weight || 2"
          />
          <!-- Edge labels -->
          <text
            v-for="edge in edges"
            :key="`label-${edge.id}`"
            :x="(edge.source.x + edge.target.x) / 2"
            :y="(edge.source.y + edge.target.y) / 2"
            class="edge-label"
            text-anchor="middle"
          >
            {{ edge.label }}
          </text>
        </g>

        <!-- Ticket nodes -->
        <g class="nodes">
          <g
            v-for="node in nodes"
            :key="node.id"
            :transform="`translate(${node.x}, ${node.y})`"
            @click="handleNodeClick(node, $event)"
            @dblclick="handleNodeDoubleClick(node, $event)"
            :class="['node', `state-${node.refinementState}`, { 'selected': selectedNode?.id === node.id }]"
          >
            <!-- Node shape based on type -->
            <circle
              v-if="node.type === 'feature'"
              :r="node.radius"
              class="node-shape feature-node"
            />
            <rect
              v-else-if="node.type === 'component'"
              :x="-node.radius"
              :y="-node.radius"
              :width="node.radius * 2"
              :height="node.radius * 2"
              class="node-shape component-node"
            />
            <polygon
              v-else
              :points="getPolygonPoints(node.radius)"
              class="node-shape ticket-node"
            />

            <!-- Node label -->
            <text
              class="node-label"
              text-anchor="middle"
              :dy="node.radius + 15"
            >
              {{ truncateText(node.title, 15) }}
            </text>

            <!-- State indicator -->
            <circle
              :cx="node.radius - 5"
              :cy="-node.radius + 5"
              r="4"
              :class="`state-indicator state-${node.refinementState}`"
            />

            <!-- Priority indicator -->
            <text
              :x="-node.radius + 5"
              :y="-node.radius + 8"
              class="priority-indicator"
              :class="`priority-${node.priority}`"
            >
              {{ getPrioritySymbol(node.priority) }}
            </text>

            <!-- Component count for features -->
            <text
              v-if="node.componentCount > 0"
              x="0"
              y="4"
              class="component-count"
              text-anchor="middle"
            >
              {{ node.componentCount }}
            </text>
          </g>
        </g>
      </svg>
    </div>

    <!-- Bottom Action Menu -->
    <div v-if="selectedNode" class="bottom-menu">
      <div class="menu-content">
        <div class="selected-info">
          <h3>{{ selectedNode.title }}</h3>
          <span class="state-badge" :class="`state-${selectedNode.refinementState}`">
            {{ selectedNode.refinementState }}
          </span>
        </div>
        
        <div class="menu-actions">
          <button @click="showNodeDetails" class="menu-btn">
            📋 Details
          </button>
          <button @click="runAnalysis" class="menu-btn">
            🧠 Analyze
          </button>
          <button @click="refineNode" class="menu-btn" v-if="canRefine">
            🔍 Refine
          </button>
          <button @click="generateInterfaces" class="menu-btn" v-if="canGenerateInterfaces">
            ⚙️ Interfaces
          </button>
          <button @click="showDependencies" class="menu-btn">
            🕸️ Dependencies
          </button>
          <button @click="deleteNode" class="menu-btn danger">
            🗑️ Delete
          </button>
        </div>
      </div>
      <button @click="clearSelection" class="close-menu">✕</button>
    </div>

    <!-- Node Details Modal -->
    <div v-if="showingDetails" class="details-modal" @click="hideDetails">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h2>{{ selectedNode.title }}</h2>
          <button @click="hideDetails" class="close-btn">✕</button>
        </div>
        
        <div class="modal-body">
          <div class="detail-section">
            <h3>Description</h3>
            <p>{{ selectedNode.description }}</p>
          </div>

          <div class="detail-section" v-if="selectedNode.interfaces?.length">
            <h3>Interfaces</h3>
            <div v-for="iface in selectedNode.interfaces" :key="iface.name" class="interface-item">
              <h4>{{ iface.name }}</h4>
              <code>{{ iface.signature }}</code>
              <p>{{ iface.purpose }}</p>
            </div>
          </div>

          <div class="detail-section" v-if="selectedNode.semanticDescription">
            <h3>Semantic Analysis</h3>
            <div class="semantic-score">
              <label>Complexity:</label>
              <div class="score-bar">
                <div 
                  class="score-fill" 
                  :style="{ width: (selectedNode.semanticDescription.complexity.confidence * 100) + '%' }"
                ></div>
                <span>{{ Math.round(selectedNode.semanticDescription.complexity.confidence * 100) }}%</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ticketStore } from '../stores/ticketStore.js'

export default {
  name: 'DependencyGraph',
  setup() {
    return { ticketStore }
  },
  data() {
    return {
      viewportWidth: 1200,
      viewportHeight: 800,
      nodes: [],
      edges: [],
      selectedNode: null,
      showingDetails: false,
      physicsEnabled: true,
      layoutMode: 'hierarchical',
      simulation: null
    }
  },
  computed: {
    canRefine() {
      return this.selectedNode && 
             ['feature', 'refined'].includes(this.selectedNode.refinementState)
    },
    canGenerateInterfaces() {
      return this.selectedNode && 
             this.selectedNode.refinementState === 'implementable'
    }
  },
  methods: {
    initializeGraph() {
      this.buildNodesAndEdges()
      this.updateLayout()
    },

    buildNodesAndEdges() {
      const tickets = ticketStore.tickets
      
      // Create nodes from tickets
      this.nodes = tickets.map(ticket => ({
        id: ticket.id,
        title: ticket.title,
        description: ticket.description,
        refinementState: ticket.refinementState,
        priority: ticket.priority,
        type: this.getNodeType(ticket),
        radius: this.getNodeRadius(ticket),
        interfaces: ticket.interfaces || [],
        semanticDescription: ticket.semanticDescription,
        componentCount: this.getComponentCount(ticket),
        x: Math.random() * (this.viewportWidth - 200) + 100,
        y: Math.random() * (this.viewportHeight - 200) + 100,
        parentId: ticket.parentTicketId,
        childIds: ticket.childTicketIds || []
      }))

      // Create edges from relationships
      this.edges = []
      let edgeId = 0

      tickets.forEach(ticket => {
        // Parent-child relationships
        if (ticket.childTicketIds && ticket.childTicketIds.length > 0) {
          ticket.childTicketIds.forEach(childId => {
            const parentNode = this.nodes.find(n => n.id === ticket.id)
            const childNode = this.nodes.find(n => n.id === childId)
            
            if (parentNode && childNode) {
              this.edges.push({
                id: `edge-${edgeId++}`,
                source: parentNode,
                target: childNode,
                type: 'hierarchy',
                label: 'contains',
                weight: 3
              })
            }
          })
        }

        // Interface dependencies
        if (ticket.interfaces) {
          ticket.interfaces.forEach(iface => {
            if (iface.dependencies) {
              iface.dependencies.forEach(depId => {
                const sourceNode = this.nodes.find(n => n.id === ticket.id)
                const targetNode = this.nodes.find(n => n.id === depId)
                
                if (sourceNode && targetNode) {
                  this.edges.push({
                    id: `edge-${edgeId++}`,
                    source: sourceNode,
                    target: targetNode,
                    type: 'dependency',
                    label: 'depends on',
                    weight: 2
                  })
                }
              })
            }
          })
        }
      })
    },

    getNodeType(ticket) {
      if (ticket.refinementState === 'feature') return 'feature'
      if (ticket.interfaces && ticket.interfaces.length > 0) return 'component'
      return 'ticket'
    },

    getNodeRadius(ticket) {
      const baseRadius = 30
      const complexityBonus = ticket.semanticDescription?.complexity?.confidence 
        ? ticket.semanticDescription.complexity.confidence * 10 
        : 0
      return baseRadius + complexityBonus
    },

    getComponentCount(ticket) {
      return ticket.childTicketIds ? ticket.childTicketIds.length : 0
    },

    getPolygonPoints(radius) {
      // Hexagon points for regular tickets
      const points = []
      for (let i = 0; i < 6; i++) {
        const angle = (i * Math.PI) / 3
        const x = radius * Math.cos(angle)
        const y = radius * Math.sin(angle)
        points.push(`${x},${y}`)
      }
      return points.join(' ')
    },

    getPrioritySymbol(priority) {
      const symbols = {
        critical: '🔴',
        high: '🟠',
        medium: '🟡',
        low: '🟢'
      }
      return symbols[priority] || '⚪'
    },

    truncateText(text, maxLength) {
      return text.length > maxLength ? text.substring(0, maxLength) + '...' : text
    },

    updateLayout() {
      if (this.layoutMode === 'hierarchical') {
        this.applyHierarchicalLayout()
      } else if (this.layoutMode === 'circular') {
        this.applyCircularLayout()
      } else {
        this.applyForceLayout()
      }
    },

    applyHierarchicalLayout() {
      // Simple hierarchical layout
      const levels = new Map()
      const rootNodes = this.nodes.filter(n => !n.parentId)
      
      // Assign levels
      const assignLevel = (node, level) => {
        levels.set(node.id, level)
        node.childIds.forEach(childId => {
          const child = this.nodes.find(n => n.id === childId)
          if (child) assignLevel(child, level + 1)
        })
      }
      
      rootNodes.forEach(node => assignLevel(node, 0))
      
      // Position nodes by level
      const levelGroups = new Map()
      this.nodes.forEach(node => {
        const level = levels.get(node.id) || 0
        if (!levelGroups.has(level)) levelGroups.set(level, [])
        levelGroups.get(level).push(node)
      })
      
      levelGroups.forEach((nodes, level) => {
        const y = 100 + level * 150
        nodes.forEach((node, index) => {
          node.x = 100 + (index * (this.viewportWidth - 200)) / Math.max(1, nodes.length - 1)
          node.y = y
        })
      })
    },

    applyCircularLayout() {
      const centerX = this.viewportWidth / 2
      const centerY = this.viewportHeight / 2
      const radius = Math.min(centerX, centerY) - 100
      
      this.nodes.forEach((node, index) => {
        const angle = (2 * Math.PI * index) / this.nodes.length
        node.x = centerX + radius * Math.cos(angle)
        node.y = centerY + radius * Math.sin(angle)
      })
    },

    applyForceLayout() {
      // Simple force-directed layout simulation
      for (let iteration = 0; iteration < 100; iteration++) {
        // Repulsion between nodes
        for (let i = 0; i < this.nodes.length; i++) {
          for (let j = i + 1; j < this.nodes.length; j++) {
            const node1 = this.nodes[i]
            const node2 = this.nodes[j]
            const dx = node2.x - node1.x
            const dy = node2.y - node1.y
            const distance = Math.sqrt(dx * dx + dy * dy)
            
            if (distance < 100) {
              const force = 100 / (distance * distance)
              const fx = force * dx / distance
              const fy = force * dy / distance
              
              node1.x -= fx
              node1.y -= fy
              node2.x += fx
              node2.y += fy
            }
          }
        }
        
        // Attraction along edges
        this.edges.forEach(edge => {
          const dx = edge.target.x - edge.source.x
          const dy = edge.target.y - edge.source.y
          const distance = Math.sqrt(dx * dx + dy * dy)
          const targetDistance = 150
          
          if (distance > targetDistance) {
            const force = 0.1 * (distance - targetDistance)
            const fx = force * dx / distance
            const fy = force * dy / distance
            
            edge.source.x += fx
            edge.source.y += fy
            edge.target.x -= fx
            edge.target.y -= fy
          }
        })
      }
    },

    handleNodeClick(node, event) {
      event.stopPropagation()
      this.selectedNode = node
    },

    handleNodeDoubleClick(node, event) {
      event.stopPropagation()
      this.selectedNode = node
      this.showingDetails = true
    },

    handleCanvasClick() {
      this.clearSelection()
    },

    clearSelection() {
      this.selectedNode = null
    },

    showNodeDetails() {
      this.showingDetails = true
    },

    hideDetails() {
      this.showingDetails = false
    },

    async runAnalysis() {
      if (this.selectedNode) {
        await ticketStore.runSemanticAnalysis(this.selectedNode.id)
        this.buildNodesAndEdges() // Refresh to show updated analysis
      }
    },

    async refineNode() {
      if (this.selectedNode) {
        const analysis = await ticketStore.analyzeRefinement(this.selectedNode.id)
        
        if (analysis.shouldRefine.value && analysis.suggestedBreakdown) {
          // Create child tickets from suggested breakdown
          const childTickets = ticketStore.createChildTickets(
            this.selectedNode.id, 
            analysis.suggestedBreakdown
          )
          
          if (childTickets.length > 0) {
            // Rebuild graph to show new child nodes
            this.buildNodesAndEdges()
            this.updateLayout()
            console.log(`Created ${childTickets.length} child tickets`)
          }
        } else {
          console.log('No refinement needed:', analysis.shouldRefine.reasoning)
        }
      }
    },

    async generateInterfaces() {
      if (this.selectedNode) {
        await ticketStore.generateInterfaces(this.selectedNode.id)
        this.buildNodesAndEdges() // Refresh to show interfaces
      }
    },

    showDependencies() {
      // Highlight dependencies in the graph
      console.log('Showing dependencies for:', this.selectedNode.title)
    },

    deleteNode() {
      if (this.selectedNode && confirm(`Delete "${this.selectedNode.title}"?`)) {
        const index = ticketStore.tickets.findIndex(t => t.id === this.selectedNode.id)
        if (index !== -1) {
          ticketStore.tickets.splice(index, 1)
          this.buildNodesAndEdges()
          this.clearSelection()
        }
      }
    },

    resetLayout() {
      this.buildNodesAndEdges()
      this.updateLayout()
    },

    togglePhysics() {
      this.physicsEnabled = !this.physicsEnabled
    },

    handleResize() {
      this.viewportWidth = this.$refs.viewport?.clientWidth || 1200
      this.viewportHeight = this.$refs.viewport?.clientHeight || 800
    }
  },

  mounted() {
    // Create sample hierarchy if no tickets exist
    if (ticketStore.tickets.length === 0) {
      ticketStore.createSampleHierarchy()
    }
    
    this.initializeGraph()
    window.addEventListener('resize', this.handleResize)
    this.handleResize()
  },

  beforeUnmount() {
    window.removeEventListener('resize', this.handleResize)
  },

  watch: {
    // Watch for changes in tickets store
    'ticketStore.tickets': {
      handler() {
        this.buildNodesAndEdges()
        this.updateLayout()
      },
      deep: true
    }
  }
}
</script>

<style scoped>
.dependency-graph-container {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #fafafa;
}

.graph-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 20px;
  background: white;
  border-bottom: 1px solid #ddd;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.graph-header h2 {
  margin: 0;
  color: #333;
}

.graph-controls {
  display: flex;
  gap: 10px;
  align-items: center;
}

.layout-select {
  padding: 5px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.graph-viewport {
  flex: 1;
  overflow: hidden;
  position: relative;
}

.graph-svg {
  width: 100%;
  height: 100%;
  cursor: grab;
}

.graph-svg:active {
  cursor: grabbing;
}

/* Node styles */
.node {
  cursor: pointer;
  transition: all 0.2s ease;
}

.node:hover {
  filter: brightness(1.1);
  transform: scale(1.05);
}

.node.selected {
  filter: drop-shadow(0 0 10px rgba(102, 126, 234, 0.8));
}

.node-shape {
  fill: #fff;
  stroke: #ddd;
  stroke-width: 2;
  transition: all 0.2s ease;
}

.feature-node {
  fill: #e74c3c;
  stroke: #c0392b;
}

.component-node {
  fill: #3498db;
  stroke: #2980b9;
}

.ticket-node {
  fill: #2ecc71;
  stroke: #27ae60;
}

.node-label {
  font-size: 12px;
  font-weight: bold;
  fill: #333;
  pointer-events: none;
}

.state-indicator {
  stroke: #fff;
  stroke-width: 1;
}

.state-indicator.state-feature { fill: #e74c3c; }
.state-indicator.state-refined { fill: #f39c12; }
.state-indicator.state-implementable { fill: #3498db; }
.state-indicator.state-planned { fill: #9b59b6; }
.state-indicator.state-in_progress { fill: #2ecc71; }
.state-indicator.state-completed { fill: #27ae60; }
.state-indicator.state-verified { fill: #16a085; }

.priority-indicator {
  font-size: 10px;
  pointer-events: none;
}

.component-count {
  font-size: 14px;
  font-weight: bold;
  fill: #333;
  pointer-events: none;
}

/* Edge styles */
.edge {
  stroke: #999;
  fill: none;
}

.edge.hierarchy {
  stroke: #667eea;
  stroke-width: 3;
  marker-end: url(#arrowhead-hierarchy);
}

.edge.dependency {
  stroke: #f39c12;
  stroke-width: 2;
  stroke-dasharray: 5,5;
  marker-end: url(#arrowhead-dependency);
}

.edge-label {
  font-size: 10px;
  fill: #666;
  pointer-events: none;
}

/* Bottom menu */
.bottom-menu {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  background: white;
  border-top: 1px solid #ddd;
  box-shadow: 0 -4px 8px rgba(0,0,0,0.1);
  padding: 15px 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  z-index: 1000;
}

.menu-content {
  display: flex;
  align-items: center;
  gap: 20px;
  flex: 1;
}

.selected-info h3 {
  margin: 0;
  color: #333;
  font-size: 1.1em;
}

.state-badge {
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 0.8em;
  font-weight: bold;
  text-transform: uppercase;
}

.state-badge.state-feature { background: #fee; color: #c0392b; }
.state-badge.state-refined { background: #ffc; color: #f39c12; }
.state-badge.state-implementable { background: #eff; color: #2980b9; }

.menu-actions {
  display: flex;
  gap: 10px;
}

.menu-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  background: #667eea;
  color: white;
  cursor: pointer;
  font-size: 0.9em;
  transition: background 0.2s ease;
}

.menu-btn:hover {
  background: #5a6fd8;
}

.menu-btn.danger {
  background: #e74c3c;
}

.menu-btn.danger:hover {
  background: #c0392b;
}

.close-menu {
  background: #6c757d;
  color: white;
  border: none;
  border-radius: 50%;
  width: 32px;
  height: 32px;
  cursor: pointer;
  font-size: 16px;
}

/* Details Modal */
.details-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0,0,0,0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 2000;
}

.modal-content {
  background: white;
  border-radius: 8px;
  max-width: 600px;
  max-height: 80vh;
  overflow-y: auto;
  box-shadow: 0 10px 30px rgba(0,0,0,0.3);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid #ddd;
}

.modal-header h2 {
  margin: 0;
  color: #333;
}

.close-btn {
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  color: #666;
}

.modal-body {
  padding: 20px;
}

.detail-section {
  margin-bottom: 20px;
}

.detail-section h3 {
  margin: 0 0 10px 0;
  color: #667eea;
}

.interface-item {
  background: #f8f9fa;
  padding: 10px;
  border-radius: 4px;
  margin-bottom: 10px;
}

.interface-item h4 {
  margin: 0 0 5px 0;
  color: #333;
}

.interface-item code {
  display: block;
  background: #e9ecef;
  padding: 5px;
  border-radius: 3px;
  margin: 5px 0;
}

.semantic-score {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.score-bar {
  flex: 1;
  height: 20px;
  background: #e9ecef;
  border-radius: 10px;
  position: relative;
  overflow: hidden;
}

.score-fill {
  height: 100%;
  background: linear-gradient(90deg, #e74c3c 0%, #f39c12 50%, #27ae60 100%);
  transition: width 0.3s ease;
}

.score-bar span {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 0.8em;
  font-weight: bold;
  color: #333;
}

.btn {
  padding: 6px 12px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9em;
  background: #667eea;
  color: white;
  transition: background 0.2s ease;
}

.btn:hover {
  background: #5a6fd8;
}

.btn-sm {
  padding: 4px 8px;
  font-size: 0.8em;
}
</style>