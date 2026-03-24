# Capability Matrix

This matrix defines the target Go implementation.
The current Go workspace already contains scaffolded slices, so the `Status` column reflects whether each slice is still planned or has landed as a scaffold.

| Capability | Level | Phase | Status | Notes |
| --- | --- | --- | --- | --- |
| Public facade scaffold | Stable Core | 0 | Scaffolded | `Harness`, `HarnessBuilder`, `Error`, `DebugView`, and `MockRegistryView` stay thin and delegate into internal packages. |
| Harness constructors | Stable Core | 0 | Scaffolded | `FromHTML`, `FromHTMLWithURL`, `FromHTMLWithLocalStorage`, `FromHTMLWithURLAndLocalStorage`, `FromHTMLWithSessionStorage`, and `FromHTMLWithURLAndSessionStorage` build the initial session snapshot through the builder. |
| Builder configuration capture | Stable Core | 0 | Scaffolded | URL, HTML, local/session storage, random seed, matchMedia rules, and open/close/print/scroll failure seeds are explicit builder fields. |
| Error taxonomy | Stable Core | 0 | Scaffolded | The package classifies HTML parse, script parse/runtime, selector, DOM, event, timer, mock, assertion, and unsupported failures. |
| Harness accessors and views | Stable Core | 0 | Scaffolded | `URL`, `HTML`, `NowMs`, `Debug`, and `Mocks` return session snapshots or views; `DebugView` includes read-only observation of configured builder state such as `RandomSeed`, plus live inspection helpers such as `DumpDOM`, `FocusedSelector`, and `Interactions`, while time and user-like actions are covered by later runtime rows. |
| DOM store and HTML parsing | Stable Core | 1 | Scaffolded | Generational node IDs, tree storage, parsing, and DOM dump helpers now live in `internal/dom`; raw `<script>` text is preserved so inline script sources can round-trip through the parser and serializer. |
| Selector subset and assertions | Stable Core | 1 | Scaffolded | The bounded selector slice (`tag`, `#id`, `.class`, `*`, simple combinations, and bounded descendant/child combinators) exists in `internal/dom`; public `Harness` assertion helpers (`AssertText`, `AssertValue`, `AssertChecked`, and `AssertExists`) now delegate to the same selector engine and include DOM dumps on failure. |
| Script runtime minimum | Stable Core | 2 | Scaffolded | Inline bootstrap, host bindings, and the minimal dispatch surface now live in `internal/script`; bounded inline scripts execute during bootstrap through the host bridge and can mutate the DOM. |
| Event dispatch and default actions | Stable Core | 3 | Scaffolded | User-like actions (`Click`, `Focus`, `Blur`, `TypeText`, `SetChecked`, `SetSelectValue`, and `Submit`) validate selectors, track focus, dispatch bounded target-phase listeners registered from inline scripts through the host bridge, apply bounded default actions (checkbox/radio toggles, reset-button form reset, hyperlink navigation, `_blank` open capture, or download capture for `a`/`area`, and submit-button form submission), and capture an interaction log; bubbling and broader listener semantics remain later. |
| Form controls and selection state | Stable Core | 3 | Scaffolded | Text-like input and textarea values, checkbox/radio checkedness (including radio-group exclusivity), and select option selectedness can be updated deterministically through user-like actions and are observable via `DebugView.DumpDOM()`. |
| Deterministic scheduler and time | Stable Core | 4 | Scaffolded | Fake clock control is explicit in `internal/runtime` through `Scheduler` and `Session` time helpers; microtasks/timers remain later runtime slices. |
| Typed mock registry families | Stable Test Mocks | 4 | Scaffolded | Fetch, dialogs, clipboard, location, open, close, print, scroll, matchMedia, downloads, file input, and storage are modeled as separate families with seed, capture, failure, and reset semantics. |
| Public mock actions on `Harness` | Stable Test Mocks | 4 | Scaffolded | Thin wrappers for `Fetch`, `Alert`, `Confirm`, `Prompt`, `ReadClipboard`, `WriteClipboard`, `Open`, `Close`, `Print`, `ScrollTo`, `ScrollBy`, `Navigate`, `SetFiles`, and `CaptureDownload` delegate into the typed registry or runtime. |
| Selector expansion | Stable Core | 6 | Scaffolded | The bounded combinator slice has landed for descendant and child relationships; sibling combinators and bounded pseudo-classes remain planned, guided by `html-standard/`. |
| Script DOM query APIs | Stable Core | 7 | Scaffolded | Host-bound query calls for `querySelector`, `querySelectorAll`, `matches`, and `closest` reuse the shared selector engine, and `querySelectorAll` returns a minimal snapshot `NodeList`; live `HTMLCollection` slices are tracked separately on `children`. |
| Live collections | Stable Core | 7 | Scaffolded | A bounded `HTMLCollection` slice covers `Element.children` and `document.children`, with live length/item/named lookup and snapshot `IDs()` helpers where applicable. |
| Attribute reflection, class, and dataset | Stable Core | 8 | Scaffolded | Bounded reflection helpers for `GetAttribute`, `HasAttribute`, `SetAttribute`, and `RemoveAttribute` are wired through the shared DOM attribute store, and internal bounded `classList` / `dataset` helpers live in `internal/dom`; public facade integration remains later. |
| Tree mutation and HTML serialization | Stable Core | 8 | Scaffolded | Selector-based public wrappers for `InnerHTML`, `OuterHTML`, `SetInnerHTML`, `SetOuterHTML`, `InsertAdjacentHTML`, and `RemoveNode` now delegate into the bounded DOM mutation slice; `cloneNode` stays internal and `document.write`-style replay remains later. |
| Debug view | Experimental Browser Facades | 1 | Scaffolded | `DebugView` is read-only and intentionally limited to inspection data, including configured builder seed visibility, a live DOM dump (`DumpDOM()`), focus state, and interaction logs where that state is otherwise non-observable. |
| Hardening suite | Experimental Project | 5 | Planned | Contract tests, subsystem tests, regression tests, and fuzz/property tests gate publication. |
| Rendering and layout engine | Out of Scope | N/A | Not Planned | No browser renderer or CSS layout engine is intended. |
| External network I/O | Out of Scope | N/A | Not Planned | Network behavior stays mock-driven. |

## Rules

- A capability does not become stable merely because code exists.
- Any new stable `Harness` API requires a matrix update before or with the implementation.
- Any new test-only mock must document seed state, capture behavior, failure injection, reset semantics, and a minimal usage example.
- Any feature that depends on HTML or DOM behavior must be checked against `../html-standard/` first.
- The capability matrix is the source of truth for what the Go workspace promises.
- Legacy and deprecated spec branches are not default targets; only current, bounded, user-visible behavior should be added unless the matrix explicitly calls out a compatibility exception.
