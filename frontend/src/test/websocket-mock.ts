import { WebSocketServer, WebSocket } from 'ws'
/* eslint-disable no-console, @typescript-eslint/no-explicit-any */
import { IncomingMessage } from 'http'

export interface MockWebSocketMessage {
  message_type: string
  data: unknown
  timestamp: string
}

export interface MockClientMessage {
  action: string
  payload?: any
}

export interface MockHiveData {
  hive_id: string
  created_at: string
  last_update: string
  metrics: {
    total_agents: number
    active_agents: number
    completed_tasks: number
    failed_tasks: number
    average_performance: number
    swarm_cohesion: number
    learning_progress: number
  }
  swarm_center: [number, number]
  total_energy: number
}

export interface MockAgent {
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

export interface MockTask {
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

/**
 * WebSocket Mock Server for Playwright E2E Tests
 * Simulates the real backend WebSocket behavior for testing without live server
 */
export class MockWebSocketServer {
  private wss: WebSocketServer | null = null
  private clients: Set<WebSocket> = new Set()
  private mockData: {
    hive: MockHiveData
    agents: MockAgent[]
    tasks: MockTask[]
  }
  private updateInterval: NodeJS.Timeout | null = null
  private port: number

  constructor(port = 3001) {
    this.port = port
    this.mockData = this.generateInitialMockData()
  }

