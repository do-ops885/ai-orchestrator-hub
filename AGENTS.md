# Agent Development Guidelines for AI Orchestrator Hub

## Build/Lint/Test Commands

### Backend (Rust)

- **Build**: `cd backend && cargo build`
- **Test all**: `cd backend && cargo test`
- **Test single**: `cd backend && cargo test test_function_name`
- **Lint**: `cd backend && cargo clippy -- -W clippy::unwrap_used -W clippy::expect_used`
- **Format**: `cd backend && cargo fmt`
- **Security**: `./scripts/unwrap-prevention-monitor.sh check_unwrap_calls` (REQUIRED)

### Frontend (TypeScript/React)

- **Build**: `cd frontend && npm run build`
- **Test all**: `cd frontend && npm test`
- **Test single**: `cd frontend && npm test -- testName`
- **Lint**: `cd frontend && npm run lint`
- **Format**: `cd frontend && npm run format`

## Code Style Guidelines

### Rust (Backend)

- **Formatting**: rustfmt (100 width, 4 spaces, Unix line endings)
- **Naming**: snake_case (vars/fns), PascalCase (types), SCREAMING_SNAKE_CASE (consts)
- **Error handling**: Result<T,E> + Option<T>, avoid unwrap/panic, use `?` operator
- **Imports**: Group std → external crates → local modules
- **Types**: Explicit types preferred, use Arc/Rc for shared ownership
- **Async**: async fn + await, tokio runtime, async channels for agent communication

### TypeScript/React (Frontend)

- **Formatting**: ESLint + Prettier (single quotes, no semicolons, trailing commas, 2 spaces)
- **Naming**: camelCase (vars/fns), PascalCase (components/types)
- **Error handling**: try/catch, React error boundaries
- **Imports**: Path mapping (@/), group React → external → internal
- **Components**: Functional with hooks, composition over inheritance
- **State**: Zustand for global, local for components

**🚫 ZERO TOLERANCE for unwrap() calls in production Rust code**

## Design Principles

### SOLID Principles (Object-Oriented Design)

Apply these principles to maintain clean, maintainable, and extensible code:

- **Single Responsibility Principle (SRP)**: Each module, function, or class should have one reason to change. Keep components focused on a single purpose.
- **Open-Closed Principle (OCP)**: Software entities should be open for extension but closed for modification. Use interfaces and abstractions to allow extension without changing existing code.
- **Liskov Substitution Principle (LSP)**: Subtypes must be substitutable for their base types. Ensure derived classes can replace base classes without breaking functionality.
- **Interface Segregation Principle (ISP)**: Clients should not be forced to depend on interfaces they don't use. Create specific interfaces rather than general-purpose ones.
- **Dependency Inversion Principle (DIP)**: High-level modules should not depend on low-level modules. Both should depend on abstractions. Use dependency injection and inversion of control.

### KISS Principle (Keep It Simple, Stupid)

- Strive for simplicity in design and implementation
- Avoid unnecessary complexity and over-engineering
- Write code that is easy to understand, maintain, and debug
- Prefer straightforward solutions over clever or convoluted ones
- Regularly refactor to remove complexity as requirements evolve

**Remember**: Think step-by-step, analyze first, validate changes, no regressions, no false positive results, File size: ≤ 600 ine of code
