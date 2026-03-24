# browser-tester Go Workspace

This directory is the Go implementation track for `browser-tester`.
It is intentionally conservative: a thin public facade, explicit builder config, typed mock families, and bounded runtime slices.

## Public Surface

- `Harness`
- `HarnessBuilder`
- `Error` and `ErrorKind`
- `DebugView`
- `Interaction` and `InteractionKind`
- `MockRegistryView`
- user-like actions that delegate into runtime or mock families:
  - `Fetch`
  - `Alert`
  - `Confirm`
  - `Prompt`
  - `Click`
  - `TypeText`
  - `SetChecked`
  - `SetSelectValue`
  - `Focus`
  - `Blur`
  - `Submit`
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
- `DebugView` is read-only and exposes inspection state such as `URL`, `HTML`, `NowMs`, `DumpDOM`, `FocusedSelector`, `Interactions`, and the configured `RandomSeed` when one was set on the builder.
- assertion helpers:
  - `AssertText`
  - `AssertValue`
  - `AssertChecked`
  - `AssertExists`
- attribute reflection helpers:
  - `GetAttribute`
  - `HasAttribute`
  - `SetAttribute`
  - `RemoveAttribute`
- tree mutation helpers:
  - `InnerHTML`
  - `OuterHTML`
  - `SetInnerHTML`
  - `SetOuterHTML`
  - `InsertAdjacentHTML`
  - `RemoveNode`
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

- Phase 0 scaffold is in place, internal DOM/runtime/script scaffolds now exist, the initial interaction slice (`Click`/`Focus`/`Blur`) is wired through the facade, the initial form-control slice (`TypeText`/`SetChecked`/`SetSelectValue`/`Submit`) updates the live DOM, and the initial assertion slice (`AssertText`/`AssertValue`/`AssertChecked`/`AssertExists`) is wired through the same selector engine. `Click` also follows bounded hyperlink default actions for `a` / `area` elements and reset-button form reset through the location, open, download, and DOM form-control helpers. Inline `<script>` listeners can register target-phase handlers through the host bridge, and those handlers can chain bounded `host:` statements separated by `;`.
- The selector engine also supports a bounded descendant/child combinator slice on top of the simple tag/id/class forms.
- Script DOM query helpers are available through host bindings for `querySelector` / `querySelectorAll` / `matches` / `closest`, `querySelectorAll` returns a minimal snapshot `NodeList`, and a minimal live `HTMLCollection` covers `children`.
- Inline `<script>` blocks are preserved as raw text and execute during bootstrap through the bounded script host bridge, so HTML source can mutate the live DOM.
- Bounded attribute reflection helpers are available through `GetAttribute` / `HasAttribute` / `SetAttribute` / `RemoveAttribute`; explicit `classList` and `dataset` views remain later.
- Internal bounded `classList` / `dataset` helpers now live in `internal/dom`; public facade exposure will wait until the need is proven.
- The public tree-mutation slice (`InnerHTML`, `OuterHTML`, `SetInnerHTML`, `SetOuterHTML`, `InsertAdjacentHTML`, `RemoveNode`) now delegates into `internal/dom`; `cloneNode` remains internal for now.
- Legacy and deprecated spec branches are not implementation targets unless the capability matrix explicitly lists a compatibility exception.
- Later DOM, script, and event/runtime slices will be added behind the same facade.

## Docs

- `doc/README.md`
- `doc/subsystem-map.md`
- `doc/capability-matrix.md`
- `doc/implementation-guide.md`
- `doc/mock-guide.md`
- `doc/roadmap.md`
