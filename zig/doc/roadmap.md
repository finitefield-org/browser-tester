# Roadmap

This roadmap mirrors [`next.md`](../../next.md) and turns it into the working order for the Zig rewrite.

## Phase 0: Scaffold

Delivered in this workspace:

- project skeleton
- `HarnessBuilder`
- `Harness`
- copied configuration state
- error taxonomy
- design docs

Exit criteria:

- build the owned session snapshot from captured configuration
- keep URL, HTML, and local storage seed configuration
- compile and test the workspace

## Phase 1: DOM Core

Delivered in this workspace:

- HTML parser
- DOM tree construction
- selector subset
- read-only assertions
- DOM dump helpers

Phase 1 is complete in this workspace.

## Phase 2: Script Runtime

- lexer
- parser
- evaluator
- host bindings for `window`, `document`, and `Element`
- inline script execution

Phase 2 is complete in this workspace.

## Phase 3: Events and Forms

Delivered in this workspace:

- event dispatch with bubbling and capture listeners, plus script-side `Element.focus()` / `Element.blur()` focus-state updates
- default actions
- form controls
- `FormData` / `formdata` events (`new FormData(form)`, `FormData.append()` / `delete()` / `get()` / `getAll()` / `has()` / `set()` / `keys()` / `values()` / `entries()` / `forEach(callback[, thisArg])` / `toString()`, and `event.formData` after a non-canceled submit)
- file-input selection snapshots (`input.files`) with `length`, `item(index)`, `keys()` / `values()` / `entries()` / `forEach(callback[, thisArg])`, and `toString()`, remaining wired to `Harness.setFiles(...)` selections
- text selection state on supported `input` / `textarea` controls (`selectionStart`, `selectionEnd`, `selectionDirection`, `setSelectionRange(...)`, `setRangeText(...)`, `select()`, document-level `selectionchange` handlers, and minimal `window.getSelection()` / `document.getSelection()` snapshots with `collapseToStart()` / `collapseToEnd()` / `containsNode(...)` / `removeAllRanges()` / `collapse(node, offset)` / `extend(node, offset)` / `setBaseAndExtent(...)` / `deleteFromDocument()` / `setPosition(node, offset)` / `selectAllChildren(node)` / `addRange(range)` / `removeRange(range)` / `getRangeAt(0)` / `Range.cloneRange()` / `Range.cloneContents()` / `Range.createContextualFragment()` / `Range.selectNode()` / `Range.selectNodeContents()` / `Range.setStart()` / `Range.setEnd()` / `Range.collapse([toStart])` / `Range.insertNode()` / `Range.surroundContents()` / `Range.deleteContents()` / `Range.isPointInRange(...)` / `Range.comparePoint(...)` / `Range.compareBoundaryPoints(...)` / `Range.extractContents()`)
- user-facing `Harness` actions

Phase 3 is complete in this workspace.

## Phase 4: Determinism and Mocks

Delivered in this workspace:

- fake clock helpers (`nowMs`, `advanceTime`, `flush`), one-shot and repeating timer queue semantics, requestAnimationFrame / cancelAnimationFrame frame queue semantics, and queued microtask drain
- typed mock registry (`mocksMut`)
- fetch, clipboard, dialog, open, close, print, scroll, location, matchMedia, and file-input mocks
- download capture

Phase 4 is complete in this workspace.

## Phase 5: Hardening

- contract tests
- regression suite
- property tests
- publication checklist

Phase 5 is complete in this workspace.

## Phase 6: Selector Expansion

- class selectors and compound simple selectors
- descendant combinators
- child combinators
- sibling combinators
- `:scope`
- `:has(...)`
- `:lang(...)` / `:dir(...)`
- `:defined`
- `:not(...)` / `:is(...)` / `:where(...)`
- bounded structural/state pseudo-classes
- `:focus-visible`
- `:blank`
- selector hardening

Phase 6 is complete in this workspace.

## Phase 7: Script DOM Query Expansion

- `document.querySelector`
- `element.querySelector`
- `Element.matches`
- `Element.closest`
- `document.querySelectorAll`
- `element.querySelectorAll`
- minimal `NodeList` collection support
- selector hardening and regression coverage

