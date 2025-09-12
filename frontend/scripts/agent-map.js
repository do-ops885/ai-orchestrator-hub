#!/usr/bin/env node

const fs = require('fs')
const path = require('path')

async function mapAgentRelationships() {
  const outputPath =
    process.argv[2] || path.join(process.cwd(), '..', 'monitoring', 'agent-map.json')

  console.log('🗺️  Mapping agent relationships...')

  try {
    // Get agents data
    const registryUrl = process.argv[3] || 'http://localhost:8000'
    const agentsUrl = `${registryUrl}/api/agents`
    const tasksUrl = `${registryUrl}/api/tasks`

    const [agentsResponse, tasksResponse] = await Promise.all([fetch(agentsUrl), fetch(tasksUrl)])

    if (!agentsResponse.ok || !tasksResponse.ok) {
      throw new Error('Failed to fetch data from API')
    }

    const agentsData = await agentsResponse.json()
    const tasksData = await tasksResponse.json()

    // Create relationship map
    const agentMap = {
      timestamp: new Date().toISOString(),
      agents: {},
      relationships: {
        communication: [],
        collaboration: [],
        hierarchy: [],
      },
      network_stats: {
        total_connections: 0,
        average_connections_per_agent: 0,
        most_connected_agent: null,
        isolated_agents: [],
      },
    }

    // Process agents
    if (agentsData.agents && Array.isArray(agentsData.agents)) {
      agentsData.agents.forEach(agent => {
        agentMap.agents[agent.id] = {
          id: agent.id,
          type: agent.type || 'unknown',
          state: agent.state || 'unknown',
          connections: [],
          tasks_assigned: 0,
          tasks_completed: 0,
        }
      })
    }

    // Process tasks to find relationships
    if (tasksData.tasks && Array.isArray(tasksData.tasks)) {
      tasksData.tasks.forEach(task => {
        const assignedAgent = task.assigned_agent
        const createdBy = task.created_by

        if (assignedAgent && agentMap.agents[assignedAgent]) {
          agentMap.agents[assignedAgent].tasks_assigned++

          if (task.status === 'completed') {
            agentMap.agents[assignedAgent].tasks_completed++
          }
        }

        // Track creator-agent relationships
        if (createdBy && assignedAgent && createdBy !== assignedAgent) {
          agentMap.relationships.collaboration.push({
            from: createdBy,
            to: assignedAgent,
            type: 'task_assignment',
            task_id: task.id,
          })
        }
      })
    }

    // Calculate network statistics
    let totalConnections = 0
    let maxConnections = 0
    let mostConnected = null
    const isolatedAgents = []

    Object.values(agentMap.agents).forEach(agent => {
      const connections = agent.tasks_assigned + agent.tasks_completed
      agent.connections_count = connections
      totalConnections += connections

      if (connections > maxConnections) {
        maxConnections = connections
        mostConnected = agent.id
      }

      if (connections === 0) {
        isolatedAgents.push(agent.id)
      }
    })

    agentMap.network_stats.total_connections = totalConnections
    agentMap.network_stats.average_connections_per_agent =
      Object.keys(agentMap.agents).length > 0
        ? totalConnections / Object.keys(agentMap.agents).length
        : 0
    agentMap.network_stats.most_connected_agent = mostConnected
    agentMap.network_stats.isolated_agents = isolatedAgents

    // Ensure output directory exists
    const outputDir = path.dirname(outputPath)
    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true })
    }

    // Write map
    fs.writeFileSync(outputPath, JSON.stringify(agentMap, null, 2))
    console.log('✅ Agent relationship map created')
    console.log(`💾 Map saved to ${outputPath}`)

    // Display summary
    console.log('\n📊 Network Statistics:')
    console.log(`   Total Agents: ${Object.keys(agentMap.agents).length}`)
    console.log(`   Total Connections: ${agentMap.network_stats.total_connections}`)
    console.log(
      `   Average Connections: ${agentMap.network_stats.average_connections_per_agent.toFixed(2)}`,
    )
    console.log(`   Most Connected: ${agentMap.network_stats.most_connected_agent || 'None'}`)
    console.log(`   Isolated Agents: ${agentMap.network_stats.isolated_agents.length}`)
  } catch (error) {
    console.error('❌ Agent mapping failed:', error.message)
    process.exit(1)
  }
}

mapAgentRelationships()
