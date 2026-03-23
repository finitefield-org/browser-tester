# Limitations

The direct reflected attribute set also includes `name`.
The direct reflected attribute set also includes `placeholder`.
The direct reflected attribute set also includes `pattern`.
The direct reflected attribute set also includes `slot` and `part`.
The direct reflected attribute set also includes `defaultValue` / `defaultChecked` / `min` / `max` / `step` / `minLength` / `maxLength` / `noValidate` / `formNoValidate` for form controls, plus `selected` and `value`, and `select.selectedIndex` stays aligned with `select.value` / `select.selectedOptions`; form-associated controls also expose read-only `form` owner reflection through the explicit `form` attribute or the nearest owning `form` / `select` chain, and `form.elements` stays in document order while including controls associated via `form=` outside the subtree. Supported `number` / `range` / `date` / `datetime-local` / `time` / `month` / `week` controls also expose `valueAsNumber` getters and setters and `valueAsDate` getters and setters.
`HTMLLabelElement.htmlFor` / `control` are available on label elements, using the same explicit `for` association and implicit descendant labelable-control resolution as the `element.labels` surface.
Form submission reflection is also available on `form` and submit controls through `action`, `method`, `enctype`, `encoding`, `target`, `acceptCharset`, `formAction`, `formMethod`, `formEnctype`, and `formTarget`, and `form.submit()` / `form.requestSubmit()` / `form.reset()` dispatch deterministic `submit` and `reset` events, but actual submission / navigation remains mock-driven and does not post data to a network endpoint.

Form-control validity methods (`checkValidity()` / `reportValidity()`) are also available on `input`, `textarea`, `select`, and `form`; supported form controls also expose `setCustomValidity()` / `validationMessage` / `willValidate` / `validity`, they reuse the same minimal built-in constraint state that powers `:valid` / `:invalid`, including `typeMismatch` and `patternMismatch` for supported text-like inputs, supported `number` / `range` / `date` / `datetime-local` / `time` / `month` / `week` controls also expose `valueAsNumber` getters and setters and `valueAsDate` getters and setters, and the broader `ValidityState` surface remains absent.

The screen-position aliases (`window.screenX`, `window.screenY`, `window.screenLeft`, and `window.screenTop`) are also provided as deterministic constants and remain part of the same document/window alias slice. `window.screen` is also present as a deterministic read-only `Screen` host object, but it only exposes a small fixed geometry surface rather than a full browser screen model; the only orientation data exposed is the fixed `orientation.type` / `orientation.angle` pair. `window.Math` is also present as a deterministic global host object, but only the standard constants and `Math.random()` are currently provided, and `Math.random()` follows the same deterministic workspace-local sequence in every harness instance. `window.crypto` is also present as a deterministic global host object, but only `randomUUID()` is currently provided, and `randomUUID()` follows the same deterministic workspace-local sequence in every harness instance. `document.defaultView`, `window.window`, `window.self`, `window.top`, `window.parent`, `window.opener`, and `window.closed` are also present as deterministic identity aliases (`window.opener` stays `null`, `window.closed` stays `false`).
`HTMLStyleElement.sheet` / `HTMLLinkElement.sheet` are also available, and `style.media` / `link.media` / `link.rel` / `link.relList` / `link.relList.supports()` / `link.relList.replace()` / `link.as` / `link.charset` / `link.imageSrcset` / `link.imageSizes` / `link.fetchPriority` / `link.hreflang` / `link.crossOrigin` / `link.integrity` / `link.type` are reflected metadata on stylesheet owner elements. `CSSStyleSheet.media.mediaText` is writable, while `CSSMediaRule.media` and `CSSImportRule.media` remain read-only `MediaList` surfaces. `window.onpageshow` and `window.onpagehide` are also exposed as deterministic page lifecycle handlers for explicit navigation and reload, `window.onscroll` and `document.onscroll` are exposed as deterministic scroll event handlers for explicit `scrollTo()` / `scrollBy()` calls, but the workspace does not model renderer-driven scrolling.
`link.referrerPolicy` is also reflected on `HTMLLinkElement`.

