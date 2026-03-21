# Limitations

The Zig rewrite currently provides the phase 0 scaffold, the internal DOM bootstrap slice of phase 1, the phase 2 script runtime minimum slice, the phase 3 event/default-action and form-control slice, the phase 4 deterministic mock and fake-clock slice, the phase 5 hardening suite, the phase 6 selector expansion slice plus sibling combinators, `:scope`, `:has(...)`, `:lang(...)`, `:dir(...)`, `:not(...)`, `:is(...)`, `:where(...)`, `:defined`, the bounded structural/state pseudo-class slice (`:root`, `:empty`, `:first-child`, `:last-child`, `:only-child`, `:first-of-type`, `:last-of-type`, `:only-of-type`, `:checked`, `:disabled`, `:enabled`, `:required`, `:optional`, `:link`, `:any-link`, `:placeholder-shown`, `:indeterminate`, `:default`, `:valid`, `:invalid`, `:in-range`, `:out-of-range`, `:read-only`, and `:read-write`), and the focus/target/nth pseudo-class slice (`:focus`, `:focus-within`, `:target`, and bounded `:nth-*` forms including `of <selector-list>` support on `:nth-child`, `:nth-last-child`, `:nth-of-type`, and `:nth-last-of-type`), the phase 7 script DOM query and collection slices, the phase 8 attribute reflection, class/dataset view, inline style declaration, tree mutation, HTML serialization, and namespace-aware serialization slices, the phase 9 document/window alias slice plus a limited `Location` host object navigation slice, the phase 10 limited history navigation slice, plus collection API broadening slices 1 (`NodeList.forEach`), 2 (`document.scripts`), 3 (`document.anchors`), 4 (`NodeList.keys()` / `NodeList.values()` / `HTMLCollection.keys()` / `HTMLCollection.values()` / `entries()`), 5 (`Element.children` / `document.children` / `document.childNodes` / `element.childNodes` / `template.content.childNodes` / `template.content.children`), 6 (`document.forms`), 7 (`form.elements`), 8 (`select.options`), 9 (`select.selectedOptions`), 10 (`fieldset.elements`), 11 (`datalist.options`), 12 (`map.areas`), 13 (`table.tBodies`), 14 (`element.labels`), 15 (`document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.applets` / `document.all`), 16 (`document.styleSheets`), 17 (`table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells`), 18 (`getElementsByTagName` / `getElementsByTagNameNS` / `getElementsByClassName` / `getElementsByName`), 19 (`entries()` helpers across `NodeList`, `HTMLCollection`, `StyleSheetList`, and `RadioNodeList`), and 20 (`select.options.add()` / `select.options.remove()`), plus the document metadata / `window.children` / scroll / navigator / performance / viewport / visibility alias slice (`document.compatMode`, `document.characterSet`, `document.charset`, `document.contentType`, `document.referrer`, `document.dir`, `document.activeElement`, `document.visibilityState`, `document.hidden`, `document.hasFocus()`, `window.navigator` (`userAgent`, `appCodeName`, `appName`, `appVersion`, `product`, `productSub`, `vendor`, `vendorSub`, `platform`, `language`, `cookieEnabled`, `onLine`, `webdriver`, `hardwareConcurrency`, `maxTouchPoints`, `javaEnabled()`), `window.performance` (`now()` / `timeOrigin`), `window.devicePixelRatio`, `window.innerWidth`, `window.innerHeight`, `window.outerWidth`, `window.outerHeight`, `window.scrollX`, `window.scrollY`, `window.pageXOffset`, `window.pageYOffset`, `window.name`, and `window.children`), plus script-side `window.localStorage` / `window.sessionStorage` storage surfaces backed by deterministic seeds, and the window identity aliases `document.defaultView`, `window.window`, `window.self`, `window.top`, `window.parent`, `window.opener`, and `window.closed` backed by deterministic constants.

The screen-position aliases (`window.screenX`, `window.screenY`, `window.screenLeft`, and `window.screenTop`) are also provided as deterministic constants and remain part of the same document/window alias slice. `window.screen` is also present as a deterministic read-only `Screen` host object, but it only exposes a small fixed geometry surface rather than a full browser screen model. `document.defaultView`, `window.window`, `window.self`, `window.top`, `window.parent`, `window.opener`, and `window.closed` are also present as deterministic identity aliases (`window.opener` stays `null`, `window.closed` stays `false`).

The open/close/print/scroll test-only mock family is also implemented, with bootstrap failure seeds on `HarnessBuilder` and call capture via `Harness.open(...)`, `Harness.close()`, `Harness.print()`, `Harness.scrollTo(...)`, and `Harness.scrollBy(...)` plus inline `window.open()` / `window.close()` / `window.print()` / `window.scrollTo()` / `window.scrollBy()`.

