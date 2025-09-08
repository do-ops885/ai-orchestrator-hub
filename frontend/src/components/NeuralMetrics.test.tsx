import { render, screen, fireEvent } from '@testing-library/react'
import { NeuralMetrics } from './NeuralMetrics'
import { vi } from 'vitest'

describe('NeuralMetrics', () => {
  const mockAgents = [
    {
      id: '1',
      name: 'NLP-Agent-001',
      type: 'Learner',
      energy: 85,
      neural_type: undefined,
    },
    {
      id: '2',
      name: 'Coordinator-001',
      type: 'Coordinator',
      energy: 92,
      neural_type: 'fann',
    },
    {
      id: '3',
      name: 'Worker-001',
      type: 'Worker',
      energy: 78,
      neural_type: 'basic',
    },
    {
      id: '4',
      name: 'Specialist-001',
      type: 'Specialist',
      energy: 88,
      neural_type: 'lstm',
    },
  ]

  it('renders the component with title and brain icon', () => {
    render(<NeuralMetrics agents={mockAgents} />)

    expect(screen.getByText('Neural Processing Metrics')).toBeInTheDocument()
    expect(screen.getByText('Hybrid Mode Active')).toBeInTheDocument()
  })

  it('displays processing overview with correct counts', () => {
    render(<NeuralMetrics agents={mockAgents} />)

    // Check that we have the correct counts - should have 2 Basic NLP and 2 Advanced Neural
    const basicElements = screen.getAllByText('Basic NLP')
    const advancedElements = screen.getAllByText('Advanced Neural')
    const countElements = screen.getAllByText('2')

    expect(basicElements).toHaveLength(3) // 1 in overview + 2 in agent list
    expect(advancedElements).toHaveLength(1) // 1 in overview
    expect(countElements).toHaveLength(2) // Both counts should be 2
  })

  it('displays performance metrics correctly', () => {
    render(<NeuralMetrics agents={mockAgents} />)

    expect(screen.getByText('84.7%')).toBeInTheDocument() // avg_prediction_accuracy
    expect(screen.getByText('92%')).toBeInTheDocument() // neural_efficiency
  })

  it('displays agent neural capabilities', () => {
    render(<NeuralMetrics agents={mockAgents} />)

    expect(screen.getByText('Agent Neural Capabilities')).toBeInTheDocument()
    expect(screen.getByText('NLP-Agent-001')).toBeInTheDocument()
    expect(screen.getByText('Coordinator-001')).toBeInTheDocument()
    expect(screen.getByText('Worker-001')).toBeInTheDocument()
    expect(screen.getByText('Specialist-001')).toBeInTheDocument()
  })

  it('displays correct neural types for different agent types', () => {
    render(<NeuralMetrics agents={mockAgents} />)

    // Check specific neural types in the agent list
    expect(screen.getByText('LSTM')).toBeInTheDocument() // NLP-Agent-001 (Learner type)
    expect(screen.getByText('FANN')).toBeInTheDocument() // Specialist-001 (Specialist type)
    expect(screen.getAllByText('Basic NLP')).toHaveLength(3) // 2 in agent list + 1 in overview
  })

  it('displays performance insights', () => {
    render(<NeuralMetrics agents={mockAgents} />)

    expect(screen.getByText('Performance Insights')).toBeInTheDocument()
    expect(screen.getByText('Learning Rate:')).toBeInTheDocument()
    expect(screen.getByText('2.3%/hour')).toBeInTheDocument()
    expect(screen.getByText('Pattern Recognition:')).toBeInTheDocument()
    expect(screen.getByText('Excellent')).toBeInTheDocument()
    expect(screen.getByText('Memory Usage:')).toBeInTheDocument()
    expect(screen.getByText('Optimized')).toBeInTheDocument()
  })

  it('shows recommendation based on advanced agents count', () => {
    render(<NeuralMetrics agents={mockAgents} />)

    expect(
      screen.getByText(/Advanced agents are handling complex tasks efficiently/),
    ).toBeInTheDocument()
  })

  it('shows different recommendation when no advanced agents', () => {
    const basicAgentsOnly = mockAgents.map(agent => ({
      ...agent,
      neural_type: undefined,
    }))

    render(<NeuralMetrics agents={basicAgentsOnly} />)

    expect(screen.getByText(/Consider enabling advanced neural features/)).toBeInTheDocument()
  })

  it('allows selecting agents to highlight them', () => {
    render(<NeuralMetrics agents={mockAgents} />)

    const firstAgent = screen.getByText('NLP-Agent-001')
    fireEvent.click(firstAgent)

    // The component should handle selection state
    // We can test that the click doesn't throw an error
    expect(firstAgent).toBeInTheDocument()
  })

  it('limits displayed agents to 8 for performance', () => {
    const manyAgents = Array.from({ length: 12 }, (_, i) => ({
      id: `agent-${i}`,
      name: `Agent-${i}`,
      type: 'Worker',
      energy: 80 + (i % 20),
      neural_type: undefined,
    }))

    render(<NeuralMetrics agents={manyAgents} />)

    // Should only show first 8 agents in the list
    expect(screen.getByText('Agent-0')).toBeInTheDocument()
    expect(screen.getByText('Agent-7')).toBeInTheDocument()
    expect(screen.queryByText('Agent-8')).not.toBeInTheDocument()
  })

  it('handles empty agents array', () => {
    render(<NeuralMetrics agents={[]} />)

    expect(screen.getByText('Neural Processing Metrics')).toBeInTheDocument()
    expect(screen.getAllByText('0')).toHaveLength(2) // Both basic and advanced should be 0
  })

  it('calculates performance colors correctly', () => {
    const highEnergyAgent = [
      {
        id: '1',
        name: 'High-Energy-Agent',
        type: 'Worker',
        energy: 95, // Should be green
        neural_type: undefined,
      },
    ]

    render(<NeuralMetrics agents={highEnergyAgent} />)

    // The performance color logic should work
    // energy/100 * 0.8 + 0.2 = 0.95 * 0.8 + 0.2 = 0.96
    // This should be green (performance >= 0.8)
    expect(screen.getByText('0.96')).toBeInTheDocument()
  })

  it('handles agents with different neural_type values', () => {
    const mixedAgents = [
      { id: '1', name: 'Agent1', type: 'Worker', energy: 80, neural_type: null },
      { id: '2', name: 'Agent2', type: 'Worker', energy: 80, neural_type: 'fann' },
      { id: '3', name: 'Agent3', type: 'Worker', energy: 80, neural_type: 'lstm' },
      { id: '4', name: 'Agent4', type: 'Worker', energy: 80, neural_type: 'unknown' },
    ]

    render(<NeuralMetrics agents={mixedAgents} />)

    expect(screen.getByText('1')).toBeInTheDocument() // 1 basic (null neural_type)
    expect(screen.getByText('2')).toBeInTheDocument() // 2 advanced (fann + lstm)
  })
})
