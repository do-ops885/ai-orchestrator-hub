#!/usr/bin/env node

async function monitorResourceUsage() {
  const resources = process.argv.slice(2)
  const registryUrl = resources.find(arg => arg.startsWith('http')) || 'http://localhost:8000'
  const monitorCpu = resources.includes('--cpu') || resources.length === 0
  const monitorMemory = resources.includes('--memory') || resources.length === 0
  const monitorNetwork = resources.includes('--network') || resources.length === 0

  console.log('🔍 Monitoring resource usage...')
  if (monitorCpu) {
    console.log('   📊 CPU usage')
  }
  if (monitorMemory) {
    console.log('   🧠 Memory usage')
  }
  if (monitorNetwork) {
    console.log('   🌐 Network usage')
  }

  try {
    // Get system resource information
    const resourcesUrl = `${registryUrl}/api/resources`
    const response = await fetch(resourcesUrl)

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    }

    const data = await response.json()

    console.log('\n📊 Resource Usage Report:')
    console.log('═'.repeat(50))

    if (data.system_resources) {
      const sys = data.system_resources

      if (monitorCpu && sys.cpu_usage !== undefined) {
        console.log(`CPU Usage: ${sys.cpu_usage.toFixed(2)}%`)
        console.log(`   Cores: ${sys.cpu_cores || 'Unknown'}`)
        console.log(`   Load Average: ${sys.load_average || 'Unknown'}`)
      }

      if (monitorMemory && sys.memory_usage !== undefined) {
        console.log(`Memory Usage: ${sys.memory_usage.toFixed(2)}%`)
        console.log(`   Used: ${sys.used_memory_mb || 0} MB`)
        console.log(`   Available: ${sys.available_memory_mb || 0} MB`)
        console.log(`   Total: ${sys.total_memory_mb || 0} MB`)
      }

      if (monitorNetwork && sys.network_usage) {
        console.log('Network Usage:')
        console.log(`   RX: ${sys.network_usage.rx_bytes || 0} bytes`)
        console.log(`   TX: ${sys.network_usage.tx_bytes || 0} bytes`)
        console.log(`   Connections: ${sys.network_usage.active_connections || 0}`)
      }
    }

    // Agent-specific resource usage
    if (data.agent_resources && Array.isArray(data.agent_resources)) {
      console.log('\n🤖 Agent Resource Usage:')
      console.log('─'.repeat(30))

      data.agent_resources.forEach(agent => {
        console.log(`Agent ${agent.id}:`)
        if (monitorCpu && agent.cpu_usage !== undefined) {
          console.log(`   CPU: ${agent.cpu_usage.toFixed(2)}%`)
        }
        if (monitorMemory && agent.memory_usage !== undefined) {
          console.log(`   Memory: ${agent.memory_usage.toFixed(2)}% (${agent.memory_mb || 0} MB)`)
        }
        if (monitorNetwork && agent.network_usage) {
          console.log(`   Network: ${agent.network_usage.connections || 0} connections`)
        }
      })
    }

    // Check for resource alerts
    const alerts = []

    if (data.system_resources) {
      const sys = data.system_resources
      if (monitorCpu && sys.cpu_usage > 90) {
        alerts.push(`🚨 HIGH CPU USAGE: ${sys.cpu_usage.toFixed(2)}%`)
      }
      if (monitorMemory && sys.memory_usage > 95) {
        alerts.push(`🚨 HIGH MEMORY USAGE: ${sys.memory_usage.toFixed(2)}%`)
      }
    }

    if (alerts.length > 0) {
      console.log('\n🚨 ALERTS:')
      alerts.forEach(alert => console.log(`   ${alert}`))
    } else {
      console.log('\n✅ All resources within normal limits')
    }
  } catch (error) {
    console.error('❌ Resource monitoring failed:', error.message)
    process.exit(1)
  }
}

monitorResourceUsage()
