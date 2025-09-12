import { create } from 'zustand'

export interface Agent {
  id: string
  name: string
  type: string
  state: string
  capabilities: Array<{
    name: string
    proficiency: number
    learning_rate: number
  }>
  position: [number, number]
  energy: number
  experience_count: number
  social_connections: number
}

export interface HiveMetrics {
  total_agents: number
  active_agents: number
  completed_tasks: number
  failed_tasks: number
  average_performance: number
  swarm_cohesion: number
  learning_progress: number
}

export interface Task {
  id: string
  description: string
  type: string
  priority: number
  status: string
  assigned_agent?: string
  created_at: string
  completed_at?: string
  required_capabilities?: Array<{
    name: string
    min_proficiency: number
    weight: number
  }>
}

export interface HiveStatus {
  hive_id: string
  created_at: string
  last_update: string
  metrics: HiveMetrics
  swarm_center: [number, number]
  total_energy: number
}

interface HiveStore {
  // Connection state
  isConnected: boolean
  socket: WebSocket | null
  connectionAttempts: number
  maxReconnectAttempts: number
  reconnectDelay: number
  lastHeartbeat: number
  heartbeatInterval: NodeJS.Timeout | null
  connectionQuality: 'excellent' | 'good' | 'poor' | 'disconnected'

  // Data
  agents: Agent[]
  hiveStatus: HiveStatus | null
  tasks: Task[]

  // Actions
  connectWebSocket: (url: string) => void
  disconnect: () => void
  createAgent: (config: unknown) => void
  createTask: (config: unknown) => void
  updateAgents: (agents: Agent[]) => void
  updateHiveStatus: (status: HiveStatus) => void
  updateTasks: (tasks: Task[]) => void
  startHeartbeat: () => void
  stopHeartbeat: () => void
  updateConnectionQuality: () => void
}

