'use client'

import React, { useCallback, useState } from 'react'
import { useHiveStore } from '@/store/hiveStore'
import { Plus, CheckCircle, XCircle, Clock, AlertCircle } from 'lucide-react'

export const TaskManager = React.memo(function TaskManager() {
  const { createTask, tasks } = useHiveStore()
  const [showCreateForm, setShowCreateForm] = useState(false)
  const [newTask, setNewTask] = useState({
    description: '',
    type: 'general',
    priority: 5,
    required_capabilities: [{ name: '', min_proficiency: 0.5, weight: 1.0 }],
  })

  const handleCreateTask = useCallback(() => {
    // Filter out empty capabilities
    const filteredCapabilities = newTask.required_capabilities.filter(cap => cap.name.trim() !== '')

    const taskConfig = {
      ...newTask,
      required_capabilities: filteredCapabilities.length > 0 ? filteredCapabilities : undefined,
    }

    createTask(taskConfig)
    setNewTask({
      description: '',
      type: 'general',
      priority: 5,
      required_capabilities: [{ name: '', min_proficiency: 0.5, weight: 1.0 }],
    })
    setShowCreateForm(false)
  }, [createTask, newTask])

  const addRequiredCapability = useCallback(() => {
    setNewTask(prev => ({
      ...prev,
      required_capabilities: [
        ...prev.required_capabilities,
        { name: '', min_proficiency: 0.5, weight: 1.0 },
      ],
    }))
  }, [])

  const updateRequiredCapability = useCallback(
    (index: number, field: string, value: string | number) => {
      setNewTask(prev => {
        const updated = [...prev.required_capabilities]
        updated[index] = { ...updated[index], [field]: value }
        return { ...prev, required_capabilities: updated }
      })
    },
    [],
  )

  const removeRequiredCapability = useCallback((index: number) => {
    setNewTask(prev => ({
      ...prev,
      required_capabilities: prev.required_capabilities.filter((_, i) => i !== index),
    }))
  }, [])

  const getPriorityColor = (priority: number) => {
    if (priority >= 8) {
      return 'bg-red-100 text-red-800'
    }
    if (priority >= 6) {
      return 'bg-yellow-100 text-yellow-800'
    }
    if (priority >= 4) {
      return 'bg-blue-100 text-blue-800'
    }
    return 'bg-gray-100 text-gray-800'
  }

  const getPriorityLabel = (priority: number) => {
    if (priority >= 8) {
      return 'Critical'
    }
    if (priority >= 6) {
      return 'High'
    }
    if (priority >= 4) {
      return 'Medium'
    }
    return 'Low'
  }

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'Completed':
        return <CheckCircle className="w-5 h-5 text-green-500" />
      case 'Failed':
        return <XCircle className="w-5 h-5 text-red-500" />
      case 'InProgress':
        return <Clock className="w-5 h-5 text-blue-500" />
      case 'Pending':
        return <AlertCircle className="w-5 h-5 text-yellow-500" />
      default:
        return <Clock className="w-5 h-5 text-gray-500" />
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h2 className="text-2xl font-bold text-gray-900">Task Management</h2>
        <button
          onClick={() => setShowCreateForm(true)}
          className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700"
        >
          <Plus className="w-4 h-4 mr-2" />
          Create Task
        </button>
      </div>

      {showCreateForm && (
        <div className="bg-white shadow rounded-lg p-6">
          <h3 className="text-lg font-medium text-gray-900 mb-4">Create New Task</h3>

          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Description</label>
              <textarea
                value={newTask.description}
                onChange={e => setNewTask({ ...newTask, description: e.target.value })}
                rows={3}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="Describe what this task should accomplish..."
              />
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Task Type</label>
                <select
                  value={newTask.type}
                  onChange={e => setNewTask({ ...newTask, type: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="general">General</option>
                  <option value="nlp">NLP Processing</option>
                  <option value="data_processing">Data Processing</option>
                  <option value="coordination">Coordination</option>
                  <option value="learning">Learning</option>
                  <option value="analysis">Analysis</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Priority (1-10)
                </label>
                <input
                  type="number"
                  min="1"
                  max="10"
                  value={newTask.priority}
                  onChange={e => setNewTask({ ...newTask, priority: parseInt(e.target.value) })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
            </div>

            <div>
              <div className="flex justify-between items-center mb-2">
                <label className="block text-sm font-medium text-gray-700">
                  Required Capabilities
                </label>
                <button
                  onClick={addRequiredCapability}
                  className="text-sm text-blue-600 hover:text-blue-800"
                >
                  Add Capability
                </button>
              </div>

              {newTask.required_capabilities.map((cap, index) => (
                <div key={index} className="grid grid-cols-4 gap-2 mb-2">
                  <input
                    type="text"
                    value={cap.name}
                    onChange={e => updateRequiredCapability(index, 'name', e.target.value)}
                    placeholder="Capability name"
                    className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                  <input
                    type="number"
                    min="0"
                    max="1"
                    step="0.1"
                    value={cap.min_proficiency}
                    onChange={e =>
                      updateRequiredCapability(index, 'min_proficiency', parseFloat(e.target.value))
                    }
                    placeholder="Min proficiency"
                    className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                  <input
                    type="number"
                    min="0"
                    max="10"
                    step="0.1"
                    value={cap.weight}
                    onChange={e =>
                      updateRequiredCapability(index, 'weight', parseFloat(e.target.value))
                    }
                    placeholder="Weight"
                    className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                  <button
                    onClick={() => removeRequiredCapability(index)}
                    className="px-3 py-2 text-red-600 hover:text-red-800"
                  >
                    Remove
                  </button>
                </div>
              ))}
            </div>
          </div>

          <div className="flex justify-end space-x-3 mt-6">
            <button
              onClick={() => setShowCreateForm(false)}
              className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
            >
              Cancel
            </button>
            <button
              onClick={handleCreateTask}
              className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700"
            >
              Create Task
            </button>
          </div>
        </div>
      )}

      <div className="bg-white shadow overflow-hidden sm:rounded-md">
        <ul className="divide-y divide-gray-200">
          {tasks.map(task => (
            <li key={task.id} className="px-6 py-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center">
                  <div className="flex-shrink-0">{getStatusIcon(task.status)}</div>
                  <div className="ml-4">
                    <div className="flex items-center">
                      <div className="text-sm font-medium text-gray-900">{task.description}</div>
                      <span
                        className={`ml-2 inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getPriorityColor(task.priority)}`}
                      >
                        {getPriorityLabel(task.priority)}
                      </span>
                    </div>
                    <div className="text-sm text-gray-500">
                      Type: {task.type} • Created: {new Date(task.created_at).toLocaleString()}
                    </div>
                    {task.assigned_agent !== null &&
                      task.assigned_agent !== undefined &&
                      task.assigned_agent !== '' && (
                      <div className="text-sm text-blue-600">
                          Assigned to: {task.assigned_agent}
                      </div>
                    )}
                  </div>
                </div>

                <div className="text-right">
                  <div className="text-sm font-medium text-gray-900">{task.status}</div>
                  {task.completed_at !== null &&
                    task.completed_at !== undefined &&
                    task.completed_at !== '' && (
                    <div className="text-sm text-gray-500">
                        Completed: {new Date(task.completed_at).toLocaleString()}
                    </div>
                  )}
                </div>
              </div>
            </li>
          ))}
        </ul>

        {tasks.length === 0 && (
          <div className="text-center py-12">
            <AlertCircle className="mx-auto h-12 w-12 text-gray-400" />
            <h3 className="mt-2 text-sm font-medium text-gray-900">No tasks</h3>
            <p className="mt-1 text-sm text-gray-500">Get started by creating your first task.</p>
          </div>
        )}
      </div>
    </div>
  )
})
