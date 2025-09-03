#!/usr/bin/env node

/* eslint-disable no-console */

const fs = require('fs')
const path = require('path')

async function monitorAgentHealth() {
  const interval = parseInt(process.argv[2]) || 5 // seconds
  const outputPath = process.argv[3] || path.join(process.cwd(), '..', 'monitoring', 'health.log')
  const registryUrl = process.argv[4] || 'http://localhost:8000'

  console.log(`🏥 Starting agent health monitoring (interval: ${interval}s)`)
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

  async function checkHealth() {
    try {
      // Check agents health
      const agentsUrl = `${registryUrl}/api/agents`
      const agentsResponse = await fetch(agentsUrl)

      if (!agentsResponse.ok) {
        log(`❌ Agents API error: ${agentsResponse.status} ${agentsResponse.statusText}`)
        return
      }

      const agentsData = await agentsResponse.json()

      // Check system health
      const healthUrl = `${registryUrl}/health`
      const healthResponse = await fetch(healthUrl)
      const healthData = healthResponse.ok ? await healthResponse.json() : null

      // Analyze agent health
      let healthyAgents = 0
      let unhealthyAgents = 0
      let totalAgents = 0

      if (agentsData.agents && Array.isArray(agentsData.agents)) {
        totalAgents = agentsData.agents.length

        agentsData.agents.forEach(agent => {
          const state = agent.state || 'unknown'
          if (['active', 'idle', 'running'].includes(state.toLowerCase())) {
            healthyAgents++
          } else if (['failed', 'error', 'stopped'].includes(state.toLowerCase())) {
            unhealthyAgents++
          }
        })
      }

      // Log health status
      log(`📊 Health Check - Total: ${totalAgents}, Healthy: ${healthyAgents}, Unhealthy: ${unhealthyAgents}`)

      if (healthData) {
        const overallStatus = healthData.status
        const responseTime = healthData.response_time_ms
        log(`🏥 System Status: ${overallStatus} (Response: ${responseTime}ms)`)

        // Check component health
        if (healthData.components) {
          Object.entries(healthData.components).forEach(([component, info]) => {
            if (info.status !== 'healthy') {
              log(`⚠️  ${component}: ${info.status}`)
            }
          })
        }
      }

      // Check for alerts
      if (unhealthyAgents > 0) {
        log(`🚨 ALERT: ${unhealthyAgents} unhealthy agents detected`)
      }

    } catch (error) {
      log(`❌ Health check failed: ${error.message}`)
    }
  }

  // Initial check
  await checkHealth()

  // Set up interval monitoring
  const intervalId = setInterval(checkHealth, interval * 1000)

  // Handle graceful shutdown
  process.on('SIGINT', () => {
    log('🛑 Health monitoring stopped')
    logStream.end()
    clearInterval(intervalId)
    process.exit(0)
  })

  log('✅ Health monitoring started - Press Ctrl+C to stop')
}

monitorAgentHealth().catch(error => {
  console.error('❌ Failed to start health monitoring:', error)
  process.exit(1)
})
