import Anthropic from '@anthropic-ai/sdk'
import { validateApiKey, config } from '../config/env.js'

// Initialize Claude client with proper API key validation
const anthropic = new Anthropic({
  apiKey: validateApiKey(),
  dangerouslyAllowBrowser: true,
})

// Core Claude API call wrapper
async function callClaude(prompt) {
  try {
    const response = await anthropic.messages.create({
      model: config.anthropic.model,
      max_tokens: config.anthropic.maxTokens,
      messages: [{
        role: 'user',
        content: prompt
      }]
    })
    
    const content = response.content[0]
    if (content && content.type === 'text') {
      return content.text
    }
    throw new Error('Unexpected response format from Claude')
  } catch (error) {
    console.error('Claude API error:', error)
    throw error
  }
}

// Extract missing context from ticket description
export async function extractMissingContext(planDescription) {
  const prompt = `
TASK: Analyze what critical information is missing from this software project description.

PROJECT DESCRIPTION: "${planDescription}"

What essential details are missing that would be needed to make concrete implementation decisions?

Consider:
- Technical platform/framework decisions
- Scale and user requirements  
- UI/UX approach
- Integration requirements
- Non-functional requirements

OUTPUT FORMAT (JSON only, no other text):
{
  "missing_context": ["item1", "item2", "item3"],
  "confidence": 0.85,
  "reasoning": "explanation of why these items are critical"
}
`

  try {
    const response = await callClaude(prompt)
    const parsed = JSON.parse(response.trim())
    
    return {
      value: parsed.missing_context,
      confidence: parsed.confidence,
      reasoning: parsed.reasoning
    }
  } catch (error) {
    return {
      value: ["requirements", "scope", "technical_constraints"],
      confidence: 0.3,
      reasoning: `Error in Claude call: ${error instanceof Error ? error.message : String(error)}`
    }
  }
}

// Analyze semantic properties (complexity, scope, implementability)
export async function analyzeSemantics(ticketDescription, ticketContext = {}) {
  const prompt = `
TASK: Analyze the semantic properties of this software development ticket.

TICKET DESCRIPTION: "${ticketDescription}"
CONTEXT: ${JSON.stringify(ticketContext, null, 2)}

Analyze three key aspects:
1. COMPLEXITY: How complex is this work to implement?
2. SCOPE: How well-defined and bounded is the scope?
3. IMPLEMENTABILITY: How ready is this for immediate implementation?

For each aspect, provide a confidence score (0-1) and reasoning.

OUTPUT FORMAT (JSON only, no other text):
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

  try {
    const response = await callClaude(prompt)
    const parsed = JSON.parse(response.trim())
    return parsed
  } catch (error) {
    return {
      complexity: { confidence: 0.5, reasoning: `Error analyzing complexity: ${error.message}` },
      scope: { confidence: 0.5, reasoning: `Error analyzing scope: ${error.message}` },
      implementability: { confidence: 0.5, reasoning: `Error analyzing implementability: ${error.message}` }
    }
  }
}

// Analyze if ticket should be refined/decomposed
export async function analyzeRefinement(ticketDescription, currentState) {
  const prompt = `
TASK: Decide if this ticket should be refined/decomposed into smaller components or implemented as-is.

TICKET: "${ticketDescription}"
CURRENT STATE: "${currentState}"

Consider:
- Complexity of the requirements
- Number of distinct responsibilities
- Whether components would have clear interfaces
- If decomposition would aid development and testing
- Whether this is appropriate scope for a single developer to handle in 1-2 days

OUTPUT FORMAT (JSON only, no other text):
{
  "should_refine": true,
  "confidence": 0.85,
  "reasoning": "detailed explanation of the decision",
  "suggested_breakdown": [
    {
      "title": "Component 1 Name",
      "description": "What this component would do",
      "estimated_complexity": "low|medium|high"
    }
  ]
}
`

  try {
    const response = await callClaude(prompt)
    const parsed = JSON.parse(response.trim())
    
    return {
      shouldRefine: {
        value: parsed.should_refine,
        confidence: parsed.confidence,
        reasoning: parsed.reasoning
      },
      suggestedBreakdown: parsed.suggested_breakdown || null,
      refinementReasoning: parsed.reasoning
    }
  } catch (error) {
    return {
      shouldRefine: {
        value: false,
        confidence: 0.3,
        reasoning: `Error in refinement analysis: ${error.message}`
      },
      suggestedBreakdown: null,
      refinementReasoning: `Analysis failed: ${error.message}`
    }
  }
}

// Generate component interfaces for implementable tickets
export async function generateComponentInterfaces(ticketDescription, context = {}) {
  const prompt = `
TASK: Generate specific component interfaces for this implementable ticket.

TICKET: "${ticketDescription}"
CONTEXT: ${JSON.stringify(context, null, 2)}

Create clear, implementable interfaces with:
- Specific method signatures with types
- Clear input/output specifications
- Preconditions and postconditions where relevant

OUTPUT FORMAT (JSON only, no other text):
{
  "interfaces": [
    {
      "name": "InterfaceName",
      "signature": "methodName(param: type): returnType",
      "purpose": "what this interface does",
      "inputs": [
        {
          "name": "paramName",
          "type": "string",
          "description": "what this parameter represents"
        }
      ],
      "outputs": [
        {
          "name": "returnValue",
          "type": "Promise<boolean>",
          "description": "what is returned"
        }
      ],
      "preconditions": ["condition that must be true before calling"],
      "postconditions": ["condition that will be true after calling"]
    }
  ],
  "confidence": 0.85,
  "reasoning": "explanation of interface design decisions"
}
`

  try {
    const response = await callClaude(prompt)
    const parsed = JSON.parse(response.trim())
    return {
      value: parsed.interfaces,
      confidence: parsed.confidence,
      reasoning: parsed.reasoning
    }
  } catch (error) {
    return {
      value: [],
      confidence: 0.3,
      reasoning: `Error generating interfaces: ${error.message}`
    }
  }
}

// Validate ticket implementation plan
export async function validateImplementation(ticketDescription, interfaces, implementationPlan) {
  const prompt = `
TASK: Validate that this implementation plan actually solves the ticket requirements.

ORIGINAL TICKET: "${ticketDescription}"

DEFINED INTERFACES:
${JSON.stringify(interfaces, null, 2)}

IMPLEMENTATION PLAN:
${JSON.stringify(implementationPlan, null, 2)}

Analysis questions:
- Do the interfaces cover all the requirements from the ticket?
- Are there any missing capabilities?
- Do the interfaces make sense for the stated purpose?
- Is the implementation plan realistic and complete?

OUTPUT FORMAT (JSON only, no other text):
{
  "is_valid": true,
  "confidence": 0.85,
  "reasoning": "detailed analysis of coverage and design quality",
  "gaps": ["any missing capabilities"],
  "issues": ["any design problems identified"]
}
`

  try {
    const response = await callClaude(prompt)
    const parsed = JSON.parse(response.trim())
    
    return {
      value: parsed.is_valid,
      confidence: parsed.confidence,
      reasoning: `${parsed.reasoning}\nGaps: ${parsed.gaps?.join(', ') || 'none'}\nIssues: ${parsed.issues?.join(', ') || 'none'}`
    }
  } catch (error) {
    return {
      value: false,
      confidence: 0.3,
      reasoning: `Error in validation: ${error.message}`
    }
  }
}