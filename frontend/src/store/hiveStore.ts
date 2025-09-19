import { create } from 'zustand'

export interface Agent {
  id: string
  name: string
  type: string
  state: string
  capabilities: Array<{
    name: string
    proficiency: number
    learning_rate: number
  }>
  position: [number, number]
  energy: number
  experience_count: number
  social_connections: number
}

interface Message {
  action: string
  payload?: unknown
  id?: string
  sequenceId?: number
  timestamp?: number
}

export interface HiveMetrics {
  total_agents: number
  active_agents: number
  completed_tasks: number
  failed_tasks: number
  average_performance: number
  swarm_cohesion: number
  learning_progress: number
}

export interface Task {
  id: string
  description: string
  type: string
  priority: number
  status: string
  assigned_agent?: string
  created_at: string
  completed_at?: string
  required_capabilities?: Array<{
    name: string
    min_proficiency: number
    weight: number
  }>
}

export interface HiveStatus {
  hive_id: string
  created_at: string
  last_update: string
  metrics: HiveMetrics
  swarm_center: [number, number]
  total_energy: number
}

export type ConnectionErrorType =
  | 'network_unreachable'
  | 'dns_failure'
  | 'tls_handshake'
  | 'server_unavailable'
  | 'protocol_error'
  | 'timeout'
  | 'connection_refused'
  | 'connection_reset'
  | 'unknown'

export type ConnectionState =
  | 'disconnected'
  | 'connecting'
  | 'connected'
  | 'reconnecting'
  | 'degraded'
  | 'failed'
  | 'circuit_breaker_open'

export type ConnectionQuality = 'excellent' | 'good' | 'fair' | 'poor' | 'critical' | 'disconnected'

export interface ConnectionHistoryEntry {
  timestamp: number
  success: boolean
  latency?: number
  errorType?: ConnectionErrorType
  errorMessage?: string
  reconnectDelay?: number
}

export interface Connection {
  socket: WebSocket
  url: string
  priority: number
  lastUsed: number
  usageCount?: number
}

interface HiveStore {
  // Connection state
  isConnected: boolean
  socket: WebSocket | null
  connectionState: ConnectionState
  connectionAttempts: number
  maxReconnectAttempts: number
  reconnectDelay: number
  lastHeartbeat: number
  heartbeatInterval: NodeJS.Timeout | null
  connectionQuality: ConnectionQuality
  backoffMultiplier: number

  // Enhanced heartbeat system
  heartbeatSequence: number
  pendingHeartbeats: Array<{ id: number; timestamp: number; timeoutId: NodeJS.Timeout }>
  heartbeatLatencies: number[]
  heartbeatFailures: number
  lastHeartbeatResponse: number
  heartbeatHealthScore: number

  // Network interruption detection
  networkInterruptionDetected: boolean
  lastNetworkCheck: number
  networkCheckInterval: NodeJS.Timeout | null
  networkOutageStart: number | null
  networkRecoveryAttempts: number
  connectionHistory: ConnectionHistoryEntry[]
  isReconnecting: boolean
  forceReconnectFlag: boolean
  lastErrorType: ConnectionErrorType | null
  consecutiveFailures: number
  circuitBreakerOpen: boolean
  circuitBreakerTimeout: number

  // Fallback and pooling
  fallbackUrls: string[]
  currentUrlIndex: number
  connectionPool: Connection[]
  maxPoolSize: number
  currentUrl: string | null
  multiplexingEnabled: boolean

  // Browser environment awareness
  tabVisible: boolean
  memoryPressure: 'low' | 'medium' | 'high' | 'critical'
  networkType: string
  effectiveConnectionType: string
  browserSupported: boolean

  // Progressive degradation
  degradationLevel: 'normal' | 'reduced' | 'minimal' | 'critical'
  messageFrequency: number
  payloadSizeLimit: number
  heartbeatFrequency: number

  // Timeout handling
  connectionTimeout: number
  messageTimeout: number
  heartbeatTimeout: number
  connectionTimeoutId: NodeJS.Timeout | null
  messageTimeoutId: NodeJS.Timeout | null
  pendingMessages: Array<{
    id: string
    message: Message
    timestamp: number
    timeoutId: NodeJS.Timeout
  }>

  // Monitoring and alerting
  monitoringEnabled: boolean
  alertThresholds: {
    consecutiveFailures: number
    packetLoss: number
    averageLatency: number
    stability: number
  }
  activeAlerts: Array<{
    id: string
    type: string
    message: string
    timestamp: number
    severity: 'low' | 'medium' | 'high' | 'critical'
  }>
  performanceMetrics: {
    totalConnections: number
    totalMessages: number
    totalErrors: number
    uptime: number
    lastReset: number
  }

  // Data
  agents: Agent[]
  hiveStatus: HiveStatus | null
  tasks: Task[]

  // Actions
  classifyWebSocketError: (event: CloseEvent | Event, error?: Event) => ConnectionErrorType
  shouldRetryConnection: (errorType: ConnectionErrorType, attemptCount: number) => boolean
  calculateSmartReconnectDelay: (
    errorType: ConnectionErrorType,
    attemptCount: number,
    stability: number,
  ) => number
  checkConnectionHealth: () => Promise<boolean>
  transitionConnectionState: (newState: ConnectionState, reason?: string) => void
  getConnectionStateDescription: (state: ConnectionState) => string
  shouldAttemptReconnection: () => boolean
  connectWebSocket: (url: string | string[]) => void
  tryNextFallbackUrl: () => string | null
  addFallbackUrl: (url: string) => void
  removeFallbackUrl: (url: string) => void
  getConnectionPoolStatus: () => { active: number; total: number; urls: string[] }
  enableMultiplexing: () => void
  disableMultiplexing: () => void
  addToConnectionPool: (url: string, priority?: number) => Promise<void>
  removeFromConnectionPool: (url: string) => void
  getBestConnection: () => WebSocket | null
  distributeMessage: (message: Message) => void
  initializeBrowserAwareness: () => void
  updateBrowserEnvironment: () => void
  shouldThrottleConnection: () => boolean
  updateDegradationLevel: () => void
  compressMessage: (message: Message) => Message
  shouldSendMessage: () => boolean
  startConnectionTimeout: () => void
  clearConnectionTimeout: () => void
  sendMessageWithTimeout: (message: Message, timeout?: number) => Promise<void>
  handleMessageTimeout: (messageId: string) => void
  handleHeartbeatTimeout: (sequenceId: number) => void
  enableMonitoring: () => void
  disableMonitoring: () => void
  updateAlertThresholds: (
    thresholds: Partial<{
      consecutiveFailures: number
      packetLoss: number
      averageLatency: number
      stability: number
    }>,
  ) => void
  checkAlertConditions: () => void
  createAlert: (
    type: string,
    message: string,
    severity: 'low' | 'medium' | 'high' | 'critical',
  ) => void
  clearAlert: (alertId: string) => void
  getMonitoringReport: () => Record<string, unknown>
  resetPerformanceMetrics: () => void
  disconnect: () => void
  createAgent: (config: unknown) => void
  createTask: (config: unknown) => void
  updateAgents: (agents: Agent[]) => void
  updateHiveStatus: (status: HiveStatus) => void
  updateTasks: (tasks: Task[]) => void
  startHeartbeat: () => void
  stopHeartbeat: () => void
  updateConnectionQuality: () => void
  sendHeartbeat: () => Promise<boolean>
  handleHeartbeatResponse: (sequenceId: number) => void
  calculateHeartbeatHealthScore: () => number
  getHeartbeatStats: () => {
    averageLatency: number
    successRate: number
    healthScore: number
    consecutiveFailures: number
  }
  detectNetworkInterruption: () => Promise<boolean>
  handleNetworkInterruption: () => void
  startNetworkMonitoring: () => void
  stopNetworkMonitoring: () => void
  isTemporaryOutage: () => boolean
  getUserFriendlyErrorMessage: (
    errorType: ConnectionErrorType,
    context?: Record<string, unknown>,
  ) => string
  getRecoverySuggestions: (
    errorType: ConnectionErrorType,
    connectionState: ConnectionState,
  ) => string[]
  getConnectionStatusMessage: () => string
  getActionableAdvice: () => string
  getReliabilityMetrics: () => {
    uptime: number
    availability: number
    meanTimeBetweenFailures: number
    meanTimeToRecovery: number
    failureRate: number
    recoveryRate: number
    networkStability: number
    performanceScore: number
  }
  getDetailedConnectionReport: () => Record<string, unknown>
  getSmartRetryStrategy: (
    errorType: ConnectionErrorType,
    attemptCount: number,
  ) => {
    shouldRetry: boolean
    delay: number
    priority: 'high' | 'medium' | 'low'
    reason: string
  }
  updateCircuitBreakerState: () => void
  getRetryBudgetStatus: () => {
    remainingRetries: number
    budgetResetTime: number
    isBudgetExceeded: boolean
  }
  forceReconnect: () => void
  getConnectionStats: () => {
    stability: number
    averageLatency: number
    successRate: number
    packetLoss: number
    bandwidthEstimate: number
    failurePrediction: number
    jitter: number
    heartbeatHealth: number
  }
  calculateConnectionHealthScore: (connection: Connection) => number
  calculateConnectionLoadFactor: (connection: Connection) => number
  calculateAdaptiveHeartbeatTimeout: () => number
  startPredictiveRecovery: () => void
  stopPredictiveRecovery: () => void
  executePreventiveRecovery: () => void
  prepareBackupConnections: () => void
  sendConnectionTest: () => void
  handleConnectionTestFailure: () => void
  predictConnectionFailure: () => { risk: number; reasons: string[]; recommendedAction: string }
}