The query selector and collection slices are implemented in this workspace now.
Phase 7 is complete in this workspace.

## Phase 8: DOM Mutation and Reflection Expansion

- attribute reflection
- class and dataset views
- tree mutation primitives (`insertAdjacentElement()` / `insertAdjacentText()`)
- HTML serialization surfaces
- HTML serialization broadening slice 1 (`insertAdjacentHTML`)
- HTML serialization broadening slice 2 (`template.content.innerHTML` / `DocumentFragment` serialization)
- command-button activation slice (`HTMLButtonElement.command` / `HTMLButtonElement.commandForElement` dispatching `CommandEvent.command` / `CommandEvent.source` through the existing click/default-action path)
- table section access/mutation helpers (`table.caption` / `table.tHead` / `table.tFoot` / `createCaption()` / `deleteCaption()` / `createTHead()` / `deleteTHead()` / `createTFoot()` / `deleteTFoot()` / `createTBody()`) and table section row mutation helpers (`thead.insertRow(...)` / `thead.deleteRow(...)` / `tbody.insertRow(...)` / `tbody.deleteRow(...)` / `tfoot.insertRow(...)` / `tfoot.deleteRow(...)`) plus table row/cell mutation helpers (`table.insertRow(...)` / `table.deleteRow(...)` / `tr.insertCell(...)` / `tr.deleteCell(...)`) and the read-only `HTMLTableRowElement.rowIndex` / `sectionRowIndex` plus `HTMLTableCellElement.cellIndex` and `HTMLTableCellElement.colSpan` / `rowSpan` / `headers` plus `HTMLTableHeaderCellElement.scope` / `abbr` views; add column reflection helpers for `HTMLTableColElement.span` / `width` / `align` / `ch` / `chOff` / `vAlign` / `bgColor` on `col` / `colgroup` and legacy `HTMLTableElement.align` / `border` / `frame` / `rules` / `summary` / `width` / `bgColor` / `cellPadding` / `cellSpacing`, plus the remaining `HTMLTableSectionElement` / `HTMLTableRowElement` / `HTMLTableCellElement` legacy reflection surface

