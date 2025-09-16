'use client'

import { useState, useCallback, useRef, useEffect, useMemo } from 'react'
import { logError, logNetworkError, logAPIError } from '@/utils/errorLogger'

export interface RetryConfig {
  maxRetries: number
  baseDelay: number
  maxDelay: number
  backoffFactor: number
  retryCondition?: (error: Error) => boolean
}

export interface RecoveryState {
  isRetrying: boolean
  retryCount: number
  lastError: Error | null
  canRetry: boolean
  nextRetryIn: number
}

const defaultRetryConfig: RetryConfig = {
  maxRetries: 3,
  baseDelay: 1000, // 1 second
  maxDelay: 30000, // 30 seconds
  backoffFactor: 2,
  retryCondition: (error: Error) => {
    // Retry on network errors, timeouts, and 5xx server errors
    const message = error.message.toLowerCase()
    return (
      message.includes('network') ||
      message.includes('timeout') ||
      message.includes('fetch') ||
      message.includes('connection')
    )
  },
}

export function useErrorRecovery<T>(
  operation: () => Promise<T>,
  config: Partial<RetryConfig> = {},
) {
  const [state, setState] = useState<RecoveryState>({
    isRetrying: false,
    retryCount: 0,
    lastError: null,
    canRetry: true,
    nextRetryIn: 0,
  })

  const retryConfig = useMemo(() => ({ ...defaultRetryConfig, ...config }), [config])
  const retryTimeoutRef = useRef<NodeJS.Timeout | null>(null)
  const abortControllerRef = useRef<AbortController | null>(null)

  const calculateDelay = useCallback(
    (retryCount: number): number => {
      const delay = retryConfig.baseDelay * Math.pow(retryConfig.backoffFactor, retryCount)
      return Math.min(delay, retryConfig.maxDelay)
    },
    [retryConfig],
  )

  const executeWithRetry = useCallback(async (): Promise<T | null> => {
    let lastError: Error | null = null

    for (let attempt = 0; attempt <= retryConfig.maxRetries; attempt++) {
      try {
        // Create new abort controller for this attempt
        abortControllerRef.current = new AbortController()

        setState(prev => ({
          ...prev,
          isRetrying: attempt > 0,
          retryCount: attempt,
        }))

        const result = await operation()

        // Success - reset state
        setState({
          isRetrying: false,
          retryCount: 0,
          lastError: null,
          canRetry: true,
          nextRetryIn: 0,
        })

        return result
      } catch (error) {
        lastError = error as Error

        // Log the error
        logError(lastError, {
          attempt,
          maxRetries: retryConfig.maxRetries,
          operation: operation.name || 'anonymous',
        })

        // Check if we should retry
        const shouldRetry =
          attempt < retryConfig.maxRetries && retryConfig.retryCondition?.(lastError)

        if (!shouldRetry) {
          break
        }

        // Calculate delay for next attempt
        const delay = calculateDelay(attempt)

        setState(prev => ({
          ...prev,
          lastError,
          canRetry: attempt < retryConfig.maxRetries,
          nextRetryIn: delay,
        }))

        // Wait before retrying
        await new Promise(resolve => {
          retryTimeoutRef.current = setTimeout(resolve, delay)
        })
      }
    }

    // All retries exhausted
    setState(prev => ({
      ...prev,
      isRetrying: false,
      lastError,
      canRetry: false,
      nextRetryIn: 0,
    }))

    return null
  }, [operation, retryConfig, calculateDelay])

  const retry = useCallback(() => {
    if (state.canRetry && !state.isRetrying) {
      return executeWithRetry()
    }
    return Promise.resolve(null)
  }, [state.canRetry, state.isRetrying, executeWithRetry])

  const reset = useCallback(() => {
    setState({
      isRetrying: false,
      retryCount: 0,
      lastError: null,
      canRetry: true,
      nextRetryIn: 0,
    })
  }, [])

  const abort = useCallback(() => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort()
    }
    if (retryTimeoutRef.current) {
      clearTimeout(retryTimeoutRef.current)
    }
    setState(prev => ({
      ...prev,
      isRetrying: false,
      canRetry: false,
    }))
  }, [])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (retryTimeoutRef.current) {
        clearTimeout(retryTimeoutRef.current)
      }
      if (abortControllerRef.current) {
        abortControllerRef.current.abort()
      }
    }
  }, [])

  return {
    execute: executeWithRetry,
    retry,
    reset,
    abort,
    state,
  }
}

