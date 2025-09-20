/**
 * @jest-environment jsdom
 */
import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import { Badge } from './ui/badge';
import { Button } from './ui/button';
import { Alert, AlertDescription } from './ui/alert';
import {
  Activity,
  AlertTriangle,
  Cpu,
  HardDrive,
  MemoryStick,
  Network,
  RefreshCw,
  TrendingUp,
  Users,
  Zap
} from 'lucide-react';

interface SystemMetrics {
  timestamp: string;
  cpu_usage: number;
  memory_usage: number;
  disk_usage: number;
  network_io: {
    bytes_sent: number;
    bytes_received: number;
    connections_active: number;
    latency_ms: number;
  };
  agent_metrics: {
    total_agents: number;
    active_agents: number;
    idle_agents: number;
    failed_agents: number;
    average_response_time: number;
    agent_health_scores: Record<string, number>;
  };
  swarm_metrics: {
    total_tasks: number;
    completed_tasks: number;
    failed_tasks: number;
    pending_tasks: number;
    average_task_duration: number;
    task_success_rate: number;
    load_distribution: Record<string, number>;
  };
  performance_metrics: {
    throughput: number;
    latency_p50: number;
    latency_p95: number;
    latency_p99: number;
    error_rate: number;
    resource_utilization: number;
  };
}

interface Alert {
  id: string;
  title: string;
  description: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  source: string;
  timestamp: string;
  acknowledged: boolean;
  resolved: boolean;
}

interface HealthStatus {
  status: 'healthy' | 'unhealthy';
  timestamp: string;
  uptime_seconds: number;
  components: Record<string, any>;
  performance: Record<string, number>;
}

