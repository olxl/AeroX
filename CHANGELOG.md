# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- KCP transport protocol support
- QUIC transport protocol support
- WebSocket support
- Distributed architecture
- Performance optimizations

## [0.1.0] - 2025-12-27

### Added

#### Core Framework
- **aerox_core**: Core runtime and plugin system
  - `App` builder with plugin registration
  - `Plugin` trait with dependency management
  - Error handling with `AeroXError` and `Result`
  - State management system

#### Network Layer
- **aerox_network**: Network abstraction and protocol implementation
  - TCP transport with `TcpReactor`
  - Multi-threaded Reactor pattern with `Worker` threads
  - Connection management with `ConnectionId` and `ConnectionPool`
  - Message codec with `Frame` and `MessageCodec`
  - Zero-copy message handling with `bytes::Bytes`

#### Configuration
- **aerox_config**: Configuration management
  - TOML file loading
  - Environment variable override
  - Configuration validation
  - Default and chain configurations

#### Routing
- **aerox_router**: Routing and middleware system
  - `Router` with message handler registration
  - `Context` for request/response handling
  - Axum-style middleware system
  - Built-in `LoggingMiddleware` and `TimeoutMiddleware`

#### ECS Integration
- **aerox_ecs**: Bevy ECS integration layer
  - `EcsWorld` wrapper around Bevy World
  - Network event bridge with `NetworkBridge`
  - 7 network event types
  - 9 built-in components (PlayerConnection, Position, Health, etc.)
  - 7 example systems (connection management, position update, etc.)

#### Protobuf Support
- **aerox_protobuf**: Protocol Buffers support
  - `MessageRegistry` for message type management
  - Zero-copy encoding/decoding
  - Message wrapping/unwrapping
  - Build script for code generation

#### Plugins
- **aerox_plugins**: Built-in plugins
  - `HeartbeatPlugin` for connection heartbeat
  - `RateLimitPlugin` for rate limiting

### Examples

- **echo_server**: Echo server example
- **chat_room**: Multi-client chat room with commands
- **load_config_example**: Configuration loading example
- **env_override_example**: Environment variable override example
- **hello_plugin**: Plugin system example
- **protobuf_usage**: Protobuf encoding/decoding example

### Testing

- **Unit Tests**: 113 tests across all crates
- **Integration Tests**: 16 tests covering:
  - Configuration system (3 tests)
  - Network layer (2 tests)
  - ECS (5 tests)
  - Plugin system (2 tests)
  - Error handling (2 tests)
  - Router (2 tests)

### Performance

- **Benchmarks**: Performance benchmarks for:
  - ConnectionId generation
  - Frame operations
  - Message encoding/decoding
  - Router dispatch
  - ECS operations
  - Memory usage
  - Concurrent operations

### Documentation

- **README.md**: Project overview and quick start
- **docs/getting_started.md**: Comprehensive getting started guide
- **docs/architecture.md**: Architecture design documentation
- **docs/configuration.md**: Configuration reference
- **docs/plan_process.md**: Development progress tracking
- **API Documentation**: Full rustdoc coverage

### Statistics

- **Total Crates**: 7
- **Total Tests**: 129 (all passing ✅)
- **Code Coverage**: >80%
- **Completion**: 11/12 Phases (92%)

### Dependencies

- tokio 1.40 - Async runtime
- bevy 0.14 - ECS framework
- prost 0.13 - Protobuf support
- bytes 1.7 - Zero-copy byte buffer
- serde 1.0 - Serialization
- thiserror 2.0 - Error handling
- toml 0.8 - Config parsing

### Breaking Changes

None (initial release)

### Deprecated

None

### Removed

None

### Fixed

- Fixed plugin dependency resolution
- Fixed timer component naming conflict with Bevy
- Fixed test compilation errors

### Security

No security issues reported

## [0.0.1] - Unreleased

### Initial Development
- Project setup
- Workspace configuration
- Basic structure

---

[Unreleased]: https://github.com/aerox/aerox/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/aerox/aerox/releases/tag/v0.1.0
[0.0.1]: https://github.com/aerox/aerox/releases/tag/v0.0.1
