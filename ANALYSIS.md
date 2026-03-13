# ZeroClaw Code Analysis (ZeroClaw 코드 분석)

ZeroClaw is a fast, small, and fully autonomous AI assistant infrastructure built in Rust. It abstracts models, tools, memory, and execution into a trait-driven architecture.
ZeroClaw는 Rust로 작성된 빠르고 가벼우며 완전 자율적인 AI 어시스턴트 인프라입니다. 모델, 도구, 메모리 및 실행을 Trait 기반 아키텍처로 추상화합니다.

## 1. Project Overview (프로젝트 개요)

ZeroClaw follows a modular design where every major subsystem is defined by a Rust Trait. This allows for easy swapping of providers (AI models), channels (messaging platforms), tools (capabilities), and memory backends.
ZeroClaw는 모든 주요 하위 시스템이 Rust Trait로 정의된 모듈형 설계를 따릅니다. 이를 통해 제공자(AI 모델), 채널(메시징 플랫폼), 도구(기능) 및 메모리 백엔드를 쉽게 교체할 수 있습니다.

## 2. Core Modules and Files (주요 모듈 및 파일)

### `src/main.rs` (CLI Entry Point / CLI 진입점)
The main execution entry point for the `zeroclaw` binary.
`zeroclaw` 바이너리의 주요 실행 진입점입니다.
- **Role (역할)**: Parses command-line arguments using `clap` and routes them to the appropriate subsystem.
  `clap`을 사용하여 명령줄 인수를 구문 분석하고 적절한 하위 시스템으로 라우팅합니다.
- **Key Functions (주요 함수)**:
    - `main()`: Orchestrates initialization (logging, crypto, config loading) and command dispatching (`agent`, `onboard`, `gateway`, `daemon`, etc.).
      초기화(로깅, 암호화, 설정 로드) 및 명령 디스패칭(`agent`, `onboard`, `gateway`, `daemon` 등)을 조율합니다.
    - `handle_estop_command()`: Manages emergency stop states.
      비상 정지(emergency stop) 상태를 관리합니다.
    - `handle_auth_command()`: Handles provider authentication (OAuth, tokens).
      제공자 인증(OAuth, 토큰)을 처리합니다.

### `src/lib.rs` (Library Root / 라이브러리 루트)
The library entry point that defines the public API and subcommands.
공용 API와 하위 명령을 정의하는 라이브러리 진입점입니다.
- **Role (역할)**: Re-exports core modules and defines the subcommand enums (`GatewayCommands`, `ChannelCommands`, `CronCommands`, etc.) used by both the binary and internal systems.
  핵심 모듈을 다시 내보내고 바이너리와 내부 시스템 모두에서 사용되는 하위 명령 열거형(`GatewayCommands`, `ChannelCommands`, `CronCommands` 등)을 정의합니다.

### `src/agent/` (AI Agent Logic / AI 에이전트 로직)
Contains the core "brain" of the assistant.
어시스턴트의 핵심 "두뇌"를 포함합니다.
- **`agent.rs`**: Defines the `Agent` struct and `AgentBuilder`.
  `Agent` 구조체와 `AgentBuilder`를 정의합니다.
    - `Agent::turn()`: Executes a single conversation turn (context loading -> LLM call -> tool execution loop -> final response).
      단일 대화 턴을 실행합니다 (컨텍스트 로딩 -> LLM 호출 -> 도구 실행 루프 -> 최종 응답).
    - `Agent::from_config()`: Factory to instantiate an agent with all necessary subsystems (observer, runtime, memory, tools).
      필요한 모든 하위 시스템(관찰자, 런타임, 메모리, 도구)을 갖춘 에이전트를 인스턴스화하는 팩토리 함수입니다.
