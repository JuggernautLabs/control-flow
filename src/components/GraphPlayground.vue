<template>
  <div class="graph-playground">
    <div class="playground-header">
      <h3>🕸️ Graph Layout Playground</h3>
      <div class="controls">
        <select v-model="selectedDataset" @change="loadDataset" class="dataset-select">
          <option value="simple">Simple Tree</option>
          <option value="complex">Complex Network</option>
          <option value="hierarchy">Deep Hierarchy</option>
          <option value="circular">Circular Dependencies</option>
        </select>
        <button @click="generateRandomGraph" class="generate-btn">🎲 Random</button>
      </div>
    </div>

    <div class="playground-content">
      <!-- Layout Controls -->
      <div class="layout-panel">
        <h4>🎛️ Layout Controls</h4>
        
        <div class="layout-grid">
          <button
            v-for="layout in layouts"
            :key="layout.name"
            @click="applyLayout(layout.name)"
            :class="['layout-btn', { active: currentLayout === layout.name }]"
          >
            <div class="layout-icon">{{ layout.icon }}</div>
            <div class="layout-name">{{ layout.name }}</div>
          </button>
        </div>

        <div class="layout-options" v-if="layoutOptions[currentLayout]">
          <h5>Options</h5>
          <div v-for="option in layoutOptions[currentLayout]" :key="option.key" class="option-row">
            <label>{{ option.label }}:</label>
            <input
              v-if="option.type === 'range'"
              type="range"
              :min="option.min"
              :max="option.max"
              :step="option.step"
              v-model="option.value"
              @input="updateLayoutOption(option.key, option.value)"
            >
            <input
              v-else-if="option.type === 'number'"
              type="number"
              :min="option.min"
              :max="option.max"
              v-model="option.value"
              @input="updateLayoutOption(option.key, option.value)"
            >
            <select
              v-else-if="option.type === 'select'"
              v-model="option.value"
              @change="updateLayoutOption(option.key, option.value)"
            >
              <option v-for="choice in option.choices" :key="choice" :value="choice">
                {{ choice }}
              </option>
            </select>
            <span class="option-value">{{ option.value }}</span>
          </div>
          <button @click="applyLayoutWithOptions" class="apply-btn">Apply Changes</button>
        </div>

        <div class="graph-stats">
          <h5>📊 Graph Stats</h5>
          <div class="stat-row">
            <span>Nodes:</span>
            <span>{{ graphStats.nodes }}</span>
          </div>
          <div class="stat-row">
            <span>Edges:</span>
            <span>{{ graphStats.edges }}</span>
          </div>
          <div class="stat-row">
            <span>Density:</span>
            <span>{{ graphStats.density }}%</span>
          </div>
        </div>
      </div>

      <!-- Graph Visualization -->
      <div class="graph-area">
        <div class="graph-controls">
          <button @click="fitGraph" class="control-btn">🔍 Fit</button>
          <button @click="resetZoom" class="control-btn">🏠 Reset</button>
          <button @click="exportGraph" class="control-btn">💾 Export</button>
          <span class="zoom-info">Zoom: {{ Math.round(zoomLevel * 100) }}%</span>
        </div>
        <div class="graph-container" ref="graphContainer"></div>
      </div>
    </div>

    <!-- Performance Metrics -->
    <div class="metrics-panel">
      <h4>⚡ Performance Metrics</h4>
      <div class="metrics-grid">
        <div class="metric">
          <span class="metric-label">Layout Time:</span>
          <span class="metric-value">{{ performanceMetrics.layoutTime }}ms</span>
        </div>
        <div class="metric">
          <span class="metric-label">Render Time:</span>
          <span class="metric-value">{{ performanceMetrics.renderTime }}ms</span>
        </div>
        <div class="metric">
          <span class="metric-label">FPS:</span>
          <span class="metric-value">{{ performanceMetrics.fps }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import cytoscape from 'cytoscape'
import dagre from 'cytoscape-dagre'
import cola from 'cytoscape-cola'
import elk from 'cytoscape-elk'

// Register layout extensions
cytoscape.use(dagre)
cytoscape.use(cola)
cytoscape.use(elk)

export default {
  name: 'GraphPlayground',
  data() {
    return {
      cy: null,
      currentLayout: 'dagre',
      selectedDataset: 'simple',
      zoomLevel: 1,
      performanceMetrics: {
        layoutTime: 0,
        renderTime: 0,
        fps: 60
      },
      graphStats: {
        nodes: 0,
        edges: 0,
        density: 0
      },
      layouts: [
        { name: 'dagre', icon: '📊' },
        { name: 'cola', icon: '🌐' },
        { name: 'elk', icon: '🦌' },
        { name: 'breadthfirst', icon: '🌳' },
        { name: 'circle', icon: '⭕' },
        { name: 'grid', icon: '⬜' },
        { name: 'random', icon: '🎲' },
        { name: 'concentric', icon: '🎯' }
      ],
      layoutOptions: {
        dagre: [
          { key: 'rankDir', label: 'Direction', type: 'select', choices: ['TB', 'BT', 'LR', 'RL'], value: 'TB' },
          { key: 'nodeSep', label: 'Node Separation', type: 'range', min: 10, max: 200, step: 10, value: 50 },
          { key: 'rankSep', label: 'Rank Separation', type: 'range', min: 10, max: 200, step: 10, value: 50 }
        ],
        cola: [
          { key: 'nodeSpacing', label: 'Node Spacing', type: 'range', min: 5, max: 100, step: 5, value: 20 },
          { key: 'edgeLengthVal', label: 'Edge Length', type: 'range', min: 10, max: 200, step: 10, value: 45 },
          { key: 'animate', label: 'Animate', type: 'select', choices: [true, false], value: true }
        ],
        elk: [
          { key: 'algorithm', label: 'Algorithm', type: 'select', choices: ['layered', 'stress', 'mrtree'], value: 'layered' },
          { key: 'nodePlacement', label: 'Node Placement', type: 'select', choices: ['BRANDES_KOEPF', 'LINEAR_SEGMENTS', 'INTERACTIVE', 'SIMPLE'], value: 'BRANDES_KOEPF' }
        ],
        circle: [
          { key: 'radius', label: 'Radius', type: 'range', min: 50, max: 300, step: 10, value: 150 },
          { key: 'startAngle', label: 'Start Angle', type: 'range', min: 0, max: 360, step: 15, value: 0 }
        ],
        grid: [
          { key: 'cols', label: 'Columns', type: 'number', min: 1, max: 10, value: 4 },
          { key: 'rows', label: 'Rows', type: 'number', min: 1, max: 10, value: 4 }
        ]
      },
      datasets: {
        simple: {
          nodes: [
            { id: 'root', label: 'Root' },
            { id: 'a', label: 'Node A' },
            { id: 'b', label: 'Node B' },
            { id: 'c', label: 'Node C' },
            { id: 'a1', label: 'A.1' },
            { id: 'a2', label: 'A.2' },
            { id: 'b1', label: 'B.1' }
          ],
          edges: [
            { source: 'root', target: 'a' },
            { source: 'root', target: 'b' },
            { source: 'root', target: 'c' },
            { source: 'a', target: 'a1' },
            { source: 'a', target: 'a2' },
            { source: 'b', target: 'b1' }
          ]
        },
        complex: {
          nodes: Array.from({ length: 20 }, (_, i) => ({ id: `node${i}`, label: `Node ${i}` })),
          edges: [
            { source: 'node0', target: 'node1' },
            { source: 'node0', target: 'node2' },
            { source: 'node1', target: 'node3' },
            { source: 'node1', target: 'node4' },
            { source: 'node2', target: 'node5' },
            { source: 'node3', target: 'node6' },
            { source: 'node4', target: 'node7' },
            { source: 'node5', target: 'node8' },
            { source: 'node6', target: 'node9' },
            { source: 'node7', target: 'node10' },
            { source: 'node8', target: 'node11' },
            { source: 'node9', target: 'node12' },
            { source: 'node10', target: 'node13' },
            { source: 'node11', target: 'node14' },
            { source: 'node12', target: 'node15' },
            { source: 'node13', target: 'node16' },
            { source: 'node14', target: 'node17' },
            { source: 'node15', target: 'node18' },
            { source: 'node16', target: 'node19' },
            { source: 'node3', target: 'node8' },
            { source: 'node7', target: 'node12' },
            { source: 'node5', target: 'node15' }
          ]
        }
      }
    }
  },
  mounted() {
    this.initializeGraph()
    this.loadDataset()
  },
  beforeUnmount() {
    if (this.cy) {
      this.cy.destroy()
    }
  },
  methods: {
    initializeGraph() {
      const container = this.$refs.graphContainer
      if (!container) return

      this.cy = cytoscape({
        container: container,
        style: [
          {
            selector: 'node',
            style: {
              'background-color': '#3498db',
              'label': 'data(label)',
              'text-valign': 'center',
              'text-halign': 'center',
              'color': '#fff',
              'font-size': '12px',
              'font-weight': 'bold',
              'width': '50px',
              'height': '50px',
              'border-width': '2px',
              'border-color': '#2980b9'
            }
          },
          {
            selector: 'edge',
            style: {
              'line-color': '#7f8c8d',
              'target-arrow-color': '#7f8c8d',
              'target-arrow-shape': 'triangle',
              'curve-style': 'bezier',
              'width': '2px'
            }
          },
          {
            selector: 'node:selected',
            style: {
              'background-color': '#e74c3c',
              'border-color': '#c0392b'
            }
          }
        ],
        layout: { name: 'dagre' }
      })

      // Event listeners
      this.cy.on('zoom', () => {
        this.zoomLevel = this.cy.zoom()
      })

      this.cy.on('tap', 'node', (event) => {
        const node = event.target
        console.log('Selected node:', node.id())
      })
    },

    loadDataset() {
      if (!this.cy) return

      const dataset = this.datasets[this.selectedDataset]
      if (!dataset) return

      const elements = [
        ...dataset.nodes.map(node => ({ data: node })),
        ...dataset.edges.map(edge => ({ data: edge }))
      ]

      this.cy.elements().remove()
      this.cy.add(elements)
      
      this.updateGraphStats()
      this.applyLayout(this.currentLayout)
    },

    generateRandomGraph() {
      const nodeCount = Math.floor(Math.random() * 15) + 10
      const nodes = Array.from({ length: nodeCount }, (_, i) => ({
        id: `rand${i}`,
        label: `R${i}`
      }))

      const edges = []
      for (let i = 0; i < nodeCount; i++) {
        const connectionCount = Math.floor(Math.random() * 3) + 1
        for (let j = 0; j < connectionCount; j++) {
          const target = Math.floor(Math.random() * nodeCount)
          if (target !== i) {
            edges.push({
              source: `rand${i}`,
              target: `rand${target}`
            })
          }
        }
      }

      this.datasets.random = { nodes, edges }
      this.selectedDataset = 'random'
      this.loadDataset()
    },

    applyLayout(layoutName) {
      if (!this.cy) return

      const startTime = performance.now()
      
      this.currentLayout = layoutName
      
      const layoutOptions = this.getLayoutOptions(layoutName)
      const layout = this.cy.layout(layoutOptions)
      
      layout.on('layoutready', () => {
        const endTime = performance.now()
        this.performanceMetrics.layoutTime = Math.round(endTime - startTime)
      })

      layout.run()
    },

    getLayoutOptions(layoutName) {
      const baseOptions = { name: layoutName }
      
      if (this.layoutOptions[layoutName]) {
        this.layoutOptions[layoutName].forEach(option => {
          baseOptions[option.key] = option.value
        })
      }

      return baseOptions
    },

    applyLayoutWithOptions() {
      this.applyLayout(this.currentLayout)
    },

    updateLayoutOption(key, value) {
      // Options are already bound to the data, so just trigger a re-layout
      // This method can be used for additional validation if needed
    },

    fitGraph() {
      if (this.cy) {
        this.cy.fit()
      }
    },

    resetZoom() {
      if (this.cy) {
        this.cy.zoom(1)
        this.cy.center()
      }
    },

    exportGraph() {
      if (!this.cy) return

      const exportData = {
        nodes: this.cy.nodes().map(node => node.data()),
        edges: this.cy.edges().map(edge => edge.data()),
        layout: this.currentLayout,
        layoutOptions: this.layoutOptions[this.currentLayout] || []
      }

      const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = 'graph-export.json'
      a.click()
      URL.revokeObjectURL(url)
    },

    updateGraphStats() {
      if (!this.cy) return

      const nodes = this.cy.nodes().length
      const edges = this.cy.edges().length
      const maxEdges = nodes * (nodes - 1) / 2
      const density = maxEdges > 0 ? Math.round((edges / maxEdges) * 100) : 0

      this.graphStats = {
        nodes,
        edges,
        density
      }
    }
  }
}
</script>

<style scoped>
.graph-playground {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #f8f9fa;
}

.playground-header {
  background: white;
  padding: 15px 20px;
  border-bottom: 1px solid #dee2e6;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.playground-header h3 {
  margin: 0;
  color: #333;
}

.controls {
  display: flex;
  gap: 10px;
  align-items: center;
}

.dataset-select {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  background: white;
}

.generate-btn {
  background: #28a745;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
}

.playground-content {
  flex: 1;
  display: grid;
  grid-template-columns: 300px 1fr;
  gap: 0;
  min-height: 0;
}

.layout-panel {
  background: white;
  border-right: 1px solid #dee2e6;
  padding: 20px;
  overflow-y: auto;
}

.layout-panel h4 {
  margin: 0 0 15px 0;
  color: #495057;
}

.layout-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
  margin-bottom: 20px;
}

