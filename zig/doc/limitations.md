# Limitations

The Zig rewrite currently provides the phase 0 scaffold, the internal DOM bootstrap slice of phase 1, the phase 2 script runtime minimum slice, the phase 3 event/default-action and form-control slice, the phase 4 deterministic mock and fake-clock slice, and the phase 6 selector expansion slice.
It copies configuration into an owned `Session`, can parse HTML into an internal `DomStore`, executes inline scripts for the `document.getElementById(...).textContent = ...` slice, and exposes `Harness.assertExists(...)`, `Harness.assertValue(...)`, `Harness.assertChecked(...)`, `Harness.dumpDom(...)`, the phase 3 user-action methods, the phase 4 clock helpers, and the public mock families, but it does not yet expose the later script expansion slices.

What exists today:

- `HarnessBuilder`
- `Harness`
- `StorageSeed`
- `Harness.assertExists(...)`
- `Harness.assertValue(...)`
- `Harness.assertChecked(...)`
- `Harness.click(...)`
- `Harness.typeText(...)`
- `Harness.setChecked(...)`
- `Harness.setSelectValue(...)`
- `Harness.focus(...)`
- `Harness.blur(...)`
- `Harness.submit(...)`
- `Harness.dispatch(...)`
- `Harness.dumpDom(...)`
- `Harness.nowMs(...)`
- `Harness.advanceTime(...)`
- `Harness.flush(...)`
- `Harness.mocksMut(...)`
- `Harness.fetch(...)`
- `Harness.alert(...)`
- `Harness.confirm(...)`
- `Harness.prompt(...)`
- `Harness.readClipboard(...)`
- `Harness.writeClipboard(...)`
- `Harness.captureDownload(...)`
- `Harness.navigate(...)`
- `Harness.setFiles(...)`
- `MockRegistry`
- `FetchMocks`
- `DialogMocks`
- `ClipboardMocks`
- `LocationMocks`
- `DownloadMocks`
- `FileInputMocks`
- `StorageSeeds`
- copied URL, HTML, and local storage seeds
- internal `DomStore` tree construction and `dumpDom()` support for tests
- internal selector expansion support for `#id`, `.class`, tag, compound simple selectors, selector lists, descendant combinators, child combinators, and bounded attribute selectors
- inline script bootstrapping for the text-content mutation slice
- internal event dispatch, default actions, and form-control state
- a small error surface for invalid URLs, malformed HTML, DOM/event/mock semantic failures, script parse/runtime failures, timer failure, and allocation failure

What does not exist yet:

- broader public DOM query or mutation APIs beyond the current inspection, action, and file-input/mock slices
- broader script parsing and execution
- timer scheduling and microtask semantics
- rendering or layout
- broad browser compatibility

## Important Consequence

The current public methods are constructor, inspection, user-action, clock, and mock helpers only.
Anything browser-like beyond that belongs in the next phase, not in ad hoc facade growth.
