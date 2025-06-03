import { test, expect } from '@playwright/test';

test.describe('Adventure Game UI Interactions', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Navigate to the adventure game
    await page.click('text=Adventure Game');
  });

  test('should display initial game state correctly', async ({ page }) => {
    // Check game header
    await expect(page.locator('h3')).toContainText('The Software Architecture Quest');
    
    // Check initial stats
    await expect(page.locator('.stat-value').first()).toContainText('1'); // Level
    await expect(page.locator('.stat-value').nth(1)).toContainText('0'); // Experience  
    await expect(page.locator('.stat-value').nth(2)).toContainText('50💰'); // Gold

    // Check initial location
    await expect(page.locator('.current-location')).toContainText('The Town Square of Architectura');
    
    // Check initial phase - should be waiting for choices
    await expect(page.locator('.waiting-phase')).toBeVisible();
    await expect(page.locator('text=Generate Adventure Choices')).toBeVisible();
  });

  test('should generate choices when button is clicked', async ({ page }) => {
    // Click generate choices button
    await page.click('text=Generate Adventure Choices');
    
    // Should show generating phase
    await expect(page.locator('.generating-phase')).toBeVisible();
    await expect(page.locator('text=Generating adventure choices')).toBeVisible();
    
    // Wait for generation to complete (with timeout)
    await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
    
    // Should now show choices
    await expect(page.locator('.choices-grid')).toBeVisible();
    await expect(page.locator('.choice-btn')).toHaveCount(3); // Should have 3 choices
    
    // Verify choice structure
    const firstChoice = page.locator('.choice-btn').first();
    await expect(firstChoice.locator('.choice-icon')).toBeVisible();
    await expect(firstChoice.locator('.choice-text')).toBeVisible();
    await expect(firstChoice.locator('.choice-stats')).toBeVisible();
  });

  test('should advance story when choice is selected', async ({ page }) => {
    // Generate choices first
    await page.click('text=Generate Adventure Choices');
    await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
    
    // Get initial location
    const initialLocation = await page.locator('.current-location').textContent();
    
    // Click the first choice
    await page.click('.choice-btn:first-child');
    
    // Should show advancing phase
    await expect(page.locator('.advancing-phase')).toBeVisible();
    await expect(page.locator('text=Advancing to new location')).toBeVisible();
    
    // Wait for advancement to complete
    await expect(page.locator('.waiting-phase')).toBeVisible({ timeout: 15000 });
    
    // Location should have changed
    const newLocation = await page.locator('.current-location').textContent();
    expect(newLocation).not.toBe(initialLocation);
    
    // Experience should have increased
    const experience = await page.locator('.stat-value').nth(1).textContent();
    expect(parseInt(experience)).toBeGreaterThan(0);
  });

  test('should show choice requirements and disable unaffordable choices', async ({ page }) => {
    // Generate choices
    await page.click('text=Generate Adventure Choices');
    await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
    
    // Look for choices with costs
    const choicesWithCost = page.locator('.choice-btn:has(.cost)');
    const costlyChoiceCount = await choicesWithCost.count();
    
    if (costlyChoiceCount > 0) {
      // Check that expensive choices show cost
      const expensiveChoice = choicesWithCost.first();
      await expect(expensiveChoice.locator('.cost')).toBeVisible();
      
      // Reduce gold to 0 by manipulating game state through debug panel
      await page.click('.debug-toggle-btn');
      await expect(page.locator('.debug-panel')).toBeVisible();
      
      // Set gold to 0 via engine (we'll need to expose this)
      await page.evaluate(() => {
        // Access the Vue component and set gold to 0
        const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
        if (app && app.ctx.engine) {
          app.ctx.engine.state.gold = 0;
          app.ctx.updateUIState();
        }
      });
      
      // Now expensive choices should be disabled
      await expect(expensiveChoice).toHaveClass(/disabled/);
    }
  });

  test('should track game progression through multiple choices', async ({ page }) => {
    let currentLevel = 1;
    let currentExp = 0;
    
    // Play through several rounds
    for (let round = 0; round < 3; round++) {
      // Generate choices if in waiting phase
      if (await page.locator('.waiting-phase').isVisible()) {
        await page.click('text=Generate Adventure Choices');
        await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
      }
      
      // Select first affordable choice
      const affordableChoices = page.locator('.choice-btn:not(.disabled)');
      const choiceCount = await affordableChoices.count();
      
      if (choiceCount > 0) {
        await affordableChoices.first().click();
        await expect(page.locator('.waiting-phase')).toBeVisible({ timeout: 15000 });
        
        // Check that stats updated
        const newLevel = parseInt(await page.locator('.stat-value').first().textContent());
        const newExp = parseInt(await page.locator('.stat-value').nth(1).textContent());
        
        // Experience should have increased
        expect(newExp).toBeGreaterThanOrEqual(currentExp);
        
        // Level might have increased
        expect(newLevel).toBeGreaterThanOrEqual(currentLevel);
        
        currentLevel = newLevel;
        currentExp = newExp;
      }
    }
    
    // Should have made meaningful progress
    expect(currentExp).toBeGreaterThan(20); // At least some experience gained
  });

  test('should handle errors gracefully', async ({ page }) => {
    // Enable debug mode to monitor errors
    await page.click('.debug-toggle-btn');
    await expect(page.locator('.debug-panel')).toBeVisible();
    
    // Simulate an error by breaking the AI service
    await page.evaluate(() => {
      const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
      if (app && app.ctx.engine) {
        // Make AI service throw errors
        app.ctx.engine.aiService.generateChoices = () => {
          throw new Error('Simulated AI failure');
        };
      }
    });
    
    // Try to generate choices
    await page.click('text=Generate Adventure Choices');
    
    // Should show error
    await expect(page.locator('.error-display')).toBeVisible({ timeout: 5000 });
    await expect(page.locator('.error-message')).toContainText('Simulated AI failure');
    
    // Should be able to dismiss error
    await page.click('.error-close');
    await expect(page.locator('.error-display')).not.toBeVisible();
  });

  test('should show debug information when enabled', async ({ page }) => {
    // Enable debug mode
    await page.click('.debug-toggle-btn');
    await expect(page.locator('.debug-panel')).toBeVisible();
    
    // Check debug information is displayed
    await expect(page.locator('.debug-panel')).toContainText('Phase:');
    await expect(page.locator('.debug-panel')).toContainText('Current Node:');
    await expect(page.locator('.debug-panel')).toContainText('Can Advance:');
    await expect(page.locator('.debug-panel')).toContainText('Validation:');
    
    // Should show export button
    await expect(page.locator('text=Export Debug Data')).toBeVisible();
    
    // Can hide debug panel
    await page.click('text=Hide Debug');
    await expect(page.locator('.debug-panel')).not.toBeVisible();
  });

  test('should update graph visualization as story progresses', async ({ page }) => {
    // Check initial graph stats
    await expect(page.locator('.graph-stats')).toContainText('1 locations');
    await expect(page.locator('.graph-stats')).toContainText('0 paths');
    
    // Generate choices
    await page.click('text=Generate Adventure Choices');
    await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
    
    // Graph should show paths now
    await expect(page.locator('.graph-stats')).toContainText('3 paths'); // 3 choices = 3 edges
    
    // Make a choice
    await page.click('.choice-btn:first-child');
    await expect(page.locator('.waiting-phase')).toBeVisible({ timeout: 15000 });
    
    // Graph should show new location
    await expect(page.locator('.graph-stats')).toContainText('2 locations');
  });

  test('should persist and restore game state', async ({ page }) => {
    // Generate choices and make one
    await page.click('text=Generate Adventure Choices');
    await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
    await page.click('.choice-btn:first-child');
    await expect(page.locator('.waiting-phase')).toBeVisible({ timeout: 15000 });
    
    // Get current state
    const location = await page.locator('.current-location').textContent();
    const experience = await page.locator('.stat-value').nth(1).textContent();
    
    // Reload page
    await page.reload();
    await page.click('text=Adventure Game');
    
    // State should be restored
    await expect(page.locator('.current-location')).toContainText(location);
    await expect(page.locator('.stat-value').nth(1)).toContainText(experience);
  });

  test('should handle game reset correctly', async ({ page }) => {
    // Advance the game
    await page.click('text=Generate Adventure Choices');
    await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
    await page.click('.choice-btn:first-child');
    await expect(page.locator('.waiting-phase')).toBeVisible({ timeout: 15000 });
    
    // Trigger game over condition (simulate)
    await page.evaluate(() => {
      const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
      if (app && app.ctx.engine) {
        app.ctx.engine.state.isGameOver = true;
        app.ctx.engine.state.isWin = true;
        app.ctx.engine.state.endMessage = 'Test victory!';
        app.ctx.updateUIState();
      }
    });
    
    // Should show game over screen
    await expect(page.locator('.game-over')).toBeVisible();
    await expect(page.locator('text=Victory!')).toBeVisible();
    await expect(page.locator('text=Start New Adventure')).toBeVisible();
    
    // Reset game
    await page.click('text=Start New Adventure');
    
    // Should be back to initial state
    await expect(page.locator('.current-location')).toContainText('The Town Square of Architectura');
    await expect(page.locator('.stat-value').first()).toContainText('1'); // Level reset
    await expect(page.locator('.stat-value').nth(1)).toContainText('0'); // Experience reset
    await expect(page.locator('text=Generate Adventure Choices')).toBeVisible();
  });

  test('should show appropriate loading states', async ({ page }) => {
    // Click generate choices
    await page.click('text=Generate Adventure Choices');
    
    // Should immediately show loading
    await expect(page.locator('.generating-phase')).toBeVisible();
    await expect(page.locator('.spinner')).toBeVisible();
    await expect(page.locator('text=Generating adventure choices')).toBeVisible();
    
    // Wait for completion
    await expect(page.locator('.choosing-phase')).toBeVisible({ timeout: 10000 });
    
    // Select choice
    await page.click('.choice-btn:first-child');
    
    // Should show advancing loading state
    await expect(page.locator('.advancing-phase')).toBeVisible();
    await expect(page.locator('text=Advancing to new location')).toBeVisible();
    
    // Should complete
    await expect(page.locator('.waiting-phase')).toBeVisible({ timeout: 15000 });
  });
});