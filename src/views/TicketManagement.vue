<template>
  <div class="ticket-management">
    <div class="management-header">
      <h1>🎯 Global Feature Queue</h1>
      <div class="queue-stats">
        <div class="stat-card">
          <span class="stat-number">{{ tickets.length }}</span>
          <span class="stat-label">Total Tickets</span>
        </div>
        <div class="stat-card">
          <span class="stat-number">{{ availableWork.length }}</span>
          <span class="stat-label">Available Work</span>
        </div>
        <div class="stat-card">
          <span class="stat-number">{{ inProgressTickets.length }}</span>
          <span class="stat-label">In Progress</span>
        </div>
      </div>
    </div>

    <!-- API Key Status -->
    <ApiKeyStatus />

    <!-- Ticket Creator -->
    <TicketCreator @ticket-created="onTicketCreated" />

    <!-- Filter Controls -->
    <div class="filter-controls">
      <div class="filter-group">
        <label>Filter by State:</label>
        <select v-model="stateFilter" class="filter-select">
          <option value="">All States</option>
          <option value="feature">Features</option>
          <option value="refined">Refined</option>
          <option value="implementable">Implementable</option>
          <option value="planned">Planned</option>
          <option value="in_progress">In Progress</option>
          <option value="completed">Completed</option>
          <option value="verified">Verified</option>
        </select>
      </div>
      
      <div class="filter-group">
        <label>Filter by Priority:</label>
        <select v-model="priorityFilter" class="filter-select">
          <option value="">All Priorities</option>
          <option value="critical">Critical</option>
          <option value="high">High</option>
          <option value="medium">Medium</option>
          <option value="low">Low</option>
        </select>
      </div>

      <div class="filter-group">
        <label>Show Only:</label>
        <div class="checkbox-group">
          <label class="checkbox-label">
            <input type="checkbox" v-model="showOnlyAvailable" />
            Available Work
          </label>
          <label class="checkbox-label">
            <input type="checkbox" v-model="showOnlyAssigned" />
            Assigned to Me
          </label>
        </div>
      </div>
    </div>

    <!-- Ticket List -->
    <div class="ticket-list">
      <div v-if="filteredTickets.length === 0" class="empty-state">
        <p>No tickets match your current filters.</p>
        <button @click="clearFilters" class="btn btn-secondary">Clear Filters</button>
      </div>
      
      <TicketCard 
        v-for="ticket in filteredTickets" 
        :key="ticket.id" 
        :ticket="ticket"
        @ticket-updated="refreshTickets"
      />
    </div>

    <!-- Bulk Actions -->
    <div v-if="filteredTickets.length > 0" class="bulk-actions">
      <h3>Bulk Actions</h3>
      <div class="bulk-buttons">
        <button @click="runBulkSemanticAnalysis" class="btn btn-analysis">
          🧠 Run Semantic Analysis on All Visible
        </button>
        <button @click="exportTickets" class="btn btn-export">
          📄 Export Visible Tickets
        </button>
      </div>
    </div>
  </div>
</template>

<script>
import { ticketStore } from '../stores/ticketStore.js'
import TicketCreator from '../components/TicketCreator.vue'
import TicketCard from '../components/TicketCard.vue'
import ApiKeyStatus from '../components/ApiKeyStatus.vue'

