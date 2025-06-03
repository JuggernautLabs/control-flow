/**
 * Cognitive Analysis Framework with Claude API
 * Real implementation using Anthropic's Claude API
 */

import 'dotenv/config';
import Anthropic from '@anthropic-ai/sdk';
import { writeFileSync } from 'fs';
import { join } from 'path';

// Initialize Claude client
const anthropic = new Anthropic({
  apiKey: process.env.ANTHROPIC_API_KEY || 'your-api-key-here',
});

// Core types
interface AnalysisResult<T> {
  value: T;
  confidence: number;
  reasoning: string;
}

interface Component {
  name: string;
  purpose: string;
  interfaces: string[];
  dependencies: string[];
}

// Real Claude API call
async function callClaude(prompt: string): Promise<string> {
  try {
    const response = await anthropic.messages.create({
      model: 'claude-3-5-sonnet-20241022',
      max_tokens: 1024,
      messages: [{
        role: 'user',
        content: prompt
      }]
    });
    
    const content = response.content[0];
    if (content && content.type === 'text') {
      return content.text;
    }
    throw new Error('Unexpected response format from Claude');
  } catch (error) {
    console.error('Claude API error:', error);
    throw error;
  }
}

// Cognitive Primitives
async function extractMissingContext(planDescription: string): Promise<AnalysisResult<string[]>> {
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
`;

  try {
    const response = await callClaude(prompt);
    const parsed = JSON.parse(response.trim());
    
    return {
      value: parsed.missing_context,
      confidence: parsed.confidence,
      reasoning: parsed.reasoning
    };
  } catch (error) {
    return {
      value: ["requirements", "scope", "technical_constraints"],
      confidence: 0.3,
      reasoning: `Error in Claude call: ${error instanceof Error ? error.message : String(error)}`
    };
  }
}

async function shouldDecompose(planDescription: string, context: Record<string, any>): Promise<AnalysisResult<boolean>> {
  const prompt = `
TASK: Decide if this software project should be decomposed into components or implemented as a single unit.

PROJECT: "${planDescription}"
CONTEXT: ${JSON.stringify(context, null, 2)}

Consider:
- Complexity of the requirements
- Number of distinct responsibilities
- Whether components would have clear interfaces
- If decomposition would aid development and testing

OUTPUT FORMAT (JSON only, no other text):
{
  "should_decompose": true,
  "confidence": 0.85,
  "reasoning": "detailed explanation of the decision"
}
`;

  try {
    const response = await callClaude(prompt);
    const parsed = JSON.parse(response.trim());
    
    return {
      value: parsed.should_decompose,
      confidence: parsed.confidence,
      reasoning: parsed.reasoning
    };
  } catch (error) {
    return {
      value: false,
      confidence: 0.3,
      reasoning: `Error in decomposition analysis: ${error instanceof Error ? error.message : String(error)}`
    };
  }
}

async function generateComponents(planDescription: string, context: Record<string, any>): Promise<AnalysisResult<Component[]>> {
  const prompt = `
TASK: Break down this software project into implementable components with clear interfaces.

PROJECT: "${planDescription}"
CONTEXT: ${JSON.stringify(context, null, 2)}

Generate 2-6 components that:
- Have single, clear responsibilities
- Have well-defined interfaces with specific method signatures
- Minimize coupling between components
- Are actually implementable by a developer

OUTPUT FORMAT (JSON only, no other text):
{
  "components": [
    {
      "name": "ComponentName",
      "purpose": "What this component does",
      "interfaces": ["method1(param: type): returnType", "method2(param: type): returnType"],
      "dependencies": ["other_component_names"]
    }
  ],
  "confidence": 0.85,
  "reasoning": "explanation of the decomposition approach"
}
`;

  try {
    const response = await callClaude(prompt);
    const parsed = JSON.parse(response.trim());
    
    return {
      value: parsed.components,
      confidence: parsed.confidence,
      reasoning: parsed.reasoning
    };
  } catch (error) {
    return {
      value: [],
      confidence: 0.2,
      reasoning: `Error in component generation: ${error instanceof Error ? error.message : String(error)}`
    };
  }
}

async function validateArchitecture(originalPlan: string, components: Component[]): Promise<AnalysisResult<boolean>> {
  const prompt = `
TASK: Validate that these components actually solve the original project requirements.

ORIGINAL PROJECT: "${originalPlan}"

PROPOSED COMPONENTS:
${JSON.stringify(components, null, 2)}

Analysis questions:
- Do these components cover all the requirements from the original plan?
- Are there any missing capabilities?
- Do the interfaces make sense for integration?
- Are there any obvious design flaws or circular dependencies?

OUTPUT FORMAT (JSON only, no other text):
{
  "is_valid": true,
  "confidence": 0.85,
  "reasoning": "detailed analysis of coverage and design quality",
  "gaps": ["any missing capabilities"],
  "issues": ["any design problems identified"]
}
`;

  try {
    const response = await callClaude(prompt);
    const parsed = JSON.parse(response.trim());
    
    return {
      value: parsed.is_valid,
      confidence: parsed.confidence,
      reasoning: `${parsed.reasoning}\nGaps: ${parsed.gaps?.join(', ') || 'none'}\nIssues: ${parsed.issues?.join(', ') || 'none'}`
    };
  } catch (error) {
    return {
      value: false,
      confidence: 0.3,
      reasoning: `Error in architecture validation: ${error instanceof Error ? error.message : String(error)}`
    };
  }
}

// Save analysis results as tickets
function saveTickets(projectName: string, analysis: any) {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
  const filename = `${projectName.replace(/[^a-zA-Z0-9]/g, '_')}_${timestamp}.json`;
  const ticketPath = join(process.cwd(), 'tickets', filename);
  
  writeFileSync(ticketPath, JSON.stringify(analysis, null, 2));
  console.log(`💾 Tickets saved to: ${ticketPath}`);
  return ticketPath;
}

// Main workflow
async function analyzeProject(planDescription: string): Promise<{
  plan: string;
  missing_context: AnalysisResult<string[]>;
  should_decompose?: AnalysisResult<boolean>;
  components?: Component[];
  validation?: AnalysisResult<boolean>;
  ticketPath?: string;
}> {
  console.log(`🎯 Analyzing project: "${planDescription}"`);
  
  // Step 1: Extract missing context
  console.log('📝 Extracting missing context...');
  const missingContext = await extractMissingContext(planDescription);
  console.log(`Missing context (${missingContext.confidence}): ${missingContext.value.join(', ')}`);
  
  // Step 2: For demo, use minimal context (normally would gather from user)
  const context = {
    target_platform: 'web',
    complexity_level: 'moderate'
  };
  
  // Step 3: Check if decomposition needed
  console.log('🔍 Checking if decomposition is needed...');
  const shouldDecomp = await shouldDecompose(planDescription, context);
  console.log(`Should decompose: ${shouldDecomp.value} (${shouldDecomp.confidence})`);
  console.log(`Reasoning: ${shouldDecomp.reasoning}`);
  
  const result = {
    plan: planDescription,
    missing_context: missingContext,
    should_decompose: shouldDecomp
  };
  
  if (shouldDecomp.value) {
    // Step 4: Generate components
    console.log('🧩 Generating components...');
    const componentResult = await generateComponents(planDescription, context);
    console.log(`Generated ${componentResult.value.length} components (${componentResult.confidence}):`);
    componentResult.value.forEach(comp => {
      console.log(`  - ${comp.name}: ${comp.purpose}`);
    });
    
    // Step 5: Validate architecture
    console.log('✅ Validating architecture...');
    const validation = await validateArchitecture(planDescription, componentResult.value);
    console.log(`Architecture valid: ${validation.value} (${validation.confidence})`);
    console.log(`Validation: ${validation.reasoning}`);
    
    const finalResult = {
      ...result,
      components: componentResult.value,
      validation: validation
    };
    
    const ticketPath = saveTickets(planDescription, finalResult);
    return { ...finalResult, ticketPath };
  } else {
    console.log('✅ Simple enough - implement directly without decomposition');
    const ticketPath = saveTickets(planDescription, result);
    return { ...result, ticketPath };
  }
}

// Example usage
async function main() {
  try {
    console.log('=== Cognitive Analysis Demo ===\n');
    
    // Test with a simple project
    await analyzeProject('build a todo app with real-time sync');
    
    console.log('\n' + '='.repeat(50) + '\n');
    
    // Test with a more complex project
    await analyzeProject('create an enterprise-grade e-commerce platform with inventory management, payment processing, and analytics dashboard');
    
  } catch (error) {
    console.error('Error running analysis:', error);
  }
}

// Export for use as module
export {
  extractMissingContext,
  shouldDecompose,
  generateComponents,
  validateArchitecture,
  analyzeProject
};

// Run if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}