// Specialized hook for API calls with error recovery
export function useAPIErrorRecovery<T>(
  apiCall: (signal?: AbortSignal) => Promise<T>,
  config: Partial<RetryConfig> = {},
) {
  const enhancedConfig: Partial<RetryConfig> = {
    ...config,
    retryCondition: (error: Error) => {
      const message = error.message.toLowerCase()
      // Retry on network errors, timeouts, and server errors (5xx)
      return (
        message.includes('network') ||
        message.includes('timeout') ||
        message.includes('fetch') ||
        message.includes('connection') ||
        message.includes('500') ||
        message.includes('502') ||
        message.includes('503') ||
        message.includes('504')
      )
    },
  }

  const recovery = useErrorRecovery(() => apiCall(), enhancedConfig)

  return {
    ...recovery,
    execute: async () => {
      try {
        return await recovery.execute()
      } catch (error) {
        // Log API-specific error
        logAPIError(
          'unknown', // We don't have the endpoint here
          'GET', // Default method
          0, // Unknown status
          undefined,
          { error: (error as Error).message },
        )
        throw error
      }
    },
  }
}

// Hook for handling network connectivity issues
export function useNetworkRecovery() {
  const [isOnline, setIsOnline] = useState(true)
  const [connectionType, setConnectionType] = useState<string>('unknown')

  useEffect(() => {
    const updateOnlineStatus = () => {
      const online = navigator.onLine
      setIsOnline(online)

      if (!online) {
        logNetworkError(
          window.location.href,
          'CONNECTIVITY_CHECK',
          undefined,
          'Network connection lost',
        )
      }
    }

    const updateConnectionType = () => {
      const nav = navigator as Navigator & {
        connection?: { effectiveType?: string }
        mozConnection?: { effectiveType?: string }
        webkitConnection?: { effectiveType?: string }
      }
      const connection = nav.connection || nav.mozConnection || nav.webkitConnection
      if (connection) {
        setConnectionType(connection.effectiveType || 'unknown')
      }
    }

    // Set initial state
    updateOnlineStatus()
    updateConnectionType()

    // Listen for connectivity changes
    window.addEventListener('online', updateOnlineStatus)
    window.addEventListener('offline', updateOnlineStatus)

    // Listen for connection changes (if supported)
    // @ts-expect-error - navigator.connection may not be available in all browsers
    if (navigator.connection) {
      // @ts-expect-error - navigator.connection may not be available in all browsers
      navigator.connection.addEventListener('change', updateConnectionType)
    }

    return () => {
      window.removeEventListener('online', updateOnlineStatus)
      window.removeEventListener('offline', updateOnlineStatus)

      // @ts-expect-error - navigator.connection may not be available in all browsers
      if (navigator.connection) {
        // @ts-expect-error - navigator.connection may not be available in all browsers
        navigator.connection.removeEventListener('change', updateConnectionType)
      }
    }
  }, [])

  const checkConnectivity = useCallback(async (): Promise<boolean> => {
    try {
      // Try to fetch a small resource to verify connectivity
      const response = await fetch('/api/health', {
        method: 'HEAD',
        cache: 'no-cache',
      })
      return response.ok
    } catch {
      logNetworkError('/api/health', 'HEAD', undefined, 'Connectivity check failed')
      return false
    }
  }, [])

  return {
    isOnline,
    connectionType,
    checkConnectivity,
  }
}

