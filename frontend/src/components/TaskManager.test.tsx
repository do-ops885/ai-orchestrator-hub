import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { TaskManager } from './TaskManager'
import { useHiveStore } from '@/store/hiveStore'
import { vi, type MockedFunction } from 'vitest'

// Mock the store
vi.mock('@/store/hiveStore')

const mockUseHiveStore = useHiveStore as unknown as MockedFunction<typeof useHiveStore>

describe('TaskManager', () => {
  const mockTasks = [
    {
      id: '1',
      description: 'Analyze customer sentiment data',
      type: 'nlp',
      priority: 7,
      status: 'Completed',
      assigned_agent: 'NLP-Agent-001',
      created_at: '2024-01-15T10:30:00Z',
      completed_at: '2024-01-15T10:45:00Z',
    },
    {
      id: '2',
      description: 'Coordinate swarm movement patterns',
      type: 'coordination',
      priority: 6,
      status: 'InProgress',
      assigned_agent: 'Coordinator-001',
      created_at: '2024-01-15T11:00:00Z',
    },
    {
      id: '3',
      description: 'Process incoming data stream',
      type: 'data_processing',
      priority: 5,
      status: 'Pending',
      created_at: '2024-01-15T11:15:00Z',
    },
  ]

  const mockCreateTask = vi.fn()

  beforeEach(() => {
    mockUseHiveStore.mockReturnValue({
      tasks: mockTasks,
      createTask: mockCreateTask,
    } as any)
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  it('renders the component with title and create button', () => {
    render(<TaskManager />)

    expect(screen.getByText('Task Management')).toBeInTheDocument()
    expect(screen.getByText('Create Task')).toBeInTheDocument()
  })

  it('displays tasks from the store', () => {
    render(<TaskManager />)

    expect(screen.getByText('Analyze customer sentiment data')).toBeInTheDocument()
    expect(screen.getByText('Coordinate swarm movement patterns')).toBeInTheDocument()
    expect(screen.getByText('Process incoming data stream')).toBeInTheDocument()
  })

  it('displays task status with correct icons', () => {
    render(<TaskManager />)

    // Check that status text is displayed
    expect(screen.getByText('Completed')).toBeInTheDocument()
    expect(screen.getByText('InProgress')).toBeInTheDocument()
    expect(screen.getByText('Pending')).toBeInTheDocument()
  })

  it('displays task priorities with correct colors', () => {
    render(<TaskManager />)

    // Priority 7 (High) - should have yellow background (2 tasks)
    const highElements = screen.getAllByText('High')
    expect(highElements).toHaveLength(2)
    // Priority 5 (Medium) - should have blue background
    expect(screen.getByText('Medium')).toBeInTheDocument()
  })

  it('displays task types and creation dates', () => {
    render(<TaskManager />)

    expect(screen.getByText((content) => content.includes('Type:') && content.includes('nlp'))).toBeInTheDocument()
    expect(screen.getByText((content) => content.includes('Type:') && content.includes('coordination'))).toBeInTheDocument()
    expect(screen.getByText((content) => content.includes('Type:') && content.includes('data_processing'))).toBeInTheDocument()

    // Should display formatted creation dates
    expect(screen.getAllByText(/Created:/)).toHaveLength(3)
  })

  it('displays assigned agents when available', () => {
    render(<TaskManager />)

    expect(screen.getByText('Assigned to: NLP-Agent-001')).toBeInTheDocument()
    expect(screen.getByText('Assigned to: Coordinator-001')).toBeInTheDocument()
  })

  it('displays completion dates for completed tasks', () => {
    render(<TaskManager />)

    expect(screen.getByText(/Completed:/)).toBeInTheDocument()
  })

  it('shows create form when create button is clicked', () => {
    render(<TaskManager />)

    const createButton = screen.getByText('Create Task')
    fireEvent.click(createButton)

    expect(screen.getByText('Create New Task')).toBeInTheDocument()
    expect(screen.getByPlaceholderText('Describe what this task should accomplish...')).toBeInTheDocument()
  })

  it('allows filling out the create task form', () => {
    render(<TaskManager />)

    const createButton = screen.getByText('Create Task')
    fireEvent.click(createButton)

    const descriptionInput = screen.getByPlaceholderText('Describe what this task should accomplish...')
    const typeSelect = screen.getByDisplayValue('General')
    const priorityInput = screen.getByDisplayValue('5')

    fireEvent.change(descriptionInput, { target: { value: 'Test task description' } })
    fireEvent.change(typeSelect, { target: { value: 'nlp' } })
    fireEvent.change(priorityInput, { target: { value: '8' } })

    expect(descriptionInput).toHaveValue('Test task description')
    expect(typeSelect).toHaveValue('nlp')
    expect(priorityInput).toHaveValue(8)
  })

  it('allows adding required capabilities', () => {
    render(<TaskManager />)

    const createButton = screen.getByText('Create Task')
    fireEvent.click(createButton)

    const addCapabilityButton = screen.getByText('Add Capability')
    fireEvent.click(addCapabilityButton)

    const capabilityInputs = screen.getAllByPlaceholderText('Capability name')
    expect(capabilityInputs).toHaveLength(2) // Original + new one
  })

  it('creates task when form is submitted', () => {
    render(<TaskManager />)

    const createButton = screen.getByText('Create Task')
    fireEvent.click(createButton)

    const descriptionInput = screen.getByPlaceholderText('Describe what this task should accomplish...')
    fireEvent.change(descriptionInput, { target: { value: 'New test task' } })

    const [, submitButton] = screen.getAllByText('Create Task') // The form submit button
    fireEvent.click(submitButton)

    expect(mockCreateTask).toHaveBeenCalledWith({
      description: 'New test task',
      type: 'general',
      priority: 5,
      required_capabilities: undefined,
    })
  })

  it('resets form after task creation', () => {
    render(<TaskManager />)

    const createButton = screen.getByText('Create Task')
    fireEvent.click(createButton)

    const descriptionInput = screen.getByPlaceholderText('Describe what this task should accomplish...')
    fireEvent.change(descriptionInput, { target: { value: 'Test task' } })

    const [, submitButton] = screen.getAllByText('Create Task')
    fireEvent.click(submitButton)

    // Form should be closed
    expect(screen.queryByText('Create New Task')).not.toBeInTheDocument()
  })

  it('shows empty state when no tasks exist', () => {
    mockUseHiveStore.mockReturnValue({
      tasks: [],
      createTask: mockCreateTask,
    } as any)

    render(<TaskManager />)

    expect(screen.getByText('No tasks')).toBeInTheDocument()
    expect(screen.getByText('Get started by creating your first task.')).toBeInTheDocument()
  })

  it('handles tasks without assigned agents', () => {
    const tasksWithoutAgents = [
      {
        id: '1',
        description: 'Task without agent',
        type: 'general',
        priority: 5,
        status: 'Pending',
        created_at: '2024-01-15T10:30:00Z',
      },
    ]

    mockUseHiveStore.mockReturnValue({
      tasks: tasksWithoutAgents,
      createTask: mockCreateTask,
    } as any)

    render(<TaskManager />)

    expect(screen.getByText('Task without agent')).toBeInTheDocument()
    // Should not show "Assigned to:" text
    expect(screen.queryByText(/Assigned to:/)).not.toBeInTheDocument()
  })

  it('handles tasks without completion dates', () => {
    const tasksWithoutCompletion = [
      {
        id: '1',
        description: 'Incomplete task',
        type: 'general',
        priority: 5,
        status: 'InProgress',
        created_at: '2024-01-15T10:30:00Z',
      },
    ]

    mockUseHiveStore.mockReturnValue({
      tasks: tasksWithoutCompletion,
      createTask: mockCreateTask,
    } as any)

    render(<TaskManager />)

    expect(screen.getByText('Incomplete task')).toBeInTheDocument()
    // Should not show "Completed:" text
    expect(screen.queryByText(/Completed:/)).not.toBeInTheDocument()
  })

  it('cancels form when cancel button is clicked', () => {
    render(<TaskManager />)

    const createButton = screen.getByText('Create Task')
    fireEvent.click(createButton)

    expect(screen.getByText('Create New Task')).toBeInTheDocument()

    const cancelButton = screen.getByText('Cancel')
    fireEvent.click(cancelButton)

    expect(screen.queryByText('Create New Task')).not.toBeInTheDocument()
  })
})
