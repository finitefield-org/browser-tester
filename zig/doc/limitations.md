# Limitations

The Zig rewrite currently provides the phase 0 scaffold, the internal DOM bootstrap and selector slices of phase 1, and the phase 2 script runtime minimum slice.
It copies configuration into an owned `Session`, can parse HTML into an internal `DomStore`, executes inline scripts for the `document.getElementById(...).textContent = ...` slice, and exposes `Harness.assertExists(...)` plus `Harness.dumpDom(...)`, but it does not yet expose public DOM query or mutation APIs, broader script runtime features, events, or runtime mocks.

What exists today:

- `HarnessBuilder`
- `Harness`
- `StorageSeed`
- `Harness.assertExists(...)`
- `Harness.dumpDom(...)`
- copied URL, HTML, and local storage seeds
- internal `DomStore` tree construction and `dumpDom()` support for tests
- internal selector subset support for `#id`, tag, selector lists, and bounded attribute selectors
- inline script bootstrapping for the text-content mutation slice
- a small error surface for invalid URLs, malformed HTML, script parse/runtime failures, and allocation failure

What does not exist yet:

- public DOM query or mutation APIs
- broader script parsing and execution
- event dispatch
- timers and microtasks
- public mock families
- rendering or layout
- broad browser compatibility

## Important Consequence

The current public methods are constructor and accessor helpers only.
Anything browser-like beyond that belongs in the next phase, not in ad hoc facade growth.
