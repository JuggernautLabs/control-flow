<template>
  <div class="layout-comparison">
    <div class="comparison-header">
      <h3>📊 Layout Algorithm Comparison</h3>
      <div class="comparison-controls">
        <select v-model="selectedDataset" @change="updateAllLayouts" class="dataset-select">
          <option value="small">Small Graph (10 nodes)</option>
          <option value="medium">Medium Graph (25 nodes)</option>
          <option value="large">Large Graph (50 nodes)</option>
        </select>
        <button @click="generateRandomData" class="random-btn">🎲 Random Data</button>
      </div>
    </div>

    <div class="comparison-grid">
      <div 
        v-for="layout in comparisonLayouts" 
        :key="layout.name"
        class="layout-comparison-item"
      >
        <div class="layout-header">
          <h4>{{ layout.icon }} {{ layout.name }}</h4>
          <div class="layout-metrics">
            <span class="metric">{{ layout.metrics.time }}ms</span>
            <span class="metric">{{ layout.metrics.quality }}/10</span>
          </div>
        </div>
        <div class="layout-graph" :ref="`graph_${layout.name}`"></div>
        <div class="layout-description">
          <p>{{ layout.description }}</p>
          <div class="pros-cons">
            <div class="pros">
              <strong>Pros:</strong> {{ layout.pros }}
            </div>
            <div class="cons">
              <strong>Cons:</strong> {{ layout.cons }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="performance-summary">
      <h4>⚡ Performance Summary</h4>
      <div class="summary-table">
        <div class="summary-header">
          <span>Algorithm</span>
          <span>Time (ms)</span>
          <span>Quality</span>
          <span>Best For</span>
        </div>
        <div v-for="layout in comparisonLayouts" :key="layout.name" class="summary-row">
          <span>{{ layout.name }}</span>
          <span>{{ layout.metrics.time }}</span>
          <span>{{ layout.metrics.quality }}/10</span>
          <span>{{ layout.bestFor }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import cytoscape from 'cytoscape'
import dagre from 'cytoscape-dagre'
import cola from 'cytoscape-cola'

cytoscape.use(dagre)
cytoscape.use(cola)

export default {
  name: 'LayoutComparison',
  data() {
    return {
      selectedDataset: 'medium',
      graphs: {},
      comparisonLayouts: [
        {
          name: 'Dagre',
          icon: '📊',
          description: 'Hierarchical layout algorithm optimized for directed acyclic graphs.',
          pros: 'Clear hierarchy, good for workflows',
          cons: 'Rigid structure, not ideal for cycles',
          bestFor: 'Directed workflows, org charts',
          metrics: { time: 0, quality: 8 }
        },
        {
          name: 'Cola',
          icon: '🌐',
          description: 'Force-directed layout with constraints and interactive positioning.',
          pros: 'Natural clustering, handles cycles well',
          cons: 'Can be slow, may need tuning',
          bestFor: 'Social networks, general graphs',
          metrics: { time: 0, quality: 7 }
        },
        {
          name: 'Circle',
          icon: '⭕',
          description: 'Arranges nodes in a circular pattern for equal emphasis.',
          pros: 'Fast, good for small graphs',
          cons: 'Poor for large graphs, loses structure',
          bestFor: 'Small networks, equal relationships',
          metrics: { time: 0, quality: 5 }
        },
        {
          name: 'Grid',
          icon: '⬜',
          description: 'Organizes nodes in a regular grid pattern.',
          pros: 'Predictable, space-efficient',
          cons: 'Ignores graph structure completely',
          bestFor: 'Documentation, regular structures',
          metrics: { time: 0, quality: 4 }
        }
      ],
      testData: {
        small: this.generateGraphData(10, 12),
        medium: this.generateGraphData(25, 35),
        large: this.generateGraphData(50, 75)
      }
    }
  },
  mounted() {
    this.initializeGraphs()
    this.updateAllLayouts()
  },
  beforeUnmount() {
    Object.values(this.graphs).forEach(cy => {
      if (cy) cy.destroy()
    })
  },
  methods: {
    generateGraphData(nodeCount, edgeCount) {
      const nodes = Array.from({ length: nodeCount }, (_, i) => ({
        id: `n${i}`,
        label: `N${i}`
      }))

      const edges = []
      for (let i = 0; i < edgeCount; i++) {
        const source = Math.floor(Math.random() * nodeCount)
        const target = Math.floor(Math.random() * nodeCount)
        if (source !== target) {
          edges.push({
            source: `n${source}`,
            target: `n${target}`
          })
        }
      }

      return { nodes, edges }
    },

    initializeGraphs() {
      this.comparisonLayouts.forEach(layout => {
        this.$nextTick(() => {
          const container = this.$refs[`graph_${layout.name}`]?.[0]
          if (container) {
            this.graphs[layout.name] = cytoscape({
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
                    'font-size': '8px',
                    'width': '20px',
                    'height': '20px'
                  }
                },
                {
                  selector: 'edge',
                  style: {
                    'line-color': '#7f8c8d',
                    'target-arrow-color': '#7f8c8d',
                    'target-arrow-shape': 'triangle',
                    'width': '1px',
                    'curve-style': 'bezier'
                  }
                }
              ]
            })
          }
        })
      })
    },

    updateAllLayouts() {
      const data = this.testData[this.selectedDataset]
      
      this.comparisonLayouts.forEach(layout => {
        this.updateLayoutGraph(layout, data)
      })
    },

    updateLayoutGraph(layout, data) {
      const cy = this.graphs[layout.name]
      if (!cy) return

      const elements = [
        ...data.nodes.map(node => ({ data: node })),
        ...data.edges.map(edge => ({ data: edge }))
      ]

      cy.elements().remove()
      cy.add(elements)

      const startTime = performance.now()
      
      const layoutName = layout.name.toLowerCase()
      const layoutOptions = this.getLayoutOptions(layoutName)
      
      const cyLayout = cy.layout(layoutOptions)
      
      cyLayout.on('layoutready', () => {
        const endTime = performance.now()
        layout.metrics.time = Math.round(endTime - startTime)
        
        // Calculate quality score based on edge crossings and node spacing
        layout.metrics.quality = this.calculateLayoutQuality(cy)
      })

      cyLayout.run()
    },

    getLayoutOptions(layoutName) {
      const options = { name: layoutName }
      
      switch (layoutName) {
        case 'dagre':
          options.rankDir = 'TB'
          options.nodeSep = 30
          options.rankSep = 30
          break
        case 'cola':
          options.nodeSpacing = 10
          options.edgeLengthVal = 25
          options.animate = false
          break
        case 'circle':
          options.radius = 80
          break
        case 'grid':
          options.cols = Math.ceil(Math.sqrt(this.testData[this.selectedDataset].nodes.length))
          break
      }
      
      return options
    },

    calculateLayoutQuality(cy) {
      // Simple quality metric based on edge length variance and node distribution
      const edges = cy.edges()
      if (edges.length === 0) return 10

      const edgeLengths = edges.map(edge => {
        const source = edge.source().position()
        const target = edge.target().position()
        return Math.sqrt(Math.pow(target.x - source.x, 2) + Math.pow(target.y - source.y, 2))
      })

      const avgLength = edgeLengths.reduce((a, b) => a + b, 0) / edgeLengths.length
      const variance = edgeLengths.reduce((acc, length) => acc + Math.pow(length - avgLength, 2), 0) / edgeLengths.length
      
      // Lower variance = higher quality (more consistent edge lengths)
      const quality = Math.max(1, 10 - Math.sqrt(variance) / 10)
      return Math.round(quality)
    },

    generateRandomData() {
      const nodeCount = Math.floor(Math.random() * 20) + 15
      const edgeCount = Math.floor(Math.random() * 30) + 20
      
      this.testData.random = this.generateGraphData(nodeCount, edgeCount)
      this.selectedDataset = 'random'
      this.updateAllLayouts()
    }
  }
}
</script>

