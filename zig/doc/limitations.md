# Limitations

The Zig rewrite currently provides the phase 0 scaffold, the internal DOM bootstrap slice of phase 1, the phase 2 script runtime minimum slice, the phase 3 event/default-action and form-control slice, the phase 4 deterministic mock and fake-clock slice, the phase 5 hardening suite, the phase 6 selector expansion slice, the phase 7 script DOM query and collection slices, and the phase 8 attribute reflection, class/dataset view, tree mutation, HTML serialization, and namespace-aware serialization slices, plus collection API broadening slices 1 (`NodeList.forEach`) and 2 (`document.scripts`).
It copies configuration into an owned `Session`, can parse HTML into an internal `DomStore`, executes inline scripts for the `document.getElementById(...).textContent = ...` slice, and exposes `Harness.assertExists(...)`, `Harness.assertValue(...)`, `Harness.assertChecked(...)`, `Harness.dumpDom(...)`, the phase 3 user-action methods, the phase 4 clock helpers, the public mock families, script-side `querySelector`, `querySelectorAll`, `matches`, and `closest`, script-side class and dataset views (`className`, `classList`, and `dataset`), script-side attribute reflection methods (`getAttribute`, `setAttribute`, `removeAttribute`, `hasAttribute`, `toggleAttribute`), script-side tree mutation primitives (`appendChild`, `insertBefore`, `replaceChild`, `replaceChildren`, `append`, `prepend`, `before`, `after`, and `remove`), script-side HTML serialization surfaces (`innerHTML`, `outerHTML`, `insertAdjacentHTML`, and `template.content.innerHTML`) with namespace-aware SVG / MathML name adjustments, plus `NodeList.forEach` and a live `document.scripts` HTMLCollection surface, but it does not yet expose collection APIs beyond those or broader selector grammar slices.

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
- script-side query selector expansion for `document.querySelector`, `element.querySelector`, `Element.matches`, and `Element.closest`
- script-side collection expansion for `document.querySelectorAll`, `element.querySelectorAll`, and minimal `NodeList` snapshots with `length`, `item(index)`, and `forEach(callback[, thisArg])`, plus live `document.scripts` HTMLCollection support with `length`, `item(index)`, and `namedItem(name)`
- script-side class and dataset views for `className`, `classList`, and `dataset`
- script-side attribute reflection methods for `getAttribute`, `setAttribute`, `removeAttribute`, `hasAttribute`, and `toggleAttribute`
- script-side tree mutation primitives for `appendChild`, `insertBefore`, `replaceChild`, `replaceChildren`, `append`, `prepend`, `before`, `after`, and `remove`
- script-side HTML serialization surfaces for `innerHTML`, `outerHTML`, `insertAdjacentHTML`, and `template.content.innerHTML`
- inline script bootstrapping for the text-content mutation slice
- internal event dispatch, default actions, and form-control state
- a small error surface for invalid URLs, malformed HTML, DOM/event/mock semantic failures, script parse/runtime failures, timer failure, and allocation failure

What does not exist yet:

- broader public DOM query or mutation APIs beyond the current inspection, action, and file-input/mock slices
- broader HTML serialization surfaces beyond `innerHTML`, `outerHTML`, `insertAdjacentHTML`, and `template.content.innerHTML`
- broader collection APIs beyond `NodeList.forEach` and `document.scripts`
- broader selector grammar slices
- broader script parsing and execution
- timer scheduling and microtask semantics
- rendering or layout
- broad browser compatibility

## Important Consequence

The current public methods are constructor, inspection, user-action, clock, and mock helpers only.
Anything browser-like beyond that belongs in the next phase, not in ad hoc facade growth.
