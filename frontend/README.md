# Multiagent Hive Frontend

The frontend component of the Multiagent Hive System, built with Next.js, React, and TypeScript for a modern, responsive user interface.

## Overview

This Next.js-based frontend provides:

- **Real-time dashboard** for monitoring agent swarms
- **Interactive visualizations** of swarm behavior
- **Task management interface** for creating and monitoring tasks
- **Agent configuration tools** for setting up and managing agents
- **Performance metrics** with charts and analytics
- **Responsive design** that works on all devices
- **Accessibility features** following WCAG 2.1 AA standards

## Architecture

### Technology Stack

- **Framework**: Next.js 15 with App Router
- **Language**: TypeScript for type safety
- **Styling**: Tailwind CSS for utility-first styling
- **State Management**: Zustand for client-side state
- **Charts**: Recharts for data visualization
- **Icons**: Lucide React for consistent iconography
- **Build Tool**: Next.js built-in bundler with optimizations

### Project Structure

```
frontend/
├── src/
│   ├── app/                    # Next.js App Router
│   │   ├── globals.css         # Global styles
│   │   ├── layout.tsx          # Root layout
│   │   ├── page.tsx            # Home page
│   │   └── api/                # API routes (if needed)
│   ├── components/             # React components
│   │   ├── AgentManager.tsx    # Agent management
│   │   ├── HiveDashboard.tsx   # Main dashboard
│   │   ├── MetricsPanel.tsx    # Metrics display
│   │   ├── NeuralMetrics.tsx   # Neural processing metrics
│   │   ├── ResourceMonitor.tsx # System resources
│   │   ├── SwarmVisualization.tsx # Swarm visualization
│   │   ├── TaskManager.tsx     # Task management
│   │   └── ui/                 # Reusable UI components
│   ├── store/                  # State management
│   │   └── hiveStore.ts        # Zustand store
│   └── lib/                    # Utilities
│       ├── api.ts              # API client
│       ├── types.ts            # TypeScript types
│       └── utils.ts            # Helper functions
├── public/                     # Static assets
├── eslint.config.js            # ESLint configuration
├── next.config.js              # Next.js configuration
├── package.json                # Dependencies
├── tailwind.config.js          # Tailwind configuration
└── tsconfig.json               # TypeScript configuration
```

## Quick Start

### Prerequisites

- Node.js 18+
- npm or yarn
- Backend server running (see backend README)

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/multiagent-hive.git
cd multiagent-hive/frontend

# Install dependencies
npm install

# Start development server
npm run dev
```

The frontend will be available at `http://localhost:3000`.

### Basic Usage

```typescript
// Example: Creating an agent
import { useHiveStore } from '../store/hiveStore';

function AgentCreator() {
  const createAgent = useHiveStore(state => state.createAgent);

  const handleCreateAgent = async () => {
    await createAgent({
      name: 'New Agent',
      type: 'Worker',
      capabilities: [
        { name: 'data_processing', proficiency: 0.8 }
      ]
    });
  };

  return (
    <button onClick={handleCreateAgent}>
      Create Agent
    </button>
  );
}
```

## Configuration

### Environment Variables

Create `.env.local` in the frontend directory:

```env
# API Configuration
NEXT_PUBLIC_API_URL=http://localhost:3001
NEXT_PUBLIC_WS_URL=ws://localhost:3001/ws

# Application Settings
NEXT_PUBLIC_APP_NAME="Multiagent Hive"
NEXT_PUBLIC_APP_VERSION="0.1.0"
NEXT_PUBLIC_APP_ENV=development

# Feature Flags
NEXT_PUBLIC_ADVANCED_METRICS=true
NEXT_PUBLIC_DEBUG_PANEL=false
NEXT_PUBLIC_EXPERIMENTAL_FEATURES=false

# UI Configuration
NEXT_PUBLIC_THEME=default
NEXT_PUBLIC_LOCALE=en
NEXT_PUBLIC_TIMEZONE=UTC

# Performance
NEXT_PUBLIC_POLLING_INTERVAL=5000
NEXT_PUBLIC_MAX_RETRIES=3
NEXT_PUBLIC_REQUEST_TIMEOUT=10000
```

### Build Configuration