<style scoped>
.layout-comparison {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 20px;
  background: #f8f9fa;
  overflow-y: auto;
}

.comparison-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 25px;
  padding-bottom: 15px;
  border-bottom: 2px solid #dee2e6;
}

.comparison-header h3 {
  margin: 0;
  color: #495057;
}

.comparison-controls {
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

.random-btn {
  background: #28a745;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
}

.comparison-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 20px;
  margin-bottom: 30px;
}

.layout-comparison-item {
  background: white;
  border-radius: 12px;
  padding: 15px;
  box-shadow: 0 2px 10px rgba(0,0,0,0.1);
  display: flex;
  flex-direction: column;
}

.layout-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
  padding-bottom: 10px;
  border-bottom: 1px solid #e9ecef;
}

.layout-header h4 {
  margin: 0;
  color: #343a40;
}

.layout-metrics {
  display: flex;
  gap: 10px;
}

.metric {
  background: #007bff;
  color: white;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 0.8em;
  font-weight: bold;
}

.layout-graph {
  height: 200px;
  background: #fafafa;
  border: 1px solid #e9ecef;
  border-radius: 6px;
  margin-bottom: 15px;
}

.layout-description {
  font-size: 0.9em;
  color: #6c757d;
}

.pros-cons {
  margin-top: 10px;
  font-size: 0.8em;
}

.pros, .cons {
  margin-bottom: 5px;
}

.pros strong {
  color: #28a745;
}

.cons strong {
  color: #dc3545;
}

.performance-summary {
  background: white;
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 2px 10px rgba(0,0,0,0.1);
}

.performance-summary h4 {
  margin: 0 0 15px 0;
  color: #343a40;
}

.summary-table {
  display: grid;
  gap: 1px;
  background: #dee2e6;
  border-radius: 6px;
  overflow: hidden;
}

.summary-header, .summary-row {
  display: grid;
  grid-template-columns: 1fr 80px 80px 2fr;
  background: white;
  padding: 12px 15px;
  align-items: center;
}

.summary-header {
  background: #f8f9fa;
  font-weight: bold;
  color: #495057;
}

.summary-row:nth-child(even) {
  background: #f8f9fa;
}
</style>