  /**
   * Start the mock WebSocket server
   */
  async start(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        this.wss = new WebSocketServer({ port: this.port })

        this.wss.on('connection', (ws: WebSocket, request: IncomingMessage) => {
          console.log(`Mock WebSocket client connected: ${request.socket.remoteAddress}`)
          this.clients.add(ws)

          // Send initial hive status
          this.sendToClient(ws, {
            message_type: 'hive_status',
            data: this.mockData.hive,
            timestamp: new Date().toISOString(),
          })

          // Handle incoming messages
          ws.on('message', (data: Buffer) => {
            try {
              const message: MockClientMessage = JSON.parse(data.toString())
              this.handleClientMessage(ws, message)
            } catch (error) {
              console.error('Failed to parse client message:', error)
              this.sendToClient(ws, {
                message_type: 'error',
                data: { error: 'Invalid message format' },
                timestamp: new Date().toISOString(),
              })
            }
          })

          ws.on('close', () => {
            console.log('Mock WebSocket client disconnected')
            this.clients.delete(ws)
          })

          ws.on('error', error => {
            console.error('Mock WebSocket client error:', error)
            this.clients.delete(ws)
          })
        })

        this.wss.on('listening', () => {
          console.log(`Mock WebSocket server started on port ${this.port}`)
          this.startPeriodicUpdates()
          resolve()
        })

        this.wss.on('error', error => {
          console.error('Mock WebSocket server error:', error)
          reject(error)
        })
      } catch (error) {
        reject(error)
      }
    })
  }

  /**
   * Stop the mock WebSocket server
   */
  async stop(): Promise<void> {
    return new Promise(resolve => {
      if (this.updateInterval) {
        clearInterval(this.updateInterval)
        this.updateInterval = null
      }

      if (this.wss) {
        this.wss.close(() => {
          console.log('Mock WebSocket server stopped')
          this.clients.clear()
          resolve()
        })
      } else {
        resolve()
      }
    })
  }

  /**
   * Handle incoming client messages
   */
  private handleClientMessage(ws: WebSocket, message: MockClientMessage): void {
    switch (message.action) {
      case 'create_agent':
        this.handleCreateAgent(ws, message.payload)
        break
      case 'create_task':
        this.handleCreateTask(ws, message.payload)
        break
      case 'get_status':
        this.sendToClient(ws, {
          message_type: 'hive_status',
          data: this.mockData.hive,
          timestamp: new Date().toISOString(),
        })
        break
      case 'ping':
        // Respond to ping with pong
        this.sendToClient(ws, {
          message_type: 'pong',
          data: { timestamp: message.payload?.timestamp || Date.now() },
          timestamp: new Date().toISOString(),
        })
        break
      default:
        this.sendToClient(ws, {
          message_type: 'error',
          data: { error: `Unknown action: ${message.action}` },
          timestamp: new Date().toISOString(),
        })
    }
  }

  /**
   * Handle agent creation
   */
  private handleCreateAgent(ws: WebSocket, payload?: any): void {
    const newAgent = this.generateMockAgent(payload)
    this.mockData.agents.push(newAgent)
    this.mockData.hive.metrics.total_agents++
    this.mockData.hive.metrics.active_agents++

    this.sendToClient(ws, {
      message_type: 'agent_created',
      data: {
        success: true,
        agent_id: newAgent.id,
        message: `Created ${newAgent.type} agent with ID: ${newAgent.id}`,
      },
      timestamp: new Date().toISOString(),
    })

    // Broadcast agent update to all clients
    this.broadcast({
      message_type: 'agents_update',
      data: { agents: this.mockData.agents },
      timestamp: new Date().toISOString(),
    })
  }

  /**
   * Handle task creation
   */
  private handleCreateTask(ws: WebSocket, payload?: any): void {
    const newTask = this.generateMockTask(payload)
    this.mockData.tasks.push(newTask)

    this.sendToClient(ws, {
      message_type: 'task_created',
      data: {
        success: true,
        task_id: newTask.id,
        message: `Created task with ID: ${newTask.id}`,
      },
      timestamp: new Date().toISOString(),
    })

    // Broadcast task update to all clients
    this.broadcast({
      message_type: 'tasks_update',
      data: { tasks: this.mockData.tasks },
      timestamp: new Date().toISOString(),
    })
  }

  /**
   * Start periodic updates to simulate real-time data
   */
  private startPeriodicUpdates(): void {
    this.updateInterval = setInterval(() => {
      // Update metrics
      this.updateMockMetrics()

      // Send metrics update
      this.broadcast({
        message_type: 'metrics_update',
        data: {
          metrics: this.mockData.hive.metrics,
          swarm_center: this.mockData.hive.swarm_center,
          total_energy: this.mockData.hive.total_energy,
        },
        timestamp: new Date().toISOString(),
      })

      // Send agents update
      this.broadcast({
        message_type: 'agents_update',
        data: { agents: this.mockData.agents },
        timestamp: new Date().toISOString(),
      })

      // Send tasks update
      this.broadcast({
        message_type: 'tasks_update',
        data: { tasks: this.mockData.tasks },
        timestamp: new Date().toISOString(),
      })
    }, 5000) // Update every 5 seconds
  }

  /**
   * Update mock metrics with some variation
   */
  private updateMockMetrics(): void {
    const { metrics } = this.mockData.hive

    // Simulate some activity
    metrics.average_performance = Math.max(
      0,
      Math.min(1, metrics.average_performance + (Math.random() - 0.5) * 0.1),
    )
    metrics.swarm_cohesion = Math.max(
      0,
      Math.min(1, metrics.swarm_cohesion + (Math.random() - 0.5) * 0.05),
    )
    metrics.learning_progress = Math.max(
      0,
      Math.min(1, metrics.learning_progress + (Math.random() - 0.5) * 0.02),
    )

    // Update swarm center slightly
    this.mockData.hive.swarm_center = [
      this.mockData.hive.swarm_center[0] + (Math.random() - 0.5) * 2,
      this.mockData.hive.swarm_center[1] + (Math.random() - 0.5) * 2,
    ]

    // Update total energy
    this.mockData.hive.total_energy = Math.max(
      0,
      this.mockData.hive.total_energy + (Math.random() - 0.5) * 10,
    )

    this.mockData.hive.last_update = new Date().toISOString()
  }

  /**
   * Send message to specific client
   */
  private sendToClient(ws: WebSocket, message: MockWebSocketMessage): void {
    if (ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify(message))
    }
  }

  /**
   * Broadcast message to all connected clients
   */
  public broadcast(message: MockWebSocketMessage): void {
    const messageStr = JSON.stringify(message)
    this.clients.forEach(ws => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(messageStr)
      }
    })
  }

  /**
   * Generate initial mock data
   */
  private generateInitialMockData(): {
    hive: MockHiveData
    agents: MockAgent[]
    tasks: MockTask[]
  } {
    const agents = Array.from({ length: 5 }, (_, i) =>
      this.generateMockAgent({
        type: ['Worker', 'Coordinator', 'Specialist', 'Learner'][i % 4],
        name: `Agent-${i + 1}`,
      }),
    )

    const tasks = Array.from({ length: 3 }, (_, i) =>
      this.generateMockTask({
        description: `Mock task ${i + 1}`,
        priority: Math.floor(Math.random() * 5) + 1,
      }),
    )

    return {
      hive: {
        hive_id: 'mock-hive-001',
        created_at: new Date().toISOString(),
        last_update: new Date().toISOString(),
        metrics: {
          total_agents: agents.length,
          active_agents: agents.length,
          completed_tasks: 2,
          failed_tasks: 0,
          average_performance: 0.85,
          swarm_cohesion: 0.92,
          learning_progress: 0.78,
        },
        swarm_center: [50, 50],
        total_energy: 1000,
      },
      agents,
      tasks,
    }
  }

  /**
   * Generate a mock agent
   */
  private generateMockAgent(config?: any): MockAgent {
    const types = ['Worker', 'Coordinator', 'Specialist', 'Learner']
    const agentType = config?.type || types[Math.floor(Math.random() * types.length)]

    return {
      id: `agent-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      name: config?.name || `${agentType}-${Math.floor(Math.random() * 1000)}`,
      type: agentType,
      state: 'active',
      capabilities: [
        {
          name: 'processing',
          proficiency: Math.random() * 0.5 + 0.5,
          learning_rate: Math.random() * 0.1 + 0.05,
        },
        {
          name: 'communication',
          proficiency: Math.random() * 0.5 + 0.5,
          learning_rate: Math.random() * 0.1 + 0.05,
        },
      ],
      position: [Math.random() * 100, Math.random() * 100],
      energy: Math.random() * 50 + 50,
      experience_count: Math.floor(Math.random() * 100),
      social_connections: Math.floor(Math.random() * 10),
    }
  }

  /**
   * Generate a mock task
   */
  private generateMockTask(config?: any): MockTask {
    const statuses = ['pending', 'running', 'completed']
    const types = ['analysis', 'processing', 'coordination', 'learning']

    return {
      id: `task-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      description: config?.description || 'Mock task description',
      type: config?.type || types[Math.floor(Math.random() * types.length)],
      priority: config?.priority || Math.floor(Math.random() * 5) + 1,
      status: config?.status || statuses[Math.floor(Math.random() * statuses.length)],
      created_at: new Date().toISOString(),
      required_capabilities: [
        {
          name: 'processing',
          min_proficiency: Math.random() * 0.5 + 0.3,
          weight: Math.random() * 0.5 + 0.5,
        },
      ],
    }
  }

  /**
   * Get current mock data (useful for assertions)
   */
  getMockData() {
    return { ...this.mockData }
  }

  /**
   * Manually trigger an update (useful for testing)
   */
  triggerUpdate(): void {
    this.updateMockMetrics()
    this.broadcast({
      message_type: 'metrics_update',
      data: {
        metrics: this.mockData.hive.metrics,
        swarm_center: this.mockData.hive.swarm_center,
        total_energy: this.mockData.hive.total_energy,
      },
      timestamp: new Date().toISOString(),
    })
  }

  /**
   * Simulate an error condition
   */
  simulateError(errorMessage: string): void {
    this.broadcast({
      message_type: 'error',
      data: { error: errorMessage },
      timestamp: new Date().toISOString(),
    })
  }

  /**
   * Get number of connected clients
   */
  getConnectedClientsCount(): number {
    return this.clients.size
  }
}
