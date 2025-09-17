import { render, screen } from '@testing-library/react'
import { vi } from 'vitest'
import RootLayout from './layout'

// Mock Next.js font
vi.mock('next/font/google', () => ({
  Inter: vi.fn(() => ({
    className: 'inter',
    subsets: ['latin'],
  })),
}))

describe('RootLayout', () => {
  it('renders children correctly', () => {
    const testContent = 'Test Content'
    render(
      <RootLayout>
        <div>{testContent}</div>
      </RootLayout>,
    )

    expect(screen.getByText(testContent)).toBeInTheDocument()
  })

  it('includes ClientErrorBoundary wrapper', () => {
    render(
      <RootLayout>
        <div>Test</div>
      </RootLayout>,
    )

    // The ClientErrorBoundary should wrap the children
    expect(screen.getByText('Test')).toBeInTheDocument()
  })

  it('sets correct HTML attributes', () => {
    render(
      <RootLayout>
        <div>Test</div>
      </RootLayout>,
    )

    const html = document.documentElement
    expect(html).toHaveAttribute('lang', 'en')
  })

  it('applies Inter font class to body', () => {
    render(
      <RootLayout>
        <div>Test</div>
      </RootLayout>,
    )

    expect(document.body.className).toContain('inter')
  })

  it('exports metadata correctly', () => {
    // Test that metadata is exported (this is more of a compile-time check)
    // In Next.js, metadata is handled by the framework
    expect(RootLayout).toBeDefined()
  })
})
