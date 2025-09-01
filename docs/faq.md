# Frequently Asked Questions

This document addresses common questions and issues users encounter with the Multiagent Hive System.

## General Questions

### What is the Multiagent Hive System?

The Multiagent Hive System is a sophisticated swarm intelligence platform that combines neural processing, real-time communication, and adaptive learning. It's designed to coordinate multiple AI agents working together to solve complex problems, with a focus on CPU-native performance that scales to GPU acceleration when available.

### What makes it different from other multiagent systems?

- **CPU-Native Design**: Maximum intelligence on minimal hardware, no GPU required
- **Hybrid Neural Processing**: Combines lightweight NLP with optional advanced neural networks
- **Real-time Coordination**: WebSocket-based communication for instant agent coordination
- **Adaptive Learning**: Agents learn and improve through experience
- **Extensible Architecture**: Easy to add new agent types and capabilities

### What are the system requirements?

**Minimum Requirements:**
- CPU: 2 cores, 2.5 GHz
- RAM: 2 GB
- Storage: 1 GB
- OS: Linux, macOS, Windows

**Recommended Requirements:**
- CPU: 4+ cores, 3.0 GHz
- RAM: 4 GB
- Storage: 2 GB
- OS: Linux (Ubuntu 20.04+)

### Is it free to use?

Yes, the Multiagent Hive System is open-source and free to use under the MIT License. You can download, modify, and distribute it freely.

## Installation and Setup

### I'm getting a "Rust not found" error. What should I do?

You need to install Rust first. Run this command:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

Then verify the installation:

```bash
rustc --version
cargo --version
```

### The frontend won't start. What's wrong?

Common issues:

