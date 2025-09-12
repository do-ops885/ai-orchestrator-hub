#!/usr/bin/env node

const fs = require('fs')
const path = require('path')

async function analyzeDecisionPatterns() {
  const outputPath =
    process.argv[2] || path.join(process.cwd(), '..', 'monitoring', 'decision-patterns.json')
  const registryUrl = process.argv[3] || 'http://localhost:8000'

  console.log('🧠 Analyzing agent decision patterns...')

  try {
    // Get tasks and agents data
    const [tasksResponse, agentsResponse] = await Promise.all([
      fetch(`${registryUrl}/api/tasks`),
      fetch(`${registryUrl}/api/agents`),
    ])

    if (!tasksResponse.ok || !agentsResponse.ok) {
      throw new Error('Failed to fetch data from API')
    }

    const tasksData = await tasksResponse.json()
    const _agentsData = await agentsResponse.json()

    // Analyze decision patterns
    const decisionPatterns = {
      timestamp: new Date().toISOString(),
      decision_metrics: {
        total_decisions: 0,
        successful_decisions: 0,
        failed_decisions: 0,
        decision_success_rate: 0,
        average_decision_time: 0,
      },
      decision_types: {},
      agent_decision_profiles: {},
      decision_trends: {
        by_hour: {},
        by_day: {},
        by_type: {},
      },
      patterns: {
        peak_decision_hours: [],
        most_decisive_agent: null,
        fastest_decision_maker: null,
        most_reliable_decision_maker: null,
      },
    }

    // Process tasks as decision points
    if (tasksData.tasks && Array.isArray(tasksData.tasks)) {
      decisionPatterns.decision_metrics.total_decisions = tasksData.tasks.length

      let totalDecisionTime = 0
      let decisionCount = 0

      tasksData.tasks.forEach(task => {
        // Categorize decision types
        const decisionType = task.type || 'task_execution'
        decisionPatterns.decision_types[decisionType] =
          (decisionPatterns.decision_types[decisionType] || 0) + 1

        // Track agent decision profiles
        const agent = task.assigned_agent || 'system'
        if (!decisionPatterns.agent_decision_profiles[agent]) {
          decisionPatterns.agent_decision_profiles[agent] = {
            total_decisions: 0,
            successful: 0,
            failed: 0,
            average_time: 0,
            total_time: 0,
          }
        }

        decisionPatterns.agent_decision_profiles[agent].total_decisions++

        // Analyze decision outcomes
        if (task.status === 'completed') {
          decisionPatterns.decision_metrics.successful_decisions++
          decisionPatterns.agent_decision_profiles[agent].successful++
        } else if (task.status === 'failed') {
          decisionPatterns.decision_metrics.failed_decisions++
          decisionPatterns.agent_decision_profiles[agent].failed++
        }

        // Calculate decision time
        if (task.created_at && task.completed_at) {
          const created = new Date(task.created_at)
          const completed = new Date(task.completed_at)
          const decisionTime = completed - created

          totalDecisionTime += decisionTime
          decisionCount++
          decisionPatterns.agent_decision_profiles[agent].total_time += decisionTime

          // Track decision trends by hour
          const hour = created.getHours()
          decisionPatterns.decision_trends.by_hour[hour] =
            (decisionPatterns.decision_trends.by_hour[hour] || 0) + 1
        }

        // Track by day
        if (task.created_at) {
          const [day] = new Date(task.created_at).toISOString().split('T')
          decisionPatterns.decision_trends.by_day[day] =
            (decisionPatterns.decision_trends.by_day[day] || 0) + 1
        }
      })

      // Calculate averages
      if (decisionCount > 0) {
        decisionPatterns.decision_metrics.average_decision_time = totalDecisionTime / decisionCount
      }

      if (decisionPatterns.decision_metrics.total_decisions > 0) {
        decisionPatterns.decision_metrics.decision_success_rate =
          (decisionPatterns.decision_metrics.successful_decisions /
            decisionPatterns.decision_metrics.total_decisions) *
          100
      }

      // Calculate agent averages
      Object.values(decisionPatterns.agent_decision_profiles).forEach(profile => {
        if (profile.total_decisions > 0) {
          profile.average_time = profile.total_time / profile.total_decisions
        }
      })
    }

    // Find patterns
    const agents = Object.keys(decisionPatterns.agent_decision_profiles)
    if (agents.length > 0) {
      // Most decisive agent
      decisionPatterns.patterns.most_decisive_agent = agents.reduce((max, agent) =>
        decisionPatterns.agent_decision_profiles[agent].total_decisions >
        decisionPatterns.agent_decision_profiles[max].total_decisions
          ? agent
          : max,
      )

      // Fastest decision maker
      decisionPatterns.patterns.fastest_decision_maker = agents.reduce((min, agent) => {
        const currentAvg = decisionPatterns.agent_decision_profiles[agent].average_time || Infinity
        const minAvg = decisionPatterns.agent_decision_profiles[min].average_time || Infinity
        return currentAvg < minAvg ? agent : min
      })

      // Most reliable decision maker
      decisionPatterns.patterns.most_reliable_decision_maker = agents.reduce((max, agent) => {
        const currentRate =
          decisionPatterns.agent_decision_profiles[agent].total_decisions > 0
            ? (decisionPatterns.agent_decision_profiles[agent].successful /
                decisionPatterns.agent_decision_profiles[agent].total_decisions) *
              100
            : 0
        const maxRate =
          decisionPatterns.agent_decision_profiles[max].total_decisions > 0
            ? (decisionPatterns.agent_decision_profiles[max].successful /
                decisionPatterns.agent_decision_profiles[max].total_decisions) *
              100
            : 0
        return currentRate > maxRate ? agent : max
      })
    }

    // Find peak decision hours
    const hourEntries = Object.entries(decisionPatterns.decision_trends.by_hour).sort(
      ([, a], [, b]) => b - a,
    )
    decisionPatterns.patterns.peak_decision_hours = hourEntries
      .slice(0, 3)
      .map(([hour]) => parseInt(hour))

    // Ensure output directory exists
    const outputDir = path.dirname(outputPath)
    if (!fs.existsSync(outputDir)) {
      fs.mkdirSync(outputDir, { recursive: true })
    }

    // Write analysis
    fs.writeFileSync(outputPath, JSON.stringify(decisionPatterns, null, 2))
    console.log('✅ Decision pattern analysis completed')
    console.log(`💾 Analysis saved to ${outputPath}`)

    // Display summary
    console.log('\n📊 Decision Analysis Summary:')
    console.log(`   Total Decisions: ${decisionPatterns.decision_metrics.total_decisions}`)
    console.log(
      `   Success Rate: ${decisionPatterns.decision_metrics.decision_success_rate.toFixed(2)}%`,
    )
    console.log(
      `   Average Decision Time: ${(decisionPatterns.decision_metrics.average_decision_time / 1000).toFixed(2)}s`,
    )
    console.log(
      `   Most Decisive Agent: ${decisionPatterns.patterns.most_decisive_agent || 'None'}`,
    )
    console.log(
      `   Fastest Decision Maker: ${decisionPatterns.patterns.fastest_decision_maker || 'None'}`,
    )
    console.log(
      `   Most Reliable: ${decisionPatterns.patterns.most_reliable_decision_maker || 'None'}`,
    )
    console.log(`   Peak Hours: ${decisionPatterns.patterns.peak_decision_hours.join(', ')}`)
  } catch (error) {
    console.error('❌ Decision pattern analysis failed:', error.message)
    process.exit(1)
  }
}

analyzeDecisionPatterns()
