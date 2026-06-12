## What's Changed

### Headline area

- User-facing change 1.
- User-facing change 2.
- Reliability/supportability change.

### macOS Accessibility and diagnostics

Include this section when the release touches Quick Enhance selected-text capture/replacement.

Selected-text capture/replacement on macOS requires Accessibility permission.

Proompt writes a local troubleshooting file at:

`~/Library/Application Support/proompt/selection-diagnostics.json`

This file contains only metadata such as capture status, error codes, active app/window metadata, and character counts. It does **not** contain selected text or prompt content. The file stays on your device and can be deleted anytime.

### Desktop release note

- Desktop DMG is macOS Apple Silicon only for now.
- Desktop app is unsigned/not notarized.
- If macOS says the app is damaged, run:

```bash
xattr -dr com.apple.quarantine /Applications/Proompt.app
open /Applications/Proompt.app
```

**Full Changelog**: https://github.com/imrajyavardhan12/Proompt/compare/vPREVIOUS...vNEXT