// Hook for handling component errors with recovery
export function useComponentErrorRecovery() {
  const [error, setError] = useState<Error | null>(null)
  const [recoveryAttempts, setRecoveryAttempts] = useState(0)
  const maxRecoveryAttempts = 3

  const handleError = useCallback(
    (error: Error, context?: Record<string, unknown>) => {
      logError(error, {
        ...context,
        component: 'unknown',
        recoveryAttempts,
      })

      setError(error)
    },
    [recoveryAttempts],
  )

  const attemptRecovery = useCallback(() => {
    if (recoveryAttempts < maxRecoveryAttempts) {
      setRecoveryAttempts(prev => prev + 1)
      setError(null)

      // Log recovery attempt
      logError(new Error(`Component recovery attempt ${recoveryAttempts + 1}`), {
        type: 'recovery_attempt',
        attemptNumber: recoveryAttempts + 1,
        maxAttempts: maxRecoveryAttempts,
      })
    }
  }, [recoveryAttempts, maxRecoveryAttempts])

  const reset = useCallback(() => {
    setError(null)
    setRecoveryAttempts(0)
  }, [])

  return {
    error,
    recoveryAttempts,
    maxRecoveryAttempts,
    canRecover: recoveryAttempts < maxRecoveryAttempts,
    handleError,
    attemptRecovery,
    reset,
  }
}