export default {
  name: 'TicketManagement',
  components: {
    TicketCreator,
    TicketCard,
    ApiKeyStatus
  },
  data() {
    return {
      stateFilter: '',
      priorityFilter: '',
      showOnlyAvailable: false,
      showOnlyAssigned: false,
      refreshKey: 0
    }
  },
  computed: {
    tickets() {
      // Force reactivity refresh
      this.refreshKey
      return ticketStore.tickets
    },
    
    availableWork() {
      return ticketStore.getAvailableWork()
    },
    
    inProgressTickets() {
      return ticketStore.getTicketsByState('in_progress')
    },
    
    filteredTickets() {
      let filtered = [...this.tickets]
      
      // State filter
      if (this.stateFilter) {
        filtered = filtered.filter(ticket => ticket.refinementState === this.stateFilter)
      }
      
      // Priority filter
      if (this.priorityFilter) {
        filtered = filtered.filter(ticket => ticket.priority === this.priorityFilter)
      }
      
      // Available work filter
      if (this.showOnlyAvailable) {
        filtered = filtered.filter(ticket => 
          ['implementable', 'planned'].includes(ticket.refinementState) && 
          !ticket.assignedTo
        )
      }
      
      // Assigned filter (mock - would need user context in real app)
      if (this.showOnlyAssigned) {
        filtered = filtered.filter(ticket => ticket.assignedTo === 'Current User')
      }
      
      // Sort by priority and creation date
      return filtered.sort((a, b) => {
        const priorityOrder = { critical: 4, high: 3, medium: 2, low: 1 }
        const priorityDiff = priorityOrder[b.priority] - priorityOrder[a.priority]
        if (priorityDiff !== 0) return priorityDiff
        return new Date(b.createdAt) - new Date(a.createdAt)
      })
    }
  },
  methods: {
    onTicketCreated(ticket) {
      // Ticket is already added to store, just refresh
      this.refreshTickets()
    },
    
    refreshTickets() {
      // Force reactivity by updating refresh key
      this.refreshKey++
    },
    
    clearFilters() {
      this.stateFilter = ''
      this.priorityFilter = ''
      this.showOnlyAvailable = false
      this.showOnlyAssigned = false
    },
    
    async runBulkSemanticAnalysis() {
      const promises = this.filteredTickets.map(ticket => 
        ticketStore.runSemanticAnalysis(ticket.id)
      )
      
      try {
        await Promise.all(promises)
        this.refreshTickets()
      } catch (error) {
        console.error('Bulk semantic analysis failed:', error)
      }
    },
    
    exportTickets() {
      const exportData = this.filteredTickets.map(ticket => ({
        id: ticket.id,
        title: ticket.title,
        description: ticket.description,
        state: ticket.refinementState,
        priority: ticket.priority,
        assignedTo: ticket.assignedTo,
        createdAt: ticket.createdAt,
        semanticAnalysis: ticket.semanticDescription
      }))
      
      const blob = new Blob([JSON.stringify(exportData, null, 2)], { 
        type: 'application/json' 
      })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `tickets-export-${new Date().toISOString().slice(0, 10)}.json`
      document.body.appendChild(a)
      a.click()
      document.body.removeChild(a)
      URL.revokeObjectURL(url)
    }
  },
  
  mounted() {
    // Create sample hierarchy if no tickets exist
    if (ticketStore.tickets.length === 0) {
      ticketStore.createSampleHierarchy()
    }
  }
}
</script>

<style scoped>
.ticket-management {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.management-header {
  margin-bottom: 30px;
}

.management-header h1 {
  margin: 0 0 20px 0;
  color: #333;
}

.queue-stats {
  display: flex;
  gap: 20px;
  margin-bottom: 20px;
}

.stat-card {
  background: white;
  padding: 15px 20px;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  text-align: center;
  min-width: 120px;
}

.stat-number {
  display: block;
  font-size: 2em;
  font-weight: bold;
  color: #667eea;
  line-height: 1;
}

.stat-label {
  display: block;
  font-size: 0.9em;
  color: #666;
  margin-top: 5px;
}

.filter-controls {
  background: white;
  padding: 20px;
  border-radius: 8px;
  margin-bottom: 20px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  display: flex;
  gap: 20px;
  align-items: end;
  flex-wrap: wrap;
}

.filter-group {
  display: flex;
  flex-direction: column;
  min-width: 150px;
}

.filter-group label {
  font-weight: bold;
  margin-bottom: 5px;
  color: #333;
  font-size: 0.9em;
}

.filter-select {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 0.9em;
}

.checkbox-group {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: normal;
  margin-bottom: 0;
  cursor: pointer;
}

.checkbox-label input[type="checkbox"] {
  margin: 0;
}

.ticket-list {
  margin-bottom: 30px;
}

.empty-state {
  text-align: center;
  padding: 40px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.empty-state p {
  color: #666;
  margin-bottom: 15px;
}

.bulk-actions {
  background: white;
  padding: 20px;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.bulk-actions h3 {
  margin: 0 0 15px 0;
  color: #333;
}

.bulk-buttons {
  display: flex;
  gap: 15px;
  flex-wrap: wrap;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9em;
  font-weight: bold;
  transition: background-color 0.3s ease;
}

.btn-secondary {
  background: #6c757d;
  color: white;
}

.btn-secondary:hover {
  background: #545b62;
}

.btn-analysis {
  background: #667eea;
  color: white;
}

.btn-analysis:hover {
  background: #5a6fd8;
}

.btn-export {
  background: #28a745;
  color: white;
}

.btn-export:hover {
  background: #218838;
}
</style>