- **`loop_.rs`**: Implements the iterative tool-calling loop.
  반복적인 도구 호출 루프를 구현합니다.
    - `run_tool_call_loop()`: The heart of the agentic behavior. It sends messages to the LLM, parses tool calls from the response, executes them, and repeats until a final answer is produced.
      에이전트 동작의 핵심입니다. LLM에 메시지를 보내고, 응답에서 도구 호출을 구문 분석하고, 이를 실행하며, 최종 답변이 생성될 때까지 반복합니다.
    - `parse_tool_calls()`: Parses LLM responses for tool calls in various formats (XML tags, OpenAI JSON, GLM line-based).
      다양한 형식(XML 태그, OpenAI JSON, GLM 라인 기반)의 도구 호출에 대해 LLM 응답을 구문 분석합니다.
    - `scrub_credentials()`: Redacts sensitive information (API keys, passwords) from tool outputs.
      도구 출력에서 민감한 정보(API 키, 비밀번호)를 가립니다.

### `src/providers/` (AI Model Providers / AI 모델 제공자)
Abstraction layer for various LLM backends.
다양한 LLM 백엔드를 위한 추상화 계층입니다.
- **`mod.rs`**: Factory for model providers.
  모델 제공자를 위한 팩토리입니다.
    - `create_provider()`: Instantiates a provider (OpenAI, Anthropic, Gemini, Ollama, etc.) based on configuration.
      설정에 따라 제공자(OpenAI, Anthropic, Gemini, Ollama 등)를 인스턴스화합니다.
    - `resolve_provider_credential()`: Resolves API keys from config or environment variables.
      설정 또는 환경 변수에서 API 키를 확인합니다.
    - `create_resilient_provider()`: Wraps providers with retry and fallback logic.
      재시도 및 폴백 로직으로 제공자를 래핑합니다.
- **`traits.rs`**: Defines the `Provider` trait which all backends must implement.
  모든 백엔드가 구현해야 하는 `Provider` Trait을 정의합니다.

### `src/tools/` (Agent Capabilities / 에이전트 기능)
Registry of functions the agent can call.
에이전트가 호출할 수 있는 함수들의 레지스트리입니다.
- **`mod.rs`**: Assembles tool registries.
  도구 레지스트리를 조립합니다.
    - `all_tools_with_runtime()`: Returns a full set of tools (shell, file I/O, memory, cron, browser, etc.) initialized with security policies.
      보안 정책으로 초기화된 전체 도구 세트(셸, 파일 I/O, 메모리, 크론, 브라우저 등)를 반환합니다.
- **Individual Tool Files (개별 도구 파일)**: Each tool (e.g., `shell.rs`, `file_read.rs`, `memory_store.rs`) implements the `Tool` trait.
  각 도구(예: `shell.rs`, `file_read.rs`, `memory_store.rs`)는 `Tool` Trait을 구현합니다.

### `src/channels/` (Messaging Integrations / 메시징 통합)
Infrastructure for connecting to external platforms.
외부 플랫폼 연결을 위한 인프라입니다.
- **`mod.rs`**: Manages multi-channel messaging and per-sender history.
  다중 채널 메시징 및 발신자별 이력을 관리합니다.
    - `start_channels()`: Starts all configured channel listeners (Telegram, Discord, Slack, etc.).
      설정된 모든 채널 리스너(Telegram, Discord, Slack 등)를 시작합니다.
    - `process_channel_message()`: Orchestrates the end-to-end processing of an inbound message, including history management, tool summary extraction, and response delivery.
      이력 관리, 도구 요약 추출 및 응답 전달을 포함하여 수신 메시지의 엔드투엔드 처리를 조율합니다.
    - `build_system_prompt()`: Dynamically constructs the system prompt using workspace identity files and skill instructions.
      워크스페이스 ID 파일과 스킬 지침을 사용하여 시스템 프롬프트를 동적으로 구성합니다.

