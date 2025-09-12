#!/usr/bin/env node

const fs = require('fs')
const path = require('path')

async function analyzeAdaptationBehavior() {
  const outputPath =
    process.argv[2] || path.join(process.cwd(), '..', 'monitoring', 'adaptation-metrics.json')
  const registryUrl = process.argv[3] || 'http://localhost:8000'

  try {
    // Get agents and system status data
    const [agentsResponse, statusResponse] = await Promise.all([
      fetch(`${registryUrl}/api/agents`),
      fetch(`${registryUrl}/api/hive/status`),
    ])

    if (!agentsResponse.ok || !statusResponse.ok) {
      throw new Error('Failed to fetch data from API')
    }

    const agentsData = await agentsResponse.json()
    const statusData = await statusResponse.json()

    // Analyze adaptation metrics
    const adaptationMetrics = {
      timestamp: new Date().toISOString(),
      system_adaptation: {
        learning_progress: 0,
        swarm_cohesion: 0,
        adaptation_events: 0,
        performance_trend: 'stable',
      },
      agent_adaptation: {},
      adaptation_patterns: {
        learning_agents: [],
        adaptive_agents: [],
        stagnant_agents: [],
        improvement_rate: 0,
      },
      metrics: {
        total_agents: 0,
        adapting_agents: 0,
        average_adaptation_score: 0,
        adaptation_success_rate: 0,
      },
    }

    // Extract system-level adaptation metrics
    if (statusData.metrics) {
      adaptationMetrics.system_adaptation.learning_progress =
        statusData.metrics.learning_progress || 0
      adaptationMetrics.system_adaptation.swarm_cohesion = statusData.metrics.swarm_cohesion || 0
      adaptationMetrics.system_adaptation.adaptation_events =
        statusData.metrics.adaptation_events || 0
    }

    // Analyze individual agent adaptation
    if (agentsData.agents && Array.isArray(agentsData.agents)) {
      adaptationMetrics.metrics.total_agents = agentsData.agents.length

      agentsData.agents.forEach(agent => {
        const agentId = agent.id
        adaptationMetrics.agent_adaptation[agentId] = {
          id: agentId,
          type: agent.type || 'unknown',
          adaptation_score: agent.adaptation_score || 0,
          learning_sessions: agent.learning_sessions || 0,
          performance_trend: agent.performance_trend || 'stable',
          last_adaptation: agent.last_adaptation,
          adaptation_events: agent.adaptation_events || [],
          improvement_rate: agent.improvement_rate || 0,
        }

        // Categorize agents by adaptation behavior
        const adaptationScore = agent.adaptation_score || 0
        const learningSessions = agent.learning_sessions || 0

        if (adaptationScore > 0.7) {
          adaptationMetrics.adaptation_patterns.adaptive_agents.push(agentId)
        }

        if (learningSessions > 0) {
          adaptationMetrics.adaptation_patterns.learning_agents.push(agentId)
        }

        if (adaptationScore < 0.3 && learningSessions === 0) {
          adaptationMetrics.adaptation_patterns.stagnant_agents.push(agentId)
        }

        // Count adapting agents
        if (adaptationScore > 0 || learningSessions > 0) {
          adaptationMetrics.metrics.adapting_agents++
        }
      })

      // Calculate averages
      const agentList = Object.values(adaptationMetrics.agent_adaptation)
      if (agentList.length > 0) {
        adaptationMetrics.metrics.average_adaptation_score =
          agentList.reduce((sum, agent) => sum + (agent.adaptation_score || 0), 0) /
          agentList.length

        const totalImprovement = agentList.reduce(
          (sum, agent) => sum + (agent.improvement_rate || 0),
          0,
        )
        adaptationMetrics.adaptation_patterns.improvement_rate = totalImprovement / agentList.length
      }

      if (adaptationMetrics.metrics.total_agents > 0) {
        adaptationMetrics.metrics.adaptation_success_rate =
          (adaptationMetrics.metrics.adapting_agents / adaptationMetrics.metrics.total_agents) * 100
      }
    }

    // Determine system performance trend
    const learningProgress = adaptationMetrics.system_adaptation.learning_progress
    if (learningProgress > 0.7) {
      adaptationMetrics.system_adaptation.performance_trend = 'improving'
    } else if (learningProgress < 0.3) {
      adaptationMetrics.system_adaptation.performance_trend = 'declining'
    }

    // Ensure output directory exists
    const outputDir = path.dirname(outputPath)
    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true })
    }

    // Write analysis
    fs.writeFileSync(outputPath, JSON.stringify(adaptationMetrics, null, 2))
    console.log('✅ Adaptation behavior analysis completed')
    console.log(`💾 Analysis saved to ${outputPath}`)

    // Display summary
    console.log('\n📊 Adaptation Analysis Summary:')
    console.log(`   Total Agents: ${adaptationMetrics.metrics.total_agents}`)
    console.log(`   Adapting Agents: ${adaptationMetrics.metrics.adapting_agents}`)
    console.log(
      `   Adaptation Success Rate: ${adaptationMetrics.metrics.adaptation_success_rate.toFixed(2)}%`,
    )
    console.log(
      `   Average Adaptation Score: ${adaptationMetrics.metrics.average_adaptation_score.toFixed(2)}`,
    )
    console.log(
      `   System Learning Progress: ${adaptationMetrics.system_adaptation.learning_progress.toFixed(2)}`,
    )
    console.log(
      `   Swarm Cohesion: ${adaptationMetrics.system_adaptation.swarm_cohesion.toFixed(2)}`,
    )
    console.log(`   Performance Trend: ${adaptationMetrics.system_adaptation.performance_trend}`)
    console.log(
      `   Adaptive Agents: ${adaptationMetrics.adaptation_patterns.adaptive_agents.length}`,
    )
    console.log(
      `   Learning Agents: ${adaptationMetrics.adaptation_patterns.learning_agents.length}`,
    )
    console.log(
      `   Stagnant Agents: ${adaptationMetrics.adaptation_patterns.stagnant_agents.length}`,
    )
  } catch (error) {
    console.error('❌ Adaptation behavior analysis failed:', error.message)
    process.exit(1)
  }
}

analyzeAdaptationBehavior()
