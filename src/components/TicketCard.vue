<template>
  <div class="ticket-card" :class="stateClass">
    <div class="card-header">
      <div class="ticket-info">
        <h3 class="ticket-title">{{ ticket.title }}</h3>
        <div class="ticket-meta">
          <span class="ticket-id">{{ ticket.id }}</span>
          <span class="ticket-priority" :class="priorityClass">{{ ticket.priority }}</span>
          <span class="ticket-state" :class="stateClass">{{ ticket.refinementState }}</span>
        </div>
      </div>
      <div class="ticket-actions">
        <button @click="toggleExpanded" class="btn btn-sm">
          {{ expanded ? '▼' : '▶' }}
        </button>
      </div>
    </div>

    <div class="card-body">
      <p class="ticket-description">{{ ticket.description }}</p>
      
      <div v-if="ticket.tags.length > 0" class="tags">
        <span v-for="tag in ticket.tags" :key="tag" class="tag">{{ tag }}</span>
      </div>

      <div v-if="expanded" class="expanded-content">
        <!-- Semantic Analysis Section -->
        <div class="analysis-section">
          <h4>Semantic Analysis</h4>
          <div class="semantic-metrics">
            <div class="metric">
              <label>Complexity:</label>
              <div class="confidence-bar">
                <div class="confidence-fill" :style="{ width: (ticket.semanticDescription.complexity.confidence * 100) + '%' }"></div>
                <span class="confidence-text">{{ Math.round(ticket.semanticDescription.complexity.confidence * 100) }}%</span>
              </div>
              <p class="reasoning">{{ ticket.semanticDescription.complexity.reasoning }}</p>
            </div>
            <div class="metric">
              <label>Scope:</label>
              <div class="confidence-bar">
                <div class="confidence-fill" :style="{ width: (ticket.semanticDescription.scope.confidence * 100) + '%' }"></div>
                <span class="confidence-text">{{ Math.round(ticket.semanticDescription.scope.confidence * 100) }}%</span>
              </div>
              <p class="reasoning">{{ ticket.semanticDescription.scope.reasoning }}</p>
            </div>
            <div class="metric">
              <label>Implementability:</label>
              <div class="confidence-bar">
                <div class="confidence-fill" :style="{ width: (ticket.semanticDescription.implementability.confidence * 100) + '%' }"></div>
                <span class="confidence-text">{{ Math.round(ticket.semanticDescription.implementability.confidence * 100) }}%</span>
              </div>
              <p class="reasoning">{{ ticket.semanticDescription.implementability.reasoning }}</p>
            </div>
          </div>
          <div class="analysis-buttons">
            <button @click="runSemanticAnalysis" class="btn btn-analysis">🧠 Run Semantic Analysis</button>
            <button @click="extractMissingContext" class="btn btn-analysis">📝 Extract Missing Context</button>
          </div>
        </div>

        <!-- Context Analysis Results -->
        <div v-if="contextAnalysis" class="context-results">
          <h4>Missing Context Analysis</h4>
          <div class="analysis-result">
            <p><strong>Confidence:</strong> {{ Math.round(contextAnalysis.confidence * 100) }}%</p>
            <p><strong>Missing Items:</strong></p>
            <ul>
              <li v-for="item in contextAnalysis.value" :key="item">{{ item }}</li>
            </ul>
            <p class="reasoning">{{ contextAnalysis.reasoning }}</p>
          </div>
        </div>

        <!-- State Transition Controls -->
        <div class="state-controls">
          <h4>State Management</h4>
          <div class="state-buttons">
            <button 
              v-for="nextState in availableTransitions" 
              :key="nextState"
              @click="transitionState(nextState)"
              class="btn btn-transition"
            >
              → {{ nextState }}
            </button>
          </div>
          <div class="analysis-buttons">
            <button @click="analyzeRefinement" class="btn btn-analysis">🔍 Analyze Refinement</button>
            <button v-if="ticket.refinementState === 'implementable'" @click="generateInterfaces" class="btn btn-analysis">⚙️ Generate Interfaces</button>
            <button v-if="ticket.interfaces && ticket.interfaces.length > 0" @click="generateImplementationPlan" class="btn btn-analysis">📋 Create Implementation Plan</button>
            <button v-if="ticket.implementationPlan" @click="validateImplementation" class="btn btn-analysis">✅ Validate Implementation</button>
          </div>
        </div>

        <!-- Refinement Analysis Results -->
        <div v-if="refinementAnalysis" class="refinement-results">
          <h4>Refinement Analysis</h4>
          <div class="analysis-result">
            <p><strong>Should Refine:</strong> 
              <span :class="refinementAnalysis.shouldRefine.confidence > 0.7 ? 'text-success' : 'text-warning'">
                {{ refinementAnalysis.shouldRefine.confidence > 0.7 ? 'Yes' : 'No' }}
              </span>
              ({{ Math.round(refinementAnalysis.shouldRefine.confidence * 100) }}%)
            </p>
            <p class="reasoning">{{ refinementAnalysis.shouldRefine.reasoning }}</p>
            
            <div v-if="refinementAnalysis.suggestedBreakdown" class="suggested-breakdown">
              <h5>Suggested Breakdown:</h5>
              <div v-for="suggestion in refinementAnalysis.suggestedBreakdown" :key="suggestion.title" class="breakdown-item">
                <h6>{{ suggestion.title }}</h6>
                <p>{{ suggestion.description }}</p>
                <span class="complexity-badge">{{ suggestion.estimatedComplexity }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Context Editing -->
        <div class="context-editing">
          <h4>Additional Context</h4>
          <textarea 
            v-model="additionalContext" 
            class="form-control"
            placeholder="Add additional context, requirements, or notes..."
            rows="3"
          ></textarea>
          <button @click="saveContext" class="btn btn-save">💾 Save Context</button>
        </div>

        <!-- Interfaces (if implementable) -->
        <div v-if="ticket.interfaces && ticket.interfaces.length > 0" class="interfaces-section">
          <h4>Defined Interfaces</h4>
          <div v-for="iface in ticket.interfaces" :key="iface.name" class="interface-item">
            <h5>{{ iface.name }}</h5>
            <code>{{ iface.signature }}</code>
            <p>{{ iface.purpose }}</p>
            <div v-if="iface.inputs && iface.inputs.length > 0" class="interface-details">
              <strong>Inputs:</strong>
              <ul>
                <li v-for="input in iface.inputs" :key="input.name">
                  <code>{{ input.name }}: {{ input.type }}</code> - {{ input.description }}
                </li>
              </ul>
            </div>
            <div v-if="iface.outputs && iface.outputs.length > 0" class="interface-details">
              <strong>Outputs:</strong>
              <ul>
                <li v-for="output in iface.outputs" :key="output.name">
                  <code>{{ output.name }}: {{ output.type }}</code> - {{ output.description }}
                </li>
              </ul>
            </div>
          </div>
        </div>

        <!-- Implementation Plan -->
        <div v-if="ticket.implementationPlan" class="implementation-plan">
          <h4>Implementation Plan</h4>
          <div class="plan-details">
            <p><strong>Estimated Complexity:</strong> {{ ticket.implementationPlan.estimatedComplexity }}</p>
            <p><strong>Created:</strong> {{ formatDate(ticket.implementationPlan.createdAt) }}</p>
            <div v-if="ticket.implementationPlan.dependencies.length > 0">
              <strong>Dependencies:</strong> {{ ticket.implementationPlan.dependencies.join(', ') }}
            </div>
          </div>
        </div>

        <!-- Validation Results -->
        <div v-if="validationResult" class="validation-results">
          <h4>Implementation Validation</h4>
          <div class="analysis-result">
            <p><strong>Valid:</strong> 
              <span :class="validationResult.value ? 'text-success' : 'text-danger'">
                {{ validationResult.value ? 'Yes' : 'No' }}
              </span>
              ({{ Math.round(validationResult.confidence * 100) }}%)
            </p>
            <p class="reasoning">{{ validationResult.reasoning }}</p>
          </div>
        </div>

        <!-- Assignment Controls -->
        <div class="assignment-controls">
          <h4>Assignment</h4>
          <div class="assignment-row">
            <input 
              v-model="assigneeInput" 
              type="text" 
              placeholder="Assignee name"
              class="form-control"
            />
            <button @click="assignTicket" class="btn btn-assign">👤 Assign</button>
            <button v-if="ticket.assignedTo" @click="unassignTicket" class="btn btn-unassign">❌ Unassign</button>
          </div>
          <p v-if="ticket.assignedTo" class="current-assignee">
            Currently assigned to: <strong>{{ ticket.assignedTo }}</strong>
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ticketStore } from '../stores/ticketStore.js'

