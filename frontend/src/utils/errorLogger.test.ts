import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { ErrorLogger, logError, logNetworkError, logAPIError, getErrorLogger } from './errorLogger'

// Mock console methods
const originalConsoleError = console.error
const mockConsoleError = vi.fn()

// Mock fetch
const mockFetch = vi.fn()

global.fetch = mockFetch

describe('ErrorLogger', () => {
  let logger: ErrorLogger

  beforeEach(() => {
    vi.clearAllMocks()
    console.error = mockConsoleError
    logger = new ErrorLogger({
      enableConsoleLogging: true,
      enableRemoteReporting: false,
      maxStoredErrors: 10,
    })
  })

  afterEach(() => {
    console.error = originalConsoleError
    logger.destroy()
  })

  describe('constructor', () => {
    it('should initialize with default config', () => {
      const defaultLogger = new ErrorLogger()
      expect(defaultLogger).toBeDefined()
      defaultLogger.destroy()
    })

    it('should merge config with defaults', () => {
      const customLogger = new ErrorLogger({
        enableConsoleLogging: false,
        maxStoredErrors: 5,
      })
      expect(customLogger).toBeDefined()
      customLogger.destroy()
    })
  })

  describe('logError', () => {
    it('should log an error and return an ID', () => {
      const error = new Error('Test error')
      const context = { component: 'TestComponent' }

      const errorId = logger.logError(error, context)

      expect(errorId).toMatch(/^error_\d+_[a-z0-9]+$/)
      expect(mockConsoleError).toHaveBeenCalled()
    })

    it('should determine severity correctly', () => {
      const networkError = new Error('Network connection failed')
      const authError = new Error('Unauthorized access')
      const jsError = new Error('TypeError: Cannot read property')

      logger.logError(networkError)
      logger.logError(authError)
      logger.logError(jsError)

      const errors = logger.getQueuedErrors()
      expect(errors[0].severity).toBe('medium') // network
      expect(errors[1].severity).toBe('high') // auth
      expect(errors[2].severity).toBe('high') // js error
    })

    it('should determine category correctly', () => {
      const networkError = new Error('Network error')
      const apiError = new Error('API request failed')
      const uiError = new Error('Component render error')

      logger.logError(networkError)
      logger.logError(apiError, { componentName: 'Button' })
      logger.logError(uiError)

      const errors = logger.getQueuedErrors()
      expect(errors[0].category).toBe('network')
      expect(errors[1].category).toBe('ui') // has componentName
      expect(errors[2].category).toBe('ui') // contains 'component'
    })

    it('should maintain queue size limit', () => {
      const smallLogger = new ErrorLogger({ maxStoredErrors: 2 })

      for (let i = 0; i < 5; i++) {
        smallLogger.logError(new Error(`Error ${i}`))
      }

      const errors = smallLogger.getQueuedErrors()
      expect(errors).toHaveLength(2)
      smallLogger.destroy()
    })

    it('should include context and retry count', () => {
      const error = new Error('Test error')
      const context = { userId: '123', action: 'save' }
      const retryCount = 2

      logger.logError(error, context, 'component stack', retryCount)

      const errors = logger.getQueuedErrors()
      const [loggedError] = errors
      expect(loggedError.context).toEqual(context)
      expect(loggedError.retryCount).toBe(retryCount)
      expect(loggedError.componentStack).toBe('component stack')
    })
  })

  describe('logNetworkError', () => {
    it('should log network errors with specific details', () => {
      const url = '/api/users'
      const method = 'GET'
      const status = 404
      const responseText = 'Not found'

      logger.logNetworkError(url, method, status, responseText)

      const errors = logger.getQueuedErrors()
      const [error] = errors
      expect(error.message).toContain('Network request failed')
      expect(error.message).toContain('GET /api/users (404)')
      expect(error.context?.url).toBe(url)
      expect(error.context?.method).toBe(method)
      expect(error.context?.status).toBe(status)
      expect(error.context?.responseText).toBe(responseText)
    })
  })

  describe('logAPIError', () => {
    it('should log API errors with specific details', () => {
      const endpoint = '/api/users'
      const method = 'POST'
      const status = 500
      const responseData = { error: 'Internal server error' }

      logger.logAPIError(endpoint, method, status, responseData)

      const errors = logger.getQueuedErrors()
      const [error] = errors
      expect(error.message).toContain('API request failed')
      expect(error.message).toContain('POST /api/users (500)')
      expect(error.context?.endpoint).toBe(endpoint)
      expect(error.context?.method).toBe(method)
      expect(error.context?.status).toBe(status)
      expect(error.context?.responseData).toBe('{"error":"Internal server error"}')
    })
  })

  describe('markResolved', () => {
    it('should mark an error as resolved', () => {
      const errorId = logger.logError(new Error('Test error'))
      logger.markResolved(errorId, 'Fixed by user')

      const errors = logger.getQueuedErrors()
      const error = errors.find(e => e.id === errorId)
      expect(error?.resolved).toBe(true)
      expect(error?.resolution).toBe('Fixed by user')
    })
  })

  describe('getErrorStats', () => {
    it('should return correct statistics', () => {
      logger.logError(new Error('Network error')) // medium, network
      logger.logError(new Error('Auth error')) // high, javascript
      logger.logError(new Error('UI error'), { componentName: 'Button' }) // low, ui

      const stats = logger.getErrorStats()

      expect(stats.total).toBe(3)
      expect(stats.bySeverity.medium).toBe(1)
      expect(stats.bySeverity.high).toBe(1)
      expect(stats.bySeverity.low).toBe(1)
      expect(stats.byCategory.network).toBe(1)
      expect(stats.byCategory.javascript).toBe(1)
      expect(stats.byCategory.ui).toBe(1)
      expect(stats.unresolved).toBe(3)
    })
  })

  describe('flush', () => {
    it('should not flush when remote reporting is disabled', async () => {
      logger.logError(new Error('Test error'))
      await logger.forceFlush()

      expect(mockFetch).not.toHaveBeenCalled()
    })

    it('should flush errors to remote endpoint', async () => {
      const remoteLogger = new ErrorLogger({
        enableRemoteReporting: true,
        remoteEndpoint: '/api/errors',
        batchSize: 2,
      })

      mockFetch.mockResolvedValue({ ok: true })

      remoteLogger.logError(new Error('Error 1'))
      remoteLogger.logError(new Error('Error 2'))
      remoteLogger.logError(new Error('Error 3'))

      await remoteLogger.forceFlush()

      expect(mockFetch).toHaveBeenCalledWith('/api/errors', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: expect.stringContaining('"errors":'),
      })

      // Should have sent 2 errors (batch size), 1 remaining
      expect(remoteLogger.getQueuedErrors()).toHaveLength(1)

      remoteLogger.destroy()
    })

    it('should re-queue errors on flush failure', async () => {
      const remoteLogger = new ErrorLogger({
        enableRemoteReporting: true,
        remoteEndpoint: '/api/errors',
      })

      mockFetch.mockRejectedValue(new Error('Network error'))

      remoteLogger.logError(new Error('Test error'))
      await remoteLogger.forceFlush()

      // Error should still be in queue
      expect(remoteLogger.getQueuedErrors()).toHaveLength(1)

      remoteLogger.destroy()
    })
  })

  describe('global functions', () => {
    it('should use global logger instance', () => {
      const error = new Error('Global test error')
      const errorId = logError(error)

      expect(errorId).toMatch(/^error_\d+_[a-z0-9]+$/)

      const globalLogger = getErrorLogger()
      const errors = globalLogger.getQueuedErrors()
      expect(errors.length).toBeGreaterThan(0)
    })

    it('should log network errors globally', () => {
      logNetworkError('/api/test', 'GET', 500)

      const globalLogger = getErrorLogger()
      const errors = globalLogger.getQueuedErrors()
      expect(errors[errors.length - 1].category).toBe('network')
    })

    it('should log API errors globally', () => {
      logAPIError('/api/test', 'POST', 400, { message: 'Bad request' })

      const globalLogger = getErrorLogger()
      const errors = globalLogger.getQueuedErrors()
      expect(errors[errors.length - 1].category).toBe('api')
    })
  })

  describe('clearQueue', () => {
    it('should clear all queued errors', () => {
      logger.logError(new Error('Error 1'))
      logger.logError(new Error('Error 2'))

      expect(logger.getQueuedErrors()).toHaveLength(2)

      logger.clearQueue()

      expect(logger.getQueuedErrors()).toHaveLength(0)
    })
  })
})
