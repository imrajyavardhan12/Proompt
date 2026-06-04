# Coding Agent Mode Plan

Last updated: 2026-06-04
Target release: v0.2.0
Status: released in v0.2.0

## Purpose

Coding Agent Mode should turn a rough developer task into an execution-ready prompt for AI coding agents such as Claude Code, Cursor, Codex, and generic repo-aware agents.

This is the next strategic wedge after quick enhance and local history because developers are the strongest paid customer segment: bad task prompts produce bad code changes, while better prompts save engineering time.

## Product goal

Make Proompt the fastest way to compose high-quality AI coding tasks from rough intent without leaving the tool the user is already using.

The primary workflow is Quick Enhance, not opening the app:

```text
copy rough task anywhere -> press Cmd/Ctrl+Shift+E -> paste into coding agent
```

A user should be able to write:

```text
fix upload bug
```

and get a prompt with enough structure for a coding agent to investigate, explain, implement, and test the change safely.

## Target platforms

Add new text target platforms:

- `claude-code`
- `cursor`
- `codex`
- `coding-agent` / generic coding agent

These are target platforms, not LLM providers. Provider selection remains OpenAI, Anthropic, Google, or OpenRouter.

Example CLI usage:

```bash
proompt --platform claude-code "fix upload bug"
proompt --platform cursor "add auth middleware"
proompt --platform codex "write tests for billing edge cases"
proompt --platform coding-agent "refactor config loading"
```

Quick Enhance prefix overrides:

```text
/cc fix upload bug
/cursor add auth middleware
/codex write tests for billing edge cases
/agent refactor config loading
```

Proompt strips the prefix before sending the prompt to the provider.

## Desired output shape

Coding-agent prompts should usually include:

```text
Task Summary
Repo / Context Assumptions
Current Behavior / Problem
Constraints
Implementation Guidance
Acceptance Criteria
Test Plan
Files Likely Involved, if inferable
Before Changing Code: explain root cause and plan
```

The exact structure can vary by target platform, but all coding-agent prompts should bias toward:

- clear task framing
- scoped changes
- explicit acceptance criteria
- test instructions
- root-cause analysis before editing
- no unrelated refactors
- preserving existing style and architecture

## Platform-specific behavior

### Claude Code

Optimize for Claude Code's strengths:

- XML-style sections where useful
- explicit repo investigation steps
- ask it to explain root cause before editing
- ask it to summarize changed files and tests run
- keep instructions direct and paste-ready

Potential sections:

```text
<context>
<task>
<constraints>
<acceptance_criteria>
<test_plan>
<workflow>
```

### Cursor

Optimize for an IDE assistant:

- concise task brief
- likely files / symbols if inferable
- clear edit constraints
- test instructions
- request minimal diffs

### Codex

Optimize for autonomous coding-agent execution:

- unambiguous requirements
- deterministic acceptance criteria
- explicit commands/tests to run when known
- instructions to avoid broad rewrites
- final response should include summary and verification

### Generic coding agent

Use universal coding-agent best practices:

- structured markdown
- task/context/constraints/test plan
- root-cause-first workflow
- clear completion checklist

## Non-goals for v0.2.0

Do not include these in the first version:

- repo indexing
- reading local files automatically
- vector search over the repository
- background agents
- team templates
- billing / auth / hosted mode
- deep IDE integration

Those can come later after the prompt shape proves useful.

## Engineering plan

### 1. Extend platform model

Files:

- `crates/core/src/platform/mod.rs`
- `crates/core/src/platform/detector.rs`

Tasks:

- Add platform enum variants for coding-agent targets.
- Treat them as text platforms.
- Add aliases:
  - `claude-code`, `claudecode`, `cc`
  - `cursor`
  - `codex`, `openai-codex`
  - `coding-agent`, `agent`, `generic-agent`
- Update display values.
- Add parser tests.

### 2. Add coding-agent prompt builders

File:

- `crates/core/src/enhance/text.rs`

Tasks:

- Add platform-specific system prompts.
- Keep output paste-ready with no meta-commentary.
- Add tests that each new platform selects the expected prompt.

### 3. Wire CLI

Files:

- `crates/cli/src/main.rs`
- `README.md`

Tasks:

- Update platform help text.
- Add README examples.
- Add CLI integration test that a new platform is accepted far enough to fail only on missing API key, not invalid platform.

### 4. Wire desktop app

Files:

- `app/src/lib/components/EnhancePanel.svelte`

Tasks:

- Add coding-agent target choices in text mode.
- Consider visually grouping text targets:
  - Chat assistants: Claude, GPT, Gemini, Generic
  - Coding agents: Claude Code, Cursor, Codex, Coding Agent
- Keep default text platform unchanged for now.

### 5. Quick Enhance routing

Files:

- `crates/core/src/routing.rs`
- `app/src-tauri/src/commands.rs`
- `app/src/lib/components/SettingsPanel.svelte`
- `app/src/lib/components/EnhancePanel.svelte`

Tasks:

- Keep Quick Enhance hotkey as the primary workflow.
- Add prefix routing for `/cc`, `/cursor`, `/codex`, `/agent`, `/gpt`, `/claude`, and `/gemini`.
- Strip routing prefixes before enhancement.
- Notify the user which target was used.
- Clarify Settings copy: Quick Enhance target, not generic default platform.

### 6. History compatibility

No schema change should be required. New platforms serialize through the existing `Platform` enum and should appear in local history like other platforms.

Verify:

- desktop history renders new platform labels
- CLI history shows new platform display value

## Acceptance criteria

- CLI accepts each new coding-agent platform.
- Desktop exposes each new coding-agent platform in text mode.
- Quick Enhance can route with explicit prefixes without opening the app.
- Generated prompts are coding-task-specific, not generic prompt rewrites.
- Existing text/image platforms still work.
- Local history records new platforms correctly.
- All checks pass:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cd app && bun run build
```

## Suggested first vertical slice

Implement `Platform::ClaudeCode` end to end first:

1. Add enum variant and parser aliases.
2. Add Claude Code system prompt.
3. Add CLI help/docs.
4. Add desktop chip.
5. Add tests.

Then repeat for Cursor, Codex, and generic coding agent.

This keeps the work reviewable and avoids designing all platforms in the abstract.
