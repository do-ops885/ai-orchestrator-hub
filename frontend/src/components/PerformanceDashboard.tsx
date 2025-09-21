import React, { useState, useEffect, useCallback } from 'react'
import { AlertTriangle, TrendingUp, TrendingDown, Activity, Cpu, HardDrive, Zap } from 'lucide-react'

// Performance data types matching backend
interface PerformanceDataPoint {
  timestamp: number
  throughput_ops_sec: number
  latency_ms: number
  memory_mb: number
  cpu_utilization: number
  active_connections: number
  error_rate: number
  optimization_score: number
}

interface PerformanceSummary {
  avg_throughput: number
  peak_throughput: number
  avg_latency: number
  p95_latency: number
  avg_memory_mb: number
  peak_memory_mb: number
  health_score: number
  uptime_percent: number
}

interface OptimizationImpact {
  improvement_percent: number
  memory_efficiency_gain: number
  cpu_efficiency_gain: number
  communication_improvement: number
  overall_effectiveness: number
}

interface PerformanceAlert {
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

interface DashboardMetrics {
  current: PerformanceDataPoint
  history: PerformanceDataPoint[]
  summary: PerformanceSummary
  alerts: PerformanceAlert[]
  optimization_impact: OptimizationImpact
}

// Mock data generator for demo purposes
const generateMockData = (): DashboardMetrics => {
  const now = Date.now()
  const history: PerformanceDataPoint[] = []
  
  // Generate 60 data points (1 minute of data)
  for (let i = 59; i >= 0; i--) {
    history.push({
      timestamp: now - (i * 1000),
      throughput_ops_sec: 850 + Math.random() * 100 - 50,
      latency_ms: 85 + Math.random() * 20 - 10,
      memory_mb: 48 + Math.random() * 4 - 2,
      cpu_utilization: 65 + Math.random() * 20 - 10,
      active_connections: 45 + Math.floor(Math.random() * 20),
      error_rate: 0.1 + Math.random() * 0.3 - 0.15,
      optimization_score: 92 + Math.random() * 8 - 4
    })
  }

  const current = history[history.length - 1]
  
  return {
    current,
    history,
    summary: {
      avg_throughput: 847.5,
      peak_throughput: 925.3,
      avg_latency: 84.2,
      p95_latency: 98.7,
      avg_memory_mb: 48.1,
      peak_memory_mb: 52.3,
      health_score: 94.2,
      uptime_percent: 99.97
    },
    alerts: [
      {
        id: '1',
        threshold: {
          name: 'High Latency',
          metric: 'latency',
          threshold: 100
        },
        triggered_at: new Date(now - 300000).toISOString(),
        current_value: 98.7,
        message: 'Latency approaching threshold',
        acknowledged: false
      }
    ],
    optimization_impact: {
      improvement_percent: 84.2,
      memory_efficiency_gain: 30.1,
      cpu_efficiency_gain: 31.3,
      communication_improvement: 47.8,
      overall_effectiveness: 48.4
    }
  }
}

const MetricCard: React.FC<{
  title: string
  value: number | string
  unit?: string
  trend?: number
  icon: React.ReactNode
  color: string
}> = ({ title, value, unit = '', trend, icon, color }) => (
  <div className="bg-white rounded-lg shadow-md p-6 border-l-4" style={{ borderLeftColor: color }}>
    <div className="flex items-center justify-between">
      <div>
        <p className="text-sm font-medium text-gray-600">{title}</p>
        <p className="text-2xl font-bold text-gray-900">
          {typeof value === 'number' ? value.toFixed(1) : value}{unit}
        </p>
        {trend !== undefined && (
          <div className="flex items-center mt-1">
            {trend > 0 ? (
              <TrendingUp className="h-4 w-4 text-green-500 mr-1" />
            ) : (
              <TrendingDown className="h-4 w-4 text-red-500 mr-1" />
            )}
            <span className={`text-sm ${trend > 0 ? 'text-green-600' : 'text-red-600'}`}>
              {Math.abs(trend).toFixed(1)}%
            </span>
          </div>
        )}
      </div>
      <div className="text-3xl" style={{ color }}>
        {icon}
      </div>
    </div>
  </div>
)

const SimpleChart: React.FC<{
  data: number[]
  color: string
  height?: number
}> = ({ data, color, height = 60 }) => {
  const max = Math.max(...data)
  const min = Math.min(...data)
  const range = max - min || 1

  return (
    <div className="w-full" style={{ height }}>
      <svg width="100%" height="100%" viewBox={`0 0 ${data.length * 2} ${height}`}>
        <polyline
          points={data
            .map((value, index) => {
              const x = index * 2
              const y = height - ((value - min) / range) * height
              return `${x},${y}`
            })
            .join(' ')}
          fill="none"
          stroke={color}
          strokeWidth="2"
        />
      </svg>
    </div>
  )
}

const AlertBadge: React.FC<{ alert: PerformanceAlert; onAcknowledge: (id: string) => void }> = ({
  alert,
  onAcknowledge
}) => (
  <div className={`p-3 rounded-lg border-l-4 ${alert.acknowledged ? 'bg-gray-50 border-gray-400' : 'bg-red-50 border-red-400'}`}>
    <div className="flex items-center justify-between">
      <div className="flex items-center">
        <AlertTriangle className={`h-5 w-5 mr-2 ${alert.acknowledged ? 'text-gray-500' : 'text-red-500'}`} />
        <div>
          <p className={`font-medium ${alert.acknowledged ? 'text-gray-700' : 'text-red-800'}`}>
            {alert.threshold.name}
          </p>
          <p className={`text-sm ${alert.acknowledged ? 'text-gray-600' : 'text-red-600'}`}>
            {alert.message}
          </p>
        </div>
      </div>
      {!alert.acknowledged && (
        <button
          onClick={() => onAcknowledge(alert.id)}
          className="px-3 py-1 bg-red-100 text-red-800 rounded text-sm hover:bg-red-200"
        >
          Acknowledge
        </button>
      )}
    </div>
  </div>
)

const PerformanceDashboard: React.FC = () => {
  const [metrics, setMetrics] = useState<DashboardMetrics>(generateMockData())
  const [isConnected, setIsConnected] = useState(false)

  // Simulate real-time updates
  useEffect(() => {
    const interval = setInterval(() => {
      setMetrics(generateMockData())
    }, 1000)

    return () => clearInterval(interval)
  }, [])

  // Mock WebSocket connection status
  useEffect(() => {
    setIsConnected(true)
  }, [])

  const handleAcknowledgeAlert = useCallback((alertId: string) => {
    setMetrics(prev => ({
      ...prev,
      alerts: prev.alerts.map(alert =>
        alert.id === alertId ? { ...alert, acknowledged: true } : alert
      )
    }))
  }, [])

  const throughputData = metrics.history.map(point => point.throughput_ops_sec)
  const latencyData = metrics.history.map(point => point.latency_ms)
  const memoryData = metrics.history.map(point => point.memory_mb)
  const cpuData = metrics.history.map(point => point.cpu_utilization)

  return (
    <div className="min-h-screen bg-gray-100 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center justify-between">
            <h1 className="text-3xl font-bold text-gray-900">Performance Dashboard</h1>
            <div className="flex items-center space-x-2">
              <div className={`w-3 h-3 rounded-full ${isConnected ? 'bg-green-500' : 'bg-red-500'}`} />
              <span className="text-sm text-gray-600">
                {isConnected ? 'Connected' : 'Disconnected'}
              </span>
            </div>
          </div>
          <p className="text-gray-600 mt-2">
            Real-time monitoring of AI Orchestrator Hub performance optimizations
          </p>
        </div>

        {/* Key Metrics Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          <MetricCard
            title="Throughput"
            value={metrics.current.throughput_ops_sec}
            unit=" ops/sec"
            trend={metrics.optimization_impact.improvement_percent}
            icon={<TrendingUp />}
            color="#10B981"
          />
          <MetricCard
            title="Latency"
            value={metrics.current.latency_ms}
            unit="ms"
            trend={-12.3}
            icon={<Zap />}
            color="#F59E0B"
          />
          <MetricCard
            title="Memory Usage"
            value={metrics.current.memory_mb}
            unit="MB"
            trend={-metrics.optimization_impact.memory_efficiency_gain}
            icon={<HardDrive />}
            color="#8B5CF6"
          />
          <MetricCard
            title="CPU Usage"
            value={metrics.current.cpu_utilization}
            unit="%"
            trend={-metrics.optimization_impact.cpu_efficiency_gain}
            icon={<Cpu />}
            color="#EF4444"
          />
        </div>

        {/* Charts Section */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
          {/* Throughput Chart */}
          <div className="bg-white rounded-lg shadow-md p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">
              Throughput Trend (ops/sec)
            </h3>
            <SimpleChart data={throughputData} color="#10B981" height={120} />
            <div className="mt-4 text-sm text-gray-600">
              Last 60 seconds • Peak: {Math.max(...throughputData).toFixed(1)} ops/sec
            </div>
          </div>

          {/* Latency Chart */}
          <div className="bg-white rounded-lg shadow-md p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">
              Latency Trend (ms)
            </h3>
            <SimpleChart data={latencyData} color="#F59E0B" height={120} />
            <div className="mt-4 text-sm text-gray-600">
              Last 60 seconds • P95: {metrics.summary.p95_latency.toFixed(1)}ms
            </div>
          </div>

          {/* Memory Chart */}
          <div className="bg-white rounded-lg shadow-md p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">
              Memory Usage (MB)
            </h3>
            <SimpleChart data={memoryData} color="#8B5CF6" height={120} />
            <div className="mt-4 text-sm text-gray-600">
              Last 60 seconds • Peak: {Math.max(...memoryData).toFixed(1)}MB
            </div>
          </div>

          {/* CPU Chart */}
          <div className="bg-white rounded-lg shadow-md p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">
              CPU Utilization (%)
            </h3>
            <SimpleChart data={cpuData} color="#EF4444" height={120} />
            <div className="mt-4 text-sm text-gray-600">
              Last 60 seconds • Avg: {metrics.summary.avg_memory_mb.toFixed(1)}%
            </div>
          </div>
        </div>

        {/* Optimization Impact & Alerts */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Optimization Impact */}
          <div className="bg-white rounded-lg shadow-md p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">
              Optimization Impact
            </h3>
            <div className="space-y-4">
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">Overall Effectiveness</span>
                <span className="font-semibold text-green-600">
                  +{metrics.optimization_impact.overall_effectiveness.toFixed(1)}%
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">Throughput Improvement</span>
                <span className="font-semibold text-green-600">
                  +{metrics.optimization_impact.improvement_percent.toFixed(1)}%
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">Memory Efficiency</span>
                <span className="font-semibold text-blue-600">
                  +{metrics.optimization_impact.memory_efficiency_gain.toFixed(1)}%
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">CPU Efficiency</span>
                <span className="font-semibold text-purple-600">
                  +{metrics.optimization_impact.cpu_efficiency_gain.toFixed(1)}%
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm text-gray-600">Communication Speed</span>
                <span className="font-semibold text-orange-600">
                  +{metrics.optimization_impact.communication_improvement.toFixed(1)}%
                </span>
              </div>
            </div>
          </div>

          {/* Active Alerts */}
          <div className="bg-white rounded-lg shadow-md p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">
              Performance Alerts ({metrics.alerts.filter(a => !a.acknowledged).length} active)
            </h3>
            <div className="space-y-3">
              {metrics.alerts.length > 0 ? (
                metrics.alerts.map(alert => (
                  <AlertBadge
                    key={alert.id}
                    alert={alert}
                    onAcknowledge={handleAcknowledgeAlert}
                  />
                ))
              ) : (
                <div className="text-center py-8 text-gray-500">
                  <Activity className="h-12 w-12 mx-auto mb-2 text-green-500" />
                  <p>All systems operating normally</p>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* System Health Summary */}
        <div className="mt-8 bg-white rounded-lg shadow-md p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">
            System Health Summary
          </h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="text-center">
              <p className="text-2xl font-bold text-green-600">
                {metrics.summary.health_score.toFixed(1)}%
              </p>
              <p className="text-sm text-gray-600">Health Score</p>
            </div>
            <div className="text-center">
              <p className="text-2xl font-bold text-blue-600">
                {metrics.summary.uptime_percent.toFixed(2)}%
              </p>
              <p className="text-sm text-gray-600">Uptime</p>
            </div>
            <div className="text-center">
              <p className="text-2xl font-bold text-purple-600">
                {metrics.current.active_connections}
              </p>
              <p className="text-sm text-gray-600">Active Connections</p>
            </div>
            <div className="text-center">
              <p className="text-2xl font-bold text-orange-600">
                {metrics.current.error_rate.toFixed(2)}%
              </p>
              <p className="text-sm text-gray-600">Error Rate</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default PerformanceDashboard