`document.domain` is also deterministic, but intentionally minimal: reads derive the host component from the current URL, and assignments fail explicitly.
`document.cookie` is also deterministic, but intentionally minimal: reads come from the session-owned cookie jar, assignments accept only simple `name=value` forms, and malformed writes fail explicitly.

The phase 8 detached construction primitives (`document.createElement()`, `document.createElementNS()`, `document.createAttribute()`, `document.createAttributeNS()`, `document.createTextNode()`, `document.createComment()`, and `document.createDocumentFragment()`) are also implemented; they create detached nodes, detached attribute objects, or fragment-like containers and rely on the existing mutation helpers when they are later attached to the tree. Attribute node accessors (`getAttributeNode()`, `getAttributeNodeNS()`, `setAttributeNode()`, `setAttributeNodeNS()`, and `removeAttributeNode()`) are available too, but `createElementNS()` is still limited to the HTML, SVG, and MathML namespaces.

The open/close/print/scroll test-only mock family is also implemented, with bootstrap failure seeds on `HarnessBuilder` and call capture via `Harness.open(...)`, `Harness.close()`, `Harness.print()`, `Harness.scrollTo(...)`, and `Harness.scrollBy(...)` plus inline `window.open()` / `window.close()` / `window.print()` / `window.scrollTo()` / `window.scrollBy()`.

