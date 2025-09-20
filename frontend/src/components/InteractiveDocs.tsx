/**
 * @jest-environment jsdom
 */
import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import { Button } from './ui/button';
import { Badge } from './ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './ui/tabs';
import { Alert, AlertDescription } from './ui/alert';
import {
  BookOpen,
  Code,
  Play,
  Copy,
  Check,
  ExternalLink,
  Search,
  Filter,
  ChevronRight,
  ChevronDown
} from 'lucide-react';

interface DocSection {
  id: string;
  title: string;
  content: string;
  examples: CodeExample[];
  related: string[];
  tags: string[];
}

interface CodeExample {
  id: string;
  title: string;
  language: 'rust' | 'typescript' | 'bash' | 'json';
  code: string;
  description: string;
  runnable?: boolean;
  output?: string;
}

const InteractiveDocs: React.FC = () => {
  const [sections, setSections] = useState<DocSection[]>([]);
  const [activeSection, setActiveSection] = useState<string>('');
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [expandedSections, setExpandedSections] = useState<Set<string>>(new Set());
  const [copiedCode, setCopiedCode] = useState<string | null>(null);

  // Mock documentation data - in a real app, this would come from an API
  useEffect(() => {
    const mockSections: DocSection[] = [
      {
        id: 'getting-started',
        title: 'Getting Started',
        content: 'Learn how to set up and run the AI Orchestrator Hub in your environment.',
        examples: [
          {
            id: 'install-rust',
            title: 'Install Rust',
            language: 'bash',
            code: 'curl --proto \'=https\' --tlsv1.2 -sSf https://sh.rustup.rs | sh\nsource ~/.cargo/env\nrustup component add clippy rustfmt',
            description: 'Install Rust and required components',
            runnable: false
          },
          {
            id: 'install-node',
            title: 'Install Node.js',
            language: 'bash',
            code: 'curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -\nsudo apt-get install -y nodejs',
            description: 'Install Node.js 20.x',
            runnable: false
          },
          {
            id: 'clone-repo',
            title: 'Clone Repository',
            language: 'bash',
            code: 'git clone https://github.com/your-org/ai-orchestrator-hub.git\ncd ai-orchestrator-hub',
            description: 'Clone the repository and navigate to the project directory',
            runnable: false
          }
        ],
        related: ['configuration', 'deployment'],
        tags: ['setup', 'installation', 'beginner']
      },
      {
        id: 'authentication',
        title: 'Authentication & Security',
        content: 'Secure your AI Orchestrator Hub with JWT-based authentication and role-based access control.',
        examples: [
          {
            id: 'register-user',
            title: 'Register a New User',
            language: 'bash',
            code: `curl -X POST http://localhost:3001/api/auth/register \\
  -H "Content-Type: application/json" \\
  -d '{
    "username": "admin",
    "password": "secure_password123!",
    "role": "admin"
  }'`,
            description: 'Register a new user with admin role',
            runnable: true,
            output: '{"user":{"username":"admin","role":"admin"},"message":"User registered successfully"}'
          },
          {
            id: 'login-user',
            title: 'User Login',
            language: 'bash',
            code: `curl -X POST http://localhost:3001/api/auth/login \\
  -H "Content-Type: application/json" \\
  -d '{
    "username": "admin",
    "password": "secure_password123!"
  }'`,
            description: 'Login and receive JWT tokens',
            runnable: true,
            output: '{"access_token":"eyJ...","refresh_token":"refresh...","user":{"username":"admin","role":"admin"}}'
          },
          {
            id: 'rust-auth-example',
            title: 'Rust Authentication Client',
            language: 'rust',
            code: `use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // Register user
    let register_response = client
        .post("http://localhost:3001/api/auth/register")
        .json(&json!({
            "username": "testuser",
            "password": "password123!",
            "role": "user"
        }))
        .send()
        .await?;

    println!("Registration: {:?}", register_response.text().await?);

    // Login
    let login_response = client
        .post("http://localhost:3001/api/auth/login")
        .json(&json!({
            "username": "testuser",
            "password": "password123!"
        }))
        .send()
        .await?;

    let login_data: serde_json::Value = login_response.json().await?;
    let token = login_data["access_token"].as_str().unwrap();

    // Use authenticated endpoint
    let protected_response = client
        .get("http://localhost:3001/api/agents")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    println!("Protected resource: {:?}", protected_response.text().await?);

    Ok(())
}`,
            description: 'Complete Rust client example for authentication',
            runnable: false
          }
        ],
        related: ['api-reference', 'security'],
        tags: ['authentication', 'security', 'api', 'jwt']
      },
      {
        id: 'agent-management',
        title: 'Agent Management',
        content: 'Create, configure, and manage AI agents in your swarm.',
        examples: [
          {
            id: 'create-agent',
            title: 'Create a Worker Agent',
            language: 'bash',
            code: `curl -X POST http://localhost:3001/api/agents \\
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \\
  -H "Content-Type: application/json" \\
  -d '{
    "name": "data-processor",
    "type": "worker",
    "capabilities": ["data_processing", "analysis"],
    "config": {
      "max_concurrent_tasks": 5,
      "memory_limit_mb": 512,
      "timeout_seconds": 300
    }
  }'`,
            description: 'Create a new worker agent with specific capabilities',
            runnable: true,
            output: '{"agent_id":"agent-123","status":"created","message":"Agent created successfully"}'
          },
          {
            id: 'list-agents',
            title: 'List All Agents',
            language: 'bash',
            code: `curl -X GET http://localhost:3001/api/agents \\
  -H "Authorization: Bearer YOUR_JWT_TOKEN"`,
            description: 'Get a list of all agents in the system',
            runnable: true,
            output: `{
  "agents": [
    {
      "id": "agent-123",
      "name": "data-processor",
      "type": "worker",
      "status": "active",
      "capabilities": ["data_processing", "analysis"],
      "active_tasks": 2
    }
  ]
}`
          },
          {
            id: 'typescript-agent-client',
            title: 'TypeScript Agent Management',
            language: 'typescript',
            code: `interface Agent {
  id: string;
  name: string;
  type: 'worker' | 'coordinator' | 'specialist';
  status: 'active' | 'idle' | 'failed';
  capabilities: string[];
  active_tasks: number;
}

class AgentManager {
  private baseUrl: string;
  private token: string;

  constructor(baseUrl: string, token: string) {
    this.baseUrl = baseUrl;
    this.token = token;
  }

  async createAgent(config: {
    name: string;
    type: string;
    capabilities: string[];
    config?: Record<string, any>;
  }): Promise<Agent> {
    const response = await fetch(\`\${this.baseUrl}/api/agents\`, {
      method: 'POST',
      headers: {
        'Authorization': \`Bearer \${this.token}\`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(config),
    });

    if (!response.ok) {
      throw new Error(\`Failed to create agent: \${response.statusText}\`);
    }

    return response.json();
  }

  async getAgents(): Promise<Agent[]> {
    const response = await fetch(\`\${this.baseUrl}/api/agents\`, {
      headers: {
        'Authorization': \`Bearer \${this.token}\`,
      },
    });

    if (!response.ok) {
      throw new Error(\`Failed to get agents: \${response.statusText}\`);
    }

    const data = await response.json();
    return data.agents || [];
  }

  async getAgentStatus(agentId: string): Promise<Agent> {
    const response = await fetch(\`\${this.baseUrl}/api/agents/\${agentId}\`, {
      headers: {
        'Authorization': \`Bearer \${this.token}\`,
      },
    });

    if (!response.ok) {
      throw new Error(\`Failed to get agent status: \${response.statusText}\`);
    }

    return response.json();
  }
}

// Usage example
const agentManager = new AgentManager('http://localhost:3001', 'your-jwt-token');

async function example() {
  try {
    // Create a new agent
    const agent = await agentManager.createAgent({
      name: 'ml-processor',
      type: 'worker',
      capabilities: ['machine_learning', 'data_analysis'],
      config: {
        max_concurrent_tasks: 3,
        memory_limit_mb: 1024,
      },
    });

    console.log('Created agent:', agent);

    // List all agents
    const agents = await agentManager.getAgents();
    console.log('All agents:', agents);

    // Get specific agent status
    const status = await agentManager.getAgentStatus(agent.id);
    console.log('Agent status:', status);
  } catch (error) {
    console.error('Error:', error);
  }
}`,
            description: 'Complete TypeScript client for agent management',
            runnable: false
          }
        ],
        related: ['task-management', 'swarm-coordination'],
        tags: ['agents', 'management', 'api', 'typescript']
      },
      {
        id: 'monitoring',
        title: 'Monitoring & Metrics',
        content: 'Monitor system performance, health, and metrics in real-time.',
        examples: [
          {
            id: 'get-metrics',
            title: 'Get System Metrics',
            language: 'bash',
            code: `curl -X GET http://localhost:3001/api/monitoring/metrics \\
  -H "Authorization: Bearer YOUR_JWT_TOKEN"`,
            description: 'Retrieve current system metrics and performance data',
            runnable: true,
            output: `{
  "current": {
    "cpu_usage": 45.5,
    "memory_usage": 67.8,
    "agent_metrics": {
      "total_agents": 5,
      "active_agents": 4,
      "average_response_time": 125.3
    },
    "performance_metrics": {
      "throughput": 150.5,
      "latency_p50": 45.2,
      "error_rate": 0.02
    }
  }
}`
          },
          {
            id: 'get-alerts',
            title: 'Get Active Alerts',
            language: 'bash',
            code: `curl -X GET http://localhost:3001/api/monitoring/alerts \\
  -H "Authorization: Bearer YOUR_JWT_TOKEN"`,
            description: 'Retrieve active system alerts and notifications',
            runnable: true,
            output: `{
  "alerts": [
    {
      "id": "alert-1",
      "title": "High CPU Usage",
      "description": "CPU usage exceeded 80%",
      "severity": "medium",
      "source": "system_monitor",
      "timestamp": "2024-01-15T10:30:00Z",
      "acknowledged": false,
      "resolved": false
    }
  ],
  "total": 1,
  "active": 1
}`
          },
          {
            id: 'health-check',
            title: 'System Health Check',
            language: 'bash',
            code: `curl -X GET http://localhost:3001/api/monitoring/health`,
            description: 'Check overall system health and component status',
            runnable: true,
            output: `{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "uptime_seconds": 3600,
  "components": {
    "hive_coordinator": {
      "status": "healthy",
      "active_agents": 5,
      "pending_tasks": 2
    },
    "monitoring_system": {
      "status": "healthy",
      "metrics_collected": 150,
      "alerts_active": 1
    }
  },
  "performance": {
    "response_time_p50": 45.2,
    "throughput": 150.5,
    "error_rate": 0.02
  }
}`
          }
        ],
        related: ['troubleshooting', 'performance-optimization'],
        tags: ['monitoring', 'metrics', 'health', 'alerts']
      }
    ];

    setSections(mockSections);
    if (mockSections.length > 0) {
      setActiveSection(mockSections[0].id);
    }
  }, []);

  const copyToClipboard = async (code: string, id: string) => {
    try {
      await navigator.clipboard.writeText(code);
      setCopiedCode(id);
      setTimeout(() => setCopiedCode(null), 2000);
    } catch (err) {
      console.error('Failed to copy code:', err);
    }
  };

  const runExample = async (example: CodeExample) => {
    // In a real implementation, this would execute the code
    console.log('Running example:', example.title);
    alert(`Example "${example.title}" would be executed here. In a real implementation, this would run the code and show results.`);
  };

  const filteredSections = sections.filter(section => {
    const matchesSearch = section.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         section.content.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         section.examples.some(ex => ex.title.toLowerCase().includes(searchTerm.toLowerCase()));

    const matchesTags = selectedTags.length === 0 ||
                       selectedTags.some(tag => section.tags.includes(tag));

    return matchesSearch && matchesTags;
  });

  const allTags = Array.from(new Set(sections.flatMap(s => s.tags))).sort();

  const toggleSection = (sectionId: string) => {
    const newExpanded = new Set(expandedSections);
    if (newExpanded.has(sectionId)) {
      newExpanded.delete(sectionId);
    } else {
      newExpanded.add(sectionId);
    }
    setExpandedSections(newExpanded);
  };

  const CodeBlock: React.FC<{ example: CodeExample }> = ({ example }) => (
    <div className="border rounded-lg overflow-hidden">
      <div className="flex items-center justify-between px-4 py-2 bg-gray-50 border-b">
        <div className="flex items-center space-x-2">
          <Code className="h-4 w-4" />
          <span className="font-medium">{example.title}</span>
          <Badge variant="outline" className="text-xs">
            {example.language}
          </Badge>
        </div>
        <div className="flex items-center space-x-1">
          {example.runnable && (
            <Button
              size="sm"
              variant="outline"
              onClick={() => runExample(example)}
              className="h-7 px-2"
            >
              <Play className="h-3 w-3 mr-1" />
              Run
            </Button>
          )}
          <Button
            size="sm"
            variant="outline"
            onClick={() => copyToClipboard(example.code, example.id)}
            className="h-7 px-2"
          >
            {copiedCode === example.id ? (
              <Check className="h-3 w-3" />
            ) : (
              <Copy className="h-3 w-3" />
            )}
          </Button>
        </div>
      </div>
      <div className="relative">
        <pre className="p-4 text-sm overflow-x-auto bg-gray-900 text-gray-100">
          <code>{example.code}</code>
        </pre>
      </div>
      {example.output && (
        <div className="border-t bg-gray-50">
          <div className="px-4 py-2 text-xs font-medium text-gray-600">Output:</div>
          <pre className="px-4 pb-4 text-sm text-gray-800">
            <code>{example.output}</code>
          </pre>
        </div>
      )}
      <div className="px-4 py-3 bg-blue-50 border-t">
        <p className="text-sm text-blue-800">{example.description}</p>
      </div>
    </div>
  );

  return (
    <div className="max-w-7xl mx-auto p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold flex items-center">
            <BookOpen className="h-8 w-8 mr-3 text-blue-600" />
            Interactive Documentation
          </h1>
          <p className="text-gray-600 mt-2">
            Explore the AI Orchestrator Hub with interactive examples and cross-referenced guides
          </p>
        </div>
      </div>

      {/* Search and Filters */}
      <Card>
        <CardContent className="pt-6">
          <div className="flex flex-col md:flex-row gap-4">
            <div className="flex-1">
              <div className="relative">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
                <input
                  type="text"
                  placeholder="Search documentation..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>
            </div>
            <div className="flex items-center space-x-2">
              <Filter className="h-4 w-4 text-gray-400" />
              <div className="flex flex-wrap gap-2">
                {allTags.map(tag => (
                  <Badge
                    key={tag}
                    variant={selectedTags.includes(tag) ? "default" : "outline"}
                    className="cursor-pointer"
                    onClick={() => {
                      setSelectedTags(prev =>
                        prev.includes(tag)
                          ? prev.filter(t => t !== tag)
                          : [...prev, tag]
                      );
                    }}
                  >
                    {tag}
                  </Badge>
                ))}
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Documentation Content */}
      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        {/* Table of Contents */}
        <Card className="lg:col-span-1">
          <CardHeader>
            <CardTitle className="text-lg">Contents</CardTitle>
          </CardHeader>
          <CardContent className="p-0">
            <nav className="space-y-1">
              {filteredSections.map(section => (
                <div key={section.id}>
                  <button
                    onClick={() => {
                      setActiveSection(section.id);
                      toggleSection(section.id);
                    }}
                    className={`w-full text-left px-4 py-2 hover:bg-gray-50 flex items-center justify-between ${
                      activeSection === section.id ? 'bg-blue-50 text-blue-700' : 'text-gray-700'
                    }`}
                  >
                    <span className="font-medium">{section.title}</span>
                    {expandedSections.has(section.id) ? (
                      <ChevronDown className="h-4 w-4" />
                    ) : (
                      <ChevronRight className="h-4 w-4" />
                    )}
                  </button>
                  {expandedSections.has(section.id) && (
                    <div className="ml-4 space-y-1">
                      {section.examples.map(example => (
                        <button
                          key={example.id}
                          onClick={() => setActiveSection(section.id)}
                          className="w-full text-left px-4 py-1 text-sm text-gray-600 hover:text-gray-900 hover:bg-gray-50"
                        >
                          {example.title}
                        </button>
                      ))}
                    </div>
                  )}
                </div>
              ))}
            </nav>
          </CardContent>
        </Card>

        {/* Main Content */}
        <div className="lg:col-span-3">
          {filteredSections.map(section => (
            <Card key={section.id} className={activeSection === section.id ? '' : 'hidden'}>
              <CardHeader>
                <div className="flex items-center justify-between">
                  <CardTitle className="text-2xl">{section.title}</CardTitle>
                  <div className="flex items-center space-x-2">
                    {section.tags.map(tag => (
                      <Badge key={tag} variant="secondary" className="text-xs">
                        {tag}
                      </Badge>
                    ))}
                  </div>
                </div>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="prose max-w-none">
                  <p className="text-gray-700">{section.content}</p>
                </div>

                {/* Examples */}
                <div className="space-y-4">
                  <h3 className="text-lg font-semibold flex items-center">
                    <Code className="h-5 w-5 mr-2" />
                    Code Examples
                  </h3>
                  {section.examples.map(example => (
                    <CodeBlock key={example.id} example={example} />
                  ))}
                </div>

                {/* Related Documentation */}
                {section.related.length > 0 && (
                  <div className="border-t pt-4">
                    <h4 className="font-medium mb-2">Related Documentation</h4>
                    <div className="flex flex-wrap gap-2">
                      {section.related.map(relatedId => {
                        const relatedSection = sections.find(s => s.id === relatedId);
                        return relatedSection ? (
                          <Button
                            key={relatedId}
                            variant="outline"
                            size="sm"
                            onClick={() => {
                              setActiveSection(relatedId);
                              toggleSection(relatedId);
                            }}
                            className="h-8"
                          >
                            <ExternalLink className="h-3 w-3 mr-1" />
                            {relatedSection.title}
                          </Button>
                        ) : null;
                      })}
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>
          ))}

          {filteredSections.length === 0 && (
            <Card>
              <CardContent className="py-12 text-center">
                <BookOpen className="h-12 w-12 mx-auto text-gray-400 mb-4" />
                <h3 className="text-lg font-medium text-gray-900 mb-2">No results found</h3>
                <p className="text-gray-600">
                  Try adjusting your search terms or filters to find what you're looking for.
                </p>
              </CardContent>
            </Card>
          )}
        </div>
      </div>

      {/* Quick Actions */}
      <Card>
        <CardHeader>
          <CardTitle>Quick Actions</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <Button className="h-16 flex-col">
              <Play className="h-6 w-6 mb-2" />
              Try Live Demo
            </Button>
            <Button variant="outline" className="h-16 flex-col">
              <ExternalLink className="h-6 w-6 mb-2" />
              API Reference
            </Button>
            <Button variant="outline" className="h-16 flex-col">
              <BookOpen className="h-6 w-6 mb-2" />
              Full Documentation
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default InteractiveDocs;