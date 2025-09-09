'use client'

import React from 'react'
import { AlertTriangle, Wifi, Server, RefreshCw, Home, Settings, MessageSquare } from 'lucide-react'

interface FallbackUIProps {
  type: 'network' | 'server' | 'component' | 'data' | 'auth' | 'generic'
  title?: string
  message?: string
  onRetry?: () => void
  onGoHome?: () => void
  showDetails?: boolean
  error?: Error
  compact?: boolean
}

export function FallbackUI({
  type,
  title,
  message,
  onRetry,
  onGoHome,
  showDetails = false,
  error,
  compact = false,
}: FallbackUIProps) {
  const getIcon = () => {
    switch (type) {
      case 'network':
        return <Wifi className="h-8 w-8 text-blue-600" />
      case 'server':
        return <Server className="h-8 w-8 text-orange-600" />
      case 'auth':
        return <Settings className="h-8 w-8 text-purple-600" />
      case 'data':
        return <MessageSquare className="h-8 w-8 text-green-600" />
      default:
        return <AlertTriangle className="h-8 w-8 text-red-600" />
    }
  }

  const getDefaultTitle = () => {
    switch (type) {
      case 'network':
        return 'Connection Lost'
      case 'server':
        return 'Server Unavailable'
      case 'auth':
        return 'Authentication Required'
      case 'data':
        return 'Data Unavailable'
      case 'component':
        return 'Component Error'
      default:
        return 'Something went wrong'
    }
  }

  const getDefaultMessage = () => {
    switch (type) {
      case 'network':
        return 'Unable to connect to the server. Please check your internet connection and try again.'
      case 'server':
        return 'The server is currently unavailable. Our team has been notified and is working on a fix.'
      case 'auth':
        return 'You need to sign in to access this feature. Please log in and try again.'
      case 'data':
        return 'Unable to load the requested data. This might be a temporary issue.'
      case 'component':
        return 'This component encountered an error and cannot be displayed properly.'
      default:
        return 'We encountered an unexpected error. Please try again or contact support if the problem persists.'
    }
  }

  const displayTitle = title || getDefaultTitle()
  const displayMessage = message || getDefaultMessage()

  if (compact) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-md p-3">
        <div className="flex items-center">
          <AlertTriangle className="h-4 w-4 text-red-600 mr-2 flex-shrink-0" />
          <div className="flex-1 min-w-0">
            <p className="text-sm text-red-800 font-medium">{displayTitle}</p>
            <p className="text-xs text-red-600 mt-1">{displayMessage}</p>
          </div>
          {onRetry && (
            <button
              onClick={onRetry}
              className="ml-3 bg-red-100 text-red-800 px-2 py-1 rounded text-xs hover:bg-red-200 focus:outline-none focus:ring-2 focus:ring-red-500"
            >
              Retry
            </button>
          )}
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-[200px] flex items-center justify-center p-6">
      <div className="max-w-md w-full text-center">
        <div className="flex justify-center mb-4">
          <div className="bg-gray-100 rounded-full p-3">
            {getIcon()}
          </div>
        </div>

        <h2 className="text-lg font-semibold text-gray-900 mb-2">
          {displayTitle}
        </h2>

        <p className="text-gray-600 mb-6">
          {displayMessage}
        </p>

        {showDetails && error && (
          <div className="bg-gray-50 rounded-md p-4 mb-4 text-left">
            <h3 className="text-sm font-medium text-gray-900 mb-2">
              Error Details
            </h3>
            <details className="text-xs text-gray-600">
              <summary className="cursor-pointer hover:text-gray-900">
                Click to show technical details
              </summary>
              <pre className="mt-2 whitespace-pre-wrap break-words">
                {error.message}
                {error.stack && `\n\n${error.stack}`}
              </pre>
            </details>
          </div>
        )}

        <div className="flex flex-col sm:flex-row gap-3 justify-center">
          {onRetry && (
            <button
              onClick={onRetry}
              className="bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 flex items-center justify-center"
            >
              <RefreshCw className="h-4 w-4 mr-2" />
              Try Again
            </button>
          )}

          {onGoHome && (
            <button
              onClick={onGoHome}
              className="bg-gray-600 text-white px-4 py-2 rounded-md hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 flex items-center justify-center"
            >
              <Home className="h-4 w-4 mr-2" />
              Go Home
            </button>
          )}
        </div>
      </div>
    </div>
  )
}

// Specialized fallback components for common scenarios

export function NetworkErrorFallback({ onRetry }: { onRetry?: () => void }) {
  return (
    <FallbackUI
      type="network"
      onRetry={onRetry}
      message="Please check your internet connection and try again. If the problem persists, contact your network administrator."
    />
  )
}

export function ServerErrorFallback({ onRetry }: { onRetry?: () => void }) {
  return (
    <FallbackUI
      type="server"
      onRetry={onRetry}
      message="Our servers are temporarily unavailable. We're working to restore service as quickly as possible."
    />
  )
}

export function AuthErrorFallback({ onRetry }: { onRetry?: () => void }) {
  return (
    <FallbackUI
      type="auth"
      onRetry={onRetry}
      message="Your session has expired or you don't have permission to access this resource."
    />
  )
}

export function DataErrorFallback({ onRetry }: { onRetry?: () => void }) {
  return (
    <FallbackUI
      type="data"
      onRetry={onRetry}
      message="Unable to load the requested information. This might be due to a temporary data issue."
    />
  )
}

// Loading fallback for when data is being fetched
export function LoadingFallback({ message = 'Loading...' }: { message?: string }) {
  return (
    <div className="min-h-[200px] flex items-center justify-center">
      <div className="text-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4"></div>
        <p className="text-gray-600">{message}</p>
      </div>
    </div>
  )
}

// Skeleton loader for content
export function SkeletonLoader({ lines = 3 }: { lines?: number }) {
  return (
    <div className="animate-pulse">
      {Array.from({ length: lines }).map((_, i) => (
        <div key={i} className="mb-3">
          <div className="h-4 bg-gray-200 rounded w-full mb-2"></div>
          <div className="h-4 bg-gray-200 rounded w-3/4"></div>
        </div>
      ))}
    </div>
  )
}

// Empty state component
export function EmptyState({
  icon: Icon,
  title,
  message,
  action,
}: {
  icon?: React.ComponentType<{ className?: string }>
  title: string
  message: string
  action?: React.ReactNode
}) {
  return (
    <div className="min-h-[200px] flex items-center justify-center p-6">
      <div className="text-center max-w-md">
        {Icon && (
          <div className="flex justify-center mb-4">
            <div className="bg-gray-100 rounded-full p-3">
              <Icon className="h-8 w-8 text-gray-600" />
            </div>
          </div>
        )}

        <h3 className="text-lg font-medium text-gray-900 mb-2">
          {title}
        </h3>

        <p className="text-gray-600 mb-6">
          {message}
        </p>

        {action}
      </div>
    </div>
  )
}