It copies configuration into an owned `Session`, can parse HTML into an internal `DomStore`, executes inline scripts for the `document.getElementById(...).textContent = ...` slice, and exposes `Harness.assertExists(...)`, `Harness.assertValue(...)`, `Harness.assertChecked(...)`, `Harness.dumpDom(...)`, the phase 3 user-action methods, the phase 4 clock helpers, the public mock families, script-side `querySelector`, `querySelectorAll`, `matches`, and `closest`, plus `template.content.querySelector(All)` / `template.content.getElementById()`, including sibling combinators, the universal selector `*`, the `:scope`, `:has(...)`, `:lang(...)`, `:dir(...)`, `:not(...)`, `:is(...)`, `:where(...)`, `:focus`, `:focus-within`, `:target`, and bounded `:nth-*` pseudo-classes, the bounded structural/state pseudo-class slice including `:blank`, script-side class and dataset views (`className`, `classList` (including `value`, `item(index)`, `replace()`, `keys()`, `values()`, `entries()`, and `forEach()`), and `dataset`), script-side inline style declaration views (`style`, `cssText`, `getPropertyValue(...)`, `setProperty(...)`, `removeProperty(...)`, `length`, and `item(index)`), script-side attribute reflection methods (`getAttribute`, `getAttributeNS`, `setAttribute`, `setAttributeNS`, `removeAttribute`, `removeAttributeNS`, `hasAttribute`, `hasAttributeNS`, `hasAttributes()`, `getAttributeNames()`, and `toggleAttribute`), `Element.attributes` as a minimal read-only `NamedNodeMap` snapshot with `keys()`, `values()`, and `entries()`, `Element.innerText` and `Element.outerText` as deterministic `textContent`-like aliases on Element nodes, script-side tree mutation primitives (`appendChild`, `insertBefore`, `replaceChild`, `replaceWith`, `replaceChildren`, `append`, `prepend`, `before`, `after`, and `remove`), script-side HTML serialization surfaces (`innerHTML`, `outerHTML`, `insertAdjacentHTML`, and `template.content.innerHTML`) with namespace-aware SVG / MathML name adjustments, bootstrap-only `document.currentScript` / `document.readyState` / `document.onreadystatechange` state, script-side document/window alias surfaces (`document.documentElement`, `document.head`, `document.body`, `document.scrollingElement`, `document.activeElement`, `document.defaultView`, `document.title`, `document.location`, `document.URL`, `document.documentURI`, `document.baseURI`, `document.compatMode`, `document.characterSet`, `document.charset`, `document.contentType`, `document.referrer`, `document.dir`, `document.origin`, `window.window`, `window.self`, `window.top`, `window.parent`, `window.opener`, `window.closed`, `window.children`, `window.frames`, `window.length`, `window.name`, `window.title`, `window.location`, `window.origin`, `Element.baseURI`, and `Element.origin`), plus `NodeList.forEach`, `NodeList.keys()`, `NodeList.values()`, and `NodeList.entries()`, live `document.scripts` / `document.anchors` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `keys()`, `values()`, and `entries()`, live `document.forms`, `form.elements`, `fieldset.elements`, `datalist.options`, `map.areas`, and `table.tBodies` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, where `form.elements.namedItem(name)` returns a `RadioNodeList` when multiple matching controls share a name, read-only `form` owner reflection on form-associated controls, live `select.options` and `select.selectedOptions` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, live `element.labels` NodeList support on labelable form controls and `fieldset`, and live `document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.applets` / `document.all` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, live `document.styleSheets` StyleSheetList support with `length`, `item(index)`, `keys()`, `values()`, and `entries()`, live `table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, and live child-element / child-node surfaces on `Element`, `Document`, and `template.content` with `length`, `item(index)`, `namedItem(name)` where applicable, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`; `template.content` also exposes `firstElementChild`, `lastElementChild`, `childElementCount`, `isConnected`, and the same detached fragment sibling helpers, and live `getElementsByTagName`, `getElementsByTagNameNS`, and `getElementsByClassName` HTMLCollection surfaces plus live `getElementsByName` NodeList support; inline scripts can schedule microtasks via `queueMicrotask()` / `window.queueMicrotask()` and timers via `setTimeout()` / `window.setTimeout()` / `clearTimeout()` / `window.clearTimeout()` / `setInterval()` / `window.setInterval()` / `clearInterval()` / `window.clearInterval()`, and `advanceTime()` / `flush()` drive due timers, animation frames, plus queued microtasks. `click()` also drives anchor navigation/download/reset observation deterministically, and bootstrap completion now dispatches deterministic `readystatechange` events through `document.onreadystatechange`.

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
- `Element.focus(...)` / `Element.blur(...)` only update the deterministic focused-node state; they do not model the full browser focus algorithm
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
- file-input selections are exposed to inline scripts through a minimal read-only `input.files` snapshot rather than a full `FileList` / `File` model
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
- internal selector expansion support for `#id`, `.class`, tag, compound simple selectors, selector lists, descendant combinators, child combinators, sibling combinators, bounded attribute selectors, `:scope`, `:has(...)`, `:lang(...)`, `:dir(...)`, `:not(...)`, `:is(...)`, `:where(...)`, `:focus`, `:focus-visible`, `:focus-within`, `:target`, `:defined`, bounded `:nth-*` forms, and the bounded structural/state pseudo-class slice including `:blank`
- script-side query selector expansion for `document.querySelector`, `element.querySelector`, `Element.matches`, and `Element.closest`, including sibling combinators, `:scope`, `:has(...)`, `:lang(...)`, `:dir(...)`, `:not(...)`, `:is(...)`, `:where(...)`, `:focus`, `:focus-visible`, `:focus-within`, `:target`, `:defined`, bounded `:nth-*` forms, and the bounded structural/state pseudo-class slice including `:blank`
- script-side collection expansion for `document.querySelectorAll`, `element.querySelectorAll`, and `template.content.querySelector(All)` / `template.content.getElementById()` plus minimal `NodeList` snapshots with `length`, `item(index)`, `forEach(callback[, thisArg])`, `keys()`, and `values()`, plus live `document.scripts` and `document.anchors` HTMLCollection support with `length`, `item(index)`, `namedItem(name)`, `keys()`, `values()`, and `entries()`, `document.forms` / `form.elements` / `fieldset.elements` / `datalist.options` / `map.areas` / `table.tBodies` HTMLCollection support with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, and `values()`, where `form.elements` stays in document order and includes controls associated via `form=` outside the subtree, `select.options` / `select.selectedOptions` HTMLCollection support with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, and `values()`, live `element.labels` NodeList support on labelable form controls and `fieldset`, live `document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.applets` / `document.all` HTMLCollection support with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, and `values()`, and live `Element.children` / `document.children` / `document.childNodes` / `element.childNodes` / `template.content.childNodes` / `template.content.children` surfaces with `length`, `item(index)`, `namedItem(name)` where applicable, `forEach(callback[, thisArg])`, `keys()`, and `values()`, plus the `getElementsByTagName` / `getElementsByTagNameNS` / `getElementsByClassName` / `getElementsByName` collection family
- `form.elements.namedItem(name)` can surface `RadioNodeList` objects with writable `value` semantics; assigning a missing value clears the checked radio group in this workspace
- script-side class and dataset views for `className`, `classList` (including `value`, `item(index)`, `replace()`, `keys()`, `values()`, `entries()`, and `forEach()`), and `dataset`
- script-side inline style declaration views for `Element.style` / `CSSStyleDeclaration`, including `cssText`, `getPropertyValue(...)`, `getPropertyPriority(...)`, `setProperty(...)`, `removeProperty(...)`, `length`, `item(index)`, and property reflection for semicolon-aware declaration lists with comment stripping and `!important` priority handling
- script-side attribute reflection methods for `getAttribute`, `getAttributeNS`, `setAttribute`, `setAttributeNS`, `removeAttribute`, `removeAttributeNS`, `hasAttribute`, `hasAttributeNS`, `hasAttributes()`, `getAttributeNames()`, and `toggleAttribute`, plus `Element.attributes` as a minimal read-only `NamedNodeMap` snapshot with `keys()`, `values()`, and `entries()`, plus direct reflected `id`, `title`, `role`, `slot`, `part`, `ariaLabel`, `ariaDescription`, `ariaRoleDescription`, `ariaHidden`, `lang`, `dir`, `hidden`, `inert`, `translate`, `draggable`, `disabled`, `required`, `noValidate`, `formNoValidate`, `name`, `placeholder`, `nonce`, `autocapitalize`, `spellcheck`, `inputMode`, `tabIndex`, `accessKey`, `contentEditable`, and `isContentEditable` state, plus read-only `form` owner reflection on form-associated controls
- script-side tree mutation primitives for `cloneNode()`, `normalize()`, `document.importNode(...)`, `appendChild`, `insertBefore`, `replaceChild`, `replaceWith`, `replaceChildren`, `append`, `prepend`, `before`, `after`, `insertAdjacentElement(...)`, `insertAdjacentText(...)`, and `remove`
- script-side HTML serialization surfaces for `innerHTML`, `outerHTML`, `insertAdjacentHTML`, `template.content.innerHTML`, and the buffered `document.open()` / `document.write()` / `document.writeln()` / `document.close()` replay slice
- inline script bootstrapping for the text-content mutation slice
- internal event dispatch, default actions, form-control state, and text-selection state, including document-level `selectionchange` handlers from the supported selection APIs
- a small error surface for invalid URLs, malformed HTML, DOM/event/mock semantic failures, script parse/runtime failures, timer failure, and allocation failure

