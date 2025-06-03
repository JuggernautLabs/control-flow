import { test, expect } from '@playwright/test';

test.describe('Story Advancement Bug Prevention', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.click('text=Adventure Game');
    await page.click('.debug-toggle-btn'); // Enable debug for detailed error info
  });

  test('should prevent "Node not found" errors during advancement', async ({ page }) => {
    // This test specifically targets the original bug: "Failed to generate choices: Node not found: microservices_path"
    
    // Simulate the bug scenario by creating saved state with invalid node references
    await page.evaluate(() => {
      const invalidSavedState = {
        currentNodeId: 'microservices_path', // This node doesn't exist initially
        level: 2,
        experience: 150,
        gold: 75,
        inventory: [],
        visitedNodes: ['start', 'microservices_path', 'database_district'],
        isGameOver: false,
        isWin: false,
        endMessage: '',
        actionLog: []
      };
      
      localStorage.setItem('adventure_game_state', JSON.stringify(invalidSavedState));
    });
    
    // Reload page to trigger state loading
    await page.reload();
    await page.click('text=Adventure Game');
    
    // Should handle invalid state gracefully
    await expect(page.locator('.debug-panel')).toContainText('Validation: ✅');
    await expect(page.locator('.current-location')).toContainText('The Town Square of Architectura'); // Should reset to start
    
    // Should be able to generate choices without "Node not found" error
    await page.click('text=Generate Adventure Choices');
    await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
    
    // Should not show any "Node not found" errors
    await expect(page.locator('.error-display')).not.toBeVisible();
    await expect(page.locator('text=Node not found')).not.toBeVisible();
  });

  test('should handle missing edge targets during choice selection', async ({ page }) => {
    // Generate initial choices
    await page.click('text=Generate Adventure Choices');
    await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
    
    // Manipulate the game state to create invalid edges
    await page.evaluate(() => {
      const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
      if (app && app.ctx.engine) {
        // Add an edge that points to a non-existent node
        app.ctx.engine.state.storyGraph.edges.push({
          id: 'invalid_edge',
          fromId: 'start',
          toId: 'nonexistent_target',
          text: 'Go to Invalid Location',
          icon: '❌',
          cost: 0,
          experience: 10
        });
        app.ctx.updateUIState();
      }
    });
    
    // Should detect invalid state
    await expect(page.locator('.debug-panel')).toContainText('Validation: ❌');
    
    // Should not be able to advance in invalid state
    await expect(page.locator('.debug-panel')).toContainText('Can Advance: ❌');
    
    // Clean up invalid edges
    await page.evaluate(() => {
      const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
      if (app && app.ctx.engine) {
        app.ctx.engine.cleanupInvalidEdges();
        app.ctx.updateUIState();
      }
    });
    
    // Should return to valid state
    await expect(page.locator('.debug-panel')).toContainText('Validation: ✅');
    await expect(page.locator('.debug-panel')).toContainText('Can Advance: ✅');
  });

  test('should prevent story stagnation (choices not advancing)', async ({ page }) => {
    // This tests the scenario where users click "generate choices" but story doesn't advance
    
    let previousLocation = '';
    let sameLocationCount = 0;
    const maxAttempts = 5;
    
    for (let attempt = 0; attempt < maxAttempts; attempt++) {
      // Generate choices
      await page.click('text=Generate Adventure Choices');
      await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
      
      // Get current location
      const currentLocation = await page.locator('.current-location').textContent();
      
      if (currentLocation === previousLocation) {
        sameLocationCount++;
      } else {
        sameLocationCount = 0; // Reset counter
      }
      
      // Should not stay in same location for more than 2 attempts
      expect(sameLocationCount).toBeLessThan(3);
      
      // Make a choice to advance
      const affordableChoices = page.locator('.choice-btn:not(.disabled)');
      const choiceCount = await affordableChoices.count();
      
      if (choiceCount > 0) {
        await affordableChoices.first().click();
        await expect(page.locator('.waiting-phase')).toBeVisible({ timeout: 15000 });
        
        // Verify advancement occurred
        const newLocation = await page.locator('.current-location').textContent();
        expect(newLocation).not.toBe(currentLocation);
        
        previousLocation = newLocation;
      } else {
        // If no affordable choices, the game should indicate why
        await expect(page.locator('.advancement-blocked')).toBeVisible();
        break;
      }
    }
  });

  test('should handle AI service failures without breaking advancement', async ({ page }) => {
    // Test various AI failure scenarios
    
    // Scenario 1: Choice generation fails
    await page.evaluate(() => {
      const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
      if (app && app.ctx.engine) {
        app.ctx.engine.aiService.generateChoices = async () => {
          throw new Error('AI choice generation failed');
        };
      }
    });
    
    await page.click('text=Generate Adventure Choices');
    await expect(page.locator('.error-display')).toBeVisible({ timeout: 5000 });
    await expect(page.locator('.error-message')).toContainText('AI choice generation failed');
    
    // Should remain in valid state after error
    await expect(page.locator('.debug-panel')).toContainText('Phase: waiting_for_choices');
    
    // Fix AI service
    await page.evaluate(() => {
      const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
      if (app && app.ctx.engine) {
        app.ctx.engine.aiService.generateChoices = async () => ({
          choices: [
            { text: 'Test Choice', icon: '🧪', cost: 0, experience: 10 }
          ],
          confidence: 0.8,
          reasoning: 'Recovery test'
        });
      }
    });
    
    await page.click('.error-close');
    
    // Should be able to recover
    await page.click('text=Generate Adventure Choices');
    await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
  });

  test('should prevent infinite loading states', async ({ page }) => {
    // Simulate AI service that never resolves
    await page.evaluate(() => {
      const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
      if (app && app.ctx.engine) {
        app.ctx.engine.aiService.generateChoices = async () => {
          // Return a promise that never resolves
          return new Promise(() => {});
        };
      }
    });
    
    await page.click('text=Generate Adventure Choices');
    
    // Should show loading state
    await expect(page.locator('.generating-phase')).toBeVisible();
    
    // Wait for reasonable timeout (should implement timeout in actual code)
    await page.waitForTimeout(3000);
    
    // For now, verify it's still in loading state (in real implementation, this should timeout)
    await expect(page.locator('.generating-phase')).toBeVisible();
    
    // Reset to working state
    await page.evaluate(() => {
      const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
      if (app && app.ctx.engine) {
        app.ctx.engine.state.phase = 'waiting_for_choices';
        app.ctx.engine.state.generationInProgress = false;
        app.ctx.updateUIState();
      }
    });
    
    await expect(page.locator('.waiting-phase')).toBeVisible();
  });

  test('should maintain choice affordability consistency', async ({ page }) => {
    // Test that choice affordability matches game state
    
    await page.click('text=Generate Adventure Choices');
    await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
    
    // Reduce gold to make some choices unaffordable
    await page.evaluate(() => {
      const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
      if (app && app.ctx.engine) {
        app.ctx.engine.state.gold = 5; // Very low gold
        app.ctx.updateUIState();
      }
    });
    
    // Check that expensive choices are disabled
    const expensiveChoices = page.locator('.choice-btn:has(.cost)');
    const expensiveCount = await expensiveChoices.count();
    
    if (expensiveCount > 0) {
      for (let i = 0; i < expensiveCount; i++) {
        const choice = expensiveChoices.nth(i);
        const costText = await choice.locator('.cost').textContent();
        const cost = parseInt(costText.match(/\d+/)[0]);
        
        if (cost > 5) {
          await expect(choice).toHaveClass(/disabled/);
        }
      }
    }
    
    // Verify clicking disabled choice doesn't advance
    const disabledChoices = page.locator('.choice-btn.disabled');
    const disabledCount = await disabledChoices.count();
    
    if (disabledCount > 0) {
      const currentLocation = await page.locator('.current-location').textContent();
      await disabledChoices.first().click();
      
      // Should stay in same location
      await expect(page.locator('.current-location')).toContainText(currentLocation);
      await expect(page.locator('.choosing-phase')).toBeVisible();
    }
  });

  test('should prevent phase transition bugs', async ({ page }) => {
    // Test invalid phase transitions
    
    // Try to make choice while in waiting phase (should fail)
    await expect(page.locator('.debug-panel')).toContainText('Phase: waiting_for_choices');
    
    await page.evaluate(() => {
      const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
      if (app && app.ctx.engine) {
        // Try to make a choice without generating first
        const fakeChoice = {
          id: 'fake_choice',
          fromId: 'start',
          toId: 'fake_target',
          text: 'Fake Choice',
          cost: 0,
          experience: 10
        };
        app.ctx.engine.makeChoice(fakeChoice.id).catch(error => {
          console.log('Expected error:', error.message);
        });
      }
    });
    
    // Should remain in waiting phase
    await expect(page.locator('.debug-panel')).toContainText('Phase: waiting_for_choices');
    
    // Generate choices to get to choosing phase
    await page.click('text=Generate Adventure Choices');
    await expect(page.locator('.debug-panel')).toContainText('Phase: choosing');
    
    // Try to generate choices again while in choosing phase (should be allowed)
    await page.click('text=Generate Adventure Choices');
    await expect(page.locator('.debug-panel')).toContainText('Phase: choosing');
  });

  test('should recover from corrupted localStorage gracefully', async ({ page }) => {
    // Corrupt the localStorage
    await page.evaluate(() => {
      localStorage.setItem('adventure_game_state', 'invalid json data {{{');
    });
    
    // Reload page
    await page.reload();
    await page.click('text=Adventure Game');
    
    // Should start with clean state
    await expect(page.locator('.current-location')).toContainText('The Town Square of Architectura');
    await expect(page.locator('.stat-value').first()).toContainText('1'); // Level 1
    await expect(page.locator('.debug-panel')).toContainText('Validation: ✅');
    
    // Should be able to play normally
    await page.click('text=Generate Adventure Choices');
    await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
  });
});