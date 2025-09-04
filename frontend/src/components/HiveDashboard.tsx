'use client'

import React from 'react'
import { useHiveStore } from '@/store/hiveStore'
import { SwarmVisualization } from './SwarmVisualization'
import { MetricsPanel } from './MetricsPanel'
import { NeuralMetrics } from './NeuralMetrics'
import { ResourceMonitor } from './ResourceMonitor'

export const HiveDashboard = React.memo(function HiveDashboard() {
  const { hiveStatus, agents } = useHiveStore()

  if (hiveStatus === null || hiveStatus === undefined) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-gray-500">Loading hive status...</div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="bg-white overflow-hidden shadow rounded-lg">
        <div className="px-4 py-5 sm:p-6">
          <h3 className="text-lg leading-6 font-medium text-gray-900 mb-4">
            Hive Overview
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="bg-blue-50 p-4 rounded-lg">
              <div className="text-2xl font-bold text-blue-600">
                {hiveStatus.metrics.total_agents}
              </div>
              <div className="text-sm text-blue-600">Total Agents</div>
            </div>
            <div className="bg-green-50 p-4 rounded-lg">
              <div className="text-2xl font-bold text-green-600">
                {hiveStatus.metrics.active_agents}
              </div>
              <div className="text-sm text-green-600">Active Agents</div>
            </div>
            <div className="bg-purple-50 p-4 rounded-lg">
              <div className="text-2xl font-bold text-purple-600">
                {hiveStatus.metrics.completed_tasks}
              </div>
              <div className="text-sm text-purple-600">Completed Tasks</div>
            </div>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <SwarmVisualization agents={agents} swarmCenter={hiveStatus.swarm_center} />
        <MetricsPanel metrics={hiveStatus.metrics} />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mt-6">
        <NeuralMetrics agents={agents} />
        <ResourceMonitor />
      </div>
    </div>
  )
})
