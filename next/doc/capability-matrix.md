# Capability Matrix

This matrix defines what `next/` already exposes, what is only scaffolded, and what is intentionally out of scope.

| Capability | Level | Phase | Status | Notes |
| --- | --- | --- | --- | --- |
| `HarnessBuilder` and empty session bootstrap | Stable Core | 0 | Available | Builds a session with URL, parsed optional HTML source, and storage seeds. |
| Error taxonomy (`HtmlParse`, `Script`, `Timer`, `Mock`, `Assertion`, ...) | Stable Core | 0 | Available | The classification exists even though most branches are not exercised yet. |
| `DomStore` with generational `NodeId` and side-table skeletons | Internal Only | 0 | Available | Tree building, selector-backed traversal, and minimal `textContent` mutation now exist; side tables are still sparse. |
| Scheduler clock (`now_ms`, `advance_time`, `flush`) | Stable Core | 0 | Available | Fake-time semantics now drain due timers deterministically and `flush` clears the remaining timer and microtask queue. |
| Typed mock registry families | Stable Test Mocks | 0 | Available | Fetch, dialogs, clipboard, location, file input, downloads, and storage seeds are modeled, with public mock actions wired for the browser-service families, including download capture. |
| HTML parser and DOM tree construction | Stable Core | 1 | Available | `html(...)` now builds a real tree and rejects malformed markup. |
| Selector subset (`#id`, tag, attr) | Stable Core | 1 | Available | `#id`, tag, and `[attr]` are supported as the original Phase 1 subset. |
| Selector slice 1 (`.class`, `tag.class`, `#id.class`) | Stable Core | 6 | Available | The shared selector engine now resolves class selectors and compound simple selectors through existing `Harness` APIs. |
| Selector slice 2 (descendant combinators) | Stable Core | 6 | Available | `A B` resolves nested matches in document order through the same selector engine. |
| Selector slice 3 (child combinators) | Stable Core | 6 | Available | `A > B` resolves direct children through the same selector engine. |
| Selector backlog slices (`A + B`, `A ~ B`) | Stable Core | post-7 | Available | Immediate and general sibling matching now reuse the same bounded selector engine. |
| Selector hardening and bounded selector grammar | Stable Core | 6 | Available | unsupported syntax continues to fail explicitly, and no broader CSS parsing is intended. |
| Script DOM query slice 1 (`document.querySelector`, `element.querySelector`) | Stable Core | 7 | Available | Script-side selector lookup reuses the bounded engine and returns the first match in document order or `null` on miss. |
| Script DOM query slice 2 (`Element.matches`) | Stable Core | 7 | Available | Current-element selector checks now reuse the bounded selector engine and return a boolean. |
| Script DOM query slice 3 (`Element.closest`) | Stable Core | 7 | Available | Ancestor-walk selector lookup now reuses the bounded selector engine and returns the nearest match or `null`. |
| Script DOM query slice 4 (`querySelectorAll`, minimal `NodeList`) | Stable Core | post-7 | Available | Document- and element-scoped selector collections now expose `length` and `item()`; selector lists are supported by the bounded engine, while `HTMLCollection` and broader collection APIs stay out of scope. |
| Script DOM query hardening (pseudo-classes deferred) | Stable Core | 7 | Available | Unsupported selector syntax remains explicit, selector lists are supported, and broader CSS parsing stays out of scope. |
| `assert_exists` and DOM assertions | Stable Core | 1 | Available | `assert_exists` queries the DOM and includes a dump in failure messages. |
| Script lexer / parser / evaluator | Stable Core | 2 | Available | Minimal statement/expression support powers inline DOM mutation, selector lookup, ancestor-walk lookup, and listener registration. |
| Window/document/Element host bindings | Stable Core | 2 | Available | `document.getElementById`, `document.querySelector`, `element.querySelector`, `Element.matches`, `Element.closest`, `textContent` mutation, and listener registration are wired through `Session`. |
| Inline script execution | Stable Core | 2 | Available | Inline `<script>` blocks execute during session bootstrap in document order. |
| Event dispatch and default actions | Stable Core | 3 | Available | Target-phase dispatch, ancestor bubbling, capture listeners, checkbox toggles, cancelable submit/click default actions, and form actions (`click`, `type_text`, `set_checked`, `set_select_value`, `focus`, `blur`, `submit`, `dispatch`) are available. |
| Fetch mock response and failure injection | Stable Test Mocks | 4 | Available | Response rules, error rules, and call capture are wired through `Harness::fetch(...)`. |
| Clipboard, dialog, location, file-input, and download capture | Stable Test Mocks | 4 | Available | Public actions are wired and each family exposes deterministic capture through the registry. |
| Download artifact capture | Stable Test Mocks | 4 | Available | `Harness::capture_download(...)` writes artifacts into the registry, and callers inspect them through `downloads().artifacts()`. |
| Hardening suite and publication checklist | Experimental Project | 5 | Available | Contract tests, subsystem tests, regression tests, property tests, and `./scripts/test-quick.sh` / `./scripts/test-hardening.sh` define the Phase 5 release gate. |
| Debug view | Experimental Browser Facades | 1 | Available | Limited to URL, source HTML, node count, DOM dump, trace flag, and seeded storage. |
| Rendering and layout engine | Out of Scope | N/A | Not Planned | No renderer or CSS layout engine is intended. |
| External network I/O | Out of Scope | N/A | Not Planned | Network behavior is expected to stay mock-driven. |

## Rules

- A capability does not become stable merely because code exists.
- Any new stable `Harness` API requires a matrix update before or with the implementation.
- Any new test-only mock must document response injection, error injection, call or artifact capture, and reset behavior.
