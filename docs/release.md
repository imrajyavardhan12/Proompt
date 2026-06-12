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
- If Quick Enhance selected-text behavior changed, run `docs/unsigned-macos-qa.md`

macOS signing note:

- Developer ID signing/notarization is optional and currently deferred. No Apple Developer account is required for unsigned releases.
- The release workflow signs/notarizes the DMG only when Apple signing secrets are configured. See `docs/macos-signing.md`.
- If signing secrets are absent, the workflow publishes an unsigned fallback DMG and emits a warning.
- Unsigned builds may trigger Gatekeeper's "damaged" warning. If testing an official unsigned GitHub Release artifact, remove quarantine once:

```bash
xattr -dr com.apple.quarantine /Applications/Proompt.app
open /Applications/Proompt.app
```

## 3. Write release notes

Create curated release notes before tagging so GitHub does not fall back to auto-generated PR bullets:

```bash
cp docs/releases/TEMPLATE.md docs/releases/vX.Y.Z.md
$EDITOR docs/releases/vX.Y.Z.md
```

Use the `v0.3.0`/`v0.3.1` style: grouped headings, user impact first, macOS Accessibility/signing caveats when relevant, and a full changelog compare link.

## 4. Commit and push

```bash
git status --short
git add .
git commit -m "chore: release vX.Y.Z"
git push origin main
```

Wait for CI on `main` to pass.

## 5. Tag

```bash
git tag -a vX.Y.Z -m "Proompt vX.Y.Z"
git push origin vX.Y.Z
```

The release workflow will:

1. Create a draft GitHub Release.
2. Use `docs/releases/vX.Y.Z.md` if present; otherwise generate notes once.
3. Build and upload:
   - Linux CLI archive
   - macOS CLI archive
   - Windows CLI archive
   - macOS Apple Silicon desktop `.dmg`
4. Sign/notarize the macOS app when Apple secrets are configured; otherwise publish an unsigned fallback with a warning. The unsigned path is expected while Developer ID signing is deferred.
5. Publish the draft only after all build/upload jobs pass.

## 6. Verify GitHub Release

```bash
gh release view vX.Y.Z --json url,assets --jq '{url, assets:[.assets[].name]}'
```

Confirm the release is not a draft and contains all expected assets. If Apple signing secrets were configured for the release, download the DMG and verify Gatekeeper/stapling locally:

```bash
codesign --verify --deep --strict --verbose=2 /Applications/Proompt.app
spctl --assess --type execute --verbose /Applications/Proompt.app
xcrun stapler validate /Applications/Proompt.app
```

Release notes should clearly say:

- Desktop DMG is macOS Apple Silicon only for now.
- Whether the desktop app is signed/notarized or unsigned/not notarized.
- If the release includes selected-text replacement, macOS requires Accessibility permission.
- If the release includes selected-text replacement, Proompt writes a local diagnostics file at `~/Library/Application Support/proompt/selection-diagnostics.json` for troubleshooting capture/replacement issues. Note that it contains metadata/status/error codes, active app/window metadata, and character counts only — not selected text or prompt content — and can be deleted anytime.
- If macOS says the app is damaged, run:

```bash
xattr -dr com.apple.quarantine /Applications/Proompt.app
open /Applications/Proompt.app
```

## 7. If a release workflow fix is needed after tagging

Do not mutate the old tag unless it was never shared. Prefer a patch release:

```bash
# after fixing main
git tag -a vX.Y.(Z+1) -m "Proompt vX.Y.(Z+1)"
git push origin vX.Y.(Z+1)
```