The attribute reflection (`getAttribute`, `getAttributeNS`, `setAttribute`, `setAttributeNS`, `removeAttribute`, `removeAttributeNS`, `hasAttribute`, `hasAttributeNS`, `hasAttributes()`, `getAttributeNames()`, and `toggleAttribute`), including direct reflected `id`, `title`, `slot`, `part`, `lang`, `dir`, `hidden`, `inert`, `translate`, `draggable`, `disabled`, `required`, `noValidate`, `formNoValidate`, `nonce`, `autocapitalize`, `autocomplete`, `autofocus`, `spellcheck`, `inputMode`, `readOnly`, `tabIndex`, `accessKey`, `contentEditable`, and `isContentEditable` state, class/dataset view (with `classList.value`, `classList.item(index)`, `classList.replace()`, `classList.keys()`, `classList.values()`, `classList.entries()`, and `classList.forEach()`), tree mutation, HTML serialization, and namespace-aware serialization slices are implemented in this workspace now, and collection API broadening slices 1 (`NodeList.forEach`), 2 (`document.scripts`), 3 (`document.anchors`), 4 (`NodeList.keys()` / `NodeList.values()` / `HTMLCollection.keys()` / `HTMLCollection.values()` / `entries()`), 5 (`Element.children` / `document.children` / `document.childNodes` / `element.childNodes` / `template.content.childNodes` / `template.content.children`), 6 (`document.forms`), 7 (`form.elements` / `form.length`, including controls associated via `form=` outside the subtree in document order), 8 (`select.options` / `select.length`), 9 (`select.selectedOptions`), 10 (`fieldset.elements`), 11 (`datalist.options`), 12 (`map.areas`), 13 (`table.tBodies`), 14 (`element.labels`), 15 (`document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.applets` / `document.all`), 16 (`document.styleSheets`), 17 (`table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells`), 18 (`getElementsByTagName` / `getElementsByTagNameNS` / `getElementsByClassName` / `getElementsByName`), 19 (`entries()` helpers across `NodeList`, `HTMLCollection`, `StyleSheetList`, and `RadioNodeList`), and 20 (`select.options.add()` / `select.options.remove()`) are implemented as well. The `StyleSheetList`, `CSSRuleList`, and `RadioNodeList` `forEach(callback[, thisArg])` helper parity is implemented too, and `CSSStyleSheet.insertRule()` / `deleteRule()` are available on inline `<style>` sheets, while `CSSMediaRule.insertRule()` / `deleteRule()` mutate top-level `@media` rules in place. `document.open()` / `document.write()` / `document.writeln()` / `document.close()` are also landed as a buffered HTML replay slice that flushes accumulated markup on close, and `HTMLInputElement.list` resolves the associated `datalist` element by `id` or `null`, while `HTMLLegendElement.form` follows the same owner rule when the legend is a direct child of a `fieldset`, `HTMLFieldSetElement.type` returns `"fieldset"`, and `HTMLTrackElement` also exposes modern track reflection for `kind`, `src`, `srclang`, `label`, `default`, `readyState`, and `track` with a minimal `TextTrack` snapshot; `HTMLMediaElement` / `HTMLAudioElement` / `HTMLVideoElement` also expose modern media reflection through `src`, `currentSrc`, `currentTime`, `buffered`, `seekable`, `played`, `textTracks`, `crossOrigin`, `preload`, `autoplay`, `loop`, `controls`, `defaultMuted`, `muted`, `volume`, `defaultPlaybackRate`, `playbackRate`, `preservesPitch`, `disableRemotePlayback`, and `controlsList`, plus deterministic `load()`, `pause()`, and `canPlayType(type)` methods, while `HTMLVideoElement` also exposes `disablePictureInPicture`, `poster`, `playsInline`, `width`, and `height`; `textTracks` is a minimal `TextTrackList` snapshot over descendant `<track>` elements, while cue loading and media-element track integration are not modeled. `HTMLInputElement.accept`, `HTMLInputElement.size`, and `HTMLInputElement.capture` are available on supported inputs as modern file/text entry reflections. `HTMLTemplateElement.shadowRootMode` / `shadowRootDelegatesFocus` / `shadowRootClonable` / `shadowRootSerializable` / `shadowRootCustomElementRegistry` expose declarative shadow-root flags on `<template>` elements while leaving `template.content` on the existing inert `DocumentFragment` slice.
The attribute reflection (`getAttribute`, `getAttributeNS`, `setAttribute`, `setAttributeNS`, `removeAttribute`, `removeAttributeNS`, `hasAttribute`, `hasAttributeNS`, `hasAttributes()`, `getAttributeNames()`, and `toggleAttribute`), including direct reflected `id`, `title`, `slot`, `part`, `lang`, `dir`, `hidden`, `inert`, `translate`, `draggable`, `disabled`, `required`, `noValidate`, `formNoValidate`, `nonce`, `autocapitalize`, `autocomplete`, `autofocus`, `spellcheck`, `inputMode`, `enterKeyHint`, `readOnly`, `tabIndex`, `accessKey`, `contentEditable`, and `isContentEditable` state, class/dataset view (with `classList.value`, `classList.item(index)`, `classList.replace()`, `classList.keys()`, `classList.values()`, `classList.entries()`, and `classList.forEach()`), tree mutation, HTML serialization, and namespace-aware serialization slices are implemented in this workspace now, and collection API broadening slices 1 (`NodeList.forEach`), 2 (`document.scripts`), 3 (`document.anchors`), 4 (`NodeList.keys()` / `NodeList.values()` / `HTMLCollection.keys()` / `HTMLCollection.values()` / `entries()`), 5 (`Element.children` / `document.children` / `document.childNodes` / `element.childNodes` / `template.content.childNodes` / `template.content.children`), 6 (`document.forms`), 7 (`form.elements` / `form.length`, including controls associated via `form=` outside the subtree in document order), 8 (`select.options` / `select.length`), 9 (`select.selectedOptions`), 10 (`fieldset.elements`), 11 (`datalist.options`), 12 (`map.areas`), 13 (`table.tBodies`), 14 (`element.labels`), 15 (`document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.applets` / `document.all`), 16 (`document.styleSheets`), 17 (`table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells`), 18 (`getElementsByTagName` / `getElementsByTagNameNS` / `getElementsByClassName` / `getElementsByName`), 19 (`entries()` helpers across `NodeList`, `HTMLCollection`, `StyleSheetList`, and `RadioNodeList`), and 20 (`HTMLSelectElement.add(option[, before])` / `HTMLSelectElement.remove(index?)` plus `select.options.add()` / `select.options.remove()`) are implemented as well. The `StyleSheetList`, `CSSRuleList`, and `RadioNodeList` `forEach(callback[, thisArg])` helper parity is implemented too, and `CSSStyleSheet.insertRule()` / `deleteRule()` are available on inline `<style>` sheets, while `CSSMediaRule.insertRule()` / `deleteRule()` mutate top-level `@media` rules in place. `document.open()` / `document.write()` / `document.writeln()` / `document.close()` are also landed as a buffered HTML replay slice that flushes accumulated markup on close.
`HTMLDataElement.value` and `HTMLTimeElement.dateTime` are also available as modern machine-readable content reflections on `data` and `time` elements; `time.dateTime` falls back to text content when `datetime` is absent. `HTMLQuoteElement.cite` and `HTMLModElement.cite` / `HTMLModElement.dateTime` are also available as raw quote/mod metadata reflections on `blockquote` / `q` / `ins` / `del` elements.
`HTMLLabelElement.htmlFor` / `control` / `form` are also implemented on label elements, using the existing `for` association and descendant labelable-control resolution, and `form` follows the labeled control's form owner when one exists. HTML serialization now uses browser-style mixed-quote escaping and basic character reference decoding for parsed HTML text and attributes.
The direct reflected attribute set also includes `name`.
The direct reflected attribute set also includes `placeholder`.
The direct reflected attribute set also includes `pattern`.
The direct reflected attribute set also includes `defaultValue` / `defaultChecked` / `min` / `max` / `step` / `noValidate` / `formNoValidate` / `multiple` / `type` for form controls, plus `selected` and `value`, and `select.selectedIndex` / `select.size` / `select.type` are available as the select-side mirrors; `HTMLOptionElement.label` / `defaultSelected` / `text` / `index` and `HTMLOptGroupElement.label` are also reflected on option and optgroup elements; the form submission reflection slice (`form.action`, `form.method`, `form.enctype`, `form.encoding`, `form.target`, `form.acceptCharset`, `form.rel`, `form.relList`, `formAction`, `formMethod`, `formEnctype`, and `formTarget`) is also delivered, with action URLs resolving against the current location and method/enctype staying limited to known values; `form.relList.supports()` is limited to `noreferrer` / `noopener` / `opener`; script-side `form.submit()` / `form.requestSubmit()` / `form.reset()` dispatch `submit` and `reset` events without real navigation while `SubmitEvent.submitter` reflects the explicit submitter on submit events from submit-button clicks and `requestSubmit(submitter)`; form-associated controls also expose read-only `form` owner reflection through the explicit `form` attribute or the nearest owning `form` / `select` chain; `HTMLDialogElement.open` / `HTMLDialogElement.returnValue` / `HTMLDialogElement.closedBy` / `show()` / `showModal()` / `requestClose([returnValue])` / `close([returnValue])` are also delivered as a deterministic cancel/close-event surface, without top-layer rendering or blocking modality; the modern popover API slice (`popover`, `showPopover()`, `hidePopover()`, `togglePopover()`, `popoverTargetElement`, `popoverTargetAction`, and `:popover-open`) is also delivered as a deterministic toggle-event surface with button/input target activation, without top-layer rendering or light-dismiss behavior; `HTMLProgressElement` and `HTMLMeterElement` are also part of the same form-control slice, exposing the modern reflected numeric state (`value`, `max`, `position` on progress; `value`, `min`, `max`, `low`, `high`, `optimum` on meter) plus `form` and `labels`; `HTMLFieldSetElement.type` returns `"fieldset"`; `HTMLOutputElement` is part of the same form-control slice, exposing `type`, `defaultValue`, `value`, `htmlFor`, `form`, `labels`, `willValidate`, `validity`, `validationMessage`, `checkValidity()`, `reportValidity()`, and `setCustomValidity()`; the details element slice (`HTMLDetailsElement.name` / `HTMLDetailsElement.open`) is also delivered as a deterministic toggle-event surface where the first `summary` child toggles `open`, without name-group exclusivity; `form.elements` is live in document order and includes controls associated via `form=` outside the subtree; minimal `checkValidity()` / `reportValidity()` methods are also available on `input`, `textarea`, `select`, `form`, and `output`, with `reportValidity()` dispatching deterministic `invalid` events on invalid controls, and supported form controls also expose `setCustomValidity()` / `validationMessage` / `willValidate` / `validity`, including `typeMismatch` and `patternMismatch` for supported text-like inputs, and supported `number` / `range` / `date` / `datetime-local` / `time` / `month` / `week` controls and `HTMLOutputElement` also expose `valueAsNumber` getters and setters and `valueAsDate` getters and setters where applicable, while `input.accept` and `input.size` are available on supported inputs as modern file/text entry reflections, `input[type=color]` also exposes `alpha` and `colorSpace`, and `input` type=image also exposes modern image-input reflection for `src`, `alt`, `useMap`, `width`, and `height`, `input.indeterminate` feeds the existing `:indeterminate` selector state, `input.dirName` / `textarea.dirName` are available as reflected directionality-name fields on supported text controls, `HTMLSourceElement` exposes modern source reflection for `src`, `srcset`, `sizes`, `media`, and `type`, and `input.showPicker()` / `select.showPicker()` are deterministic no-op surfaces on supported controls.

