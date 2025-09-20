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
  Code,
  Send,
  Eye,
  EyeOff,
  Copy,
  Check,
  ExternalLink,
  Search,
  Filter,
  ChevronRight,
  ChevronDown,
  Zap,
  Shield,
  Users,
  Activity
} from 'lucide-react';

interface ApiEndpoint {
  id: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  path: string;
  title: string;
  description: string;
  category: 'auth' | 'agents' | 'tasks' | 'monitoring' | 'system';
  parameters: ApiParameter[];
  requestBody?: ApiSchema;
  responses: ApiResponse[];
  examples: ApiExample[];
  authentication: boolean;
  rateLimited: boolean;
}

interface ApiParameter {
  name: string;
  type: string;
  required: boolean;
  description: string;
  location: 'path' | 'query' | 'header';
}

interface ApiSchema {
  type: 'object' | 'array' | 'string' | 'number' | 'boolean';
  properties?: Record<string, ApiSchema>;
  items?: ApiSchema;
  description?: string;
  example?: any;
}

interface ApiResponse {
  status: number;
  description: string;
  schema?: ApiSchema;
  example?: any;
}

interface ApiExample {
  title: string;
  language: 'curl' | 'javascript' | 'python' | 'rust';
  code: string;
}

const ApiReference: React.FC = () => {
  const [endpoints, setEndpoints] = useState<ApiEndpoint[]>([]);
  const [selectedEndpoint, setSelectedEndpoint] = useState<string>('');
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('all');
  const [showRequestBody, setShowRequestBody] = useState<Record<string, boolean>>({});
  const [showResponse, setShowResponse] = useState<Record<string, boolean>>({});
  const [copiedCode, setCopiedCode] = useState<string | null>(null);

  // Mock API endpoint data - in a real app, this would come from an API
  useEffect(() => {
    const mockEndpoints: ApiEndpoint[] = [
      {
        id: 'auth-login',
        method: 'POST',
        path: '/api/auth/login',
        title: 'User Login',
        description: 'Authenticate a user and receive JWT access tokens',
        category: 'auth',
        parameters: [],
        requestBody: {
          type: 'object',
          properties: {
            username: {
              type: 'string',
              description: 'User username',
              example: 'admin'
            },
            password: {
              type: 'string',
              description: 'User password',
              example: 'secure_password123!'
            }
          }
        },
        responses: [
          {
            status: 200,
            description: 'Login successful',
            schema: {
              type: 'object',
              properties: {
                access_token: { type: 'string', description: 'JWT access token' },
                refresh_token: { type: 'string', description: 'JWT refresh token' },
                user: {
                  type: 'object',
                  properties: {
                    username: { type: 'string' },
                    role: { type: 'string' }
                  }
                },
                expires_in: { type: 'number', description: 'Token expiration time in seconds' }
              }
            },
            example: {
              access_token: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...',
              refresh_token: 'refresh_token_here',
              user: { username: 'admin', role: 'admin' },
              expires_in: 3600
            }
          },
          {
            status: 401,
            description: 'Invalid credentials',
            example: { error: 'Invalid username or password' }
          }
        ],
        examples: [
          {
            title: 'Login with curl',
            language: 'curl',
            code: `curl -X POST http://localhost:3001/api/auth/login \\
  -H "Content-Type: application/json" \\
  -d '{
    "username": "admin",
    "password": "secure_password123!"
  }'`
          },
          {
            title: 'Login with JavaScript',
            language: 'javascript',
            code: `const response = await fetch('/api/auth/login', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    username: 'admin',
    password: 'secure_password123!'
  })
});

const data = await response.json();
console.log('Access token:', data.access_token);`
          }
        ],
        authentication: false,
        rateLimited: true
      },
      {
        id: 'auth-register',
        method: 'POST',
        path: '/api/auth/register',
        title: 'User Registration',
        description: 'Register a new user account',
        category: 'auth',
        parameters: [],
        requestBody: {
          type: 'object',
          properties: {
            username: {
              type: 'string',
              description: 'Desired username',
              example: 'newuser'
            },
            password: {
              type: 'string',
              description: 'User password (min 8 characters)',
              example: 'secure_password123!'
            },
            role: {
              type: 'string',
              description: 'User role',
              example: 'user'
            }
          }
        },
        responses: [
          {
            status: 201,
            description: 'User registered successfully',
            example: {
              user: { username: 'newuser', role: 'user' },
              message: 'User registered successfully'
            }
          },
          {
            status: 409,
            description: 'Username already exists',
            example: { error: 'Username already exists' }
          }
        ],
        examples: [
          {
            title: 'Register user',
            language: 'curl',
            code: `curl -X POST http://localhost:3001/api/auth/register \\
  -H "Content-Type: application/json" \\
  -d '{
    "username": "newuser",
    "password": "secure_password123!",
    "role": "user"
  }'`
          }
        ],
        authentication: false,
        rateLimited: true
      },
      {
        id: 'agents-list',
        method: 'GET',
        path: '/api/agents',
        title: 'List Agents',
        description: 'Get a list of all agents in the system',
        category: 'agents',
        parameters: [
          {
            name: 'status',
            type: 'string',
            required: false,
            description: 'Filter by agent status (active, idle, failed)',
            location: 'query'
          },
          {
            name: 'type',
            type: 'string',
            required: false,
            description: 'Filter by agent type (worker, coordinator, specialist)',
            location: 'query'
          }
        ],
        responses: [
          {
            status: 200,
            description: 'List of agents',
            schema: {
              type: 'object',
              properties: {
                agents: {
                  type: 'array',
                  items: {
                    type: 'object',
                    properties: {
                      id: { type: 'string' },
                      name: { type: 'string' },
                      type: { type: 'string' },
                      status: { type: 'string' },
                      capabilities: { type: 'array', items: { type: 'string' } },
                      active_tasks: { type: 'number' }
                    }
                  }
                }
              }
            }
          }
        ],
        examples: [
          {
            title: 'List all agents',
            language: 'curl',
            code: `curl -X GET http://localhost:3001/api/agents \\
  -H "Authorization: Bearer YOUR_JWT_TOKEN"`
          },
          {
            title: 'List active agents only',
            language: 'curl',
            code: `curl -X GET "http://localhost:3001/api/agents?status=active" \\
  -H "Authorization: Bearer YOUR_JWT_TOKEN"`
          }
        ],
        authentication: true,
        rateLimited: true
      },
      {
        id: 'agents-create',
        method: 'POST',
        path: '/api/agents',
        title: 'Create Agent',
        description: 'Create a new agent in the system',
        category: 'agents',
        parameters: [],
        requestBody: {
          type: 'object',
          properties: {
            name: {
              type: 'string',
              description: 'Agent name',
              example: 'data-processor'
            },
            type: {
              type: 'string',
              description: 'Agent type',
              example: 'worker'
            },
            capabilities: {
              type: 'array',
              items: { type: 'string' },
              description: 'Agent capabilities',
              example: ['data_processing', 'analysis']
            },
            config: {
              type: 'object',
              description: 'Agent configuration',
              properties: {
                max_concurrent_tasks: { type: 'number', example: 5 },
                memory_limit_mb: { type: 'number', example: 512 },
                timeout_seconds: { type: 'number', example: 300 }
              }
            }
          }
        },
        responses: [
          {
            status: 201,
            description: 'Agent created successfully',
            example: {
              agent_id: 'agent-123',
              status: 'created',
              message: 'Agent created successfully'
            }
          }
        ],
        examples: [
          {
            title: 'Create worker agent',
            language: 'curl',
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
  }'`
          }
        ],
        authentication: true,
        rateLimited: true
      },
      {
        id: 'monitoring-metrics',
        method: 'GET',
        path: '/api/monitoring/metrics',
        title: 'Get System Metrics',
        description: 'Retrieve current system performance metrics',
        category: 'monitoring',
        parameters: [
          {
            name: 'hours',
            type: 'number',
            required: false,
            description: 'Number of hours of historical data to include (default: 1)',
            location: 'query'
          }
        ],
        responses: [
          {
            status: 200,
            description: 'System metrics data',
            schema: {
              type: 'object',
              properties: {
                current: {
                  type: 'object',
                  description: 'Current metrics snapshot'
                },
                history: {
                  type: 'array',
                  description: 'Historical metrics data'
                },
                timestamp: { type: 'string', description: 'Response timestamp' }
              }
            }
          }
        ],
        examples: [
          {
            title: 'Get current metrics',
            language: 'curl',
            code: `curl -X GET http://localhost:3001/api/monitoring/metrics \\
  -H "Authorization: Bearer YOUR_JWT_TOKEN"`
          },
          {
            title: 'Get 24 hours of metrics',
            language: 'curl',
            code: `curl -X GET "http://localhost:3001/api/monitoring/metrics?hours=24" \\
  -H "Authorization: Bearer YOUR_JWT_TOKEN"`
          }
        ],
        authentication: true,
        rateLimited: true
      },
      {
        id: 'monitoring-health',
        method: 'GET',
        path: '/api/monitoring/health',
        title: 'System Health Check',
        description: 'Check overall system health and component status',
        category: 'monitoring',
        parameters: [],
        responses: [
          {
            status: 200,
            description: 'System is healthy',
            example: {
              status: 'healthy',
              timestamp: '2024-01-15T10:30:00Z',
              uptime_seconds: 3600,
              components: {
                hive_coordinator: { status: 'healthy', active_agents: 5 },
                monitoring_system: { status: 'healthy', metrics_collected: 150 }
              }
            }
          },
          {
            status: 503,
            description: 'System is unhealthy',
            example: {
              status: 'unhealthy',
              timestamp: '2024-01-15T10:30:00Z',
              issues: ['Database connection failed', 'High memory usage']
            }
          }
        ],
        examples: [
          {
            title: 'Health check',
            language: 'curl',
            code: 'curl -X GET http://localhost:3001/api/monitoring/health'
          }
        ],
        authentication: false,
        rateLimited: false
      }
    ];

    setEndpoints(mockEndpoints);
    if (mockEndpoints.length > 0) {
      setSelectedEndpoint(mockEndpoints[0].id);
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

  const getMethodColor = (method: string) => {
    switch (method) {
      case 'GET': return 'bg-green-100 text-green-800';
      case 'POST': return 'bg-blue-100 text-blue-800';
      case 'PUT': return 'bg-yellow-100 text-yellow-800';
      case 'DELETE': return 'bg-red-100 text-red-800';
      case 'PATCH': return 'bg-purple-100 text-purple-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  const getCategoryIcon = (category: string) => {
    switch (category) {
      case 'auth': return <Shield className="h-4 w-4" />;
      case 'agents': return <Users className="h-4 w-4" />;
      case 'monitoring': return <Activity className="h-4 w-4" />;
      default: return <Code className="h-4 w-4" />;
    }
  };

  const filteredEndpoints = endpoints.filter(endpoint => {
    const matchesSearch = endpoint.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         endpoint.path.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         endpoint.description.toLowerCase().includes(searchTerm.toLowerCase());

    const matchesCategory = selectedCategory === 'all' || endpoint.category === selectedCategory;

    return matchesSearch && matchesCategory;
  });

  const categories = ['all', ...Array.from(new Set(endpoints.map(e => e.category)))];

  const toggleRequestBody = (endpointId: string) => {
    setShowRequestBody(prev => ({
      ...prev,
      [endpointId]: !prev[endpointId]
    }));
  };

  const toggleResponse = (endpointId: string, responseIndex: number) => {
    const key = `${endpointId}-${responseIndex}`;
    setShowResponse(prev => ({
      ...prev,
      [key]: !prev[key]
    }));
  };

  const renderSchema = (schema: ApiSchema, level = 0): JSX.Element => {
    const indent = '  '.repeat(level);

    switch (schema.type) {
      case 'object':
        return (
          <div>
            <span className="text-purple-600">{'{'}</span>
            {schema.properties && (
              <div className="ml-4">
                {Object.entries(schema.properties).map(([key, propSchema]) => (
                  <div key={key} className="flex items-start">
                    <span className="text-blue-600">"{key}"</span>
                    <span className="text-gray-600">:</span>
                    <div className="ml-2">
                      {renderSchema(propSchema, level + 1)}
                      {propSchema.description && (
                        <span className="text-gray-500 text-sm ml-2">// {propSchema.description}</span>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
            <span className="text-purple-600">{'}'}</span>
          </div>
        );

      case 'array':
        return (
          <div>
            <span className="text-purple-600">[</span>
            {schema.items && (
              <div className="ml-4">
                {renderSchema(schema.items, level + 1)}
              </div>
            )}
            <span className="text-purple-600">]</span>
          </div>
        );

      default:
        return (
          <span className="text-green-600">
            "{schema.type}"
            {schema.example !== undefined && (
              <span className="text-gray-500"> // e.g., {JSON.stringify(schema.example)}</span>
            )}
          </span>
        );
    }
  };

  return (
    <div className="max-w-7xl mx-auto p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold flex items-center">
            <Code className="h-8 w-8 mr-3 text-blue-600" />
            API Reference
          </h1>
          <p className="text-gray-600 mt-2">
            Complete API documentation with interactive examples and cross-references
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
                  placeholder="Search endpoints..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>
            </div>
            <div className="flex items-center space-x-2">
              <Filter className="h-4 w-4 text-gray-400" />
              <select
                value={selectedCategory}
                onChange={(e) => setSelectedCategory(e.target.value)}
                className="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              >
                {categories.map(category => (
                  <option key={category} value={category}>
                    {category === 'all' ? 'All Categories' : category.charAt(0).toUpperCase() + category.slice(1)}
                  </option>
                ))}
              </select>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* API Content */}
      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        {/* Endpoint List */}
        <Card className="lg:col-span-1">
          <CardHeader>
            <CardTitle className="text-lg">Endpoints</CardTitle>
          </CardHeader>
          <CardContent className="p-0">
            <nav className="space-y-1 max-h-96 overflow-y-auto">
              {filteredEndpoints.map(endpoint => (
                <button
                  key={endpoint.id}
                  onClick={() => setSelectedEndpoint(endpoint.id)}
                  className={`w-full text-left px-4 py-3 hover:bg-gray-50 border-l-4 ${
                    selectedEndpoint === endpoint.id
                      ? 'bg-blue-50 border-blue-500'
                      : 'border-transparent'
                  }`}
                >
                  <div className="flex items-center space-x-2">
                    <Badge className={`text-xs ${getMethodColor(endpoint.method)}`}>
                      {endpoint.method}
                    </Badge>
                    <div className="flex items-center space-x-1">
                      {getCategoryIcon(endpoint.category)}
                      <span className="text-sm font-medium truncate">{endpoint.title}</span>
                    </div>
                  </div>
                  <div className="text-xs text-gray-500 mt-1 truncate">
                    {endpoint.path}
                  </div>
                </button>
              ))}
            </nav>
          </CardContent>
        </Card>

        {/* Endpoint Details */}
        <div className="lg:col-span-3">
          {filteredEndpoints.map(endpoint => (
            <Card key={endpoint.id} className={selectedEndpoint === endpoint.id ? '' : 'hidden'}>
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-3">
                    <Badge className={getMethodColor(endpoint.method)}>
                      {endpoint.method}
                    </Badge>
                    <div>
                      <CardTitle className="text-xl">{endpoint.title}</CardTitle>
                      <code className="text-sm text-gray-600 bg-gray-100 px-2 py-1 rounded">
                        {endpoint.path}
                      </code>
                    </div>
                  </div>
                  <div className="flex items-center space-x-2">
                    {endpoint.authentication && (
                      <Badge variant="outline" className="text-xs">
                        <Shield className="h-3 w-3 mr-1" />
                        Auth Required
                      </Badge>
                    )}
                    {endpoint.rateLimited && (
                      <Badge variant="outline" className="text-xs">
                        <Zap className="h-3 w-3 mr-1" />
                        Rate Limited
                      </Badge>
                    )}
                  </div>
                </div>
              </CardHeader>
              <CardContent className="space-y-6">
                <p className="text-gray-700">{endpoint.description}</p>

                {/* Parameters */}
                {endpoint.parameters.length > 0 && (
                  <div>
                    <h3 className="font-semibold mb-3">Parameters</h3>
                    <div className="space-y-2">
                      {endpoint.parameters.map((param, index) => (
                        <div key={index} className="flex items-start space-x-3 p-3 bg-gray-50 rounded-lg">
                          <div className="flex-1">
                            <div className="flex items-center space-x-2">
                              <code className="font-mono text-sm bg-white px-2 py-1 rounded border">
                                {param.name}
                              </code>
                              <Badge variant="outline" className="text-xs">
                                {param.location}
                              </Badge>
                              {param.required && (
                                <Badge variant="destructive" className="text-xs">
                                  Required
                                </Badge>
                              )}
                            </div>
                            <p className="text-sm text-gray-600 mt-1">{param.description}</p>
                            <p className="text-xs text-gray-500 mt-1">Type: {param.type}</p>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {/* Request Body */}
                {endpoint.requestBody && (
                  <div>
                    <div className="flex items-center justify-between mb-3">
                      <h3 className="font-semibold">Request Body</h3>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => toggleRequestBody(endpoint.id)}
                      >
                        {showRequestBody[endpoint.id] ? (
                          <><EyeOff className="h-4 w-4 mr-1" /> Hide</>
                        ) : (
                          <><Eye className="h-4 w-4 mr-1" /> Show</>
                        )}
                      </Button>
                    </div>
                    {showRequestBody[endpoint.id] && (
                      <div className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                        <pre className="text-sm">
                          {renderSchema(endpoint.requestBody!)}
                        </pre>
                      </div>
                    )}
                  </div>
                )}

                {/* Responses */}
                <div>
                  <h3 className="font-semibold mb-3">Responses</h3>
                  <div className="space-y-3">
                    {endpoint.responses.map((response, index) => (
                      <div key={index} className="border rounded-lg">
                        <div className="flex items-center justify-between p-3 bg-gray-50">
                          <div className="flex items-center space-x-2">
                            <Badge
                              variant={response.status >= 200 && response.status < 300 ? "default" : "destructive"}
                            >
                              {response.status}
                            </Badge>
                            <span className="font-medium">{response.description}</span>
                          </div>
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => toggleResponse(endpoint.id, index)}
                          >
                            {showResponse[`${endpoint.id}-${index}`] ? (
                              <><EyeOff className="h-4 w-4 mr-1" /> Hide</>
                            ) : (
                              <><Eye className="h-4 w-4 mr-1" /> Show</>
                            )}
                          </Button>
                        </div>
                        {showResponse[`${endpoint.id}-${index}`] && response.example && (
                          <div className="p-4 bg-gray-900 text-gray-100">
                            <pre className="text-sm overflow-x-auto">
                              {JSON.stringify(response.example, null, 2)}
                            </pre>
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                </div>

                {/* Examples */}
                <div>
                  <h3 className="font-semibold mb-3">Examples</h3>
                  <Tabs defaultValue={endpoint.examples[0]?.language} className="w-full">
                    <TabsList className="grid w-full grid-cols-4">
                      {endpoint.examples.map(example => (
                        <TabsTrigger key={example.language} value={example.language}>
                          {example.language}
                        </TabsTrigger>
                      ))}
                    </TabsList>
                    {endpoint.examples.map(example => (
                      <TabsContent key={example.language} value={example.language} className="mt-4">
                        <div className="border rounded-lg overflow-hidden">
                          <div className="flex items-center justify-between px-4 py-2 bg-gray-50 border-b">
                            <span className="font-medium">{example.title}</span>
                            <Button
                              size="sm"
                              variant="outline"
                              onClick={() => copyToClipboard(example.code, `${endpoint.id}-${example.language}`)}
                              className="h-7 px-2"
                            >
                              {copiedCode === `${endpoint.id}-${example.language}` ? (
                                <Check className="h-3 w-3" />
                              ) : (
                                <Copy className="h-3 w-3" />
                              )}
                            </Button>
                          </div>
                          <pre className="p-4 text-sm overflow-x-auto bg-gray-900 text-gray-100">
                            <code>{example.code}</code>
                          </pre>
                        </div>
                      </TabsContent>
                    ))}
                  </Tabs>
                </div>
              </CardContent>
            </Card>
          ))}

          {filteredEndpoints.length === 0 && (
            <Card>
              <CardContent className="py-12 text-center">
                <Code className="h-12 w-12 mx-auto text-gray-400 mb-4" />
                <h3 className="text-lg font-medium text-gray-900 mb-2">No endpoints found</h3>
                <p className="text-gray-600">
                  Try adjusting your search terms or category filter.
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
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <Button className="h-16 flex-col">
              <Send className="h-6 w-6 mb-2" />
              Test API
            </Button>
            <Button variant="outline" className="h-16 flex-col">
              <ExternalLink className="h-6 w-6 mb-2" />
              OpenAPI Spec
            </Button>
            <Button variant="outline" className="h-16 flex-col">
              <Code className="h-6 w-6 mb-2" />
              SDKs
            </Button>
            <Button variant="outline" className="h-16 flex-col">
              <Activity className="h-6 w-6 mb-2" />
              Status Page
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

export default ApiReference;