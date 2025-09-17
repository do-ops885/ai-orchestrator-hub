// Error logging and reporting utilities for the frontend

export interface ErrorReport {
  id: string
  timestamp: string
  message: string
  stack?: string
  componentStack?: string
  userAgent: string
  url: string
  userId?: string
  sessionId?: string
  severity: 'low' | 'medium' | 'high' | 'critical'
  category: 'javascript' | 'network' | 'api' | 'ui' | 'unknown'
  context?: Record<string, unknown>
  retryCount?: number
  resolved?: boolean
  resolution?: string
}

export interface ErrorLoggerConfig {
  enableConsoleLogging: boolean
  enableRemoteReporting: boolean
  remoteEndpoint?: string
  maxStoredErrors: number
  batchSize: number
  flushInterval: number
  enableUserTracking: boolean
  enableSessionTracking: boolean
}

class ErrorLogger {
  private config: ErrorLoggerConfig
  private errorQueue: ErrorReport[] = []
  private sessionId: string
  private flushTimer?: NodeJS.Timeout

  constructor(config: Partial<ErrorLoggerConfig> = {}) {
    this.config = {
      enableConsoleLogging: true,
      enableRemoteReporting: false,
      maxStoredErrors: 100,
      batchSize: 10,
      flushInterval: 30000, // 30 seconds
      enableUserTracking: false,
      enableSessionTracking: true,
      ...config,
    }

    this.sessionId = this.generateSessionId()

    if (this.config.enableRemoteReporting && this.config.flushInterval > 0) {
      this.startPeriodicFlush()
    }
  }

  private generateSessionId(): string {
    return `session_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`
  }

  private generateErrorId(): string {
    return `error_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`
  }

  private determineSeverity(error: Error): ErrorReport['severity'] {
    const message = error.message.toLowerCase()

    if (
      message.includes('network') ||
      message.includes('fetch') ||
      message.includes('connection')
    ) {
      return 'medium'
    }
    if (
      message.includes('unauthorized') ||
      message.includes('forbidden') ||
      message.includes('auth')
    ) {
      return 'high'
    }
    if (
      message.includes('typeerror') ||
      message.includes('referenceerror') ||
      message.includes('syntaxerror')
    ) {
      return 'high'
    }
    if (message.includes('rangeerror') || message.includes('urierror')) {
      return 'medium'
    }

    return 'low'
  }

  private determineCategory(
    error: Error,
    context?: Record<string, unknown>,
  ): ErrorReport['category'] {
    const message = error.message.toLowerCase()

    if (context?.componentName || message.includes('component') || message.includes('render')) {
      return 'ui'
    }
    if (
      message.includes('network') ||
      message.includes('fetch') ||
      message.includes('connection')
    ) {
      return 'network'
    }
    if (message.includes('api') || message.includes('http') || message.includes('request')) {
      return 'api'
    }

    return 'javascript'
  }

  public logError(
    error: Error,
    context?: Record<string, unknown>,
    componentStack?: string,
    retryCount?: number,
  ): string {
    const errorReport: ErrorReport = {
      id: this.generateErrorId(),
      timestamp: new Date().toISOString(),
      message: error.message,
      stack: error.stack,
      componentStack,
      userAgent: navigator.userAgent,
      url: window.location.href,
      sessionId: this.config.enableSessionTracking ? this.sessionId : undefined,
      severity: this.determineSeverity(error),
      category: this.determineCategory(error, context),
      context,
      retryCount,
      resolved: false,
    }

    // Add to queue
    this.errorQueue.push(errorReport)

    // Console logging
    if (this.config.enableConsoleLogging) {
      console.error('[ErrorLogger]', errorReport)
    }

    // Maintain queue size
    if (this.errorQueue.length > this.config.maxStoredErrors) {
      this.errorQueue.shift()
    }

    // Immediate flush for critical errors
    if (errorReport.severity === 'critical') {
      this.flush()
    }

    return errorReport.id
  }

  public logNetworkError(
    url: string,
    method: string,
    status?: number,
    responseText?: string,
    context?: Record<string, unknown>,
  ): string {
    const error = new Error(
      `Network request failed: ${method} ${url}${status ? ` (${status})` : ''}`,
    )
    const networkContext = {
      ...context,
      url,
      method,
      status,
      responseText: responseText?.substring(0, 500), // Limit response text
    }

    return this.logError(error, networkContext)
  }

  public logAPIError(
    endpoint: string,
    method: string,
    status: number,
    responseData?: unknown,
    context?: Record<string, unknown>,
  ): string {
    const error = new Error(`API request failed: ${method} ${endpoint} (${status})`)
    const apiContext = {
      ...context,
      endpoint,
      method,
      status,
      responseData:
        typeof responseData === 'object'
          ? JSON.stringify(responseData).substring(0, 500)
          : responseData,
    }

    return this.logError(error, apiContext)
  }

