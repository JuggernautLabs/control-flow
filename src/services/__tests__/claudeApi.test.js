import { describe, it, expect, vi, beforeEach } from 'vitest'
import * as claudeApi from '../claudeApi.js'

// Mock the Anthropic SDK
vi.mock('@anthropic-ai/sdk', () => {
  const mockCreate = vi.fn()
  
  return {
    default: vi.fn(() => ({
      messages: {
        create: mockCreate
      }
    }))
  }
})

// Mock the env config
vi.mock('../config/env.js', () => ({
  validateApiKey: vi.fn(() => 'test-api-key'),
  config: {
    anthropic: {
      model: 'claude-3-5-sonnet-20241022',
      maxTokens: 1024
    }
  }
}))

describe('Claude API Service', () => {
  let mockAnthropicCreate

  beforeEach(async () => {
    // Get reference to the mocked create function
    const Anthropic = await import('@anthropic-ai/sdk')
    const anthropicInstance = new Anthropic.default()
    mockAnthropicCreate = anthropicInstance.messages.create
  })

  describe('extractMissingContext', () => {
    it('should extract missing context successfully', async () => {
      const mockResponse = {
        content: [{
          type: 'text',
          text: JSON.stringify({
            missing_context: ['platform', 'scale', 'timeline'],
            confidence: 0.85,
            reasoning: 'Project lacks technical and scope details'
          })
        }]
      }
      
      mockAnthropicCreate.mockResolvedValue(mockResponse)
      
      const result = await claudeApi.extractMissingContext('Build a todo app')
      
      expect(result.value).toEqual(['platform', 'scale', 'timeline'])
      expect(result.confidence).toBe(0.85)
      expect(result.reasoning).toContain('technical and scope details')
      
      expect(mockAnthropicCreate).toHaveBeenCalledWith(
        expect.objectContaining({
          model: 'claude-3-5-sonnet-20241022',
          max_tokens: 1024,
          messages: expect.arrayContaining([
            expect.objectContaining({
              role: 'user',
              content: expect.stringContaining('Build a todo app')
            })
          ])
        })
      )
    })

    it('should handle API errors gracefully', async () => {
      mockAnthropicCreate.mockRejectedValue(new Error('API Error'))
      
      const result = await claudeApi.extractMissingContext('Build a todo app')
      
      expect(result.value).toEqual(['requirements', 'scope', 'technical_constraints'])
      expect(result.confidence).toBe(0.3)
      expect(result.reasoning).toContain('Error in Claude call')
    })

    it('should handle malformed JSON response', async () => {
      const mockResponse = {
        content: [{
          type: 'text',
          text: 'Invalid JSON response'
        }]
      }
      
      mockAnthropicCreate.mockResolvedValue(mockResponse)
      
      const result = await claudeApi.extractMissingContext('Build a todo app')
      
      expect(result.value).toEqual(['requirements', 'scope', 'technical_constraints'])
      expect(result.confidence).toBe(0.3)
      expect(result.reasoning).toContain('Error in Claude call')
    })

    it('should handle unexpected response format', async () => {
      const mockResponse = {
        content: [{
          type: 'image',
          data: 'image data'
        }]
      }
      
      mockAnthropicCreate.mockRejectedValue(new Error('Unexpected response format from Claude'))
      
      const result = await claudeApi.extractMissingContext('Build a todo app')
      
      expect(result.confidence).toBe(0.3)
      expect(result.reasoning).toContain('Error in Claude call')
    })
  })

  describe('analyzeSemantics', () => {
    it('should analyze semantic properties successfully', async () => {
      const mockResponse = {
        content: [{
          type: 'text',
          text: JSON.stringify({
            complexity: {
              confidence: 0.8,
              reasoning: 'Medium complexity due to multiple components'
            },
            scope: {
              confidence: 0.9,
              reasoning: 'Well-defined scope with clear boundaries'
            },
            implementability: {
              confidence: 0.7,
              reasoning: 'Needs more technical details for implementation'
            }
          })
        }]
      }
      
      mockAnthropicCreate.mockResolvedValue(mockResponse)
      
      const result = await claudeApi.analyzeSemantics('Build user authentication system', {
        platform: 'React',
        backend: 'Node.js'
      })
      
      expect(result.complexity.confidence).toBe(0.8)
      expect(result.scope.confidence).toBe(0.9)
      expect(result.implementability.confidence).toBe(0.7)
      expect(result.complexity.reasoning).toContain('Medium complexity')
      
      expect(mockAnthropicCreate).toHaveBeenCalledWith(
        expect.objectContaining({
          messages: expect.arrayContaining([
            expect.objectContaining({
              content: expect.stringContaining('user authentication system')
            })
          ])
        })
      )
    })

    it('should handle API errors with default fallback', async () => {
      mockAnthropicCreate.mockRejectedValue(new Error('Network error'))
      
      const result = await claudeApi.analyzeSemantics('Build user system')
      
      expect(result.complexity.confidence).toBe(0.5)
      expect(result.scope.confidence).toBe(0.5)
      expect(result.implementability.confidence).toBe(0.5)
      expect(result.complexity.reasoning).toContain('Error analyzing complexity')
    })
  })

  describe('analyzeRefinement', () => {
    it('should analyze refinement needs successfully', async () => {
      const mockResponse = {
        content: [{
          type: 'text',
          text: JSON.stringify({
            should_refine: true,
            confidence: 0.9,
            reasoning: 'Task is too complex and should be broken down',
            suggested_breakdown: [
              {
                title: 'User Registration',
                description: 'Handle user signup process',
                estimated_complexity: 'medium'
              },
              {
                title: 'Authentication',
                description: 'Handle login/logout',
                estimated_complexity: 'low'
              }
            ]
          })
        }]
      }
      
      mockAnthropicCreate.mockResolvedValue(mockResponse)
      
      const result = await claudeApi.analyzeRefinement(
        'Build complete user management system',
        'feature'
      )
      
      expect(result.shouldRefine.value).toBe(true)
      expect(result.shouldRefine.confidence).toBe(0.9)
      expect(result.suggestedBreakdown).toHaveLength(2)
      expect(result.suggestedBreakdown[0].title).toBe('User Registration')
      expect(result.suggestedBreakdown[1].estimated_complexity).toBe('low')
    })

    it('should handle decision not to refine', async () => {
      const mockResponse = {
        content: [{
          type: 'text',
          text: JSON.stringify({
            should_refine: false,
            confidence: 0.8,
            reasoning: 'Task is appropriately scoped for single developer',
            suggested_breakdown: []
          })
        }]
      }
      
      mockAnthropicCreate.mockResolvedValue(mockResponse)
      
      const result = await claudeApi.analyzeRefinement(
        'Add password reset button',
        'implementable'
      )
      
      expect(result.shouldRefine.value).toBe(false)
      expect(result.shouldRefine.confidence).toBe(0.8)
      expect(result.suggestedBreakdown).toEqual([])
    })

    it('should handle API errors with safe defaults', async () => {
      mockAnthropicCreate.mockRejectedValue(new Error('Rate limit exceeded'))
      
      const result = await claudeApi.analyzeRefinement('Some task', 'feature')
      
      expect(result.shouldRefine.value).toBe(false)
      expect(result.shouldRefine.confidence).toBe(0.3)
      expect(result.suggestedBreakdown).toBeNull()
      expect(result.refinementReasoning).toContain('Analysis failed')
    })
  })

  describe('generateComponentInterfaces', () => {
    it('should generate interfaces successfully', async () => {
      const mockResponse = {
        content: [{
          type: 'text',
          text: JSON.stringify({
            interfaces: [
              {
                name: 'UserService',
                signature: 'createUser(userData: UserData): Promise<User>',
                purpose: 'Create new user account',
                inputs: [
                  {
                    name: 'userData',
                    type: 'UserData',
                    description: 'User registration information'
                  }
                ],
                outputs: [
                  {
                    name: 'user',
                    type: 'Promise<User>',
                    description: 'Created user object'
                  }
                ],
                preconditions: ['Email must be valid format'],
                postconditions: ['User exists in database']
              }
            ],
            confidence: 0.9,
            reasoning: 'Clear interface design based on requirements'
          })
        }]
      }
      
      mockAnthropicCreate.mockResolvedValue(mockResponse)
      
      const result = await claudeApi.generateComponentInterfaces(
        'Implement user registration',
        { platform: 'Node.js', database: 'PostgreSQL' }
      )
      
      expect(result.value).toHaveLength(1)
      expect(result.value[0].name).toBe('UserService')
      expect(result.value[0].signature).toContain('createUser')
      expect(result.value[0].inputs).toHaveLength(1)
      expect(result.value[0].preconditions).toContain('Email must be valid format')
      expect(result.confidence).toBe(0.9)
    })

    it('should handle empty interface generation', async () => {
      const mockResponse = {
        content: [{
          type: 'text',
          text: JSON.stringify({
            interfaces: [],
            confidence: 0.4,
            reasoning: 'Insufficient information to generate specific interfaces'
          })
        }]
      }
      
      mockAnthropicCreate.mockResolvedValue(mockResponse)
      
      const result = await claudeApi.generateComponentInterfaces('Vague requirement')
      
      expect(result.value).toEqual([])
      expect(result.confidence).toBe(0.4)
      expect(result.reasoning).toContain('Insufficient information')
    })

    it('should handle API errors gracefully', async () => {
      mockAnthropicCreate.mockRejectedValue(new Error('Service unavailable'))
      
      const result = await claudeApi.generateComponentInterfaces('Some task')
      
      expect(result.value).toEqual([])
      expect(result.confidence).toBe(0.3)
      expect(result.reasoning).toContain('Error generating interfaces')
    })
  })

  describe('validateImplementation', () => {
    it('should validate implementation successfully', async () => {
      const mockResponse = {
        content: [{
          type: 'text',
          text: JSON.stringify({
            is_valid: true,
            confidence: 0.95,
            reasoning: 'Implementation covers all requirements comprehensively',
            gaps: [],
            issues: []
          })
        }]
      }
      
      mockAnthropicCreate.mockResolvedValue(mockResponse)
      
      const interfaces = [{
        name: 'UserService',
        signature: 'createUser(data: UserData): Promise<User>'
      }]
      
      const plan = {
        steps: ['Validate input', 'Hash password', 'Save to database']
      }
      
      const result = await claudeApi.validateImplementation(
        'Create user registration endpoint',
        interfaces,
        plan
      )
      
      expect(result.value).toBe(true)
      expect(result.confidence).toBe(0.95)
      expect(result.reasoning).toContain('covers all requirements')
    })

    it('should identify implementation gaps', async () => {
      const mockResponse = {
        content: [{
          type: 'text',
          text: JSON.stringify({
            is_valid: false,
            confidence: 0.7,
            reasoning: 'Missing critical error handling components',
            gaps: ['Email validation', 'Password strength check'],
            issues: ['No rollback mechanism', 'Missing rate limiting']
          })
        }]
      }
      
      mockAnthropicCreate.mockResolvedValue(mockResponse)
      
      const result = await claudeApi.validateImplementation(
        'User registration',
        [],
        {}
      )
      
      expect(result.value).toBe(false)
      expect(result.confidence).toBe(0.7)
      expect(result.reasoning).toContain('Email validation, Password strength check')
      expect(result.reasoning).toContain('No rollback mechanism, Missing rate limiting')
    })

    it('should handle validation errors', async () => {
      mockAnthropicCreate.mockRejectedValue(new Error('Validation service down'))
      
      const result = await claudeApi.validateImplementation('Some task', [], {})
      
      expect(result.value).toBe(false)
      expect(result.confidence).toBe(0.3)
      expect(result.reasoning).toContain('Error in validation')
    })
  })

  describe('Error Handling', () => {
    it('should handle network timeouts', async () => {
      mockAnthropicCreate.mockRejectedValue(new Error('timeout'))
      
      const result = await claudeApi.extractMissingContext('test')
      
      expect(result.confidence).toBe(0.3)
      expect(result.reasoning).toContain('timeout')
    })

    it('should handle rate limiting', async () => {
      mockAnthropicCreate.mockRejectedValue(new Error('Rate limit exceeded'))
      
      const result = await claudeApi.analyzeSemantics('test')
      
      expect(result.complexity.confidence).toBe(0.5)
      expect(result.complexity.reasoning).toContain('Rate limit exceeded')
    })

    it('should handle malformed API responses', async () => {
      const mockResponse = {
        content: null
      }
      
      mockAnthropicCreate.mockResolvedValue(mockResponse)
      
      const result = await claudeApi.extractMissingContext('test')
      
      expect(result.confidence).toBe(0.3)
    })
  })

  describe('Input Validation', () => {
    it('should handle empty input strings', async () => {
      const mockResponse = {
        content: [{
          type: 'text',
          text: JSON.stringify({
            missing_context: ['everything'],
            confidence: 0.1,
            reasoning: 'No input provided'
          })
        }]
      }
      
      mockAnthropicCreate.mockResolvedValue(mockResponse)
      
      const result = await claudeApi.extractMissingContext('')
      
      expect(result.value).toEqual(['everything'])
      expect(result.confidence).toBe(0.1)
    })

    it('should handle very long input strings', async () => {
      const longInput = 'a'.repeat(10000)
      
      const mockResponse = {
        content: [{
          type: 'text',
          text: JSON.stringify({
            missing_context: ['scope'],
            confidence: 0.5,
            reasoning: 'Input too verbose'
          })
        }]
      }
      
      mockAnthropicCreate.mockResolvedValue(mockResponse)
      
      const result = await claudeApi.extractMissingContext(longInput)
      
      expect(mockAnthropicCreate).toHaveBeenCalledWith(
        expect.objectContaining({
          messages: expect.arrayContaining([
            expect.objectContaining({
              content: expect.stringContaining(longInput)
            })
          ])
        })
      )
      
      expect(result.value).toEqual(['scope'])
    })
  })
})