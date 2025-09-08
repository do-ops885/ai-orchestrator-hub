#!/usr/bin/env node

/* eslint-disable no-console */

async function monitorTaskPerformance() {
  const registryUrl = process.argv[2] || 'http://localhost:8000'

  try {
    // Get tasks data
    const tasksUrl = `${registryUrl}/api/tasks`
    const tasksResponse = await fetch(tasksUrl)

    if (!tasksResponse.ok) {
      throw new Error(`HTTP ${tasksResponse.status}: ${tasksResponse.statusText}`)
    }

    const tasksData = await tasksResponse.json()

    // Get agents data for correlation
    const agentsUrl = `${registryUrl}/api/agents`
    const agentsResponse = await fetch(agentsUrl)
    const agentsData = agentsResponse.ok ? await agentsResponse.json() : { agents: [] }

    // Analyze task metrics
    const taskMetrics = {
      total_tasks: 0,
      completed_tasks: 0,
      failed_tasks: 0,
      pending_tasks: 0,
      average_completion_time: 0,
      task_success_rate: 0,
      tasks_by_status: {},
      tasks_by_agent: {},
      recent_performance: [],
    }

    if (tasksData.tasks && Array.isArray(tasksData.tasks)) {
      taskMetrics.total_tasks = tasksData.tasks.length

      let totalCompletionTime = 0
      let completedCount = 0

      tasksData.tasks.forEach(task => {
        // Count by status
        const status = task.status || 'unknown'
        taskMetrics.tasks_by_status[status] = (taskMetrics.tasks_by_status[status] || 0) + 1

        // Count by agent
        const agent = task.assigned_agent || 'unassigned'
        taskMetrics.tasks_by_agent[agent] = (taskMetrics.tasks_by_agent[agent] || 0) + 1

        // Calculate completion metrics
        if (status === 'completed') {
          taskMetrics.completed_tasks++
          if (task.created_at && task.completed_at) {
            const created = new Date(task.created_at)
            const completed = new Date(task.completed_at)
            const completionTime = completed - created
            totalCompletionTime += completionTime
            completedCount++
          }
        } else if (status === 'failed') {
          taskMetrics.failed_tasks++
        } else if (['pending', 'queued', 'running'].includes(status)) {
          taskMetrics.pending_tasks++
        }

        // Track recent tasks (last 24 hours)
        if (task.created_at) {
          const created = new Date(task.created_at)
          const oneDayAgo = new Date(Date.now() - 24 * 60 * 60 * 1000)
          if (created > oneDayAgo) {
            taskMetrics.recent_performance.push({
              id: task.id,
              status,
              created_at: task.created_at,
              completion_time_ms: task.completed_at ? new Date(task.completed_at) - created : null,
            })
          }
        }
      })

      // Calculate averages
      if (completedCount > 0) {
        taskMetrics.average_completion_time = totalCompletionTime / completedCount
      }

      if (taskMetrics.total_tasks > 0) {
        taskMetrics.task_success_rate =
          (taskMetrics.completed_tasks / taskMetrics.total_tasks) * 100
      }
    }

    // Display results
    console.log('\n📊 Task Performance Metrics:')
    console.log('═'.repeat(50))
    console.log(`Total Tasks: ${taskMetrics.total_tasks}`)
    console.log(`Completed: ${taskMetrics.completed_tasks}`)
    console.log(`Failed: ${taskMetrics.failed_tasks}`)
    console.log(`Pending/Running: ${taskMetrics.pending_tasks}`)
    console.log(`Success Rate: ${taskMetrics.task_success_rate.toFixed(2)}%`)

    if (taskMetrics.average_completion_time > 0) {
      console.log(
        `Average Completion Time: ${(taskMetrics.average_completion_time / 1000).toFixed(2)}s`,
      )
    }

    console.log('\n📋 Tasks by Status:')
    Object.entries(taskMetrics.tasks_by_status).forEach(([status, count]) => {
      console.log(`   ${status}: ${count}`)
    })

    console.log('\n🤖 Tasks by Agent:')
    Object.entries(taskMetrics.tasks_by_agent).forEach(([agent, count]) => {
      console.log(`   ${agent}: ${count}`)
    })

    // Agent performance correlation
    if (agentsData.agents && Array.isArray(agentsData.agents)) {
      console.log('\n⚡ Agent Performance Correlation:')
      agentsData.agents.forEach(agent => {
        const tasksAssigned = taskMetrics.tasks_by_agent[agent.id] || 0
        const _tasksCompleted = taskMetrics.tasks_by_status['completed'] || 0
        const agentSuccessRate =
          tasksAssigned > 0 ? (taskMetrics.completed_tasks / taskMetrics.total_tasks) * 100 : 0

        console.log(
          `   ${agent.id}: ${tasksAssigned} tasks, ${agentSuccessRate.toFixed(2)}% success rate`,
        )
      })
    }

    // Performance alerts
    const alerts = []
    if (taskMetrics.task_success_rate < 80) {
      alerts.push(`⚠️ LOW SUCCESS RATE: ${taskMetrics.task_success_rate.toFixed(2)}%`)
    }
    if (taskMetrics.failed_tasks > taskMetrics.total_tasks * 0.1) {
      alerts.push(`🚨 HIGH FAILURE RATE: ${taskMetrics.failed_tasks} failed tasks`)
    }

    if (alerts.length > 0) {
      console.log('\n🚨 PERFORMANCE ALERTS:')
      alerts.forEach(alert => console.log(`   ${alert}`))
    } else {
      console.log('\n✅ Task performance within acceptable limits')
    }
  } catch (error) {
    console.error('❌ Task performance monitoring failed:', error.message)
    process.exit(1)
  }
}

monitorTaskPerformance()
