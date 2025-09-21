import { useState, useEffect, useCallback, useRef } from 'react'

export interface PerformanceDataPoint {
  timestamp: number
  throughput_ops_sec: number
  latency_ms: number
  memory_mb: number
  cpu_utilization: number
  active_connections: number
  error_rate: number
  optimization_score: number
}

export interface PerformanceSummary {
  avg_throughput: number
  peak_throughput: number
  avg_latency: number
  p95_latency: number
  avg_memory_mb: number
  peak_memory_mb: number
  health_score: number
  uptime_percent: number
}

export interface OptimizationImpact {
  improvement_percent: number
  memory_efficiency_gain: number
  cpu_efficiency_gain: number
  communication_improvement: number
  overall_effectiveness: number
}

export interface PerformanceAlert {
  id: string
  threshold: {
    name: string
    metric: string
    threshold: number
  }
  triggered_at: string
  current_value: number
  message: string
  acknowledged: boolean
}

export interface DashboardMetrics {
  current: PerformanceDataPoint
  history: PerformanceDataPoint[]
  summary: PerformanceSummary
  alerts: PerformanceAlert[]
  optimization_impact: OptimizationImpact
}

export interface UsePerformanceMetricsResult {
  metrics: DashboardMetrics | null
  isConnected: boolean
  error: string | null
  acknowledgeAlert: (alertId: string) => void
  reconnect: () => void
}

const WEBSOCKET_URL = process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8081/ws'

export const usePerformanceMetrics = (): UsePerformanceMetricsResult => {
  const [metrics, setMetrics] = useState<DashboardMetrics | null>(null)
  const [isConnected, setIsConnected] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const wsRef = useRef<WebSocket | null>(null)
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null)
  const reconnectAttempts = useRef(0)

  const connect = useCallback(() => {
    try {
      const ws = new WebSocket(WEBSOCKET_URL)
      wsRef.current = ws

      ws.onopen = () => {
        console.log('📊 Connected to performance metrics WebSocket')
        setIsConnected(true)
        setError(null)
        reconnectAttempts.current = 0
        
        // Send initial ping
        ws.send(JSON.stringify({ action: 'ping' }))
      }

      ws.onmessage = (event) => {
        try {
          const data: DashboardMetrics = JSON.parse(event.data)
          setMetrics(data)
        } catch (err) {
          console.error('Failed to parse metrics data:', err)
          setError('Failed to parse metrics data')
        }
      }

      ws.onclose = (event) => {
        console.log('📊 WebSocket connection closed:', event.code, event.reason)
        setIsConnected(false)
        wsRef.current = null

        // Attempt to reconnect with exponential backoff
        if (!event.wasClean && reconnectAttempts.current < 10) {
          const delay = Math.min(1000 * Math.pow(2, reconnectAttempts.current), 30000)
          console.log(`🔄 Attempting to reconnect in ${delay}ms...`)
          
          reconnectTimeoutRef.current = setTimeout(() => {
            reconnectAttempts.current++
            connect()
          }, delay)
        } else if (reconnectAttempts.current >= 10) {
          setError('Connection failed after multiple attempts')
        }
      }

      ws.onerror = (event) => {
        console.error('📊 WebSocket error:', event)
        setError('WebSocket connection error')
      }

    } catch (err) {
      console.error('Failed to create WebSocket connection:', err)
      setError('Failed to create WebSocket connection')
    }
  }, [])

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current)
      reconnectTimeoutRef.current = null
    }

    if (wsRef.current) {
      wsRef.current.close(1000, 'User disconnected')
      wsRef.current = null
    }
    
    setIsConnected(false)
  }, [])

  const acknowledgeAlert = useCallback((alertId: string) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      const message = {
        action: 'acknowledge_alert',
        data: { alert_id: alertId }
      }
      wsRef.current.send(JSON.stringify(message))
      
      // Optimistically update local state
      setMetrics(prev => {
        if (!prev) return prev
        return {
          ...prev,
          alerts: prev.alerts.map(alert =>
            alert.id === alertId ? { ...alert, acknowledged: true } : alert
          )
        }
      })
    }
  }, [])

  const reconnect = useCallback(() => {
    disconnect()
    reconnectAttempts.current = 0
    setError(null)
    setTimeout(connect, 1000)
  }, [connect, disconnect])

  // Setup WebSocket connection on mount
  useEffect(() => {
    connect()

    // Cleanup on unmount
    return () => {
      disconnect()
    }
  }, [connect, disconnect])

  // Setup heartbeat
  useEffect(() => {
    if (!isConnected || !wsRef.current) return

    const heartbeat = setInterval(() => {
      if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({ action: 'ping' }))
      }
    }, 30000) // 30 seconds

    return () => clearInterval(heartbeat)
  }, [isConnected])

  return {
    metrics,
    isConnected,
    error,
    acknowledgeAlert,
    reconnect,
  }
}