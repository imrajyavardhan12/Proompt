# Release Checklist

Use this checklist for every Proompt release.

## 1. Pick the version

Use semver:

- Patch (`0.1.1`) for fixes, CI, docs, and low-risk UX polish
- Minor (`0.2.0`) for user-visible features such as new providers
- Major (`1.0.0`) only after the product is stable and publicly supported

Update all version fields:

- `Cargo.toml` workspace version
- `app/src-tauri/Cargo.toml`
- `app/src-tauri/tauri.conf.json`
- `app/package.json`
- `app/src/App.svelte` sidebar version

## 2. Verify locally

From the repository root:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cd app
bun install --frozen-lockfile
bun run build
```

Optional desktop smoke test:

```bash
cd app
bunx tauri build
open ../target/release/bundle/macos/Proompt.app
```

Smoke-check:

- App opens
- Settings shows expected version
- Provider list includes OpenRouter
- Templates tab loads built-in templates
- Enhance tab validates missing provider key with an actionable error

Unsigned macOS note:

- The DMG is currently unsigned because Developer ID signing/notarization is not set up yet.
- Gatekeeper may report `Proompt.app` is "damaged" on first launch.
- If testing an official GitHub Release artifact, remove quarantine once:

```bash
xattr -dr com.apple.quarantine /Applications/Proompt.app
open /Applications/Proompt.app
```

## 3. Commit and push

```bash
git status --short
git add .
git commit -m "chore: release vX.Y.Z"
git push origin main
```

Wait for CI on `main` to pass.

## 4. Tag

```bash
git tag -a vX.Y.Z -m "Proompt vX.Y.Z"
git push origin vX.Y.Z
```

The release workflow will create/update the GitHub Release and upload:

- Linux CLI archive
- macOS CLI archive
- Windows CLI archive
- macOS Apple Silicon desktop `.dmg`

## 5. Verify GitHub Release

```bash
gh release view vX.Y.Z --json url,assets --jq '{url, assets:[.assets[].name]}'
```

Confirm the release is not a draft and contains all expected assets.

Release notes should clearly say:

- Desktop DMG is macOS Apple Silicon only for now.
- Desktop app is unsigned/not notarized.
- If macOS says the app is damaged, run:

```bash
xattr -dr com.apple.quarantine /Applications/Proompt.app
open /Applications/Proompt.app
```

## 6. If a release workflow fix is needed after tagging

Do not mutate the old tag unless it was never shared. Prefer a patch release:

```bash
# after fixing main
git tag -a vX.Y.(Z+1) -m "Proompt vX.Y.(Z+1)"
git push origin vX.Y.(Z+1)
```