`HTMLMediaElement` / `HTMLAudioElement` / `HTMLVideoElement` also expose deterministic playback state for `currentTime`, `buffered`, `seekable`, `played`, `duration`, `paused`, `seeking`, `ended`, `readyState`, `networkState`, `muted`, `volume`, `defaultPlaybackRate`, `playbackRate`, and `preservesPitch`, plus the boolean flag `disableRemotePlayback`, while `HTMLVideoElement` also exposes `disablePictureInPicture`; the underlying media pipeline and playback-specific events such as `volumechange` and `ratechange` are not modeled. `buffered` / `seekable` / `played` are exposed as empty `TimeRanges` snapshots.
`HTMLObjectElement` is also part of the same form-control slice, exposing `data`, `type`, `name`, `width`, `height`, `useMap`, `form`, `contentDocument`, `contentWindow`, `getSVGDocument()`, `willValidate`, `validity`, `validationMessage`, `checkValidity()`, `reportValidity()`, and `setCustomValidity()`, with the embedded browsing-context getters modeled as `null` in this workspace.
The detached construction slice (`document.createElement()`, `document.createElementNS()`, `document.createAttribute()`, `document.createAttributeNS()`, `document.createTextNode()`, `document.createComment()`, and `document.createDocumentFragment()`) is also delivered, attribute node accessors (`getAttributeNode()`, `getAttributeNodeNS()`, `setAttributeNode()`, `setAttributeNodeNS()`, and `removeAttributeNode()`) are available, `Element.attributes` exposes a live `NamedNodeMap` surface with `length`, `item(index)`, `getNamedItem(name)`, `getNamedItemNS(namespace, localName)`, `setNamedItem(...)`, `setNamedItemNS(...)`, `removeNamedItem(...)`, `removeNamedItemNS(...)`, `keys()`, `values()`, `entries()`, and `forEach(callback[, thisArg])`, `Element.innerText` and `Element.outerText` are available as deterministic `textContent`-like aliases, `HTMLStyleElement.sheet` / `HTMLLinkElement.sheet` and reflected `media` / `rel` / `crossOrigin` / `disabled` are available on stylesheet owner elements, `HTMLLinkElement` / `HTMLScriptElement` / `HTMLStyleElement` also expose `blocking` as a `DOMTokenList` with supported token `render`, but render-blocking behavior itself is not modeled, and the tree mutation slice now also includes `normalize()`, `document.importNode(...)`, `insertAdjacentElement()` and `insertAdjacentText()`, while `createElementNS()` is still limited to the HTML, SVG, and MathML namespaces.
The stylesheet owner element slice also includes reflected `type`, `hreflang`, `charset`, `imageSrcset`, `imageSizes`, and `fetchPriority` on `HTMLLinkElement`, and `HTMLLinkElement` / `HTMLScriptElement` / `HTMLStyleElement` also expose `blocking` as a `DOMTokenList` with supported token `render`, but render-blocking behavior itself is not modeled.
It also includes reflected `referrerPolicy`, `integrity`, and `as` on `HTMLLinkElement`.
`HTMLMetaElement` also exposes reflected `name`, `content`, `httpEquiv`, `charset`, and `media` metadata on the same modern reflection slice.
The link-action slice also includes reflected `ping` on `HTMLAnchorElement` / `HTMLAreaElement`, `HTMLAnchorElement.text` as a `textContent` alias, and image-map metadata `alt` / `coords` / `shape` / `noHref` on `HTMLAreaElement`.
The tree mutation slice also includes `removeChild()`.
The minimal inline style declaration slice, including semicolon-aware declaration lists, comment stripping, `!important` priority handling, and `getPropertyPriority(...)`, is implemented; the minimal `CSSStyleSheet.cssRules` slice for inline `<style>` sheets is implemented too, including bounded `@media` / `@supports` / `@document` / `@container` / `@starting-style` / `@position-try` / `@scope` / `@keyframes` / `@font-face` / `@font-feature-values` / `@font-palette-values` rules with `CSSFontPaletteValuesRule` exposing `name`, `fontFamily`, `basePalette`, `overrideColors`, and writable `CSSFontPaletteValuesRule.cssText` / `@color-profile` / `@page` / `@layer` block / statement rules / `@property` block rules with writable `CSSPropertyRule.cssText` and `@counter-style` rules with writable `CSSCounterStyleRule.cssText`, plus `@charset` / `@import` / `@namespace` statements, and `CSSStyleRule.style` is available as a live writable `CSSStyleDeclaration` with writable top-level `CSSStyleRule.selectorText`, `CSSStyleRule.cssText`, `CSSPageRule.selectorText` for top-level `@page` rules, `CSSPageRule.cssText`, writable `CSSPageRule.style`, and `CSSRule.type` exposes the legacy CSSOM integer mapping for classic rule kinds (with newer at-rules returning `0`), and `CSSRule.parentStyleSheet` and `CSSRule.parentRule` return the owning stylesheet and owning rule on rule objects, and `CSSStyleSheet.ownerNode` returns the owner element, while `CSSStyleSheet.href` / `CSSStyleSheet.title` / `CSSStyleSheet.disabled` expose owner metadata, and `CSSStyleSheet.media.mediaText` is writable and `CSSStyleSheet.media.appendMedium()` / `deleteMedium()` are available on stylesheet media lists, while `CSSMediaRule.media` is writable and read-only `CSSMediaRule.matches` mirrors the seeded `window.matchMedia(...)` result, and `CSSMediaRule.insertRule()` / `deleteRule()` are available on top-level `@media` rules, and `CSSImportRule.media` is read-only minimal `MediaList` surfaces and `CSSImportRule.styleSheet` / `CSSStyleSheet.ownerRule` stay null linkage surfaces, with `CSSImportRule.supportsText` / `CSSImportRule.layerName` as read-only metadata; legacy `CSSStyleSheet.rules` / `addRule()` / `removeRule()` aliases are available alongside `CSSStyleSheet.insertRule()` / `deleteRule()` / `replaceSync()` on inline `<style>` owners, and stylesheet owner elements now also expose reflected `media`, `rel`, `relList`, `relList.supports()`, `hreflang`, `charset`, `imageSrcset`, `imageSizes`, `fetchPriority`, and `crossOrigin`, while the next named work is broader CSS parsing beyond the bounded selector engine, if a specific user-visible gap needs it. `CSSFontFeatureValuesRule.cssText` and `CSSColorProfileRule.cssText` are writable too.
`CSSStartingStyleRule.cssText` and `CSSPositionTryRule.cssText` are writable too.
`CSSLayerBlockRule.nameText` / `CSSLayerStatementRule.nameText` and writable `CSSLayerBlockRule.cssText` / `CSSLayerStatementRule.cssText` are available on `@layer` rules and rewrite the owning block in place while preserving nested rules; `CSSLayerStatementRule.nameList` exposes the comma-separated names as a `DOMStringList`.
`CSSScopeRule.start` / `CSSScopeRule.end` are available on `@scope` rules; the getters expose the scope root and scope limit strings (or `null` when absent), and `CSSScopeRule.cssText` / `CSSScopeRule.cssRules` remain available on the same slice.
Nested `@keyframes` rules also expose writable `CSSKeyframeRule.keyText` and writable `CSSKeyframesRule.cssText`, plus `CSSKeyframesRule.appendRule()`, `deleteRule()`, and `findRule()`.
`CSSContainerRule` also exposes `containerName` and `containerQuery`, with `containerQuery` returning the specified query text without logical simplification, and `CSSContainerRule.conditionText` is writable on `@container` rules while preserving nested rules, and `CSSContainerRule.cssText` is writable through the same bounded rewrite path; `CSSContainerRule.insertRule()` / `deleteRule()` mutate the nested rule list on top-level `@container` rules. `CSSSupportsConditionRule` exposes `name` and an empty deterministic `cssRules` slice for `@supports-condition` rules, but the inner support-condition block is not semantically interpreted yet. `CSSSupportsRule.conditionText` is writable on `@supports` rules and rewrites the owning block in place while preserving nested rules, and `CSSSupportsRule.cssText` is writable through the same bounded rewrite path; `CSSSupportsRule.insertRule()` / `deleteRule()` mutate the nested rule list on top-level `@supports` rules.
`CSSPageRule.selectorText` is writable for top-level `@page` rules, using the same bounded stylesheet rewrite path as `CSSStyleRule.selectorText`; `CSSPageRule.cssText` is writable through the same bounded stylesheet rewrite path as `CSSStyleRule.cssText`.
`RadioNodeList.value` is writable in the form-elements slice, and unmatched assignments clear the checked radio group in this workspace.

