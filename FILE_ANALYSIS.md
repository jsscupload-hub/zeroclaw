# ZeroClaw File-by-File Analysis

This document provides a detailed breakdown of the ZeroClaw source code, organized by directory and file.

---

## 1. Root Directory (`src/`)

### `src/main.rs`
- **Purpose**: The main CLI entry point.
- **Key Logic**: 
    - Uses `clap` to define and parse the command-line interface.
    - Routes user commands to respective modules (`onboard`, `agent`, `gateway`, `daemon`, `status`, `estop`, `cron`, `models`, `providers`, `channel`, `integrations`, `skills`, `migrate`, `auth`, `hardware`, `peripheral`, `config`, `completions`).
    - Handles high-level initialization such as logging and global crypto setup.

### `src/lib.rs`
- **Purpose**: The library root for ZeroClaw.
- **Key Logic**: 
    - Exports all public submodules.
    - Defines shared subcommand enums (e.g., `GatewayCommands`, `ChannelCommands`) used by both the CLI and the daemon.

### `src/identity.rs`
- **Purpose**: Manages AI assistant personas.
- **Key Logic**: 
    - Supports two formats: **OpenClaw** (Markdown files like `SOUL.md`) and **AIEOS** (JSON-based specification).
    - Functions: `load_aieos_identity`, `aieos_to_system_prompt`.

### `src/multimodal.rs`
- **Purpose**: Handles vision and multimodal inputs.
- **Key Logic**: 
    - Defines `[IMAGE:<path>]` marker syntax.
    - Handles image normalization (local file reading, remote URL fetching) and base64 encoding for LLM providers.
    - Functions: `parse_image_markers`, `prepare_messages_for_provider`.

### `src/util.rs`
- **Purpose**: Shared utility functions.
- **Key Logic**: 
    - String manipulation, path helpers, and generic helpers used across the project.

---

## 2. Agent Subsystem (`src/agent/`)

### `src/agent/mod.rs`
- **Purpose**: Module entry point for agent logic.

### `src/agent/agent.rs`
- **Purpose**: Defines the core `Agent` struct.
- **Key Logic**: 
    - Orchestrates the full conversation lifecycle.
    - `Agent::turn()`: The main method to process a user message. It loads context, calls the LLM, enters the tool execution loop, and returns the final answer.

### `src/agent/loop_.rs`
- **Purpose**: Implements the iterative "Think-Act-Observe" loop.
- **Key Logic**: 
    - `run_tool_call_loop()`: Iterates until the model produces a text-only response or hits a limit.
    - `parse_tool_calls()`: A robust multi-format parser for extracting tool calls from model output (XML, JSON, GLM-style).
    - `scrub_credentials()`: Security function to redact sensitive info from tool logs.

### `src/agent/classifier.rs`
- **Purpose**: Query classification for model routing.
- **Key Logic**: 
    - Matches user messages against rules (keywords, patterns, length) to "hint" which model or provider should handle the request.

### `src/agent/dispatcher.rs`
- **Purpose**: Handles different tool-calling protocols.
- **Key Logic**: 
    - `XmlToolDispatcher`: For models that use XML tags for tools (e.g., Anthropic, DeepSeek).
    - `NativeToolDispatcher`: For models with native tool-calling APIs (e.g., OpenAI).

### `src/agent/prompt.rs`
- **Purpose**: Dynamic system prompt construction.
- **Key Logic**: 
    - `SystemPromptBuilder`: Assembles sections like Identity, Tools, Safety, Skills, and Runtime Info into a cohesive prompt.

### `src/agent/memory_loader.rs`
- **Purpose**: Semantic context injection.
- **Key Logic**: 
    - Searches memory for relevant snippets based on the user's current query and injects them into the prompt.

---

## 3. Providers Subsystem (`src/providers/`)

### `src/providers/mod.rs`
- **Purpose**: Factory for creating model backend instances.
- **Key Logic**: 
    - `create_provider()`: Instantiates backends for OpenAI, Anthropic, Gemini, Ollama, DeepSeek, etc.
    - Handles provider-specific environment variables and API key resolution.

### `src/providers/traits.rs`
- **Purpose**: Defines the `Provider` trait.
- **Key Logic**: 
    - Defines standard methods like `chat()`, `chat_with_history()`, and `warmup()`.

### `src/providers/reliable.rs`
- **Purpose**: Resilience wrapper.
- **Key Logic**: 
    - Implements retries, backoff, and fallback provider chains (if OpenAI fails, try Anthropic).

---

## 4. Tools Subsystem (`src/tools/`)

