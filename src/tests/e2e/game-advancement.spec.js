import { test, expect } from '@playwright/test';

test.describe('Game Engine Advancement via UI', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.click('text=Adventure Game');
    
    // Enable debug mode for better testing
    await page.click('.debug-toggle-btn');
    await expect(page.locator('.debug-panel')).toBeVisible();
  });

  test('should advance through complete game cycle with state validation', async ({ page }) => {
    // Verify initial state
    await expect(page.locator('.debug-panel')).toContainText('Phase: waiting_for_choices');
    await expect(page.locator('.debug-panel')).toContainText('Can Advance: ✅');
    await expect(page.locator('.debug-panel')).toContainText('Validation: ✅');
    
    // Step 1: Generate Choices
    await page.click('text=Generate Adventure Choices');
    
    // Verify generation phase
    await expect(page.locator('.debug-panel')).toContainText('Phase: generating_choices');
    await expect(page.locator('.generating-phase')).toBeVisible();
    
    // Wait for generation to complete
    await expect(page.locator('.debug-panel')).toContainText('Phase: choosing', { timeout: 10000 });
    await expect(page.locator('.choosing-phase')).toBeVisible();
    
    // Verify choices are available
    await expect(page.locator('.choice-btn')).toHaveCount(3);
    await expect(page.locator('.debug-panel')).toContainText('Validation: ✅');
    
    // Step 2: Make Choice
    await page.click('.choice-btn:first-child');
    
    // Verify advancing phase
    await expect(page.locator('.debug-panel')).toContainText('Phase: advancing');
    await expect(page.locator('.advancing-phase')).toBeVisible();
    
    // Wait for advancement to complete
    await expect(page.locator('.debug-panel')).toContainText('Phase: waiting_for_choices', { timeout: 15000 });
    await expect(page.locator('.waiting-phase')).toBeVisible();
    
    // Verify state after advancement
    await expect(page.locator('.debug-panel')).toContainText('Can Advance: ✅');
    await expect(page.locator('.debug-panel')).toContainText('Validation: ✅');
    
    // Verify current node changed
    await expect(page.locator('.debug-panel')).not.toContainText('Current Node: start');
    
    // Verify stats updated
    const experience = await page.locator('.stat-value').nth(1).textContent();
    expect(parseInt(experience)).toBeGreaterThan(0);
    
    // Verify can continue advancing
    await page.click('text=Generate Adventure Choices');
    await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('.choice-btn')).toHaveCount(3);
  });

  test('should handle advancement failures gracefully', async ({ page }) => {
    // Generate initial choices
    await page.click('text=Generate Adventure Choices');
    await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
    
    // Break the AI service to cause choice failure
    await page.evaluate(() => {
      const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
      if (app && app.ctx.engine) {
        app.ctx.engine.aiService.generateStoryNode = async () => {
          throw new Error('Node generation failed');
        };
      }
    });
    
    // Try to make a choice (should fail)
    await page.click('.choice-btn:first-child');
    
    // Should show error
    await expect(page.locator('.error-display')).toBeVisible({ timeout: 5000 });
    await expect(page.locator('.error-message')).toContainText('Node generation failed');
    
    // Should remain in choosing phase
    await expect(page.locator('.debug-panel')).toContainText('Phase: choosing');
    await expect(page.locator('.choosing-phase')).toBeVisible();
    
    // Should still be able to try again after fixing
    await page.click('.error-close');
    await page.evaluate(() => {
      const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
      if (app && app.ctx.engine) {
        // Restore working AI service
        app.ctx.engine.aiService.generateStoryNode = async (choice, fromNode, context) => {
          return {
            location: "Test Location",
            situation: "<p>Test situation</p>",
            confidence: 0.9,
            reasoning: 'Test node generation'
          };
        };
      }
    });
    
    // Now choice should work
    await page.click('.choice-btn:first-child');
    await expect(page.locator('.waiting-phase')).toBeVisible({ timeout: 15000 });
  });

  test('should validate state consistency throughout advancement', async ({ page }) => {
    const maxSteps = 5;
    
    for (let step = 0; step < maxSteps; step++) {
      console.log(`Testing advancement step ${step + 1}`);
      
      // Verify state is valid before each step
      await expect(page.locator('.debug-panel')).toContainText('Validation: ✅');
      
      // If waiting for choices, generate them
      if (await page.locator('.waiting-phase').isVisible()) {
        await page.click('text=Generate Adventure Choices');
        await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
      }
      
      // Verify state is still valid after generation
      await expect(page.locator('.debug-panel')).toContainText('Validation: ✅');
      
      // Make a choice
      const affordableChoices = page.locator('.choice-btn:not(.disabled)');
      const choiceCount = await affordableChoices.count();
      
      if (choiceCount > 0) {
        await affordableChoices.first().click();
        await expect(page.locator('.waiting-phase')).toBeVisible({ timeout: 15000 });
        
        // Verify state is valid after advancement
        await expect(page.locator('.debug-panel')).toContainText('Validation: ✅');
        await expect(page.locator('.debug-panel')).toContainText('Can Advance: ✅');
      } else {
        console.log('No affordable choices available, ending test');
        break;
      }
    }
  });

  test('should prevent invalid state transitions', async ({ page }) => {
    // Try to manipulate state directly to invalid values
    await page.evaluate(() => {
      const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
      if (app && app.ctx.engine) {
        // Set invalid current node
        app.ctx.engine.state.currentNodeId = 'nonexistent_node';
        app.ctx.updateUIState();
      }
    });
    
    // Should show validation error
    await expect(page.locator('.debug-panel')).toContainText('Validation: ❌');
    await expect(page.locator('.debug-panel')).toContainText('Can Advance: ❌');
    
    // Should not be able to generate choices in invalid state
    await page.click('text=Generate Adventure Choices');
    await expect(page.locator('.error-display')).toBeVisible({ timeout: 5000 });
    
    // Reset to valid state
    await page.evaluate(() => {
      const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
      if (app && app.ctx.engine) {
        app.ctx.engine.state.currentNodeId = 'start';
        app.ctx.updateUIState();
      }
    });
    
    await expect(page.locator('.debug-panel')).toContainText('Validation: ✅');
    await expect(page.locator('.debug-panel')).toContainText('Can Advance: ✅');
  });

  test('should track complete user journey through debug data', async ({ page }) => {
    // Export initial debug data
    const downloadPromise = page.waitForEvent('download');
    await page.click('text=Export Debug Data');
    const download = await downloadPromise;
    
    // Make several moves
    for (let i = 0; i < 3; i++) {
      await page.click('text=Generate Adventure Choices');
      await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
      await page.click('.choice-btn:first-child');
      await expect(page.locator('.waiting-phase')).toBeVisible({ timeout: 15000 });
    }
    
    // Export final debug data
    const finalDownloadPromise = page.waitForEvent('download');
    await page.click('text=Export Debug Data');
    const finalDownload = await finalDownloadPromise;
    
    // Verify downloads occurred
    expect(download.suggestedFilename()).toMatch(/adventure-debug-\d+\.json/);
    expect(finalDownload.suggestedFilename()).toMatch(/adventure-debug-\d+\.json/);
  });

  test('should show detailed advancement failure reasons', async ({ page }) => {
    // Set up a scenario where advancement should fail
    await page.evaluate(() => {
      const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
      if (app && app.ctx.engine) {
        // Set game over to block advancement
        app.ctx.engine.state.isGameOver = true;
        app.ctx.updateUIState();
      }
    });
    
    // Should show cannot advance
    await expect(page.locator('.debug-panel')).toContainText('Can Advance: ❌');
    
    // Generate choices button should be disabled
    await expect(page.locator('text=Generate Adventure Choices')).toBeDisabled();
    
    // Should show why advancement is blocked
    await expect(page.locator('.advancement-blocked')).toBeVisible();
    await expect(page.locator('.advancement-blocked')).toContainText('Game is over');
  });

  test('should handle concurrent operations correctly', async ({ page }) => {
    // Try to start generation while already generating
    await page.click('text=Generate Adventure Choices');
    
    // Immediately try to click again while generation is in progress
    await expect(page.locator('.generating-phase')).toBeVisible();
    
    // Button should be disabled during generation
    await expect(page.locator('text=Generate Adventure Choices')).toBeDisabled();
    
    // Wait for completion
    await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
    
    // Now try to make multiple choice clicks
    await page.click('.choice-btn:first-child');
    await expect(page.locator('.advancing-phase')).toBeVisible();
    
    // Other choice buttons should be disabled during advancement
    await expect(page.locator('.choice-btn').nth(1)).toBeDisabled();
    
    await expect(page.locator('.waiting-phase')).toBeVisible({ timeout: 15000 });
  });

  test('should maintain UI consistency during state changes', async ({ page }) => {
    // Generate choices
    await page.click('text=Generate Adventure Choices');
    await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
    
    // Verify UI elements are consistent with phase
    await expect(page.locator('.waiting-phase')).not.toBeVisible();
    await expect(page.locator('.advancing-phase')).not.toBeVisible();
    await expect(page.locator('.generating-phase')).not.toBeVisible();
    await expect(page.locator('.choosing-phase')).toBeVisible();
    
    // Make choice
    await page.click('.choice-btn:first-child');
    await expect(page.locator('.advancing-phase')).toBeVisible();
    
    // Verify UI elements are consistent with new phase
    await expect(page.locator('.waiting-phase')).not.toBeVisible();
    await expect(page.locator('.choosing-phase')).not.toBeVisible();
    await expect(page.locator('.generating-phase')).not.toBeVisible();
    await expect(page.locator('.advancing-phase')).toBeVisible();
    
    // Wait for completion
    await expect(page.locator('.waiting-phase')).toBeVisible({ timeout: 15000 });
    
    // Verify final UI state
    await expect(page.locator('.choosing-phase')).not.toBeVisible();
    await expect(page.locator('.advancing-phase')).not.toBeVisible();
    await expect(page.locator('.generating-phase')).not.toBeVisible();
    await expect(page.locator('.waiting-phase')).toBeVisible();
  });
});