import { vi } from 'vitest'
import RootLayout, { metadata } from './layout'

// Mock Next.js font
vi.mock('next/font/google', () => ({
  Inter: vi.fn(() => ({
    className: 'inter',
    subsets: ['latin'],
  })),
}))

describe('RootLayout', () => {
  it('exports metadata correctly', () => {
    // Test that metadata is exported (this is more of a compile-time check)
    // In Next.js, metadata is handled by the framework
    expect(RootLayout).toBeDefined()
    expect(metadata).toBeDefined()
  })

  it('has correct metadata structure', () => {
    expect(metadata).toEqual({
      title: 'AI Orchestrator Hub',
      description: expect.stringContaining('multiagent system'),
    })
  })

  it('can be imported without errors', () => {
    expect(() => {
      expect(typeof RootLayout).toBe('function')
      expect(RootLayout.name).toBe('RootLayout')
    }).not.toThrow()
  })

  it('metadata has required properties', () => {
    expect(metadata.title).toBe('AI Orchestrator Hub')
    expect(metadata.description).toContain('multiagent system')
    expect(metadata.description).toContain('hive/swarm intelligence')
    expect(metadata.description).toContain('NLP self-learning')
  })
})
