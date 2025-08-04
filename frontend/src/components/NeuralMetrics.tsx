'use client';

import { useState } from 'react';
import { Brain, Cpu, Zap, TrendingUp } from 'lucide-react';

interface NeuralMetricsProps {
  agents: any[];
}

export function NeuralMetrics({ agents }: NeuralMetricsProps) {
  const [selectedAgent, setSelectedAgent] = useState<string | null>(null);

  // Mock neural performance data (in real app, this would come from backend)
  const neuralData = {
    processing_mode: 'hybrid',
    basic_agents: agents.filter(a => !a.neural_type || a.neural_type === 'basic').length,
    advanced_agents: agents.filter(a => a.neural_type === 'fann' || a.neural_type === 'lstm').length,
    avg_prediction_accuracy: 0.847,
    learning_rate: 0.023,
    neural_efficiency: 0.92,
  };

  const getAgentNeuralType = (agent: any) => {
    // Mock neural type detection
    if (agent.type === 'Learner') return 'LSTM';
    if (agent.type === 'Specialist') return 'FANN';
    return 'Basic NLP';
  };

  const getPerformanceColor = (performance: number) => {
    if (performance >= 0.8) return 'text-green-600';
    if (performance >= 0.6) return 'text-yellow-600';
    return 'text-red-600';
  };

  return (
    <div className="bg-white shadow rounded-lg p-6">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-medium text-gray-900 flex items-center">
          <Brain className="w-5 h-5 mr-2 text-purple-600" />
          Neural Processing Metrics
        </h3>
        <div className="flex items-center space-x-2">
          <div className="w-3 h-3 bg-green-400 rounded-full"></div>
          <span className="text-sm text-gray-600">Hybrid Mode Active</span>
        </div>
      </div>

      {/* Processing Overview */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
        <div className="text-center p-3 bg-blue-50 rounded-lg">
          <Cpu className="w-6 h-6 mx-auto text-blue-600 mb-1" />
          <div className="text-lg font-semibold text-blue-600">{neuralData.basic_agents}</div>
          <div className="text-xs text-blue-600">Basic NLP</div>
        </div>
        
        <div className="text-center p-3 bg-purple-50 rounded-lg">
          <Brain className="w-6 h-6 mx-auto text-purple-600 mb-1" />
          <div className="text-lg font-semibold text-purple-600">{neuralData.advanced_agents}</div>
          <div className="text-xs text-purple-600">Advanced Neural</div>
        </div>
        
        <div className="text-center p-3 bg-green-50 rounded-lg">
          <TrendingUp className="w-6 h-6 mx-auto text-green-600 mb-1" />
          <div className="text-lg font-semibold text-green-600">
            {(neuralData.avg_prediction_accuracy * 100).toFixed(1)}%
          </div>
          <div className="text-xs text-green-600">Accuracy</div>
        </div>
        
        <div className="text-center p-3 bg-yellow-50 rounded-lg">
          <Zap className="w-6 h-6 mx-auto text-yellow-600 mb-1" />
          <div className="text-lg font-semibold text-yellow-600">
            {(neuralData.neural_efficiency * 100).toFixed(0)}%
          </div>
          <div className="text-xs text-yellow-600">Efficiency</div>
        </div>
      </div>

      {/* Agent Neural Types */}
      <div className="mb-4">
        <h4 className="text-sm font-medium text-gray-700 mb-2">Agent Neural Capabilities</h4>
        <div className="space-y-2 max-h-40 overflow-y-auto">
          {agents.slice(0, 8).map((agent) => (
            <div
              key={agent.id}
              className={`flex items-center justify-between p-2 rounded cursor-pointer transition-colors ${
                selectedAgent === agent.id ? 'bg-blue-50 border border-blue-200' : 'hover:bg-gray-50'
              }`}
              onClick={() => setSelectedAgent(selectedAgent === agent.id ? null : agent.id)}
            >
              <div className="flex items-center">
                <div className="w-2 h-2 rounded-full mr-2 bg-blue-400"></div>
                <span className="text-sm font-medium">{agent.name}</span>
              </div>
              <div className="flex items-center space-x-2">
                <span className="text-xs px-2 py-1 bg-gray-100 rounded">
                  {getAgentNeuralType(agent)}
                </span>
                <span className={`text-xs font-medium ${getPerformanceColor(agent.energy / 100)}`}>
                  {(agent.energy / 100 * 0.8 + 0.2).toFixed(2)}
                </span>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Performance Insights */}
      <div className="border-t pt-4">
        <h4 className="text-sm font-medium text-gray-700 mb-2">Performance Insights</h4>
        <div className="space-y-2 text-sm text-gray-600">
          <div className="flex justify-between">
            <span>Learning Rate:</span>
            <span className="font-medium">{(neuralData.learning_rate * 100).toFixed(1)}%/hour</span>
          </div>
          <div className="flex justify-between">
            <span>Pattern Recognition:</span>
            <span className="font-medium text-green-600">Excellent</span>
          </div>
          <div className="flex justify-between">
            <span>Memory Usage:</span>
            <span className="font-medium text-blue-600">Optimized</span>
          </div>
        </div>
      </div>

      {/* Recommendation */}
      <div className="mt-4 p-3 bg-gray-50 rounded-lg">
        <div className="text-xs text-gray-600">
          💡 <strong>Tip:</strong> Your hive is using hybrid neural processing. 
          {neuralData.advanced_agents > 0 
            ? ' Advanced agents are handling complex tasks efficiently.'
            : ' Consider enabling advanced neural features for performance-critical tasks.'
          }
        </div>
      </div>
    </div>
  );
}