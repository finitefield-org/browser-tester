# Capability Matrix

This matrix defines what `next/` already exposes, what is only scaffolded, and what is intentionally out of scope.

| Capability | Level | Phase | Status | Notes |
| --- | --- | --- | --- | --- |
| `HarnessBuilder` and empty session bootstrap | Stable Core | 0 | Available | Builds a session with URL, optional HTML source, and storage seeds. |
| Error taxonomy (`HtmlParse`, `Script`, `Timer`, `Mock`, `Assertion`, ...) | Stable Core | 0 | Available | The classification exists even though most branches are not exercised yet. |
| `DomStore` with generational `NodeId` and side-table skeletons | Internal Only | 0 | Available | Tree building is still placeholder-only. |
| Scheduler clock (`now_ms`, `advance_time`, `flush`) | Stable Core | 0 | Skeleton | Fake-time semantics exist, timer execution policy is still minimal. |
| Typed mock registry families | Stable Test Mocks | 0 | Skeleton | Fetch, dialogs, clipboard, location, downloads, file input, and storage seeds are modeled. |
| HTML parser and DOM tree construction | Stable Core | 1 | Planned | `html(...)` currently stores bootstrap source only. |
| Selector subset (`#id`, tag, attr, combinators) | Stable Core | 1 | Planned | Unsupported today and must fail explicitly. |
| `assert_exists` and DOM assertions | Stable Core | 1-3 | Planned | Assertion API exists only as a gated placeholder. |
| Script lexer / parser / evaluator | Stable Core | 2 | Skeleton | Host-binding seam exists, no language behavior yet. |
| Inline script execution | Stable Core | 2 | Planned | Will be wired through `Session` once DOM and bindings land. |
| Event dispatch and default actions | Stable Core | 3 | Planned | `click`, `type_text`, `focus`, and friends are intentionally gated. |
| Fetch mock response and failure injection | Stable Test Mocks | 4 | Skeleton | Response rules, error rules, and call capture structs exist. |
| Clipboard, dialog, location, download, and file-input capture | Stable Test Mocks | 4 | Skeleton | Families are present; runtime integration is still pending. |
| Debug view | Experimental Browser Facades | 0 | Available | Limited to URL, source HTML, node count, trace flag, and seeded storage. |
| Rendering and layout engine | Out of Scope | N/A | Not Planned | No renderer or CSS layout engine is intended. |
| External network I/O | Out of Scope | N/A | Not Planned | Network behavior is expected to stay mock-driven. |

## Rules

- A capability does not become stable merely because code exists.
- Any new stable `Harness` API requires a matrix update before or with the implementation.
- Any new test-only mock must document response injection, error injection, call or artifact capture, and reset behavior.

