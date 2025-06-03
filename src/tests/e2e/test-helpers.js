/**
 * Test helpers for Playwright UI tests
 */

/**
 * Navigate to adventure game and enable debug mode
 */
export async function setupAdventureGame(page, enableDebug = true) {
  await page.goto('/');
  await page.click('text=Adventure Game');
  
  if (enableDebug) {
    await page.click('.debug-toggle-btn');
    await page.waitForSelector('.debug-panel', { state: 'visible' });
  }
  
  return page;
}

/**
 * Wait for a specific game phase
 */
export async function waitForGamePhase(page, phase, timeout = 10000) {
  await page.waitForSelector('.debug-panel', { state: 'visible' });
  await page.waitForFunction(
    (expectedPhase) => {
      const debugPanel = document.querySelector('.debug-panel');
      return debugPanel && debugPanel.textContent.includes(`Phase: ${expectedPhase}`);
    },
    phase,
    { timeout }
  );
}

/**
 * Generate choices and wait for completion
 */
export async function generateChoices(page) {
  await page.click('text=Generate Adventure Choices');
  await waitForGamePhase(page, 'choosing');
  return page.locator('.choice-btn');
}

/**
 * Make a choice and wait for advancement
 */
export async function makeChoice(page, choiceIndex = 0) {
  const choices = page.locator('.choice-btn:not(.disabled)');
  await choices.nth(choiceIndex).click();
  await waitForGamePhase(page, 'waiting_for_choices', 15000);
}

/**
 * Complete a full game cycle (generate choices -> make choice)
 */
export async function completeGameCycle(page, choiceIndex = 0) {
  await generateChoices(page);
  await makeChoice(page, choiceIndex);
}

/**
 * Get current game stats
 */
export async function getGameStats(page) {
  const level = await page.locator('.stat-value').nth(0).textContent();
  const experience = await page.locator('.stat-value').nth(1).textContent();
  const gold = await page.locator('.stat-value').nth(2).textContent();
  
  return {
    level: parseInt(level),
    experience: parseInt(experience),
    gold: parseInt(gold.replace('💰', ''))
  };
}

/**
 * Verify game state is valid
 */
export async function verifyValidGameState(page) {
  await page.waitForSelector('.debug-panel', { state: 'visible' });
  await page.waitForFunction(() => {
    const debugPanel = document.querySelector('.debug-panel');
    return debugPanel && debugPanel.textContent.includes('Validation: ✅');
  });
}

/**
 * Manipulate game engine state for testing
 */
export async function setEngineState(page, stateChanges) {
  await page.evaluate((changes) => {
    const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
    if (app && app.ctx.engine) {
      Object.assign(app.ctx.engine.state, changes);
      app.ctx.updateUIState();
    }
  }, stateChanges);
}

/**
 * Break AI service to simulate failures
 */
export async function breakAIService(page, method, error = 'Simulated AI failure') {
  await page.evaluate(({ method, error }) => {
    const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
    if (app && app.ctx.engine) {
      app.ctx.engine.aiService[method] = async () => {
        throw new Error(error);
      };
    }
  }, { method, error });
}

/**
 * Restore working AI service
 */
export async function restoreAIService(page) {
  await page.evaluate(() => {
    const app = document.querySelector('.adventure-game-v2').__vueParentComponent;
    if (app && app.ctx.engine) {
      // Restore basic working methods
      app.ctx.engine.aiService.generateChoices = async () => ({
        choices: [
          { text: 'Test Choice 1', icon: '🧪', cost: 0, experience: 10 },
          { text: 'Test Choice 2', icon: '🔬', cost: 5, experience: 15 },
          { text: 'Test Choice 3', icon: '⚗️', cost: 10, experience: 20 }
        ],
        confidence: 0.8,
        reasoning: 'Test choices'
      });
      
      app.ctx.engine.aiService.generateStoryNode = async (choice) => ({
        location: `Test Location: ${choice.text}`,
        situation: `<p>You chose "${choice.text}" and find yourself in a test scenario.</p>`,
        confidence: 0.9,
        reasoning: 'Test node generation'
      });
    }
  });
}

/**
 * Simulate invalid localStorage data
 */
export async function setInvalidLocalStorage(page, data) {
  await page.evaluate((data) => {
    if (typeof data === 'string') {
      localStorage.setItem('adventure_game_state', data);
    } else {
      localStorage.setItem('adventure_game_state', JSON.stringify(data));
    }
  }, data);
}

/**
 * Clear localStorage
 */
export async function clearLocalStorage(page) {
  await page.evaluate(() => {
    localStorage.clear();
  });
}

/**
 * Wait for error to appear and verify message
 */
export async function expectError(page, errorMessage, timeout = 5000) {
  await page.waitForSelector('.error-display', { state: 'visible', timeout });
  if (errorMessage) {
    await page.waitForFunction(
      (message) => {
        const errorDisplay = document.querySelector('.error-message');
        return errorDisplay && errorDisplay.textContent.includes(message);
      },
      errorMessage,
      { timeout }
    );
  }
}

/**
 * Export debug data
 */
export async function exportDebugData(page) {
  const downloadPromise = page.waitForEvent('download');
  await page.click('text=Export Debug Data');
  return await downloadPromise;
}

/**
 * Simulate user journey through multiple game cycles
 */
export async function simulateUserJourney(page, cycles = 3) {
  const journey = [];
  
  for (let i = 0; i < cycles; i++) {
    const statsBefore = await getGameStats(page);
    
    await completeGameCycle(page, i % 3); // Vary choice selection
    
    const statsAfter = await getGameStats(page);
    const location = await page.locator('.current-location').textContent();
    
    journey.push({
      cycle: i + 1,
      statsBefore,
      statsAfter,
      location,
      experienceGained: statsAfter.experience - statsBefore.experience,
      goldSpent: statsBefore.gold - statsAfter.gold
    });
    
    await verifyValidGameState(page);
  }
  
  return journey;
}