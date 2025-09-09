'use client'

import { useEffect, useState } from 'react'
import { ErrorBoundary } from '@/components/ErrorBoundary'
import { setupGlobalErrorHandler } from '@/utils/errorLogger'

interface ClientErrorBoundaryProps {
  children: React.ReactNode
}

export function ClientErrorBoundary({ children }: ClientErrorBoundaryProps) {
  const [mounted, setMounted] = useState(false)

  useEffect(() => {
    setMounted(true)
    setupGlobalErrorHandler()
  }, [])

  if (!mounted) {
    return <>{children}</>
  }

  return (
    <ErrorBoundary
      showDetails={process.env.NODE_ENV === 'development'}
      onError={(error, errorInfo) => {
        // Additional error reporting can be added here
        console.error('Root layout error:', error, errorInfo)
      }}
    >
      {children}
    </ErrorBoundary>
  )
}