```javascript
// next.config.js
/** @type {import('next').NextConfig} */
const nextConfig = {
  // Enable experimental features
  experimental: {
    appDir: true,
  },

  // Image optimization
  images: {
    domains: ['localhost'],
  },

  // Environment variables
  env: {
    API_URL: process.env.NEXT_PUBLIC_API_URL,
  },

  // Bundle analyzer (optional)
  ...(process.env.ANALYZE === 'true' && {
    webpack: (config) => {
      // Add bundle analyzer
      return config;
    },
  }),
};

module.exports = nextConfig;
```

## Key Components

### HiveDashboard

The main dashboard component that displays:

- Real-time swarm status
- Active agents count
- Task completion metrics
- System health indicators

```typescript
// components/HiveDashboard.tsx
import { useHiveStore } from '../store/hiveStore';
import { MetricsPanel } from './MetricsPanel';
import { SwarmVisualization } from './SwarmVisualization';

export function HiveDashboard() {
  const { status, agents, tasks } = useHiveStore();

  return (
    <div className="dashboard">
      <header>
        <h1>Hive Status: {status?.state}</h1>
        <div className="stats">
          <div>Agents: {agents.length}</div>
          <div>Tasks: {tasks.length}</div>
        </div>
      </header>

      <div className="dashboard-grid">
        <MetricsPanel metrics={status?.metrics} />
        <SwarmVisualization agents={agents} />
      </div>
    </div>
  );
}
```

### SwarmVisualization

Interactive visualization of the agent swarm:

```typescript
// components/SwarmVisualization.tsx
import { useEffect, useRef } from 'react';

interface SwarmVisualizationProps {
  agents: Agent[];
  width?: number;
  height?: number;
}

export function SwarmVisualization({
  agents,
  width = 800,
  height = 600
}: SwarmVisualizationProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas
    ctx.clearRect(0, 0, width, height);

    // Draw agents
    agents.forEach(agent => {
      ctx.beginPath();
      ctx.arc(agent.position[0], agent.position[1], 5, 0, 2 * Math.PI);
      ctx.fillStyle = getAgentColor(agent);
      ctx.fill();
    });
  }, [agents, width, height]);

  return (
    <canvas
      ref={canvasRef}
      width={width}
      height={height}
      className="swarm-canvas"
    />
  );
}
```

### AgentManager

Component for managing agents:

```typescript
// components/AgentManager.tsx
import { useState } from 'react';
import { useHiveStore } from '../store/hiveStore';

export function AgentManager() {
  const { agents, createAgent, updateAgent, deleteAgent } = useHiveStore();
  const [newAgent, setNewAgent] = useState({
    name: '',
    type: 'Worker' as AgentType,
    capabilities: [] as Capability[]
  });

  const handleCreateAgent = async () => {
    await createAgent(newAgent);
    setNewAgent({ name: '', type: 'Worker', capabilities: [] });
  };

  return (
    <div className="agent-manager">
      <h2>Agent Management</h2>

      {/* Create Agent Form */}
      <form onSubmit={handleCreateAgent}>
        <input
          type="text"
          placeholder="Agent name"
          value={newAgent.name}
          onChange={(e) => setNewAgent(prev => ({
            ...prev,
            name: e.target.value
          }))}
        />
        <select
          value={newAgent.type}
          onChange={(e) => setNewAgent(prev => ({
            ...prev,
            type: e.target.value as AgentType
          }))}
        >
          <option value="Worker">Worker</option>
          <option value="Coordinator">Coordinator</option>
          <option value="Specialist">Specialist</option>
        </select>
        <button type="submit">Create Agent</button>
      </form>

      {/* Agent List */}
      <div className="agent-list">
        {agents.map(agent => (
          <div key={agent.id} className="agent-card">
            <h3>{agent.name}</h3>
            <p>Type: {agent.type}</p>
            <p>Status: {agent.status}</p>
            <button onClick={() => deleteAgent(agent.id)}>
              Delete
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
```

## State Management

### Zustand Store

