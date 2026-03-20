# Capability Matrix

This matrix classifies the current public surface of `zig/` by support level.

| Capability | Level | Phase | Status | Notes |
| --- | --- | --- | --- | --- |
| `HarnessBuilder` configuration capture | Stable Core | 0 | Available | Collects URL, HTML, and local storage seeds before ownership is transferred into `Session`. |
| `Harness` constructors | Stable Core | 0 | Available | `fromHtml`, `fromHtmlWithUrl`, `fromHtmlWithLocalStorage`, and `fromHtmlWithUrlAndLocalStorage` are available. |
| `Harness` accessors | Stable Core | 0 | Available | `url()`, `html()`, and `localStorage()` expose the copied session snapshot. |
| `StorageSeed` | Stable Core | 0 | Available | Represents deterministic local-storage seed pairs. |
| `Error` and `Result(T)` | Stable Core | 0 | Available | `error.InvalidUrl`, `error.InvalidSelector`, `error.AssertionFailed`, `error.ScriptParse`, `error.ScriptRuntime`, and `error.OutOfMemory` cover the public surface; `error.HtmlParse` remains for malformed HTML. |
| `Session` | Internal Only | 0 | Available | Internal copied state with an arena-owned lifetime. It now also owns the internal DOM store and script runtime state. |
| Reserved mock registry slot | Internal Only | 0 | Reserved | The workspace is already shaped for a future mock registry, but no public mock family is exposed yet. |
| HTML parsing and DOM tree construction | Internal Only | 1 | Available | `src/dom.zig` parses HTML into `DomStore` and rejects malformed markup explicitly. |
| Selector subset | Internal Only | 1 | Available | `DomStore.select()` resolves `#id`, tag, selector lists, and bounded attribute selectors for the internal DOM slice. |
| DOM dump helpers | Internal Only | 1 | Available | `dumpDom()` is used for tree snapshots in tests and is not exposed publicly. |
| Harness read-only inspection | Stable Core | 1 | Available | `assertExists()` validates selector presence and `dumpDom()` returns the textual DOM snapshot. |
| Script runtime internals | Internal Only | 2 | Available | `src/script.zig` owns the lexer, parser, evaluator, and host-binding seam used during inline script bootstrap. |
| Script runtime minimum slice | Stable Core | 2 | Available | Inline `<script>` bootstrapping can resolve `document.getElementById(...)` and mutate `textContent` during harness construction; missing element access and unsupported syntax fail explicitly. |
| Event dispatch and default actions | Stable Core | 3 | Planned | User-like interactions are deferred until the DOM and script layers are trustworthy. |
| Deterministic mocks | Stable Test Mocks | 4 | Planned | Public mock families will be documented once they are promoted. |
| Hardening suite | Experimental Project | 5 | Planned | Contract, regression, and property coverage will be added after the public surface stabilizes. |
| Rendering and layout engine | Out of Scope | N/A | Not Planned | This workspace does not aim to become a browser renderer. |
| External network I/O | Out of Scope | N/A | Not Planned | Network behavior is expected to stay mock-driven. |

## Rules

- A capability does not become stable merely because code exists.
- Any new public `Harness` API requires a matrix update before or with the implementation.
- Any new test-only mock must document response injection, failure injection, capture behavior, and reset semantics.
- Planned rows describe the next named milestone and are not user-facing guarantees.
