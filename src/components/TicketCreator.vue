<template>
  <div class="ticket-creator">
    <h2>Create New Work Item</h2>
    
    <form @submit.prevent="createTicket" class="creation-form">
      <div class="form-group">
        <label>Type:</label>
        <select v-model="form.type" class="form-control">
          <option value="feature">Feature (High-level)</option>
          <option value="ticket">Ticket (Refined)</option>
        </select>
      </div>

      <div class="form-group">
        <label>Title:</label>
        <input 
          v-model="form.title" 
          type="text" 
          class="form-control" 
          placeholder="Brief descriptive title"
          required 
        />
      </div>

      <div class="form-group">
        <label>Description:</label>
        <textarea 
          v-model="form.description" 
          class="form-control description-field" 
          placeholder="Detailed description of the work to be done"
          rows="6"
          required
        ></textarea>
      </div>

      <div class="form-group">
        <label>Priority:</label>
        <select v-model="form.priority" class="form-control">
          <option value="low">Low</option>
          <option value="medium">Medium</option>
          <option value="high">High</option>
          <option value="critical">Critical</option>
        </select>
      </div>

      <div class="form-group">
        <label>Tags (comma-separated):</label>
        <input 
          v-model="tagsInput" 
          type="text" 
          class="form-control" 
          placeholder="frontend, api, database"
        />
      </div>

      <!-- Feature-specific fields -->
      <div v-if="form.type === 'feature'" class="feature-fields">
        <div class="form-group">
          <label>Business Value:</label>
          <textarea 
            v-model="form.businessValue" 
            class="form-control" 
            placeholder="Why is this feature valuable to users/business?"
            rows="3"
          ></textarea>
        </div>

        <div class="form-group">
          <label>Stakeholders (comma-separated):</label>
          <input 
            v-model="stakeholdersInput" 
            type="text" 
            class="form-control" 
            placeholder="Product Manager, Users, Engineering Team"
          />
        </div>

        <div class="form-group">
          <label>Acceptance Criteria:</label>
          <div class="criteria-list">
            <div v-for="(criteria, index) in form.acceptanceCriteria" :key="index" class="criteria-item">
              <input 
                v-model="form.acceptanceCriteria[index]" 
                type="text" 
                class="form-control"
                placeholder="Acceptance criteria"
              />
              <button type="button" @click="removeCriteria(index)" class="remove-btn">×</button>
            </div>
            <button type="button" @click="addCriteria" class="add-criteria-btn">+ Add Criteria</button>
          </div>
        </div>
      </div>

      <div class="form-actions">
        <button type="submit" class="btn btn-primary">Create Work Item</button>
        <button type="button" @click="resetForm" class="btn btn-secondary">Reset</button>
      </div>
    </form>
  </div>
</template>

<script>
import { ticketStore } from '../stores/ticketStore.js'

export default {
  name: 'TicketCreator',
  data() {
    return {
      form: {
        type: 'feature',
        title: '',
        description: '',
        priority: 'medium',
        businessValue: '',
        acceptanceCriteria: ['']
      },
      tagsInput: '',
      stakeholdersInput: ''
    }
  },
  methods: {
    createTicket() {
      const ticketData = {
        ...this.form,
        tags: this.tagsInput.split(',').map(tag => tag.trim()).filter(tag => tag),
        stakeholders: this.stakeholdersInput.split(',').map(s => s.trim()).filter(s => s)
      }

      const newTicket = ticketStore.addTicket(ticketData)
      
      this.$emit('ticket-created', newTicket)
      this.resetForm()
    },

    resetForm() {
      this.form = {
        type: 'feature',
        title: '',
        description: '',
        priority: 'medium',
        businessValue: '',
        acceptanceCriteria: ['']
      }
      this.tagsInput = ''
      this.stakeholdersInput = ''
    },

    addCriteria() {
      this.form.acceptanceCriteria.push('')
    },

    removeCriteria(index) {
      this.form.acceptanceCriteria.splice(index, 1)
    }
  }
}
</script>

<style scoped>
.ticket-creator {
  background: white;
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 20px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}

.creation-form {
  max-width: 600px;
}

.form-group {
  margin-bottom: 15px;
}

.form-group label {
  display: block;
  margin-bottom: 5px;
  font-weight: bold;
  color: #333;
}

.form-control {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.form-control:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 2px rgba(102, 126, 234, 0.2);
}

.description-field {
  min-height: 120px;
  resize: vertical;
}

.feature-fields {
  border-top: 1px solid #eee;
  padding-top: 15px;
  margin-top: 15px;
}

.criteria-list {
  border: 1px solid #ddd;
  border-radius: 4px;
  padding: 10px;
}

.criteria-item {
  display: flex;
  gap: 10px;
  margin-bottom: 10px;
  align-items: center;
}

.criteria-item .form-control {
  flex: 1;
}

.remove-btn {
  background: #ff4757;
  color: white;
  border: none;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  cursor: pointer;
  font-size: 16px;
  line-height: 1;
}

.add-criteria-btn {
  background: #667eea;
  color: white;
  border: none;
  padding: 6px 12px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}

.form-actions {
  display: flex;
  gap: 10px;
  margin-top: 20px;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  font-weight: bold;
}

.btn-primary {
  background: #667eea;
  color: white;
}

.btn-primary:hover {
  background: #5a6fd8;
}

.btn-secondary {
  background: #6c757d;
  color: white;
}

.btn-secondary:hover {
  background: #545b62;
}
</style>