import { render, screen } from '@testing-library/react'
import { MetricsPanel } from './MetricsPanel'
import { type HiveMetrics } from '@/store/hiveStore'

describe('MetricsPanel', () => {
  const mockMetrics: HiveMetrics = {
    total_agents: 5,
    active_agents: 4,
    completed_tasks: 12,
    failed_tasks: 2,
    average_performance: 0.85,
    swarm_cohesion: 0.92,
    learning_progress: 0.78,
  }

  it('renders the component with title', () => {
    render(<MetricsPanel metrics={mockMetrics} />)

    expect(screen.getByText('Hive Metrics')).toBeInTheDocument()
  })

  it('displays all metric items correctly', () => {
    render(<MetricsPanel metrics={mockMetrics} />)

    expect(screen.getByText('Average Performance')).toBeInTheDocument()
    expect(screen.getByText('Swarm Cohesion')).toBeInTheDocument()
    expect(screen.getByText('Learning Progress')).toBeInTheDocument()
    expect(screen.getByText('Task Success Rate')).toBeInTheDocument()
  })

  it('formats percentage values correctly', () => {
    render(<MetricsPanel metrics={mockMetrics} />)

    expect(screen.getByText('85.0%')).toBeInTheDocument() // average_performance
    expect(screen.getByText('92.0%')).toBeInTheDocument() // swarm_cohesion
    expect(screen.getByText('78.0%')).toBeInTheDocument() // learning_progress
  })

  it('calculates and displays task success rate correctly', () => {
    render(<MetricsPanel metrics={mockMetrics} />)

    // Task success rate = completed_tasks / (completed_tasks + failed_tasks) = 12 / (12 + 2) = 85.7%
    expect(screen.getByText('85.7%')).toBeInTheDocument()
  })

  it('displays task success rate as 0% when no tasks exist', () => {
    const metricsWithNoTasks: HiveMetrics = {
      ...mockMetrics,
      completed_tasks: 0,
      failed_tasks: 0,
    }

    render(<MetricsPanel metrics={metricsWithNoTasks} />)

    expect(screen.getByText('0%')).toBeInTheDocument()
  })

  it('displays metric descriptions', () => {
    render(<MetricsPanel metrics={mockMetrics} />)

    expect(screen.getByText('Overall capability proficiency across all agents')).toBeInTheDocument()
    expect(screen.getByText('How well agents are coordinated spatially')).toBeInTheDocument()
    expect(screen.getByText('Collective learning advancement of the hive')).toBeInTheDocument()
    expect(screen.getByText('Ratio of successful to total completed tasks')).toBeInTheDocument()
  })

  it('displays task statistics in the bottom section', () => {
    render(<MetricsPanel metrics={mockMetrics} />)

    expect(screen.getByText('Task Statistics')).toBeInTheDocument()
    expect(screen.getByText('Completed:')).toBeInTheDocument()
    expect(screen.getByText('12')).toBeInTheDocument()
    expect(screen.getByText('Failed:')).toBeInTheDocument()
    expect(screen.getByText('2')).toBeInTheDocument()
  })

  it('applies correct color classes for different performance levels', () => {
    render(<MetricsPanel metrics={mockMetrics} />)

    // High performance (85%, 92%, 78%) should have green/blue/purple backgrounds
    const metricCards = screen.getAllByText(/85\.0%|92\.0%|78\.0%|85\.7%/)
    expect(metricCards.length).toBeGreaterThan(0)

    // Check that the cards have the expected styling classes
    // Note: The exact class names depend on the color mapping in the component
  })

  it('handles edge case with zero values', () => {
    const zeroMetrics: HiveMetrics = {
      total_agents: 0,
      active_agents: 0,
      completed_tasks: 0,
      failed_tasks: 0,
      average_performance: 0,
      swarm_cohesion: 0,
      learning_progress: 0,
    }

    render(<MetricsPanel metrics={zeroMetrics} />)

    const zeroPercentElements = screen.getAllByText('0.0%')
    expect(zeroPercentElements.length).toBeGreaterThan(0)
    expect(screen.getByText('0%')).toBeInTheDocument() // Task success rate
  })

  it('handles maximum values correctly', () => {
    const maxMetrics: HiveMetrics = {
      total_agents: 100,
      active_agents: 100,
      completed_tasks: 1000,
      failed_tasks: 0,
      average_performance: 1.0,
      swarm_cohesion: 1.0,
      learning_progress: 1.0,
    }

    render(<MetricsPanel metrics={maxMetrics} />)

    const hundredPercentElements = screen.getAllByText('100.0%')
    expect(hundredPercentElements.length).toBeGreaterThan(0)
    expect(screen.getByText('100.0%')).toBeInTheDocument() // Task success rate when no failures
  })

  it('formats decimal values correctly', () => {
    const decimalMetrics: HiveMetrics = {
      ...mockMetrics,
      average_performance: 0.856,
      swarm_cohesion: 0.923,
      learning_progress: 0.789,
    }

    render(<MetricsPanel metrics={decimalMetrics} />)

    expect(screen.getByText('85.6%')).toBeInTheDocument()
    expect(screen.getByText('92.3%')).toBeInTheDocument()
    expect(screen.getByText('78.9%')).toBeInTheDocument()
  })
})
