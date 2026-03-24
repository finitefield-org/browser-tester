# browser-tester Go Workspace

This directory is the Go implementation track for `browser-tester`.
It is intentionally conservative: a thin public facade, explicit builder config, typed mock families, and bounded runtime slices.

## Public Surface

- `Harness`
- `HarnessBuilder`
- `Error` and `ErrorKind`
- `DebugView`
- `MockRegistryView`
- user-like actions that delegate into runtime or mock families:
  - `Fetch`
  - `Alert`
  - `Confirm`
  - `Prompt`
  - `ReadClipboard`
  - `WriteClipboard`
  - `Open`
  - `Close`
  - `Print`
  - `ScrollTo`
  - `ScrollBy`
  - `Navigate`
  - `SetFiles`
  - `CaptureDownload`
- `Prompt` returns the submitted text plus a boolean that is `false` when the prompt is canceled.
- typed mock families for:
  - `Fetch`
  - `Dialogs`
  - `Clipboard`
  - `Location`
  - `Open`
  - `Close`
  - `Print`
  - `Scroll`
  - `MatchMedia`
  - `Downloads`
  - `FileInput`
  - `Storage`

## Current Scope

- Phase 0 scaffold is in place, and internal DOM/runtime/script scaffolds now exist.
- Legacy and deprecated spec branches are not implementation targets unless the capability matrix explicitly lists a compatibility exception.
- Later DOM, script, and event/runtime slices will be added behind the same facade.

## Docs

- `doc/README.md`
- `doc/subsystem-map.md`
- `doc/capability-matrix.md`
- `doc/implementation-guide.md`
- `doc/mock-guide.md`
- `doc/roadmap.md`