1. **Node.js not installed**: Install Node.js 18+ from [nodejs.org](https://nodejs.org/)
2. **Dependencies not installed**: Run `npm install` in the frontend directory
3. **Port conflict**: Check if port 3000 is already in use
4. **Environment variables**: Ensure `.env.local` is properly configured

### How do I enable advanced neural features?

1. Install the advanced features during build:
```bash
cargo build --features advanced-neural
```

2. Or run with features:
```bash
cargo run --features advanced-neural
```

3. For GPU support:
```bash
cargo run --features advanced-neural,gpu-acceleration
```

### Can I run this on Windows?

Yes! The system supports Windows, but we recommend using WSL (Windows Subsystem for Linux) for the best experience. The native Windows build is also supported.

## Usage Questions

### How do I create my first agent?

1. Start the system (backend and frontend)
2. Open the web interface at `http://localhost:3000`
3. Click "Create Agent" in the Agent Manager
4. Fill in:
   - Name: Choose a descriptive name
   - Type: Select Worker, Coordinator, or Specialist
   - Capabilities: Add relevant skills with proficiency levels
5. Click "Create"

### What agent types are available?

- **Worker**: General-purpose task execution
- **Coordinator**: Manages task distribution and swarm coordination
- **Specialist**: Domain-specific expertise (e.g., data analysis, NLP)
- **Learner**: Focuses on learning and adaptation

### How do tasks get assigned to agents?

Tasks are assigned based on:
1. **Capability Matching**: Agents must have required capabilities
2. **Proficiency Levels**: Agents with higher proficiency get priority
3. **Workload Balancing**: Less busy agents are preferred
4. **Priority Levels**: Critical tasks get immediate assignment

### What's the difference between basic and advanced neural processing?

- **Basic**: Lightweight CPU processing, fast, low resource usage
- **Advanced**: FANN neural networks, more accurate but requires more resources
- **GPU**: Hardware acceleration for advanced processing

### How do I monitor system performance?

The system provides multiple monitoring options:

1. **Web Dashboard**: Real-time metrics at `http://localhost:3000`
2. **Metrics API**: `/api/hive/metrics` for programmatic access
3. **Logs**: Check `backend/hive.log` for detailed logs
4. **Resource Monitor**: Built-in component for system resources

## Configuration

### How do I change the default ports?

Edit the backend configuration:

```env
# .env file in backend directory
HIVE_PORT=3001
# Frontend will be on 3000 by default
```

Or modify the settings file:

```toml
# backend/settings/default.toml
[server]
port = 3001
host = "localhost"
```

### Can I use a different database?

Yes, the system supports multiple databases:

- **SQLite** (default): Simple, file-based, good for development
- **PostgreSQL**: Production-ready, scalable
- **MySQL**: Alternative relational database

Configure in your `.env` file:

```env
DATABASE_URL=postgresql://user:password@localhost/hive_db
```

### How do I configure logging?

```env
# .env file
LOG_LEVEL=info  # debug, info, warn, error
LOG_FORMAT=json # json or plain text
LOG_FILE=hive.log
```

### What's the maximum number of agents I can run?

It depends on your hardware:
- **Basic setup**: 100-500 agents
- **Advanced setup**: 1000-5000 agents
- **High-end hardware**: 10,000+ agents

Monitor system resources and adjust `MAX_AGENTS` accordingly.

## Performance and Scaling

### Why is the system running slow?

Common causes:

1. **Resource constraints**: Check CPU, memory, and disk usage
2. **Too many agents**: Reduce `MAX_AGENTS` setting
3. **Database performance**: Optimize queries or upgrade database
4. **Network latency**: Check WebSocket connection quality
5. **Feature overload**: Disable advanced features if not needed

### How can I improve performance?

1. **Enable basic mode**: Use `NEURAL_MODE=basic` for faster processing
2. **Optimize database**: Add indexes, use connection pooling
3. **Increase resources**: More CPU cores and RAM
4. **Use caching**: Enable Redis for session storage
5. **Profile bottlenecks**: Use `cargo flamegraph` to identify slow code

### Can I run this in the cloud?

Yes! The system works well in cloud environments:

- **AWS**: ECS, EKS, or EC2 instances
- **Google Cloud**: Cloud Run, GKE, or Compute Engine
- **Azure**: Container Instances, AKS, or VMs
- **DigitalOcean**: Droplets or Kubernetes

### What's the best way to scale horizontally?

1. **Load balancer**: Distribute requests across multiple instances
2. **Database clustering**: Use managed database services
3. **Session storage**: Use Redis for shared sessions
4. **File storage**: Use cloud storage for persistent data
5. **Monitoring**: Centralized logging and metrics

## Troubleshooting

### The backend crashes on startup. What can I do?

1. **Check logs**: Look at `backend/hive.log` for error messages
2. **Verify dependencies**: Run `cargo check` to check for compilation errors
3. **Check configuration**: Validate `.env` and settings files
4. **Test database**: Ensure database is accessible
5. **Check ports**: Make sure required ports are available

### WebSocket connections are failing. Why?

Possible causes:

1. **Firewall**: Check if port 3001 is blocked
2. **CORS issues**: Verify CORS configuration
3. **Network proxy**: Some proxies don't support WebSocket
4. **SSL termination**: If using HTTPS, ensure proper WebSocket handling
5. **Browser issues**: Try a different browser or incognito mode

### Tasks are not being completed. What's wrong?

Check these:

1. **Agent availability**: Are there agents with required capabilities?
2. **Task queue**: Is the queue full? Check `TASK_QUEUE_SIZE`
3. **Agent status**: Are agents in "Working" or "Idle" state?
4. **Capability matching**: Do agents have the required proficiency levels?
5. **System resources**: Are agents running out of energy?

### I'm seeing "Connection refused" errors. How to fix?

1. **Service not running**: Start the backend service
2. **Wrong port**: Check if using correct port (default 3001)
3. **Firewall**: Allow connections on the required port
4. **Network issues**: Check if services are on the same network
5. **Docker networking**: If using containers, check network configuration

### Memory usage keeps growing. Is this normal?

Some growth is normal, but excessive growth indicates issues:

1. **Memory leaks**: Check for unclosed connections or resources
2. **Large datasets**: Reduce `MAX_AGENTS` or use pagination
3. **Debug mode**: Disable debug features in production
4. **Profile memory**: Use `valgrind` to identify leaks

## Development

### How do I add a new agent type?

1. **Define the agent type** in the code
2. **Add capabilities** specific to that type
3. **Update the UI** to support the new type
4. **Test thoroughly** with different scenarios

### Can I extend the API?

Yes! The system is designed to be extensible:

1. **Add new endpoints** in the backend
2. **Update OpenAPI spec** for documentation
3. **Add frontend components** for new features
4. **Write tests** for new functionality

### How do I contribute to the project?

1. **Fork the repository** on GitHub
2. **Create a feature branch**: `git checkout -b feature/my-feature`
3. **Make your changes** following the coding standards
4. **Write tests** for new functionality
5. **Submit a pull request** with a clear description

### What's the best way to debug the system?

1. **Enable debug logging**: `LOG_LEVEL=debug`
2. **Use breakpoints**: In your IDE for step-through debugging
3. **Monitor WebSocket**: Use browser dev tools to inspect messages
4. **Profile performance**: Use `cargo flamegraph` for CPU profiling
5. **Check metrics**: Use the `/api/hive/metrics` endpoint

## Security

### Is the system secure?

The system includes several security measures:

- **Input validation**: All inputs are validated and sanitized
- **Authentication**: Optional JWT-based authentication
- **HTTPS support**: TLS encryption for production
- **Rate limiting**: Protection against abuse
- **CORS configuration**: Proper cross-origin handling

### How do I enable authentication?

1. **Set environment variables**:
```env
API_KEY_REQUIRED=true
JWT_SECRET=your-secret-key-here
```

2. **Configure authentication provider** (optional)
3. **Update client code** to include authentication headers

### Can I run this behind a reverse proxy?

Yes! The system works well with reverse proxies like Nginx:

```nginx
server {
    listen 80;
    server_name your-domain.com;

    location /api/ {
        proxy_pass http://localhost:3001;
        proxy_set_header Host $host;
    }

    location /ws/ {
        proxy_pass http://localhost:3001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    location / {
        proxy_pass http://localhost:3000;
    }
}
```

## Advanced Features

### How do I use the MCP integration?

The Model Context Protocol allows external tools to interact with the hive:

1. **Enable MCP server**: Run `cargo run --bin mcp_server`
2. **Connect external tools**: Use MCP-compatible clients
3. **Available tools**:
   - `create_swarm_agent`: Create new agents
   - `assign_swarm_task`: Assign tasks to the swarm
   - `get_swarm_status`: Get current hive status
   - `analyze_with_nlp`: Perform NLP analysis

### What's the difference between the dashboard and API?

- **Dashboard**: User-friendly web interface for monitoring and control
- **API**: Programmatic access for automation and integration
- **WebSocket**: Real-time updates for both dashboard and API clients

### Can I export data from the system?

Yes, you can export various types of data:

1. **Metrics data**: Use `/api/hive/metrics` endpoint
2. **Agent data**: Use `/api/agents` with export format
3. **Task data**: Use `/api/tasks` with export format
4. **Logs**: Access log files directly

### How do I backup the system?

1. **Database backup**: Use your database's backup tools
2. **Configuration files**: Backup `.env` and settings files
3. **Log files**: Archive log files if needed
4. **Application data**: Backup any persistent data

## Getting Help

### Where can I find more documentation?

- **README.md**: Main project documentation
- **docs/**: Detailed guides and API documentation
- **GitHub Issues**: Search for similar problems
- **GitHub Discussions**: Community discussions

### How do I report a bug?

1. **Check existing issues** on GitHub
2. **Gather information**:
   - System information (OS, versions)
   - Steps to reproduce
   - Expected vs actual behavior
   - Log files and error messages
3. **Create a new issue** with the bug report template

### Is there a community or support forum?

- **GitHub Discussions**: For questions and discussions
- **GitHub Issues**: For bug reports and feature requests
- **Documentation**: Comprehensive guides in the `docs/` directory

### Can I get commercial support?

While the core system is open-source and free, commercial support options may be available through:
- Consulting services
- Enterprise support packages
- Custom development services

Contact the maintainers for more information.

## Contributing

### How can I help improve the project?

1. **Report bugs** and issues
2. **Suggest features** and improvements
3. **Submit pull requests** with fixes or enhancements
4. **Improve documentation**
5. **Help other users** in discussions

### What are the coding standards?

- **Rust**: Follow `rustfmt` and `clippy` recommendations
- **TypeScript**: Use ESLint configuration
- **Documentation**: Clear, concise, and comprehensive
- **Testing**: Write tests for all new functionality
- **Commits**: Use conventional commit format

### How do I set up a development environment?

1. **Clone the repository**
2. **Install dependencies** (Rust, Node.js)
3. **Set up pre-commit hooks** (optional)
4. **Configure IDE** with Rust and TypeScript support
5. **Run tests** to ensure everything works

## License and Legal

### What license is the project under?

The Multiagent Hive System is licensed under the MIT License, which allows:
- Free use for personal and commercial purposes
- Modification and distribution
- Private use without disclosing source code
- No warranty or liability from the authors

### Can I use this in commercial products?

Yes, the MIT License allows commercial use. However, you should:
- Include the original copyright notice
- Include the license text in your distribution
- Not use the authors' names for endorsement

### Are there any patents or IP restrictions?

The project is open-source under MIT License. There are no known patents specifically covering this implementation, but users should consult their own legal counsel for commercial use.

### How do I comply with the license?

To comply with the MIT License:
1. **Keep the copyright notice** in all copies
2. **Include the license text** in distributions
3. **Don't remove attribution** from the original work

## Future and Roadmap

### What's planned for future versions?

- **Enhanced AI capabilities**: Better neural processing
- **Cloud integration**: Improved cloud deployment options
- **Performance improvements**: Better scalability
- **New agent types**: More specialized agent roles
- **UI enhancements**: Improved user interface

### How can I stay updated?

- **Watch the repository** on GitHub for releases
- **Follow the maintainers** for announcements
- **Check the changelog** for detailed changes
- **Join discussions** for community updates

### Can I suggest features?

Yes! Feature requests are welcome:
1. **Check existing issues** for similar requests
2. **Use the feature request template**
3. **Provide detailed description** of the proposed feature
4. **Explain the use case** and benefits

## Still Need Help?

If you can't find the answer here:

1. **Search GitHub Issues** for similar problems
2. **Check the documentation** in the `docs/` directory
3. **Ask in GitHub Discussions**
4. **Create a new issue** if it's a bug or feature request

The community and maintainers are here to help!