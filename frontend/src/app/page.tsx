'use client'

import { useState, useEffect } from 'react'
import { HiveDashboard } from '@/components/HiveDashboard'
import { AgentManager } from '@/components/AgentManager'
import { TaskManager } from '@/components/TaskManager'
import { useHiveStore } from '@/store/hiveStore'
import { ErrorBoundary } from '@/components/ErrorBoundary'
import { NetworkErrorFallback, LoadingFallback } from '@/components/FallbackUI'
import { useNetworkRecovery, useErrorRecovery } from '@/hooks/useErrorRecovery'
import { AlertTriangle, Wifi, WifiOff } from 'lucide-react'

export default function Home() {
  const [activeTab, setActiveTab] = useState('dashboard')
  const [mounted, setMounted] = useState(false)
  const { connectWebSocket, disconnect, isConnected } = useHiveStore()
  const { isOnline } = useNetworkRecovery()

  // Error recovery for WebSocket connection
  const { execute: connectWithRetry, state: connectionState } = useErrorRecovery(
    async () => {
      const wsUrl = process.env.NEXT_PUBLIC_WS_URL ?? 'ws://localhost:3001/ws'
      connectWebSocket(wsUrl)
      // Wait a bit for connection to establish
      await new Promise(resolve => setTimeout(resolve, 1000))
      if (!isConnected) {
        throw new Error('WebSocket connection failed')
      }
    },
    {
      maxRetries: 5,
      baseDelay: 2000,
      retryCondition: error => {
        return error.message.includes('connection') || error.message.includes('network')
      },
    },
  )

  useEffect(() => {
    setMounted(true)

    // Attempt initial connection with retry logic
    connectWithRetry().catch(error => {
      console.error('Failed to establish WebSocket connection:', error)
    })

    return () => {
      disconnect()
    }
  }, [connectWithRetry, disconnect])

  // Prevent hydration mismatch by not rendering connection status until mounted
  if (!mounted) {
    return <LoadingFallback message="Initializing application..." />
  }

  // Show network error if offline
  if (!isOnline) {
    return <NetworkErrorFallback onRetry={() => window.location.reload()} />
  }

  // Show connection error with retry option
  if (!isConnected && !connectionState.isRetrying && connectionState.lastError) {
    return (
      <div className="min-h-screen bg-gray-100 flex items-center justify-center px-4">
        <div className="max-w-md w-full bg-white rounded-lg shadow-lg p-6">
          <div className="flex items-center justify-center mb-4">
            <div className="bg-red-100 rounded-full p-3">
              <WifiOff className="h-8 w-8 text-red-600" />
            </div>
          </div>

          <h1 className="text-xl font-semibold text-gray-900 text-center mb-2">
            Connection Failed
          </h1>

          <p className="text-gray-600 text-center mb-6">
            Unable to connect to the Multiagent Hive server. Please check your connection and try
            again.
          </p>

          <div className="flex flex-col gap-3">
            <button
              onClick={() => connectWithRetry()}
              disabled={connectionState.isRetrying}
              className="bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center"
            >
              {connectionState.isRetrying ? (
                <>
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                  Retrying... ({connectionState.retryCount}/5)
                </>
              ) : (
                'Retry Connection'
              )}
            </button>

            <button
              onClick={() => window.location.reload()}
              className="bg-gray-600 text-white px-4 py-2 rounded-md hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2"
            >
              Reload Page
            </button>
          </div>

          {connectionState.lastError && (
            <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-md">
              <p className="text-sm text-red-800">
                <strong>Last error:</strong> {connectionState.lastError.message}
              </p>
            </div>
          )}
        </div>
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

              {/* Connection status with network indicator */}
              <div className="flex items-center space-x-2">
                <div
                  className={`flex items-center px-2 py-1 rounded-full text-xs font-medium ${
                    isConnected === true
                      ? 'bg-green-100 text-green-800'
                      : connectionState.isRetrying
                        ? 'bg-yellow-100 text-yellow-800'
                        : 'bg-red-100 text-red-800'
                  }`}
                >
                  {isConnected === true ? (
                    <Wifi className="h-3 w-3 mr-1" />
                  ) : (
                    <WifiOff className="h-3 w-3 mr-1" />
                  )}
                  {isConnected === true
                    ? 'Connected'
                    : connectionState.isRetrying
                      ? `Reconnecting... (${connectionState.retryCount}/5)`
                      : 'Disconnected'}
                </div>

                {/* Network status */}
                {!isOnline && (
                  <div className="flex items-center px-2 py-1 rounded-full text-xs font-medium bg-orange-100 text-orange-800">
                    <AlertTriangle className="h-3 w-3 mr-1" />
                    Offline
                  </div>
                )}
              </div>
            </div>

            <nav className="flex space-x-4">
              {['dashboard', 'agents', 'tasks'].map(tab => (
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
        <ErrorBoundary
          fallback={
            <div className="bg-red-50 border border-red-200 rounded-md p-4">
              <div className="flex">
                <AlertTriangle className="h-5 w-5 text-red-400" />
                <div className="ml-3">
                  <h3 className="text-sm font-medium text-red-800">Component Error</h3>
                  <div className="mt-2 text-sm text-red-700">
                    <p>
                      The {activeTab} component encountered an error. Please try refreshing the
                      page.
                    </p>
                  </div>
                  <div className="mt-4">
                    <button
                      onClick={() => window.location.reload()}
                      className="bg-red-100 text-red-800 px-3 py-1 rounded-md text-sm hover:bg-red-200"
                    >
                      Refresh Page
                    </button>
                  </div>
                </div>
              </div>
            </div>
          }
        >
          {activeTab === 'dashboard' && <HiveDashboard />}
          {activeTab === 'agents' && <AgentManager />}
          {activeTab === 'tasks' && <TaskManager />}
        </ErrorBoundary>
      </main>
    </div>
  )
}