export const useHiveStore = create<HiveStore>((set, get) => ({
  isConnected: false,
  socket: null,
  connectionAttempts: 0,
  maxReconnectAttempts: 10,
  reconnectDelay: 1000,
  lastHeartbeat: Date.now(),
  heartbeatInterval: null,
  connectionQuality: 'disconnected' as const,
  agents: [],
  hiveStatus: null,
  tasks: [],

  connectWebSocket: (url: string) => {
    // Prevent multiple connection attempts
    const currentSocket = get().socket
    if (currentSocket !== null && currentSocket.readyState === WebSocket.OPEN) {
      console.warn('WebSocket already connected')
      return
    }

    console.warn('🔌 Attempting WebSocket connection to:', url)
    const socket = new WebSocket(url)

    socket.onopen = () => {
      console.warn('✅ WebSocket connected successfully')
      set({
        isConnected: true,
        socket,
        connectionAttempts: 0,
        lastHeartbeat: Date.now(),
        connectionQuality: 'excellent',
      })
      get().startHeartbeat()
    }

    socket.onmessage = event => {
      try {
        const message = JSON.parse(event.data)

        if (typeof message.message_type === 'string') {
          switch (message.message_type) {
            case 'hive_status':
              set({ hiveStatus: message.data })
              break
            case 'agents_update':
              set({ agents: message.data?.agents ?? [] })
              break
            case 'metrics_update': {
              const currentStatus = get().hiveStatus
              if (
                currentStatus !== null &&
                currentStatus !== undefined &&
                message.data?.metrics !== null &&
                message.data?.metrics !== undefined
              ) {
                set({
                  hiveStatus: {
                    ...currentStatus,
                    metrics: message.data.metrics,
                    swarm_center: message.data.swarm_center ?? currentStatus.swarm_center,
                    total_energy: message.data.total_energy ?? currentStatus.total_energy,
                  },
                })
              }
              break
            }
            case 'agent_created':
            case 'task_created':
              console.warn('Created:', message.data)
              break
            case 'tasks_update':
              set({ tasks: message.data?.tasks ?? [] })
              break
            case 'task_status_update': {
              const currentTasks = get().tasks
              const updatedTask = message.data?.task
              if (updatedTask !== null && updatedTask !== undefined) {
                const updatedTasks = currentTasks.map(task =>
                  task.id === updatedTask.id ? { ...task, ...updatedTask } : task,
                )
                set({ tasks: updatedTasks })
              }
              break
            }
            case 'error':
              console.warn('Hive error:', message.data?.error)
              break
          }
        }
      } catch (error) {
        console.warn('Failed to parse WebSocket message:', error)
      }
    }

    socket.onclose = event => {
      const attempts = get().connectionAttempts
      const maxAttempts = get().maxReconnectAttempts
      console.warn(
        `🔌 WebSocket disconnected (code: ${event.code}, reason: ${event.reason !== '' ? event.reason : 'Unknown'})`,
      )

      get().stopHeartbeat()
      set({ isConnected: false, socket: null, connectionQuality: 'disconnected' })

      // Auto-retry with exponential backoff
      if (attempts < maxAttempts && event.code !== 1000) {
        // Don't retry on normal closure
        const baseDelay = get().reconnectDelay
        const retryDelay = Math.min(baseDelay * Math.pow(2, attempts), 30000) // Max 30 seconds
        console.warn(
          `🔄 Retrying WebSocket connection in ${retryDelay}ms... (attempt ${attempts + 1}/${maxAttempts})`,
        )
        setTimeout(() => {
          set({ connectionAttempts: attempts + 1 })
          get().connectWebSocket(url)
        }, retryDelay)
      } else if (attempts >= maxAttempts) {
        console.warn('❌ Max WebSocket connection attempts reached. Please refresh the page.')
        set({ connectionQuality: 'disconnected' })
      }
    }

    socket.onerror = error => {
      console.warn(
        'WebSocket connection error - this is normal during development. Retrying...',
        error,
      )
      set({ isConnected: false })
      // Auto-retry connection after 3 seconds
      setTimeout(() => {
        if (get().socket?.readyState !== WebSocket.OPEN) {
          console.warn('Attempting to reconnect WebSocket...')
          get().connectWebSocket(url)
        }
      }, 3000)
    }
  },

  disconnect: () => {
    const { socket } = get()
    get().stopHeartbeat()
    if (socket !== null && socket !== undefined) {
      socket.close(1000, 'User initiated disconnect') // Normal closure
    }
    set({
      isConnected: false,
      socket: null,
      connectionQuality: 'disconnected',
      connectionAttempts: 0,
    })
  },

  createAgent: (config: unknown) => {
    const { socket } = get()
    if (socket !== null && socket !== undefined && socket.readyState === WebSocket.OPEN) {
      socket.send(
        JSON.stringify({
          action: 'create_agent',
          payload: config,
        }),
      )
    }
  },

  createTask: (config: unknown) => {
    const { socket } = get()
    if (socket !== null && socket !== undefined && socket.readyState === WebSocket.OPEN) {
      socket.send(
        JSON.stringify({
          action: 'create_task',
          payload: config,
        }),
      )
    }
  },

  updateAgents: (agents: Agent[]) => set({ agents }),
  updateHiveStatus: (status: HiveStatus) => set({ hiveStatus: status }),
  updateTasks: (tasks: Task[]) => set({ tasks }),

  startHeartbeat: () => {
    const { heartbeatInterval } = get()
    if (heartbeatInterval) {
      clearInterval(heartbeatInterval)
    }

    const interval = setInterval(() => {
      const { socket, isConnected } = get()
      if (socket && isConnected && socket.readyState === WebSocket.OPEN) {
        try {
          socket.send(JSON.stringify({ action: 'ping', timestamp: Date.now() }))
          set({ lastHeartbeat: Date.now() })
          get().updateConnectionQuality()
        } catch (error) {
          console.warn('Failed to send heartbeat:', error)
          set({ connectionQuality: 'poor' })
        }
      }
    }, 30000) // Send heartbeat every 30 seconds

    set({ heartbeatInterval: interval })
  },

  stopHeartbeat: () => {
    const { heartbeatInterval } = get()
    if (heartbeatInterval) {
      clearInterval(heartbeatInterval)
      set({ heartbeatInterval: null })
    }
  },

  updateConnectionQuality: () => {
    const { lastHeartbeat, isConnected } = get()
    const now = Date.now()
    const timeSinceHeartbeat = now - lastHeartbeat

    if (!isConnected) {
      set({ connectionQuality: 'disconnected' })
    } else if (timeSinceHeartbeat < 35000) {
      // Within 35 seconds
      set({ connectionQuality: 'excellent' })
    } else if (timeSinceHeartbeat < 60000) {
      // Within 1 minute
      set({ connectionQuality: 'good' })
    } else {
      set({ connectionQuality: 'poor' })
    }
  },
}))
