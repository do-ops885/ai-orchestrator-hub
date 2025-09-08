'use client'

import { useState, useEffect } from 'react'

interface SystemResources {
  cpu_cores: number;
  available_memory: number;
  cpu_usage: number;
  memory_usage: number;
  simd_capabilities: string[];
  last_updated: string;
}

interface ResourceProfile {
  profile_name: string;
  max_agents: number;
  neural_complexity: number;
  batch_size: number;
  update_frequency: number;
}

export function ResourceMonitor() {
  const [systemResources, setSystemResources] = useState<SystemResources | null>(null)
  const [resourceProfile, setResourceProfile] = useState<ResourceProfile | null>(null)
  const [hardwareClass, setHardwareClass] = useState<string>('Unknown')

  useEffect(() => {
    // Fetch resource information from backend
    const fetchResourceInfo = async () => {
      try {
        const apiUrl = process.env.NEXT_PUBLIC_API_URL ?? 'http://localhost:3001'
        const response = await fetch(`${apiUrl}/api/resources`)
        if (response.ok) {
          const data = await response.json()
          setSystemResources(data.system_resources)
          setResourceProfile(data.resource_profile)
          setHardwareClass(data.hardware_class)
        }
      } catch (error) {
        console.warn('Failed to fetch resource info:', error)
      }
    }

    fetchResourceInfo()
    const interval = setInterval(fetchResourceInfo, 30000) // Update every 30 seconds

    return () => clearInterval(interval)
  }, [])

  if (systemResources === null || resourceProfile === null) {
    return (
      <div className="bg-white overflow-hidden shadow rounded-lg">
        <div className="px-4 py-5 sm:p-6">
          <h3 className="text-lg leading-6 font-medium text-gray-900 mb-4">
            🖥️ System Resources
          </h3>
          <div className="text-gray-500">Loading resource information...</div>
        </div>
      </div>
    )
  }

  const getUsageColor = (usage: number) => {
    if (usage < 50) {return 'text-green-600 bg-green-50'}
    if (usage < 80) {return 'text-yellow-600 bg-yellow-50'}
    return 'text-red-600 bg-red-50'
  }

  const getHardwareIcon = (hwClass: string) => {
    switch (hwClass) {
      case 'EdgeDevice': return '📱'
      case 'Desktop': return '🖥️'
      case 'Server': return '🖥️'
      case 'Cloud': return '☁️'
      default: return '💻'
    }
  }

  return (
    <div className="bg-white overflow-hidden shadow rounded-lg">
      <div className="px-4 py-5 sm:p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg leading-6 font-medium text-gray-900">
            🖥️ System Resources
          </h3>
          <div className="flex items-center space-x-2">
            <span className="text-2xl">{getHardwareIcon(hardwareClass)}</span>
            <span className="text-sm font-medium text-gray-600">{hardwareClass}</span>
          </div>
        </div>

        {/* CPU-Native Badge */}
        <div className="mb-4">
          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
            🚀 CPU-Native, GPU-Optional
          </span>
        </div>

        {/* System Metrics */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
          <div className={`p-4 rounded-lg ${getUsageColor(systemResources.cpu_usage)}`}>
            <div className="flex items-center justify-between">
              <div>
                <div className="text-sm font-medium">CPU Usage</div>
                <div className="text-2xl font-bold">{systemResources.cpu_usage.toFixed(1)}%</div>
              </div>
              <div className="text-sm text-gray-600">
                {systemResources.cpu_cores} cores
              </div>
            </div>
          </div>

          <div className={`p-4 rounded-lg ${getUsageColor(systemResources.memory_usage)}`}>
            <div className="flex items-center justify-between">
              <div>
                <div className="text-sm font-medium">Memory Usage</div>
                <div className="text-2xl font-bold">{systemResources.memory_usage.toFixed(1)}%</div>
              </div>
              <div className="text-sm text-gray-600">
                {(systemResources.available_memory / 1_000_000_000).toFixed(1)}GB
              </div>
            </div>
          </div>
        </div>

        {/* Resource Profile */}
        <div className="border-t pt-4">
          <h4 className="text-md font-medium text-gray-900 mb-3">
            ⚡ Current Profile: {resourceProfile.profile_name}
          </h4>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
            <div>
              <div className="text-gray-500">Max Agents</div>
              <div className="font-medium">{resourceProfile.max_agents}</div>
            </div>
            <div>
              <div className="text-gray-500">Neural Complexity</div>
              <div className="font-medium">{(resourceProfile.neural_complexity * 100).toFixed(0)}%</div>
            </div>
            <div>
              <div className="text-gray-500">Batch Size</div>
              <div className="font-medium">{resourceProfile.batch_size}</div>
            </div>
            <div>
              <div className="text-gray-500">Update Freq</div>
              <div className="font-medium">{resourceProfile.update_frequency}ms</div>
            </div>
          </div>
        </div>

        {/* SIMD Capabilities */}
        {systemResources.simd_capabilities.length > 0 && (
          <div className="border-t pt-4 mt-4">
            <h4 className="text-md font-medium text-gray-900 mb-2">
              🔧 CPU Optimizations
            </h4>
            <div className="flex flex-wrap gap-2">
              {systemResources.simd_capabilities.map((capability, index) => (
                <span
                  key={index}
                  className="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-green-100 text-green-800"
                >
                  {capability}
                </span>
              ))}
            </div>
          </div>
        )}

        {/* Phase 2 Status */}
        <div className="border-t pt-4 mt-4">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium text-gray-900">Phase 2 Status</span>
            <span className="inline-flex items-center px-2 py-1 rounded text-xs font-medium bg-green-100 text-green-800">
              ✅ Active
            </span>
          </div>
          <div className="text-xs text-gray-500 mt-1">
            Intelligent resource management and auto-optimization enabled
          </div>
        </div>
      </div>
    </div>
  )
}
