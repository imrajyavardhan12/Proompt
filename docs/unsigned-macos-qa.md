# Unsigned macOS Quick Enhance QA

Use this runbook while Proompt ships unsigned macOS DMGs. The goal is to keep the hotkey-first selected-text loop reliable enough without Apple Developer ID signing.

## Install/reset baseline

```bash
xattr -dr com.apple.quarantine /Applications/Proompt.app
open /Applications/Proompt.app
```

If selected-text capture behaves strangely after replacing an unsigned build:

```bash
tccutil reset Accessibility com.proompt.desktop
open /Applications/Proompt.app
```

Then grant Proompt under Privacy & Security → Accessibility.

## Smoke matrix

For each host app:

1. Select rough prompt text.
2. Press Quick Enhance hotkey (`Cmd/Ctrl+Shift+E`).
3. Confirm Proompt enhances selected text.
4. Confirm result replaces the selection when safe, otherwise is copied to clipboard with a clear notification.
5. Check Settings → Selected-text diagnostics if capture falls back.

| Host app | Expected result | Notes |
| --- | --- | --- |
| TextEdit | Selection captured and replaced | Basic native text field baseline. |
| Notes | Selection captured or clipboard fallback explained | Useful real-world rich text target. |
| Ghostty | Selection captured or clipboard fallback explained | Important user workflow. |
| Cursor | Selection captured and route can target Cursor | Coding-agent workflow. |
| Browser text field | Selection captured and replaced when focus is stable | Test ChatGPT/Claude web inputs. |

## Diagnostics privacy check

Diagnostics live at:

`~/Library/Application Support/proompt/selection-diagnostics.json`

Verify the file contains only:

- capture status/outcome
- error/status codes
- active app/window metadata
- character counts
- timestamps and steps

It must not contain selected text or prompt content.

## Release acceptance

Before shipping an unsigned release that touches Quick Enhance selected-text behavior:

- Settings opens and shows Selected-text diagnostics.
- Accessibility trusted/untrusted state is accurate.
- Open System Settings works on macOS.
- Reset command is copyable from Settings.
- At least TextEdit plus one coding workflow host are smoke-tested.
- Release notes say the DMG is unsigned/not notarized and include quarantine reset guidance.