```typescript
// store/hiveStore.ts
import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';

interface HiveState {
  // State
  status: HiveStatus | null;
  agents: Agent[];
  tasks: Task[];
  metrics: Metrics | null;
  isConnected: boolean;

  // Actions
  connect: () => void;
  disconnect: () => void;
  createAgent: (agent: CreateAgentRequest) => Promise<void>;
  updateAgent: (id: string, updates: UpdateAgentRequest) => Promise<void>;
  deleteAgent: (id: string) => Promise<void>;
  createTask: (task: CreateTaskRequest) => Promise<void>;
  updateTask: (id: string, updates: UpdateTaskRequest) => Promise<void>;
  deleteTask: (id: string) => Promise<void>;
}

export const useHiveStore = create<HiveState>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    status: null,
    agents: [],
    tasks: [],
    metrics: null,
    isConnected: false,

    // WebSocket connection
    connect: () => {
      const ws = new WebSocket(process.env.NEXT_PUBLIC_WS_URL!);

      ws.onopen = () => {
        set({ isConnected: true });
      };

      ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        handleWebSocketMessage(data, set);
      };

      ws.onclose = () => {
        set({ isConnected: false });
      };

      // Store WebSocket instance for cleanup
      (window as any).hiveWebSocket = ws;
    },

    disconnect: () => {
      const ws = (window as any).hiveWebSocket;
      if (ws) {
        ws.close();
      }
      set({ isConnected: false });
    },

    // Agent actions
    createAgent: async (agent) => {
      const response = await fetch('/api/agents', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(agent)
      });

      if (!response.ok) {
        throw new Error('Failed to create agent');
      }
    },

    updateAgent: async (id, updates) => {
      const response = await fetch(`/api/agents/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(updates)
      });

      if (!response.ok) {
        throw new Error('Failed to update agent');
      }
    },

    deleteAgent: async (id) => {
      const response = await fetch(`/api/agents/${id}`, {
        method: 'DELETE'
      });

      if (!response.ok) {
        throw new Error('Failed to delete agent');
      }
    },

    // Task actions
    createTask: async (task) => {
      const response = await fetch('/api/tasks', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(task)
      });

      if (!response.ok) {
        throw new Error('Failed to create task');
      }
    },

    updateTask: async (id, updates) => {
      const response = await fetch(`/api/tasks/${id}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(updates)
      });

      if (!response.ok) {
        throw new Error('Failed to update task');
      }
    },

    deleteTask: async (id) => {
      const response = await fetch(`/api/tasks/${id}`, {
        method: 'DELETE'
      });

      if (!response.ok) {
        throw new Error('Failed to delete task');
      }
    }
  }))
);

// WebSocket message handler
function handleWebSocketMessage(data: any, set: any) {
  switch (data.type) {
    case 'hive_status':
      set({ status: data.data });
      break;
    case 'agents_update':
      set({ agents: data.data.agents });
      break;
    case 'task_update':
      // Update specific task in the list
      set((state: HiveState) => ({
        tasks: state.tasks.map(task =>
          task.id === data.data.task_id
            ? { ...task, ...data.data }
            : task
        )
      }));
      break;
    case 'metrics_update':
      set({ metrics: data.data });
      break;
  }
}
```

## API Integration

### API Client

```typescript
// lib/api.ts
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';

class ApiClient {
  private baseURL: string;

  constructor(baseURL: string) {
    this.baseURL = baseURL;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseURL}${endpoint}`;
    const config: RequestInit = {
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
      ...options,
    };

    const response = await fetch(url, config);

    if (!response.ok) {
      throw new Error(`API request failed: ${response.statusText}`);
    }

    return response.json();
  }

  // Agent endpoints
  async getAgents(): Promise<Agent[]> {
    return this.request('/api/agents');
  }

  async createAgent(agent: CreateAgentRequest): Promise<Agent> {
    return this.request('/api/agents', {
      method: 'POST',
      body: JSON.stringify(agent),
    });
  }

  async getAgent(id: string): Promise<Agent> {
    return this.request(`/api/agents/${id}`);
  }

  async updateAgent(id: string, updates: UpdateAgentRequest): Promise<Agent> {
    return this.request(`/api/agents/${id}`, {
      method: 'PUT',
      body: JSON.stringify(updates),
    });
  }

  async deleteAgent(id: string): Promise<void> {
    return this.request(`/api/agents/${id}`, {
      method: 'DELETE',
    });
  }

  // Task endpoints
  async getTasks(): Promise<Task[]> {
    return this.request('/api/tasks');
  }

  async createTask(task: CreateTaskRequest): Promise<Task> {
    return this.request('/api/tasks', {
      method: 'POST',
      body: JSON.stringify(task),
    });
  }

  async getTask(id: string): Promise<Task> {
    return this.request(`/api/tasks/${id}`);
  }

  async updateTask(id: string, updates: UpdateTaskRequest): Promise<Task> {
    return this.request(`/api/tasks/${id}`, {
      method: 'PUT',
      body: JSON.stringify(updates),
    });
  }

  async deleteTask(id: string): Promise<void> {
    return this.request(`/api/tasks/${id}`, {
      method: 'DELETE',
    });
  }

  // Hive endpoints
  async getHiveStatus(): Promise<HiveStatus> {
    return this.request('/api/hive/status');
  }

  async getHiveMetrics(): Promise<Metrics> {
    return this.request('/api/hive/metrics');
  }

  async resetHive(): Promise<void> {
    return this.request('/api/hive/reset', {
      method: 'POST',
    });
  }
}

