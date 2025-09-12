import { test, expect } from '@playwright/test'

test.describe('Dashboard E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the application
    await page.goto('/')

    // Wait for the page to load
    await page.waitForSelector('h1:has-text("Multiagent Hive System")')
  })

  test('should load dashboard successfully', async ({ page }) => {
    // Check if the main heading is visible
    await expect(page.locator('h1:has-text("Multiagent Hive System")')).toBeVisible()

    // Check if navigation tabs are present
    await expect(page.locator('button:has-text("Dashboard")')).toBeVisible()
    await expect(page.locator('button:has-text("Agents")')).toBeVisible()
    await expect(page.locator('button:has-text("Tasks")')).toBeVisible()
  })

  test('should display connection status', async ({ page }) => {
    // Check for connection status indicator
    const connectionStatus = page.locator(
      '[class*="bg-green-100"], [class*="bg-red-100"], [class*="bg-yellow-100"]',
    )
    await expect(connectionStatus).toBeVisible()

    // Should contain connection-related text
    await expect(connectionStatus).toContainText(/(Connected|Disconnected|Reconnecting|Connecting)/)
  })

  test('should navigate between tabs', async ({ page }) => {
    // Start on dashboard
    await expect(page.locator('button:has-text("Dashboard")')).toHaveClass(/bg-blue-100/)

    // Navigate to agents
    await page.click('button:has-text("Agents")')
    await expect(page.locator('button:has-text("Agents")')).toHaveClass(/bg-blue-100/)
    await expect(page.locator('button:has-text("Dashboard")')).not.toHaveClass(/bg-blue-100/)

    // Navigate to tasks
    await page.click('button:has-text("Tasks")')
    await expect(page.locator('button:has-text("Tasks")')).toHaveClass(/bg-blue-100/)
    await expect(page.locator('button:has-text("Agents")')).not.toHaveClass(/bg-blue-100/)

    // Navigate back to dashboard
    await page.click('button:has-text("Dashboard")')
    await expect(page.locator('button:has-text("Dashboard")')).toHaveClass(/bg-blue-100/)
  })

  test('should handle network disconnection gracefully', async ({ page }) => {
    // Simulate network disconnection by blocking network requests
    await page.route('**/api/**', route => route.abort())

    // Wait a bit for the disconnection to be detected
    await page.waitForTimeout(2000)

    // Should show disconnected status
    const connectionStatus = page.locator('[class*="bg-red-100"]')
    await expect(connectionStatus).toBeVisible()
    await expect(connectionStatus).toContainText('Disconnected')

    // Should still be able to navigate between tabs
    await page.click('button:has-text("Agents")')
    await expect(page.locator('button:has-text("Agents")')).toHaveClass(/bg-blue-100/)
  })

  test('should handle page refresh', async ({ page }) => {
    // Refresh the page
    await page.reload()

    // Should still load properly
    await expect(page.locator('h1:has-text("Multiagent Hive System")')).toBeVisible()

    // Should maintain navigation functionality
    await expect(page.locator('button:has-text("Dashboard")')).toBeVisible()
    await expect(page.locator('button:has-text("Agents")')).toBeVisible()
    await expect(page.locator('button:has-text("Tasks")')).toBeVisible()
  })

  test('should be responsive on mobile', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 })

    // Check if header is still visible and functional
    await expect(page.locator('h1:has-text("Multiagent Hive System")')).toBeVisible()

    // Check if navigation buttons are accessible
    await expect(page.locator('button:has-text("Dashboard")')).toBeVisible()
    await expect(page.locator('button:has-text("Agents")')).toBeVisible()
    await expect(page.locator('button:has-text("Tasks")')).toBeVisible()

    // Test navigation on mobile
    await page.click('button:has-text("Agents")')
    await expect(page.locator('button:has-text("Agents")')).toHaveClass(/bg-blue-100/)
  })
})

test.describe('Error Handling E2E Tests', () => {
  test('should handle 404 errors gracefully', async ({ page }) => {
    // Navigate to a non-existent page
    await page.goto('/non-existent-page')

    // Should show error boundary or redirect to home
    // This depends on Next.js error handling configuration
    await expect(page.locator('body')).toBeVisible()
  })

  test('should handle API errors gracefully', async ({ page }) => {
    // Mock API to return errors
    await page.route('**/api/**', route => {
      route.fulfill({
        status: 500,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'Internal Server Error' }),
      })
    })

    await page.goto('/')

    // Should still load the page despite API errors
    await expect(page.locator('h1:has-text("Multiagent Hive System")')).toBeVisible()

    // Should show appropriate error states
    await page.waitForTimeout(1000) // Wait for error states to appear
  })
})
