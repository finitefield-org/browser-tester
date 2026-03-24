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
  - `MatchMedia`
  - `Open`
  - `Close`
  - `Print`
  - `ScrollTo`
  - `ScrollBy`
  - `Navigate`
  - `AdvanceTime`
  - `SetFiles`
  - `CaptureDownload`
- bounded `window.history` host helpers for inline scripts:
  - `historyPushState`
  - `historyReplaceState`
  - `historyBack`
  - `historyForward`
  - `historyGo`
  - `historyLength`
  - `historyState`
  - `historyScrollRestoration`
  - `historySetScrollRestoration`
- bounded cookie helpers for inline scripts:
  - `documentCookie`
  - `setDocumentCookie`
  - `navigatorCookieEnabled`
- bounded current-script helper for inline scripts:
  - `documentCurrentScript`
- nested expression wrapper for inline scripts:
  - `expr(...)`
- bounded `window.name` helpers for inline scripts:
  - `windowName`
  - `setWindowName`
- `Prompt` returns the submitted text plus a boolean that is `false` when the prompt is canceled.
- `DebugView` is read-only and exposes inspection state such as `URL`, `HTML`, `NowMs`, `DumpDOM`, `FocusedSelector`, `ScrollPosition`, `WindowName`, `Interactions`, and the configured `RandomSeed` when one was set on the builder.
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
- live class/dataset views:
  - `ClassList`
  - `Dataset`
- tree mutation helpers:
  - `InnerHTML`
  - `OuterHTML`
  - `SetInnerHTML`
  - `SetOuterHTML`
  - `InsertAdjacentHTML`
  - `RemoveNode`
  - `WriteHTML`
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

- Phase 0 scaffold is in place, internal DOM/runtime/script scaffolds now exist, the initial interaction slice (`Click`/`Focus`/`Blur`) is wired through the facade, the initial form-control slice (`TypeText`/`SetChecked`/`SetSelectValue`/`Submit`) updates the live DOM, and the initial assertion slice (`AssertText`/`AssertValue`/`AssertChecked`/`AssertExists`) is wired through the same selector engine. `Click` also follows bounded hyperlink default actions for `a` / `area` elements and reset-button form reset through the location, open, download, and DOM form-control helpers. Inline `<script>` listeners can register capture/target/bubble handlers through the host bridge for the bounded event slice, can call `host:preventDefault()` to suppress click/reset default actions, can call `host:stopPropagation()` to stop later propagation, can opt into one-shot handling with a boolean `once` flag, can remove a previously registered handler with `host:removeEventListener()`, can queue bounded microtasks with `host:queueMicrotask()`, can schedule bounded timers with `host:setTimeout()` / `host:setInterval()` and `host:clearTimeout()` / `host:clearInterval()`, can schedule bounded animation-frame callbacks with `host:requestAnimationFrame()` / `host:cancelAnimationFrame()`, can drive the location mock through `host:locationAssign()` / `host:locationReplace()` / `host:locationReload()` / `host:locationSet()`, and location URLs are resolved against the current URL just like navigation links. It can also trigger bounded synthetic event helpers such as `Dispatch` and `DispatchKeyboard` for custom and keyboard event sequences, and it can query bounded `matchMedia` state through `MatchMedia()`.
- The selector engine also supports a bounded descendant/child/sibling combinator slice on top of the simple tag/id/class forms, plus a bounded pseudo-class slice (`:root`, `:scope`, `:defined`, `:state(identifier)`, `:active`, `:hover`, `:empty`, `:checked`, `:indeterminate`, `:autofill`, `:-webkit-autofill`, `:default`, `:enabled`, `:disabled`, `:required`, `:optional`, `:read-only`, `:read-write`, `:valid`, `:invalid`, `:user-valid`, `:user-invalid`, `:in-range`, `:out-of-range`, `:first-child`, `:last-child`, `:first-of-type`, `:last-of-type`, `:only-child`, `:only-of-type`, `:nth-child()`, `:nth-of-type()`, `:nth-last-child()`, `:nth-last-of-type()`, `:link`, `:any-link`, `:visited`, `:local-link`, `:lang()`, `:dir()`, `:placeholder-shown`, `:blank`, `:heading`, `:heading(integer#)`, `:playing`, `:paused`, `:seeking`, `:buffering`, `:stalled`, `:muted`, `:volume-locked`, `:modal`, `:popover-open`, `:open`, `:focus`, `:focus-visible`, `:focus-within`, `:target`, `:target-within`, `:is()`, `:where()`, `:not()`, and `:has()`). In the current document-context API, `:scope` is approximated as the document root element, `:blank` is approximated for text-like inputs and textareas with empty or whitespace-only values, `:local-link` is approximated as a same-document link against the current session URL, and `:visited` is approximated against the current session history URLs. Custom element states are approximated through a tokenized `state` attribute on custom elements.
- Script DOM query helpers are available through host bindings for `querySelector` / `querySelectorAll` / `matches` / `closest`, `querySelectorAll` returns a minimal snapshot `NodeList`, and a minimal live `HTMLCollection` covers `children`.
- Inline `<script>` blocks are preserved as raw text and execute during bootstrap through the bounded script host bridge, so HTML source can mutate the live DOM.
- Bounded attribute reflection helpers are available through `GetAttribute` / `HasAttribute` / `SetAttribute` / `RemoveAttribute`, and public live `ClassList` / `Dataset` views expose the same DOM slice through the facade.
- Internal bounded `classList` / `dataset` helpers still live in `internal/dom` and remain the source of truth for the live views.
- The public tree-mutation slice (`InnerHTML`, `OuterHTML`, `SetInnerHTML`, `SetOuterHTML`, `InsertAdjacentHTML`, `RemoveNode`, `WriteHTML`) now delegates into `internal/dom`; `cloneNode` remains internal for now, and `WriteHTML()` provides the bounded document-write-style replay slice.
- Legacy and deprecated spec branches are not implementation targets unless the capability matrix explicitly lists a compatibility exception.
- Later DOM, script, and event/runtime slices will be added behind the same facade.

## Docs

- `doc/README.md`
- `doc/subsystem-map.md`
- `doc/capability-matrix.md`
- `doc/implementation-guide.md`
- `doc/mock-guide.md`
- `doc/roadmap.md`
