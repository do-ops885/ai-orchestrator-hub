#!/usr/bin/env node

/* eslint-disable no-console */

const fs = require('fs')
const path = require('path')

async function trackAgentLifecycle() {
  const events = (process.argv[2] || 'start,stop,restart').split(',')
  const outputPath = process.argv[3] || path.join(process.cwd(), '..', 'monitoring', 'lifecycle.log')
  const registryUrl = process.argv[4] || 'http://localhost:8000'

  console.log(`🔄 Tracking agent lifecycle events: ${events.join(', ')}`)
  console.log(`📝 Output: ${outputPath}`)

  // Ensure output directory exists
  const outputDir = path.dirname(outputPath)
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true })
  }

  const logStream = fs.createWriteStream(outputPath, { flags: 'a' })

  function log(message) {
    const timestamp = new Date().toISOString()
    const logMessage = `[${timestamp}] ${message}`
    console.log(logMessage)
    logStream.write(`${logMessage}\n`)
  }

  let previousAgents = new Set()

  async function checkLifecycle() {
    try {
      const agentsUrl = `${registryUrl}/api/agents`
      const response = await fetch(agentsUrl)

      if (!response.ok) {
        log(`❌ Failed to fetch agents: ${response.status} ${response.statusText}`)
        return
      }

      const data = await response.json()
      const currentAgents = new Set()

      if (data.agents && Array.isArray(data.agents)) {
        data.agents.forEach(agent => {
          currentAgents.add(agent.id)

          // Check for new agents (start event)
          if (!previousAgents.has(agent.id) && events.includes('start')) {
            log(`🚀 Agent started: ${agent.id} (${agent.type || 'unknown'})`)
          }
        })

        // Check for stopped agents
        if (events.includes('stop')) {
          previousAgents.forEach(agentId => {
            if (!currentAgents.has(agentId)) {
              log(`🛑 Agent stopped: ${agentId}`)
            }
          })
        }

        // Check for state changes (potential restart)
        if (events.includes('restart')) {
          // This would require tracking agent states over time
          // For now, we'll track basic lifecycle
        }
      }

      previousAgents = currentAgents

    } catch (error) {
      log(`❌ Lifecycle check failed: ${error.message}`)
    }
  }

  // Initial check
  await checkLifecycle()

  // Set up periodic checking
  const intervalId = setInterval(checkLifecycle, 10000) // Check every 10 seconds

  log('✅ Lifecycle tracking started - Press Ctrl+C to stop')

  // Handle graceful shutdown
  process.on('SIGINT', () => {
    log('🛑 Lifecycle tracking stopped')
    logStream.end()
    clearInterval(intervalId)
    process.exit(0)
  })
}

trackAgentLifecycle().catch(error => {
  console.error('❌ Failed to start lifecycle tracking:', error)
  process.exit(1)
})