const StateTransitions = {
  'feature': ['refined'],
  'refined': ['implementable'],
  'implementable': ['planned'],
  'planned': ['in_progress'],
  'in_progress': ['completed'],
  'completed': ['verified'],
  'verified': []
}

export default {
  name: 'TicketCard',
  props: {
    ticket: {
      type: Object,
      required: true
    }
  },
  data() {
    return {
      expanded: false,
      refinementAnalysis: null,
      additionalContext: '',
      assigneeInput: '',
      validationResult: null,
      contextAnalysis: null
    }
  },
  computed: {
    stateClass() {
      return `state-${this.ticket.refinementState}`
    },
    priorityClass() {
      return `priority-${this.ticket.priority}`
    },
    availableTransitions() {
      return StateTransitions[this.ticket.refinementState] || []
    }
  },
  methods: {
    toggleExpanded() {
      this.expanded = !this.expanded
    },

    async runSemanticAnalysis() {
      try {
        await ticketStore.runSemanticAnalysis(this.ticket.id)
        this.$emit('ticket-updated')
      } catch (error) {
        console.error('Semantic analysis failed:', error)
      }
    },

    async analyzeRefinement() {
      try {
        this.refinementAnalysis = await ticketStore.analyzeRefinement(this.ticket.id)
      } catch (error) {
        console.error('Refinement analysis failed:', error)
        this.refinementAnalysis = {
          shouldRefine: {
            value: false,
            confidence: 0.1,
            reasoning: `Analysis failed: ${error.message}`
          },
          suggestedBreakdown: null,
          refinementReasoning: `Error: ${error.message}`
        }
      }
    },

    async generateInterfaces() {
      try {
        const result = await ticketStore.generateInterfaces(this.ticket.id)
        if (result.value.length > 0) {
          this.$emit('ticket-updated')
        }
      } catch (error) {
        console.error('Interface generation failed:', error)
      }
    },

    async generateImplementationPlan() {
      try {
        await ticketStore.generateImplementationPlan(this.ticket.id)
        this.$emit('ticket-updated')
      } catch (error) {
        console.error('Implementation plan generation failed:', error)
      }
    },

    async validateImplementation() {
      try {
        const validation = await ticketStore.validateImplementation(this.ticket.id)
        this.validationResult = validation
      } catch (error) {
        console.error('Implementation validation failed:', error)
      }
    },

    async extractMissingContext() {
      try {
        this.contextAnalysis = await ticketStore.extractMissingContext(this.ticket.id)
      } catch (error) {
        console.error('Context extraction failed:', error)
      }
    },

    transitionState(newState) {
      ticketStore.transitionTicketState(this.ticket.id, newState)
      this.$emit('ticket-updated')
    },

    saveContext() {
      if (this.additionalContext.trim()) {
        const updatedDescription = this.ticket.description + '\n\nAdditional Context:\n' + this.additionalContext
        ticketStore.updateTicket(this.ticket.id, { description: updatedDescription })
        this.additionalContext = ''
        this.$emit('ticket-updated')
      }
    },

    assignTicket() {
      if (this.assigneeInput.trim()) {
        ticketStore.updateTicket(this.ticket.id, { 
          assignedTo: this.assigneeInput.trim(),
          pickupTimestamp: new Date().toISOString()
        })
        this.assigneeInput = ''
        this.$emit('ticket-updated')
      }
    },

    unassignTicket() {
      ticketStore.updateTicket(this.ticket.id, { 
        assignedTo: null,
        pickupTimestamp: null
      })
      this.$emit('ticket-updated')
    }
  }
}
</script>

