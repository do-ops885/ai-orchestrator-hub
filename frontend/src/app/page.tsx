'use client'

import { useState, useEffect } from 'react'
import { HiveDashboard } from '@/components/HiveDashboard'
import { AgentManager } from '@/components/AgentManager'
import { TaskManager } from '@/components/TaskManager'
import { useHiveStore } from '@/store/hiveStore'

export default function Home() {
  const [activeTab, setActiveTab] = useState('dashboard')
  const [mounted, setMounted] = useState(false)
  const { connectWebSocket, disconnect, isConnected } = useHiveStore()

  useEffect(() => {
    setMounted(true)
    // Use the correct backend URL from environment variable
    const wsUrl = process.env.NEXT_PUBLIC_WS_URL ?? 'ws://localhost:3001/ws'
    connectWebSocket(wsUrl)

    return () => {
      disconnect()
    }
  }, [connectWebSocket, disconnect])

  // Prevent hydration mismatch by not rendering connection status until mounted
  if (!mounted) {
    return (
      <div className="min-h-screen bg-gray-100">
        <header className="bg-white shadow-sm border-b">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="flex justify-between items-center py-4">
              <div className="flex items-center space-x-4">
                <h1 className="text-2xl font-bold text-gray-900">🐝 Multiagent Hive System</h1>
                <div className="px-2 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-600">
                  Connecting...
                </div>
              </div>
              <nav className="flex space-x-4">
                {['dashboard', 'agents', 'tasks'].map((tab) => (
                  <button
                    key={tab}
                    onClick={() => setActiveTab(tab)}
                    className={`px-3 py-2 rounded-md text-sm font-medium ${
                      activeTab === tab
                        ? 'bg-blue-100 text-blue-700'
                        : 'text-gray-500 hover:text-gray-700'
                    }`}
                  >
                    {tab.charAt(0).toUpperCase() + tab.slice(1)}
                  </button>
                ))}
              </nav>
            </div>
          </div>
        </header>
        <main className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
          <div className="flex items-center justify-center h-64">
            <div className="text-gray-500">Loading...</div>
          </div>
        </main>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-100">
      <header className="bg-white shadow-sm border-b">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-4">
            <div className="flex items-center space-x-4">
              <h1 className="text-2xl font-bold text-gray-900">🐝 Multiagent Hive System</h1>
              <div className={`px-2 py-1 rounded-full text-xs font-medium ${
                isConnected === true ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
              }`}>
                {isConnected === true ? 'Connected' : 'Disconnected'}
              </div>
            </div>
            <nav className="flex space-x-4">
              {['dashboard', 'agents', 'tasks'].map((tab) => (
                <button
                  key={tab}
                  onClick={() => setActiveTab(tab)}
                  className={`px-3 py-2 rounded-md text-sm font-medium ${
                    activeTab === tab
                      ? 'bg-blue-100 text-blue-700'
                      : 'text-gray-500 hover:text-gray-700'
                  }`}
                >
                  {tab.charAt(0).toUpperCase() + tab.slice(1)}
                </button>
              ))}
            </nav>
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
        {activeTab === 'dashboard' && <HiveDashboard />}
        {activeTab === 'agents' && <AgentManager />}
        {activeTab === 'tasks' && <TaskManager />}
      </main>
    </div>
  )
}
