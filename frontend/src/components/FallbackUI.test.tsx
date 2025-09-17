import { render, screen, fireEvent } from '@testing-library/react'
import { vi } from 'vitest'
import {
  FallbackUI,
  NetworkErrorFallback,
  ServerErrorFallback,
  AuthErrorFallback,
  DataErrorFallback,
  LoadingFallback,
  SkeletonLoader,
  EmptyState,
} from './FallbackUI'
import { Wifi, Server, AlertTriangle, Settings, MessageSquare } from 'lucide-react'

describe('FallbackUI', () => {
  describe('FallbackUI Component', () => {
    it('renders network type correctly', () => {
      render(<FallbackUI type="network" />)

      expect(screen.getByText('Connection Lost')).toBeInTheDocument()
      expect(
        screen.getByText(
          'Unable to connect to the server. Please check your internet connection and try again.',
        ),
      ).toBeInTheDocument()
    })

    it('renders server type correctly', () => {
      render(<FallbackUI type="server" />)

      expect(screen.getByText('Server Unavailable')).toBeInTheDocument()
      expect(
        screen.getByText(
          'The server is currently unavailable. Our team has been notified and is working on a fix.',
        ),
      ).toBeInTheDocument()
    })

    it('renders auth type correctly', () => {
      render(<FallbackUI type="auth" />)

      expect(screen.getByText('Authentication Required')).toBeInTheDocument()
      expect(
        screen.getByText(
          'You need to sign in to access this feature. Please log in and try again.',
        ),
      ).toBeInTheDocument()
    })

    it('renders data type correctly', () => {
      render(<FallbackUI type="data" />)

      expect(screen.getByText('Data Unavailable')).toBeInTheDocument()
      expect(
        screen.getByText('Unable to load the requested data. This might be a temporary issue.'),
      ).toBeInTheDocument()
    })

    it('renders component type correctly', () => {
      render(<FallbackUI type="component" />)

      expect(screen.getByText('Component Error')).toBeInTheDocument()
      expect(
        screen.getByText('This component encountered an error and cannot be displayed properly.'),
      ).toBeInTheDocument()
    })

    it('renders generic type correctly', () => {
      render(<FallbackUI type="generic" />)

      expect(screen.getByText('Something went wrong')).toBeInTheDocument()
      expect(
        screen.getByText(
          'We encountered an unexpected error. Please try again or contact support if the problem persists.',
        ),
      ).toBeInTheDocument()
    })

    it('uses custom title and message when provided', () => {
      const customTitle = 'Custom Error'
      const customMessage = 'Custom error message'

      render(<FallbackUI type="generic" title={customTitle} message={customMessage} />)

      expect(screen.getByText(customTitle)).toBeInTheDocument()
      expect(screen.getByText(customMessage)).toBeInTheDocument()
    })

    it('renders retry button when onRetry is provided', () => {
      const mockOnRetry = vi.fn()

      render(<FallbackUI type="network" onRetry={mockOnRetry} />)

      const retryButton = screen.getByText('Try Again')
      expect(retryButton).toBeInTheDocument()

      fireEvent.click(retryButton)
      expect(mockOnRetry).toHaveBeenCalledTimes(1)
    })

    it('renders go home button when onGoHome is provided', () => {
      const mockOnGoHome = vi.fn()

      render(<FallbackUI type="server" onGoHome={mockOnGoHome} />)

      const goHomeButton = screen.getByText('Go Home')
      expect(goHomeButton).toBeInTheDocument()

      fireEvent.click(goHomeButton)
      expect(mockOnGoHome).toHaveBeenCalledTimes(1)
    })

    it('renders error details when showDetails is true and error is provided', () => {
      const testError = new Error('Test error message')
      testError.stack = 'Test stack trace'

      render(<FallbackUI type="generic" showDetails={true} error={testError} />)

      expect(screen.getByText('Error Details')).toBeInTheDocument()

      // Expand the details element
      const summary = screen.getByText('Click to show technical details')
      fireEvent.click(summary)

      // Check that the error message is in the document
      expect(
        screen.getByText((content, element) => {
          return element?.tagName === 'PRE' && content.includes('Test error message')
        }),
      ).toBeInTheDocument()

      // Check that the stack trace is in the document
      expect(
        screen.getByText((content, element) => {
          return element?.tagName === 'PRE' && content.includes('Test stack trace')
        }),
      ).toBeInTheDocument()
    })

    it('does not render error details when showDetails is false', () => {
      const testError = new Error('Test error message')

      render(<FallbackUI type="generic" showDetails={false} error={testError} />)

      expect(screen.queryByText('Error Details')).not.toBeInTheDocument()
    })

    it('renders compact version when compact is true', () => {
      render(<FallbackUI type="component" compact={true} />)

      // Compact version should have different styling
      expect(screen.getByText('Component Error')).toBeInTheDocument()
      expect(
        screen.getByText('This component encountered an error and cannot be displayed properly.'),
      ).toBeInTheDocument()
    })

    it('renders compact retry button when compact and onRetry provided', () => {
      const mockOnRetry = vi.fn()

      render(<FallbackUI type="network" compact={true} onRetry={mockOnRetry} />)

      const retryButton = screen.getByText('Retry')
      expect(retryButton).toBeInTheDocument()

      fireEvent.click(retryButton)
      expect(mockOnRetry).toHaveBeenCalledTimes(1)
    })
  })

  describe('Specialized Fallback Components', () => {
    it('NetworkErrorFallback renders correctly', () => {
      const mockOnRetry = vi.fn()

      render(<NetworkErrorFallback onRetry={mockOnRetry} />)

      expect(screen.getByText('Connection Lost')).toBeInTheDocument()
      expect(
        screen.getByText(
          'Please check your internet connection and try again. If the problem persists, contact your network administrator.',
        ),
      ).toBeInTheDocument()

      const retryButton = screen.getByText('Try Again')
      fireEvent.click(retryButton)
      expect(mockOnRetry).toHaveBeenCalledTimes(1)
    })

    it('ServerErrorFallback renders correctly', () => {
      const mockOnRetry = vi.fn()

      render(<ServerErrorFallback onRetry={mockOnRetry} />)

      expect(screen.getByText('Server Unavailable')).toBeInTheDocument()
      expect(
        screen.getByText(
          "Our servers are temporarily unavailable. We're working to restore service as quickly as possible.",
        ),
      ).toBeInTheDocument()
    })

    it('AuthErrorFallback renders correctly', () => {
      const mockOnRetry = vi.fn()

      render(<AuthErrorFallback onRetry={mockOnRetry} />)

      expect(screen.getByText('Authentication Required')).toBeInTheDocument()
      expect(
        screen.getByText(
          "Your session has expired or you don't have permission to access this resource.",
        ),
      ).toBeInTheDocument()
    })

    it('DataErrorFallback renders correctly', () => {
      const mockOnRetry = vi.fn()

      render(<DataErrorFallback onRetry={mockOnRetry} />)

      expect(screen.getByText('Data Unavailable')).toBeInTheDocument()
      expect(
        screen.getByText(
          'Unable to load the requested information. This might be due to a temporary data issue.',
        ),
      ).toBeInTheDocument()
    })
  })

  describe('LoadingFallback', () => {
    it('renders with default message', () => {
      render(<LoadingFallback />)

      expect(screen.getByText('Loading...')).toBeInTheDocument()
    })

    it('renders with custom message', () => {
      const customMessage = 'Custom loading message'

      render(<LoadingFallback message={customMessage} />)

      expect(screen.getByText(customMessage)).toBeInTheDocument()
    })

    it('renders spinner', () => {
      render(<LoadingFallback />)

      const spinner = document.querySelector('.animate-spin')
      expect(spinner).toBeInTheDocument()
    })
  })

  describe('SkeletonLoader', () => {
    it('renders default number of lines', () => {
      render(<SkeletonLoader />)

      const skeletonLines = document.querySelectorAll('.animate-pulse > div')
      expect(skeletonLines).toHaveLength(3)
    })

    it('renders custom number of lines', () => {
      render(<SkeletonLoader lines={5} />)

      const skeletonLines = document.querySelectorAll('.animate-pulse > div')
      expect(skeletonLines).toHaveLength(5)
    })

    it('renders skeleton structure correctly', () => {
      render(<SkeletonLoader lines={2} />)

      const skeletonContainer = document.querySelector('.animate-pulse')
      expect(skeletonContainer).toBeInTheDocument()

      const skeletonLines = skeletonContainer?.querySelectorAll('.mb-3')
      expect(skeletonLines).toHaveLength(2) // 2 lines
    })
  })

  describe('EmptyState', () => {
    it('renders with required props', () => {
      render(<EmptyState title="No Data" message="There is no data to display" />)

      expect(screen.getByText('No Data')).toBeInTheDocument()
      expect(screen.getByText('There is no data to display')).toBeInTheDocument()
    })

    it('renders with icon', () => {
      render(<EmptyState icon={AlertTriangle} title="Error" message="Something went wrong" />)

      expect(screen.getByText('Error')).toBeInTheDocument()
      expect(screen.getByText('Something went wrong')).toBeInTheDocument()

      // Check if icon is rendered
      const iconContainer = document.querySelector('.bg-gray-100')
      expect(iconContainer).toBeInTheDocument()
    })

    it('renders with action', () => {
      const mockAction = <button>Click me</button>

      render(
        <EmptyState title="Action Required" message="Please take action" action={mockAction} />,
      )

      expect(screen.getByText('Click me')).toBeInTheDocument()
    })

    it('renders without icon when not provided', () => {
      render(<EmptyState title="Simple Message" message="No icon needed" />)

      const iconContainer = document.querySelector('.bg-gray-100')
      expect(iconContainer).not.toBeInTheDocument()
    })
  })
})
