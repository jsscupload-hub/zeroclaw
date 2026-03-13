# ZeroClaw Code Analysis

ZeroClaw is a fast, small, and fully autonomous AI assistant infrastructure built in Rust. It abstracts models, tools, memory, and execution into a trait-driven architecture.

## 1. Project Overview

ZeroClaw follows a modular design where every major subsystem is defined by a Rust Trait. This allows for easy swapping of providers (AI models), channels (messaging platforms), tools (capabilities), and memory backends.

## 2. Core Modules and Files

### `src/main.rs` (CLI Entry Point)
The main execution entry point for the `zeroclaw` binary.
- **Role**: Parses command-line arguments using `clap` and routes them to the appropriate subsystem.
- **Key Functions**:
    - `main()`: Orchestrates initialization (logging, crypto, config loading) and command dispatching (`agent`, `onboard`, `gateway`, `daemon`, etc.).
    - `handle_estop_command()`: Manages emergency stop states.
    - `handle_auth_command()`: Handles provider authentication (OAuth, tokens).

### `src/lib.rs` (Library Root)
The library entry point that defines the public API and subcommands.
- **Role**: Re-exports core modules and defines the subcommand enums (`GatewayCommands`, `ChannelCommands`, `CronCommands`, etc.) used by both the binary and internal systems.

### `src/agent/` (AI Agent Logic)
Contains the core "brain" of the assistant.
- **`agent.rs`**: Defines the `Agent` struct and `AgentBuilder`.
    - `Agent::turn()`: Executes a single conversation turn (context loading -> LLM call -> tool execution loop -> final response).
    - `Agent::from_config()`: Factory to instantiate an agent with all necessary subsystems (observer, runtime, memory, tools).
- **`loop_.rs`**: Implements the iterative tool-calling loop.
    - `run_tool_call_loop()`: The heart of the agentic behavior. It sends messages to the LLM, parses tool calls from the response, executes them, and repeats until a final answer is produced.
    - `parse_tool_calls()`: Parses LLM responses for tool calls in various formats (XML tags, OpenAI JSON, GLM line-based).
    - `scrub_credentials()`: Redacts sensitive information (API keys, passwords) from tool outputs.

### `src/providers/` (AI Model Providers)
Abstraction layer for various LLM backends.
- **`mod.rs`**: Factory for model providers.
    - `create_provider()`: Instantiates a provider (OpenAI, Anthropic, Gemini, Ollama, etc.) based on configuration.
    - `resolve_provider_credential()`: Resolves API keys from config or environment variables.
    - `create_resilient_provider()`: Wraps providers with retry and fallback logic.
- **`traits.rs`**: Defines the `Provider` trait which all backends must implement.

### `src/tools/` (Agent Capabilities)
Registry of functions the agent can call.
- **`mod.rs`**: Assembles tool registries.
    - `all_tools_with_runtime()`: Returns a full set of tools (shell, file I/O, memory, cron, browser, etc.) initialized with security policies.
- **Individual Tool Files**: Each tool (e.g., `shell.rs`, `file_read.rs`, `memory_store.rs`) implements the `Tool` trait.

### `src/channels/` (Messaging Integrations)
Infrastructure for connecting to external platforms.
- **`mod.rs`**: Manages multi-channel messaging and per-sender history.
    - `start_channels()`: Starts all configured channel listeners (Telegram, Discord, Slack, etc.).
    - `process_channel_message()`: Orchestrates the end-to-end processing of an inbound message, including history management, tool summary extraction, and response delivery.
    - `build_system_prompt()`: Dynamically constructs the system prompt using workspace identity files and skill instructions.

### `src/memory/` (Search and Persistence)
The long-term and short-term memory system.
- **`mod.rs`**: Factory for creating memory backends.
    - `create_memory()`: Instantiates a backend (SQLite, Postgres, Lucid, Markdown, or None).
- **`sqlite.rs`**: Main persistent backend using vector search (via embeddings) and keyword search (FTS5).
- **`vector.rs`**: Implements cosine similarity search for vector embeddings.

### `src/security/` (Policy and Sandboxing)
Enforces safety and isolation.
- **`mod.rs`**: Defines security infrastructure.
    - `create_sandbox()`: Returns a pluggable sandbox (Docker, Firejail, Bubblewrap, etc.) for tool execution.
- **`policy.rs`**: Defines `SecurityPolicy`, which controls autonomy levels, workspace boundaries, and allowed commands.
- **`pairing.rs`**: Implements device pairing for the gateway server.

### `src/config/` (Configuration Management)
- **`schema.rs`**: Defines the `Config` struct and all nested configuration sections (agent, security, channels, etc.) using `serde`.

### `src/gateway/` (API Gateway)
- **`api.rs`**: Implements the HTTP/WebSocket gateway server for webhooks and pairing.

## 3. Key Function Summary

| Function | Location | Role |
| :--- | :--- | :--- |
| `run_tool_call_loop` | `agent/loop_.rs` | Core agentic iteration (LLM call <-> Tool execution). |
| `Agent::turn` | `agent/agent.rs` | Processes a single user message into a final agent response. |
| `process_channel_message` | `channels/mod.rs` | High-level handler for messages arriving from Telegram/Discord/etc. |
| `create_provider` | `providers/mod.rs` | Factory to create model backend instances. |
| `all_tools_with_runtime` | `tools/mod.rs` | Factory to create the agent's toolset with security enforced. |
| `create_memory` | `memory/mod.rs` | Factory to create memory persistence backends. |
| `parse_tool_calls` | `agent/loop_.rs` | Robust parser for extracting tool calls from LLM text. |
| `build_system_prompt` | `channels/mod.rs` | Composes the prompt from IDENTITY.md, SOUL.md, and skills. |