// Enhanced WebSocket connection recovery with predictive capabilities
export function useWebSocketErrorRecovery() {
  const [connectionState, setConnectionState] = useState<{
    isConnected: boolean
    reconnectAttempts: number
    lastError: Error | null
    circuitBreakerOpen: boolean
    circuitBreakerTimeout: number
    connectionQuality: 'excellent' | 'good' | 'fair' | 'poor' | 'critical'
    stability: number
    lastSuccessfulConnection: number
    failurePrediction: number
  }>({
    isConnected: false,
    reconnectAttempts: 0,
    lastError: null,
    circuitBreakerOpen: false,
    circuitBreakerTimeout: 0,
    connectionQuality: 'good',
    stability: 1.0,
    lastSuccessfulConnection: Date.now(),
    failurePrediction: 0,
  })

  const maxReconnectAttempts = 15 // Increased for better recovery
  const circuitBreakerThreshold = 7 // Increased threshold
  const circuitBreakerTimeout = 90000 // 1.5 minutes

  // Enhanced recovery operation with connection health checks
  const webSocketRecoveryOperation = useCallback(async (): Promise<void> => {
    // Perform pre-recovery health check
    const isNetworkHealthy = await checkNetworkHealth()

    if (!isNetworkHealthy) {
      console.warn('🌐 Network health check failed - delaying recovery')
      await new Promise(resolve => setTimeout(resolve, 5000)) // Wait 5 seconds
    }

    // The actual reconnection logic is handled by the WebSocket store
    return Promise.resolve()
  }, [])

  // Enhanced retry configuration with adaptive parameters
  const getAdaptiveRecoveryConfig = useCallback((): Partial<RetryConfig> => {
    const { stability, connectionQuality, reconnectAttempts } = connectionState

    // Adaptive base delay based on connection quality
    let baseDelay = 1000
    switch (connectionQuality) {
      case 'excellent':
        baseDelay = 500
        break
      case 'good':
        baseDelay = 1000
        break
      case 'fair':
        baseDelay = 2000
        break
      case 'poor':
        baseDelay = 4000
        break
      case 'critical':
        baseDelay = 8000
        break
    }

    // Adjust based on stability
    if (stability < 0.3) {
      baseDelay *= 1.5
    } else if (stability > 0.8) {
      baseDelay *= 0.7
    }

    return {
      maxRetries: maxReconnectAttempts,
      baseDelay,
      maxDelay: 60000, // 1 minute max
      backoffFactor: stability < 0.5 ? 1.8 : 1.4, // Higher backoff for unstable connections
      retryCondition: (error: Error) => {
        const message = error.message.toLowerCase()
        const errorCode = extractWebSocketErrorCode(error)

        // Enhanced retry conditions with error code analysis
        const retryableErrors = [
          'network', 'timeout', 'connection', 'websocket',
          '1006', '1001', '1008', '1011', // WebSocket close codes
          'ENOTFOUND', 'ECONNREFUSED', 'ETIMEDOUT', // Network errors
        ]

        const isRetryable = retryableErrors.some(keyword => message.includes(keyword)) ||
                           (errorCode >= 1000 && errorCode <= 1015 && errorCode !== 1000)

        // Don't retry on authentication errors or explicit rejections
        const nonRetryableErrors = ['1008', 'authentication', 'unauthorized', 'forbidden']
        const isNonRetryable = nonRetryableErrors.some(keyword => message.includes(keyword))

        return isRetryable && !isNonRetryable && reconnectAttempts < maxReconnectAttempts
      },
    }
  }, [connectionState])

  const recovery = useErrorRecovery(webSocketRecoveryOperation, getAdaptiveRecoveryConfig())

  // Enhanced error handler with predictive analytics
  const handleWebSocketError = useCallback(
    (error: Error, errorType: string, context?: Record<string, unknown>) => {
      const newReconnectAttempts = connectionState.reconnectAttempts + 1
      const shouldOpenCircuitBreaker = newReconnectAttempts >= circuitBreakerThreshold

      // Calculate updated stability and failure prediction
      const timeSinceLastSuccess = Date.now() - connectionState.lastSuccessfulConnection
      const updatedStability = Math.max(0, connectionState.stability - 0.1)
      const updatedFailurePrediction = Math.min(1, connectionState.failurePrediction + 0.15)

      // Update connection quality based on error pattern
      let newQuality = connectionState.connectionQuality
      if (newReconnectAttempts >= 5) {
        newQuality = 'critical'
      } else if (newReconnectAttempts >= 3) {
        newQuality = 'poor'
      } else if (newReconnectAttempts >= 1) {
        newQuality = 'fair'
      }

      logError(error, {
        ...context,
        errorType,
        reconnectAttempts: newReconnectAttempts,
        circuitBreakerOpen: shouldOpenCircuitBreaker,
        stability: updatedStability,
        failurePrediction: updatedFailurePrediction,
        connectionQuality: newQuality,
        timeSinceLastSuccess,
        operation: 'websocket_connection',
      })

      setConnectionState(prev => ({
        ...prev,
        reconnectAttempts: newReconnectAttempts,
        lastError: error,
        circuitBreakerOpen: shouldOpenCircuitBreaker,
        circuitBreakerTimeout: shouldOpenCircuitBreaker ? Date.now() + circuitBreakerTimeout : 0,
        stability: updatedStability,
        failurePrediction: updatedFailurePrediction,
        connectionQuality: newQuality,
      }))

      // Trigger recovery if circuit breaker is not open
      if (!shouldOpenCircuitBreaker && newReconnectAttempts <= maxReconnectAttempts) {
        // Add delay based on failure prediction
        const delay = updatedFailurePrediction > 0.7 ? 2000 : 500
        setTimeout(() => {
          recovery.execute()
        }, delay)
      }
    },
    [connectionState, recovery, circuitBreakerThreshold, circuitBreakerTimeout, maxReconnectAttempts],
  )

  const handleWebSocketSuccess = useCallback(() => {
    const now = Date.now()
    const timeSinceLastSuccess = now - connectionState.lastSuccessfulConnection

    // Calculate improved stability based on recovery time
    const stabilityImprovement = timeSinceLastSuccess < 30000 ? 0.2 : 0.1 // Faster recovery = bigger improvement
    const newStability = Math.min(1, connectionState.stability + stabilityImprovement)

    // Reduce failure prediction on successful connection
    const newFailurePrediction = Math.max(0, connectionState.failurePrediction - 0.3)

    // Improve connection quality
    let newQuality: 'excellent' | 'good' = 'excellent'
    if (connectionState.reconnectAttempts > 0) {
      newQuality = 'good'
    }

    setConnectionState({
      isConnected: true,
      reconnectAttempts: 0,
      lastError: null,
      circuitBreakerOpen: false,
      circuitBreakerTimeout: 0,
      connectionQuality: newQuality,
      stability: newStability,
      lastSuccessfulConnection: now,
      failurePrediction: newFailurePrediction,
    })

    recovery.reset()

    console.warn(`✅ WebSocket recovery successful - stability: ${newStability.toFixed(2)}, quality: ${newQuality}`)
  }, [connectionState, recovery])

  const handleWebSocketDisconnect = useCallback(() => {
    setConnectionState(prev => ({
      ...prev,
      isConnected: false,
      // Slightly reduce stability on disconnect
      stability: Math.max(0, prev.stability - 0.05),
    }))
  }, [])

  const canAttemptReconnection = useCallback(() => {
    const now = Date.now()
    const { circuitBreakerOpen, circuitBreakerTimeout, reconnectAttempts, failurePrediction } = connectionState

    // Check circuit breaker
    if (circuitBreakerOpen && now < circuitBreakerTimeout) {
      return false
    }

    // Check attempt limit
    if (reconnectAttempts >= maxReconnectAttempts) {
      return false
    }

    // Check failure prediction - don't attempt if too risky
    if (failurePrediction > 0.9) {
      return false
    }

    return true
  }, [connectionState])

  const resetCircuitBreaker = useCallback(() => {
    setConnectionState(prev => ({
      ...prev,
      circuitBreakerOpen: false,
      circuitBreakerTimeout: 0,
      reconnectAttempts: 0,
      failurePrediction: Math.max(0, prev.failurePrediction - 0.2), // Reduce failure prediction
    }))
  }, [])

  // Predictive recovery suggestions
  const getRecoverySuggestions = useCallback(() => {
    const { reconnectAttempts, stability, failurePrediction, connectionQuality } = connectionState

    const suggestions: string[] = []

    if (failurePrediction > 0.8) {
      suggestions.push('High failure risk detected - consider manual intervention')
    }

    if (stability < 0.3) {
      suggestions.push('Connection is very unstable - check network conditions')
    }

    if (reconnectAttempts >= circuitBreakerThreshold - 2) {
      suggestions.push('Approaching circuit breaker threshold - prepare for potential disconnection')
    }

    if (connectionQuality === 'critical') {
      suggestions.push('Connection quality is critical - immediate action recommended')
    }

    if (suggestions.length === 0) {
      suggestions.push('Connection appears stable - monitoring continuously')
    }

    return suggestions
  }, [connectionState])

  return {
    connectionState,
    recovery,
    handleWebSocketError,
    handleWebSocketSuccess,
    handleWebSocketDisconnect,
    canAttemptReconnection,
    resetCircuitBreaker,
    getRecoverySuggestions,
    maxReconnectAttempts,
    circuitBreakerThreshold,
  }
}

// Helper function to extract WebSocket error codes
function extractWebSocketErrorCode(error: Error): number {
  const message = error.message.toLowerCase()

  // Try to extract close code from message
  const codeMatch = message.match(/code[:\s]+(\d+)/i)
  if (codeMatch) {
    return parseInt(codeMatch[1], 10)
  }

  return 0
}

// Network health check utility
async function checkNetworkHealth(): Promise<boolean> {
  try {
    const controller = new AbortController()
    const timeoutId = setTimeout(() => controller.abort(), 5000)

    const response = await fetch('/api/health', {
      method: 'HEAD',
      signal: controller.signal,
      cache: 'no-cache',
    })

    clearTimeout(timeoutId)
    return response.ok
  } catch {
    return false
  }
}
