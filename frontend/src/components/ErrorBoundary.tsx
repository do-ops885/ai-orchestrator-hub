'use client'

import React, { Component, ErrorInfo, ReactNode } from 'react'
import { AlertTriangle, RefreshCw, Home, Bug } from 'lucide-react'

interface Props {
  children: ReactNode
  fallback?: ReactNode
  onError?: (error: Error, errorInfo: ErrorInfo) => void
  showDetails?: boolean
}

interface State {
  hasError: boolean
  error?: Error
  errorInfo?: ErrorInfo
  retryCount: number
}

export class ErrorBoundary extends Component<Props, State> {
  private retryTimeouts: NodeJS.Timeout[] = []

  constructor(props: Props) {
    super(props)
    this.state = {
      hasError: false,
      retryCount: 0,
    }
  }

  static getDerivedStateFromError(error: Error): State {
    return {
      hasError: true,
      error,
      retryCount: 0,
    }
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Log error to console for development
    console.error('ErrorBoundary caught an error:', error, errorInfo)

    // Call custom error handler if provided
    this.props.onError?.(error, errorInfo)

    // Report error to error tracking service (if configured)
    this.reportError(error, errorInfo)

    this.setState({
      error,
      errorInfo,
    })
  }

  componentWillUnmount() {
    // Clear any pending retry timeouts
    this.retryTimeouts.forEach(timeout => clearTimeout(timeout))
  }

  private reportError = (error: Error, errorInfo: ErrorInfo) => {
    // In a real application, you would send this to an error tracking service
    // like Sentry, LogRocket, or Bugsnag

    const errorReport = {
      message: error.message,
      stack: error.stack,
      componentStack: errorInfo.componentStack,
      timestamp: new Date().toISOString(),
      userAgent: navigator.userAgent,
      url: window.location.href,
      retryCount: this.state.retryCount,
    }

    // For now, we'll just log it. In production, send to your error tracking service
    console.error('Error Report:', errorReport)

    // Example: Send to error tracking service
    // if (process.env.NEXT_PUBLIC_ERROR_TRACKING_ENABLED) {
    //   // Send errorReport to your error tracking service
    // }
  }

  private handleRetry = () => {
    this.setState(prevState => ({
      hasError: false,
      error: undefined,
      errorInfo: undefined,
      retryCount: prevState.retryCount + 1,
    }))
  }

  private handleAutoRetry = () => {
    const timeout = setTimeout(() => {
      this.handleRetry()
    }, 2000) // Auto retry after 2 seconds

    this.retryTimeouts.push(timeout)
  }

  private handleGoHome = () => {
    window.location.href = '/'
  }

  render() {
    if (this.state.hasError) {
      // Custom fallback UI
      if (this.props.fallback) {
        return this.props.fallback
      }

      // Default error UI
      return (
        <div className="min-h-screen bg-gray-50 flex items-center justify-center px-4">
          <div className="max-w-md w-full bg-white rounded-lg shadow-lg p-6">
            <div className="flex items-center justify-center mb-4">
              <div className="bg-red-100 rounded-full p-3">
                <AlertTriangle className="h-8 w-8 text-red-600" />
              </div>
            </div>

            <h1 className="text-xl font-semibold text-gray-900 text-center mb-2">
              Something went wrong
            </h1>

            <p className="text-gray-600 text-center mb-6">
              We encountered an unexpected error. Please try refreshing the page or contact support
              if the problem persists.
            </p>

            {this.props.showDetails && this.state.error && (
              <div className="bg-gray-50 rounded-md p-4 mb-4">
                <h3 className="text-sm font-medium text-gray-900 mb-2 flex items-center">
                  <Bug className="h-4 w-4 mr-2" />
                  Error Details
                </h3>
                <details className="text-xs text-gray-600">
                  <summary className="cursor-pointer hover:text-gray-900">
                    Click to show technical details
                  </summary>
                  <pre className="mt-2 whitespace-pre-wrap break-words">
                    {this.state.error.message}
                    {this.state.error.stack && `\n\n${this.state.error.stack}`}
                    {this.state.errorInfo?.componentStack &&
                      `\n\nComponent Stack:\n${this.state.errorInfo.componentStack}`}
                  </pre>
                </details>
              </div>
            )}

            <div className="flex flex-col sm:flex-row gap-3">
              <button
                onClick={this.handleRetry}
                className="flex-1 bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 flex items-center justify-center"
              >
                <RefreshCw className="h-4 w-4 mr-2" />
                Try Again
              </button>

              <button
                onClick={this.handleGoHome}
                className="flex-1 bg-gray-600 text-white px-4 py-2 rounded-md hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 flex items-center justify-center"
              >
                <Home className="h-4 w-4 mr-2" />
                Go Home
              </button>
            </div>

            {this.state.retryCount > 0 && (
              <p className="text-xs text-gray-500 text-center mt-4">
                Retry attempts: {this.state.retryCount}
              </p>
            )}
          </div>
        </div>
      )
    }

    return this.props.children
  }
}

// Hook for functional components to use error boundaries
export function withErrorBoundary<P extends object>(
  Component: React.ComponentType<P>,
  errorBoundaryProps?: Omit<Props, 'children'>,
) {
  const WrappedComponent = (props: P) => (
    <ErrorBoundary {...errorBoundaryProps}>
      <Component {...props} />
    </ErrorBoundary>
  )

  WrappedComponent.displayName = `withErrorBoundary(${Component.displayName || Component.name})`

  return WrappedComponent
}

// Specialized error boundary for async operations
export class AsyncErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props)
    this.state = {
      hasError: false,
      retryCount: 0,
    }
  }

  static getDerivedStateFromError(error: Error): State {
    return {
      hasError: true,
      error,
      retryCount: 0,
    }
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Handle async-specific errors
    if (error.message.includes('NetworkError') || error.message.includes('fetch')) {
      console.error('Network error in async operation:', error)
    } else if (error.message.includes('Timeout')) {
      console.error('Timeout error in async operation:', error)
    } else {
      console.error('Async operation error:', error)
    }

    this.props.onError?.(error, errorInfo)
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="bg-yellow-50 border border-yellow-200 rounded-md p-4">
          <div className="flex">
            <AlertTriangle className="h-5 w-5 text-yellow-400" />
            <div className="ml-3">
              <h3 className="text-sm font-medium text-yellow-800">Operation Failed</h3>
              <div className="mt-2 text-sm text-yellow-700">
                <p>
                  The requested operation could not be completed. This might be due to a network
                  issue or server problem.
                </p>
              </div>
              <div className="mt-4">
                <button
                  onClick={() => this.setState({ hasError: false, error: undefined })}
                  className="bg-yellow-100 text-yellow-800 px-3 py-1 rounded-md text-sm hover:bg-yellow-200 focus:outline-none focus:ring-2 focus:ring-yellow-500"
                >
                  Retry
                </button>
              </div>
            </div>
          </div>
        </div>
      )
    }

    return this.props.children
  }
}
