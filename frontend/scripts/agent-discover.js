#!/usr/bin/env node

async function discoverAgents() {
  const registryUrl = process.argv[2] || 'http://localhost:8000'
  const apiUrl = `${registryUrl}/api/agents`

  console.log(`🔍 Discovering agents from ${apiUrl}...`)

  try {
    const response = await fetch(apiUrl)
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    }

    const data = await response.json()
    console.log('✅ Agent discovery completed')

    // Process and display agent information
    if (data.agents && Array.isArray(data.agents)) {
      console.log(`📊 Found ${data.agents.length} agents:`)
      data.agents.forEach((agent, index) => {
        console.log(`${index + 1}. ${agent.id || 'Unknown'} - ${agent.state || 'Unknown state'}`)
      })
    } else {
      console.log('📊 Agent data structure:', JSON.stringify(data, null, 2))
    }

    // Save to monitoring directory
    const fs = require('fs')
    const path = require('path')
    const monitoringDir = path.join(process.cwd(), '..', 'monitoring')
    const outputFile = path.join(monitoringDir, 'agent-discovery.json')

    fs.writeFileSync(outputFile, JSON.stringify(data, null, 2))
    console.log(`💾 Discovery results saved to ${outputFile}`)
  } catch (error) {
    console.error('❌ Agent discovery failed:', error.message)
    process.exit(1)
  }
}

discoverAgents()
