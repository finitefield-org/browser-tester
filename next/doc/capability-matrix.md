# Capability Matrix

This matrix defines what `next/` already exposes, what is only scaffolded, and what is intentionally out of scope.

| Capability | Level | Phase | Status | Notes |
| --- | --- | --- | --- | --- |
| `HarnessBuilder` and empty session bootstrap | Stable Core | 0 | Available | Builds a session with URL, parsed optional HTML source, and storage seeds. |
| Error taxonomy (`HtmlParse`, `Script`, `Timer`, `Mock`, `Assertion`, ...) | Stable Core | 0 | Available | The classification exists even though most branches are not exercised yet. |
| `DomStore` with generational `NodeId` and side-table skeletons | Internal Only | 0 | Available | Tree building, selector-backed traversal, and minimal `textContent` mutation now exist; side tables are still sparse. |
| Scheduler clock (`now_ms`, `advance_time`, `flush`) | Stable Core | 0 | Skeleton | Fake-time semantics exist, timer execution policy is still minimal. |
| Typed mock registry families | Stable Test Mocks | 0 | Skeleton | Fetch, dialogs, clipboard, location, downloads, file input, and storage seeds are modeled. |
| HTML parser and DOM tree construction | Stable Core | 1 | Available | `html(...)` now builds a real tree and rejects malformed markup. |
| Selector subset (`#id`, tag, attr, combinators) | Stable Core | 1 | Available | `#id`, tag, and `[attr]` are supported; combinators fail explicitly. |
| `assert_exists` and DOM assertions | Stable Core | 1 | Available | `assert_exists` queries the DOM and includes a dump in failure messages. |
| Script lexer / parser / evaluator | Stable Core | 2 | Available | Minimal statement/expression support powers inline DOM mutation and listener registration. |
| Window/document/Element host bindings | Stable Core | 2 | Available | `document.getElementById`, `textContent` mutation, and listener registration are wired through `Session`. |
| Inline script execution | Stable Core | 2 | Available | Inline `<script>` blocks execute during session bootstrap in document order. |
| Event dispatch and default actions | Stable Core | 3 | Available | Target-phase dispatch, checkbox toggles, submit-button default actions, and form actions (`click`, `type_text`, `set_checked`, `submit`, `dispatch`) are available; `focus`, `blur`, and `set_select_value` remain gated. |
| Fetch mock response and failure injection | Stable Test Mocks | 4 | Skeleton | Response rules, error rules, and call capture structs exist. |
| Clipboard, dialog, location, download, and file-input capture | Stable Test Mocks | 4 | Skeleton | Families are present; runtime integration is still pending. |
| Debug view | Experimental Browser Facades | 1 | Available | Limited to URL, source HTML, node count, DOM dump, trace flag, and seeded storage. |
| Rendering and layout engine | Out of Scope | N/A | Not Planned | No renderer or CSS layout engine is intended. |
| External network I/O | Out of Scope | N/A | Not Planned | Network behavior is expected to stay mock-driven. |

## Rules

- A capability does not become stable merely because code exists.
- Any new stable `Harness` API requires a matrix update before or with the implementation.
- Any new test-only mock must document response injection, error injection, call or artifact capture, and reset behavior.
