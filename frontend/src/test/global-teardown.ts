import { teardownWebSocketMock } from './playwright-websocket-utils'

/* eslint-disable no-console, @typescript-eslint/no-explicit-any */
/**
 * Global teardown for Playwright tests
 * Cleans up the WebSocket mock server after all tests complete
 */
export default async function globalTeardown() {
  console.log('🛑 Tearing down WebSocket mock server...')

  try {
    // Stop the WebSocket mock server
    await teardownWebSocketMock()

    // Clean up global reference
    delete (globalThis as any).__MOCK_WEBSOCKET_SERVER__

    console.log('✅ WebSocket mock server stopped successfully')
  } catch (error) {
    console.error('❌ Failed to stop WebSocket mock server:', error)
    throw error
  }
}
