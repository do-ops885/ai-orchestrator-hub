import { defineConfig, devices } from '@playwright/test'
import { setupWebSocketMock, teardownWebSocketMock } from './src/test/playwright-websocket-utils'

/**
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './e2e',
  /* Run tests in files in parallel */
  fullyParallel: true,
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* Retry on CI only */
  retries: process.env.CI ? 2 : 0,
  /* Opt out of parallel tests on CI. */
  workers: process.env.CI ? 1 : undefined,
  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: 'html',
  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
  use: {
    /* Base URL to use in actions like `await page.goto('/')`. */
    baseURL: 'http://localhost:3000',

    /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
    trace: 'on-first-retry',
  },

  /* Global setup for WebSocket mocking */
  globalSetup: './src/test/global-setup.ts',

  /* Global teardown */
  globalTeardown: './src/test/global-teardown.ts',

  /* Configure projects for major browsers */
  projects: [
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        /* Set WebSocket URL for mock server */
        extraHTTPHeaders: {
          'X-WebSocket-URL': process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:3001/ws',
        },
      },
    },

    {
      name: 'firefox',
      use: {
        ...devices['Desktop Firefox'],
        /* Set WebSocket URL for mock server */
        extraHTTPHeaders: {
          'X-WebSocket-URL': process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:3001/ws',
        },
      },
    },

    {
      name: 'webkit',
      use: {
        ...devices['Desktop Safari'],
        /* Set WebSocket URL for mock server */
        extraHTTPHeaders: {
          'X-WebSocket-URL': process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:3001/ws',
        },
      },
    },

    /* Test against mobile viewports. */
    {
      name: 'Mobile Chrome',
      use: {
        ...devices['Pixel 5'],
        /* Set WebSocket URL for mock server */
        extraHTTPHeaders: {
          'X-WebSocket-URL': process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:3001/ws',
        },
      },
    },
    {
      name: 'Mobile Safari',
      use: {
        ...devices['iPhone 12'],
        /* Set WebSocket URL for mock server */
        extraHTTPHeaders: {
          'X-WebSocket-URL': process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:3001/ws',
        },
      },
    },

    /* Test against branded browsers. */
    // {
    //   name: 'Microsoft Edge',
    //   use: { ...devices['Desktop Edge'], channel: 'msedge' },
    // },
    // {
    //   name: 'Google Chrome',
    //   use: { ...devices['Desktop Chrome'], channel: 'chrome' },
    // },
  ],

  /* Run your local dev server before starting the tests */
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:3000',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,
    /* Set environment variables for WebSocket mocking */
    env: {
      NEXT_PUBLIC_WS_URL: 'ws://localhost:3001/ws',
      USE_MOCK_WEBSOCKET: 'true',
    },
  },
})