It copies configuration into an owned `Session`, can parse HTML into an internal `DomStore`, executes inline scripts for the `document.getElementById(...).textContent = ...` slice, and exposes `Harness.assertExists(...)`, `Harness.assertValue(...)`, `Harness.assertChecked(...)`, `Harness.dumpDom(...)`, the phase 3 user-action methods, the phase 4 clock helpers, the public mock families, script-side `querySelector`, `querySelectorAll`, `matches`, and `closest`, including sibling combinators, the universal selector `*`, the `:scope`, `:has(...)`, `:lang(...)`, `:dir(...)`, `:not(...)`, `:is(...)`, `:where(...)`, `:focus`, `:focus-within`, `:target`, and bounded `:nth-*` pseudo-classes, the bounded structural/state pseudo-class slice, script-side class and dataset views (`className`, `classList`, and `dataset`), script-side inline style declaration views (`style`, `cssText`, `getPropertyValue(...)`, `setProperty(...)`, `removeProperty(...)`, `length`, and `item(index)`), script-side attribute reflection methods (`getAttribute`, `setAttribute`, `removeAttribute`, `hasAttribute`, and `toggleAttribute`), script-side tree mutation primitives (`appendChild`, `insertBefore`, `replaceChild`, `replaceChildren`, `append`, `prepend`, `before`, `after`, and `remove`), script-side HTML serialization surfaces (`innerHTML`, `outerHTML`, `insertAdjacentHTML`, and `template.content.innerHTML`) with namespace-aware SVG / MathML name adjustments, bootstrap-only `document.currentScript` / `document.readyState` state, script-side document/window alias surfaces (`document.documentElement`, `document.head`, `document.body`, `document.activeElement`, `document.defaultView`, `document.title`, `document.location`, `document.URL`, `document.documentURI`, `document.baseURI`, `document.compatMode`, `document.characterSet`, `document.charset`, `document.contentType`, `document.referrer`, `document.dir`, `document.origin`, `window.window`, `window.self`, `window.top`, `window.parent`, `window.opener`, `window.closed`, `window.children`, `window.name`, `window.title`, `window.location`, `window.origin`, `Element.baseURI`, and `Element.origin`), plus `NodeList.forEach`, `NodeList.keys()`, `NodeList.values()`, and `NodeList.entries()`, live `document.scripts` / `document.anchors` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `keys()`, `values()`, and `entries()`, live `document.forms`, `form.elements`, `fieldset.elements`, `datalist.options`, `map.areas`, and `table.tBodies` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, where `form.elements.namedItem(name)` returns a `RadioNodeList` when multiple matching controls share a name, live `select.options` and `select.selectedOptions` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, live `element.labels` NodeList support on labelable form controls and `fieldset`, and live `document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.applets` / `document.all` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, live `document.styleSheets` StyleSheetList support with `length`, `item(index)`, `keys()`, `values()`, and `entries()`, live `table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, live child-element / child-node surfaces on `Element`, `Document`, and `template.content` with `length`, `item(index)`, `namedItem(name)` where applicable, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, and live `getElementsByTagName`, `getElementsByTagNameNS`, and `getElementsByClassName` HTMLCollection surfaces plus live `getElementsByName` NodeList support; inline scripts can schedule microtasks via `queueMicrotask()` / `window.queueMicrotask()` and timers via `setTimeout()` / `window.setTimeout()` / `clearTimeout()` / `window.clearTimeout()` / `setInterval()` / `window.setInterval()` / `clearInterval()` / `window.clearInterval()`, and `advanceTime()` / `flush()` drive due timers plus queued microtasks.

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
- `HarnessBuilder.openFailure(...)`
- `HarnessBuilder.closeFailure(...)`
- `HarnessBuilder.printFailure(...)`
- `HarnessBuilder.scrollFailure(...)`
- `Harness.open(...)`
- `Harness.close()`
- `Harness.print()`
- `Harness.scrollTo(...)`
- `Harness.scrollBy(...)`
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
- `OpenCall`
- `OpenMocks`
- `CloseCall`
- `CloseMocks`
- `PrintCall`
- `PrintMocks`
- `ScrollMethod`
- `ScrollCall`
- `ScrollMocks`
- `LocationMocks`
- `MatchMediaMocks`
- `DownloadMocks`
- `FileInputMocks`
- `StorageSeeds`
- copied URL, HTML, local storage seeds, and session storage seeds
- internal `DomStore` tree construction and `dumpDom()` support for tests
- internal selector expansion support for `#id`, `.class`, tag, compound simple selectors, selector lists, descendant combinators, child combinators, sibling combinators, bounded attribute selectors, `:scope`, `:has(...)`, `:lang(...)`, `:dir(...)`, `:not(...)`, `:is(...)`, `:where(...)`, `:focus`, `:focus-within`, `:target`, `:defined`, bounded `:nth-*` forms, and the bounded structural/state pseudo-class slice
- script-side query selector expansion for `document.querySelector`, `element.querySelector`, `Element.matches`, and `Element.closest`, including sibling combinators, `:scope`, `:has(...)`, `:lang(...)`, `:dir(...)`, `:not(...)`, `:is(...)`, `:where(...)`, `:focus`, `:focus-within`, `:target`, `:defined`, bounded `:nth-*` forms, and the bounded structural/state pseudo-class slice
- script-side collection expansion for `document.querySelectorAll`, `element.querySelectorAll`, and minimal `NodeList` snapshots with `length`, `item(index)`, `forEach(callback[, thisArg])`, `keys()`, and `values()`, plus live `document.scripts` and `document.anchors` HTMLCollection support with `length`, `item(index)`, `namedItem(name)`, `keys()`, `values()`, and `entries()`, `document.forms` / `form.elements` / `fieldset.elements` / `datalist.options` / `map.areas` / `table.tBodies` HTMLCollection support with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, and `values()`, `select.options` / `select.selectedOptions` HTMLCollection support with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, and `values()`, live `element.labels` NodeList support on labelable form controls and `fieldset`, live `document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.applets` / `document.all` HTMLCollection support with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, and `values()`, and live `Element.children` / `document.children` / `document.childNodes` / `element.childNodes` / `template.content.childNodes` / `template.content.children` surfaces with `length`, `item(index)`, `namedItem(name)` where applicable, `forEach(callback[, thisArg])`, `keys()`, and `values()`, plus the `getElementsByTagName` / `getElementsByTagNameNS` / `getElementsByClassName` / `getElementsByName` collection family
- `form.elements.namedItem(name)` can surface `RadioNodeList` objects with writable `value` semantics; assigning a missing value clears the checked radio group in this workspace
- script-side class and dataset views for `className`, `classList`, and `dataset`
- script-side inline style declaration views for `Element.style` / `CSSStyleDeclaration`, including `cssText`, `getPropertyValue(...)`, `getPropertyPriority(...)`, `setProperty(...)`, `removeProperty(...)`, `length`, `item(index)`, and property reflection for simple declaration lists with comment stripping and `!important` priority handling
- script-side attribute reflection methods for `getAttribute`, `setAttribute`, `removeAttribute`, `hasAttribute`, and `toggleAttribute`
- script-side tree mutation primitives for `appendChild`, `insertBefore`, `replaceChild`, `replaceChildren`, `append`, `prepend`, `before`, `after`, and `remove`
- script-side HTML serialization surfaces for `innerHTML`, `outerHTML`, `insertAdjacentHTML`, and `template.content.innerHTML`
- inline script bootstrapping for the text-content mutation slice
- internal event dispatch, default actions, and form-control state
- a small error surface for invalid URLs, malformed HTML, DOM/event/mock semantic failures, script parse/runtime failures, timer failure, and allocation failure

