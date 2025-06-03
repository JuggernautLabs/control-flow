/**
 * AI Service Interface - Abstracts AI calls with consistent error handling
 */

export class AIServiceInterface {
  constructor() {
    if (this.constructor === AIServiceInterface) {
      throw new Error('AIServiceInterface is abstract and cannot be instantiated')
    }
  }

  async generateChoices(currentNode, context = {}) {
    throw new Error('generateChoices must be implemented by subclass')
  }

  async generateStoryNode(choice, fromNode, context = {}) {
    throw new Error('generateStoryNode must be implemented by subclass')
  }

  async analyzeContext(description, context = {}) {
    throw new Error('analyzeContext must be implemented by subclass')
  }
}

/**
 * Mock AI Service - Simulates AI responses with realistic delays
 */
export class MockAIService extends AIServiceInterface {
  constructor(options = {}) {
    super()
    this.minDelay = options.minDelay || 800
    this.maxDelay = options.maxDelay || 2000
    this.errorRate = options.errorRate || 0.05 // 5% error rate
    this.logger = options.logger || console
  }

  async _simulateDelay() {
    const delay = Math.random() * (this.maxDelay - this.minDelay) + this.minDelay
    this.logger.log(`🤖 [MOCK AI] Simulating ${Math.round(delay)}ms processing delay`)
    await new Promise(resolve => setTimeout(resolve, delay))
  }

  async _maybeThrowError(operation) {
    if (Math.random() < this.errorRate) {
      const errors = [
        'Mock API rate limit exceeded',
        'Mock service temporarily unavailable', 
        'Mock network timeout',
        'Mock authentication failed'
      ]
      const error = errors[Math.floor(Math.random() * errors.length)]
      this.logger.error(`🤖 [MOCK AI] Simulated error: ${error}`)
      throw new Error(error)
    }
  }

  async generateChoices(currentNode, context = {}) {
    this.logger.log('🤖 [MOCK AI] Generating choices for:', currentNode.location)
    
    await this._simulateDelay()
    await this._maybeThrowError('generateChoices')

    const mockChoices = [
      {
        text: "Learn about Microservices",
        icon: "🏗️",
        cost: 0,
        experience: 15,
        description: "Explore distributed architecture patterns",
        requiresItem: null,
        risk: "low"
      },
      {
        text: "Study Monolithic Design", 
        icon: "🏰",
        cost: 0,
        experience: 15,
        description: "Understand single-deployment architectures",
        requiresItem: null,
        risk: "low"
      },
      {
        text: "Visit the Knowledge Shop",
        icon: "🏪", 
        cost: 5,
        experience: 5,
        description: "Buy helpful architectural tools",
        requiresItem: null,
        risk: "low"
      }
    ]

    this.logger.log(`🤖 [MOCK AI] Generated ${mockChoices.length} choices`)
    return {
      choices: mockChoices,
      confidence: 0.85,
      reasoning: 'Mock generated choices for architectural learning'
    }
  }

