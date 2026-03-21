# Limitations

The Zig rewrite currently provides the phase 0 scaffold, the internal DOM bootstrap slice of phase 1, the phase 2 script runtime minimum slice, the phase 3 event/default-action and form-control slice, the phase 4 deterministic mock and fake-clock slice, the phase 5 hardening suite, the phase 6 selector expansion slice plus sibling combinators, the phase 7 script DOM query and collection slices, and the phase 8 attribute reflection, class/dataset view, tree mutation, HTML serialization, and namespace-aware serialization slices, plus collection API broadening slices 1 (`NodeList.forEach`), 2 (`document.scripts`), 3 (`document.anchors`), 4 (`NodeList.keys()` / `NodeList.values()` / `HTMLCollection.keys()` / `HTMLCollection.values()`), 5 (`Element.children` / `document.children` / `document.childNodes` / `element.childNodes` / `template.content.childNodes` / `template.content.children`), 6 (`document.forms`), 7 (`form.elements`), 8 (`select.options`), 9 (`select.selectedOptions`), 10 (`fieldset.elements`), 11 (`datalist.options`), 12 (`map.areas`), 13 (`table.tBodies`), 14 (`element.labels`), 15 (`document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.applets` / `document.all`), 16 (`document.styleSheets`), 17 (`table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells`), and 18 (`getElementsByTagName` / `getElementsByTagNameNS` / `getElementsByClassName` / `getElementsByName`).
It copies configuration into an owned `Session`, can parse HTML into an internal `DomStore`, executes inline scripts for the `document.getElementById(...).textContent = ...` slice, and exposes `Harness.assertExists(...)`, `Harness.assertValue(...)`, `Harness.assertChecked(...)`, `Harness.dumpDom(...)`, the phase 3 user-action methods, the phase 4 clock helpers, the public mock families, script-side `querySelector`, `querySelectorAll`, `matches`, and `closest`, including sibling combinators, script-side class and dataset views (`className`, `classList`, and `dataset`), script-side attribute reflection methods (`getAttribute`, `setAttribute`, `removeAttribute`, `hasAttribute`, `toggleAttribute`), script-side tree mutation primitives (`appendChild`, `insertBefore`, `replaceChild`, `replaceChildren`, `append`, `prepend`, `before`, `after`, and `remove`), script-side HTML serialization surfaces (`innerHTML`, `outerHTML`, `insertAdjacentHTML`, and `template.content.innerHTML`) with namespace-aware SVG / MathML name adjustments, plus `NodeList.forEach`, `NodeList.keys()`, `NodeList.values()`, live `document.scripts` / `document.anchors` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `keys()`, and `values()`, live `document.forms`, `form.elements`, `fieldset.elements`, `datalist.options`, `map.areas`, and `table.tBodies` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, and `values()`, where `form.elements.namedItem(name)` returns a `RadioNodeList` when multiple matching controls share a name, live `select.options` and `select.selectedOptions` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, and `values()`, live `element.labels` NodeList support on labelable form controls and `fieldset`, and live `document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.applets` / `document.all` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, and `values()`, live `document.styleSheets` StyleSheetList support with `length` and `item(index)`, live `table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, and `values()`, live child-element / child-node surfaces on `Element`, `Document`, and `template.content` with `length`, `item(index)`, `namedItem(name)` where applicable, `forEach(callback[, thisArg])`, `keys()`, and `values()`, and live `getElementsByTagName`, `getElementsByTagNameNS`, and `getElementsByClassName` HTMLCollection surfaces plus live `getElementsByName` NodeList support, but it does not yet expose the bounded pseudo-class slices.

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
- internal selector expansion support for `#id`, `.class`, tag, compound simple selectors, selector lists, descendant combinators, child combinators, sibling combinators, and bounded attribute selectors
- script-side query selector expansion for `document.querySelector`, `element.querySelector`, `Element.matches`, and `Element.closest`, including sibling combinators
- script-side collection expansion for `document.querySelectorAll`, `element.querySelectorAll`, and minimal `NodeList` snapshots with `length`, `item(index)`, `forEach(callback[, thisArg])`, `keys()`, and `values()`, plus live `document.scripts` and `document.anchors` HTMLCollection support with `length`, `item(index)`, `namedItem(name)`, `keys()`, and `values()`, `document.forms` / `form.elements` / `fieldset.elements` / `datalist.options` / `map.areas` / `table.tBodies` HTMLCollection support with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, and `values()`, `select.options` / `select.selectedOptions` HTMLCollection support with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, and `values()`, live `element.labels` NodeList support on labelable form controls and `fieldset`, live `document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.applets` / `document.all` HTMLCollection support with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, and `values()`, and live `Element.children` / `document.children` / `document.childNodes` / `element.childNodes` / `template.content.childNodes` / `template.content.children` surfaces with `length`, `item(index)`, `namedItem(name)` where applicable, `forEach(callback[, thisArg])`, `keys()`, and `values()`, plus the `getElementsByTagName` / `getElementsByTagNameNS` / `getElementsByClassName` / `getElementsByName` collection family
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
- broader collection APIs beyond `NodeList.forEach`, `NodeList.keys()`, `NodeList.values()`, `document.scripts`, `document.anchors`, `element.labels`, `document.images`, `document.links`, `document.embeds`, `document.plugins`, `document.applets`, `document.all`, `document.styleSheets`, `table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells`, the current `children` / `childNodes` live collection surfaces, and the `getElementsByTagName` / `getElementsByTagNameNS` / `getElementsByClassName` / `getElementsByName` family; next slice: the bounded pseudo-class slices
- bounded pseudo-class slices
- broader script parsing and execution
- timer scheduling and microtask semantics
- rendering or layout
- broad browser compatibility

## Important Consequence

The current public methods are constructor, inspection, user-action, clock, and mock helpers only.
Anything browser-like beyond that belongs in the next phase, not in ad hoc facade growth.