## Phase 9: Document and Window Surface Expansion

- `document.documentElement`, `document.head`, `document.body`, `document.scrollingElement`, `document.activeElement`, `document.referrer`, `document.dir`, `document.visibilityState`, `document.hidden`, `document.hasFocus()`, `document.ownerDocument`, `document.parentNode`, `document.parentElement`, `document.firstElementChild`, `document.lastElementChild`, `document.childElementCount`, `window.children`, `window.frames`, `window.length`, `window.navigator` (`userAgent`, `appCodeName`, `appName`, `appVersion`, `product`, `productSub`, `vendor`, `vendorSub`, `pdfViewerEnabled`, `doNotTrack`, `javaEnabled()`, `plugins`, `platform`, `language`, `cookieEnabled`, `onLine`, `webdriver`, `hardwareConcurrency`, `maxTouchPoints`, `refresh()`), `window.performance` (`now()` / `timeOrigin`), `window.devicePixelRatio`, `window.innerWidth`, `window.innerHeight`, `window.outerWidth`, `window.outerHeight`, `window.scrollX`, `window.scrollY`, `window.pageXOffset`, `window.pageYOffset`, and `window.name`
- `HTMLIFrameElement` modern reflection (`src`, `srcdoc`, `name`, `loading`, `referrerPolicy`, `allow`, `allowFullscreen`, `credentialless`, `width`, `height`, `fetchPriority`, `contentDocument`, and `contentWindow` modeled as `null`)
- `document.title` and `window.title`
- `document.location` and `window.location` as a `Location` host object with `href`, `hash`, `protocol`, `host`, `hostname`, `port`, `username`, `password`, `pathname`, `search`, `assign()`, `replace()`, `reload()`, `toString()`, and `valueOf()`, plus deterministic `hashchange` events, deterministic `popstate` events on history traversal, `window.onhashchange`, `window.onpopstate`, `window.onfocus`, `window.onblur`, `window.onbeforeunload`, `window.onpagehide`, `window.onunload`, `window.onpageshow`, `window.onscroll`, and `document.onscroll`, plus bootstrap completion `readystatechange` events through `document.onreadystatechange` and `load` events through `window.onload`
- `document.URL`, `document.documentURI`, `document.baseURI`, `document.compatMode`, `document.characterSet`, `document.charset`, and `document.contentType`
- `document.origin`, `window.origin`, `Element.baseURI`, and `Element.origin`
- `document.domain` as a deterministic read-only host-derived alias
- `document.cookie` as a deterministic session-owned cookie jar

