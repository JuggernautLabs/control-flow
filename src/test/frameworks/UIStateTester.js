/**
 * Comprehensive UI State Testing Framework
 * Catches real-world bugs through systematic state validation and interaction testing
 */

import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'

export class UIStateTester {
  constructor(component, props = {}) {
    this.component = component
    this.props = props
    this.wrapper = null
    this.stateSnapshots = []
    this.interactions = []
    this.errors = []
  }

  async setup() {
    this.wrapper = mount(this.component, {
      props: this.props,
      attachTo: document.body
    })
    await this.takeStateSnapshot('initial')
    return this
  }

  async takeStateSnapshot(label) {
    if (!this.wrapper) throw new Error('Setup not called')
    
    const snapshot = {
      label,
      timestamp: Date.now(),
      gameState: JSON.parse(JSON.stringify(this.wrapper.vm.gameState)),
      storyGraph: JSON.parse(JSON.stringify(this.wrapper.vm.storyGraph)),
      errors: [...this.errors],
      domState: {
        hasChoices: this.wrapper.findAll('.choice-btn').length,
        hasGenerateButton: this.wrapper.find('.generate-btn').exists(),
        hasError: this.wrapper.find('.error-bar').exists(),
        hasLoading: this.wrapper.find('.loading-bar').exists(),
        currentLocation: this.wrapper.find('.current-location').text(),
        logEntries: this.wrapper.findAll('.log-entry').length
      }
    }
    
    this.stateSnapshots.push(snapshot)
    return snapshot
  }

  async performInteraction(interaction) {
    if (!this.wrapper) throw new Error('Setup not called')
    
    const before = await this.takeStateSnapshot(`before_${interaction.name}`)
    
    try {
      await interaction.action(this.wrapper)
      await nextTick()
      await this.waitForStability()
    } catch (error) {
      this.errors.push({
        interaction: interaction.name,
        error: error.message,
        stack: error.stack,
        timestamp: Date.now()
      })
    }
    
    const after = await this.takeStateSnapshot(`after_${interaction.name}`)
    
    this.interactions.push({
      name: interaction.name,
      before,
      after,
      timestamp: Date.now()
    })
    
    return { before, after }
  }

  async waitForStability(maxWait = 3000) {
    // Wait for async operations to complete
    let previousState = null
    const startTime = Date.now()
    
    while (Date.now() - startTime < maxWait) {
      const currentState = {
        choices: this.wrapper.findAll('.choice-btn').length,
        loading: this.wrapper.find('.loading-bar').exists(),
        error: this.wrapper.find('.error-bar').exists(),
        gameStateHash: JSON.stringify(this.wrapper.vm.gameState),
        graphHash: JSON.stringify(this.wrapper.vm.storyGraph)
      }
      
      if (previousState && JSON.stringify(currentState) === JSON.stringify(previousState)) {
        break // State stabilized
      }
      
      previousState = currentState
      await new Promise(resolve => setTimeout(resolve, 100))
    }
  }

  validateGraphConsistency() {
    const issues = []
    const { nodes, edges } = this.wrapper.vm.storyGraph
    const nodeIds = new Set(nodes.map(n => n.id))
    
    // Check all edges reference existing nodes
    for (const edge of edges) {
      if (!nodeIds.has(edge.fromId)) {
        issues.push(`Edge ${edge.id} references missing fromId: ${edge.fromId}`)
      }
      if (!nodeIds.has(edge.toId)) {
        issues.push(`Edge ${edge.id} references missing toId: ${edge.toId}`)
      }
    }
    
    // Check current node exists
    const currentNodeId = this.wrapper.vm.gameState.currentNodeId
    if (!nodeIds.has(currentNodeId)) {
      issues.push(`Current node ID does not exist: ${currentNodeId}`)
    }
    
    // Check visited nodes exist
    for (const visitedId of this.wrapper.vm.gameState.visitedNodes) {
      if (!nodeIds.has(visitedId)) {
        issues.push(`Visited node does not exist: ${visitedId}`)
      }
    }
    
    return issues
  }

  validateChoiceIntegrity() {
    const issues = []
    const choices = this.wrapper.vm.currentChoices
    
    for (const choice of choices) {
      // Check required properties
      if (!choice.text) issues.push(`Choice ${choice.id} missing text`)
      if (!choice.icon) issues.push(`Choice ${choice.id} missing icon`)
      if (typeof choice.cost !== 'number') issues.push(`Choice ${choice.id} invalid cost`)
      if (typeof choice.experience !== 'number') issues.push(`Choice ${choice.id} invalid experience`)
      
      // Check if choice can be made
      const canMake = this.wrapper.vm.canMakeChoice(choice)
      const hasDisabledClass = this.wrapper.findAll('.choice-btn').some(btn => 
        btn.text().includes(choice.text) && btn.classes('disabled')
      )
      
      if (!canMake && !hasDisabledClass) {
        issues.push(`Choice "${choice.text}" should be disabled but isn't`)
      }
      if (canMake && hasDisabledClass) {
        issues.push(`Choice "${choice.text}" should be enabled but is disabled`)
      }
    }
    
    return issues
  }