### `src/memory/` (Search and Persistence / 검색 및 지속성)
The long-term and short-term memory system.
장기 및 단기 메모리 시스템입니다.
- **`mod.rs`**: Factory for creating memory backends.
  메모리 백엔드 생성을 위한 팩토리입니다.
    - `create_memory()`: Instantiates a backend (SQLite, Postgres, Lucid, Markdown, or None).
      백엔드(SQLite, Postgres, Lucid, Markdown 또는 None)를 인스턴스화합니다.
- **`sqlite.rs`**: Main persistent backend using vector search (via embeddings) and keyword search (FTS5).
  벡터 검색(임베딩 사용) 및 키워드 검색(FTS5)을 사용하는 주요 지속성 백엔드입니다.
- **`vector.rs`**: Implements cosine similarity search for vector embeddings.
  벡터 임베딩에 대한 코사인 유사도 검색을 구현합니다.

### `src/security/` (Policy and Sandboxing / 정책 및 샌드박싱)
Enforces safety and isolation.
안전 및 격리를 강제합니다.
- **`mod.rs`**: Defines security infrastructure.
  보안 인프라를 정의합니다.
    - `create_sandbox()`: Returns a pluggable sandbox (Docker, Firejail, Bubblewrap, etc.) for tool execution.
      도구 실행을 위한 플러그형 샌드박스(Docker, Firejail, Bubblewrap 등)를 반환합니다.
- **`policy.rs`**: Defines `SecurityPolicy`, which controls autonomy levels, workspace boundaries, and allowed commands.
  자율성 수준, 워크스페이스 경계 및 허용된 명령을 제어하는 `SecurityPolicy`를 정의합니다.
- **`pairing.rs`**: Implements device pairing for the gateway server.
  게이트웨이 서버를 위한 장치 페어링을 구현합니다.

### `src/config/` (Configuration Management / 설정 관리)
- **`schema.rs`**: Defines the `Config` struct and all nested configuration sections (agent, security, channels, etc.) using `serde`.
  `serde`를 사용하여 `Config` 구조체와 모든 중첩된 설정 섹션(에이전트, 보안, 채널 등)을 정의합니다.

### `src/gateway/` (API Gateway / API 게이트웨이)
- **`api.rs`**: Implements the HTTP/WebSocket gateway server for webhooks and pairing.
  웹훅 및 페어링을 위한 HTTP/WebSocket 게이트웨이 서버를 구현합니다.

## 3. Key Function Summary (주요 함수 요약)

| Function (함수) | Location (위치) | Role (역할) |
| :--- | :--- | :--- |
| `run_tool_call_loop` | `agent/loop_.rs` | Core agentic iteration (LLM call <-> Tool execution). <br>핵심 에이전트 반복 (LLM 호출 <-> 도구 실행). |
| `Agent::turn` | `agent/agent.rs` | Processes a single user message into a final agent response. <br>단일 사용자 메시지를 최종 에이전트 응답으로 처리합니다. |
| `process_channel_message` | `channels/mod.rs` | High-level handler for messages arriving from Telegram/Discord/etc. <br>Telegram/Discord 등에서 도착하는 메시지에 대한 상위 수준 처리기입니다. |
| `create_provider` | `providers/mod.rs` | Factory to create model backend instances. <br>모델 백엔드 인스턴스를 생성하는 팩토리입니다. |
| `all_tools_with_runtime` | `tools/mod.rs` | Factory to create the agent's toolset with security enforced. <br>보안이 적용된 에이전트 도구 세트를 생성하는 팩토리입니다. |
| `create_memory` | `memory/mod.rs` | Factory to create memory persistence backends. <br>메모리 지속성 백엔드를 생성하는 팩토리입니다. |
| `parse_tool_calls` | `agent/loop_.rs` | Robust parser for extracting tool calls from LLM text. <br>LLM 텍스트에서 도구 호출을 추출하기 위한 견고한 파서입니다. |
| `build_system_prompt` | `channels/mod.rs` | Composes the prompt from IDENTITY.md, SOUL.md, and skills. <br>IDENTITY.md, SOUL.md 및 스킬에서 프롬프트를 구성합니다. |
