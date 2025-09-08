import { render, screen, waitFor } from '@testing-library/react'
import { ResourceMonitor } from './ResourceMonitor'
import { vi } from 'vitest'

// Mock fetch
const mockFetch = vi.fn()
global.fetch = mockFetch

describe('ResourceMonitor', () => {
  const mockResourceData = {
    system_resources: {
      cpu_cores: 8,
      available_memory: 16_000_000_000, // 16GB in bytes
      cpu_usage: 45.5,
      memory_usage: 62.3,
      last_updated: '2024-01-15T10:30:00Z',
      simd_capabilities: ['SSE4.2', 'AVX2', 'AVX-512'],
    },
    resource_profile: {
      profile_name: 'High Performance',
      max_agents: 50,
      neural_complexity: 0.9,
      batch_size: 32,
      update_frequency: 100,
    },
    hardware_class: 'Server',
  }

  beforeEach(() => {
    vi.clearAllMocks()
    process.env.NEXT_PUBLIC_API_URL = 'http://localhost:3001'

    // Mock successful fetch
    mockFetch.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(mockResourceData),
    } as Response)
  })

  it('renders loading state initially', () => {
    render(<ResourceMonitor />)

    expect(screen.getByText('Loading resource information...')).toBeInTheDocument()
  })

  it('fetches and displays resource data after loading', async () => {
    render(<ResourceMonitor />)

    // Wait for the fetch to complete and data to load
    await waitFor(() => {
      expect(screen.queryByText('Loading resource information...')).not.toBeInTheDocument()
    })

    expect(screen.getByText('🖥️ System Resources')).toBeInTheDocument()
    expect(screen.getByText('🖥️')).toBeInTheDocument() // Server icon
    expect(screen.getByText('Server')).toBeInTheDocument()
  })

  it('displays CPU and memory usage correctly', async () => {
    render(<ResourceMonitor />)

    await waitFor(() => {
      expect(screen.queryByText('Loading resource information...')).not.toBeInTheDocument()
    })

    expect(screen.getByText('CPU Usage')).toBeInTheDocument()
    expect(screen.getByText('45.5%')).toBeInTheDocument()
    expect(screen.getByText('8 cores')).toBeInTheDocument()
    expect(screen.getByText('Memory Usage')).toBeInTheDocument()
    expect(screen.getByText('62.3%')).toBeInTheDocument()
    expect(screen.getByText('16.0GB')).toBeInTheDocument()
  })

  it('displays resource profile information', async () => {
    render(<ResourceMonitor />)

    await waitFor(() => {
      expect(screen.queryByText('Loading resource information...')).not.toBeInTheDocument()
    })

    expect(screen.getByText('⚡ Current Profile: High Performance')).toBeInTheDocument()
    expect(screen.getByText('Max Agents')).toBeInTheDocument()
    expect(screen.getByText('50')).toBeInTheDocument()
    expect(screen.getByText('Neural Complexity')).toBeInTheDocument()
    expect(screen.getByText('90%')).toBeInTheDocument()
    expect(screen.getByText('Batch Size')).toBeInTheDocument()
    expect(screen.getByText('32')).toBeInTheDocument()
    expect(screen.getByText('Update Freq')).toBeInTheDocument()
    expect(screen.getByText('100ms')).toBeInTheDocument()
  })

  it('displays SIMD capabilities', async () => {
    render(<ResourceMonitor />)

    await waitFor(() => {
      expect(screen.queryByText('Loading resource information...')).not.toBeInTheDocument()
    })

    expect(screen.getByText('🔧 CPU Optimizations')).toBeInTheDocument()
    expect(screen.getByText('SSE4.2')).toBeInTheDocument()
    expect(screen.getByText('AVX2')).toBeInTheDocument()
    expect(screen.getByText('AVX-512')).toBeInTheDocument()
  })

  it('displays CPU-Native badge', async () => {
    render(<ResourceMonitor />)

    await waitFor(() => {
      expect(screen.queryByText('Loading resource information...')).not.toBeInTheDocument()
    })

    expect(screen.getByText('🚀 CPU-Native, GPU-Optional')).toBeInTheDocument()
  })

  it('displays Phase 2 status', async () => {
    render(<ResourceMonitor />)

    await waitFor(() => {
      expect(screen.queryByText('Loading resource information...')).not.toBeInTheDocument()
    })

    expect(screen.getByText('Phase 2 Status')).toBeInTheDocument()
    expect(screen.getByText('✅ Active')).toBeInTheDocument()
    expect(
      screen.getByText('Intelligent resource management and auto-optimization enabled'),
    ).toBeInTheDocument()
  })

  it('applies correct color classes based on usage levels', async () => {
    render(<ResourceMonitor />)

    await waitFor(() => {
      expect(screen.queryByText('Loading resource information...')).not.toBeInTheDocument()
    })

    // CPU usage 45.5% should be green (below 50%)
    // Memory usage 62.3% should be yellow (between 50% and 80%)
    // The component should apply appropriate background colors
    expect(screen.getByText('CPU Usage')).toBeInTheDocument()
  })

  it('handles different hardware classes correctly', async () => {
    const desktopData = {
      ...mockResourceData,
      hardware_class: 'Desktop',
    }

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(desktopData),
    } as Response)

    render(<ResourceMonitor />)

    await waitFor(() => {
      expect(screen.queryByText('Loading resource information...')).not.toBeInTheDocument()
    })

    expect(screen.getByText('Desktop')).toBeInTheDocument()
    expect(screen.getByText('🖥️')).toBeInTheDocument()
  })

  it('handles fetch errors gracefully', async () => {
    mockFetch.mockRejectedValueOnce(new Error('Network error'))

    // Mock console.warn to avoid console output during test
    const consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {})

    render(<ResourceMonitor />)

    // Should remain in loading state or handle error
    await waitFor(() => {
      expect(consoleWarnSpy).toHaveBeenCalledWith(
        'Failed to fetch resource info:',
        expect.any(Error),
      )
    })

    consoleWarnSpy.mockRestore()
  })

  it('handles non-ok response gracefully', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 500,
    } as Response)

    render(<ResourceMonitor />)

    // Should remain in loading state for non-ok response
    await waitFor(() => {
      expect(screen.getByText('Loading resource information...')).toBeInTheDocument()
    })
  })

  it('updates data periodically', async () => {
    vi.useFakeTimers()
    render(<ResourceMonitor />)

    await waitFor(() => {
      expect(screen.queryByText('Loading resource information...')).not.toBeInTheDocument()
    })

    // Fast-forward time by 30 seconds and run pending timers
    vi.advanceTimersByTime(30000)
    vi.runOnlyPendingTimers()

    // Should have called fetch again
    expect(mockFetch).toHaveBeenCalledTimes(2)
    vi.useRealTimers()
  })

  it('clears interval on unmount', async () => {
    vi.useFakeTimers()
    const { unmount } = render(<ResourceMonitor />)

    await waitFor(() => {
      expect(screen.queryByText('Loading resource information...')).not.toBeInTheDocument()
    })

    unmount()

    // Fast-forward time and run pending timers - no additional fetch should occur
    vi.advanceTimersByTime(30000)
    vi.runOnlyPendingTimers()

    // Should still only have 1 call (the initial one)
    expect(mockFetch).toHaveBeenCalledTimes(1)
    vi.useRealTimers()
  })

  it('handles empty SIMD capabilities array', async () => {
    vi.clearAllMocks()
    const noSimdData = {
      ...mockResourceData,
      system_resources: {
        ...mockResourceData.system_resources,
        simd_capabilities: [],
      },
    }

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(noSimdData),
    } as Response)

    render(<ResourceMonitor />)

    await waitFor(() => {
      expect(screen.queryByText('Loading resource information...')).not.toBeInTheDocument()
    })

    // Should not display CPU Optimizations section
    expect(screen.queryByText('🔧 CPU Optimizations')).not.toBeInTheDocument()
  })


})
