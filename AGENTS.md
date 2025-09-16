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