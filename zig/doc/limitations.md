# Limitations

The Zig rewrite currently only provides the phase 0 scaffold.
It copies configuration into an owned `Session`, but it does not yet implement DOM, selectors, script execution, events, or runtime mocks.

What exists today:

- `HarnessBuilder`
- `Harness`
- `StorageSeed`
- copied URL, HTML, and local storage seeds
- a small error surface for invalid URLs and allocation failure

What does not exist yet:

- DOM tree construction
- selector matching
- script parsing and execution
- event dispatch
- timers and microtasks
- public mock families
- rendering or layout
- broad browser compatibility

## Important Consequence

The current public methods are constructor and accessor helpers only.
Anything browser-like beyond that belongs in the next phase, not in ad hoc facade growth.