.layout-btn {
  background: #f8f9fa;
  border: 2px solid #dee2e6;
  border-radius: 8px;
  padding: 10px;
  cursor: pointer;
  text-align: center;
  transition: all 0.2s ease;
}

.layout-btn:hover {
  background: #e9ecef;
  border-color: #adb5bd;
}

.layout-btn.active {
  background: #007bff;
  border-color: #0056b3;
  color: white;
}

.layout-icon {
  font-size: 1.5em;
  margin-bottom: 5px;
}

.layout-name {
  font-size: 0.8em;
  font-weight: 500;
  text-transform: capitalize;
}

.layout-options {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 15px;
  margin-bottom: 20px;
}

.layout-options h5 {
  margin: 0 0 10px 0;
  color: #495057;
}

.option-row {
  display: grid;
  grid-template-columns: 1fr 2fr auto;
  gap: 10px;
  align-items: center;
  margin-bottom: 10px;
}

.option-row label {
  font-size: 0.9em;
  color: #6c757d;
}

.option-row input,
.option-row select {
  padding: 4px 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.option-value {
  font-size: 0.8em;
  color: #6c757d;
  min-width: 40px;
  text-align: right;
}

.apply-btn {
  background: #007bff;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  width: 100%;
  margin-top: 10px;
}

.graph-stats {
  background: #e9ecef;
  border-radius: 8px;
  padding: 15px;
}

.graph-stats h5 {
  margin: 0 0 10px 0;
  color: #495057;
}

.stat-row {
  display: flex;
  justify-content: space-between;
  margin-bottom: 5px;
  font-size: 0.9em;
}

.graph-area {
  display: flex;
  flex-direction: column;
  background: white;
}

.graph-controls {
  background: #f8f9fa;
  padding: 10px 15px;
  border-bottom: 1px solid #dee2e6;
  display: flex;
  gap: 10px;
  align-items: center;
}

.control-btn {
  background: #6c757d;
  color: white;
  border: none;
  padding: 6px 12px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.85em;
}

.control-btn:hover {
  background: #5a6268;
}

.zoom-info {
  margin-left: auto;
  font-size: 0.85em;
  color: #6c757d;
}

.graph-container {
  flex: 1;
  min-height: 400px;
  background: #fafafa;
}

.metrics-panel {
  background: white;
  border-top: 1px solid #dee2e6;
  padding: 15px 20px;
}

.metrics-panel h4 {
  margin: 0 0 10px 0;
  color: #495057;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
}

.metric {
  display: flex;
  justify-content: space-between;
  padding: 8px 12px;
  background: #f8f9fa;
  border-radius: 6px;
}

.metric-label {
  color: #6c757d;
  font-size: 0.9em;
}

.metric-value {
  font-weight: 600;
  color: #495057;
}
</style>