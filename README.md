# Proompt

A desktop app and CLI that turns rough prompts into execution-ready AI tasks for coding agents, AI assistants, and image generators.

You type "fix upload bug", and Proompt rewrites it into a well-structured prompt with context, constraints, acceptance criteria, and platform-specific formatting -- ready for Claude Code, Cursor, Codex, Claude, GPT, Gemini, Midjourney, DALL-E, or Stable Diffusion.

## How it works

```
your rough prompt  -->  Proompt  -->  platform-optimized prompt
```

Proompt sends your input through an LLM with carefully tuned system prompts that know the quirks of each target platform. Claude Code gets repo-investigation workflow and acceptance criteria. Cursor gets minimal-diff IDE guidance. GPT gets structured markdown. Midjourney gets style parameters and aspect ratios. You bring your own API key.

## Install

### Prerequisites

- Rust 1.75+ (`rustup` to install)
- Node.js 18+
- [bun](https://bun.sh) 1.0+

### CLI

```bash
cargo build --release
sudo cp ./target/release/proompt /usr/local/bin/

proompt config set byok.api_key "YOUR_OPENAI_KEY"
proompt "explain how docker works"
```

### Desktop app

```bash
cd app
bun install
bunx tauri build
```

The `.dmg` lands in `app/src-tauri/target/release/bundle/dmg/`. Open it and drag to Applications.

#### Unsigned macOS builds

Current macOS desktop releases are unsigned while we defer Apple Developer ID signing/notarization. On first launch, Gatekeeper may report that `Proompt.app` is "damaged". If you downloaded Proompt from the official GitHub release, remove the quarantine flag once:

```bash
xattr -dr com.apple.quarantine /Applications/Proompt.app
open /Applications/Proompt.app
```

For development with hot-reload: `bunx tauri dev`

## Usage

### Quick Enhance from anywhere

The fastest workflow is the global hotkey. Use the app once to set your **Quick Enhance fallback target** and optional active-app/terminal routing, then stay in your current tool:

```text
select or copy rough task -> press Cmd/Ctrl+Shift+E -> Proompt replaces the selection or copies the enhanced prompt
```

For coding agents, prefix clipboard text to override the target without opening Proompt:

```text
/cc fix upload bug                 -> Claude Code
/cursor add auth middleware        -> Cursor
/codex write billing edge tests    -> Codex
/agent refactor config loading     -> generic coding agent
/gpt make this clearer             -> GPT
/claude explain this tradeoff      -> Claude
```

Proompt strips the prefix, enhances for that target, replaces the selected text when possible, otherwise copies the result, and notifies which target was used and why. When auto-detect is enabled, Quick Enhance can route from active apps such as Cursor, ChatGPT, Claude, supported browser window titles, or a configured terminal default. Settings includes a local clipboard route preview, and History records actual hotkey routing decisions.

Selected-text capture/replacement on macOS requires Accessibility permission. Settings includes Selected-text diagnostics, a System Settings shortcut, and a copyable reset command. If an unsigned build appears enabled but stops capturing selections after an update/rebuild, reset permission with `tccutil reset Accessibility com.proompt.desktop`, relaunch the current `/Applications/Proompt.app`, and grant Accessibility again.

### CLI

```bash
# text prompt (streams on OpenAI)
proompt "explain kubernetes in simple terms"

# target a specific chat assistant
proompt --platform claude "sort users by age and filter inactive"

# compose a task for a coding agent
proompt --platform claude-code "fix upload bug"
proompt --platform cursor "add auth middleware"
proompt --platform codex "write tests for billing edge cases"
proompt --platform coding-agent "refactor config loading"

# image prompt
proompt --image "a cat floating in space"
proompt --image --platform midjourney "sunset over mountains"

# use a template
proompt --template ghibli-style "my cat on the couch"

# pipe from stdin
echo "explain rust ownership" | proompt

# configuration
proompt config show
proompt config set byok.provider anthropic
proompt history list
proompt templates list
```

### Desktop app

Four tabs: Enhance, History, Templates, Settings.

- Set your Quick Enhance target for the global hotkey
- Pick text or image mode, choose a target platform, type your prompt, hit Enhance when you want a visible workspace
- Review local prompt history, favorite useful prompts, copy enhanced output, or reuse originals
- Browse 10 built-in templates for common tasks (code review, Ghibli-style images, etc.)
- Settings has provider/model switching, API key management, local history controls, and connection testing

## Providers

| Provider | Streaming | Default model |
|----------|-----------|---------------|
| OpenAI | Yes (SSE) | gpt-4o |
| Anthropic | No (batch) | claude-sonnet-4 |
| Google | No (batch) | gemini-2.0-flash |
| OpenRouter | Yes (SSE) | openai/gpt-4o-mini |

Switch with `proompt config set byok.provider <openai|anthropic|google|openrouter>`. The model auto-updates when you switch providers.

## API keys

Keys are stored in your OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service). They never leave your machine except when sent to the provider you chose.

```bash
proompt config set byok.api_key YOUR_KEY           # stores for active provider
proompt config set openai.api_key YOUR_KEY          # stores for specific provider
proompt config set openrouter.api_key YOUR_KEY      # stores OpenRouter key
```

Environment variables (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GEMINI_API_KEY`, `OPENROUTER_API_KEY`) work as a fallback.

## Local history

Successful enhancements are saved locally on your device so you can search, favorite, copy, and reuse previous prompts. Quick Enhance history entries also show the routing source, confidence, and reason. Disable history anytime:

```bash
proompt config set preferences.save_history false
```

## Project structure

```
├── crates/
│   ├── core/          # shared Rust library -- config, LLM clients, enhancement engine
│   └── cli/           # CLI binary (clap + indicatif + console)
├── app/
│   ├── src/           # Svelte 5 frontend
│   └── src-tauri/     # Tauri 2 backend (bridges frontend to core)
├── templates/         # external template definitions
└── Cargo.toml         # workspace root
```

The core library is shared between CLI and desktop app. Both are thin wrappers calling the same `enhance()` and `enhance_stream()` functions.

## Tech stack

Rust, Tauri 2, Svelte 5, Vite, OpenAI/Anthropic/Google/OpenRouter APIs.

Config stored as TOML at `~/.config/proompt/config.toml`.

## Running tests

```bash
cargo test                         # all tests
cargo test -p proompt-core         # core library only
```

## Releasing

GitHub Actions publishes release assets when a `v*` tag is pushed:

```bash
git tag v0.1.0
git push origin main --tags
```

The release workflow builds CLI archives for Linux/macOS/Windows and a macOS Apple Silicon desktop `.dmg`. The macOS app is currently unsigned; see the install note above if Gatekeeper blocks the first launch.

## Status

Milestones 1 and 2 are complete -- core engine, multi-provider support, OpenRouter, streaming, CLI with animations, desktop app with full UI, local history, and CI. v0.2.0 adds hotkey-first Coding Agent Mode for Claude Code, Cursor, Codex, and generic coding agents. See `docs/coding-agent-mode.md`.

Not yet built: Supabase auth, Stripe billing, hosted mode, code signing.

## License

MIT
