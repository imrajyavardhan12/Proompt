# macOS signing and notarization

Proompt's desktop product is hotkey-first, so macOS Accessibility/TCC reliability matters. Unsigned or ad-hoc signed builds can appear trusted in System Settings while a rebuilt or replaced binary is not actually trusted. Release builds should eventually be Developer ID signed and notarized before broader distribution.

Developer ID signing is currently **optional and deferred** until the Apple Developer Program cost makes sense. The project can keep shipping unsigned macOS DMGs in the meantime.

## Current workflow behavior

The release workflow supports two modes for the macOS desktop app:

1. **Unsigned fallback**: if `PROOMPT_ENABLE_MACOS_SIGNING` is not set to `true`, the workflow builds with `--no-sign` and uploads the unsigned DMG. This is the expected path while Developer ID signing is deferred and requires no Apple Developer account.
2. **Signed and notarized**: if `PROOMPT_ENABLE_MACOS_SIGNING=true` and Apple signing secrets are configured, the workflow fails fast on partial configuration, builds with Tauri's signing/notarization support, verifies the resulting app, validates stapled tickets, then uploads the DMG.

The workflow intentionally does **not** publish a signed-but-not-notarized app. Partial signing configuration fails the release instead of silently shipping a confusing Gatekeeper/TCC state. If signing is intentionally deferred, leave `PROOMPT_ENABLE_MACOS_SIGNING` unset or set to anything other than `true`.

## Required Apple assets for future signed releases

You need:

- Apple Developer Program membership.
- A **Developer ID Application** certificate exported as a password-protected `.p12`.
- Notarization credentials, preferably App Store Connect API key credentials.

## GitHub secrets

### Signing certificate

Set this explicit opt-in first:

| Secret | Value |
| --- | --- |
| `PROOMPT_ENABLE_MACOS_SIGNING` | `true` only when Developer ID signing should be active. Leave unset while signing is deferred. |

Then set both certificate secrets:

| Secret | Value |
| --- | --- |
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` Developer ID Application certificate. |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the exported `.p12`. |

Optional:

| Secret | Value |
| --- | --- |
| `APPLE_SIGNING_IDENTITY` | Explicit signing identity, for example `Developer ID Application: Name (TEAMID)`. Tauri can infer this from `APPLE_CERTIFICATE`, so only set it if inference is ambiguous. |
| `APPLE_PROVIDER_SHORT_NAME` | Apple provider short name, only needed for some Apple accounts. |

Create the base64 certificate value locally:

```bash
base64 -i DeveloperIDApplication.p12 | tr -d '\n' | pbcopy
```

Paste the clipboard contents into `APPLE_CERTIFICATE`.

### Preferred notarization: App Store Connect API key

Set all three:

| Secret | Value |
| --- | --- |
| `APPLE_API_KEY` | App Store Connect API key ID, e.g. `ABC123DEFG`. |
| `APPLE_API_ISSUER` | App Store Connect issuer UUID. |
| `APPLE_API_PRIVATE_KEY` | Full contents of the downloaded `AuthKey_ABC123DEFG.p8` file. |

The workflow writes `APPLE_API_PRIVATE_KEY` to a temporary `AuthKey_<APPLE_API_KEY>.p8` file and exposes `APPLE_API_KEY_PATH` only for the build job.

### Alternative notarization: Apple ID app-specific password

Set all three instead of the API-key secrets:

| Secret | Value |
| --- | --- |
| `APPLE_ID` | Apple ID email. |
| `APPLE_PASSWORD` | App-specific password. |
| `APPLE_TEAM_ID` | Developer Team ID. |

API-key notarization is preferred because it is less coupled to a personal Apple ID.

## Release verification

When signing secrets are present, the release workflow runs:

```bash
codesign --verify --deep --strict --verbose=2 target/release/bundle/macos/Proompt.app
spctl --assess --type execute --verbose target/release/bundle/macos/Proompt.app
xcrun stapler validate target/release/bundle/macos/Proompt.app
xcrun stapler validate target/release/bundle/dmg/*.dmg
```

Expected result:

- the app is Developer ID signed
- Gatekeeper accepts it
- notarization tickets are stapled to both the `.app` bundle and `.dmg`

## Local inspection commands

Inspect a release artifact:

```bash
codesign -dvvv /Applications/Proompt.app 2>&1
codesign --verify --deep --strict --verbose=2 /Applications/Proompt.app
spctl --assess --type execute --verbose /Applications/Proompt.app
xcrun stapler validate /Applications/Proompt.app
```

Check Accessibility/TCC after installing a signed release:

```bash
tccutil reset Accessibility com.proompt.desktop
open /Applications/Proompt.app
```

Then grant Accessibility once and verify selected-text Quick Enhance in a few host apps.

## Privacy boundary

Signing and notarization do not change Proompt's prompt/privacy model. Selected-text diagnostics remain local-only and contain metadata/status/error codes, active app/window metadata, and character counts only — not selected text or prompt content.