  public markResolved(errorId: string, resolution?: string): void {
    const error = this.errorQueue.find(e => e.id === errorId)
    if (error) {
      error.resolved = true
      error.resolution = resolution
    }
  }

  private async flush(): Promise<void> {
    if (this.errorQueue.length === 0 || !this.config.enableRemoteReporting) {
      return
    }

    const errorsToSend = this.errorQueue.splice(0, this.config.batchSize)

    try {
      if (this.config.remoteEndpoint) {
        const response = await fetch(this.config.remoteEndpoint, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            errors: errorsToSend,
            sessionId: this.sessionId,
            timestamp: new Date().toISOString(),
          }),
        })

        if (!response.ok) {
          console.warn('[ErrorLogger] Failed to send error reports:', response.status)
          // Re-queue errors for retry
          this.errorQueue.unshift(...errorsToSend)
        } else {
          // Successfully sent error reports
        }
      }
    } catch (error) {
      console.warn('[ErrorLogger] Failed to flush error reports:', error)
      // Re-queue errors for retry
      this.errorQueue.unshift(...errorsToSend)
    }
  }

  private startPeriodicFlush(): void {
    this.flushTimer = setInterval(() => {
      this.flush()
    }, this.config.flushInterval)
  }

  public async forceFlush(): Promise<void> {
    await this.flush()
  }

  public getQueuedErrors(): ErrorReport[] {
    return [...this.errorQueue]
  }

  public getErrorStats(): {
    total: number
    bySeverity: Record<ErrorReport['severity'], number>
    byCategory: Record<ErrorReport['category'], number>
    unresolved: number
  } {
    const stats = {
      total: this.errorQueue.length,
      bySeverity: {
        low: 0,
        medium: 0,
        high: 0,
        critical: 0,
      } as Record<ErrorReport['severity'], number>,
      byCategory: {
        javascript: 0,
        network: 0,
        api: 0,
        ui: 0,
        unknown: 0,
      } as Record<ErrorReport['category'], number>,
      unresolved: 0,
    }

    this.errorQueue.forEach(error => {
      stats.bySeverity[error.severity]++
      stats.byCategory[error.category]++
      if (!error.resolved) {
        stats.unresolved++
      }
    })

    return stats
  }

  public clearQueue(): void {
    this.errorQueue = []
  }

  public destroy(): void {
    if (this.flushTimer) {
      clearInterval(this.flushTimer)
    }
    this.clearQueue()
  }
}

// Global error logger instance
let globalErrorLogger: ErrorLogger | null = null

export function getErrorLogger(config?: Partial<ErrorLoggerConfig>): ErrorLogger {
  if (!globalErrorLogger) {
    globalErrorLogger = new ErrorLogger(config)
  }
  return globalErrorLogger
}

export function logError(
  error: Error,
  context?: Record<string, unknown>,
  componentStack?: string,
  retryCount?: number,
): string {
  return getErrorLogger().logError(error, context, componentStack, retryCount)
}

export function logNetworkError(
  url: string,
  method: string,
  status?: number,
  responseText?: string,
  context?: Record<string, unknown>,
): string {
  return getErrorLogger().logNetworkError(url, method, status, responseText, context)
}

export function logAPIError(
  endpoint: string,
  method: string,
  status: number,
  responseData?: unknown,
  context?: Record<string, unknown>,
): string {
  return getErrorLogger().logAPIError(endpoint, method, status, responseData, context)
}

// Global error handler for unhandled errors
export function setupGlobalErrorHandler(): void {
  // Handle unhandled promise rejections
  window.addEventListener('unhandledrejection', event => {
    const error = new Error(`Unhandled promise rejection: ${event.reason}`)
    logError(error, {
      type: 'unhandledrejection',
      reason: event.reason,
      promise: event.promise,
    })
  })

  // Handle uncaught errors
  window.addEventListener('error', event => {
    const error = event.error || new Error(event.message)
    logError(error, {
      type: 'uncaughterror',
      filename: event.filename,
      lineno: event.lineno,
      colno: event.colno,
    })
  })

  // Handle console errors (if needed)
  const originalConsoleError = console.error
  console.error = (...args) => {
    originalConsoleError.apply(console, args)

    // Log console errors as well
    if (args.length > 0) {
      const message = args
        .map(arg => (typeof arg === 'object' ? JSON.stringify(arg) : String(arg)))
        .join(' ')

      const error = new Error(`Console error: ${message}`)
      logError(error, {
        type: 'console_error',
        originalArgs: args,
      })
    }
  }
}

export { ErrorLogger }
export default ErrorLogger
