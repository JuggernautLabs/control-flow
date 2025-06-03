// Environment configuration loader that works in both browser and Node.js

// Check if we're in a browser environment
const isBrowser = typeof window !== 'undefined'

// Get API key from various sources
export const getAnthropicApiKey = () => {
  if (isBrowser) {
    // In browser, try Vite environment variables first
    return import.meta.env.VITE_ANTHROPIC_API_KEY || 
           (typeof __ANTHROPIC_API_KEY__ !== 'undefined' ? __ANTHROPIC_API_KEY__ : null)
  } else {
    // In Node.js, use process.env
    return process.env.ANTHROPIC_API_KEY || process.env.VITE_ANTHROPIC_API_KEY
  }
}

// Validate that we have an API key
export const validateApiKey = () => {
  const apiKey = getAnthropicApiKey()
  if (!apiKey || apiKey === 'your-api-key-here') {
    throw new Error('Anthropic API key not found. Please set VITE_ANTHROPIC_API_KEY in your .env file')
  }
  return apiKey
}

// Export configuration object
export const config = {
  anthropic: {
    apiKey: getAnthropicApiKey(),
    model: 'claude-3-5-sonnet-20241022',
    maxTokens: 1024
  }
}