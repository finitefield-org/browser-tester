# Limitations

The Zig rewrite currently provides the phase 0 scaffold plus the internal DOM bootstrap and selector slices of phase 1.
It copies configuration into an owned `Session` and can parse HTML into an internal `DomStore`, but it does not yet expose public DOM read APIs, script execution, events, or runtime mocks.

What exists today:

- `HarnessBuilder`
- `Harness`
- `StorageSeed`
- copied URL, HTML, and local storage seeds
- internal `DomStore` tree construction and `dumpDom()` support for tests
- internal selector subset support for `#id`, tag, selector lists, and bounded attribute selectors
- a small error surface for invalid URLs and allocation failure

What does not exist yet:

- public DOM read APIs
- script parsing and execution
- event dispatch
- timers and microtasks
- public mock families
- rendering or layout
- broad browser compatibility

## Important Consequence

The current public methods are constructor and accessor helpers only.
Anything browser-like beyond that belongs in the next phase, not in ad hoc facade growth.
