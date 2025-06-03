<template>
  <div class="home">
    <div class="welcome-section">
      <h2>Welcome to Control Flow</h2>
      <p>
        This system demonstrates cognitive analysis programming primitives - 
        transforming AI reasoning into composable, testable functions.
      </p>
    </div>

    <div class="features-grid">
      <div class="feature-card">
        <h3>🧠 Context Extraction</h3>
        <p>Identify missing information needed for architectural decisions</p>
      </div>
      
      <div class="feature-card">
        <h3>🔍 Decomposition Analysis</h3>
        <p>Intelligent component breakdown with confidence scoring</p>
      </div>
      
      <div class="feature-card">
        <h3>✅ Semantic Verification</h3>
        <p>Multi-model validation of architectural consistency</p>
      </div>
      
      <div class="feature-card">
        <h3>📊 Ticket Visualization</h3>
        <p>Real-time view of cognitive analysis results</p>
      </div>
    </div>

    <div class="quick-actions">
      <button @click="goToGraph" class="action-btn primary">
        🕸️ Open Dependency Graph
      </button>
      <button @click="goToTickets" class="action-btn secondary">
        📋 Ticket List View
      </button>
      <button @click="startServer" class="action-btn secondary">
        🚀 Start Server
      </button>
    </div>
  </div>
</template>

<script>
export default {
  name: 'Home',
  methods: {
    goToGraph() {
      this.$router.push('/graph')
    },
    goToTickets() {
      this.$router.push('/tickets')
    },
    startServer() {
      alert('Run: node ticket-server.cjs in your terminal to start the server')
    }
  }
}
</script>

<style scoped>
.home {
  padding: 20px 0;
}

.welcome-section {
  text-align: center;
  margin-bottom: 40px;
}

.welcome-section h2 {
  color: #333;
  font-size: 2em;
  margin-bottom: 15px;
}

.welcome-section p {
  color: #666;
  font-size: 1.1em;
  line-height: 1.6;
  max-width: 600px;
  margin: 0 auto;
}

.features-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 20px;
  margin-bottom: 40px;
}

.feature-card {
  background: #f8f9ff;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  padding: 20px;
  text-align: center;
  transition: transform 0.3s ease;
}

.feature-card:hover {
  transform: translateY(-2px);
}

.feature-card h3 {
  color: #667eea;
  margin-bottom: 10px;
}

.feature-card p {
  color: #666;
  font-size: 0.9em;
  line-height: 1.5;
}

.quick-actions {
  display: flex;
  justify-content: center;
  gap: 20px;
}

.action-btn {
  padding: 12px 24px;
  background: #667eea;
  color: white;
  text-decoration: none;
  border: none;
  border-radius: 6px;
  font-size: 1em;
  cursor: pointer;
  transition: background 0.3s ease;
  display: inline-block;
}

.action-btn:hover {
  background: #764ba2;
}

.action-btn.primary {
  background: #667eea;
  font-size: 1.1em;
  padding: 15px 30px;
}

.action-btn.primary:hover {
  background: #5a6fd8;
}

.action-btn.secondary {
  background: #6c757d;
}

.action-btn.secondary:hover {
  background: #5a6268;
}
</style>