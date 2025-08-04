# Multiagent Hive System

A sophisticated **hybrid neural multiagent hive system** implementing swarm intelligence with NLP self-learning capabilities. **CPU-native, GPU-optional - built for the GPU-poor.**

## System Architecture

### Core Components

#### Backend (Rust)
- **Main Entry**: `backend/src/main.rs` - Axum web server with WebSocket support
- **Hive Coordinator**: `backend/src/hive.rs` - Central orchestration system managing agent swarms
- **Agent System**: `backend/src/agent.rs` - Individual agent implementations with capabilities and behaviors
- **Task Management**: `backend/src/task.rs` - Task queue, distribution, and execution system
- **Communication**: `backend/src/communication.rs` - WebSocket handlers for real-time coordination
- **NLP Processing**: `backend/src/nlp.rs` - Lightweight natural language processing for agent communication
- **Neural Networks**: `backend/src/neural.rs` - Hybrid neural architecture with optional ruv-FANN integration

#### Frontend (TypeScript/React/Next.js)
- **Dashboard**: `frontend/src/components/HiveDashboard.tsx` - Main monitoring interface
- **Visualization**: `frontend/src/components/SwarmVisualization.tsx` - Real-time agent swarm display
- **Metrics**: `frontend/src/components/MetricsPanel.tsx` & `NeuralMetrics.tsx` - Performance monitoring
- **State Management**: `frontend/src/store/hiveStore.ts` - Zustand store for WebSocket communication
- **Agent Management**: `frontend/src/components/AgentManager.tsx` - Agent creation and configuration
- **Task Management**: `frontend/src/components/TaskManager.tsx` - Task creation and monitoring

## Key Features

### Hybrid Neural Architecture - CPU-native, GPU-optional
- **Basic NLP** (default): Lightweight CPU processing for real-time swarm coordination
- **Advanced Neural** (optional): ruv-FANN integration for complex pattern recognition
- **Philosophy**: Built for the GPU-poor - maximum intelligence on minimal hardware
- **Feature Flags**: `basic-nlp`, `advanced-neural`, `gpu-acceleration`

### Agent Types & Capabilities
- **Worker**: General task execution
- **Coordinator**: Swarm coordination and task distribution
- **Specialist**: Domain-specific expertise
- **Learner**: Continuous learning and adaptation

### Communication System
- **WebSocket**: Real-time bidirectional communication
- **REST API**: Standard CRUD operations for agents/tasks
- **Message Types**: `hive_status`, `agents_update`, `metrics_update`, `agent_created`, `task_created`

### Task Management
- **Priority Levels**: Low, Medium, High, Critical
- **Status Tracking**: Pending, Assigned, InProgress, Completed, Failed, Cancelled
- **Capability Matching**: Automatic agent assignment based on required capabilities

## Quick Start

### Basic Setup (Recommended)
```bash
# Backend with basic NLP
cd backend
cargo run

# Frontend
cd frontend
npm install
npm run dev
```

### Advanced Neural Features (Optional)
```bash
# Backend with ruv-FANN integration
cd backend
cargo run --features advanced-neural

# Run neural comparison demo
cargo run --features advanced-neural --example neural_comparison
```

## Ports & Endpoints
- **Backend**: `http://localhost:3001`
- **Frontend**: `http://localhost:3000`
- **WebSocket**: `ws://localhost:3001/ws`
- **API**: `/api/agents`, `/api/tasks`, `/api/hive/status`

## Data Structures

### Core Types
- **Agent**: ID, type, state, capabilities, position, energy, social connections
- **Task**: ID, description, priority, status, required capabilities, assigned agents
- **HiveStatus**: Metrics, swarm center, total energy, creation/update timestamps
- **SwarmMetrics**: Agent counts, task completion, performance, cohesion, learning progress

### Neural Processing
- **NLPProcessor**: Pattern recognition, semantic analysis, learning insights
- **HybridNeuralProcessor**: Optional FANN networks for advanced capabilities
- **NetworkType**: Basic, FANN, LSTM configurations

## Testing & Examples
- **Neural Comparison**: `backend/examples/neural_comparison.rs`
- **Feature Demonstrations**: Various agent types and neural processing modes
- **Performance Benchmarks**: Basic vs advanced neural processing comparisons