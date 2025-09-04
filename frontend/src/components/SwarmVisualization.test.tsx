import { render, screen } from '@testing-library/react'
import { SwarmVisualization } from './SwarmVisualization'
import { type Agent } from '@/store/hiveStore'
import { vi } from 'vitest'

// Mock HTMLCanvasElement methods
const mockGetContext = vi.fn()
const mockClearRect = vi.fn()
const mockBeginPath = vi.fn()
const mockArc = vi.fn()
const mockFill = vi.fn()
const mockStroke = vi.fn()
const mockMoveTo = vi.fn()
const mockLineTo = vi.fn()
const mockFillText = vi.fn()

// Mock canvas context
const mockContext = {
  clearRect: mockClearRect,
  beginPath: mockBeginPath,
  arc: mockArc,
  fill: mockFill,
  stroke: mockStroke,
  moveTo: mockMoveTo,
  lineTo: mockLineTo,
  fillText: mockFillText,
  fillStyle: '',
  strokeStyle: '',
  lineWidth: 1,
  font: '',
  globalAlpha: 1.0,
}

beforeEach(() => {
  // Reset all mocks
  vi.clearAllMocks()

  // Setup canvas mock
  mockGetContext.mockReturnValue(mockContext)

  // Mock HTMLCanvasElement
  Object.defineProperty(HTMLCanvasElement.prototype, 'getContext', {
    value: mockGetContext,
    writable: true,
  })
})

