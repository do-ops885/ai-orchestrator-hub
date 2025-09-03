#!/usr/bin/env node

/* eslint-disable no-console */

async function checkAgentHealth() {
  const timeout = parseInt(process.argv[2]) || 30 // seconds
  const registryUrl = process.argv[3] || 'http://localhost:8000'

  console.log(`🔍 Checking agent health with ${timeout}s timeout...`)

  const startTime = Date.now()

  try {
    // Check main API endpoints
    const endpoints = [
      { url: `${registryUrl}/health`, name: 'Health Check' },
      { url: `${registryUrl}/api/agents`, name: 'Agents API' },
      { url: `${registryUrl}/api/tasks`, name: 'Tasks API' },
      { url: `${registryUrl}/metrics`, name: 'Metrics API' },
    ]

    const results = []

    for (const endpoint of endpoints) {
      const endpointStart = Date.now()

      try {
        const controller = new AbortController()
        const timeoutId = setTimeout(() => controller.abort(), timeout * 1000)

        const response = await fetch(endpoint.url, {
          signal: controller.signal,
          headers: {
            'User-Agent': 'Agent-Monitor/1.0',
          },
        })

        clearTimeout(timeoutId)

        const responseTime = Date.now() - endpointStart
        const status = response.ok ? '✅ Healthy' : '❌ Unhealthy'

        results.push({
          endpoint: endpoint.name,
          url: endpoint.url,
          status,
          response_time_ms: responseTime,
          http_status: response.status,
          error: null,
        })

        console.log(`${status} ${endpoint.name}: ${responseTime}ms (${response.status})`)

      } catch (error) {
        const responseTime = Date.now() - endpointStart
        results.push({
          endpoint: endpoint.name,
          url: endpoint.url,
          status: '❌ Failed',
          response_time_ms: responseTime,
          http_status: null,
          error: error.message,
        })

        console.log(`❌ ${endpoint.name}: Failed (${error.message})`)
      }
    }

    // Overall assessment
    const totalTime = Date.now() - startTime
    const healthyCount = results.filter(r => r.status.includes('Healthy')).length
    const overallStatus = healthyCount === endpoints.length ? '✅ All Healthy' : `⚠️ ${healthyCount}/${endpoints.length} Healthy`

    console.log(`\n📊 Overall Status: ${overallStatus}`)
    console.log(`⏱️  Total Check Time: ${totalTime}ms`)

    // Detailed results
    console.log('\n📋 Detailed Results:')
    results.forEach(result => {
      console.log(`   ${result.endpoint}: ${result.status} - ${result.response_time_ms}ms`)
      if (result.error) {
        console.log(`      Error: ${result.error}`)
      }
    })

    // Exit with appropriate code
    if (healthyCount < endpoints.length) {
      process.exit(1)
    }

  } catch (error) {
    console.error('❌ Health check failed:', error.message)
    process.exit(1)
  }
}

checkAgentHealth()
