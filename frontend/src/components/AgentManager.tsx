'use client'

import { useState } from 'react'
import { useHiveStore } from '@/store/hiveStore'
import { Plus, User, Brain, Settings, Zap } from 'lucide-react'

export function AgentManager() {
  const { agents, createAgent } = useHiveStore()
  const [showCreateForm, setShowCreateForm] = useState(false)
  const [newAgent, setNewAgent] = useState({
    name: '',
    type: 'Worker',
    capabilities: [{ name: '', proficiency: 0.5, learning_rate: 0.1 }],
  })

  const handleCreateAgent = () => {
    createAgent(newAgent)
    setNewAgent({
      name: '',
      type: 'Worker',
      capabilities: [{ name: '', proficiency: 0.5, learning_rate: 0.1 }],
    })
    setShowCreateForm(false)
  }

  const addCapability = () => {
    setNewAgent({
      ...newAgent,
      capabilities: [...newAgent.capabilities, { name: '', proficiency: 0.5, learning_rate: 0.1 }],
    })
  }

  const updateCapability = (index: number, field: string, value: string | number) => {
    const updated = [...newAgent.capabilities]
    updated[index] = { ...updated[index], [field]: value }
    setNewAgent({ ...newAgent, capabilities: updated })
  }

  const getAgentIcon = (type: string) => {
    switch (type) {
      case 'Coordinator': return <Settings className="w-5 h-5" />
      case 'Learner': return <Brain className="w-5 h-5" />
      default: return <User className="w-5 h-5" />
    }
  }

  const getStateColor = (state: string) => {
    switch (state) {
      case 'Working': return 'bg-green-100 text-green-800'
      case 'Learning': return 'bg-blue-100 text-blue-800'
      case 'Idle': return 'bg-gray-100 text-gray-800'
      case 'Failed': return 'bg-red-100 text-red-800'
      default: return 'bg-gray-100 text-gray-800'
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="text-2xl font-bold text-gray-900">Agent Management</h2>
        <button
          onClick={() => setShowCreateForm(true)}
          className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700"
        >
          <Plus className="w-4 h-4 mr-2" />
          Create Agent
        </button>
      </div>

      {showCreateForm && (
        <div className="bg-white shadow rounded-lg p-6">
          <h3 className="text-lg font-medium text-gray-900 mb-4">Create New Agent</h3>
          
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Name</label>
              <input
                type="text"
                value={newAgent.name}
                onChange={(e) => setNewAgent({ ...newAgent, name: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="Agent name"
              />
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Type</label>
              <select
                value={newAgent.type}
                onChange={(e) => setNewAgent({ ...newAgent, type: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="Worker">Worker</option>
                <option value="Coordinator">Coordinator</option>
                <option value="Learner">Learner</option>
                <option value="specialist:data">Specialist: Data</option>
                <option value="specialist:nlp">Specialist: NLP</option>
                <option value="specialist:coordination">Specialist: Coordination</option>
              </select>
            </div>
          </div>

          <div className="mb-4">
            <div className="flex justify-between items-center mb-2">
              <label className="block text-sm font-medium text-gray-700">Capabilities</label>
              <button
                onClick={addCapability}
                className="text-sm text-blue-600 hover:text-blue-800"
              >
                Add Capability
              </button>
            </div>
            
            {newAgent.capabilities.map((cap, index) => (
              <div key={index} className="grid grid-cols-3 gap-2 mb-2">
                <input
                  type="text"
                  value={cap.name}
                  onChange={(e) => updateCapability(index, 'name', e.target.value)}
                  placeholder="Capability name"
                  className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
                <input
                  type="number"
                  min="0"
                  max="1"
                  step="0.1"
                  value={cap.proficiency}
                  onChange={(e) => updateCapability(index, 'proficiency', parseFloat(e.target.value))}
                  placeholder="Proficiency (0-1)"
                  className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
                <input
                  type="number"
                  min="0"
                  max="1"
                  step="0.01"
                  value={cap.learning_rate}
                  onChange={(e) => updateCapability(index, 'learning_rate', parseFloat(e.target.value))}
                  placeholder="Learning rate"
                  className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
            ))}
          </div>

          <div className="flex justify-end space-x-3">
            <button
              onClick={() => setShowCreateForm(false)}
              className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
            >
              Cancel
            </button>
            <button
              onClick={handleCreateAgent}
              className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700"
            >
              Create Agent
            </button>
          </div>
        </div>
      )}

      <div className="bg-white shadow overflow-hidden sm:rounded-md">
        <ul className="divide-y divide-gray-200">
          {agents.map((agent) => (
            <li key={agent.id} className="px-6 py-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center">
                  <div className="flex-shrink-0">
                    {getAgentIcon(agent.type)}
                  </div>
                  <div className="ml-4">
                    <div className="flex items-center">
                      <div className="text-sm font-medium text-gray-900">{agent.name}</div>
                      <span className={`ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStateColor(agent.state)}`}>
                        {agent.state}
                      </span>
                    </div>
                    <div className="text-sm text-gray-500">{agent.type}</div>
                  </div>
                </div>
                
                <div className="flex items-center space-x-4">
                  <div className="text-right">
                    <div className="text-sm text-gray-900 flex items-center">
                      <Zap className="w-4 h-4 mr-1" />
                      {agent.energy.toFixed(1)}%
                    </div>
                    <div className="text-sm text-gray-500">
                      {agent.experience_count} experiences
                    </div>
                  </div>
                  
                  <div className="text-right">
                    <div className="text-sm text-gray-900">
                      {agent.capabilities.length} capabilities
                    </div>
                    <div className="text-sm text-gray-500">
                      {agent.social_connections} connections
                    </div>
                  </div>
                </div>
              </div>
              
              {agent.capabilities.length > 0 && (
                <div className="mt-3">
                  <div className="text-xs text-gray-500 mb-1">Capabilities:</div>
                  <div className="flex flex-wrap gap-1">
                    {agent.capabilities.map((cap, index) => (
                      <span
                        key={index}
                        className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800"
                      >
                        {cap.name} ({(cap.proficiency * 100).toFixed(0)}%)
                      </span>
                    ))}
                  </div>
                </div>
              )}
            </li>
          ))}
        </ul>
        
        {agents.length === 0 && (
          <div className="text-center py-12">
            <User className="mx-auto h-12 w-12 text-gray-400" />
            <h3 className="mt-2 text-sm font-medium text-gray-900">No agents</h3>
            <p className="mt-1 text-sm text-gray-500">Get started by creating your first agent.</p>
          </div>
        )}
      </div>
    </div>
  )
}