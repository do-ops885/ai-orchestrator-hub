import { setupWebSocketMock } from './playwright-websocket-utils'

/* eslint-disable no-console, @typescript-eslint/no-explicit-any */
/**
 * Global setup for Playwright tests
 * Sets up the WebSocket mock server before all tests run
 */
export default async function globalSetup() {
  console.log('🚀 Setting up WebSocket mock server for E2E tests...')

  try {
    // Start the WebSocket mock server
    const mockServer = await setupWebSocketMock(3001)

    // Store reference for global teardown
    ;(globalThis as any).__MOCK_WEBSOCKET_SERVER__ = mockServer

    console.log('✅ WebSocket mock server started successfully')
    console.log('📡 Mock server running on ws://localhost:3001/ws')
  } catch (error) {
    console.error('❌ Failed to start WebSocket mock server:', error)
    throw error
  }
}
