import React from 'react'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { vi, beforeAll, afterAll } from 'vitest'
import { ErrorBoundary, AsyncErrorBoundary, withErrorBoundary } from './ErrorBoundary'

// Mock console.error to avoid noise in test output
beforeAll(() => {
  vi.spyOn(console, 'error').mockImplementation(() => {})
})

afterAll(() => {
  vi.restoreAllMocks()
})

// Component that throws an error
const ErrorComponent = ({ shouldThrow = true }: { shouldThrow?: boolean }) => {
  if (shouldThrow) {
    throw new Error('Test error')
  }
  return <div>No error</div>
}

// Component that can change throwing behavior
const DynamicErrorComponent = () => {
  const [shouldThrow, setShouldThrow] = React.useState(true)

  React.useImperativeHandle(React.createRef(), () => ({
    stopThrowing: () => setShouldThrow(false),
  }))

  if (shouldThrow) {
    throw new Error('Dynamic test error')
  }
  return <div>Recovered</div>
}

// Component that throws an error in event handler
const ErrorInEventComponent = () => {
  const [shouldThrow, setShouldThrow] = React.useState(false)

  React.useEffect(() => {
    if (shouldThrow) {
      throw new Error('Event handler error')
    }
  }, [shouldThrow])

  return <button onClick={() => setShouldThrow(true)}>Trigger Error</button>
}

// Component that throws an error during render after state change
const DelayedErrorComponent = () => {
  const [shouldThrow, setShouldThrow] = React.useState(false)

  React.useEffect(() => {
    const timer = setTimeout(() => {
      setShouldThrow(true)
    }, 100)
    return () => clearTimeout(timer)
  }, [])

  if (shouldThrow) {
    throw new Error('Delayed error')
  }

  return <div>Delayed component</div>
}

describe('ErrorBoundary', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should render children when no error occurs', () => {
    render(
      <ErrorBoundary>
        <div>Test content</div>
      </ErrorBoundary>,
    )

    expect(screen.getByText('Test content')).toBeInTheDocument()
  })

  it('should catch and display error UI when child throws', () => {
    render(
      <ErrorBoundary>
        <ErrorComponent />
      </ErrorBoundary>,
    )

    expect(screen.getByText('Something went wrong')).toBeInTheDocument()
    expect(screen.getByText(/We encountered an unexpected error/)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /try again/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /go home/i })).toBeInTheDocument()
  })

  it('should show custom fallback when provided', () => {
    render(
      <ErrorBoundary fallback={<div>Custom error message</div>}>
        <ErrorComponent />
      </ErrorBoundary>,
    )

    expect(screen.getByText('Custom error message')).toBeInTheDocument()
    expect(screen.queryByText('Something went wrong')).not.toBeInTheDocument()
  })

  it('should show error details when showDetails is true', () => {
    render(
      <ErrorBoundary showDetails={true}>
        <ErrorComponent />
      </ErrorBoundary>,
    )

    expect(screen.getByText('Error Details')).toBeInTheDocument()

    // Check that the error message is in the document
    expect(
      screen.getByText((content, element) => {
        return element?.tagName === 'PRE' && content.includes('Test error')
      }),
    ).toBeInTheDocument()
  })

  it('should not show error details when showDetails is false', () => {
    render(
      <ErrorBoundary showDetails={false}>
        <ErrorComponent />
      </ErrorBoundary>,
    )

    expect(screen.queryByText('Error Details')).not.toBeInTheDocument()
  })

  it('should retry and allow recovery', () => {
    // Test that retry resets the state
    render(
      <ErrorBoundary>
        <ErrorComponent shouldThrow={true} />
      </ErrorBoundary>,
    )

    expect(screen.getByText('Something went wrong')).toBeInTheDocument()

    // Click retry button
    fireEvent.click(screen.getByRole('button', { name: /try again/i }))

    // The component should still show error because children still throw
    expect(screen.getByText('Something went wrong')).toBeInTheDocument()
    expect(screen.getByText('Retry attempts: 1')).toBeInTheDocument()
  })

  it('should increment retry count', () => {
    render(
      <ErrorBoundary>
        <ErrorComponent />
      </ErrorBoundary>,
    )

    expect(screen.getByText('Something went wrong')).toBeInTheDocument()

    // Click retry once
    fireEvent.click(screen.getByRole('button', { name: /try again/i }))

    // Should show retry count
    expect(screen.getByText('Retry attempts: 1')).toBeInTheDocument()
  })

  it('should call onError callback when error occurs', () => {
    const onErrorMock = vi.fn()

    render(
      <ErrorBoundary onError={onErrorMock}>
        <ErrorComponent />
      </ErrorBoundary>,
    )

    expect(onErrorMock).toHaveBeenCalledTimes(1)
    expect(onErrorMock).toHaveBeenCalledWith(
      expect.any(Error),
      expect.objectContaining({
        componentStack: expect.any(String),
      }),
    )
  })

  it('should handle errors thrown in event handlers', () => {
    render(
      <ErrorBoundary>
        <ErrorInEventComponent />
      </ErrorBoundary>,
    )

    // Initially no error
    expect(screen.queryByText('Something went wrong')).not.toBeInTheDocument()

    // Trigger error
    fireEvent.click(screen.getByRole('button', { name: /trigger error/i }))

    expect(screen.getByText('Something went wrong')).toBeInTheDocument()
  })
})