export const apiClient = new ApiClient(API_BASE_URL);
```

## Styling

### Tailwind Configuration

```javascript
// tailwind.config.js
/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/pages/**/*.{js,ts,jsx,tsx,mdx}',
    './src/components/**/*.{js,ts,jsx,tsx,mdx}',
    './src/app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#eff6ff',
          500: '#3b82f6',
          600: '#2563eb',
          900: '#1e3a8a',
        },
        secondary: {
          50: '#f8fafc',
          500: '#64748b',
          600: '#475569',
          900: '#0f172a',
        },
      },
      animation: {
        'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
      },
    },
  },
  plugins: [],
};
```

### Global Styles

```css
/* src/app/globals.css */
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  html {
    @apply scroll-smooth;
  }

  body {
    @apply bg-gray-50 text-gray-900;
  }

  /* Dark mode support */
  @media (prefers-color-scheme: dark) {
    body {
      @apply bg-gray-900 text-gray-100;
    }
  }
}

@layer components {
  .btn {
    @apply px-4 py-2 rounded-md font-medium transition-colors;
  }

  .btn-primary {
    @apply bg-primary-500 text-white hover:bg-primary-600;
  }

  .btn-secondary {
    @apply bg-secondary-500 text-white hover:bg-secondary-600;
  }

  .card {
    @apply bg-white rounded-lg shadow-md p-6;
  }

  /* Dark mode variants */
  @media (prefers-color-scheme: dark) {
    .card {
      @apply bg-gray-800;
    }
  }
}
```

## Development

### Available Scripts

```json
{
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start",
    "lint": "eslint .",
    "lint:fix": "eslint . --fix",
    "type-check": "tsc --noEmit",
    "test": "jest",
    "test:watch": "jest --watch",
    "test:coverage": "jest --coverage"
  }
}
```

### Development Workflow

```bash
# Start development server
npm run dev

# Run linting
npm run lint

# Fix linting issues
npm run lint:fix

# Type checking
npm run type-check

# Run tests
npm run test

# Build for production
npm run build
```

### Testing

```typescript
// __tests__/components/AgentCard.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { AgentCard } from '../../components/AgentCard';

const mockAgent = {
  id: '1',
  name: 'Test Agent',
  type: 'Worker',
  status: 'Idle',
  capabilities: [],
  position: [0, 0],
  energy: 100,
  created_at: new Date().toISOString(),
  last_active: new Date().toISOString(),
};

describe('AgentCard', () => {
  it('renders agent information correctly', () => {
    render(<AgentCard agent={mockAgent} />);

    expect(screen.getByText('Test Agent')).toBeInTheDocument();
    expect(screen.getByText('Worker')).toBeInTheDocument();
    expect(screen.getByText('Idle')).toBeInTheDocument();
  });

  it('calls onSelect when clicked', () => {
    const mockOnSelect = jest.fn();
    render(<AgentCard agent={mockAgent} onSelect={mockOnSelect} />);

    fireEvent.click(screen.getByRole('button'));
    expect(mockOnSelect).toHaveBeenCalledWith(mockAgent);
  });
});
```

## Deployment

### Build Optimization

```javascript
// next.config.js - Production optimizations
const nextConfig = {
  // Enable SWC minification
  swcMinify: true,

  // Image optimization
  images: {
    formats: ['image/webp', 'image/avif'],
  },

  // Bundle analyzer
  ...(process.env.ANALYZE && {
    webpack: (config, { buildId, dev, isServer, defaultLoaders, webpack }) => {
      // Add webpack bundle analyzer
      return config;
    },
  }),

  // Compression
  compress: true,

  // CDN support
  assetPrefix: process.env.CDN_URL,
};