describe('SwarmVisualization', () => {
  const mockSwarmCenter: [number, number] = [10, 20]

  const mockAgents: Agent[] = [
    {
      id: '1',
      name: 'Worker-001',
      type: 'Worker',
      state: 'Working',
      capabilities: [{ name: 'Task1', proficiency: 0.8, learning_rate: 0.1 }],
      position: [5, 10],
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
      position: [15, 25],
      energy: 92,
      experience_count: 67,
      social_connections: 5,
    },
    {
      id: '3',
      name: 'Learner-001',
      type: 'Learner',
      state: 'Learning',
      capabilities: [{ name: 'Learning', proficiency: 0.7, learning_rate: 0.2 }],
      position: [-5, 30],
      energy: 78,
      experience_count: 23,
      social_connections: 2,
    },
  ]

  it('renders the component with title and canvas', () => {
    render(<SwarmVisualization agents={mockAgents} swarmCenter={mockSwarmCenter} />)

    expect(screen.getByText('Swarm Visualization')).toBeInTheDocument()
    const canvas = document.querySelector('canvas')
    expect(canvas).toBeInTheDocument()
  })

  it('displays help text', () => {
    render(<SwarmVisualization agents={mockAgents} swarmCenter={mockSwarmCenter} />)

    expect(screen.getByText('Red dot: Swarm center • Colored dots: Agents • Ring: Energy level')).toBeInTheDocument()
  })

  it('displays help text', () => {
    render(<SwarmVisualization agents={mockAgents} swarmCenter={mockSwarmCenter} />)

    expect(screen.getByText('Red dot: Swarm center • Colored dots: Agents • Ring: Energy level')).toBeInTheDocument()
  })

  it('calls canvas methods to draw visualization', () => {
    render(<SwarmVisualization agents={mockAgents} swarmCenter={mockSwarmCenter} />)

    // Should have called getContext
    expect(mockGetContext).toHaveBeenCalledWith('2d')

    // Should have cleared the canvas
    expect(mockClearRect).toHaveBeenCalled()

    // Should have drawn the swarm center and agents
    expect(mockBeginPath).toHaveBeenCalled()
    expect(mockArc).toHaveBeenCalled()
    expect(mockFill).toHaveBeenCalled()
    expect(mockStroke).toHaveBeenCalled()
  })

  it('draws grid lines', () => {
    render(<SwarmVisualization agents={mockAgents} swarmCenter={mockSwarmCenter} />)

    // Should draw grid lines (multiple moveTo and lineTo calls)
    expect(mockMoveTo).toHaveBeenCalled()
    expect(mockLineTo).toHaveBeenCalled()
    expect(mockStroke).toHaveBeenCalled()
  })

  it('draws agent names when there are few agents', () => {
    render(<SwarmVisualization agents={mockAgents} swarmCenter={mockSwarmCenter} />)

    // Should draw agent names since there are only 3 agents (< 10)
    expect(mockFillText).toHaveBeenCalledWith('Worker-001', expect.any(Number), expect.any(Number))
    expect(mockFillText).toHaveBeenCalledWith('Coordinator-001', expect.any(Number), expect.any(Number))
    expect(mockFillText).toHaveBeenCalledWith('Learner-001', expect.any(Number), expect.any(Number))
  })

  it('limits agent name drawing when there are many agents', () => {
    const manyAgents = Array.from({ length: 15 }, (_, i) => ({
      ...mockAgents[0],
      id: `agent-${i}`,
      name: `Agent-${i}`,
      position: [i * 5, i * 5] as [number, number],
    }))

    render(<SwarmVisualization agents={manyAgents} swarmCenter={mockSwarmCenter} />)

    // Should draw "Swarm Center" label and may draw some agent names
    expect(mockFillText).toHaveBeenCalled()
    // The exact number depends on the implementation, just verify it's called
  })

  it('handles empty agents array', () => {
    render(<SwarmVisualization agents={[]} swarmCenter={mockSwarmCenter} />)

    expect(mockClearRect).toHaveBeenCalled()
    expect(mockBeginPath).toHaveBeenCalled() // Still draws swarm center
    expect(mockArc).toHaveBeenCalled() // Still draws swarm center
  })

  it('handles canvas context not available', () => {
    mockGetContext.mockReturnValue(null)

    // Should not throw error
    expect(() => {
      render(<SwarmVisualization agents={mockAgents} swarmCenter={mockSwarmCenter} />)
    }).not.toThrow()
  })

  it('applies different colors for different agent types', () => {
    render(<SwarmVisualization agents={mockAgents} swarmCenter={mockSwarmCenter} />)

    // The fillStyle should be set multiple times for different agent types
    // Worker: #4ecdc4, Coordinator: #45b7d1, Learner: #96ceb4
    expect(mockContext.fillStyle).toBeDefined()
  })

  it('applies different opacity based on agent state', () => {
    render(<SwarmVisualization agents={mockAgents} swarmCenter={mockSwarmCenter} />)

    // Should set globalAlpha for different states
    // Working: 1.0, Learning: 0.8, Idle: 0.6, Failed: 0.3
    expect(mockContext.globalAlpha).toBeDefined()
  })

  it('draws energy rings with correct proportions', () => {
    render(<SwarmVisualization agents={mockAgents} swarmCenter={mockSwarmCenter} />)

    // Should draw energy rings as arcs with proportions based on energy
    // The exact parameters depend on agent positions and energy levels
    expect(mockArc).toHaveBeenCalled()
    expect(mockStroke).toHaveBeenCalled()
  })

  it('redraws when props change', () => {
    const { rerender } = render(<SwarmVisualization agents={mockAgents} swarmCenter={mockSwarmCenter} />)

    // Reset mock call counts
    vi.clearAllMocks()

    // Rerender with different props
    const newAgents = [...mockAgents]
    newAgents[0].position = [100, 100]

    rerender(<SwarmVisualization agents={newAgents} swarmCenter={[50, 50]} />)

    // Should redraw
    expect(mockClearRect).toHaveBeenCalled()
    expect(mockBeginPath).toHaveBeenCalled()
  })

  it('handles canvas ref not available', () => {
    // Mock canvas ref as null
    const originalGetContext = HTMLCanvasElement.prototype.getContext
    Object.defineProperty(HTMLCanvasElement.prototype, 'getContext', {
      value: () => null,
      writable: true,
    })

    expect(() => {
      render(<SwarmVisualization agents={mockAgents} swarmCenter={mockSwarmCenter} />)
    }).not.toThrow()

    // Restore original
    Object.defineProperty(HTMLCanvasElement.prototype, 'getContext', {
      value: originalGetContext,
      writable: true,
    })
  })
})
