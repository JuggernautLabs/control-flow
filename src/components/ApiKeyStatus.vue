<template>
  <div v-if="!apiKeyConfigured" class="api-key-warning">
    <div class="warning-content">
      <h3>⚠️ API Key Required</h3>
      <p>
        Anthropic API key is not configured. Claude-powered analysis features will not work.
      </p>
      <div class="setup-instructions">
        <p><strong>Setup Instructions:</strong></p>
        <ol>
          <li>Get an API key from <a href="https://console.anthropic.com" target="_blank">console.anthropic.com</a></li>
          <li>Add it to your <code>.env</code> file:</li>
          <li><code>VITE_ANTHROPIC_API_KEY=your-api-key-here</code></li>
          <li>Restart the development server</li>
        </ol>
      </div>
      <button @click="checkApiKey" class="btn btn-check">🔄 Check Again</button>
    </div>
  </div>
  <div v-else class="api-key-success">
    <span class="success-indicator">✅ Claude API Ready</span>
  </div>
</template>

<script>
import { getAnthropicApiKey } from '../config/env.js'

export default {
  name: 'ApiKeyStatus',
  data() {
    return {
      apiKeyConfigured: false
    }
  },
  methods: {
    checkApiKey() {
      const apiKey = getAnthropicApiKey()
      this.apiKeyConfigured = !!(apiKey && apiKey !== 'your-api-key-here' && apiKey.startsWith('sk-ant-'))
    }
  },
  mounted() {
    this.checkApiKey()
  }
}
</script>

<style scoped>
.api-key-warning {
  background: #fff3cd;
  border: 1px solid #ffeaa7;
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 20px;
}

.warning-content h3 {
  margin: 0 0 10px 0;
  color: #856404;
}

.warning-content p {
  color: #856404;
  margin-bottom: 15px;
}

.setup-instructions {
  background: #f8f9fa;
  padding: 15px;
  border-radius: 4px;
  margin: 15px 0;
}

.setup-instructions code {
  background: #e9ecef;
  padding: 2px 4px;
  border-radius: 3px;
  font-family: monospace;
}

.setup-instructions ol {
  margin: 10px 0;
  padding-left: 20px;
}

.setup-instructions a {
  color: #007bff;
  text-decoration: none;
}

.setup-instructions a:hover {
  text-decoration: underline;
}

.btn-check {
  background: #007bff;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.btn-check:hover {
  background: #0056b3;
}

.api-key-success {
  text-align: center;
  margin-bottom: 10px;
}

.success-indicator {
  color: #28a745;
  font-weight: bold;
  font-size: 14px;
}
</style>