What does not exist yet:

- broader public DOM query or mutation APIs beyond the current inspection, action, and file-input/mock slices
- broader HTML serialization surfaces beyond `innerHTML`, `outerHTML`, `insertAdjacentHTML`, `template.content.innerHTML`, and the buffered `document.open()` / `document.write()` / `document.writeln()` / `document.close()` replay slice
- broader collection APIs beyond `NodeList.forEach`, `NodeList.keys()`, `NodeList.values()`, `document.scripts`, `document.anchors`, `element.labels`, `document.images`, `document.links`, `document.embeds`, `document.plugins`, `document.applets`, `document.all`, `document.styleSheets`, `table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells`, the current `children` / `childNodes` live collection surfaces, and the `getElementsByTagName` / `getElementsByTagNameNS` / `getElementsByClassName` / `getElementsByName` family; `StyleSheetList.forEach`, `CSSRuleList.forEach`, and `RadioNodeList.forEach` are already implemented, the inline style declaration surface is intentionally bounded to semicolon-aware declaration lists with comment stripping and `!important` priority handling, including `getPropertyPriority(...)`, `CSSStyleRule.style` is a read-only snapshot `CSSStyleDeclaration`, `CSSPageRule.style` is a read-only snapshot `CSSStyleDeclaration`, `CSSRule.type` exposes the legacy CSSOM integer mapping for classic rule kinds (with newer at-rules returning `0`), `CSSRule.parentStyleSheet` is available on rule objects, `CSSRule.parentRule` is available on nested rule objects, `document.styleSheets.cssRules` is intentionally bounded to simple qualified rules plus bounded `@media` / `@supports` / `@container` / `@starting-style` / `@position-try` / `@scope` / `@keyframes` / `@font-face` / `@font-feature-values` / `@font-palette-values` rules with `CSSFontPaletteValuesRule` exposing `name`, `fontFamily`, `basePalette`, `overrideColors`, and `cssText` / `@color-profile` / `@page` / `@layer` block / statement rules / `@property` block rules / `@counter-style` rules with `name`, `system`, `symbols`, `negative`, `prefix`, `suffix`, `range`, `pad`, `fallback`, `speakAs`, `additiveSymbols`, and `cssText`, plus `@import` / `@namespace` statements for inline `<style>` sheets, and `CSSStyleSheet.insertRule(...)` / `deleteRule(...)` / `replaceSync()` are available for those same inline `<style>` owners, `CSSStyleSheet.media.appendMedium()` / `deleteMedium()` are available on stylesheet media lists, while `CSSMediaRule.media` / `CSSImportRule.media` remain read-only `MediaList` surfaces, and `matchMedia()` returns deterministic `MediaQueryList` objects whose `matches` property mirrors the current seeded mock state with legacy `addListener()` / `removeListener()` hooks plus `addEventListener('change', ...)` / `removeEventListener('change', ...)` and `onchange`, while broader CSS parsing beyond the bounded selector engine remains deferred until a specific user-visible gap needs it; `navigator.plugins` is currently a minimal `PluginArray`-like surface with deterministic `length`, `item(index)`, `namedItem(name)`, `refresh()`, and `toString()` over the embedded elements in the document, `window.frameElement` is a top-level-only null alias, and stylesheet owner element `disabled` reflection is available on `HTMLStyleElement` / `HTMLLinkElement`
- broader script parsing and execution
- timer coverage includes one-shot `setTimeout()` / `clearTimeout()`, repeating `setInterval()` / `clearInterval()`, and `requestAnimationFrame()` / `cancelAnimationFrame()`; a public scheduler API remains out of scope
- rendering or layout
- broad browser compatibility

## Important Consequence

The current public methods are constructor, inspection, user-action, clock, and mock helpers only.
Anything browser-like beyond that belongs in the next phase, not in ad hoc facade growth.

`HarnessBuilder.randomSeed(...)` can override the default `Math.random()` and `crypto.randomUUID()` seeds before bootstrap, but both sequences remain deterministic for a given seed.
`window.navigator.languages` is also deterministic, but intentionally minimal: it is exposed as a `DOMStringList`-like surface with `length`, `item(index)`, `contains(value)`, `keys()`, `values()`, `entries()`, and `toString()`, and the legacy aliases `userLanguage`, `browserLanguage`, `systemLanguage`, and `oscpu` are fixed to deterministic read-only values.