### `src/tools/mod.rs`
- **Purpose**: Tool registry and initialization.
- **Key Logic**: 
    - `all_tools_with_runtime()`: Returns the full suite of available tools (shell, file, memory, cron, browser, git, etc.).

### `src/tools/traits.rs`
- **Purpose**: Defines the `Tool` trait.
- **Key Logic**: 
    - Every tool must define a `name`, `description`, `parameters_schema`, and an `execute` function.

### Individual Tools (Examples):
- `shell.rs`: Executes terminal commands.
- `file_read.rs` / `file_write.rs`: Secure file I/O.
- `cron_add.rs` / `schedule.rs`: Task scheduling.
- `browser.rs`: Web browsing and automation.

---

## 5. Channels Subsystem (`src/channels/`)

### `src/channels/mod.rs`
- **Purpose**: Multi-channel messaging infrastructure.
- **Key Logic**: 
    - `start_channels()`: Spawns listeners for all configured platforms.
    - `process_channel_message()`: Handles incoming messages, maintains per-sender history, and delivers agent responses.

### Individual Channels:
- `telegram.rs`, `discord.rs`, `slack.rs`, `matrix.rs`, `whatsapp.rs`, etc.

---

## 6. Memory Subsystem (`src/memory/`)

### `src/memory/mod.rs`
- **Purpose**: Factory for memory backends.
- **Key Logic**: 
    - `create_memory()`: Creates backends for SQLite, Postgres, Lucid, or Markdown.

### `src/memory/sqlite.rs`
- **Purpose**: Main hybrid search engine.
- **Key Logic**: 
    - Uses vector embeddings for semantic search and FTS5 for keyword search.

### `src/memory/vector.rs`
- **Purpose**: Vector operations.
- **Key Logic**: 
    - Implements cosine similarity search for finding related memories.

---

## 7. Security Subsystem (`src/security/`)

### `src/security/mod.rs`
- **Purpose**: Security entry point.

### `src/security/policy.rs`
- **Purpose**: Defines `SecurityPolicy`.
- **Key Logic**: 
    - Enforces autonomy levels (`readonly`, `supervised`, `full`).
    - Validates paths (workspace scoping) and allowed shell commands.

### `src/security/detect.rs`
- **Purpose**: Sandbox detection.
- **Key Logic**: 
    - Detects available isolation mechanisms (Docker, Firejail, Bubblewrap, Landlock).

### `src/security/pairing.rs`
- **Purpose**: Device pairing.
- **Key Logic**: 
    - Implements the one-time code handshake for gateway authentication.

### `src/security/secrets.rs`
- **Purpose**: Encrypted storage.
- **Key Logic**: 
    - Manages local encryption keys to store API tokens securely on disk.

---

## 8. Gateway Subsystem (`src/gateway/`)

### `src/gateway/mod.rs`
- **Purpose**: Axum-based HTTP server.
- **Key Logic**: 
    - Implements rate limiting, request timeouts, and idempotency checks.

### `src/gateway/api.rs`
- **Purpose**: REST API endpoints.
- **Key Logic**: 
    - Handlers for pairing, webhooks, and dashboard data (status, config, tools, cron).

### `src/gateway/ws.rs`
- **Purpose**: WebSocket support.
- **Key Logic**: 
    - Real-time agent chat over WebSockets.

---

## 9. Configuration (`src/config/`)

### `src/config/schema.rs`
- **Purpose**: Central configuration definition.
- **Key Logic**: 
    - Defines the `Config` struct and sub-configs for every system.
    - Handles loading from and saving to `config.toml`.

---

## 10. Other Core Systems

### `src/cron/`
- **Purpose**: Job scheduling.
- **Key Logic**: 
    - `scheduler.rs`: A background worker that runs due tasks.
    - `store.rs`: Persists jobs and run history to disk.

### `src/auth/`
- **Purpose**: Authentication services.
- **Key Logic**: 
    - `openai_oauth.rs` / `gemini_oauth.rs`: Implements OAuth2 flows for subscription-based access.

### `src/observability/`
- **Purpose**: Telemetry and Monitoring.
- **Key Logic**: 
    - `prometheus.rs`: Metrics for scraping.
    - `otel.rs`: OpenTelemetry integration.
    - `log.rs`: Structured logging.

### `src/runtime/`
- **Purpose**: Tool execution environments.
- **Key Logic**: 
    - `native.rs`: Runs tools directly on the host.
    - `docker.rs`: Runs tools inside a Docker container for isolation.

### `src/skills/`
- **Purpose**: Custom capability loader.
- **Key Logic**: 
    - Loads user-defined skill directories containing `SKILL.md` or `SKILL.toml`.
    - `audit.rs`: Static security analysis of skills before loading.