module.exports = nextConfig;
```

### Docker Deployment

```dockerfile
# Dockerfile
FROM node:20-alpine as builder

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

COPY . .
RUN npm run build

FROM node:20-alpine as runner

WORKDIR /app
COPY --from=builder /app/package*.json ./
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/public ./public

EXPOSE 3000
ENV NODE_ENV=production
CMD ["npm", "start"]
```

### Static Export

```javascript
// next.config.js - Static export
const nextConfig = {
  output: 'export',
  trailingSlash: true,
  images: {
    unoptimized: true,
  },
};

module.exports = nextConfig;
```

## Performance

### Optimization Techniques

```typescript
// Lazy loading components
const MetricsPanel = lazy(() => import('./components/MetricsPanel'));

// Memoization
const AgentList = memo(({ agents }: { agents: Agent[] }) => {
  return (
    <div>
      {agents.map(agent => (
        <AgentCard key={agent.id} agent={agent} />
      ))}
    </div>
  );
});

// Data fetching optimization
import { SWRConfig } from 'swr';

function App({ Component, pageProps }: AppProps) {
  return (
    <SWRConfig
      value={{
        refreshInterval: 5000,
        revalidateOnFocus: false,
      }}
    >
      <Component {...pageProps} />
    </SWRConfig>
  );
}
```

### Bundle Analysis

```bash
# Analyze bundle size
npm install --save-dev @next/bundle-analyzer

# Add to package.json scripts
"analyze": "ANALYZE=true npm run build"
```

## Accessibility

### ARIA Support

```typescript
// Accessible components
function AccessibleButton({
  children,
  onClick,
  ariaLabel,
  disabled = false
}: {
  children: React.ReactNode;
  onClick: () => void;
  ariaLabel: string;
  disabled?: boolean;
}) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      aria-label={ariaLabel}
      aria-disabled={disabled}
      className="btn btn-primary"
    >
      {children}
    </button>
  );
}
```

### Keyboard Navigation

```typescript
// Keyboard shortcuts
useEffect(() => {
  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.ctrlKey || event.metaKey) {
      switch (event.key) {
        case '/':
          event.preventDefault();
          openHelpPanel();
          break;
        case 'n':
          event.preventDefault();
          openCreateAgentModal();
          break;
      }
    }
  };

  document.addEventListener('keydown', handleKeyDown);
  return () => document.removeEventListener('keydown', handleKeyDown);
}, []);
```

## Troubleshooting

### Common Issues

#### Build Errors

```bash
# Clear Next.js cache
rm -rf .next

# Clear node_modules
rm -rf node_modules package-lock.json
npm install

# Check Node.js version
node --version
```

#### Runtime Errors

```bash
# Enable React DevTools
# Open browser dev tools and check console

# Check network requests
# Use browser network tab to inspect API calls

# Enable debug logging
localStorage.setItem('debug', 'true');
```

#### WebSocket Issues

```javascript
// Test WebSocket connection
const ws = new WebSocket('ws://localhost:3001/ws');
ws.onopen = () => console.log('Connected');
ws.onerror = (error) => console.error('WebSocket error:', error);
```

### Performance Issues

```bash
# Check bundle size
npm run build
ls -lh .next/static/chunks/

# Profile React components
# Use React DevTools Profiler

# Check for memory leaks
# Use browser memory tab
```

## Contributing

### Code Standards

- Use TypeScript for all new code
- Follow ESLint configuration
- Use functional components with hooks
- Implement proper error boundaries
- Write comprehensive tests
- Follow accessibility guidelines

### Component Guidelines

```typescript
// Component structure
interface ComponentProps {
  // Define props interface
}

function ComponentName({ prop1, prop2 }: ComponentProps) {
  // Hooks at the top
  const [state, setState] = useState(initialState);

  // Event handlers
  const handleEvent = useCallback(() => {
    // Handle event
  }, [dependencies]);

  // Effects
  useEffect(() => {
    // Side effects
    return () => {
      // Cleanup
    };
  }, [dependencies]);

  // Render
  return (
    <div>
      {/* JSX */}
    </div>
  );
}

export default memo(ComponentName);
```

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## Support

- **Documentation**: [docs/](../docs/) directory
- **Issues**: [GitHub Issues](../../issues)
- **Discussions**: [GitHub Discussions](../../discussions)
- **Email**: support@multiagent-hive.dev