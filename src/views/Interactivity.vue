<template>
  <div class="interactivity-container">
    <div class="page-header">
      <h2>🎮 Interactive Demonstrations</h2>
      <p>Explore different visualization modes and interactive systems</p>
    </div>

    <div class="demo-navigation">
      <button 
        v-for="demo in demos" 
        :key="demo.id"
        @click="currentDemo = demo.id"
        :class="['demo-tab', { active: currentDemo === demo.id }]"
      >
        {{ demo.icon }} {{ demo.title }}
      </button>
    </div>

    <div class="demo-content">
      <!-- Choose Your Own Adventure Demo -->
      <div v-if="currentDemo === 'adventure'" class="demo-section">
        <div class="ai-controls">
          <div class="ai-toggle-group">
            <label class="ai-toggle-label">AI Mode:</label>
            <div class="ai-toggle-buttons">
              <button 
                @click="setAIMode('mock')"
                :class="['ai-mode-btn', { active: aiMode === 'mock' }]"
              >
                🤖 Mock AI
              </button>
              <button 
                @click="setAIMode('claude')"
                :class="['ai-mode-btn', { active: aiMode === 'claude' }]"
                :disabled="!hasClaudeKey"
                :title="hasClaudeKey ? 'Use real Claude API' : 'Claude API key not configured'"
              >
                🧠 Real Claude
              </button>
            </div>
            <div class="ai-status">
              <span v-if="aiMode === 'mock'" class="ai-status-mock">
                ⚡ Fast mock responses with simulated delays
              </span>
              <span v-else-if="aiMode === 'claude' && hasClaudeKey" class="ai-status-real">
                🌐 Real Claude API calls (slower, costs tokens)
              </span>
              <span v-else class="ai-status-disabled">
                ⚠️ Set VITE_ANTHROPIC_API_KEY in .env to use real Claude
              </span>
            </div>
          </div>
        </div>
        <AdventureGameV2 
          v-if="currentAIService" 
          ref="adventureGame" 
          :ai-service="currentAIService" 
        />
        <div v-else class="loading-ai">
          <div class="spinner"></div>
          <p>Initializing AI service...</p>
        </div>
      </div>

      <!-- Graph Layout Playground -->
      <div v-else-if="currentDemo === 'graph-layouts'" class="demo-section">
        <GraphPlayground />
      </div>

      <!-- Interactive Story Builder -->
      <div v-else-if="currentDemo === 'story-builder'" class="demo-section">
        <StoryBuilder />
      </div>

      <!-- Layout Comparison -->
      <div v-else-if="currentDemo === 'layout-comparison'" class="demo-section">
        <LayoutComparison />
      </div>

      <!-- Default: Overview -->
      <div v-else class="demo-section overview">
        <div class="overview-grid">
          <div v-for="demo in demos.filter(d => d.id !== 'overview')" :key="demo.id" class="overview-card">
            <div class="card-icon">{{ demo.icon }}</div>
            <h3>{{ demo.title }}</h3>
            <p>{{ demo.description }}</p>
            <button @click="currentDemo = demo.id" class="try-btn">Try It</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import AdventureGame from '../components/AdventureGame.vue'
import AdventureGameV2 from '../components/AdventureGameV2.vue'
import GraphPlayground from '../components/GraphPlayground.vue'
import StoryBuilder from '../components/StoryBuilder.vue'
import LayoutComparison from '../components/LayoutComparison.vue'
import { AIServiceFactory, AIServiceWrapper } from '../services/aiService.js'
import { getAnthropicApiKey } from '../config/env.js'

export default {
  name: 'Interactivity',
  components: {
    AdventureGame,
    AdventureGameV2,
    GraphPlayground,
    StoryBuilder,
    LayoutComparison
  },
  data() {
    return {
      currentDemo: 'overview',
      aiMode: 'mock', // 'mock' or 'claude'
      mockAIService: null,
      claudeAIService: null,
      aiServiceWrapper: null,
      demos: [
        {
          id: 'overview',
          icon: '🏠',
          title: 'Overview',
          description: 'Interactive demonstrations overview'
        },
        {
          id: 'adventure',
          icon: '🗡️',
          title: 'Adventure Game',
          description: 'Choose-your-own-adventure with graph visualization and inventory system'
        },
        {
          id: 'graph-layouts',
          icon: '🕸️',
          title: 'Graph Playground',
          description: 'Experiment with different layout algorithms and graph interactions'
        },
        {
          id: 'story-builder',
          icon: '📖',
          title: 'Story Builder',
          description: 'Build interactive stories with branching narratives'
        },
        {
          id: 'layout-comparison',
          icon: '📊',
          title: 'Layout Comparison',
          description: 'Side-by-side comparison of different graph layout algorithms'
        }
      ]
    }
  },
  computed: {
    hasClaudeKey() {
      try {
        const key = getAnthropicApiKey()
        return key && key !== 'your-api-key-here'
      } catch {
        return false
      }
    },
    
    currentAIService() {
      return this.aiServiceWrapper
    }
  },
  mounted() {
    this.initializeAIServices()
  },
  methods: {
    initializeAIServices() {
      // Create mock AI service
      this.mockAIService = AIServiceFactory.create('mock', {
        minDelay: 800,
        maxDelay: 2000,
        errorRate: 0.05,
        logger: console
      })

      // Create Claude AI service if API key is available
      if (this.hasClaudeKey) {
        try {
          this.claudeAIService = AIServiceFactory.create('claude', {
            apiKey: getAnthropicApiKey(),
            logger: console
          })
        } catch (error) {
          console.error('Failed to initialize Claude AI service:', error)
        }
      }

      this.updateAIServiceWrapper()
    },

    setAIMode(mode) {
      if (mode === 'claude' && !this.hasClaudeKey) {
        console.warn('Cannot switch to Claude mode: API key not configured')
        return
      }
      
      this.aiMode = mode
      this.updateAIServiceWrapper()
    },

    updateAIServiceWrapper() {
      const baseService = this.aiMode === 'claude' && this.claudeAIService 
        ? this.claudeAIService 
        : this.mockAIService

      if (!baseService) {
        console.error('No AI service available')
        return
      }

      // Create wrapper with event callbacks for loading/error states
      this.aiServiceWrapper = new AIServiceWrapper(baseService, {
        onStart: (operation) => {
          console.log(`🔄 AI operation started: ${operation}`)
          this.$refs.adventureGame?.setLoading(true, this.getLoadingMessage(operation))
        },
        onSuccess: (operation, result) => {
          console.log(`✅ AI operation succeeded: ${operation}`, result)
        },
        onError: (operation, error) => {
          console.error(`❌ AI operation failed: ${operation}`, error)
          this.$refs.adventureGame?.showError(`AI Error: ${error.message}`)
        },
        onComplete: (operation) => {
          console.log(`🏁 AI operation completed: ${operation}`)
          this.$refs.adventureGame?.setLoading(false)
        }
      })
    },

    getLoadingMessage(operation) {
      const messages = {
        generateChoices: 'Generating adventure choices...',
        generateStoryNode: 'Creating new story location...',
        analyzeContext: 'Analyzing context...'
      }
      return messages[operation] || 'AI processing...'
    }
  }
}
</script>

