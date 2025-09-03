import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { AgentManager } from './AgentManager'
import { useHiveStore } from '@/store/hiveStore'
import { vi, type MockedFunction } from 'vitest'

// Mock the store
vi.mock('@/store/hiveStore')

const mockUseHiveStore = useHiveStore as unknown as MockedFunction<typeof useHiveStore>

describe('AgentManager', () => {
  const mockAgents = [
    {
      id: '1',
      name: 'Test Agent',
      type: 'Worker',
      state: 'Working',
      capabilities: [
        { name: 'Coding', proficiency: 0.8, learning_rate: 0.1 },
        { name: 'Testing', proficiency: 0.6, learning_rate: 0.05 },
      ],
      position: [0, 0] as [number, number],
      energy: 85,
      experience_count: 42,
      social_connections: 5,
    },
  ]

  const mockCreateAgent = vi.fn()

  beforeEach(() => {
    mockUseHiveStore.mockReturnValue({
      agents: mockAgents,
      createAgent: mockCreateAgent,
    } as any)
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  it('renders the component with title and create button', () => {
    render(<AgentManager />)

    expect(screen.getByText('Agent Management')).toBeInTheDocument()
    expect(screen.getByText('Create Agent')).toBeInTheDocument()
  })

  it('displays existing agents', () => {
    render(<AgentManager />)

    expect(screen.getByText('Test Agent')).toBeInTheDocument()
    expect(screen.getByText('Worker')).toBeInTheDocument()
    expect(screen.getByText('Working')).toBeInTheDocument()
    expect(screen.getByText('85.0%')).toBeInTheDocument()
    expect(screen.getByText('42 experiences')).toBeInTheDocument()
    expect(screen.getByText('2 capabilities')).toBeInTheDocument()
    expect(screen.getByText('5 connections')).toBeInTheDocument()
  })

  it('displays agent capabilities', () => {
    render(<AgentManager />)

    expect(screen.getByText('Coding (80%)')).toBeInTheDocument()
    expect(screen.getByText('Testing (60%)')).toBeInTheDocument()
  })

  it('shows create form when create button is clicked', () => {
    render(<AgentManager />)

    const createButton = screen.getByText('Create Agent')
    fireEvent.click(createButton)

    expect(screen.getByText('Create New Agent')).toBeInTheDocument()
    expect(screen.getByPlaceholderText('Agent name')).toBeInTheDocument()
    expect(screen.getByDisplayValue('Worker')).toBeInTheDocument()
  })

  it('allows adding capabilities to new agent', () => {
    render(<AgentManager />)

    const createButton = screen.getByText('Create Agent')
    fireEvent.click(createButton)

    const addCapabilityButton = screen.getByText('Add Capability')
    fireEvent.click(addCapabilityButton)

    const capabilityInputs = screen.getAllByPlaceholderText('Capability name')
    expect(capabilityInputs).toHaveLength(2)
  })

  it('creates agent when form is submitted', () => {
    render(<AgentManager />)

    const createButton = screen.getByText('Create Agent')
    fireEvent.click(createButton)

    const nameInput = screen.getByPlaceholderText('Agent name')
    fireEvent.change(nameInput, { target: { value: 'New Agent' } })

    const [, submitButton] = screen.getAllByText('Create Agent') // The submit button in the form
    fireEvent.click(submitButton)

    expect(mockCreateAgent).toHaveBeenCalledWith({
      name: 'New Agent',
      type: 'Worker',
      capabilities: [{ name: '', proficiency: 0.5, learning_rate: 0.1 }],
    })
  })

  it('shows empty state when no agents exist', () => {
    mockUseHiveStore.mockReturnValue({
      agents: [],
      createAgent: mockCreateAgent,
    } as any)

    render(<AgentManager />)

    expect(screen.getByText('No agents')).toBeInTheDocument()
    expect(screen.getByText('Get started by creating your first agent.')).toBeInTheDocument()
  })

  it('displays correct icons for different agent types', () => {
    const agentsWithDifferentTypes = [
      { ...mockAgents[0], type: 'Coordinator', id: '2' },
      { ...mockAgents[0], type: 'Learner', id: '3' },
    ]

    mockUseHiveStore.mockReturnValue({
      agents: agentsWithDifferentTypes,
      createAgent: mockCreateAgent,
    } as any)

    render(<AgentManager />)

    // The icons are rendered via Lucide React components
    // We can test that the correct types are displayed
    expect(screen.getAllByText('Coordinator')).toHaveLength(1)
    expect(screen.getAllByText('Learner')).toHaveLength(1)
  })
})
