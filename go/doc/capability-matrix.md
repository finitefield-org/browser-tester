# Capability Matrix

This matrix defines the target Go implementation.
Nothing in this workspace is implemented yet, so the `Status` column is intentionally future-facing.

| Capability | Level | Phase | Status | Notes |
| --- | --- | --- | --- | --- |
| Public facade scaffold | Stable Core | 0 | Planned | `Harness`, `HarnessBuilder`, `Error`, `DebugView`, and `MockRegistryView` stay thin and delegate into internal packages. |
| Harness constructors | Stable Core | 0 | Planned | `FromHTML`, `FromHTMLWithURL`, `FromHTMLWithLocalStorage`, `FromHTMLWithURLAndLocalStorage`, `FromHTMLWithSessionStorage`, and `FromHTMLWithURLAndSessionStorage` build the initial session snapshot through the builder. |
| Builder configuration capture | Stable Core | 0 | Planned | URL, HTML, local/session storage, random seed, matchMedia rules, and open/close/print/scroll failure seeds are explicit builder fields. |
| Error taxonomy | Stable Core | 0 | Planned | The package classifies HTML parse, script parse/runtime, selector, DOM, event, timer, mock, assertion, and unsupported failures. |
| Harness accessors and views | Stable Core | 0 | Planned | `URL`, `HTML`, `NowMs`, `Debug`, and `Mocks` return session snapshots or views; time and user-like actions are covered by later runtime rows. |
| DOM store and HTML parsing | Stable Core | 1 | Planned | Generational node IDs, tree storage, parsing, and DOM dump helpers live in `internal/dom`. |
| Selector subset and assertions | Stable Core | 1 | Planned | The initial slice covers bounded simple selectors and the `Harness` assertion helpers. |
| Script runtime minimum | Stable Core | 2 | Planned | Inline bootstrap, host bindings, and the minimum script evaluator live in `internal/script`. |
| Event dispatch and default actions | Stable Core | 3 | Planned | User-like actions and bubbling/default-action semantics are owned by runtime. |
| Form controls and selection state | Stable Core | 3 | Planned | Input, textarea, select, and selection state stay deterministic and testable. |
| Deterministic scheduler and time | Stable Core | 4 | Planned | Fake clock, microtasks, timers, and animation-style scheduling live in runtime. |
| Typed mock registry families | Stable Test Mocks | 4 | Planned | Fetch, dialogs, clipboard, location, open, close, print, scroll, matchMedia, downloads, file input, and storage are modeled as separate families with seed, capture, failure, and reset semantics. |
| Public mock actions on `Harness` | Stable Test Mocks | 4 | Planned | `Fetch`, `Alert`, `Confirm`, `Prompt`, `ReadClipboard`, `WriteClipboard`, `Open`, `Close`, `Print`, `ScrollTo`, `ScrollBy`, `Navigate`, `CaptureDownload`, and `SetFiles` stay thin and call the registry or runtime. |
| Selector expansion | Stable Core | 6 | Planned | Class selectors, combinators, and bounded pseudo-classes expand in a controlled slice, guided by `html-standard/`. |
| Script DOM query APIs | Stable Core | 7 | Planned | `querySelector`, `querySelectorAll`, `matches`, `closest`, and the initial live-collection slices reuse the shared selector engine. |
| Live collections | Stable Core | 7 | Planned | `NodeList`, `HTMLCollection`, and related live collections are exposed in bounded slices with iterator helpers and named lookup where applicable. |
| Attribute reflection, class, and dataset | Stable Core | 8 | Planned | `getAttribute`/`setAttribute`-style reflection, `classList`, and `dataset` route through the shared DOM attribute store. |
| Tree mutation and HTML serialization | Stable Core | 8 | Planned | `cloneNode`, insertion/removal helpers, `innerHTML`, `outerHTML`, `insertAdjacentHTML`, and `document.write`-style replay stay bounded and deterministic. |
| Debug view | Experimental Browser Facades | 1 | Planned | `DebugView` is read-only and intentionally limited to inspection data. |
| Hardening suite | Experimental Project | 5 | Planned | Contract tests, subsystem tests, regression tests, and fuzz/property tests gate publication. |
| Rendering and layout engine | Out of Scope | N/A | Not Planned | No browser renderer or CSS layout engine is intended. |
| External network I/O | Out of Scope | N/A | Not Planned | Network behavior stays mock-driven. |

## Rules

- A capability does not become stable merely because code exists.
- Any new stable `Harness` API requires a matrix update before or with the implementation.
- Any new test-only mock must document seed state, capture behavior, failure injection, reset semantics, and a minimal usage example.
- Any feature that depends on HTML or DOM behavior must be checked against `../html-standard/` first.
- The capability matrix is the source of truth for what the Go workspace promises.
