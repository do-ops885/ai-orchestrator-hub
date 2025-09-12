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
      const connection =
        (navigator as any).connection || (navigator as any).mozConnection || (navigator as any).webkitConnection
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
