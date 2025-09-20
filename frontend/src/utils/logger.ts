// General purpose logging utility for the frontend
// Replaces console.warn statements with structured logging

export type LogLevel = 'debug' | 'info' | 'warn' | 'error'

export interface LogEntry {
  level: LogLevel
  message: string
  timestamp: string
  context?: Record<string, unknown>
  component?: string
  userId?: string
  sessionId?: string
}

export interface LoggerConfig {
  enableConsoleLogging: boolean
  enableRemoteLogging: boolean
  remoteEndpoint?: string
  minLevel: LogLevel
  enableUserTracking: boolean
  enableSessionTracking: boolean
  maxStoredLogs: number
}

class Logger {
  private config: LoggerConfig
  private logQueue: LogEntry[] = []
  private sessionId: string

  constructor(config: Partial<LoggerConfig> = {}) {
    this.config = {
      enableConsoleLogging: true,
      enableRemoteLogging: false,
      minLevel: 'info',
      enableUserTracking: false,
      enableSessionTracking: true,
      maxStoredLogs: 1000,
      ...config,
    }

    this.sessionId = this.generateSessionId()
  }

  private generateSessionId(): string {
    return `session_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`
  }

  private shouldLog(level: LogLevel): boolean {
    const levels = ['debug', 'info', 'warn', 'error']
    const minLevelIndex = levels.indexOf(this.config.minLevel)
    const currentLevelIndex = levels.indexOf(level)
    return currentLevelIndex >= minLevelIndex
  }

  private formatMessage(level: LogLevel, message: string, context?: Record<string, unknown>): string {
    const timestamp = new Date().toISOString()
    const levelEmoji = {
      debug: '🐛',
      info: 'ℹ️',
      warn: '⚠️',
      error: '❌',
    }[level]

    let formattedMessage = `${levelEmoji} [${timestamp}] ${message}`

    if (context && Object.keys(context).length > 0) {
      formattedMessage += ` | Context: ${JSON.stringify(context)}`
    }

    return formattedMessage
  }

  private log(level: LogLevel, message: string, context?: Record<string, unknown>, component?: string): void {
    if (!this.shouldLog(level)) {
      return
    }

    const entry: LogEntry = {
      level,
      message,
      timestamp: new Date().toISOString(),
      context,
      component,
      sessionId: this.config.enableSessionTracking ? this.sessionId : undefined,
    }

    // Add to queue
    this.logQueue.push(entry)

    // Maintain queue size
    if (this.logQueue.length > this.config.maxStoredLogs) {
      this.logQueue.shift()
    }

    // Console logging
    if (this.config.enableConsoleLogging) {
      const consoleMethod = level === 'debug' ? 'debug' : level === 'warn' ? 'warn' : level === 'error' ? 'error' : 'log'
      const formattedMessage = this.formatMessage(level, message, context)

      if (consoleMethod === 'debug') {
        console.debug(formattedMessage)
      } else if (consoleMethod === 'warn') {
        console.warn(formattedMessage)
      } else if (consoleMethod === 'error') {
        console.error(formattedMessage)
      } else {
        console.log(formattedMessage)
      }
    }

    // Remote logging (if enabled)
    if (this.config.enableRemoteLogging && this.config.remoteEndpoint) {
      this.sendToRemote(entry)
    }
  }

  private async sendToRemote(entry: LogEntry): Promise<void> {
    try {
      await fetch(this.config.remoteEndpoint!, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(entry),
      })
    } catch (error) {
      // Silently fail remote logging to avoid infinite loops
      console.warn('[Logger] Failed to send log to remote:', error)
    }
  }

  public debug(message: string, context?: Record<string, unknown>, component?: string): void {
    this.log('debug', message, context, component)
  }

  public info(message: string, context?: Record<string, unknown>, component?: string): void {
    this.log('info', message, context, component)
  }

  public warn(message: string, context?: Record<string, unknown>, component?: string): void {
    this.log('warn', message, context, component)
  }

  public error(message: string, context?: Record<string, unknown>, component?: string): void {
    this.log('error', message, context, component)
  }

  public getLogs(level?: LogLevel): LogEntry[] {
    if (level) {
      return this.logQueue.filter(entry => entry.level === level)
    }
    return [...this.logQueue]
  }

  public getLogStats(): {
    total: number
    byLevel: Record<LogLevel, number>
    recentErrors: LogEntry[]
  } {
    const stats = {
      total: this.logQueue.length,
      byLevel: {
        debug: 0,
        info: 0,
        warn: 0,
        error: 0,
      },
      recentErrors: [] as LogEntry[],
    }

    this.logQueue.forEach(entry => {
      stats.byLevel[entry.level]++
      if (entry.level === 'error') {
        stats.recentErrors.push(entry)
      }
    })

    // Keep only last 10 errors
    stats.recentErrors = stats.recentErrors.slice(-10)

    return stats
  }

  public clearLogs(): void {
    this.logQueue = []
  }
}

// Global logger instance
let globalLogger: Logger | null = null

export function getLogger(config?: Partial<LoggerConfig>): Logger {
  if (!globalLogger) {
    globalLogger = new Logger(config)
  }
  return globalLogger
}

// Convenience functions
export function logDebug(message: string, context?: Record<string, unknown>, component?: string): void {
  getLogger().debug(message, context, component)
}

export function logInfo(message: string, context?: Record<string, unknown>, component?: string): void {
  getLogger().info(message, context, component)
}

export function logWarn(message: string, context?: Record<string, unknown>, component?: string): void {
  getLogger().warn(message, context, component)
}

export function logError(message: string, context?: Record<string, unknown>, component?: string): void {
  getLogger().error(message, context, component)
}

export { Logger }
export default Logger