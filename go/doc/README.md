# browser-tester go

This directory is the Go rewrite workspace for `browser-tester`.
It is still a plan and specification set, but the phase 0 scaffold now exists and the module builds with skeleton tests.

The design follows the lessons captured in [`../next.md`](../../next.md) and [`../next-reflection.md`](../../next-reflection.md):

- keep `Harness` thin
- keep state in explicit subsystems
- make deterministic mocks first-class
- separate debug views from the public action surface
- keep configuration explicit instead of hiding seeds in unrelated state

## Document Index

- [Subsystem Map](subsystem-map.md)
- [Capability Matrix](capability-matrix.md)
- [Implementation Guide](implementation-guide.md)
- [Mock Guide](mock-guide.md)
- [Roadmap](roadmap.md)

## Core Rules

- Check `../html-standard/` before adding HTML, DOM, selector, or serialization behavior.
- Add a new public `Harness` method only after deciding whether it belongs on `Harness`, `DebugView`, or a mock family.
- Add a new test-only mock through the runtime registry, then expose it through the public facade.
- Prefer explicit configuration structs over hidden encodings or seed keys.
- Keep the Go implementation deterministic. Avoid background goroutines unless a subsystem explicitly requires them.

## Current Status

- Phase 0 scaffold is present, and internal DOM/runtime/script scaffolds have landed.
- The initial interaction slice (`Click`/`Focus`/`Blur`), the initial form-control slice (`TypeText`/`SetChecked`/`SetSelectValue`/`Submit`), and the initial assertion slice (`AssertText`/`AssertValue`/`AssertChecked`/`AssertExists`) are wired through the public facade and debug view, and `Click` also follows bounded hyperlink default actions for `a` / `area` elements and reset-button form reset through the location, open, download, and DOM form-control helpers. Inline `<script>` listeners can register capture/target/bubble handlers through the host bridge for the bounded event slice, can call `host:preventDefault()` to suppress click/reset default actions, can call `host:stopPropagation()` to stop later propagation, can opt into one-shot handling with a boolean `once` flag, can remove a previously registered handler with `host:removeEventListener()`, can queue bounded microtasks with `host:queueMicrotask()`, can schedule bounded timers with `host:setTimeout()` / `host:setInterval()` and `host:clearTimeout()` / `host:clearInterval()`, can schedule bounded animation-frame callbacks with `host:requestAnimationFrame()` / `host:cancelAnimationFrame()`, can observe the currently executing classic script through `host:documentCurrentScript()` and the explicit `expr(...)` wrapper in argument position, can drive the location mock through `host:locationAssign()` / `host:locationReplace()` / `host:locationReload()` / `host:locationSet()`, can drive a bounded `window.history` slice through `host:historyPushState()` / `host:historyReplaceState()` / `host:historyBack()` / `host:historyForward()` / `host:historyGo()` plus `host:historyLength()` / `host:historyState()` / `host:historyScrollRestoration()` / `host:historySetScrollRestoration()`, can read or write a bounded cookie jar through `host:documentCookie()` / `host:setDocumentCookie()` plus `host:navigatorCookieEnabled()`, and can read or write a bounded `window.name` state through `host:windowName()` / `host:setWindowName()`. Location URLs are resolved against the current URL just like navigation links, and history updates feed the same navigation log. It can also trigger bounded synthetic event helpers such as `Dispatch` and `DispatchKeyboard` for custom and keyboard event sequences, and it can query bounded `matchMedia` state through `MatchMedia()`.
- The selector engine now covers a bounded descendant/child/sibling combinator slice in addition to the simple tag/id/class forms, plus a bounded pseudo-class slice (`:root`, `:scope`, `:defined`, `:state(identifier)`, `:active`, `:hover`, `:empty`, `:checked`, `:indeterminate`, `:autofill`, `:-webkit-autofill`, `:default`, `:enabled`, `:disabled`, `:required`, `:optional`, `:read-only`, `:read-write`, `:valid`, `:invalid`, `:user-valid`, `:user-invalid`, `:in-range`, `:out-of-range`, `:first-child`, `:last-child`, `:first-of-type`, `:last-of-type`, `:only-child`, `:only-of-type`, `:nth-child()`, `:nth-of-type()`, `:nth-last-child()`, `:nth-last-of-type()`, `:link`, `:any-link`, `:visited`, `:local-link`, `:lang()`, `:dir()`, `:placeholder-shown`, `:blank`, `:heading`, `:heading(integer#)`, `:playing`, `:paused`, `:seeking`, `:buffering`, `:stalled`, `:muted`, `:volume-locked`, `:modal`, `:popover-open`, `:open`, `:focus`, `:focus-visible`, `:focus-within`, `:target`, `:target-within`, `:is()`, `:where()`, `:not()`, and `:has()`). In the current document-context API, `:scope` is approximated as the document root element, `:blank` is approximated for text-like inputs and textareas with empty or whitespace-only values, `:local-link` is approximated as a same-document link against the current session URL, and `:visited` is approximated against the current session history URLs. Custom element states are approximated through a tokenized `state` attribute on custom elements.
- Script DOM query helpers are available through host bindings for `querySelector` / `querySelectorAll` / `matches` / `closest`, `querySelectorAll` returns a minimal snapshot `NodeList`, and a minimal live `HTMLCollection` covers `children`.
- Inline `<script>` blocks are preserved as raw text and execute during bootstrap through the bounded script host bridge, so source HTML can mutate the live DOM.
- Bounded attribute reflection helpers are available for `GetAttribute` / `HasAttribute` / `SetAttribute` / `RemoveAttribute`, and public live `ClassList` / `Dataset` views expose the same DOM slice through the facade.
- Internal bounded `classList` / `dataset` helpers still live in `internal/dom` and remain the source of truth for the live views.
- The public tree-mutation slice (`InnerHTML`, `OuterHTML`, `SetInnerHTML`, `SetOuterHTML`, `InsertAdjacentHTML`, `RemoveNode`, `WriteHTML`) is wired through the facade; `cloneNode` remains internal for now, and `WriteHTML()` covers the bounded document-write-style replay slice.
- `go test ./...` passes for the current skeleton.
- The clipboard mock family scaffold is present, including `ReadClipboard` and `WriteClipboard`.
- Later phases remain intentionally bounded and future-facing.
- Keep this index aligned with the capability matrix and mock guide when the public surface changes.
- Do not implement legacy or deprecated spec branches unless they are required for a clearly bounded user-visible gap and are explicitly added to the capability matrix.

## Target Shape

- Public package: `browsertester`
- Internal packages: `internal/dom`, `internal/runtime`, `internal/script`, `internal/mocks`
- Public facade types: `Harness`, `HarnessBuilder`, `DebugView`, `MockRegistryView`, `Error`

## When Code Lands

The intended quick check is:

```bash
go test ./...
```
