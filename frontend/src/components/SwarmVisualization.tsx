'use client'

import { useEffect, useRef } from 'react'
import { Agent } from '@/store/hiveStore'

interface SwarmVisualizationProps {
  agents: Agent[];
  swarmCenter: [number, number];
}

export function SwarmVisualization({ agents, swarmCenter }: SwarmVisualizationProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)

  useEffect(() => {
    const canvas = canvasRef.current
    if (canvas === null) {return}

    const ctx = canvas.getContext('2d')
    if (ctx === null) {return}

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height)

    // Set up coordinate system (center canvas)
    const centerX = canvas.width / 2
    const centerY = canvas.height / 2
    const scale = 2 // Scale factor for positioning

    // Draw grid
    ctx.strokeStyle = '#f0f0f0'
    ctx.lineWidth = 1
    for (let i = -200; i <= 200; i += 50) {
      // Vertical lines
      ctx.beginPath()
      ctx.moveTo(centerX + i * scale, 0)
      ctx.lineTo(centerX + i * scale, canvas.height)
      ctx.stroke()
      
      // Horizontal lines
      ctx.beginPath()
      ctx.moveTo(0, centerY + i * scale)
      ctx.lineTo(canvas.width, centerY + i * scale)
      ctx.stroke()
    }

    // Draw swarm center
    ctx.fillStyle = '#ff6b6b'
    ctx.beginPath()
    ctx.arc(
      centerX + swarmCenter[0] * scale,
      centerY + swarmCenter[1] * scale,
      8,
      0,
      2 * Math.PI,
    )
    ctx.fill()

    // Draw center label
    ctx.fillStyle = '#333'
    ctx.font = '12px sans-serif'
    ctx.fillText(
      'Swarm Center',
      centerX + swarmCenter[0] * scale + 12,
      centerY + swarmCenter[1] * scale + 4,
    )

    // Draw agents
    agents.forEach((agent) => {
      const x = centerX + agent.position[0] * scale
      const y = centerY + agent.position[1] * scale

      // Agent color based on type
      let color = '#4ecdc4' // Default worker color
      switch (agent.type) {
        case 'Coordinator':
          color = '#45b7d1'
          break
        case 'Learner':
          color = '#96ceb4'
          break
        default:
          if (agent.type.startsWith('Specialist')) {
            color = '#feca57'
          }
      }

      // Agent state affects opacity
      let alpha = 1.0
      switch (agent.state) {
        case 'Working':
          alpha = 1.0
          break
        case 'Learning':
          alpha = 0.8
          break
        case 'Idle':
          alpha = 0.6
          break
        case 'Failed':
          alpha = 0.3
          color = '#ff6b6b'
          break
      }

      // Draw agent
      ctx.globalAlpha = alpha
      ctx.fillStyle = color
      ctx.beginPath()
      ctx.arc(x, y, 6, 0, 2 * Math.PI)
      ctx.fill()

      // Draw energy ring
      ctx.strokeStyle = color
      ctx.lineWidth = 2
      ctx.beginPath()
      ctx.arc(x, y, 10, 0, 2 * Math.PI * (agent.energy / 100))
      ctx.stroke()

      // Reset alpha
      ctx.globalAlpha = 1.0

      // Draw agent name on hover (simplified - always show for now)
      if (agents.length <= 10) { // Only show names if not too crowded
        ctx.fillStyle = '#333'
        ctx.font = '10px sans-serif'
        ctx.fillText(agent.name, x + 12, y + 4)
      }
    })

    // Draw legend
    const legendY = 20
    ctx.fillStyle = '#333'
    ctx.font = '12px sans-serif'
    ctx.fillText('Agent Types:', 20, legendY)

    const types = [
      { name: 'Worker', color: '#4ecdc4' },
      { name: 'Coordinator', color: '#45b7d1' },
      { name: 'Learner', color: '#96ceb4' },
      { name: 'Specialist', color: '#feca57' },
    ]

    types.forEach((type, index) => {
      const y = legendY + 20 + index * 20
      ctx.fillStyle = type.color
      ctx.beginPath()
      ctx.arc(30, y, 6, 0, 2 * Math.PI)
      ctx.fill()
      
      ctx.fillStyle = '#333'
      ctx.fillText(type.name, 45, y + 4)
    })

  }, [agents, swarmCenter])

  return (
    <div className="bg-white overflow-hidden shadow rounded-lg">
      <div className="px-4 py-5 sm:p-6">
        <h3 className="text-lg leading-6 font-medium text-gray-900 mb-4">
          Swarm Visualization
        </h3>
        <canvas
          ref={canvasRef}
          width={600}
          height={400}
          className="border border-gray-200 rounded"
        />
        <div className="mt-2 text-sm text-gray-500">
          Red dot: Swarm center • Colored dots: Agents • Ring: Energy level
        </div>
      </div>
    </div>
  )
}