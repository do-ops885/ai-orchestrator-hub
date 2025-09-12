#!/usr/bin/env node

const fs = require('fs')
const path = require('path')

async function analyzeCommunicationPatterns() {
  const outputPath =
    process.argv[2] || path.join(process.cwd(), '..', 'monitoring', 'comm-patterns.json')
  const registryUrl = process.argv[3] || 'http://localhost:8000'

  console.log('📡 Analyzing agent communication patterns...')

  try {
    // Get agents and tasks data
    const [agentsResponse, tasksResponse] = await Promise.all([
      fetch(`${registryUrl}/api/agents`),
      fetch(`${registryUrl}/api/tasks`),
    ])

    if (!agentsResponse.ok || !tasksResponse.ok) {
      throw new Error('Failed to fetch data from API')
    }

    const agentsData = await agentsResponse.json()
    const tasksData = await tasksResponse.json()

    // Analyze communication patterns
    const commPatterns = {
      timestamp: new Date().toISOString(),
      total_agents: 0,
      communication_matrix: {},
      interaction_frequency: {},
      collaboration_network: {
        nodes: [],
        links: [],
      },
      patterns: {
        most_active_agent: null,
        most_collaborative_pair: null,
        communication_clusters: [],
        isolated_agents: [],
      },
    }

    // Initialize communication matrix
    if (agentsData.agents && Array.isArray(agentsData.agents)) {
      commPatterns.total_agents = agentsData.agents.length

      agentsData.agents.forEach(agent => {
        commPatterns.communication_matrix[agent.id] = {}
        commPatterns.interaction_frequency[agent.id] = 0

        // Add nodes for visualization
        commPatterns.collaboration_network.nodes.push({
          id: agent.id,
          type: agent.type || 'unknown',
          group: agent.type || 'unknown',
        })
      })
    }

    // Analyze task-based communication
    if (tasksData.tasks && Array.isArray(tasksData.tasks)) {
      tasksData.tasks.forEach(task => {
        const creator = task.created_by
        const assignee = task.assigned_agent

        if (creator && assignee && creator !== assignee) {
          // Record communication
          if (!commPatterns.communication_matrix[creator][assignee]) {
            commPatterns.communication_matrix[creator][assignee] = 0
          }
          commPatterns.communication_matrix[creator][assignee]++

          commPatterns.interaction_frequency[creator]++
          commPatterns.interaction_frequency[assignee]++

          // Add link for visualization
          commPatterns.collaboration_network.links.push({
            source: creator,
            target: assignee,
            value: commPatterns.communication_matrix[creator][assignee],
            type: 'task_assignment',
          })
        }
      })
    }

    // Find most active agent
    let maxInteractions = 0
    let mostActive = null
    Object.entries(commPatterns.interaction_frequency).forEach(([agent, count]) => {
      if (count > maxInteractions) {
        maxInteractions = count
        mostActive = agent
      }
    })
    commPatterns.patterns.most_active_agent = mostActive

    // Find most collaborative pair
    let maxCollaboration = 0
    let mostCollaborativePair = null
    Object.entries(commPatterns.communication_matrix).forEach(([from, targets]) => {
      Object.entries(targets).forEach(([to, count]) => {
        if (count > maxCollaboration) {
          maxCollaboration = count
          mostCollaborativePair = `${from} ↔ ${to}`
        }
      })
    })
    commPatterns.patterns.most_collaborative_pair = mostCollaborativePair

    // Find isolated agents
    commPatterns.patterns.isolated_agents = Object.keys(commPatterns.interaction_frequency).filter(
      agent => commPatterns.interaction_frequency[agent] === 0,
    )

    // Ensure output directory exists
    const outputDir = path.dirname(outputPath)
    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true })
    }

    // Write analysis
    fs.writeFileSync(outputPath, JSON.stringify(commPatterns, null, 2))
    console.log('✅ Communication pattern analysis completed')
    console.log(`💾 Analysis saved to ${outputPath}`)

    // Display summary
    console.log('\n📊 Communication Analysis Summary:')
    console.log(`   Total Agents: ${commPatterns.total_agents}`)
    console.log(`   Most Active Agent: ${commPatterns.patterns.most_active_agent || 'None'}`)
    console.log(
      `   Most Collaborative Pair: ${commPatterns.patterns.most_collaborative_pair || 'None'}`,
    )
    console.log(`   Isolated Agents: ${commPatterns.patterns.isolated_agents.length}`)
    console.log(`   Network Links: ${commPatterns.collaboration_network.links.length}`)
  } catch (error) {
    console.error('❌ Communication pattern analysis failed:', error.message)
    process.exit(1)
  }
}

analyzeCommunicationPatterns()
