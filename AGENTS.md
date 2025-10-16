# Agent Guidelines for codecrafters-redis

## Build/Test Commands
- **Build**: `cargo build` or `./your_program.sh` (compiles and runs)
- **Run**: `./your_program.sh` (starts Redis server on 127.0.0.1:6379)
- **Test**: `cargo test` (runs all tests), `cargo test <test_name>` (runs single test)
- **Check**: `cargo check` (fast compilation check without building)

## Code Style
- **Edition**: Rust 2024
- **Imports**: Group std, external crates, then internal modules; use explicit imports
- **Formatting**: Standard rustfmt (4-space indents, no trailing commas for single items)
- **Types**: Use explicit types where clarity matters; prefer type inference in obvious cases
- **Naming**: snake_case for functions/variables, PascalCase for types, UPPER_CASE for constants
- **Error Handling**: Return Redis protocol error strings (e.g., `-ERR unknown command\r\n`); use `.unwrap()` for internal operations
- **Async**: All I/O operations use `tokio::async/await`; use `Arc<Mutex<T>>` for shared state across tasks
- **Patterns**: Match expressions with early returns; use `let-else` for simple unwrapping (`let Some(x) = y else { return }`); prefer immutable by default

## Architecture
- RESP protocol parser in `resp_parser.rs`, command handler in `command.rs`, data storage in `data/store.rs`
- Store uses `DashMap` for concurrent access and `tokio::sync` primitives for async coordination
- Commands return Redis protocol formatted strings (simple strings `+OK\r\n`, errors `-ERR\r\n`, integers `:<num>\r\n`, nil `$-1\r\n`)
