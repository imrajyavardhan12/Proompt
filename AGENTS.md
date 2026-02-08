# AGENTS.md - Proompt

## Project Overview

Proompt is a universal prompt enhancement tool delivered as a System Tray application (Tauri) with an accompanying CLI. It transforms rough prompts into well-crafted, platform-optimized prompts for AI assistants and image generators.

## Repository Structure

```
proompt/
├── Cargo.toml                 # Workspace root (members: core, cli, app/src-tauri)
├── crates/
│   ├── core/                  # Shared Rust library (proompt-core)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config/        # TOML config + OS keychain + env var API keys
│   │       │   ├── mod.rs     # Config structs (Config, ByokConfig, etc.)
│   │       │   └── manager.rs # load_config, save_config, get_api_key, set_api_key
│   │       ├── enhance/       # Text & image enhancement engines
│   │       │   ├── mod.rs     # enhance() and enhance_stream() entry points
│   │       │   ├── text.rs    # Platform-specific system prompts (Claude, GPT, Gemini, Generic)
│   │       │   └── image.rs   # Image system prompts (Midjourney, DALL-E, SD, Generic)
│   │       ├── integrations/
│   │       │   ├── llm/
│   │       │   │   ├── mod.rs       # LlmRequest, LlmResponse types
│   │       │   │   ├── anthropic.rs # Claude API client
│   │       │   │   ├── openai.rs    # OpenAI client (supports streaming via SSE)
│   │       │   │   └── google.rs    # Gemini API client
│   │       │   └── supermemory.rs   # SuperMemory context retrieval
│   │       ├── platform/
│   │       │   ├── mod.rs     # Platform enum, EnhanceType enum
│   │       │   └── detector.rs # detect_platform("claude") → Platform::Claude
│   │       ├── templates/
│   │       │   ├── mod.rs     # Template, TemplateField structs
│   │       │   ├── manager.rs # TemplateManager, sync_templates_from_remote()
│   │       │   └── builtin.rs # 10 hardcoded viral templates
│   │       └── usage/
│   │           └── tracker.rs # UsageStats, daily_limit_for_tier()
│   └── cli/                   # CLI binary (proompt-cli, binary name: proompt)
│       └── src/
│           ├── main.rs        # Clap CLI with subcommands
│           ├── commands/
│           │   ├── enhance.rs # Main enhance flow (spinner → stream/fetch → display)
│           │   ├── config.rs  # config show/set with styled output
│           │   └── template.rs # templates list/sync/apply
│           └── output.rs      # Styled output: banner, spinner, section_header, etc.
├── app/                       # Tauri desktop application
│   ├── src-tauri/
│   │   ├── src/
│   │   │   ├── main.rs        # Tauri setup, plugins, handler registration
│   │   │   └── commands.rs    # Tauri IPC commands bridging UI to core
│   │   ├── tauri.conf.json    # Uses bun for frontend build
│   │   └── Cargo.toml
│   ├── src/                   # Svelte 5 frontend
│   │   ├── App.svelte         # Root with tabs: Enhance, Templates, Settings
│   │   └── lib/components/
│   │       ├── EnhancePanel.svelte   # Text/image mode, platform selector, enhance button
│   │       ├── TemplatesPanel.svelte # Grid of templates, field input, generate
│   │       └── SettingsPanel.svelte  # Mode, API keys, provider, SuperMemory toggle
│   ├── package.json           # Uses bun (not pnpm)
│   └── vite.config.ts
├── backend/                   # Supabase (hosted mode, not yet implemented)
├── templates/                 # External viral templates (JSON, future CDN sync)
├── PRD.md                     # Full product requirements document (~3500 lines)
├── GETTING_STARTED.md         # Guide for devs new to Rust (maps web dev concepts)
└── .gitignore
```

## Tech Stack

| Component       | Technology                              |
|-----------------|-----------------------------------------|
| Core Engine     | Rust (shared library crate)             |
| CLI             | Rust + clap + indicatif + console       |
| Desktop App     | Tauri 2.0 + Svelte 5 + Vite            |
| Package Manager | bun (frontend), cargo (Rust)            |
| LLM Providers   | OpenAI (default), Anthropic, Google     |
| Config          | TOML + OS keychain                      |
| Backend         | Supabase (planned)                      |
| Payments        | Stripe (planned)                        |

## Build & Run

### Prerequisites

- Rust (stable, 1.75+)
- Node.js (18+)
- bun (1.0+)
- Tauri CLI: `cargo install tauri-cli --version "^2"` or `bun add -g @tauri-apps/cli`

### CLI

```bash
cargo build --release --package proompt-cli
./target/release/proompt "your prompt"
./target/release/proompt --help
```

### Desktop App

```bash
cd app
bun install
bunx tauri dev        # Development with hot-reload
bunx tauri build      # Production build
```

### Tests

```bash
cargo test                            # All tests (19 passing)
cargo test --package proompt-core     # Core library only
```

## API Key Configuration

API keys are stored in the **OS Keychain** (macOS Keychain, Windows Credential Manager, Linux Secret Service).

```bash
proompt config set byok.api_key YOUR_KEY          # stores under active provider
proompt config set openai.api_key YOUR_KEY         # stores for specific provider
proompt config set anthropic.api_key YOUR_KEY
proompt config set google.api_key YOUR_KEY
```

