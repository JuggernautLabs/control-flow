import { test, expect } from '@playwright/test';
import {
  setupAdventureGame,
  waitForGamePhase,
  generateChoices,
  makeChoice,
  completeGameCycle,
  getGameStats,
  verifyValidGameState,
  setEngineState,
  breakAIService,
  restoreAIService,
  setInvalidLocalStorage,
  clearLocalStorage,
  expectError,
  exportDebugData,
  simulateUserJourney
} from './test-helpers.js';

test.describe('Comprehensive UI Testing Framework', () => {
  test.beforeEach(async ({ page }) => {
    await clearLocalStorage(page);
    await setupAdventureGame(page);
  });

  test('should demonstrate complete user interaction testing', async ({ page }) => {
    // Test the full user journey with comprehensive validation
    
    // 1. Verify initial state
    await verifyValidGameState(page);
    const initialStats = await getGameStats(page);
    expect(initialStats.level).toBe(1);
    expect(initialStats.experience).toBe(0);
    expect(initialStats.gold).toBe(50);
    
    // 2. Test choice generation
    const choices = await generateChoices(page);
    await expect(choices).toHaveCount(3);
    
    // 3. Verify choice properties
    for (let i = 0; i < 3; i++) {
      const choice = choices.nth(i);
      await expect(choice.locator('.choice-icon')).toBeVisible();
      await expect(choice.locator('.choice-text')).toBeVisible();
      await expect(choice.locator('.choice-stats')).toBeVisible();
    }
    
    // 4. Test choice selection and advancement
    await makeChoice(page, 0);
    
    // 5. Verify advancement occurred
    const statsAfterChoice = await getGameStats(page);
    expect(statsAfterChoice.experience).toBeGreaterThan(initialStats.experience);
    
    // 6. Verify location changed
    const location = await page.locator('.current-location').textContent();
    expect(location).not.toContain('Town Square');
    
    // 7. Verify state remains valid
    await verifyValidGameState(page);
    
    // 8. Test continued advancement
    await completeGameCycle(page, 1);
    await verifyValidGameState(page);
    
    const finalStats = await getGameStats(page);
    expect(finalStats.experience).toBeGreaterThan(statsAfterChoice.experience);
  });

  test('should test error scenarios and recovery', async ({ page }) => {
    // Test comprehensive error handling
    
    // 1. Test AI generation failure
    await breakAIService(page, 'generateChoices', 'Test generation failure');
    await page.click('text=Generate Adventure Choices');
    await expectError(page, 'Test generation failure');
    
    // 2. Verify state remains valid after error
    await page.click('.error-close');
    await verifyValidGameState(page);
    
    // 3. Test recovery
    await restoreAIService(page);
    await generateChoices(page);
    await expect(page.locator('.choice-btn')).toHaveCount(3);
    
    // 4. Test choice making failure
    await breakAIService(page, 'generateStoryNode', 'Test node generation failure');
    await page.click('.choice-btn:first-child');
    await expectError(page, 'Test node generation failure');
    
    // 5. Verify recovery from choice failure
    await page.click('.error-close');
    await restoreAIService(page);
    await page.click('.choice-btn:first-child');
    await waitForGamePhase(page, 'waiting_for_choices', 15000);
  });

  test('should test state persistence and corruption handling', async ({ page }) => {
    // Test various persistence scenarios
    
    // 1. Make progress and verify persistence
    await completeGameCycle(page);
    const statsBeforeReload = await getGameStats(page);
    const locationBeforeReload = await page.locator('.current-location').textContent();
    
    // 2. Reload and verify state is restored
    await page.reload();
    await setupAdventureGame(page);
    
    const statsAfterReload = await getGameStats(page);
    const locationAfterReload = await page.locator('.current-location').textContent();
    
    expect(statsAfterReload.experience).toBe(statsBeforeReload.experience);
    expect(locationAfterReload).toBe(locationBeforeReload);
    
    // 3. Test invalid state handling
    await setInvalidLocalStorage(page, {
      currentNodeId: 'nonexistent_node',
      level: 5,
      experience: 500,
      gold: 100,
      visitedNodes: ['start', 'nonexistent_node'],
      inventory: []
    });
    
    await page.reload();
    await setupAdventureGame(page);
    
    // Should reset to valid state
    await expect(page.locator('.current-location')).toContainText('Town Square');
    await verifyValidGameState(page);
    
    // 4. Test corrupted JSON
    await setInvalidLocalStorage(page, 'invalid json {{{');
    await page.reload();
    await setupAdventureGame(page);
    
    // Should start fresh
    const freshStats = await getGameStats(page);
    expect(freshStats.level).toBe(1);
    expect(freshStats.experience).toBe(0);
  });

  test('should test advanced user journeys', async ({ page }) => {
    // Test extended gameplay scenarios
    
    // 1. Simulate extended user session
    const journey = await simulateUserJourney(page, 5);
    
    // 2. Verify progression
    expect(journey.length).toBe(5);
    
    let totalExperience = 0;
    let totalGoldSpent = 0;
    
    for (const step of journey) {
      expect(step.experienceGained).toBeGreaterThanOrEqual(0);
      expect(step.location).toBeDefined();
      totalExperience += step.experienceGained;
      totalGoldSpent += step.goldSpent;
    }
    
    expect(totalExperience).toBeGreaterThan(30); // Meaningful progress
    
    // 3. Test debug data export
    const debugData = await exportDebugData(page);
    expect(debugData.suggestedFilename()).toMatch(/adventure-debug-\d+\.json/);
    
    // 4. Verify final state is valid
    await verifyValidGameState(page);
  });

  test('should test edge cases and boundary conditions', async ({ page }) => {
    // Test edge cases that could break the game
    
    // 1. Test with very low resources
    await setEngineState(page, { gold: 1, experience: 0 });
    
    await generateChoices(page);
    
    // Should have some affordable choices
    const affordableChoices = page.locator('.choice-btn:not(.disabled)');
    await expect(affordableChoices).toHaveCountGreaterThan(0);
    
    // 2. Test with invalid phase manipulation
    await setEngineState(page, { phase: 'invalid_phase' });
    
    // Should show validation error
    await expect(page.locator('.debug-panel')).toContainText('Validation: ❌');
    await expect(page.locator('.debug-panel')).toContainText('Can Advance: ❌');
    
    // 3. Test graph corruption
    await setEngineState(page, {
      phase: 'waiting_for_choices',
      storyGraph: {
        nodes: [{ id: 'start', location: 'Start', situation: '<p>Start</p>' }],
        edges: [
          { id: 'invalid', fromId: 'start', toId: 'missing', text: 'Invalid', cost: 0, experience: 10 }
        ]
      }
    });
    
    // Should detect invalid edges
    await expect(page.locator('.debug-panel')).toContainText('Validation: ❌');
    
    // 4. Test cleanup and recovery
    await page.evaluate(() => {
      const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
      if (app && app.ctx.engine) {
        app.ctx.engine.cleanupInvalidEdges();
        app.ctx.engine.state.phase = 'waiting_for_choices';
        app.ctx.updateUIState();
      }
    });
    
    await verifyValidGameState(page);
  });

  test('should test UI responsiveness and performance', async ({ page }) => {
    // Test UI performance under various conditions
    
    // 1. Test rapid interactions
    for (let i = 0; i < 3; i++) {
      const startTime = Date.now();
      
      await generateChoices(page);
      await makeChoice(page, 0);
      
      const endTime = Date.now();
      const duration = endTime - startTime;
      
      // Should complete within reasonable time
      expect(duration).toBeLessThan(20000); // 20 seconds max
    }
    
    // 2. Test UI consistency during state changes
    await generateChoices(page);
    
    // All phase indicators should be mutually exclusive
    await expect(page.locator('.waiting-phase')).not.toBeVisible();
    await expect(page.locator('.generating-phase')).not.toBeVisible();
    await expect(page.locator('.advancing-phase')).not.toBeVisible();
    await expect(page.locator('.choosing-phase')).toBeVisible();
    
    // 3. Test debug panel accuracy
    await expect(page.locator('.debug-panel')).toContainText('Phase: choosing');
    await expect(page.locator('.debug-panel')).toContainText('Can Advance: ✅');
    
    // 4. Test graph updates
    const initialGraphStats = await page.locator('.graph-stats').textContent();
    
    await makeChoice(page, 0);
    
    const updatedGraphStats = await page.locator('.graph-stats').textContent();
    expect(updatedGraphStats).not.toBe(initialGraphStats);
  });

  test('should validate all UI elements work correctly', async ({ page }) => {
    // Comprehensive UI element testing
    
    // 1. Test all buttons are functional
    await expect(page.locator('text=Generate Adventure Choices')).toBeEnabled();
    await page.click('text=Generate Adventure Choices');
    
    await waitForGamePhase(page, 'choosing');
    
    // 2. Test choice buttons
    const choices = page.locator('.choice-btn');
    for (let i = 0; i < await choices.count(); i++) {
      const choice = choices.nth(i);
      await expect(choice).toBeVisible();
      
      if (!(await choice.getAttribute('class')).includes('disabled')) {
        await expect(choice).toBeEnabled();
      }
    }
    
    // 3. Test debug controls
    await expect(page.locator('text=Export Debug Data')).toBeEnabled();
    await expect(page.locator('text=Hide Debug')).toBeEnabled();
    
    // 4. Test graph controls (if visible)
    if (await page.locator('.graph-container').isVisible()) {
      await expect(page.locator('.graph-container')).toBeVisible();
    }
    
    // 5. Test inventory display
    await expect(page.locator('.inventory-section')).toBeVisible();
    await expect(page.locator('.empty-inventory')).toBeVisible();
    
    // 6. Test action log
    await expect(page.locator('.action-log')).toBeVisible();
    await expect(page.locator('.log-entries')).toBeVisible();
    
    // 7. Test stats display
    await expect(page.locator('.game-stats')).toBeVisible();
    
    const statValues = page.locator('.stat-value');
    await expect(statValues).toHaveCount(3);
    
    for (let i = 0; i < 3; i++) {
      await expect(statValues.nth(i)).toBeVisible();
      await expect(statValues.nth(i)).toContainText(/\d+/);
    }
  });
});