'use client'

import { HiveMetrics } from '@/store/hiveStore'

interface MetricsPanelProps {
  metrics: HiveMetrics;
}

export function MetricsPanel({ metrics }: MetricsPanelProps) {
  const formatPercentage = (value: number) => `${(value * 100).toFixed(1)}%`

  const metricItems = [
    {
      label: 'Average Performance',
      value: formatPercentage(metrics.average_performance),
      color: 'blue',
      description: 'Overall capability proficiency across all agents',
    },
    {
      label: 'Swarm Cohesion',
      value: formatPercentage(metrics.swarm_cohesion),
      color: 'green',
      description: 'How well agents are coordinated spatially',
    },
    {
      label: 'Learning Progress',
      value: formatPercentage(metrics.learning_progress),
      color: 'purple',
      description: 'Collective learning advancement of the hive',
    },
    {
      label: 'Task Success Rate',
      value: metrics.completed_tasks + metrics.failed_tasks > 0 
        ? formatPercentage(metrics.completed_tasks / (metrics.completed_tasks + metrics.failed_tasks))
        : '0%',
      color: 'yellow',
      description: 'Ratio of successful to total completed tasks',
    },
  ]

  const getColorClasses = (color: string) => {
    const colors = {
      blue: 'bg-blue-50 text-blue-700 border-blue-200',
      green: 'bg-green-50 text-green-700 border-green-200',
      purple: 'bg-purple-50 text-purple-700 border-purple-200',
      yellow: 'bg-yellow-50 text-yellow-700 border-yellow-200',
    }
    return colors[color as keyof typeof colors] ?? colors.blue
  }

  return (
    <div className="bg-white overflow-hidden shadow rounded-lg">
      <div className="px-4 py-5 sm:p-6">
        <h3 className="text-lg leading-6 font-medium text-gray-900 mb-4">
          Hive Metrics
        </h3>
        
        <div className="space-y-4">
          {metricItems.map((item, index) => (
            <div key={index} className={`p-4 rounded-lg border ${getColorClasses(item.color)}`}>
              <div className="flex justify-between items-start">
                <div>
                  <div className="text-sm font-medium">{item.label}</div>
                  <div className="text-xs opacity-75 mt-1">{item.description}</div>
                </div>
                <div className="text-xl font-bold">{item.value}</div>
              </div>
            </div>
          ))}
        </div>

        <div className="mt-6 pt-4 border-t border-gray-200">
          <h4 className="text-sm font-medium text-gray-900 mb-3">Task Statistics</h4>
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span className="text-gray-500">Completed:</span>
              <span className="ml-2 font-medium text-green-600">{metrics.completed_tasks}</span>
            </div>
            <div>
              <span className="text-gray-500">Failed:</span>
              <span className="ml-2 font-medium text-red-600">{metrics.failed_tasks}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}