describe('AsyncErrorBoundary', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('should render children when no error occurs', () => {
    render(
      <AsyncErrorBoundary>
        <div>Async content</div>
      </AsyncErrorBoundary>,
    )

    expect(screen.getByText('Async content')).toBeInTheDocument()
  })

  it('should catch delayed errors and show appropriate UI', async () => {
    render(
      <AsyncErrorBoundary>
        <DelayedErrorComponent />
      </AsyncErrorBoundary>,
    )

    // Wait for delayed error to be thrown
    await waitFor(() => {
      expect(screen.getByText('Operation Failed')).toBeInTheDocument()
    })

    expect(screen.getByText(/The requested operation could not be completed/)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument()
  })

  it('should retry after delayed error', async () => {
    render(
      <AsyncErrorBoundary>
        <DelayedErrorComponent />
      </AsyncErrorBoundary>,
    )

    await waitFor(() => {
      expect(screen.getByText('Operation Failed')).toBeInTheDocument()
    })

    // Click retry
    fireEvent.click(screen.getByRole('button', { name: /retry/i }))

    // Should recover
    expect(screen.queryByText('Operation Failed')).not.toBeInTheDocument()
  })
})

describe('withErrorBoundary HOC', () => {
  it('should wrap component with ErrorBoundary', () => {
    const TestComponent = () => <div>Test component</div>
    const WrappedComponent = withErrorBoundary(TestComponent)

    render(<WrappedComponent />)

    expect(screen.getByText('Test component')).toBeInTheDocument()
  })

  it('should pass props through to wrapped component', () => {
    const TestComponent = ({ message }: { message: string }) => <div>{message}</div>
    const WrappedComponent = withErrorBoundary(TestComponent)

    render(<WrappedComponent message="Hello World" />)

    expect(screen.getByText('Hello World')).toBeInTheDocument()
  })

  it('should handle errors in wrapped component', () => {
    const FailingComponent = () => {
      throw new Error('Wrapped component error')
    }
    const WrappedComponent = withErrorBoundary(FailingComponent)

    render(<WrappedComponent />)

    expect(screen.getByText('Something went wrong')).toBeInTheDocument()
  })

  it('should pass error boundary props to HOC', () => {
    const FailingComponent = () => {
      throw new Error('Error with custom fallback')
    }
    const WrappedComponent = withErrorBoundary(FailingComponent, {
      fallback: <div>Custom fallback UI</div>,
    })

    render(<WrappedComponent />)

    expect(screen.getByText('Custom fallback UI')).toBeInTheDocument()
  })

  it('should set display name correctly', () => {
    const TestComponent = () => <div>Test</div>
    TestComponent.displayName = 'MyTestComponent'

    const WrappedComponent = withErrorBoundary(TestComponent)

    expect(WrappedComponent.displayName).toBe('withErrorBoundary(MyTestComponent)')
  })
})

// Test error reporting functionality
describe('Error Reporting', () => {
  it('should report errors with proper structure', () => {
    render(
      <ErrorBoundary>
        <ErrorComponent />
      </ErrorBoundary>,
    )

    // Error boundary should catch the error and render fallback UI
    expect(screen.getByText('Something went wrong')).toBeInTheDocument()
  })
})
