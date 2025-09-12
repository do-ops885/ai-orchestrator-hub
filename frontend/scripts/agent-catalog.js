#!/usr/bin/env node

const fs = require('fs')
const path = require('path')

async function catalogAgents() {
  const outputPath =
    process.argv[2] || path.join(process.cwd(), '..', 'monitoring', 'agent-catalog.json')

  console.log('📚 Creating agent catalog...')

  try {
    // First discover agents
    const registryUrl = process.argv[3] || 'http://localhost:8000'
    const apiUrl = `${registryUrl}/api/agents`

    const response = await fetch(apiUrl)
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    }

    const data = await response.json()

    // Create catalog structure
    const catalog = {
      timestamp: new Date().toISOString(),
      total_agents: 0,
      agent_types: {},
      agent_states: {},
      agents: [],
    }

    if (data.agents && Array.isArray(data.agents)) {
      catalog.total_agents = data.agents.length

      data.agents.forEach(agent => {
        // Categorize by type
        const agentType = agent.type || 'unknown'
        if (!catalog.agent_types[agentType]) {
          catalog.agent_types[agentType] = 0
        }
        catalog.agent_types[agentType]++

        // Categorize by state
        const agentState = agent.state || 'unknown'
        if (!catalog.agent_states[agentState]) {
          catalog.agent_states[agentState] = 0
        }
        catalog.agent_states[agentState]++

        // Add to agents list
        catalog.agents.push({
          id: agent.id,
          type: agentType,
          state: agentState,
          created_at: agent.created_at,
          last_active: agent.last_active,
          capabilities: agent.capabilities || [],
        })
      })
    }

    // Ensure output directory exists
    const outputDir = path.dirname(outputPath)
    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true })
    }

    // Write catalog
    fs.writeFileSync(outputPath, JSON.stringify(catalog, null, 2))
    console.log(`✅ Agent catalog created with ${catalog.total_agents} agents`)
    console.log(`💾 Catalog saved to ${outputPath}`)

    // Display summary
    console.log('\n📊 Catalog Summary:')
    console.log(`   Total Agents: ${catalog.total_agents}`)
    console.log('   Agent Types:', JSON.stringify(catalog.agent_types, null, 2))
    console.log('   Agent States:', JSON.stringify(catalog.agent_states, null, 2))
  } catch (error) {
    console.error('❌ Agent catalog creation failed:', error.message)
    process.exit(1)
  }
}

catalogAgents()