<style scoped>
.interactivity-container {
  padding: 20px;
  height: calc(100vh - 40px);
  display: flex;
  flex-direction: column;
}

.page-header {
  text-align: center;
  margin-bottom: 30px;
}

.page-header h2 {
  color: #333;
  font-size: 2.2em;
  margin-bottom: 10px;
}

.page-header p {
  color: #666;
  font-size: 1.1em;
}

.demo-navigation {
  display: flex;
  justify-content: center;
  gap: 5px;
  margin-bottom: 30px;
  border-bottom: 2px solid #f0f0f0;
  padding-bottom: 15px;
}

.demo-tab {
  padding: 10px 20px;
  border: none;
  background: #f8f9fa;
  color: #666;
  border-radius: 8px 8px 0 0;
  cursor: pointer;
  font-size: 1em;
  transition: all 0.3s ease;
  border-bottom: 3px solid transparent;
}

.demo-tab:hover {
  background: #e9ecef;
  color: #333;
}

.demo-tab.active {
  background: #667eea;
  color: white;
  border-bottom-color: #667eea;
}

.demo-content {
  flex: 1;
  overflow: hidden;
}

.demo-section {
  height: 100%;
  width: 100%;
}

.overview {
  display: flex;
  align-items: center;
  justify-content: center;
}

.overview-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 25px;
  max-width: 1000px;
  width: 100%;
}

.overview-card {
  background: white;
  border: 1px solid #e0e0e0;
  border-radius: 12px;
  padding: 25px;
  text-align: center;
  transition: transform 0.3s ease, box-shadow 0.3s ease;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}

.overview-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 8px 25px rgba(0,0,0,0.15);
}

.card-icon {
  font-size: 3em;
  margin-bottom: 15px;
}

.overview-card h3 {
  color: #333;
  margin-bottom: 10px;
  font-size: 1.3em;
}

.overview-card p {
  color: #666;
  line-height: 1.6;
  margin-bottom: 20px;
}

.try-btn {
  background: #667eea;
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 1em;
  transition: background 0.3s ease;
}

.try-btn:hover {
  background: #5a6fd8;
}

/* AI Controls */
.ai-controls {
  background: rgba(102, 126, 234, 0.1);
  border: 1px solid rgba(102, 126, 234, 0.3);
  border-radius: 8px;
  padding: 15px;
  margin-bottom: 20px;
}

.ai-toggle-group {
  display: flex;
  align-items: center;
  gap: 15px;
  flex-wrap: wrap;
}

.ai-toggle-label {
  font-weight: 600;
  color: #333;
  white-space: nowrap;
}

.ai-toggle-buttons {
  display: flex;
  gap: 8px;
}

.ai-mode-btn {
  padding: 8px 16px;
  border: 2px solid #667eea;
  background: white;
  color: #667eea;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9em;
  font-weight: 500;
  transition: all 0.3s ease;
}

.ai-mode-btn:hover:not(:disabled) {
  background: #667eea;
  color: white;
}

.ai-mode-btn.active {
  background: #667eea;
  color: white;
  box-shadow: 0 2px 4px rgba(102, 126, 234, 0.3);
}

.ai-mode-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  border-color: #ccc;
  color: #999;
}

.ai-status {
  flex: 1;
  font-size: 0.85em;
  font-style: italic;
}

.ai-status-mock {
  color: #28a745;
}

.ai-status-real {
  color: #007bff;
}

.ai-status-disabled {
  color: #dc3545;
}

/* Loading AI Service */
.loading-ai {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: #666;
}

.loading-ai .spinner {
  width: 40px;
  height: 40px;
  border: 4px solid rgba(102, 126, 234, 0.2);
  border-top: 4px solid #667eea;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 20px;
}

.loading-ai p {
  font-size: 1.1em;
  margin: 0;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
</style>