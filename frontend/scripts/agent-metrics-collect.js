#!/usr/bin/env node

const fs = require('fs')
const path = require('path')

async function collectAgentMetrics() {
  const agents = (process.argv[2] || 'all').split(',')
  const interval = parseInt(process.argv[3]) || 5 // seconds
  const registryUrl = process.argv[4] || 'http://localhost:8000'

  console.log(`📊 Collecting metrics for agents: ${agents.join(', ')} (interval: ${interval}s)`)

  const metrics = {
    timestamp: new Date().toISOString(),
    collection_interval: interval,
    agents: {},
    system: {},
    summary: {
      total_agents: 0,
      active_agents: 0,
      average_response_time: 0,
      total_throughput: 0,
    },
  }

  async function collectMetrics() {
    try {
      // Collect from /metrics endpoint
      const metricsUrl = `${registryUrl}/metrics`
      const metricsResponse = await fetch(metricsUrl)

      if (metricsResponse.ok) {
        const metricsData = await metricsResponse.json()
        metrics.system = metricsData.current_metrics || {}
      }

      // Collect from /api/agents
      const agentsUrl = `${registryUrl}/api/agents`
      const agentsResponse = await fetch(agentsUrl)

      if (agentsResponse.ok) {
        const agentsData = await agentsResponse.json()

        if (agentsData.agents && Array.isArray(agentsData.agents)) {
          metrics.summary.total_agents = agentsData.agents.length
          metrics.summary.active_agents = agentsData.agents.filter(a =>
            ['active', 'running', 'idle'].includes((a.state || '').toLowerCase()),
          ).length

          agentsData.agents.forEach(agent => {
            if (agents.includes('all') || agents.includes(agent.id)) {
              metrics.agents[agent.id] = {
                id: agent.id,
                type: agent.type || 'unknown',
                state: agent.state || 'unknown',
                performance_score: agent.performance_score || 0,
                tasks_completed: agent.tasks_completed || 0,
                response_time_ms: agent.response_time_ms || 0,
                last_active: agent.last_active,
                resource_usage: agent.resource_usage || {},
              }
            }
          })
        }
      }

      // Calculate summary metrics
      const agentList = Object.values(metrics.agents)
      if (agentList.length > 0) {
        metrics.summary.average_response_time =
          agentList.reduce((sum, agent) => sum + (agent.response_time_ms || 0), 0) /
          agentList.length
        metrics.summary.total_throughput = agentList.reduce(
          (sum, agent) => sum + (agent.tasks_completed || 0),
          0,
        )
      }

      console.log(
        `✅ Metrics collected - ${metrics.summary.active_agents}/${metrics.summary.total_agents} active agents`,
      )
    } catch (error) {
      console.error(`❌ Metrics collection failed: ${error.message}`)
    }
  }

  // Initial collection
  await collectMetrics()

  // Set up interval collection
  const intervalId = setInterval(collectMetrics, interval * 1000)

  // Handle graceful shutdown
  process.on('SIGINT', () => {
    console.log('🛑 Metrics collection stopped')

    // Save final metrics
    const outputPath = path.join(process.cwd(), '..', 'monitoring', 'performance-metrics.json')
    const outputDir = path.dirname(outputPath)
    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true })
    }

    fs.writeFileSync(outputPath, JSON.stringify(metrics, null, 2))
    console.log(`💾 Final metrics saved to ${outputPath}`)

    clearInterval(intervalId)
    process.exit(0)
  })

  console.log('✅ Performance metrics collection started - Press Ctrl+C to stop and save')
}

collectAgentMetrics().catch(error => {
  console.error('❌ Failed to start metrics collection:', error)
  process.exit(1)
})