The document and window alias slice is implemented in this workspace now, including the document metadata, `document.defaultView`, `document.referrer`, `document.dir`, `document.domain`, `document.cookie`, the node reflection helpers (`ownerDocument`, `parentNode`, `parentElement`, `nodeName`, `nodeValue`, `data`, `firstElementChild`, `lastElementChild`, `childElementCount`, `isConnected`, `hasChildNodes()`, `firstChild`, `lastChild`, `nextSibling`, `previousSibling`, `nextElementSibling`, and `previousElementSibling`), `document.scrollingElement`, `document.visibilityState`, `document.hidden`, `document.hasFocus()`, `window.window`, `window.self`, `window.top`, `window.parent`, `window.opener`, `window.frameElement`, `window.closed`, `window.children`, `window.frames`, `window.length`, `window.navigator` (`userAgent`, `appCodeName`, `appName`, `appVersion`, `product`, `productSub`, `vendor`, `vendorSub`, `platform`, `language`, `cookieEnabled`, `onLine`, `webdriver`, `hardwareConcurrency`, `maxTouchPoints`, `javaEnabled()`), `window.performance` (`now()` / `timeOrigin`), `window.devicePixelRatio`, `window.innerWidth`, `window.innerHeight`, `window.outerWidth`, `window.outerHeight`, `window.scrollX`, `window.scrollY`, `window.pageXOffset`, `window.pageYOffset`, and `window.name` aliases used during inline script bootstrap, and `template.content` exposes `firstElementChild`, `lastElementChild`, `childElementCount`, and the same detached fragment traversal helpers.

