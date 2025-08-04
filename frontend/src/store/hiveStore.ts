import { create } from 'zustand';

export interface Agent {
  id: string;
  name: string;
  type: string;
  state: string;
  capabilities: Array<{
    name: string;
    proficiency: number;
    learning_rate: number;
  }>;
  position: [number, number];
  energy: number;
  experience_count: number;
  social_connections: number;
}

export interface HiveMetrics {
  total_agents: number;
  active_agents: number;
  completed_tasks: number;
  failed_tasks: number;
  average_performance: number;
  swarm_cohesion: number;
  learning_progress: number;
}

export interface HiveStatus {
  hive_id: string;
  created_at: string;
  last_update: string;
  metrics: HiveMetrics;
  swarm_center: [number, number];
  total_energy: number;
}

interface HiveStore {
  // Connection state
  isConnected: boolean;
  socket: WebSocket | null;
  
  // Data
  agents: Agent[];
  hiveStatus: HiveStatus | null;
  tasks: any[];
  
  // Actions
  connectWebSocket: (url: string) => void;
  disconnect: () => void;
  createAgent: (config: any) => void;
  createTask: (config: any) => void;
  updateAgents: (agents: Agent[]) => void;
  updateHiveStatus: (status: HiveStatus) => void;
}

export const useHiveStore = create<HiveStore>((set, get) => ({
  isConnected: false,
  socket: null,
  agents: [],
  hiveStatus: null,
  tasks: [],

  connectWebSocket: (url: string) => {
    const socket = new WebSocket(url);
    
    socket.onopen = () => {
      console.log('WebSocket connected');
      set({ isConnected: true, socket });
    };
    
    socket.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        
        switch (message.message_type) {
          case 'hive_status':
            set({ hiveStatus: message.data });
            break;
          case 'agents_update':
            set({ agents: message.data.agents || [] });
            break;
          case 'metrics_update':
            const currentStatus = get().hiveStatus;
            if (currentStatus && message.data.metrics) {
              set({
                hiveStatus: {
                  ...currentStatus,
                  metrics: message.data.metrics,
                  swarm_center: message.data.swarm_center || currentStatus.swarm_center,
                  total_energy: message.data.total_energy || currentStatus.total_energy,
                }
              });
            }
            break;
          case 'agent_created':
          case 'task_created':
            console.log('Created:', message.data);
            break;
          case 'error':
            console.error('Hive error:', message.data.error);
            break;
        }
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };
    
    socket.onclose = () => {
      console.log('WebSocket disconnected');
      set({ isConnected: false, socket: null });
    };
    
    socket.onerror = (error) => {
      console.error('WebSocket error:', error);
      set({ isConnected: false });
    };
  },

  disconnect: () => {
    const { socket } = get();
    if (socket) {
      socket.close();
    }
    set({ isConnected: false, socket: null });
  },

  createAgent: (config: any) => {
    const { socket } = get();
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify({
        action: 'create_agent',
        payload: config
      }));
    }
  },

  createTask: (config: any) => {
    const { socket } = get();
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify({
        action: 'create_task',
        payload: config
      }));
    }
  },

  updateAgents: (agents: Agent[]) => set({ agents }),
  updateHiveStatus: (status: HiveStatus) => set({ hiveStatus: status }),
}));