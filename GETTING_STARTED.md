# Getting Started - Proompt

A guide for web developers new to Rust. If you know `bun dev` / `npm run dev`, this will feel familiar.

---

## 1. Prerequisites Check

Run these to make sure everything is installed:

```bash
rustc --version    # Need 1.75+ (you have 1.92)
cargo --version    # Rust's package manager (like npm/bun)
node --version     # Need 18+
bun --version      # Need 1.0+
```

### Install Tauri CLI (one-time)

```bash
# Fastest option (no compilation):
bun add -g @tauri-apps/cli

# Verify:
bunx tauri --version
```

---

## 2. Mental Model: Rust vs Web Dev

| Web Dev Concept     | Rust Equivalent           | Example                          |
|---------------------|---------------------------|----------------------------------|
| `package.json`      | `Cargo.toml`              | Dependencies, project metadata   |
| `node_modules/`     | `target/` (auto-managed)  | Never commit, never touch        |
| `npm install`       | `cargo build`             | Fetches deps + compiles          |
| `bun dev`           | `cargo run`               | Build + run in one step          |
| `npm test`          | `cargo test`              | Runs all `#[test]` functions     |
| `npm run build`     | `cargo build --release`   | Optimized production build       |
| monorepo workspaces | Cargo workspace            | Root `Cargo.toml` lists members  |
| `npx`               | `cargo run -p <package>`  | Run a specific package           |

**Key difference:** Rust compiles to a native binary. First build is slow (1-2 min), subsequent builds are fast (1-5 sec) because only changed code recompiles.

---

## 3. Running the CLI (Fastest Way to Test)

```bash
# From project root:
cd "/Users/rvs/Developer/Prompt Enhancer"

# Step 1: Build everything (first time takes ~1 min, then ~2 sec)
cargo build

# Step 2: Set your OpenAI API key (stored in macOS Keychain)
cargo run -p proompt-cli -- config set byok.api_key "sk-YOUR_OPENAI_KEY_HERE"

# Step 3: Verify config
cargo run -p proompt-cli -- config show
```

Now test the features:

```bash
# Enhance a text prompt (default: Claude)
cargo run -p proompt-cli -- "explain how docker works"

# Enhance for a specific platform
cargo run -p proompt-cli -- --platform openai "explain kubernetes"

# Enhance an image prompt
cargo run -p proompt-cli -- --image "a cat in space"

# Image prompt for specific platform
cargo run -p proompt-cli -- --image --platform midjourney "sunset over mountains"

# List viral templates
cargo run -p proompt-cli -- templates list

# Use a template
cargo run -p proompt-cli -- --template ghibli-style "my cat on the couch"
cargo run -p proompt-cli -- --template code-review "function add(a, b) { return a + b }"

# Pipe mode (like unix pipes)
echo "explain rust ownership" | cargo run -p proompt-cli

# Show help
cargo run -p proompt-cli -- --help
```

**Shorthand:** `cargo run -p proompt-cli --` is verbose. After `cargo build --release`, you can use the binary directly:

```bash
cargo build --release
./target/release/proompt "explain docker"
./target/release/proompt --help
```

> **What does `--` mean?** Everything before `--` is for cargo. Everything after is passed to your program. So `cargo run -p proompt-cli -- --help` runs `proompt --help`.

---

## 4. Running the Desktop App

This is the closest to `bun dev` -- hot-reload frontend + native Rust backend:

```bash
cd "/Users/rvs/Developer/Prompt Enhancer/app"

# Install JS dependencies (one-time)
bun install

# Launch dev mode (hot-reload!)
bun tauri dev
```

This does three things simultaneously:
1. Starts Vite dev server (serves Svelte frontend at localhost:1420)
2. Compiles the Rust backend
3. Opens the native window pointing at the Vite server

**Frontend changes** (Svelte files) hot-reload instantly -- same as web dev.
**Backend changes** (Rust files in `src-tauri/`) trigger a recompile (~2-5 sec).

---

## 5. Running Tests

```bash
# From project root:
cd "/Users/rvs/Developer/Prompt Enhancer"

# Run ALL tests (core + cli + app)
cargo test

# Run only core library tests (most useful)
cargo test -p proompt-core

# Run a specific test by name
cargo test test_detect_platform

# Run tests with output visible (normally hidden on pass)
cargo test -- --nocapture

# Run tests matching a pattern
cargo test template
```

---

## 6. Common Workflows

### "I changed a Rust file, how do I see the effect?"

```bash
# For CLI: just run it again (cargo recompiles automatically)
cargo run -p proompt-cli -- "test prompt"

# For desktop app: if `bun tauri dev` is running, it auto-recompiles
```

### "I changed a Svelte file"

If `bun tauri dev` is running, it hot-reloads automatically. Same as web dev.

### "I added a new Rust dependency"

```bash
# Add to the specific crate's Cargo.toml, then:
cargo build
# Cargo fetches and compiles the new dependency automatically
```

### "Build feels slow"

First build downloads + compiles all dependencies. After that:
- `cargo build` (debug): 1-5 sec for your code changes
- `cargo build --release` (optimized): longer, only needed for distribution

### "I want to check for errors without running"

```bash
# Fast compile check (no binary output, just error checking)
cargo check

# Check with all warnings
cargo clippy   # (install first: rustup component add clippy)
```

---

## 7. Project Structure Quick Reference

```
Prompt Enhancer/
├── Cargo.toml              ← Workspace root (like root package.json)
├── crates/
│   ├── core/               ← Shared library (all the logic)
│   │   ├── Cargo.toml      ← Core dependencies
│   │   └── src/            ← Core source code
│   └── cli/                ← CLI binary
│       ├── Cargo.toml      ← CLI dependencies
│       └── src/            ← CLI source code
├── app/
│   ├── package.json        ← Frontend dependencies (Svelte, Vite)
│   ├── src/                ← Svelte frontend (edit these like normal web dev)
│   └── src-tauri/          ← Rust backend for the desktop app
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs     ← Tauri setup
│           └── commands.rs ← Functions the frontend can call
└── target/                 ← Build output (auto-generated, gitignored)
```

---

## 8. Troubleshooting

| Problem | Fix |
|---------|-----|
| `cargo build` fails with missing system lib | macOS: `xcode-select --install` |
| `bunx tauri dev` can't find tauri | `bun add -g @tauri-apps/cli` |
| API key error when enhancing | `cargo run -p proompt-cli -- config set byok.api_key "YOUR_KEY"` |
| "permission denied" on binary | `chmod +x ./target/release/proompt` |
| Slow first build | Normal. ~1-2 min first time, ~2 sec after |
| Port 1420 in use | Kill other dev servers, or change port in `app/vite.config.ts` |
| `cargo test` fails on keychain tests | Some keychain tests need macOS Keychain access |

---

## TL;DR

```bash
# One-time setup:
bun add -g @tauri-apps/cli

# Test the CLI:
cd "/Users/rvs/Developer/Prompt Enhancer"
cargo run -p proompt-cli -- config set byok.api_key "YOUR_KEY"
cargo run -p proompt-cli -- "explain docker"

# Test the desktop app:
cd app && bun install && bunx tauri dev

# Run tests:
cargo test
```