  async generateStoryNode(choice, fromNode, context = {}) {
    this.logger.log('🤖 [MOCK AI] Generating story node for choice:', choice.text)
    
    await this._simulateDelay()
    await this._maybeThrowError('generateStoryNode')

    const mockScenarios = {
      microservices: {
        location: "The Distributed Lands",
        situation: `
          <p>You venture into the <strong>Distributed Lands</strong> where services operate independently.</p>
          <p>Each service-village handles its own domain, but you notice communication challenges between them.</p>
          <p>A service elder approaches: <em>"How shall we handle data consistency across our distributed services?"</em></p>
        `
      },
      monolith: {
        location: "The Great Monolith",
        situation: `
          <p>You approach the <strong>Great Monolith</strong> - a massive structure housing all functionality.</p>
          <p>It's impressive but showing signs of complexity. The Guardian explains the maintenance challenges.</p>
          <p><em>"Should we refactor this into modules, or is the current structure sufficient?"</em></p>
        `
      },
      shop: {
        location: "The Knowledge Shop",
        situation: `
          <p>Inside the mystical shop, scrolls and artifacts glow with architectural wisdom.</p>
          <p>The shopkeeper offers tools that can help with future challenges.</p>
          <p><em>"Choose wisely - these items will unlock new paths in your journey."</em></p>
        `
      }
    }

    // Select appropriate scenario based on choice
    let scenario = mockScenarios.microservices
    if (choice.text.toLowerCase().includes('monolith')) {
      scenario = mockScenarios.monolith
    } else if (choice.text.toLowerCase().includes('shop')) {
      scenario = mockScenarios.shop
    }

    this.logger.log(`🤖 [MOCK AI] Generated scenario: ${scenario.location}`)
    return {
      location: scenario.location,
      situation: scenario.situation,
      confidence: 0.9,
      reasoning: 'Mock generated story node based on choice context'
    }
  }

  async analyzeContext(description, context = {}) {
    this.logger.log('🤖 [MOCK AI] Analyzing context for:', description.substring(0, 50) + '...')
    
    await this._simulateDelay()
    await this._maybeThrowError('analyzeContext')

    return {
      complexity: { confidence: 0.8, reasoning: 'Mock complexity analysis' },
      scope: { confidence: 0.7, reasoning: 'Mock scope analysis' },
      implementability: { confidence: 0.9, reasoning: 'Mock implementability analysis' }
    }
  }
}

/**
 * Real Claude AI Service - Makes actual API calls
 */
export class ClaudeAIService extends AIServiceInterface {
  constructor(options = {}) {
    super()
    this.apiKey = options.apiKey
    this.model = options.model || 'claude-3-5-sonnet-20241022'
    this.maxTokens = options.maxTokens || 1024
    this.logger = options.logger || console
    
    if (!this.apiKey) {
      throw new Error('Claude API key is required for ClaudeAIService')
    }
  }

