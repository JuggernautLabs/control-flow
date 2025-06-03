import { vi } from 'vitest'

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
}

global.localStorage = localStorageMock

// Mock fetch
global.fetch = vi.fn()

// Mock console to suppress logs in tests
global.console = {
  ...console,
  log: vi.fn(),
  error: vi.fn(),
  warn: vi.fn(),
}

// Reset all mocks before each test
beforeEach(() => {
  vi.clearAllMocks()
})