<style scoped>
.ticket-card {
  background: white;
  border-radius: 8px;
  border-left: 4px solid #ddd;
  margin-bottom: 15px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  transition: all 0.3s ease;
}

.ticket-card:hover {
  box-shadow: 0 4px 8px rgba(0,0,0,0.15);
}

.state-feature { border-left-color: #e74c3c; }
.state-refined { border-left-color: #f39c12; }
.state-implementable { border-left-color: #3498db; }
.state-planned { border-left-color: #9b59b6; }
.state-in_progress { border-left-color: #2ecc71; }
.state-completed { border-left-color: #27ae60; }
.state-verified { border-left-color: #16a085; }

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 20px;
  border-bottom: 1px solid #eee;
}

.ticket-title {
  margin: 0 0 8px 0;
  color: #333;
  font-size: 1.1em;
}

.ticket-meta {
  display: flex;
  gap: 10px;
  font-size: 0.85em;
}

.ticket-id {
  color: #666;
  font-family: monospace;
}

.ticket-priority, .ticket-state {
  padding: 2px 6px;
  border-radius: 3px;
  font-weight: bold;
  text-transform: uppercase;
  font-size: 0.7em;
}

.priority-low { background: #d4edda; color: #155724; }
.priority-medium { background: #fff3cd; color: #856404; }
.priority-high { background: #f8d7da; color: #721c24; }
.priority-critical { background: #f5c6cb; color: #491217; }

.card-body {
  padding: 15px 20px;
}

.ticket-description {
  color: #555;
  line-height: 1.5;
  margin-bottom: 10px;
}

.tags {
  margin-bottom: 15px;
}

.tag {
  display: inline-block;
  background: #f8f9fa;
  color: #495057;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 0.8em;
  margin-right: 5px;
}

.expanded-content {
  border-top: 1px solid #eee;
  padding-top: 15px;
  margin-top: 15px;
}

.analysis-section, .state-controls, .refinement-results, .context-editing, .interfaces-section, .assignment-controls, .context-results, .implementation-plan, .validation-results {
  margin-bottom: 20px;
  padding: 15px;
  background: #f8f9fa;
  border-radius: 6px;
}

.analysis-section h4, .state-controls h4, .refinement-results h4, .context-editing h4, .interfaces-section h4, .assignment-controls h4, .context-results h4, .implementation-plan h4, .validation-results h4 {
  margin: 0 0 10px 0;
  color: #333;
  font-size: 1em;
}

.analysis-buttons, .state-buttons {
  display: flex;
  gap: 10px;
  margin-bottom: 10px;
  flex-wrap: wrap;
}

.interface-details {
  margin-top: 10px;
  font-size: 0.9em;
}

.interface-details ul {
  margin: 5px 0;
  padding-left: 20px;
}

.interface-details code {
  background: #e9ecef;
  padding: 2px 4px;
  border-radius: 3px;
  font-size: 0.85em;
}

.plan-details p {
  margin: 5px 0;
}

.text-danger {
  color: #dc3545;
}

.analysis-result {
  background: white;
  padding: 10px;
  border-radius: 4px;
  margin-top: 10px;
}

.semantic-metrics .metric {
  margin-bottom: 15px;
}

.metric label {
  display: block;
  font-weight: bold;
  margin-bottom: 5px;
  color: #555;
}

.confidence-bar {
  position: relative;
  height: 20px;
  background: #e9ecef;
  border-radius: 10px;
  overflow: hidden;
  margin-bottom: 5px;
}

.confidence-fill {
  height: 100%;
  background: linear-gradient(90deg, #e74c3c 0%, #f39c12 50%, #27ae60 100%);
  transition: width 0.3s ease;
}

.confidence-text {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 0.8em;
  font-weight: bold;
  color: #333;
}

.reasoning {
  font-size: 0.85em;
  color: #666;
  margin: 0;
  font-style: italic;
}

.state-buttons {
  display: flex;
  gap: 10px;
  margin-bottom: 10px;
  flex-wrap: wrap;
}

.btn {
  padding: 6px 12px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.85em;
  transition: background-color 0.3s ease;
}

.btn-sm {
  padding: 4px 8px;
  font-size: 0.8em;
}

.btn-analysis {
  background: #667eea;
  color: white;
}

.btn-analysis:hover {
  background: #5a6fd8;
}

.btn-transition {
  background: #6c757d;
  color: white;
}

.btn-transition:hover {
  background: #545b62;
}

.btn-save {
  background: #28a745;
  color: white;
  margin-top: 10px;
}

.btn-save:hover {
  background: #218838;
}

.btn-assign {
  background: #17a2b8;
  color: white;
}

.btn-assign:hover {
  background: #138496;
}

.btn-unassign {
  background: #dc3545;
  color: white;
}

.btn-unassign:hover {
  background: #c82333;
}

.form-control {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 0.9em;
}

.assignment-row {
  display: flex;
  gap: 10px;
  align-items: center;
  margin-bottom: 10px;
}

.assignment-row .form-control {
  flex: 1;
}

.current-assignee {
  margin: 0;
  color: #555;
}

.text-success { color: #28a745; }
.text-warning { color: #ffc107; }

.breakdown-item {
  background: white;
  padding: 10px;
  border-radius: 4px;
  margin-bottom: 10px;
}

.breakdown-item h6 {
  margin: 0 0 5px 0;
  color: #333;
}

.complexity-badge {
  display: inline-block;
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 0.7em;
  font-weight: bold;
  background: #e9ecef;
  color: #495057;
}

.interface-item {
  background: white;
  padding: 10px;
  border-radius: 4px;
  margin-bottom: 10px;
}

.interface-item h5 {
  margin: 0 0 5px 0;
  color: #667eea;
}

.interface-item code {
  display: block;
  background: #f8f9fa;
  padding: 5px;
  border-radius: 3px;
  font-size: 0.9em;
  margin-bottom: 5px;
}
</style>