  async _callClaude(prompt) {
    this.logger.log('🧠 [CLAUDE AI] Making API call...')
    
    try {
      const response = await fetch('https://api.anthropic.com/v1/messages', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'x-api-key': this.apiKey,
          'anthropic-version': '2023-06-01'
        },
        body: JSON.stringify({
          model: this.model,
          max_tokens: this.maxTokens,
          messages: [{
            role: 'user',
            content: prompt
          }]
        })
      })

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}))
        throw new Error(`Claude API error: ${response.status} - ${errorData.error?.message || 'Unknown error'}`)
      }

      const data = await response.json()
      const content = data.content?.[0]?.text
      
      if (!content) {
        throw new Error('Invalid response format from Claude API')
      }

      this.logger.log('🧠 [CLAUDE AI] API call successful')
      return content
    } catch (error) {
      this.logger.error('🧠 [CLAUDE AI] API call failed:', error.message)
      throw error
    }
  }

  async generateChoices(currentNode, context = {}) {
    this.logger.log('🧠 [CLAUDE AI] Generating choices for:', currentNode.location)
    
    const prompt = `
CONTEXT: You are generating choices for an interactive adventure game teaching software architecture concepts.

CURRENT SITUATION: "${currentNode.situation}"
LOCATION: "${currentNode.location}"
CONTEXT: ${JSON.stringify(context)}

Generate 2-4 meaningful choices that:
1. Teach software architecture concepts
2. Lead to interesting new scenarios
3. Have clear consequences (positive/negative outcomes)
4. Include some that require resources (gold) or items

OUTPUT FORMAT (JSON only):
{
  "choices": [
    {
      "text": "Choice description",
      "icon": "🏗️", 
      "cost": 0,
      "experience": 10,
      "description": "What this choice leads to",
      "requiresItem": null,
      "risk": "low"
    }
  ],
  "confidence": 0.85,
  "reasoning": "explanation of choice design"
}
    `

    const response = await this._callClaude(prompt)
    const parsed = JSON.parse(response.trim())
    
    this.logger.log(`🧠 [CLAUDE AI] Generated ${parsed.choices?.length || 0} choices`)
    return parsed
  }

  async generateStoryNode(choice, fromNode, context = {}) {
    this.logger.log('🧠 [CLAUDE AI] Generating story node for choice:', choice.text)
    
    const prompt = `
CONTEXT: Player chose "${choice.text}" from "${fromNode.location}"

PREVIOUS SITUATION: "${fromNode.situation}"
CHOICE MADE: "${choice.text}"
CONTEXT: ${JSON.stringify(context)}

Generate a new story node that:
1. Follows logically from the choice
2. Teaches software architecture
3. Has meaningful consequences
4. Sets up future decisions

OUTPUT FORMAT (JSON only):
{
  "location": "New Location Name",
  "situation": "<p>HTML formatted story text with architecture lessons</p>",
  "confidence": 0.9,
  "reasoning": "explanation of story progression"
}
    `

    const response = await this._callClaude(prompt)
    const parsed = JSON.parse(response.trim())
    
    this.logger.log(`🧠 [CLAUDE AI] Generated scenario: ${parsed.location}`)
    return parsed
  }

  async analyzeContext(description, context = {}) {
    this.logger.log('🧠 [CLAUDE AI] Analyzing context for:', description.substring(0, 50) + '...')
    
    const prompt = `
TASK: Analyze the semantic properties of this software development scenario.

DESCRIPTION: "${description}"
CONTEXT: ${JSON.stringify(context)}

Analyze three key aspects:
1. COMPLEXITY: How complex is this scenario?
2. SCOPE: How well-defined and bounded is the scope?
3. IMPLEMENTABILITY: How ready is this for implementation?

OUTPUT FORMAT (JSON only):
{
  "complexity": {
    "confidence": 0.85,
    "reasoning": "explanation of complexity assessment"
  },
  "scope": {
    "confidence": 0.90,
    "reasoning": "explanation of scope clarity"
  },
  "implementability": {
    "confidence": 0.75,
    "reasoning": "explanation of implementation readiness"
  }
}
    `

    const response = await this._callClaude(prompt)
    const parsed = JSON.parse(response.trim())
    
    this.logger.log('🧠 [CLAUDE AI] Context analysis complete')
    return parsed
  }
}

/**
 * AI Service Factory - Creates appropriate service based on configuration
 */
export class AIServiceFactory {
  static create(type, options = {}) {
    switch (type) {
      case 'mock':
        return new MockAIService(options)
      case 'claude':
        return new ClaudeAIService(options)
      default:
        throw new Error(`Unknown AI service type: ${type}`)
    }
  }
}

/**
 * AI Service Wrapper - Adds consistent error handling and logging to any AI service
 */
export class AIServiceWrapper {
  constructor(aiService, eventCallbacks = {}) {
    this.aiService = aiService
    this.onStart = eventCallbacks.onStart || (() => {})
    this.onSuccess = eventCallbacks.onSuccess || (() => {})
    this.onError = eventCallbacks.onError || (() => {})
    this.onComplete = eventCallbacks.onComplete || (() => {})
  }

  async _wrapCall(operation, ...args) {
    const operationName = operation.name
    this.onStart(operationName)
    
    try {
      const result = await operation.apply(this.aiService, args)
      this.onSuccess(operationName, result)
      return result
    } catch (error) {
      this.onError(operationName, error)
      throw error
    } finally {
      this.onComplete(operationName)
    }
  }

  async generateChoices(...args) {
    return this._wrapCall(this.aiService.generateChoices, ...args)
  }

  async generateStoryNode(...args) {
    return this._wrapCall(this.aiService.generateStoryNode, ...args)
  }

  async analyzeContext(...args) {
    return this._wrapCall(this.aiService.analyzeContext, ...args)
  }
}