`window.screen` is also implemented as a deterministic read-only geometry object, including the fixed `orientation.type` / `orientation.angle` pair, `window.Math` is available as a deterministic global host object with constants and `Math.random()`, and the screen-position quartet (`window.screenX`, `window.screenY`, `window.screenLeft`, and `window.screenTop`) is already implemented in Zig as deterministic constants.

## Phase 10: Limited Navigation Model

- `window.history`
- `history.length`, `history.state`, and `history.scrollRestoration`
- `back()`, `forward()`, and `go(delta)`
- `pushState(...)` and `replaceState(...)`

The limited history navigation slice is implemented in this workspace now, and `history.scrollRestoration` is also exposed as a deterministic read/write alias with `auto` / `manual`; history traversal now also dispatches deterministic `popstate` events through `window.onpopstate`, the window focus/blur alias surface dispatches deterministic `focus` / `blur` events through `window.onfocus` and `window.onblur`, the page lifecycle alias surface dispatches deterministic `beforeunload` / `pagehide` / `unload` / `pageshow` events through `window.onbeforeunload`, `window.onpagehide`, `window.onunload`, and `window.onpageshow`, the scroll alias surface dispatches deterministic `scroll` events through `document.onscroll` and `window.onscroll`, and bootstrap completion dispatches deterministic `readystatechange` events through `document.onreadystatechange` plus deterministic `load` events through `window.onload`.
Bootstrap completion also dispatches deterministic `DOMContentLoaded` events before `readystatechange`.

The deterministic mock phase also includes `HarnessBuilder.randomSeed(...)` for seeding the deterministic `Math.random()` and `crypto.randomUUID()` sequences before inline scripts run.
`window.navigator.mimeTypes`, `window.navigator.languages`, and `window.navigator.plugins` are implemented as minimal collection surfaces: `mimeTypes` and `plugins` are `PluginArray`-like surfaces with `length`, `item(index)`, `namedItem(name)`, `keys()`, `values()`, `entries()`, `forEach(callback[, thisArg])`, `refresh()` on `plugins`, and `toString()`, while `languages` is a `DOMStringList`-like surface with `length`, `item(index)`, `contains(value)`, `keys()`, `values()`, `entries()`, `forEach(callback[, thisArg])`, and `toString()`, and the legacy aliases `userLanguage`, `browserLanguage`, `systemLanguage`, and `oscpu` are also part of the implemented slice.