const MonitoringDashboard: React.FC = () => {
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null);
  const [alerts, setAlerts] = useState<Alert[]>([]);
  const [health, setHealth] = useState<HealthStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());

  // Fetch monitoring data
  const fetchMonitoringData = async () => {
    try {
      setLoading(true);
      setError(null);

      const [metricsRes, alertsRes, healthRes] = await Promise.all([
        fetch('/api/monitoring/metrics'),
        fetch('/api/monitoring/alerts'),
        fetch('/api/monitoring/health')
      ]);

      if (!metricsRes.ok || !alertsRes.ok || !healthRes.ok) {
        throw new Error('Failed to fetch monitoring data');
      }

      const [metricsData, alertsData, healthData] = await Promise.all([
        metricsRes.json(),
        alertsRes.json(),
        healthRes.json()
      ]);

      setMetrics(metricsData.current || null);
      setAlerts(alertsData.alerts || []);
      setHealth(healthData);
      setLastUpdate(new Date());
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchMonitoringData();

    // Auto-refresh every 30 seconds
    const interval = setInterval(fetchMonitoringData, 30000);
    return () => clearInterval(interval);
  }, []);

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical': return 'bg-red-500';
      case 'high': return 'bg-orange-500';
      case 'medium': return 'bg-yellow-500';
      case 'low': return 'bg-blue-500';
      default: return 'bg-gray-500';
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'healthy': return 'text-green-600';
      case 'unhealthy': return 'text-red-600';
      default: return 'text-gray-600';
    }
  };

  const formatBytes = (bytes: number) => {
    const units = ['B', 'KB', 'MB', 'GB'];
    let value = bytes;
    let unitIndex = 0;
    while (value >= 1024 && unitIndex < units.length - 1) {
      value /= 1024;
      unitIndex++;
    }
    return `${value.toFixed(1)} ${units[unitIndex]}`;
  };

  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${ms.toFixed(0)}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  };

  if (loading && !metrics) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        <span className="ml-2">Loading monitoring data...</span>
      </div>
    );
  }

  return (
    <div className="space-y-6 p-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">System Monitoring</h1>
          <p className="text-gray-600">
            Real-time system metrics and health monitoring
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <span className="text-sm text-gray-500">
            Last updated: {lastUpdate.toLocaleTimeString()}
          </span>
          <Button
            onClick={fetchMonitoringData}
            disabled={loading}
            variant="outline"
            size="sm"
          >
            <RefreshCw className={`h-4 w-4 mr-2 ${loading ? 'animate-spin' : ''}`} />
            Refresh
          </Button>
        </div>
      </div>

      {error && (
        <Alert className="border-red-200 bg-red-50">
          <AlertTriangle className="h-4 w-4 text-red-600" />
          <AlertDescription className="text-red-800">
            Failed to load monitoring data: {error}
          </AlertDescription>
        </Alert>
      )}

      {/* System Overview */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">CPU Usage</CardTitle>
            <Cpu className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {metrics?.cpu_usage.toFixed(1) || 'N/A'}%
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2 mt-2">
              <div
                className="bg-blue-600 h-2 rounded-full"
                style={{ width: `${Math.min(metrics?.cpu_usage || 0, 100)}%` }}
              ></div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Memory Usage</CardTitle>
            <MemoryStick className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {metrics?.memory_usage.toFixed(1) || 'N/A'}%
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2 mt-2">
              <div
                className="bg-green-600 h-2 rounded-full"
                style={{ width: `${Math.min(metrics?.memory_usage || 0, 100)}%` }}
              ></div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Agents</CardTitle>
            <Users className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {metrics?.agent_metrics.active_agents || 0}
            </div>
            <p className="text-xs text-muted-foreground">
              of {metrics?.agent_metrics.total_agents || 0} total
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Task Success Rate</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {metrics ? (metrics.swarm_metrics.task_success_rate * 100).toFixed(1) : 'N/A'}%
            </div>
            <p className="text-xs text-muted-foreground">
              {metrics?.swarm_metrics.completed_tasks || 0} completed
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Performance Metrics */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center">
              <Zap className="h-5 w-5 mr-2" />
              Performance Metrics
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <p className="text-sm font-medium">Throughput</p>
                <p className="text-2xl font-bold">
                  {metrics?.performance_metrics.throughput.toFixed(1) || 'N/A'}
                </p>
                <p className="text-xs text-muted-foreground">tasks/sec</p>
              </div>
              <div>
                <p className="text-sm font-medium">Error Rate</p>
                <p className="text-2xl font-bold">
                  {(metrics?.performance_metrics.error_rate || 0 * 100).toFixed(2)}%
                </p>
                <p className="text-xs text-muted-foreground">per minute</p>
              </div>
            </div>

            <div className="space-y-2">
              <div className="flex justify-between">
                <span className="text-sm">Latency P50</span>
                <span className="font-medium">
                  {metrics ? formatDuration(metrics.performance_metrics.latency_p50) : 'N/A'}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-sm">Latency P95</span>
                <span className="font-medium">
                  {metrics ? formatDuration(metrics.performance_metrics.latency_p95) : 'N/A'}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-sm">Latency P99</span>
                <span className="font-medium">
                  {metrics ? formatDuration(metrics.performance_metrics.latency_p99) : 'N/A'}
                </span>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center">
              <Network className="h-5 w-5 mr-2" />
              Network & Resources
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <p className="text-sm font-medium">Data Sent</p>
                <p className="text-lg font-bold">
                  {metrics ? formatBytes(metrics.network_io.bytes_sent) : 'N/A'}
                </p>
              </div>
              <div>
                <p className="text-sm font-medium">Data Received</p>
                <p className="text-lg font-bold">
                  {metrics ? formatBytes(metrics.network_io.bytes_received) : 'N/A'}
                </p>
              </div>
            </div>

            <div className="space-y-2">
              <div className="flex justify-between">
                <span className="text-sm">Active Connections</span>
                <span className="font-medium">
                  {metrics?.network_io.connections_active || 0}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-sm">Network Latency</span>
                <span className="font-medium">
                  {metrics ? `${metrics.network_io.latency_ms.toFixed(1)}ms` : 'N/A'}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-sm">Disk Usage</span>
                <span className="font-medium">
                  {metrics?.disk_usage.toFixed(1) || 'N/A'}%
                </span>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Active Alerts */}
      {alerts.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center">
              <AlertTriangle className="h-5 w-5 mr-2 text-orange-500" />
              Active Alerts ({alerts.filter(a => !a.resolved).length})
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {alerts.filter(alert => !alert.resolved).map((alert) => (
                <div key={alert.id} className="flex items-start space-x-3 p-3 border rounded-lg">
                  <div className={`w-3 h-3 rounded-full mt-1 ${getSeverityColor(alert.severity)}`} />
                  <div className="flex-1">
                    <div className="flex items-center justify-between">
                      <h4 className="font-medium">{alert.title}</h4>
                      <Badge variant="outline" className="capitalize">
                        {alert.severity}
                      </Badge>
                    </div>
                    <p className="text-sm text-gray-600 mt-1">{alert.description}</p>
                    <div className="flex items-center space-x-4 mt-2 text-xs text-gray-500">
                      <span>Source: {alert.source}</span>
                      <span>{new Date(alert.timestamp).toLocaleString()}</span>
                      {alert.acknowledged && (
                        <Badge variant="secondary">Acknowledged</Badge>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* System Health */}
      {health && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center">
              <Activity className="h-5 w-5 mr-2" />
              System Health
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
              <div className="text-center">
                <div className={`text-2xl font-bold ${getStatusColor(health.status)}`}>
                  {health.status.toUpperCase()}
                </div>
                <p className="text-sm text-gray-600">Overall Status</p>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold">
                  {Math.floor(health.uptime_seconds / 3600)}h {Math.floor((health.uptime_seconds % 3600) / 60)}m
                </div>
                <p className="text-sm text-gray-600">Uptime</p>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold">
                  {Object.keys(health.components).length}
                </div>
                <p className="text-sm text-gray-600">Components</p>
              </div>
              <div className="text-center">
                <div className="text-2xl font-bold">
                  {health.performance.response_time_p50.toFixed(0)}ms
                </div>
                <p className="text-sm text-gray-600">Avg Response Time</p>
              </div>
            </div>

            <div className="mt-6">
              <h4 className="font-medium mb-3">Component Status</h4>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {Object.entries(health.components).map(([name, component]: [string, any]) => (
                  <div key={name} className="flex items-center justify-between p-3 border rounded-lg">
                    <div>
                      <p className="font-medium capitalize">{name.replace('_', ' ')}</p>
                      <p className={`text-sm ${getStatusColor(component.status)}`}>
                        {component.status}
                      </p>
                    </div>
                    {component.status === 'healthy' ? (
                      <div className="w-3 h-3 bg-green-500 rounded-full" />
                    ) : (
                      <div className="w-3 h-3 bg-red-500 rounded-full" />
                    )}
                  </div>
                ))}
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
};

export default MonitoringDashboard;