macOS Keychain will prompt for password on first access by a new binary. Click "Always Allow" and it won't ask again. This is a one-time macOS security approval.

Environment variables (`OPENAI_API_KEY`, etc.) are supported as a fallback but the primary method is keychain.

## Provider Switching

```bash
proompt config set byok.provider openai       # switches to OpenAI, auto-sets model to gpt-4o
proompt config set byok.provider anthropic    # switches to Anthropic, auto-sets model to claude-sonnet-4
proompt config set byok.provider google       # switches to Gemini, auto-sets model to gemini-2.0-flash
proompt config set byok.model gpt-4o-mini     # override model manually
```

## Architecture

The core engine is a Rust library crate (`proompt-core`) shared by both the CLI and the Tauri app. Both interfaces are thin wrappers that call into the core for all logic.

### Data Flow

```
User Input → CLI/Tauri → Core Engine → enhance() or enhance_stream()
                                           │
                              ┌─────────────┼─────────────┐
                              ▼             ▼             ▼
                          OpenAI API   Anthropic API   Gemini API
                          (streaming)   (batch)        (batch)
                              │             │             │
                              └─────────────┼─────────────┘
                                            ▼
                                    Enhanced Prompt → User
```

### Key Design Decisions

- **Monorepo with Cargo workspace**: Core, CLI, and Tauri app share types and logic.
- **BYOK first**: API keys in env vars or OS keychain, prompts go directly to LLM provider.
- **Multi-provider**: OpenAI (default), Anthropic, Google/Gemini. Provider routed in `enhance/mod.rs`.
- **Streaming**: OpenAI supports SSE streaming in CLI (tokens appear as they arrive). Others use batch.
- **Platform-specific prompts**: Each target (Claude, GPT, Gemini, Midjourney, DALL-E, SD) has its own system prompt in `enhance/text.rs` and `enhance/image.rs`.
- **Config**: TOML at `~/.config/proompt/config.toml`, defaults applied for empty/stale values in `load_config()`.

### Important Implementation Notes

- `ByokConfig::Default` is manually implemented (not derived) to ensure `provider = "openai"` and `model = "gpt-4o"` defaults. Using `#[derive(Default)]` would give empty strings.
- `get_api_key()` checks env vars first, then keychain. This avoids macOS Keychain password prompts.
- The `enhance()` function takes a `provider: &str` parameter to route to the correct LLM client.
- CLI uses `indicatif` for spinners and `console` for styled/colored output.
- Templates are loaded from builtins + cached remote JSON. `sync_templates_from_remote()` fetches and caches.

## Coding Conventions

- Rust edition 2024, stable toolchain
- Error handling: `anyhow::Result` for application code, `thiserror` for library error types
- Async runtime: Tokio
- HTTP client: reqwest (with `stream` feature for SSE)
- Config: serde + toml
- CLI parsing: clap with derive macros
- CLI output: `indicatif` (spinners), `console` (colors/styles)
- Frontend: Svelte 5 runes syntax (`$state`, `$derived`, `$effect`)
- Tauri IPC: `#[tauri::command]` async functions in `commands.rs`
- All output to user goes through `crate::output::*` helpers (never raw `println!` for UI chrome)
- Enhanced prompt content goes to stdout (for piping), UI chrome goes to stderr

## CLI Commands Reference

```bash
proompt                                       # Show banner + usage guide
proompt "prompt"                              # Enhance text (streams on OpenAI)
proompt --image "idea"                        # Enhance image prompt
proompt --platform claude "prompt"            # Target specific platform
proompt --template ghibli-style "subject"     # Apply viral template
proompt --memory "prompt"                     # Include SuperMemory context
proompt config show                           # Styled config display
proompt config set <key> <value>              # Update config or store API key
proompt templates list                        # Browse templates with badges
proompt templates list --trending             # Trending only
proompt templates sync                        # Fetch remote templates
proompt --help                                # Full help
```

## Current Status

- **Milestone 1 (Foundation)**: Complete
  - Core engine, config, Anthropic client, text/image enhancement, templates, CLI, Tauri app shell
- **Milestone 2 (Core Features)**: Complete
  - Google/Gemini LLM client, improved system prompts, changes summary, template sync, multi-provider config, CLI polish
  - Streaming output for OpenAI, animated spinners, colored sections, startup banner
  - Env var support for API keys (OPENAI_API_KEY, etc.)
  - 19 unit tests passing, 0 warnings
- **Milestone 3 (Integrations)**: Not started
  - SuperMemory end-to-end, Supabase auth, Stripe billing, hosted mode
- **Milestone 4 (Polish & Release)**: Not started
  - E2E testing, CI pipeline, code signing, Homebrew formula, release builds

## What to Work on Next

Per the PRD (Section 12), the next milestones are:
1. **Desktop App** - Get `bunx tauri dev` running, test the UI end-to-end
2. **Milestone 3** - Supabase backend, auth flow, Stripe billing, SuperMemory integration
3. **Milestone 4** - CI/CD, testing, code signing, distribution (Homebrew, .dmg, .exe)

See `PRD.md` for the full product requirements and `GETTING_STARTED.md` for local dev setup.