  validateUIState() {
    const issues = []
    
    // Check for UI inconsistencies
    const hasChoices = this.wrapper.vm.currentChoices.length > 0
    const showsChoices = this.wrapper.find('.choices-grid').exists()
    const showsGenerate = this.wrapper.find('.generate-btn').exists()
    
    if (hasChoices && !showsChoices) {
      issues.push('Has choices in state but not displaying them')
    }
    if (!hasChoices && showsChoices) {
      issues.push('Displaying choices but none exist in state')
    }
    if (hasChoices && showsGenerate) {
      issues.push('Showing generate button when choices already exist')
    }
    if (!hasChoices && !showsGenerate && !this.wrapper.vm.gameState.isGameOver) {
      issues.push('Not showing generate button when no choices exist')
    }
    
    // Check loading/error states
    const isLoading = this.wrapper.vm.isLoading
    const showsLoading = this.wrapper.find('.loading-bar').exists()
    
    if (isLoading !== showsLoading) {
      issues.push(`Loading state mismatch: state=${isLoading}, UI=${showsLoading}`)
    }
    
    return issues
  }

  async runFullValidation() {
    const validation = {
      timestamp: Date.now(),
      graphConsistency: this.validateGraphConsistency(),
      choiceIntegrity: this.validateChoiceIntegrity(),
      uiState: this.validateUIState(),
      errors: [...this.errors]
    }
    
    validation.isValid = (
      validation.graphConsistency.length === 0 &&
      validation.choiceIntegrity.length === 0 &&
      validation.uiState.length === 0 &&
      validation.errors.length === 0
    )
    
    return validation
  }

  async testCommonUserFlows() {
    const flows = []
    
    // Flow 1: Generate choices and select one
    try {
      await this.performInteraction({
        name: 'generate_initial_choices',
        action: async (wrapper) => {
          const generateBtn = wrapper.find('.generate-btn')
          if (generateBtn.exists()) {
            await generateBtn.trigger('click')
          }
        }
      })
      
      const validation1 = await this.runFullValidation()
      flows.push({ name: 'after_generation', validation: validation1 })
      
      if (this.wrapper.vm.currentChoices.length > 0) {
        await this.performInteraction({
          name: 'select_first_choice',
          action: async (wrapper) => {
            const firstChoice = wrapper.find('.choice-btn')
            if (firstChoice.exists()) {
              await firstChoice.trigger('click')
            }
          }
        })
        
        const validation2 = await this.runFullValidation()
        flows.push({ name: 'after_choice_selection', validation: validation2 })
      }
    } catch (error) {
      flows.push({ 
        name: 'flow_error', 
        error: error.message,
        validation: await this.runFullValidation()
      })
    }
    
    return flows
  }

  async simulateGameplaySession(steps = 5) {
    const session = {
      steps: [],
      initialValidation: await this.runFullValidation()
    }
    
    for (let i = 0; i < steps; i++) {
      try {
        // Generate choices if none exist
        if (this.wrapper.vm.currentChoices.length === 0) {
          await this.performInteraction({
            name: `step_${i}_generate`,
            action: async (wrapper) => {
              const generateBtn = wrapper.find('.generate-btn')
              if (generateBtn.exists()) {
                await generateBtn.trigger('click')
              }
            }
          })
        }
        
        // Make a random choice if available
        if (this.wrapper.vm.currentChoices.length > 0) {
          const randomIndex = Math.floor(Math.random() * this.wrapper.vm.currentChoices.length)
          await this.performInteraction({
            name: `step_${i}_choice_${randomIndex}`,
            action: async (wrapper) => {
              const choiceBtn = wrapper.findAll('.choice-btn')[randomIndex]
              if (choiceBtn && !choiceBtn.classes('disabled')) {
                await choiceBtn.trigger('click')
              }
            }
          })
        }
        
        const stepValidation = await this.runFullValidation()
        session.steps.push({
          step: i,
          validation: stepValidation,
          gameState: JSON.parse(JSON.stringify(this.wrapper.vm.gameState)),
          graphState: {
            nodeCount: this.wrapper.vm.storyGraph.nodes.length,
            edgeCount: this.wrapper.vm.storyGraph.edges.length
          }
        })
        
        // Stop if game ended or validation failed
        if (this.wrapper.vm.gameState.isGameOver || !stepValidation.isValid) {
          break
        }
        
      } catch (error) {
        session.steps.push({
          step: i,
          error: error.message,
          validation: await this.runFullValidation()
        })
        break
      }
    }
    
    return session
  }

  generateReport() {
    return {
      summary: {
        totalSnapshots: this.stateSnapshots.length,
        totalInteractions: this.interactions.length,
        totalErrors: this.errors.length,
        testDuration: this.stateSnapshots.length > 0 ? 
          Date.now() - this.stateSnapshots[0].timestamp : 0
      },
      snapshots: this.stateSnapshots,
      interactions: this.interactions,
      errors: this.errors,
      finalValidation: this.runFullValidation()
    }
  }

  cleanup() {
    if (this.wrapper) {
      this.wrapper.unmount()
    }
  }
}

// Convenience function for quick testing
export async function testGameFlow(component, props, aiService) {
  const tester = new UIStateTester(component, { aiService, ...props })
  
  try {
    await tester.setup()
    const flows = await tester.testCommonUserFlows()
    const session = await tester.simulateGameplaySession(3)
    const report = await tester.generateReport()
    
    return {
      flows,
      session,
      report,
      isValid: flows.every(f => !f.validation || f.validation.isValid) &&
                session.steps.every(s => !s.validation || s.validation.isValid)
    }
  } finally {
    tester.cleanup()
  }
}