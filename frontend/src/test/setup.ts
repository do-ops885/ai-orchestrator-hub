import '@testing-library/jest-dom'
import { expect, afterEach, vi } from 'vitest'
import { cleanup } from '@testing-library/react'
import * as matchers from '@testing-library/jest-dom/matchers'

// Extend expect with jest-dom matchers
expect.extend(matchers)

// Mock environment variables
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(), // deprecated
    removeListener: vi.fn(), // deprecated
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
})

// Mock ResizeObserver
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}))

// Mock IntersectionObserver
global.IntersectionObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}))

// Mock WebSocket
global.WebSocket = vi.fn().mockImplementation(() => ({
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
  dispatchEvent: vi.fn(),
  send: vi.fn(),
  close: vi.fn(),
  readyState: 1,
  CONNECTING: 0,
  OPEN: 1,
  CLOSING: 2,
  CLOSED: 3,
}))

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
  key: vi.fn(),
  length: 0,
}
global.localStorage = localStorageMock

// Mock sessionStorage
const sessionStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
  key: vi.fn(),
  length: 0,
}
global.sessionStorage = sessionStorageMock

// Mock fetch globally
global.fetch = vi.fn()

// Mock console methods to reduce noise in tests
const originalConsole = { ...console }
beforeAll(() => {
  console.warn = vi.fn()
  console.error = vi.fn()
  console.log = vi.fn()
})

afterAll(() => {
  Object.assign(console, originalConsole)
})

// Cleanup after each test
afterEach(() => {
  cleanup()
  vi.clearAllMocks()
  localStorageMock.clear()
  sessionStorageMock.clear()
})

// Custom test utilities
global.testUtils = {
  // Create mock API responses
  createMockResponse: (data: any, status = 200) => ({
    ok: status >= 200 && status < 300,
    status,
    json: () => Promise.resolve(data),
    text: () => Promise.resolve(JSON.stringify(data)),
    headers: new Headers(),
  }),

  // Create mock error response
  createMockError: (message: string, status = 500) => ({
    ok: false,
    status,
    json: () => Promise.resolve({ error: message }),
    text: () => Promise.resolve(message),
    headers: new Headers(),
  }),

  // Wait for a specific condition
  waitForCondition: (condition: () => boolean, timeout = 1000) => {
    return new Promise((resolve, reject) => {
      const startTime = Date.now()
      const checkCondition = () => {
        if (condition()) {
          resolve(void 0)
        } else if (Date.now() - startTime > timeout) {
          reject(new Error('Condition not met within timeout'))
        } else {
          setTimeout(checkCondition, 10)
        }
      }
      checkCondition()
    })
  },

  // Generate test data
  generateTestData: {
    user: () => ({
      id: 'test-user-id',
      username: 'testuser',
      email: 'test@example.com',
      role: 'user',
      createdAt: new Date().toISOString(),
    }),

    agent: () => ({
      id: 'test-agent-id',
      name: 'Test Agent',
      type: 'worker',
      status: 'idle',
      capabilities: ['test'],
      lastSeen: new Date().toISOString(),
    }),

    task: () => ({
      id: 'test-task-id',
      title: 'Test Task',
      description: 'A test task',
      status: 'pending',
      priority: 'medium',
      createdAt: new Date().toISOString(),
    }),
  },

  // Mock WebSocket connection
  createMockWebSocket: () => {
    const mockWS = {
      send: vi.fn(),
      close: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      readyState: 1,
      CONNECTING: 0,
      OPEN: 1,
      CLOSING: 2,
      CLOSED: 3,
    }

    // Simulate connection
    setTimeout(() => {
      mockWS.addEventListener.mock.calls
        .filter(([event]) => event === 'open')
        .forEach(([, handler]) => handler())
    }, 0)

    return mockWS
  },
}

// Type declarations for global test utilities
declare global {
  var testUtils: {
    createMockResponse: (data: any, status?: number) => any
    createMockError: (message: string, status?: number) => any
    waitForCondition: (condition: () => boolean, timeout?: number) => Promise<void>
    generateTestData: {
      user: () => any
      agent: () => any
      task: () => any
    }
    createMockWebSocket: () => any
  }
}
