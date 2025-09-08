import { render, screen } from '@testing-library/react'
import { HiveDashboard } from './HiveDashboard'
import { useHiveStore } from '@/store/hiveStore'
import { vi, type MockedFunction } from 'vitest'

// Mock the store
vi.mock('@/store/hiveStore')

// Mock the child components
vi.mock('./SwarmVisualization', () => ({
  SwarmVisualization: ({
    agents,
    swarmCenter,
  }: {
    agents: any[]
    swarmCenter: [number, number]
  }) => (
    <div data-testid="swarm-visualization">
      SwarmVisualization - {agents.length} agents, center: {swarmCenter.join(',')}
    </div>
  ),
}))

vi.mock('./MetricsPanel', () => ({
  MetricsPanel: ({ metrics }: { metrics: any }) => (
    <div data-testid="metrics-panel">
      MetricsPanel - Performance: {(metrics.average_performance * 100).toFixed(1)}%
    </div>
  ),
}))

vi.mock('./NeuralMetrics', () => ({
  NeuralMetrics: ({ agents }: { agents: any[] }) => (
    <div data-testid="neural-metrics">NeuralMetrics - {agents.length} agents</div>
  ),
}))

vi.mock('./ResourceMonitor', () => ({
  ResourceMonitor: () => <div data-testid="resource-monitor">ResourceMonitor</div>,
}))

const mockUseHiveStore = useHiveStore as unknown as MockedFunction<typeof useHiveStore>

describe('HiveDashboard', () => {
  const mockHiveStatus = {
    hive_id: 'test-hive-123',
    created_at: '2024-01-15T10:00:00Z',
    last_update: '2024-01-15T10:30:00Z',
    metrics: {
      total_agents: 5,
      active_agents: 4,
      completed_tasks: 12,
      failed_tasks: 2,
      average_performance: 0.85,
      swarm_cohesion: 0.92,
      learning_progress: 0.78,
    },
    swarm_center: [10, 20] as [number, number],
    total_energy: 450,
  }

  const mockAgents = [
    {
      id: '1',
      name: 'Worker-001',
      type: 'Worker',
      state: 'Working',
      capabilities: [{ name: 'Coding', proficiency: 0.8, learning_rate: 0.1 }],
      position: [5, 10] as [number, number],
      energy: 85,
      experience_count: 42,
      social_connections: 3,
    },
    {
      id: '2',
      name: 'Coordinator-001',
      type: 'Coordinator',
      state: 'Working',
      capabilities: [{ name: 'Coordination', proficiency: 0.9, learning_rate: 0.05 }],
      position: [15, 25] as [number, number],
      energy: 92,
      experience_count: 67,
      social_connections: 5,
    },
  ]

  beforeEach(() => {
    mockUseHiveStore.mockReturnValue({
      hiveStatus: mockHiveStatus,
      agents: mockAgents,
    } as any)
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  it('renders the component with hive overview', () => {
    render(<HiveDashboard />)

    expect(screen.getByText('Hive Overview')).toBeInTheDocument()
    expect(screen.getByText('5')).toBeInTheDocument() // Total Agents
    expect(screen.getByText('4')).toBeInTheDocument() // Active Agents
    expect(screen.getByText('12')).toBeInTheDocument() // Completed Tasks
  })

  it('displays correct metrics in overview cards', () => {
    render(<HiveDashboard />)

    expect(screen.getByText('Total Agents')).toBeInTheDocument()
    expect(screen.getByText('Active Agents')).toBeInTheDocument()
    expect(screen.getByText('Completed Tasks')).toBeInTheDocument()
  })

  it('renders all child components', () => {
    render(<HiveDashboard />)

    expect(screen.getByTestId('swarm-visualization')).toBeInTheDocument()
    expect(screen.getByTestId('metrics-panel')).toBeInTheDocument()
    expect(screen.getByTestId('neural-metrics')).toBeInTheDocument()
    expect(screen.getByTestId('resource-monitor')).toBeInTheDocument()
  })

  it('passes correct props to SwarmVisualization', () => {
    render(<HiveDashboard />)

    const swarmViz = screen.getByTestId('swarm-visualization')
    expect(swarmViz).toHaveTextContent('SwarmVisualization - 2 agents, center: 10,20')
  })

  it('passes correct props to MetricsPanel', () => {
    render(<HiveDashboard />)

    const metricsPanel = screen.getByTestId('metrics-panel')
    expect(metricsPanel).toHaveTextContent('MetricsPanel - Performance: 85.0%')
  })

  it('passes correct props to NeuralMetrics', () => {
    render(<HiveDashboard />)

    const neuralMetrics = screen.getByTestId('neural-metrics')
    expect(neuralMetrics).toHaveTextContent('NeuralMetrics - 2 agents')
  })

  it('shows loading state when hiveStatus is null', () => {
    mockUseHiveStore.mockReturnValue({
      hiveStatus: null,
      agents: [],
    } as any)

    render(<HiveDashboard />)

    expect(screen.getByText('Loading hive status...')).toBeInTheDocument()
  })

  it('shows loading state when hiveStatus is undefined', () => {
    mockUseHiveStore.mockReturnValue({
      hiveStatus: undefined,
      agents: [],
    } as any)

    render(<HiveDashboard />)

    expect(screen.getByText('Loading hive status...')).toBeInTheDocument()
  })

  it('handles empty agents array', () => {
    mockUseHiveStore.mockReturnValue({
      hiveStatus: mockHiveStatus,
      agents: [],
    } as any)

    render(<HiveDashboard />)

    const swarmViz = screen.getByTestId('swarm-visualization')
    expect(swarmViz).toHaveTextContent('SwarmVisualization - 0 agents, center: 10,20')

    const neuralMetrics = screen.getByTestId('neural-metrics')
    expect(neuralMetrics).toHaveTextContent('NeuralMetrics - 0 agents')
  })

  it('displays metrics with correct formatting', () => {
    render(<HiveDashboard />)

    // Check that the metrics are displayed correctly
    expect(screen.getByText('5')).toBeInTheDocument() // total_agents
    expect(screen.getByText('4')).toBeInTheDocument() // active_agents
    expect(screen.getByText('12')).toBeInTheDocument() // completed_tasks
  })
})