What does not exist yet:

- broader public DOM query or mutation APIs beyond the current inspection, action, and file-input/mock slices
- broader HTML serialization surfaces beyond `innerHTML`, `outerHTML`, `insertAdjacentHTML`, and `template.content.innerHTML`
- broader collection APIs beyond `NodeList.forEach`, `NodeList.keys()`, `NodeList.values()`, `document.scripts`, `document.anchors`, `element.labels`, `document.images`, `document.links`, `document.embeds`, `document.plugins`, `document.applets`, `document.all`, `document.styleSheets`, `table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells`, the current `children` / `childNodes` live collection surfaces, and the `getElementsByTagName` / `getElementsByTagNameNS` / `getElementsByClassName` / `getElementsByName` family; the inline style declaration surface is intentionally bounded to simple declaration lists with comment stripping and `!important` priority handling, including `getPropertyPriority(...)`, and broader CSS parsing beyond the bounded selector engine remains deferred until a specific user-visible gap needs it
- broader script parsing and execution
- timer coverage includes one-shot `setTimeout()` / `clearTimeout()` and repeating `setInterval()` / `clearInterval()`; a public scheduler API remains out of scope
- rendering or layout
- broad browser compatibility

## Important Consequence

The current public methods are constructor, inspection, user-action, clock, and mock helpers only.
Anything browser-like beyond that belongs in the next phase, not in ad hoc facade growth.