export const useHiveStore = create<HiveStore>((set, get) => ({
  isConnected: false,
  socket: null,
  connectionState: 'disconnected' as ConnectionState,
  connectionAttempts: 0,
  maxReconnectAttempts: 10,
  reconnectDelay: 1000,
  lastHeartbeat: Date.now(),
  heartbeatInterval: null,
  connectionQuality: 'disconnected' as ConnectionQuality,
  backoffMultiplier: 1.5,

  // Enhanced heartbeat system
  heartbeatSequence: 0,
  pendingHeartbeats: [],
  heartbeatLatencies: [],
  heartbeatFailures: 0,
  lastHeartbeatResponse: Date.now(),
  heartbeatHealthScore: 1.0,

  // Network interruption detection
  networkInterruptionDetected: false,
  lastNetworkCheck: Date.now(),
  networkCheckInterval: null,
  networkOutageStart: null,
  networkRecoveryAttempts: 0,
  connectionHistory: [],
  isReconnecting: false,
  forceReconnectFlag: false,
  lastErrorType: null,
  consecutiveFailures: 0,
  circuitBreakerOpen: false,
  circuitBreakerTimeout: 0,
  fallbackUrls: [],
  currentUrlIndex: 0,
  connectionPool: [],
  maxPoolSize: 3,
  currentUrl: null,
  multiplexingEnabled: false,
  tabVisible: true,
  memoryPressure: 'low' as const,
  networkType: 'unknown',
  effectiveConnectionType: 'unknown',
  browserSupported: true,
  degradationLevel: 'normal' as const,
  messageFrequency: 1.0, // 1.0 = normal frequency
  payloadSizeLimit: 1000000, // 1MB default
  heartbeatFrequency: 30000, // 30 seconds default
  connectionTimeout: 10000, // 10 seconds
  messageTimeout: 5000, // 5 seconds
  heartbeatTimeout: 30000, // 30 seconds
  connectionTimeoutId: null,
  messageTimeoutId: null,
  pendingMessages: [],
  monitoringEnabled: true,
  alertThresholds: {
    consecutiveFailures: 3,
    packetLoss: 0.5,
    averageLatency: 5000,
    stability: 0.3,
  },
  activeAlerts: [],
  performanceMetrics: {
    totalConnections: 0,
    totalMessages: 0,
    totalErrors: 0,
    uptime: Date.now(),
    lastReset: Date.now(),
  },
  agents: [],

  classifyWebSocketError: (event: CloseEvent | Event, error?: Event): ConnectionErrorType => {
    // Handle CloseEvent
    if (event instanceof CloseEvent) {
      const { code, reason } = event

      // WebSocket close codes provide valuable error information
      switch (code) {
        case 1000: // Normal closure
          return 'unknown'
        case 1001: // Going away
          return 'server_unavailable'
        case 1002: // Protocol error
          return 'protocol_error'
        case 1003: // Unsupported data
          return 'protocol_error'
        case 1004: // Reserved
          return 'protocol_error'
        case 1005: // No status received
          return 'server_unavailable'
        case 1006: // Abnormal closure
          return 'connection_reset'
        case 1007: // Invalid frame payload data
          return 'protocol_error'
        case 1008: // Policy violation
          return 'protocol_error'
        case 1009: // Message too big
          return 'protocol_error'
        case 1010: // Missing extension
          return 'protocol_error'
        case 1011: // Internal error
          return 'server_unavailable'
        case 1012: // Service restart
          return 'server_unavailable'
        case 1013: // Try again later
          return 'server_unavailable'
        case 1014: // Bad gateway
          return 'server_unavailable'
        case 1015: // TLS handshake
          return 'tls_handshake'
        default:
          // Check reason string for additional clues
          const reasonStr = reason.toLowerCase()
          if (reasonStr.includes('network') || reasonStr.includes('unreachable')) {
            return 'network_unreachable'
          }
          if (reasonStr.includes('dns') || reasonStr.includes('name resolution')) {
            return 'dns_failure'
          }
          if (reasonStr.includes('timeout')) {
            return 'timeout'
          }
          if (reasonStr.includes('refused') || reasonStr.includes('connection refused')) {
            return 'connection_refused'
          }
          return 'unknown'
      }
    }

    // Handle generic Error events
    if (error instanceof Event) {
      // Try to infer error type from event type or target
      const target = error.target as WebSocket | null
      if (target && target.constructor.name === 'WebSocket') {
        // WebSocket-specific error
        return 'connection_reset'
      }
    }

    return 'unknown'
  },

  shouldRetryConnection: (errorType: ConnectionErrorType, attemptCount: number): boolean => {
    const { maxReconnectAttempts, circuitBreakerOpen } = get()

    // Don't retry if circuit breaker is open
    if (circuitBreakerOpen) {
      return false
    }

    // Don't retry beyond max attempts
    if (attemptCount >= maxReconnectAttempts) {
      return false
    }

    // Retry strategy based on error type
    switch (errorType) {
      case 'network_unreachable':
      case 'dns_failure':
      case 'server_unavailable':
        // These are typically transient and worth retrying
        return attemptCount < maxReconnectAttempts
      case 'tls_handshake':
        // TLS issues might be persistent, but retry a few times
        return attemptCount < 3
      case 'protocol_error':
        // Protocol errors are usually persistent, retry less aggressively
        return attemptCount < 2
      case 'timeout':
        // Timeouts are often transient
        return attemptCount < maxReconnectAttempts
      case 'connection_refused':
        // Connection refused might indicate server issues
        return attemptCount < 5
      case 'connection_reset':
        // Connection resets are often recoverable
        return attemptCount < maxReconnectAttempts
      case 'unknown':
      default:
        // Unknown errors - retry with caution
        return attemptCount < Math.min(maxReconnectAttempts, 5)
    }
  },

  calculateSmartReconnectDelay: (
    errorType: ConnectionErrorType,
    attemptCount: number,
    stability: number,
  ): number => {
    const { reconnectDelay, backoffMultiplier, maxReconnectAttempts, connectionHistory } = get()
    const baseDelay = reconnectDelay

    // Analyze connection history for adaptive behavior with enhanced metrics
    const recentHistory = connectionHistory.slice(-15) // Last 15 attempts for better analysis
    const successfulAttempts = recentHistory.filter(h => h.success)
    const recentSuccessRate =
      recentHistory.length > 0 ? successfulAttempts.length / recentHistory.length : 0.5

    // Calculate historical average delay for successful reconnections
    const successfulDelays = successfulAttempts
      .filter(h => h.reconnectDelay && h.reconnectDelay > 0)
      .map(h => h.reconnectDelay as number)
    const avgSuccessfulDelay =
      successfulDelays.length > 0
        ? successfulDelays.reduce((sum, delay) => sum + delay, 0) / successfulDelays.length
        : baseDelay

    // Enhanced jitter implementation with multiple strategies
    const previousDelay =
      attemptCount > 0
        ? connectionHistory.slice(-1)[0]?.reconnectDelay || avgSuccessfulDelay || baseDelay
        : baseDelay

    // Calculate exponential backoff with enhanced formula
    const exponentialDelay = baseDelay * Math.pow(backoffMultiplier, attemptCount)

    // Dynamic cap based on connection quality and error patterns
    const qualityMultiplier =
      get().connectionQuality === 'excellent'
        ? 0.5
        : get().connectionQuality === 'good'
          ? 0.7
          : get().connectionQuality === 'poor'
            ? 1.5
            : get().connectionQuality === 'critical'
              ? 2.0
              : 1.0

    const cap = Math.min(
      previousDelay * 2.5,
      exponentialDelay * qualityMultiplier,
      45000, // Max 45 seconds for better recovery
    )

    // Advanced jitter strategies based on attempt count and connection state
    let delay: number
    if (attemptCount === 0) {
      // First attempt: minimal jitter for fast recovery
      delay = Math.random() * Math.min(cap * 0.3, exponentialDelay * 0.5)
    } else if (attemptCount < 3) {
      // Early attempts: full jitter with controlled range
      const jitterRange = Math.min(cap, exponentialDelay)
      delay = Math.random() * jitterRange * 0.8 + jitterRange * 0.1 // 10%-90% of range
    } else if (attemptCount < 6) {
      // Mid attempts: decorrelated jitter for stability
      const minDelay = Math.max(baseDelay, exponentialDelay * 0.2)
      const maxDelay = Math.min(cap, previousDelay * 2.5)
      delay = minDelay + Math.random() * (maxDelay - minDelay)
    } else {
      // Late attempts: equal jitter for maximum distribution
      const base = Math.max(baseDelay * 2, exponentialDelay * 0.3)
      const range = Math.min(cap - base, previousDelay * 1.5)
      delay = base + Math.random() * range
    }

    // Enhanced error type multipliers with predictive adjustments
    let multiplier = 1.0
    const errorFrequency =
      recentHistory.filter(h => h.errorType === errorType).length / recentHistory.length

    switch (errorType) {
      case 'network_unreachable':
      case 'dns_failure':
        multiplier = errorFrequency > 0.5 ? 2.2 : 1.8 // Increase if frequent
        break
      case 'server_unavailable':
        multiplier = errorFrequency > 0.3 ? 1.8 : 1.4
        break
      case 'tls_handshake':
        multiplier = errorFrequency > 0.2 ? 3.0 : 2.5 // TLS issues are often persistent
        break
      case 'protocol_error':
        multiplier = errorFrequency > 0.1 ? 3.5 : 3.0 // Protocol errors rarely resolve quickly
        break
      case 'timeout':
        multiplier = errorFrequency > 0.4 ? 1.0 : 0.6 // Timeouts might be transient
        break
      case 'connection_refused':
        multiplier = errorFrequency > 0.3 ? 2.8 : 2.2
        break
      case 'connection_reset':
        multiplier = errorFrequency > 0.4 ? 2.0 : 1.6
        break
      default:
        multiplier = 1.0
        break
    }

    // Advanced stability adjustment with trend analysis
    const stabilityTrend =
      recentHistory.length >= 5
        ? recentHistory.slice(-3).filter(h => h.success).length / 3 -
          recentHistory.slice(-5, -3).filter(h => h.success).length / 2
        : 0

    if (stability < 0.15) {
      multiplier *= 2.5 // Very unstable - significantly longer delays
    } else if (stability < 0.3) {
      multiplier *= stabilityTrend < -0.2 ? 2.0 : 1.8 // Unstable with worsening trend
    } else if (stability < 0.5) {
      multiplier *= 1.5 // Moderately unstable
    } else if (stability > 0.9) {
      multiplier *= stabilityTrend > 0.1 ? 0.4 : 0.5 // Very stable with improving trend
    } else if (stability > 0.75) {
      multiplier *= 0.7 // Stable
    }

    // Enhanced success rate adjustment with momentum
    if (recentSuccessRate < 0.2) {
      multiplier *= 2.2 // Very poor success rate
    } else if (recentSuccessRate < 0.4) {
      multiplier *= recentSuccessRate < 0.3 ? 1.9 : 1.7 // Poor success rate
    } else if (recentSuccessRate > 0.85) {
      multiplier *= 0.5 // Excellent success rate - fast recovery
    } else if (recentSuccessRate > 0.7) {
      multiplier *= 0.7 // Good success rate
    }

    // Time-of-day and network condition adjustments
    const hour = new Date().getHours()
    const isPeakHour = (hour >= 9 && hour <= 17) || (hour >= 19 && hour <= 22)
    if (isPeakHour) {
      multiplier *= 1.3 // Longer delays during peak hours
    }

    // Network type adjustment
    const { effectiveConnectionType } = get()
    if (effectiveConnectionType === 'slow-2g' || effectiveConnectionType === '2g') {
      multiplier *= 2.0 // Much longer delays on slow connections
    } else if (effectiveConnectionType === '3g') {
      multiplier *= 1.5 // Longer delays on 3G
    }

    delay *= multiplier

    // Adaptive minimum delay with error pattern learning
    let minDelay = 300 // Reduced base minimum for faster recovery
    switch (errorType) {
      case 'timeout':
      case 'connection_reset':
        minDelay = recentSuccessRate > 0.6 ? 800 : 1200 // Adaptive based on success rate
        break
      case 'network_unreachable':
      case 'dns_failure':
        minDelay = recentSuccessRate > 0.5 ? 1500 : 2500
        break
      case 'tls_handshake':
      case 'protocol_error':
        minDelay = errorFrequency < 0.3 ? 3000 : 6000 // Longer for persistent issues
        break
      case 'connection_refused':
        minDelay = 2000
        break
      default:
        minDelay = 500
        break
    }

    // Historical learning: adjust minimum based on successful patterns
    if (successfulDelays.length > 3) {
      const medianSuccessfulDelay = successfulDelays.sort((a, b) => a - b)[
        Math.floor(successfulDelays.length / 2)
      ]
      const historicalMin = medianSuccessfulDelay * 0.6
      minDelay = Math.max(minDelay, Math.min(historicalMin, 2000)) // Cap historical influence
    }

    // Dynamic maximum delay based on connection quality and error persistence
    const maxReconnectDelay = Math.min(
      90000, // Hard cap at 1.5 minutes for better user experience
      baseDelay *
        Math.pow(backoffMultiplier, Math.min(maxReconnectAttempts, 8)) *
        qualityMultiplier,
    )

    delay = Math.min(delay, maxReconnectDelay)
    delay = Math.max(delay, minDelay)

    // Round to nearest 50ms for cleaner logging and more precise timing
    const roundedDelay = Math.round(delay / 50) * 50

    console.warn(
      `🔄 Enhanced reconnect delay: ${roundedDelay}ms (attempt: ${attemptCount}, error: ${errorType}, stability: ${(stability * 100).toFixed(1)}%, success: ${(recentSuccessRate * 100).toFixed(1)}%, quality: ${get().connectionQuality})`,
    )

    return roundedDelay
  },

  checkConnectionHealth: async (): Promise<boolean> => {
    try {
      // Quick connectivity check using a lightweight endpoint
      const controller = new AbortController()
      const timeoutId = setTimeout(() => controller.abort(), 5000) // 5 second timeout

      const response = await fetch('/api/health', {
        method: 'HEAD',
        signal: controller.signal,
        cache: 'no-cache',
        headers: {
          'Cache-Control': 'no-cache',
          Pragma: 'no-cache',
        },
      })

      clearTimeout(timeoutId)
      return response.ok
    } catch (error) {
      console.warn('Connection health check failed:', error)
      return false
    }
  },

  transitionConnectionState: (newState: ConnectionState, reason?: string): void => {
    const { connectionState: currentState } = get()

    // Validate state transitions
    const validTransitions: Record<ConnectionState, ConnectionState[]> = {
      disconnected: ['connecting', 'circuit_breaker_open'],
      connecting: ['connected', 'failed', 'disconnected'],
      connected: ['degraded', 'reconnecting', 'disconnected'],
      reconnecting: ['connected', 'failed', 'disconnected'],
      degraded: ['connected', 'reconnecting', 'failed', 'disconnected'],
      failed: ['reconnecting', 'disconnected', 'circuit_breaker_open'],
      circuit_breaker_open: ['disconnected'],
    }

    if (!validTransitions[currentState]?.includes(newState)) {
      console.warn(`⚠️ Invalid state transition: ${currentState} -> ${newState}`)
      return
    }

    // Update connection state
    set({ connectionState: newState })

    // Update derived boolean state
    const isConnected = ['connected', 'degraded'].includes(newState)
    set({ isConnected })

    // Log state transition
    const reasonText = reason ? ` (${reason})` : ''
    console.warn(`🔄 Connection state: ${currentState} -> ${newState}${reasonText}`)

    // Trigger side effects based on new state
    switch (newState) {
      case 'connected':
        get().updateConnectionQuality()
        get().updateDegradationLevel()
        break
      case 'degraded':
        get().updateDegradationLevel()
        break
      case 'failed':
      case 'disconnected':
        set({ connectionQuality: 'disconnected' })
        break
      case 'circuit_breaker_open':
        console.warn('🚫 Circuit breaker opened - stopping reconnection attempts')
        break
    }
  },

  getConnectionStateDescription: (state: ConnectionState): string => {
    switch (state) {
      case 'disconnected':
        return 'Not connected to server'
      case 'connecting':
        return 'Establishing connection...'
      case 'connected':
        return 'Connected and operational'
      case 'reconnecting':
        return 'Reconnecting to server...'
      case 'degraded':
        return 'Connected with performance issues'
      case 'failed':
        return 'Connection failed'
      case 'circuit_breaker_open':
        return 'Circuit breaker active - temporarily stopped reconnection attempts'
      default:
        return 'Unknown connection state'
    }
  },

  shouldAttemptReconnection: (): boolean => {
    const { connectionState, circuitBreakerOpen, maxReconnectAttempts, connectionAttempts } = get()

    // Don't attempt if circuit breaker is open
    if (circuitBreakerOpen || connectionState === 'circuit_breaker_open') {
      return false
    }

    // Don't attempt if we've exceeded max attempts
    if (connectionAttempts >= maxReconnectAttempts) {
      return false
    }

    // Don't attempt if already connecting or connected
    if (['connecting', 'connected', 'reconnecting'].includes(connectionState)) {
      return false
    }

    // Don't attempt if connection is being throttled
    if (get().shouldThrottleConnection()) {
      return false
    }

    return true
  },

  tryNextFallbackUrl: (): string | null => {
    const { fallbackUrls, currentUrlIndex } = get()

    if (fallbackUrls.length === 0) {
      return null
    }

    const nextIndex = (currentUrlIndex + 1) % fallbackUrls.length
    set({ currentUrlIndex: nextIndex })

    return fallbackUrls[nextIndex] || null
  },

  addFallbackUrl: (url: string): void => {
    const { fallbackUrls } = get()

    if (!fallbackUrls.includes(url)) {
      set({ fallbackUrls: [...fallbackUrls, url] })
      console.warn(`✅ Added fallback URL: ${url}`)
    }
  },

  removeFallbackUrl: (url: string): void => {
    const { fallbackUrls } = get()
    const filteredUrls = fallbackUrls.filter(u => u !== url)

    set({ fallbackUrls: filteredUrls })
    console.warn(`❌ Removed fallback URL: ${url}`)
  },

  getConnectionPoolStatus: () => {
    const { connectionPool, fallbackUrls } = get()
    return {
      active: connectionPool.filter(conn => conn.socket.readyState === WebSocket.OPEN).length,
      total: connectionPool.length,
      urls: fallbackUrls,
    }
  },

  enableMultiplexing: () => {
    console.warn('🔄 Enabling connection multiplexing')
    set({ multiplexingEnabled: true })

    // Initialize connection pool with available URLs
    const { fallbackUrls } = get()
    fallbackUrls.forEach((url, index) => {
      get().addToConnectionPool(url, index + 1)
    })
  },

  disableMultiplexing: () => {
    console.warn('🔄 Disabling connection multiplexing')
    set({ multiplexingEnabled: false })

    // Close all connections in pool except the primary one
    const { connectionPool, socket } = get()
    connectionPool.forEach(conn => {
      if (conn.socket !== socket) {
        conn.socket.close(1000, 'Multiplexing disabled')
      }
    })

    set({ connectionPool: [] })
  },

  addToConnectionPool: async (url: string, priority = 1): Promise<void> => {
    const { connectionPool, maxPoolSize, multiplexingEnabled } = get()

    if (!multiplexingEnabled) {
      console.warn('Multiplexing is disabled')
      return
    }

    if (connectionPool.length >= maxPoolSize) {
      console.warn('Connection pool is at maximum capacity')
      return
    }

    // Check if URL already exists in pool
    if (connectionPool.some(conn => conn.url === url)) {
      console.warn(`URL ${url} already exists in connection pool`)
      return
    }

    try {
      const socket = new WebSocket(url)

      return new Promise((resolve, reject) => {
        const timeout = setTimeout(() => {
          socket.close()
          reject(new Error('Connection timeout'))
        }, 10000)

        socket.onopen = () => {
          clearTimeout(timeout)
          const newConnection = {
            socket,
            url,
            priority,
            lastUsed: Date.now(),
          }

          set({ connectionPool: [...connectionPool, newConnection] })
          console.warn(`✅ Added connection to pool: ${url} (priority: ${priority})`)
          resolve()
        }

        socket.onerror = error => {
          clearTimeout(timeout)
          console.warn(`❌ Failed to add connection to pool: ${url}`, error)
          reject(error)
        }

        socket.onclose = () => {
          clearTimeout(timeout)
          // Remove from pool if connection closes
          const updatedPool = connectionPool.filter(conn => conn.url !== url)
          set({ connectionPool: updatedPool })
        }
      })
    } catch (error) {
      console.warn(`❌ Failed to create connection for pool: ${url}`, error)
    }
  },

  removeFromConnectionPool: (url: string) => {
    const { connectionPool } = get()
    const connection = connectionPool.find(conn => conn.url === url)

    if (connection) {
      connection.socket.close(1000, 'Removed from pool')
      const updatedPool = connectionPool.filter(conn => conn.url !== url)
      set({ connectionPool: updatedPool })
      console.warn(`❌ Removed connection from pool: ${url}`)
    }
  },

  getBestConnection: (): WebSocket | null => {
    const { connectionPool, multiplexingEnabled } = get()

    if (!multiplexingEnabled) {
      return get().socket
    }

    // Get available connections with enhanced metrics
    const availableConnections = connectionPool
      .filter(conn => conn.socket.readyState === WebSocket.OPEN)
      .map(conn => ({
        ...conn,
        // Calculate connection health score
        healthScore: get().calculateConnectionHealthScore(conn),
        // Calculate load factor (lower is better)
        loadFactor: get().calculateConnectionLoadFactor(conn),
      }))
      .sort((a, b) => {
        // Primary sort: health score (higher is better)
        if (Math.abs(a.healthScore - b.healthScore) > 0.1) {
          return b.healthScore - a.healthScore
        }
        // Secondary sort: load factor (lower is better)
        if (Math.abs(a.loadFactor - b.loadFactor) > 0.1) {
          return a.loadFactor - b.loadFactor
        }
        // Tertiary sort: priority (higher is better)
        if (a.priority !== b.priority) {
          return b.priority - a.priority
        }
        // Final sort: least recently used
        return a.lastUsed - b.lastUsed
      })

    if (availableConnections.length > 0) {
      const [bestConnection] = availableConnections
      // Update last used time and usage metrics
      const poolIndex = connectionPool.findIndex(conn => conn.socket === bestConnection.socket)
      if (poolIndex !== -1) {
        connectionPool[poolIndex].lastUsed = Date.now()
        // Track usage for load balancing
        connectionPool[poolIndex].usageCount = (connectionPool[poolIndex].usageCount || 0) + 1
      }

      console.warn(
        `🔄 Selected connection from pool: priority ${bestConnection.priority}, health ${bestConnection.healthScore.toFixed(2)}, load ${bestConnection.loadFactor.toFixed(2)}`,
      )
      return bestConnection.socket
    }

    // Fallback to primary connection
    return get().socket
  },

  calculateConnectionHealthScore: (connection: Connection): number => {
    const { socket } = connection
    if (!socket || socket.readyState !== WebSocket.OPEN) {
      return 0
    }

    let score = 1.0

    // Connection age factor (slightly prefer newer connections for freshness)
    const age = Date.now() - connection.lastUsed
    if (age > 300000) {
      // 5 minutes
      score *= 0.9
    } else if (age > 60000) {
      // 1 minute
      score *= 0.95
    }

    // Priority factor
    score *= 0.8 + connection.priority * 0.2 // 0.8 to 1.0 based on priority

    // Usage balance factor (prefer less used connections)
    const totalUsage = get().connectionPool.reduce((sum, conn) => sum + (conn.usageCount || 0), 0)
    const usageRatio = totalUsage > 0 ? (connection.usageCount || 0) / totalUsage : 0
    score *= 1.0 - usageRatio * 0.3 // Reduce score by up to 30% for heavily used connections

    return Math.max(0, Math.min(1, score))
  },

  calculateConnectionLoadFactor: (connection: Connection): number => {
    // Calculate load based on recent usage patterns
    const now = Date.now()
    const recentUsage = connection.usageCount || 0
    const timeSinceLastUse = now - connection.lastUsed

    // Higher load factor for recently used connections
    const recencyFactor = Math.max(0, 1 - timeSinceLastUse / 60000) // Decay over 1 minute
    const usageFactor = Math.min(1, recentUsage / 10) // Cap at 10 uses

    return recencyFactor * 0.6 + usageFactor * 0.4
  },

  distributeMessage: (message: Message) => {
    const { multiplexingEnabled } = get()

    if (!multiplexingEnabled) {
      // Use single connection
      const { socket } = get()
      if (socket && socket.readyState === WebSocket.OPEN) {
        socket.send(JSON.stringify(message))
      }
      return
    }

    // Distribute across multiple connections
    const bestConnection = get().getBestConnection()
    if (bestConnection) {
      try {
        bestConnection.send(JSON.stringify(message))
      } catch (error) {
        console.warn('Failed to send message via best connection:', error)
        // Fallback to primary connection
        const { socket } = get()
        if (socket && socket.readyState === WebSocket.OPEN) {
          socket.send(JSON.stringify(message))
        }
      }
    }
  },

  initializeBrowserAwareness: () => {
    // Check browser WebSocket support
    const hasWebSocketSupport = typeof WebSocket !== 'undefined'
    const hasBinarySupport = typeof ArrayBuffer !== 'undefined'

    set({ browserSupported: hasWebSocketSupport && hasBinarySupport })

    if (!hasWebSocketSupport) {
      console.warn('⚠️ WebSocket not supported in this browser')
    }

    // Set up visibility change listener
    const handleVisibilityChange = () => {
      const isVisible = !document.hidden
      const wasVisible = get().tabVisible

      set({ tabVisible: isVisible })

      // Handle tab becoming visible
      if (isVisible && !wasVisible) {
        console.warn('📱 Tab became visible, checking connection...')
        const { isConnected, socket } = get()
        if (!isConnected || (socket && socket.readyState !== WebSocket.OPEN)) {
          // Attempt to reconnect if connection was lost while tab was hidden
          setTimeout(() => {
            const { currentUrl } = get()
            if (currentUrl) {
              console.warn('🔄 Reconnecting due to tab visibility change')
              get().connectWebSocket(currentUrl)
            }
          }, 1000)
        }
      }

      // Handle tab becoming hidden
      if (!isVisible && wasVisible) {
        console.warn('📱 Tab became hidden, connection may be suspended')
      }
    }

    document.addEventListener('visibilitychange', handleVisibilityChange)

    // Set up memory pressure listener (if supported)
    if ('memory' in performance) {
      const handleMemoryPressure = () => {
        const { memory } = performance as unknown as {
          memory: { usedJSHeapSize: number; totalJSHeapSize: number }
        }
        const usedPercent = (memory.usedJSHeapSize / memory.totalJSHeapSize) * 100

        let pressure: 'low' | 'medium' | 'high' | 'critical'
        if (usedPercent > 90) {
          pressure = 'critical'
        } else if (usedPercent > 75) {
          pressure = 'high'
        } else if (usedPercent > 50) {
          pressure = 'medium'
        } else {
          pressure = 'low'
        }

        set({ memoryPressure: pressure })

        if (pressure === 'critical') {
          console.warn('🚨 Critical memory pressure detected, reducing connection activity')
          // Could implement connection throttling here
        }
      }

      // Check memory pressure periodically
      const memoryCheckInterval = setInterval(handleMemoryPressure, 30000) // Check every 30 seconds

      // Store cleanup function
      ;(get() as unknown as Record<string, unknown>)._memoryCheckInterval = memoryCheckInterval
    }

    // Set up network change listener (if supported)
    if ('connection' in navigator) {
      const { connection } = navigator as {
        connection: {
          type: string
          effectiveType: string
          addEventListener: (type: string, listener: () => void) => void
        }
      }

      const handleNetworkChange = () => {
        set({
          networkType: connection.type || 'unknown',
          effectiveConnectionType: connection.effectiveType || 'unknown',
        })

        console.warn(`📡 Network changed: ${connection.effectiveType} (${connection.type})`)

        // Adjust connection strategy based on network type
        if (connection.effectiveType === 'slow-2g' || connection.effectiveType === '2g') {
          console.warn('🐌 Slow network detected, enabling connection throttling')
        }
      }

      connection.addEventListener('change', handleNetworkChange)

      // Set initial values
      handleNetworkChange()
    }

    // Initial environment update
    get().updateBrowserEnvironment()

    // Start predictive recovery monitoring
    get().startPredictiveRecovery()

    console.warn('✅ Browser environment awareness and predictive recovery initialized')
  },

  updateBrowserEnvironment: () => {
    const { tabVisible, memoryPressure, networkType } = get()

    // Update connection quality based on environment
    let qualityAdjustment = 1.0

    if (!tabVisible) {
      qualityAdjustment *= 0.7 // Reduce quality when tab is hidden
    }

    if (memoryPressure === 'high') {
      qualityAdjustment *= 0.8
    } else if (memoryPressure === 'critical') {
      qualityAdjustment *= 0.5
    }

    if (networkType === 'cellular' || networkType.includes('2g') || networkType.includes('slow')) {
      qualityAdjustment *= 0.6
    }

    // Update connection quality if connected
    const { isConnected } = get()
    if (isConnected) {
      get().updateConnectionQuality()
    }

    console.warn(
      `🌍 Environment updated - Tab visible: ${tabVisible}, Memory: ${memoryPressure}, Network: ${networkType}, Quality adjustment: ${qualityAdjustment}`,
    )
  },

  shouldThrottleConnection: () => {
    const { tabVisible, memoryPressure, effectiveConnectionType } = get()

    // Throttle if tab is hidden
    if (!tabVisible) {
      return true
    }

    // Throttle on high memory pressure
    if (memoryPressure === 'high' || memoryPressure === 'critical') {
      return true
    }

    // Throttle on slow connections
    if (effectiveConnectionType === 'slow-2g' || effectiveConnectionType === '2g') {
      return true
    }

    return false
  },

  updateDegradationLevel: () => {
    const { connectionQuality, memoryPressure, effectiveConnectionType, consecutiveFailures } =
      get()

    let newLevel: 'normal' | 'reduced' | 'minimal' | 'critical' = 'normal'
    let messageFrequency = 1.0
    let payloadSizeLimit = 1000000 // 1MB
    let heartbeatFrequency = 30000 // 30 seconds

    // Determine degradation level based on multiple factors
    const qualityScore =
      {
        excellent: 1.0,
        good: 0.9,
        fair: 0.7,
        poor: 0.5,
        critical: 0.3,
        disconnected: 0.0,
      }[connectionQuality] || 0.5

    const memoryScore =
      {
        low: 1.0,
        medium: 0.8,
        high: 0.6,
        critical: 0.3,
      }[memoryPressure] || 0.5

    const networkScore =
      {
        '4g': 1.0,
        '3g': 0.8,
        '2g': 0.6,
        'slow-2g': 0.4,
        unknown: 0.7,
      }[effectiveConnectionType] || 0.7

    const failurePenalty = Math.max(0, consecutiveFailures * 0.1)
    const overallScore = (qualityScore + memoryScore + networkScore) / 3 - failurePenalty

    if (overallScore < 0.4) {
      newLevel = 'critical'
      messageFrequency = 0.1 // 10% of normal frequency
      payloadSizeLimit = 10000 // 10KB
      heartbeatFrequency = 120000 // 2 minutes
    } else if (overallScore < 0.6) {
      newLevel = 'minimal'
      messageFrequency = 0.3 // 30% of normal frequency
      payloadSizeLimit = 50000 // 50KB
      heartbeatFrequency = 90000 // 1.5 minutes
    } else if (overallScore < 0.8) {
      newLevel = 'reduced'
      messageFrequency = 0.6 // 60% of normal frequency
      payloadSizeLimit = 200000 // 200KB
      heartbeatFrequency = 60000 // 1 minute
    }

    set({
      degradationLevel: newLevel,
      messageFrequency,
      payloadSizeLimit,
      heartbeatFrequency,
    })

    if (newLevel !== 'normal') {
      console.warn(
        `📉 Connection degradation: ${newLevel} (score: ${overallScore.toFixed(2)}, frequency: ${messageFrequency}, limit: ${payloadSizeLimit} bytes)`,
      )
    } else {
      console.warn('✅ Connection quality normal')
    }
  },

  compressMessage: (message: Message): Message => {
    const { degradationLevel, payloadSizeLimit } = get()

    // Skip compression for critical messages
    if (message.action === 'ping' || message.action === 'force_reconnect') {
      return message
    }

    // Apply compression based on degradation level
    switch (degradationLevel) {
      case 'critical':
        // Minimal payload - only essential data
        if (message.action === 'create_agent' || message.action === 'create_task') {
          return {
            action: message.action,
            payload: {
              type: (message.payload as { type?: string } | undefined)?.type || 'unknown',
            }, // Minimal payload
          }
        }
        break

      case 'minimal':
        // Reduce payload size
        if (message.payload && JSON.stringify(message.payload).length > payloadSizeLimit) {
          return {
            ...message,
            payload: { ...message.payload, _compressed: true }, // Mark as compressed
          }
        }
        break

      case 'reduced':
        // Moderate compression
        if (message.payload && typeof message.payload === 'object') {
          const compressed = { ...(message.payload as Record<string, unknown>) }
          // Remove non-essential fields
          delete compressed.metadata
          delete compressed.debug
          return { ...message, payload: compressed }
        }
        break

      default:
        // Normal - no compression
        break
    }

    return message
  },

  shouldSendMessage: (): boolean => {
    const { messageFrequency, degradationLevel } = get()

    if (degradationLevel === 'normal') {
      return true
    }

    // Apply frequency throttling
    const random = Math.random()
    return random <= messageFrequency
  },

  startConnectionTimeout: () => {
    const { connectionTimeoutId, connectionTimeout } = get()
    if (connectionTimeoutId) {
      clearTimeout(connectionTimeoutId)
    }

    const timeoutId = setTimeout(() => {
      console.warn('⏰ Connection timeout - no response from server')
      const { socket } = get()
      if (socket && socket.readyState === WebSocket.CONNECTING) {
        // Connection is still attempting, force close
        socket.close(1006, 'Connection timeout')
      }
    }, connectionTimeout)

    set({ connectionTimeoutId: timeoutId })
  },

  clearConnectionTimeout: () => {
    const { connectionTimeoutId } = get()
    if (connectionTimeoutId) {
      clearTimeout(connectionTimeoutId)
      set({ connectionTimeoutId: null })
    }
  },

  sendMessageWithTimeout: async (message: Message, timeout?: number): Promise<void> => {
    const { socket, messageTimeout, pendingMessages } = get()

    if (!socket || socket.readyState !== WebSocket.OPEN) {
      throw new Error('WebSocket is not connected')
    }

    const messageId = `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
    const actualTimeout = timeout || messageTimeout

    return new Promise((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        get().handleMessageTimeout(messageId)
        reject(new Error(`Message timeout after ${actualTimeout}ms`))
      }, actualTimeout)

      // Add to pending messages
      const pendingMessage = {
        id: messageId,
        message,
        timestamp: Date.now(),
        timeoutId,
      }

      set({ pendingMessages: [...pendingMessages, pendingMessage] })

      try {
        socket.send(JSON.stringify(message))
        resolve()
      } catch (error) {
        clearTimeout(timeoutId)
        // Remove from pending messages
        const updatedPending = pendingMessages.filter(m => m.id !== messageId)
        set({ pendingMessages: updatedPending })
        reject(error)
      }
    })
  },

  handleMessageTimeout: (messageId: string) => {
    const { pendingMessages } = get()
    const messageIndex = pendingMessages.findIndex(m => m.id === messageId)

    if (messageIndex !== -1) {
      const message = pendingMessages[messageIndex]
      console.warn(`⏰ Message timeout: ${JSON.stringify(message.message)}`)

      // Remove from pending messages
      const updatedPending = pendingMessages.filter(m => m.id !== messageId)
      set({ pendingMessages: updatedPending })

      // Clear the timeout
      clearTimeout(message.timeoutId)

      // Could implement retry logic here based on message type
      const { degradationLevel } = get()
      if (degradationLevel !== 'critical' && message.message.action !== 'ping') {
        console.warn('🔄 Message timeout - could retry based on degradation level')
      }
    }
  },

  enableMonitoring: () => {
    console.warn('📊 Enabling connection monitoring')
    set({ monitoringEnabled: true })

    // Start periodic monitoring
    const monitoringInterval = setInterval(() => {
      get().checkAlertConditions()
    }, 30000) // Check every 30 seconds

    // Store the interval for cleanup
    ;(get() as unknown as Record<string, unknown>)._monitoringInterval = monitoringInterval
  },

  disableMonitoring: () => {
    console.warn('📊 Disabling connection monitoring')
    set({ monitoringEnabled: false })

    // Clear monitoring interval
    const monitoringInterval = (get() as unknown as Record<string, unknown>)._monitoringInterval as number
    if (monitoringInterval) {
      clearInterval(monitoringInterval)
    }
  },

  updateAlertThresholds: (
    thresholds: Partial<{
      consecutiveFailures: number
      packetLoss: number
      averageLatency: number
      stability: number
    }>,
  ) => {
    const { alertThresholds } = get()
    set({ alertThresholds: { ...alertThresholds, ...thresholds } })
    console.warn('📊 Updated alert thresholds:', thresholds)
  },

  checkAlertConditions: () => {
    const { monitoringEnabled, alertThresholds, consecutiveFailures, activeAlerts } = get()

    if (!monitoringEnabled) {
      return
    }

    const stats = get().getConnectionStats()

    // Check consecutive failures
    if (consecutiveFailures >= alertThresholds.consecutiveFailures) {
      const severity =
        consecutiveFailures >= alertThresholds.consecutiveFailures * 2 ? 'critical' : 'high'
      get().createAlert(
        'consecutive_failures',
        `High number of consecutive connection failures: ${consecutiveFailures}`,
        severity,
      )
    }

    // Check packet loss
    if (stats.packetLoss >= alertThresholds.packetLoss) {
      get().createAlert(
        'high_packet_loss',
        `High packet loss detected: ${(stats.packetLoss * 100).toFixed(1)}%`,
        'high',
      )
    }

    // Check latency
    if (stats.averageLatency >= alertThresholds.averageLatency) {
      get().createAlert(
        'high_latency',
        `High average latency detected: ${stats.averageLatency.toFixed(0)}ms`,
        'medium',
      )
    }

    // Check stability
    if (stats.stability <= alertThresholds.stability) {
      get().createAlert(
        'low_stability',
        `Low connection stability detected: ${(stats.stability * 100).toFixed(1)}%`,
        'medium',
      )
    }

    // Check failure prediction
    if (stats.failurePrediction > 0.7) {
      get().createAlert(
        'failure_prediction',
        `High failure prediction: ${(stats.failurePrediction * 100).toFixed(1)}%`,
        'high',
      )
    }

    // Auto-clear resolved alerts
    activeAlerts.forEach(alert => {
      switch (alert.type) {
        case 'consecutive_failures':
          if (consecutiveFailures < alertThresholds.consecutiveFailures) {
            get().clearAlert(alert.id)
          }
          break
        case 'high_packet_loss':
          if (stats.packetLoss < alertThresholds.packetLoss) {
            get().clearAlert(alert.id)
          }
          break
        case 'high_latency':
          if (stats.averageLatency < alertThresholds.averageLatency) {
            get().clearAlert(alert.id)
          }
          break
        case 'low_stability':
          if (stats.stability > alertThresholds.stability) {
            get().clearAlert(alert.id)
          }
          break
      }
    })
  },

  createAlert: (
    type: string,
    message: string,
    severity: 'low' | 'medium' | 'high' | 'critical',
  ) => {
    const { activeAlerts } = get()

    // Check if alert already exists
    const existingAlert = activeAlerts.find(alert => alert.type === type)
    if (existingAlert) {
      return // Don't create duplicate alerts
    }

    const alert = {
      id: `alert_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      type,
      message,
      timestamp: Date.now(),
      severity,
    }

    set({ activeAlerts: [...activeAlerts, alert] })

    const emoji = {
      low: 'ℹ️',
      medium: '⚠️',
      high: '🚨',
      critical: '🔴',
    }[severity]

    console.warn(`${emoji} Connection Alert [${severity.toUpperCase()}]: ${message}`)

    // Could integrate with external alerting system here
    // Example: sendAlertToMonitoringSystem(alert)
  },

  clearAlert: (alertId: string) => {
    const { activeAlerts } = get()
    set({ activeAlerts: activeAlerts.filter(a => a.id !== alertId) })
    console.warn(`✅ Alert cleared: ${activeAlerts.find(a => a.id === alertId)?.message}`)
  },

  getMonitoringReport: () => {
    const {
      connectionHistory,
      activeAlerts,
      performanceMetrics,
      alertThresholds,
      consecutiveFailures,
      circuitBreakerOpen,
    } = get()

    const stats = get().getConnectionStats()
    const poolStatus = get().getConnectionPoolStatus()

    return {
      timestamp: Date.now(),
      connection: {
        isConnected: get().isConnected,
        quality: get().connectionQuality,
        consecutiveFailures,
        circuitBreakerOpen,
        currentUrl: get().currentUrl,
      },
      statistics: stats,
      connectionPool: poolStatus,
      alerts: {
        active: activeAlerts,
        thresholds: alertThresholds,
      },
      performance: {
        ...performanceMetrics,
        currentUptime: Date.now() - performanceMetrics.uptime,
      },
      history: {
        totalEntries: connectionHistory.length,
        recentEntries: connectionHistory.slice(-5),
      },
    }
  },

  resetPerformanceMetrics: () => {
    set({
      performanceMetrics: {
        totalConnections: 0,
        totalMessages: 0,
        totalErrors: 0,
        uptime: Date.now(),
        lastReset: Date.now(),
      },
      activeAlerts: [],
    })
    console.warn('📊 Performance metrics reset')
  },
  hiveStatus: null,
  tasks: [],

  connectWebSocket: (url: string | string[]) => {
    // Check circuit breaker
    const { circuitBreakerOpen, circuitBreakerTimeout } = get()
    if (circuitBreakerOpen) {
      const now = Date.now()
      if (now < circuitBreakerTimeout) {
        console.warn(
          `🚫 Circuit breaker is open. Skipping connection attempt until ${new Date(circuitBreakerTimeout).toISOString()}`,
        )
        return
      } else {
        // Circuit breaker timeout expired, reset it
        set({ circuitBreakerOpen: false, circuitBreakerTimeout: 0 })
      }
    }

    // Process URL(s) - if array provided, set as fallback URLs
    let targetUrl: string
    if (Array.isArray(url)) {
      if (url.length === 0) {
        console.warn('❌ No URLs provided for connection')
        return
      }
      // Set fallback URLs and use first one
      set({ fallbackUrls: url, currentUrlIndex: 0 })
      const [firstUrl] = url
      targetUrl = firstUrl
    } else {
      targetUrl = url
      // If single URL and no fallbacks set, initialize with it
      if (get().fallbackUrls.length === 0) {
        set({ fallbackUrls: [url], currentUrlIndex: 0 })
      }
    }

    // Prevent multiple connection attempts
    const currentSocket = get().socket
    const { isReconnecting } = get()

    if (
      currentSocket !== null &&
      currentSocket.readyState === WebSocket.OPEN &&
      !get().forceReconnectFlag
    ) {
      console.warn('WebSocket already connected')
      return
    }

    if (isReconnecting && !get().forceReconnectFlag) {
      console.warn('WebSocket reconnection already in progress')
      return
    }

    // Check if connection should be throttled due to browser environment
    if (get().shouldThrottleConnection() && !get().forceReconnectFlag) {
      console.warn('🛑 Connection throttled due to browser environment conditions')
      return
    }

    // Transition to connecting state
    get().transitionConnectionState('connecting', `Connecting to ${targetUrl}`)
    set({ isReconnecting: true, forceReconnectFlag: false })

    // Start network monitoring
    get().startNetworkMonitoring()

    console.warn('🔌 Attempting WebSocket connection to:', targetUrl)

    const connectStart = Date.now()
    const socket = new WebSocket(targetUrl)

    // Store the current URL being used
    set({ currentUrl: targetUrl })

    // Start connection timeout
    get().startConnectionTimeout()

    socket.onopen = () => {
      const connectEnd = Date.now()
      const latency = connectEnd - connectStart

      console.warn('✅ WebSocket connected successfully', `(${latency}ms)`)

      // Update connection history
      const history = get().connectionHistory
      const newHistory = [
        ...history.slice(-19), // Keep last 19 entries
        { timestamp: connectEnd, success: true, latency },
      ]

      // Clear connection timeout on successful connection
      get().clearConnectionTimeout()

      // Transition to connected state
      get().transitionConnectionState('connected', `Connected in ${latency}ms`)

      set({
        socket,
        connectionAttempts: 0,
        lastHeartbeat: Date.now(),
        connectionQuality: 'excellent',
        isReconnecting: false,
        connectionHistory: newHistory,
        lastErrorType: null,
        consecutiveFailures: 0,
        circuitBreakerOpen: false,
        circuitBreakerTimeout: 0,
      })
      get().startHeartbeat()
    }

    socket.onmessage = event => {
      try {
        const message = JSON.parse(event.data)

        // Handle heartbeat responses first
        if (message.action === 'pong' && message.sequenceId) {
          get().handleHeartbeatResponse(message.sequenceId)
          return
        }

        if (typeof message.message_type === 'string') {
          switch (message.message_type) {
            case 'hive_status':
              set({ hiveStatus: message.data })
              break
            case 'agents_update':
              set({ agents: message.data?.agents ?? [] })
              break
            case 'metrics_update': {
              const currentStatus = get().hiveStatus
              if (
                currentStatus !== null &&
                currentStatus !== undefined &&
                message.data?.metrics !== null &&
                message.data?.metrics !== undefined
              ) {
                set({
                  hiveStatus: {
                    ...currentStatus,
                    metrics: message.data.metrics,
                    swarm_center: message.data.swarm_center ?? currentStatus.swarm_center,
                    total_energy: message.data.total_energy ?? currentStatus.total_energy,
                  },
                })
              }
              break
            }
            case 'agent_created':
            case 'task_created':
              console.warn('Created:', message.data)
              break
            case 'tasks_update':
              set({ tasks: message.data?.tasks ?? [] })
              break
            case 'task_status_update': {
              const currentTasks = get().tasks
              const updatedTask = message.data?.task
              if (updatedTask !== null && updatedTask !== undefined) {
                const updatedTasks = currentTasks.map(task =>
                  task.id === updatedTask.id ? { ...task, ...updatedTask } : task,
                )
                set({ tasks: updatedTasks })
              }
              break
            }
            case 'error':
              console.warn('Hive error:', message.data?.error)
              break
          }
        }
      } catch (error) {
        console.warn('Failed to parse WebSocket message:', error)
      }
    }

    socket.onclose = event => {
      const attempts = get().connectionAttempts
      const { consecutiveFailures, circuitBreakerOpen } = get()

      // Classify the error
      const errorType = get().classifyWebSocketError(event)
      const errorMessage = event.reason || `WebSocket closed with code ${event.code}`

      console.warn(
        `🔌 WebSocket disconnected (code: ${event.code}, reason: ${errorMessage}, type: ${errorType})`,
      )

      // Check if we should retry (for history tracking)
      const shouldRetryForHistory = get().shouldRetryConnection(errorType, attempts)
      const stats = get().getConnectionStats()
      const retryDelay = shouldRetryForHistory
        ? get().calculateSmartReconnectDelay(errorType, attempts, stats.stability)
        : 0

      // Update connection history with detailed error information
      const history = get().connectionHistory
      const newHistory = [
        ...history.slice(-19), // Keep last 19 entries
        {
          timestamp: Date.now(),
          success: false,
          errorType,
          errorMessage,
          reconnectDelay: retryDelay,
        },
      ]

      // Update consecutive failures and check circuit breaker
      const newConsecutiveFailures = consecutiveFailures + 1
      const shouldOpenCircuitBreaker = newConsecutiveFailures >= 5 // Open after 5 consecutive failures

      get().stopHeartbeat()
      get().clearConnectionTimeout()

      // Determine new state based on situation
      let newState: ConnectionState
      let reason: string

      if (shouldOpenCircuitBreaker) {
        newState = 'circuit_breaker_open'
        reason = `Circuit breaker opened after ${newConsecutiveFailures} consecutive failures`
      } else if (event.code === 1000) {
        // Normal closure
        newState = 'disconnected'
        reason = 'Normal disconnection'
      } else if (shouldRetryForHistory) {
        newState = 'reconnecting'
        reason = `Reconnecting due to ${errorType}`
      } else {
        newState = 'failed'
        reason = `Failed to reconnect: ${errorType}`
      }

      // Transition to new state
      get().transitionConnectionState(newState, reason)

      set({
        socket: null,
        connectionHistory: newHistory,
        isReconnecting: newState === 'reconnecting',
        lastErrorType: errorType,
        consecutiveFailures: newConsecutiveFailures,
        circuitBreakerOpen: shouldOpenCircuitBreaker,
        circuitBreakerTimeout: shouldOpenCircuitBreaker ? Date.now() + 60000 : 0, // 1 minute timeout
      })

      // Check if we should retry
      const shouldRetry = get().shouldRetryConnection(errorType, attempts)

      if (shouldRetry && event.code !== 1000 && !circuitBreakerOpen) {
        // Don't retry on normal closure or if circuit breaker is open

        // Try fallback URL first if available
        const fallbackUrl = get().tryNextFallbackUrl()

        if (fallbackUrl) {
          console.warn(`🔄 Trying fallback URL: ${fallbackUrl}`)
          setTimeout(async () => {
            // Perform connection health check before retrying
            const isHealthy = await get().checkConnectionHealth()

            if (isHealthy) {
              console.warn('✅ Connection health check passed, proceeding with fallback connection')
              set({ connectionAttempts: attempts + 1 })
              get().connectWebSocket(fallbackUrl)
            } else {
              console.warn('❌ Connection health check failed, trying original URL with delay')
              // Fall back to original retry logic with delay
              const stats = get().getConnectionStats()
              const retryDelay = get().calculateSmartReconnectDelay(
                errorType,
                attempts,
                stats.stability,
              )

              setTimeout(() => {
                set({ connectionAttempts: attempts + 1 })
                get().connectWebSocket(get().currentUrl || targetUrl)
              }, retryDelay)
            }
          }, 1000) // Short delay before trying fallback
        } else {
          // No fallback URLs available, use original retry logic
          const stats = get().getConnectionStats()
          const retryDelay = get().calculateSmartReconnectDelay(
            errorType,
            attempts,
            stats.stability,
          )

          console.warn(
            `🔄 Retrying WebSocket connection in ${retryDelay}ms... (attempt ${attempts + 1}/${get().maxReconnectAttempts}, type: ${errorType}, stability: ${Math.round(stats.stability * 100)}%)`,
          )

          setTimeout(async () => {
            // Perform connection health check before retrying
            const isHealthy = await get().checkConnectionHealth()

            if (isHealthy) {
              console.warn('✅ Connection health check passed, proceeding with reconnection')
              set({ connectionAttempts: attempts + 1 })
              get().connectWebSocket(get().currentUrl || targetUrl)
            } else {
              console.warn('❌ Connection health check failed, skipping reconnection attempt')
              // Update history to reflect failed health check
              const history = get().connectionHistory
              const newHistory = [
                ...history.slice(-19),
                {
                  timestamp: Date.now(),
                  success: false,
                  errorType: 'network_unreachable' as ConnectionErrorType,
                  errorMessage: 'Connection health check failed',
                  reconnectDelay: 0,
                },
              ]
              set({ connectionHistory: newHistory })
            }
          }, retryDelay)
        }
      } else {
        const reason = circuitBreakerOpen
          ? 'Circuit breaker is open'
          : attempts >= get().maxReconnectAttempts
            ? 'Max attempts reached'
            : event.code === 1000
              ? 'Normal closure'
              : `Error type ${errorType} not retryable`

        console.warn(`❌ Not retrying WebSocket connection: ${reason}`)
        set({ connectionQuality: 'disconnected', isReconnecting: false })
      }
    }

    socket.onerror = async error => {
      const attempts = get().connectionAttempts
      const { consecutiveFailures, circuitBreakerOpen } = get()

      // Classify the error (generic error event)
      const errorType = get().classifyWebSocketError(error, error)
      const errorMessage = 'WebSocket error event occurred'

      // Check for network interruption
      const isNetworkInterrupted = await get().detectNetworkInterruption()

      console.warn(
        `WebSocket connection error (type: ${errorType}) - this is normal during development. Retrying...`,
        error,
      )

      // Check if we should retry (for history tracking)
      const shouldRetryForHistory = get().shouldRetryConnection(errorType, attempts)
      const stats = get().getConnectionStats()
      const retryDelay = shouldRetryForHistory
        ? get().calculateSmartReconnectDelay(errorType, attempts, stats.stability)
        : 0

      // Update connection history with error details
      const history = get().connectionHistory
      const newHistory = [
        ...history.slice(-19), // Keep last 19 entries
        {
          timestamp: Date.now(),
          success: false,
          errorType,
          errorMessage,
          reconnectDelay: retryDelay,
        },
      ]

      // Update consecutive failures
      const newConsecutiveFailures = consecutiveFailures + 1
      const shouldOpenCircuitBreaker = newConsecutiveFailures >= 5

      get().clearConnectionTimeout()

      // Determine new state based on error and network conditions
      let newState: ConnectionState
      let reason: string

      if (isNetworkInterrupted) {
        newState = 'failed'
        reason = `Network interruption detected: ${errorType}`
      } else if (shouldOpenCircuitBreaker) {
        newState = 'circuit_breaker_open'
        reason = `Circuit breaker opened after ${newConsecutiveFailures} consecutive errors`
      } else if (shouldRetryForHistory) {
        newState = 'failed' // Will transition to reconnecting in the retry logic
        reason = `Connection error: ${errorType}`
      } else {
        newState = 'failed'
        reason = `Unrecoverable connection error: ${errorType}`
      }

      // Transition to new state
      get().transitionConnectionState(newState, reason)

      set({
        connectionHistory: newHistory,
        isReconnecting: false,
        lastErrorType: errorType,
        consecutiveFailures: newConsecutiveFailures,
        circuitBreakerOpen: shouldOpenCircuitBreaker,
        circuitBreakerTimeout: shouldOpenCircuitBreaker ? Date.now() + 60000 : 0,
      })

      // Check if we should retry
      const shouldRetry = get().shouldRetryConnection(errorType, attempts)

      if (shouldRetry && !circuitBreakerOpen) {
        const stats = get().getConnectionStats()
        const retryDelay = get().calculateSmartReconnectDelay(errorType, attempts, stats.stability)

        console.warn(
          `Attempting to reconnect WebSocket in ${retryDelay}ms... (stability: ${Math.round(stats.stability * 100)}%)`,
        )

        setTimeout(async () => {
          if (get().socket?.readyState !== WebSocket.OPEN) {
            // Perform connection health check before retrying
            const isHealthy = await get().checkConnectionHealth()

            if (isHealthy) {
              console.warn('✅ Connection health check passed, proceeding with reconnection')
              set({ connectionAttempts: attempts + 1 })
              get().connectWebSocket(get().currentUrl || 'ws://localhost:8080')
            } else {
              console.warn('❌ Connection health check failed, skipping reconnection attempt')
              // Update history to reflect failed health check
              const history = get().connectionHistory
              const newHistory = [
                ...history.slice(-19),
                {
                  timestamp: Date.now(),
                  success: false,
                  errorType: 'network_unreachable' as ConnectionErrorType,
                  errorMessage: 'Connection health check failed',
                  reconnectDelay: 0,
                },
              ]
              set({ connectionHistory: newHistory })
            }
          }
        }, retryDelay)
      } else {
        const reason = circuitBreakerOpen ? 'Circuit breaker is open' : 'Error not retryable'
        console.warn(`❌ Not retrying WebSocket connection: ${reason}`)
      }
    }
  },

  disconnect: () => {
    const { socket } = get()
    get().stopHeartbeat()
    get().stopNetworkMonitoring()
    get().stopPredictiveRecovery()

    if (socket !== null && socket !== undefined) {
      socket.close(1000, 'User initiated disconnect') // Normal closure
    }

    // Transition to disconnected state
    get().transitionConnectionState('disconnected', 'User initiated disconnect')

    set({
      socket: null,
      connectionAttempts: 0,
      isReconnecting: false,
      forceReconnectFlag: false,
      networkInterruptionDetected: false,
      networkOutageStart: null,
      networkRecoveryAttempts: 0,
    })
  },

  createAgent: (config: unknown) => {
    // Check if we should send this message based on degradation level
    if (!get().shouldSendMessage()) {
      console.warn('📉 Message throttled due to connection degradation')
      return
    }

    const message = get().compressMessage({
      action: 'create_agent',
      payload: config,
    })

    // Use multiplexing distribution if enabled
    get().distributeMessage(message)
  },

  createTask: (config: unknown) => {
    // Check if we should send this message based on degradation level
    if (!get().shouldSendMessage()) {
      console.warn('📉 Message throttled due to connection degradation')
      return
    }

    const message = get().compressMessage({
      action: 'create_task',
      payload: config,
    })

    // Use multiplexing distribution if enabled
    get().distributeMessage(message)
  },

  updateAgents: (agents: Agent[]) => set({ agents }),
  updateHiveStatus: (status: HiveStatus) => set({ hiveStatus: status }),
  updateTasks: (tasks: Task[]) => set({ tasks }),

  startHeartbeat: () => {
    const { heartbeatInterval } = get()
    if (heartbeatInterval) {
      clearInterval(heartbeatInterval)
    }

    const interval = setInterval(() => {
      const { socket, isConnected, tabVisible, memoryPressure } = get()

      // Skip heartbeat if tab is hidden or memory pressure is critical
      if (!tabVisible || memoryPressure === 'critical') {
        console.warn('⏸️ Skipping heartbeat due to browser environment')
        return
      }

      if (socket && isConnected && socket.readyState === WebSocket.OPEN) {
        try {
          // Apply degradation level checks for heartbeat
          const { degradationLevel } = get()

          // Skip heartbeat based on degradation level
          if (degradationLevel === 'critical' && Math.random() > 0.5) {
            console.warn('⏸️ Skipping heartbeat due to critical degradation')
            return
          }

          // Send heartbeat using the enhanced system
          get().sendHeartbeat()

          get().updateConnectionQuality()
        } catch (error) {
          console.warn('Failed to send heartbeat:', error)
          set({ connectionQuality: 'poor' })
        }
      }
    }, get().heartbeatFrequency) // Use dynamic heartbeat frequency

    set({ heartbeatInterval: interval })
  },

  stopHeartbeat: () => {
    const { heartbeatInterval, pendingHeartbeats } = get()
    if (heartbeatInterval) {
      clearInterval(heartbeatInterval)
      set({ heartbeatInterval: null })
    }

    // Clear all pending heartbeat timeouts
    pendingHeartbeats.forEach(heartbeat => {
      clearTimeout(heartbeat.timeoutId)
    })
    set({ pendingHeartbeats: [] })
  },

  sendHeartbeat: async (): Promise<boolean> => {
    const { socket, isConnected, heartbeatSequence, pendingHeartbeats } = get()

    if (!socket || !isConnected || socket.readyState !== WebSocket.OPEN) {
      return false
    }

    const sequenceId = heartbeatSequence + 1
    const timestamp = Date.now()

    try {
      // Enhanced heartbeat with connection quality metrics
      const connectionStats = get().getConnectionStats()
      const heartbeatMessage = get().compressMessage({
        action: 'ping',
        sequenceId,
        timestamp,
        metrics: {
          clientLatency: connectionStats.averageLatency,
          clientStability: connectionStats.stability,
          clientQuality: get().connectionQuality,
        },
      })

      // Send heartbeat
      socket.send(JSON.stringify(heartbeatMessage))

      // Adaptive timeout based on connection quality
      const adaptiveTimeout = get().calculateAdaptiveHeartbeatTimeout()
      const timeoutId = setTimeout(() => {
        get().handleHeartbeatTimeout(sequenceId)
      }, adaptiveTimeout)

      // Add to pending heartbeats with enhanced tracking
      const newPendingHeartbeat = {
        id: sequenceId,
        timestamp,
        timeoutId,
        quality: get().connectionQuality,
        expectedLatency: connectionStats.averageLatency,
      }

      set({
        heartbeatSequence: sequenceId,
        pendingHeartbeats: [...pendingHeartbeats, newPendingHeartbeat],
        lastHeartbeat: timestamp,
      })

      return true
    } catch (error) {
      console.warn('Failed to send heartbeat:', error)
      set({ heartbeatFailures: get().heartbeatFailures + 1 })

      // Trigger connection quality degradation on send failure
      if (get().connectionQuality !== 'critical') {
        get().updateConnectionQuality()
      }

      return false
    }
  },

  calculateAdaptiveHeartbeatTimeout: (): number => {
    const { heartbeatTimeout, connectionQuality, heartbeatLatencies } = get()
    const baseTimeout = heartbeatTimeout

    // Adjust timeout based on connection quality
    let qualityMultiplier = 1.0
    switch (connectionQuality) {
      case 'excellent':
        qualityMultiplier = 0.8 // Faster timeout for excellent connections
        break
      case 'good':
        qualityMultiplier = 1.0 // Standard timeout
        break
      case 'fair':
        qualityMultiplier = 1.3 // Slightly longer timeout
        break
      case 'poor':
        qualityMultiplier = 1.8 // Much longer timeout
        break
      case 'critical':
        qualityMultiplier = 2.5 // Very long timeout for critical connections
        break
      default:
        qualityMultiplier = 1.0
        break
    }

    // Adjust based on recent latency patterns
    if (heartbeatLatencies.length > 2) {
      const avgLatency =
        heartbeatLatencies.reduce((sum, lat) => sum + lat, 0) / heartbeatLatencies.length
      const latencyMultiplier = Math.max(0.5, Math.min(2.0, avgLatency / 1000)) // 0.5x to 2x based on latency
      qualityMultiplier *= latencyMultiplier
    }

    // Add jitter to prevent synchronized timeouts
    const jitter = 0.8 + Math.random() * 0.4 // 80%-120% of calculated timeout

    const adaptiveTimeout = Math.round(baseTimeout * qualityMultiplier * jitter)

    // Ensure reasonable bounds
    return Math.max(5000, Math.min(120000, adaptiveTimeout)) // 5 seconds to 2 minutes
  },

  handleHeartbeatResponse: (sequenceId: number) => {
    const { pendingHeartbeats, heartbeatLatencies } = get()
    const now = Date.now()

    // Find the corresponding pending heartbeat
    const heartbeatIndex = pendingHeartbeats.findIndex(h => h.id === sequenceId)

    if (heartbeatIndex !== -1) {
      const heartbeat = pendingHeartbeats[heartbeatIndex]
      const latency = now - heartbeat.timestamp

      // Clear the timeout
      clearTimeout(heartbeat.timeoutId)

      // Remove from pending heartbeats
      const updatedPending = pendingHeartbeats.filter(h => h.id !== sequenceId)
      set({ pendingHeartbeats: updatedPending })

      // Record latency
      const newLatencies = [...heartbeatLatencies.slice(-9), latency] // Keep last 10 latencies
      set({
        heartbeatLatencies: newLatencies,
        lastHeartbeatResponse: now,
        heartbeatFailures: 0, // Reset failures on successful response
      })

      // Update health score
      get().calculateHeartbeatHealthScore()

      console.warn(`💓 Heartbeat response received (seq: ${sequenceId}, latency: ${latency}ms)`)
    } else {
      console.warn(`⚠️ Received heartbeat response for unknown sequence: ${sequenceId}`)
    }
  },

  handleHeartbeatTimeout: (sequenceId: number) => {
    const { pendingHeartbeats, heartbeatFailures } = get()

    // Remove from pending heartbeats
    const updatedPending = pendingHeartbeats.filter(h => h.id !== sequenceId)
    set({
      pendingHeartbeats: updatedPending,
      heartbeatFailures: heartbeatFailures + 1,
    })

    console.warn(`⏰ Heartbeat timeout (seq: ${sequenceId}, failures: ${heartbeatFailures + 1})`)

    // Update health score
    get().calculateHeartbeatHealthScore()

    // Trigger connection quality update
    get().updateConnectionQuality()
  },

  calculateHeartbeatHealthScore: () => {
    const { heartbeatLatencies, heartbeatFailures, pendingHeartbeats } = get()

    if (heartbeatLatencies.length === 0) {
      set({ heartbeatHealthScore: 1.0 })
      return 1.0
    }

    // Calculate latency score (lower latency = higher score)
    const avgLatency =
      heartbeatLatencies.reduce((sum, lat) => sum + lat, 0) / heartbeatLatencies.length
    const latencyScore = Math.max(0, Math.min(1, 1 - (avgLatency - 50) / 200)) // Optimal: 50ms, Poor: 250ms+

    // Calculate failure score (fewer failures = higher score)
    const failureScore = Math.max(0, Math.min(1, 1 - heartbeatFailures / 5)) // 5+ failures = 0 score

    // Calculate pending score (fewer pending = higher score)
    const pendingScore = Math.max(0, Math.min(1, 1 - pendingHeartbeats.length / 3)) // 3+ pending = 0 score

    // Combine scores with weights
    const healthScore = latencyScore * 0.5 + failureScore * 0.3 + pendingScore * 0.2

    set({ heartbeatHealthScore: healthScore })

    return healthScore
  },

  getHeartbeatStats: () => {
    const { heartbeatLatencies, heartbeatFailures, heartbeatHealthScore } = get()

    const averageLatency =
      heartbeatLatencies.length > 0
        ? heartbeatLatencies.reduce((sum, lat) => sum + lat, 0) / heartbeatLatencies.length
        : 0

    const successRate =
      heartbeatLatencies.length > 0
        ? heartbeatLatencies.length / (heartbeatLatencies.length + heartbeatFailures)
        : 1.0

    return {
      averageLatency,
      successRate,
      healthScore: heartbeatHealthScore,
      consecutiveFailures: heartbeatFailures,
    }
  },

  detectNetworkInterruption: async (): Promise<boolean> => {
    const now = Date.now()
    const { lastNetworkCheck } = get()

    // Don't check too frequently
    if (now - lastNetworkCheck < 5000) {
      // 5 second minimum interval
      return get().networkInterruptionDetected
    }

    set({ lastNetworkCheck: now })

    try {
      // Multiple network checks for reliability
      const checks = await Promise.allSettled([
        // Basic online check
        Promise.resolve(navigator.onLine),

        // Fetch-based connectivity check
        fetch('/api/health', {
          method: 'HEAD',
          cache: 'no-cache',
          signal: AbortSignal.timeout(3000),
        })
          .then(res => res.ok)
          .catch(() => false),

        // DNS resolution check (if available)
        typeof window !== 'undefined' && window.location.hostname !== 'localhost'
          ? fetch(`https://${window.location.hostname}`, {
              method: 'HEAD',
              mode: 'no-cors',
              signal: AbortSignal.timeout(3000),
            })
              .then(() => true)
              .catch(() => false)
          : Promise.resolve(true),
      ])

      const results = checks.map(check => (check.status === 'fulfilled' ? check.value : false))

      // Consider network interrupted if most checks fail
      const successCount = results.filter(Boolean).length
      const isInterrupted = successCount < results.length * 0.5 // Less than 50% success

      const wasInterrupted = get().networkInterruptionDetected

      if (isInterrupted && !wasInterrupted) {
        // Network interruption detected
        console.warn('🌐 Network interruption detected')
        set({
          networkInterruptionDetected: true,
          networkOutageStart: now,
          networkRecoveryAttempts: 0,
        })
        get().handleNetworkInterruption()
      } else if (!isInterrupted && wasInterrupted) {
        // Network recovered
        const outageStart = get().networkOutageStart
        const outageDuration = outageStart ? now - outageStart : 0
        console.warn(`🌐 Network recovered after ${outageDuration}ms`)
        set({
          networkInterruptionDetected: false,
          networkOutageStart: null,
        })

        // Trigger fast reconnection if we were connected
        if (get().connectionState === 'connected' || get().connectionState === 'degraded') {
          get().transitionConnectionState('reconnecting', 'Network recovered - fast reconnect')
          // Use minimal delay for fast reconnection
          setTimeout(() => {
            const { currentUrl } = get()
            if (currentUrl) {
              get().connectWebSocket(currentUrl)
            }
          }, 1000) // 1 second delay for fast reconnect
        }
      }

      return isInterrupted
    } catch (error) {
      console.warn('Network detection check failed:', error)
      return true // Assume interruption if check fails
    }
  },

  handleNetworkInterruption: () => {
    const { connectionState, networkRecoveryAttempts } = get()

    // If we're in a connected state, transition to degraded
    if (connectionState === 'connected') {
      get().transitionConnectionState('degraded', 'Network interruption detected')
    }

    // Start network monitoring if not already started
    get().startNetworkMonitoring()

    // If we have too many recovery attempts, be more conservative
    if (networkRecoveryAttempts > 3) {
      console.warn('🌐 Multiple network recovery attempts - being conservative')
      return
    }

    // For temporary outages, attempt fast reconnection
    if (get().isTemporaryOutage()) {
      set({ networkRecoveryAttempts: networkRecoveryAttempts + 1 })
      console.warn(`🌐 Attempting fast reconnection (attempt ${networkRecoveryAttempts + 1})`)

      setTimeout(() => {
        if (get().networkInterruptionDetected) {
          const { currentUrl } = get()
          if (currentUrl) {
            get().connectWebSocket(currentUrl)
          }
        }
      }, 2000) // 2 second delay for network interruption recovery
    }
  },

  startNetworkMonitoring: () => {
    const { networkCheckInterval } = get()

    if (networkCheckInterval) {
      return // Already monitoring
    }

    console.warn('🌐 Starting network monitoring')

    const interval = setInterval(async () => {
      await get().detectNetworkInterruption()
    }, 10000) // Check every 10 seconds

    set({ networkCheckInterval: interval })

    // Also listen to online/offline events
    const handleOnline = () => {
      console.warn('🌐 Browser reports online')
      get().detectNetworkInterruption()
    }

    const handleOffline = () => {
      console.warn('🌐 Browser reports offline')
      set({ networkInterruptionDetected: true })
      get().handleNetworkInterruption()
    }

    window.addEventListener('online', handleOnline)
    window.addEventListener('offline', handleOffline)

    // Store cleanup functions
    ;(get() as Record<string, unknown>)._networkOnlineHandler = handleOnline
    ;(get() as Record<string, unknown>)._networkOfflineHandler = handleOffline
  },

  stopNetworkMonitoring: () => {
    const { networkCheckInterval } = get()

    if (networkCheckInterval) {
      clearInterval(networkCheckInterval)
      set({ networkCheckInterval: null })
    }

    // Remove event listeners
    const onlineHandler = (get() as Record<string, unknown>)._networkOnlineHandler
    const offlineHandler = (get() as Record<string, unknown>)._networkOfflineHandler

    if (onlineHandler) {
      window.removeEventListener('online', onlineHandler)
    }
    if (offlineHandler) {
      window.removeEventListener('offline', offlineHandler)
    }

    console.warn('🌐 Stopped network monitoring')
  },

  isTemporaryOutage: (): boolean => {
    const { networkOutageStart, connectionHistory } = get()

    if (!networkOutageStart) {
      return false
    }

    const outageDuration = Date.now() - networkOutageStart

    // Consider it temporary if outage is less than 5 minutes
    if (outageDuration > 300000) {
      // 5 minutes
      return false
    }

    // Check recent connection history for patterns
    const recentHistory = connectionHistory.slice(-5)
    const recentFailures = recentHistory.filter(h => !h.success)

    // If we have mostly successful recent connections, likely temporary
    const successRate =
      recentHistory.length > 0
        ? (recentHistory.length - recentFailures.length) / recentHistory.length
        : 0.5

    return successRate > 0.6 // 60% success rate indicates likely temporary outage
  },

  getUserFriendlyErrorMessage: (
    errorType: ConnectionErrorType,
    _context?: Record<string, unknown>,
  ): string => {
    const { networkInterruptionDetected, consecutiveFailures } = get()

    // Base messages for different error types
    const baseMessages: Record<ConnectionErrorType, string> = {
      network_unreachable:
        'Unable to reach the server. This usually indicates a network connectivity issue.',
      dns_failure:
        'Cannot resolve the server address. The domain name may be incorrect or DNS is not working.',
      tls_handshake:
        'Secure connection failed. This could be due to certificate issues or network interception.',
      server_unavailable:
        'The server is currently unavailable. It may be undergoing maintenance or experiencing high load.',
      protocol_error:
        'Communication protocol error. This is usually a temporary issue with the connection.',
      timeout:
        'Connection timed out. The server may be slow to respond or the network connection is unstable.',
      connection_refused:
        'Connection was refused by the server. The server may be down or rejecting connections.',
      connection_reset:
        'Connection was unexpectedly reset. This often happens due to network instability.',
      unknown: 'An unexpected connection error occurred.',
    }

    let message = baseMessages[errorType] || baseMessages.unknown

    // Add context-specific information
    if (networkInterruptionDetected) {
      message += ' A network interruption has been detected on your device.'
    }

    if (consecutiveFailures > 3) {
      message += ` This has failed ${consecutiveFailures} times in a row.`
    }

    // Add time-based context
    const hour = new Date().getHours()
    if (hour >= 2 && hour <= 5) {
      message +=
        ' (Note: Connection issues during early morning hours are common due to maintenance windows.)'
    }

    return message
  },

  getRecoverySuggestions: (
    errorType: ConnectionErrorType,
    connectionState: ConnectionState,
  ): string[] => {
    const suggestions: string[] = []
    const { networkInterruptionDetected, consecutiveFailures, circuitBreakerOpen } = get()

    // Base suggestions for all errors
    suggestions.push('Wait a moment and try again - many connection issues resolve themselves.')

    // Error-specific suggestions
    switch (errorType) {
      case 'network_unreachable':
      case 'dns_failure':
        suggestions.push('Check your internet connection and try refreshing the page.')
        suggestions.push('Try switching between WiFi and mobile data if available.')
        if (networkInterruptionDetected) {
          suggestions.push(
            'Your device has detected a network interruption - wait for it to recover.',
          )
        }
        break

      case 'tls_handshake':
        suggestions.push('Try refreshing the page to establish a new secure connection.')
        suggestions.push(
          "Check if you're using a VPN or proxy that might interfere with secure connections.",
        )
        break

      case 'server_unavailable':
        suggestions.push(
          'The server might be temporarily down. Check the service status page if available.',
        )
        suggestions.push('Try again in a few minutes.')
        break

      case 'timeout':
        suggestions.push(
          'The connection is taking longer than expected. This might resolve itself.',
        )
        suggestions.push('Check your network speed and stability.')
        break

      case 'connection_refused':
      case 'connection_reset':
        suggestions.push('The server may be experiencing issues. Try again shortly.')
        if (consecutiveFailures > 5) {
          suggestions.push('If this persists, contact support with the error details.')
        }
        break

      default:
        suggestions.push('Try refreshing the page or restarting your browser.')
        break
    }

    // State-specific suggestions
    switch (connectionState) {
      case 'circuit_breaker_open':
        suggestions.unshift(
          'The system is temporarily preventing reconnection attempts to avoid overloading the server.',
        )
        suggestions.push('Wait 1-2 minutes before the system automatically tries to reconnect.')
        break

      case 'degraded':
        suggestions.push(
          'The connection is working but with reduced performance. Some features may be limited.',
        )
        break

      case 'failed':
        if (!circuitBreakerOpen) {
          suggestions.push('The system will automatically attempt to reconnect.')
        }
        break
    }

    // Add general troubleshooting if many failures
    if (consecutiveFailures > 3) {
      suggestions.push('If problems persist, try clearing your browser cache and cookies.')
      suggestions.push('Consider restarting your router or switching networks.')
    }

    return suggestions
  },

  getConnectionStatusMessage: (): string => {
    const { connectionState, connectionQuality, networkInterruptionDetected, consecutiveFailures } =
      get()

    let message = ''

    switch (connectionState) {
      case 'disconnected':
        message = 'Not connected to the server.'
        break
      case 'connecting':
        message = 'Connecting to the server...'
        break
      case 'connected':
        message = 'Connected and fully operational.'
        break
      case 'reconnecting':
        message = 'Reconnecting to the server...'
        break
      case 'degraded':
        message = 'Connected with some performance issues.'
        break
      case 'failed':
        message = 'Connection failed.'
        break
      case 'circuit_breaker_open':
        message = 'Temporarily paused reconnection attempts to prevent server overload.'
        break
    }

    // Add quality information
    switch (connectionQuality) {
      case 'excellent':
        message += ' Connection quality is excellent.'
        break
      case 'good':
        message += ' Connection quality is good.'
        break
      case 'fair':
        message += ' Connection quality is fair.'
        break
      case 'poor':
        message += ' Connection quality is poor - you may experience delays.'
        break
      case 'critical':
        message += ' Connection quality is critical - expect significant issues.'
        break
    }

    // Add network interruption warning
    if (networkInterruptionDetected) {
      message += ' Network interruption detected.'
    }

    // Add failure warning
    if (consecutiveFailures > 0) {
      message += ` (${consecutiveFailures} consecutive connection issue${consecutiveFailures > 1 ? 's' : ''})`
    }

    return message
  },

  getActionableAdvice: (): string => {
    const { connectionState, connectionQuality, networkInterruptionDetected, consecutiveFailures } =
      get()

    // Most critical issues first
    if (networkInterruptionDetected) {
      return 'Check your internet connection and wait for network recovery.'
    }

    if (connectionState === 'circuit_breaker_open') {
      return 'Please wait 1-2 minutes for automatic reconnection to resume.'
    }

    if (connectionQuality === 'critical') {
      return 'Connection is very unstable. Consider refreshing the page or checking your network.'
    }

    if (consecutiveFailures > 5) {
      return 'Multiple connection failures detected. Try refreshing the page or contact support if issues persist.'
    }

    if (connectionState === 'degraded') {
      return 'Connection is working but may be slow. Some features might be limited.'
    }

    if (connectionState === 'failed') {
      return 'Connection failed. The system will automatically try to reconnect.'
    }

    if (connectionState === 'connecting' || connectionState === 'reconnecting') {
      return 'Establishing connection... Please wait.'
    }

    if (connectionState === 'connected' && connectionQuality === 'excellent') {
      return 'Everything is working perfectly!'
    }

    // Default advice
    return 'Monitor the connection status for any issues.'
  },

  getReliabilityMetrics: () => {
    const { connectionHistory, performanceMetrics, heartbeatHealthScore } = get()
    const now = Date.now()

    if (connectionHistory.length === 0) {
      return {
        uptime: 0,
        availability: 1.0,
        meanTimeBetweenFailures: 0,
        meanTimeToRecovery: 0,
        failureRate: 0,
        recoveryRate: 1.0,
        networkStability: 1.0,
        performanceScore: 1.0,
      }
    }

    // Calculate uptime
    const totalTime = now - performanceMetrics.uptime
    const connectedTime = connectionHistory
      .filter(h => h.success)
      .reduce((sum, h) => sum + (h.latency || 0), 0)
    const uptime = totalTime > 0 ? connectedTime / totalTime : 1.0

    // Calculate availability (percentage of time connected)
    const successfulConnections = connectionHistory.filter(h => h.success).length
    const availability =
      connectionHistory.length > 0 ? successfulConnections / connectionHistory.length : 1.0

    // Calculate MTBF (Mean Time Between Failures)
    const failureTimes = connectionHistory.filter(h => !h.success).map(h => h.timestamp)
    let meanTimeBetweenFailures = 0
    if (failureTimes.length > 1) {
      const intervals = []
      for (let i = 1; i < failureTimes.length; i++) {
        intervals.push(failureTimes[i] - failureTimes[i - 1])
      }
      meanTimeBetweenFailures =
        intervals.reduce((sum, interval) => sum + interval, 0) / intervals.length
    }

    // Calculate MTTR (Mean Time To Recovery)
    const recoveryTimes = []
    let lastFailureTime = 0
    for (const entry of connectionHistory) {
      if (!entry.success) {
        lastFailureTime = entry.timestamp
      } else if (lastFailureTime > 0) {
        recoveryTimes.push(entry.timestamp - lastFailureTime)
        lastFailureTime = 0
      }
    }
    const meanTimeToRecovery =
      recoveryTimes.length > 0
        ? recoveryTimes.reduce((sum, time) => sum + time, 0) / recoveryTimes.length
        : 0

    // Calculate failure rate (failures per hour)
    const timeWindow = Math.max(now - performanceMetrics.uptime, 3600000) // At least 1 hour
    const failureRate = (connectionHistory.filter(h => !h.success).length / timeWindow) * 3600000

    // Calculate recovery rate (successful reconnections per hour)
    const recoveryRate = (successfulConnections / timeWindow) * 3600000

    // Calculate network stability (based on connection consistency)
    const recentHistory = connectionHistory.slice(-20)
    const recentSuccessRate =
      recentHistory.length > 0
        ? recentHistory.filter(h => h.success).length / recentHistory.length
        : 1.0
    const consistencyScore = 1 - Math.abs(recentSuccessRate - availability) // Lower variance = higher stability
    const networkStability = Math.min(consistencyScore, heartbeatHealthScore)

    // Calculate overall performance score
    const performanceScore =
      uptime * 0.25 + availability * 0.25 + networkStability * 0.25 + heartbeatHealthScore * 0.25

    return {
      uptime: Math.max(0, Math.min(1, uptime)),
      availability: Math.max(0, Math.min(1, availability)),
      meanTimeBetweenFailures,
      meanTimeToRecovery,
      failureRate: Math.max(0, failureRate),
      recoveryRate: Math.max(0, recoveryRate),
      networkStability: Math.max(0, Math.min(1, networkStability)),
      performanceScore: Math.max(0, Math.min(1, performanceScore)),
    }
  },

  getDetailedConnectionReport: () => {
    const basicReport = get().getMonitoringReport()
    const reliabilityMetrics = get().getReliabilityMetrics()
    const heartbeatStats = get().getHeartbeatStats()
    const connectionStats = get().getConnectionStats()

    return {
      ...basicReport,
      reliability: reliabilityMetrics,
      heartbeat: heartbeatStats,
      network: {
        interruptionDetected: get().networkInterruptionDetected,
        outageStart: get().networkOutageStart,
        recoveryAttempts: get().networkRecoveryAttempts,
        isTemporaryOutage: get().isTemporaryOutage(),
      },
      userExperience: {
        statusMessage: get().getConnectionStatusMessage(),
        actionableAdvice: get().getActionableAdvice(),
        lastError: (() => {
          const errorType = get().lastErrorType
          return errorType
            ? {
                type: errorType,
                message: get().getUserFriendlyErrorMessage(errorType),
                suggestions: get().getRecoverySuggestions(errorType, get().connectionState),
              }
            : null
        })(),
      },
      performance: {
        ...connectionStats,
        degradationLevel: get().degradationLevel,
        messageFrequency: get().messageFrequency,
        payloadSizeLimit: get().payloadSizeLimit,
      },
      recommendations: {
        shouldReconnect: get().shouldAttemptReconnection(),
        shouldThrottle: get().shouldThrottleConnection(),
        circuitBreakerStatus: get().circuitBreakerOpen ? 'open' : 'closed',
        multiplexingEnabled: get().multiplexingEnabled,
      },
    }
  },

  updateConnectionQuality: () => {
    const { lastHeartbeat, isConnected, connectionState } = get()
    const now = Date.now()
    const timeSinceHeartbeat = now - lastHeartbeat
    const stats = get().getConnectionStats()

    let newQuality: ConnectionQuality

    if (!isConnected || connectionState === 'disconnected') {
      newQuality = 'disconnected'
    } else if (connectionState === 'circuit_breaker_open') {
      newQuality = 'critical'
    } else if (timeSinceHeartbeat < 15000 && stats.stability > 0.9) {
      // Within 15 seconds and very high stability
      newQuality = 'excellent'
    } else if (timeSinceHeartbeat < 30000 && stats.stability > 0.8) {
      // Within 30 seconds and high stability
      newQuality = 'good'
    } else if (timeSinceHeartbeat < 60000 && stats.stability > 0.6) {
      // Within 1 minute and moderate stability
      newQuality = 'fair'
    } else if (timeSinceHeartbeat < 120000 && stats.stability > 0.4) {
      // Within 2 minutes and low stability
      newQuality = 'poor'
    } else {
      // Very poor conditions
      newQuality = 'critical'
    }

    // Update quality if it changed
    const { connectionQuality: currentQuality } = get()
    if (newQuality !== currentQuality) {
      set({ connectionQuality: newQuality })

      // Transition to degraded state if quality is poor
      if (newQuality === 'poor' || newQuality === 'critical') {
        const currentState = get().connectionState
        if (currentState === 'connected') {
          get().transitionConnectionState('degraded', `Quality degraded to ${newQuality}`)
        }
      } else if (newQuality === 'good' || newQuality === 'excellent') {
        const currentState = get().connectionState
        if (currentState === 'degraded') {
          get().transitionConnectionState('connected', `Quality improved to ${newQuality}`)
        }
      }

      console.warn(
        `📊 Connection quality: ${currentQuality} -> ${newQuality} (heartbeat: ${timeSinceHeartbeat}ms ago, stability: ${(stats.stability * 100).toFixed(1)}%)`,
      )
    }

    // Update degradation level based on new quality assessment
    get().updateDegradationLevel()
  },

  forceReconnect: () => {
    const { socket } = get()
    console.warn('🔄 Force reconnecting WebSocket...')

    // Reset connection attempts and force reconnection
    set({
      forceReconnectFlag: true,
      connectionAttempts: 0,
      isReconnecting: false,
      lastErrorType: null,
      consecutiveFailures: 0,
      circuitBreakerOpen: false,
      circuitBreakerTimeout: 0,
    })

    // Transition to disconnected state first
    get().transitionConnectionState('disconnected', 'Force reconnect initiated')

    // Close existing socket if any
    if (socket) {
      socket.close(1000, 'Force reconnect')
    }

    // Attempt new connection with original URL (stored in closure)
    // Note: In a real implementation, you'd store the URL in state
    // For now, we'll trigger via the component that has the URL
  },

  // Enhanced automatic recovery with predictive capabilities
  startPredictiveRecovery: () => {
    const { monitoringEnabled } = get()

    if (!monitoringEnabled) {
      return
    }

    console.warn('🔮 Starting predictive connection recovery monitoring')

    // Monitor for failure prediction every 30 seconds
    const predictiveInterval = setInterval(() => {
      const stats = get().getConnectionStats()
      const { connectionState, consecutiveFailures } = get()

      // Predictive failure indicators
      const failureIndicators = [
        stats.failurePrediction > 0.8, // High failure prediction
        stats.stability < 0.2, // Very unstable
        consecutiveFailures >= 3, // Multiple recent failures
        stats.heartbeatHealth < 0.3, // Poor heartbeat health
        connectionState === 'degraded' && stats.averageLatency > 10000, // Degraded with high latency
      ]

      const failureRisk = failureIndicators.filter(Boolean).length / failureIndicators.length

      if (failureRisk > 0.6) {
        console.warn(
          `🚨 High failure risk detected (${(failureRisk * 100).toFixed(1)}%) - initiating preventive recovery`,
        )
        get().executePreventiveRecovery()
      } else if (failureRisk > 0.4) {
        console.warn(
          `⚠️ Moderate failure risk detected (${(failureRisk * 100).toFixed(1)}%) - monitoring closely`,
        )
        // Could implement early warning system here
      }
    }, 30000) // Check every 30 seconds

    // Store cleanup function
    ;(get() as Record<string, unknown>)._predictiveRecoveryInterval = predictiveInterval
  },

  stopPredictiveRecovery: () => {
    const predictiveInterval = (get() as Record<string, unknown>)._predictiveRecoveryInterval
    if (predictiveInterval) {
      clearInterval(predictiveInterval)
      console.warn('🔮 Stopped predictive recovery monitoring')
    }
  },

  executePreventiveRecovery: () => {
    const { connectionState, isReconnecting, circuitBreakerOpen } = get()

    // Don't execute if already recovering or circuit breaker is open
    if (isReconnecting || circuitBreakerOpen || connectionState === 'connecting') {
      return
    }

    console.warn('🛡️ Executing preventive connection recovery')

    // Check connection health before deciding on recovery strategy
    get()
      .checkConnectionHealth()
      .then(isHealthy => {
        if (!isHealthy) {
          // Network is unhealthy - prepare for potential failure
          get().transitionConnectionState('degraded', 'Preventive recovery: network unhealthy')

          // Pre-emptive connection pooling if enabled
          if (get().multiplexingEnabled) {
            get().prepareBackupConnections()
          }
        } else {
          // Network is healthy but connection might be at risk
          // Send a test message to verify connection
          get().sendConnectionTest()
        }
      })
  },

  prepareBackupConnections: () => {
    const { fallbackUrls, connectionPool, maxPoolSize } = get()

    if (connectionPool.length >= maxPoolSize) {
      console.warn('Connection pool already at maximum capacity')
      return
    }

    console.warn('🔄 Preparing backup connections for preventive recovery')

    // Prepare additional connections in the background
    fallbackUrls.slice(0, 2).forEach((url, index) => {
      if (!connectionPool.some(conn => conn.url === url)) {
        get().addToConnectionPool(url, 2 + index) // Lower priority for backup connections
      }
    })
  },

  sendConnectionTest: () => {
    const { socket, isConnected } = get()

    if (!socket || !isConnected || socket.readyState !== WebSocket.OPEN) {
      return
    }

    try {
      const testMessage = get().compressMessage({
        action: 'connection_test',
        timestamp: Date.now(),
        testId: `test_${Date.now()}`,
      })

      socket.send(JSON.stringify(testMessage))

      // Set up test response timeout
      const testTimeout = setTimeout(() => {
        console.warn('⚠️ Connection test timed out - potential issue detected')
        get().handleConnectionTestFailure()
      }, 5000) // 5 second timeout for test

      // Store test timeout for cleanup
      ;(get() as Record<string, unknown>)._connectionTestTimeout = testTimeout
    } catch (error) {
      console.warn('Failed to send connection test:', error)
      get().handleConnectionTestFailure()
    }
  },

  handleConnectionTestFailure: () => {
    console.warn('🔄 Connection test failed - initiating recovery protocol')

    // Clear test timeout
    const testTimeout = (get() as Record<string, unknown>)._connectionTestTimeout
    if (testTimeout) {
      clearTimeout(testTimeout)
    }

    // Transition to degraded state and prepare for reconnection
    get().transitionConnectionState('degraded', 'Connection test failed')

    // If multiplexing is enabled, switch to backup connection
    if (get().multiplexingEnabled) {
      const backupConnection = get().getBestConnection()
      if (backupConnection && backupConnection !== get().socket) {
        console.warn('🔄 Switching to backup connection')
        set({ socket: backupConnection })
        get().transitionConnectionState('connected', 'Switched to backup connection')
        return
      }
    }

    // Fallback to standard reconnection
    setTimeout(() => {
      const { currentUrl } = get()
      if (currentUrl) {
        get().connectWebSocket(currentUrl)
      }
    }, 1000)
  },

  // Enhanced connection quality prediction
  predictConnectionFailure: (): { risk: number; reasons: string[]; recommendedAction: string } => {
    const stats = get().getConnectionStats()
    const { connectionHistory, consecutiveFailures, connectionQuality, heartbeatHealthScore } =
      get()

    const reasons: string[] = []
    let riskScore = 0

    // Analyze recent history
    const recentHistory = connectionHistory.slice(-10)
    const recentFailures = recentHistory.filter(h => !h.success).length
    const recentFailureRate = recentFailures / recentHistory.length

    if (recentFailureRate > 0.5) {
      riskScore += 0.3
      reasons.push('High recent failure rate')
    }

    // Analyze stability
    if (stats.stability < 0.3) {
      riskScore += 0.25
      reasons.push('Low connection stability')
    }

    // Analyze consecutive failures
    if (consecutiveFailures >= 3) {
      riskScore += 0.2
      reasons.push('Multiple consecutive failures')
    }

    // Analyze heartbeat health
    if (heartbeatHealthScore < 0.4) {
      riskScore += 0.15
      reasons.push('Poor heartbeat health')
    }

    // Analyze connection quality
    if (connectionQuality === 'critical' || connectionQuality === 'poor') {
      riskScore += 0.1
      reasons.push('Poor connection quality')
    }

    // Determine recommended action
    let recommendedAction = 'monitor'
    if (riskScore > 0.7) {
      recommendedAction = 'immediate_recovery'
    } else if (riskScore > 0.5) {
      recommendedAction = 'preventive_measures'
    } else if (riskScore > 0.3) {
      recommendedAction = 'enhanced_monitoring'
    }

    return {
      risk: Math.min(1, riskScore),
      reasons,
      recommendedAction,
    }
  },

  getConnectionStats: () => {
    const { connectionHistory, lastHeartbeat } = get()
    const heartbeatStats = get().getHeartbeatStats()

    if (connectionHistory.length === 0) {
      return {
        stability: 1.0,
        averageLatency: heartbeatStats.averageLatency,
        successRate: heartbeatStats.successRate,
        packetLoss: 0,
        bandwidthEstimate: 1000000, // 1Mbps default
        failurePrediction: 0,
        jitter: 0,
        heartbeatHealth: heartbeatStats.healthScore,
      }
    }

    const recentHistory = connectionHistory.slice(-20) // Last 20 connections for better analysis
    const successfulConnections = recentHistory.filter(h => h.success)
    const successRate = successfulConnections.length / recentHistory.length

    // Calculate stability as a combination of success rate and consistency
    const timeWindow = 5 * 60 * 1000 // 5 minutes
    const recentEvents = recentHistory.filter(h => Date.now() - h.timestamp < timeWindow)
    const stability =
      recentEvents.length > 0
        ? (successfulConnections.length / recentHistory.length) *
          Math.min(1.0, recentEvents.length / 5) // Prefer more data points
        : 0.5 // Default moderate stability with no data

    // Calculate average latency from successful connections
    const latencies = successfulConnections
      .filter(h => h.latency !== undefined)
      .map(h => h.latency as number)
    const averageLatency =
      latencies.length > 0 ? latencies.reduce((sum, l) => sum + l, 0) / latencies.length : 0

    // Calculate jitter (latency variation)
    const jitter =
      latencies.length > 1
        ? Math.sqrt(
            latencies.reduce((sum, latency) => sum + Math.pow(latency - averageLatency, 2), 0) /
              latencies.length,
          )
        : 0

    // Estimate packet loss based on connection failures and heartbeat patterns
    const failedConnections = recentHistory.filter(h => !h.success)
    const packetLoss = failedConnections.length / recentHistory.length

    // Estimate bandwidth based on latency and connection type
    const { effectiveConnectionType } = get()
    let bandwidthEstimate = 1000000 // Default 1Mbps

    switch (effectiveConnectionType) {
      case '4g':
        bandwidthEstimate = 10000000 // 10Mbps
        break
      case '3g':
        bandwidthEstimate = 1000000 // 1Mbps
        break
      case '2g':
        bandwidthEstimate = 500000 // 500Kbps
        break
      case 'slow-2g':
        bandwidthEstimate = 100000 // 100Kbps
        break
      default:
        // Estimate based on latency
        if (averageLatency < 50) {
          bandwidthEstimate = 50000000 // 50Mbps
        } else if (averageLatency < 100) {
          bandwidthEstimate = 10000000 // 10Mbps
        } else if (averageLatency < 200) {
          bandwidthEstimate = 5000000 // 5Mbps
        }
        break
    }

    // Adjust bandwidth estimate based on stability
    bandwidthEstimate *= stability

    // Predictive failure indicator based on recent trends
    const veryRecentHistory = recentHistory.slice(-5) // Last 5 attempts
    const recentFailures = veryRecentHistory.filter(h => !h.success).length
    let failurePrediction = recentFailures / veryRecentHistory.length

    // Additional factors for failure prediction
    const timeSinceLastFailure =
      successfulConnections.length > 0
        ? Date.now() - successfulConnections[successfulConnections.length - 1].timestamp
        : 0
    const timeSinceHeartbeat = Date.now() - lastHeartbeat

    // Increase prediction if no successful connections recently or heartbeat is stale
    if (timeSinceLastFailure > 300000) {
      // 5 minutes
      failurePrediction *= 1.5
    }
    if (timeSinceHeartbeat > 120000) {
      // 2 minutes
      failurePrediction *= 2.0
    }

    return {
      stability: Math.max(0, Math.min(1, stability)),
      averageLatency: heartbeatStats.averageLatency || averageLatency,
      successRate,
      packetLoss: Math.max(0, Math.min(1, packetLoss)),
      bandwidthEstimate: Math.max(10000, bandwidthEstimate), // Minimum 10Kbps
      failurePrediction: Math.max(0, Math.min(1, failurePrediction)),
      jitter,
      heartbeatHealth: heartbeatStats.healthScore,
    }
  },

  getSmartRetryStrategy: (errorType: ConnectionErrorType, attemptCount: number) => {
    const maxAttempts = 5
    const baseDelay = 1000

    if (attemptCount >= maxAttempts) {
      return {
        shouldRetry: false,
        delay: 0,
        priority: 'low' as const,
        reason: 'Maximum retry attempts exceeded',
      }
    }

    const delay = Math.min(baseDelay * Math.pow(2, attemptCount), 30000)
    const priority = attemptCount < 2 ? 'high' : attemptCount < 4 ? 'medium' : 'low'

    return {
      shouldRetry: true,
      delay,
      priority: priority as 'high' | 'medium' | 'low',
      reason: `Retry attempt ${attemptCount + 1} with ${delay}ms delay`,
    }
  },

  updateCircuitBreakerState: () => {
    const { connectionAttempts, maxReconnectAttempts } = get()
    const failureRate = connectionAttempts / maxReconnectAttempts

    if (failureRate > 0.8) {
      set({ connectionQuality: 'poor' })
    } else if (failureRate > 0.5) {
      set({ connectionQuality: 'good' })
    } else {
      set({ connectionQuality: 'excellent' })
    }
  },

  getRetryBudgetStatus: () => {
    const { connectionAttempts, maxReconnectAttempts } = get()
    const remainingRetries = Math.max(0, maxReconnectAttempts - connectionAttempts)
    const budgetResetTime = Date.now() + 60 * 1000 // Reset in 1 minute
    const isBudgetExceeded = connectionAttempts >= maxReconnectAttempts

    return {
      remainingRetries,
      budgetResetTime,
      isBudgetExceeded,
    }
  },
}))
