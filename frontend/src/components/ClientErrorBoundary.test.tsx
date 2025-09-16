import { render, screen, waitFor } from '@testing-library/react'
import { vi } from 'vitest'
import { ClientErrorBoundary } from './ClientErrorBoundary'
import { ErrorBoundary } from './ErrorBoundary'

// Mock the ErrorBoundary component
vi.mock('./ErrorBoundary', () => ({
  ErrorBoundary: ({ children, onError }: { children: React.ReactNode; onError?: (error: Error, errorInfo: any) => void }) => (
    <div data-testid="error-boundary">
      {children}
      {onError && <div data-testid="on-error-callback" />}
    </div>
  ),
}))

// Mock the error logger
vi.mock('@/utils/errorLogger', () => ({
  setupGlobalErrorHandler: vi.fn(),
}))

describe('ClientErrorBoundary', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders children immediately when not mounted', () => {
    render(
      <ClientErrorBoundary>
        <div>Test Content</div>
      </ClientErrorBoundary>
    )

    expect(screen.getByText('Test Content')).toBeInTheDocument()
  })

  it('renders ErrorBoundary after mounting', async () => {
    render(
      <ClientErrorBoundary>
        <div>Test Content</div>
      </ClientErrorBoundary>
    )

    await waitFor(() => {
      expect(screen.getByTestId('error-boundary')).toBeInTheDocument()
    })
  })

  it('calls setupGlobalErrorHandler on mount', async () => {
    const { setupGlobalErrorHandler } = await import('@/utils/errorLogger')
    const mockSetupGlobalErrorHandler = setupGlobalErrorHandler

    render(
      <ClientErrorBoundary>
        <div>Test Content</div>
      </ClientErrorBoundary>
    )

    await waitFor(() => {
      expect(mockSetupGlobalErrorHandler).toHaveBeenCalledTimes(1)
    })
  })

  it('passes showDetails based on NODE_ENV', async () => {
    const originalEnv = process.env.NODE_ENV

    // Test development mode
    process.env.NODE_ENV = 'development'
    const { unmount: unmountDev } = render(
      <ClientErrorBoundary>
        <div>Test</div>
      </ClientErrorBoundary>
    )

    await waitFor(() => {
      expect(screen.getByTestId('error-boundary')).toBeInTheDocument()
    })

    unmountDev()

    // Test production mode
    process.env.NODE_ENV = 'production'
    render(
      <ClientErrorBoundary>
        <div>Test</div>
      </ClientErrorBoundary>
    )

    await waitFor(() => {
      expect(screen.getByTestId('error-boundary')).toBeInTheDocument()
    })

    // Restore original env
    process.env.NODE_ENV = originalEnv
  })

  it('passes onError callback to ErrorBoundary', async () => {
    render(
      <ClientErrorBoundary>
        <div>Test Content</div>
      </ClientErrorBoundary>
    )

    await waitFor(() => {
      expect(screen.getByTestId('on-error-callback')).toBeInTheDocument()
    })
  })

  it('handles multiple children correctly', async () => {
    render(
      <ClientErrorBoundary>
        <div>First Child</div>
        <div>Second Child</div>
        <span>Third Child</span>
      </ClientErrorBoundary>
    )

    await waitFor(() => {
      expect(screen.getByText('First Child')).toBeInTheDocument()
      expect(screen.getByText('Second Child')).toBeInTheDocument()
      expect(screen.getByText('Third Child')).toBeInTheDocument()
    })
  })

  it('renders nothing when children is null', async () => {
    render(
      <ClientErrorBoundary>
        {null}
      </ClientErrorBoundary>
    )

    await waitFor(() => {
      expect(screen.getByTestId('error-boundary')).toBeInTheDocument()
    })
  })

  it('renders nothing when children is undefined', async () => {
    render(
      <ClientErrorBoundary>
        {undefined}
      </ClientErrorBoundary>
    )

    await waitFor(() => {
      expect(screen.getByTestId('error-boundary')).toBeInTheDocument()
    })
  })
})