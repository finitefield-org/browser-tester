# browser-tester zig

This directory is a clean-room Zig rewrite workspace for `browser-tester`.
The rewrite follows [`next.md`](../next.md) and keeps the public surface small while the internal phases are built out.

Current state:

- `HTMLMapElement.name` is reflected on `<map>` elements, and `map.areas` stays the live `HTMLCollection` of descendant `<area>` elements.
- `HTMLFormElement.autocomplete` and `HTMLSelectElement.autocomplete` reflect the autofill hint surface with `on` / `off` semantics, matching the spec's missing-value default on forms and the select-level hint surface.
- phase 0 scaffold plus the internal phase 1 DOM bootstrap slice, the phase 2 script runtime minimum slice, the phase 3 event/default-action and form-control slice plus the text-selection state slice (`selectionStart`, `selectionEnd`, `selectionDirection`, `setSelectionRange(...)`, `setRangeText(...)`, `select()`, `document.createRange()`, document-level `selectionchange` handlers, and minimal `window.getSelection()` / `document.getSelection()` snapshots with `collapseToStart()` / `collapseToEnd()` / `containsNode(...)` / `removeAllRanges()` / `collapse(node, offset)` / `extend(node, offset)` / `setBaseAndExtent(...)` / `deleteFromDocument()` / `setPosition(node, offset)` / `selectAllChildren(node)` / `addRange(range)` / `removeRange(range)` / `getRangeAt(0)` / `Range.cloneRange()` / `Range.cloneContents()` / `Range.createContextualFragment()` / `Range.selectNode()` / `Range.selectNodeContents()` / `Range.setStart()` / `Range.setEnd()` / `Range.collapse([toStart])` / `Range.insertNode()` / `Range.deleteContents()` / `Range.isPointInRange(...)` / `Range.comparePoint(...)` / `Range.compareBoundaryPoints(...)` / `Range.extractContents()`), plus `input.valueAsNumber` / `input.valueAsDate` on supported `number` / `range` / `date` / `datetime-local` / `time` / `month` / `week` controls and `input.showPicker()` on supported `input` / `select` controls as a deterministic no-op surface, the phase 4 deterministic mock and fake-clock slice, the phase 5 hardening suite, the phase 6 selector expansion slice, the phase 6 `:scope` pseudo-class slice, the phase 6 `:has(...)` pseudo-class slice, the phase 6 `:lang(...)` / `:dir(...)` pseudo-class slice, the phase 6 bounded structural/state pseudo-class slice (`:root`, `:empty`, `:first-child`, `:last-child`, `:only-child`, `:first-of-type`, `:last-of-type`, `:only-of-type`, `:checked`, `:disabled`, `:enabled`, `:required`, `:optional`, `:link`, `:any-link`, `:placeholder-shown`, `:blank`, `:indeterminate`, `:default`, `:valid`, `:invalid`, `:in-range`, `:out-of-range`, `:read-only`, and `:read-write`), the phase 6 `:defined` pseudo-class slice, the `:not(...)` / `:is(...)` / `:where(...)` selector-list pseudo-class slice, the focus/target/nth pseudo-class slice (`:focus`, `:focus-visible`, `:focus-within`, `:target`, and bounded `:nth-*` forms including `of <selector-list>` support on `:nth-child`, `nth-last-child`, `nth-of-type`, and `nth-last-of-type`), the phase 7 script DOM query and collection slices (`document.querySelector`, `document.querySelectorAll`, `element.querySelector`, `element.querySelectorAll`, `Element.matches`, and `Element.closest`), the phase 8 attribute reflection slice (`getAttribute`, `getAttributeNS`, `setAttribute`, `setAttributeNS`, `removeAttribute`, `removeAttributeNS`, `hasAttribute`, `hasAttributeNS`, `hasAttributes()`, `getAttributeNames()`, and `toggleAttribute`), including direct reflected `id`, `title`, `lang`, `dir`, `defaultValue`, `defaultChecked`, `minLength`, `maxLength`, and `multiple` state, the phase 8 class and dataset views slice (`className`, `classList` (including `value`, `item()`, and `replace()`), and `dataset`), the phase 8 inline style declaration slice (`style`, `cssText`, `getPropertyValue`, `setProperty`, `removeProperty`, `length`, and `item`), the phase 8 tree mutation primitives slice (`cloneNode()`, `normalize()`, `document.importNode(...)`, `appendChild`, `insertBefore`, `replaceChild`, `removeChild`, `replaceWith`, `replaceChildren`, `append`, `prepend`, `before`, `after`, `insertAdjacentElement(...)`, `insertAdjacentText(...)`, and `remove`), the phase 8 collection API broadening slices 1 (`NodeList.forEach`), 2 (`document.scripts`), 3 (`document.anchors`), 4 (`NodeList.keys()` / `NodeList.values()` / `HTMLCollection.keys()` / `HTMLCollection.values()` / `entries()`), 5 (`Element.children` / `document.children` / `document.childNodes` / `element.childNodes` / `template.content.childNodes` / `template.content.children`), 6 (`document.forms`), 7 (`form.elements` / `form.length`, including controls associated via `form=` outside the subtree in document order), 8 (`select.options` / `select.length`), 9 (`select.selectedOptions`), 10 (`fieldset.elements`), 11 (`datalist.options`), 12 (`map.areas`), 13 (`table.tBodies`), 14 (`element.labels`), 15 (`document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.applets` / `document.all`), 16 (`document.styleSheets`), 17 (`table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells`), 18 (`getElementsByTagName` / `getElementsByTagNameNS` / `getElementsByClassName` / `getElementsByName`), 19 (`entries()` helpers across `NodeList`, `HTMLCollection`, `StyleSheetList`, and `RadioNodeList`), and 20 (`select.options.add()` / `select.options.remove()`), plus sibling combinators (`A + B`, `A ~ B`) and the `Location` host object with `href`, `hash`, `protocol`, `host`, `hostname`, `port`, `username`, `password`, `pathname`, `search`, `assign()`, `replace()`, `reload()`, `toString()`, and `valueOf()`, while the stylesheet owner surface now also includes legacy `CSSStyleSheet.rules` / `addRule()` / `removeRule()` aliases alongside `insertRule()` / `deleteRule()` / `replaceSync()`, and the minimal buffered `document.open()` / `document.write()` / `document.writeln()` / `document.close()` slice is available for HTML replay during bootstrap, and bootstrap completion now dispatches deterministic `readystatechange` events through `document.onreadystatechange`, plus `HTMLFieldSetElement.type` returns `"fieldset"` on fieldset elements.
- `HTMLProgressElement` / `HTMLMeterElement` are part of the same form-control family: `HTMLProgressElement.value` / `max` / `position` and `HTMLMeterElement.value` / `min` / `max` / `low` / `high` / `optimum` are reflected, and both elements expose `form` and `labels`.
- `HTMLFieldSetElement.form` follows the same form-owner rule as the other form-associated controls, alongside `HTMLFieldSetElement.type`, `disabled`, and `elements`.
- The form submission slice also includes `new FormData(form)` and `formdata` events: `FormData` supports `append()` / `delete()` / `get()` / `getAll()` / `has()` / `set()` / `keys()` / `values()` / `entries()` / `forEach(callback[, thisArg])` / `toString()`, `event.formData` exposes the captured payload, and `formdata` fires only after a non-canceled submit.
- navigator collection helpers expose `forEach(callback[, thisArg])` on `window.navigator.mimeTypes`, `window.navigator.languages`, and `window.navigator.plugins`, alongside the existing `keys()` / `values()` / `entries()` / `toString()` helpers and the deterministic `refresh()` helper on `plugins`
- `stepUp()` / `stepDown()` are available on supported `number` / `range` / `date` / `datetime-local` / `time` / `month` / `week` controls, using the same reflected `min` / `max` / `step` / `value` state and rejecting unsupported controls explicitly.
- `click()` also drives anchor / area navigation, download, and target-open observation deterministically, and script-side `HTMLElement.click()` routes through the same default-action path; bootstrap completion now dispatches deterministic `readystatechange` events through `document.onreadystatechange`; script-side `HTMLAnchorElement.href` / `HTMLAnchorElement.rel` / `HTMLAnchorElement.relList` / `HTMLAnchorElement.relList.add()` / `HTMLAnchorElement.relList.remove()` / `HTMLAnchorElement.relList.toggle()` / `HTMLAnchorElement.relList.replace()` / `HTMLAnchorElement.download` / `HTMLAnchorElement.target` / `HTMLAnchorElement.ping` / `HTMLAnchorElement.hreflang` / `HTMLAnchorElement.referrerPolicy` / `HTMLAnchorElement.type` and `HTMLAreaElement.href` / `HTMLAreaElement.rel` / `HTMLAreaElement.relList` / `HTMLAreaElement.relList.add()` / `HTMLAreaElement.relList.remove()` / `HTMLAreaElement.relList.toggle()` / `HTMLAreaElement.relList.replace()` / `HTMLAreaElement.download` / `HTMLAreaElement.target` / `HTMLAreaElement.ping` / `HTMLAreaElement.alt` / `HTMLAreaElement.coords` / `HTMLAreaElement.shape` / `HTMLAreaElement.noHref` / `HTMLAreaElement.hreflang` / `HTMLAreaElement.referrerPolicy` / `HTMLAreaElement.type` reflection is available for the same link-action slice, `HTMLAnchorElement.text` is exposed as a `textContent` alias on anchors, and `HTMLButtonElement.command` / `HTMLButtonElement.commandForElement` / `HTMLButtonElement.type` / `HTMLButtonElement.value` cover the deterministic command-button slice.
- bootstrap completion also dispatches deterministic `DOMContentLoaded` events before `readystatechange`.
- `HTMLSelectElement.add(option[, before])` / `HTMLSelectElement.remove(index?)` mutate the same live option state as `select.options.add()` / `select.options.remove()`, and `HTMLOptionElement.label` / `defaultSelected` / `text` / `index` plus `HTMLOptGroupElement.label` are reflected on option and optgroup elements.
- `HTMLTableElement.caption` / `HTMLTableElement.tHead` / `HTMLTableElement.tFoot`, `HTMLTableElement.createCaption()` / `deleteCaption()` / `createTHead()` / `deleteTHead()` / `createTBody()` / `createTFoot()` / `deleteTFoot()`, `HTMLTableSectionElement.insertRow([index])` / `HTMLTableSectionElement.deleteRow([index])`, and `HTMLTableElement.insertRow([index])` / `HTMLTableElement.deleteRow([index])` plus `HTMLTableRowElement.insertCell([index])` / `HTMLTableRowElement.deleteCell([index])` mutate the same live table section and row/cell collections, while `HTMLTableRowElement.rowIndex` / `sectionRowIndex`, `HTMLTableCellElement.cellIndex`, and `HTMLTableCellElement.colSpan` / `rowSpan` / `headers` plus `HTMLTableHeaderCellElement.scope` / `abbr` expose the live positions and reflection state of those collections, `HTMLTableColElement.span` / `width` / `align` / `ch` / `chOff` / `vAlign` / `bgColor` are reflected on `col` / `colgroup` elements, `HTMLTableElement.align` / `border` / `frame` / `rules` / `summary` / `width` / `bgColor` / `cellPadding` / `cellSpacing` are reflected on `table` elements with legacy null-to-empty behavior for `bgColor` / `cellPadding` / `cellSpacing`, and `HTMLTableSectionElement` / `HTMLTableRowElement` / `HTMLTableCellElement` expose the rest of the legacy table reflection surface (`align` / `ch` / `chOff` / `vAlign` on sections, `align` / `ch` / `chOff` / `vAlign` / `bgColor` on rows, and `align` / `axis` / `height` / `width` / `ch` / `chOff` / `noWrap` / `vAlign` / `bgColor` on cells).
- `Range.selectNode()` / `Range.selectNodeContents()` / `Range.setStart()` / `Range.setEnd()` / `Range.collapse([toStart])` / `Range.insertNode()` / `Range.surroundContents()` are available on the same minimal `Range` snapshot surface as the other selection helpers.
- phase 8 detached construction slice (`document.createElement()`, `document.createElementNS()`, `document.createAttribute()`, `document.createAttributeNS()`, `document.createTextNode()`, `document.createComment()`, and `document.createDocumentFragment()`), plus attribute node accessors (`getAttributeNode()`, `getAttributeNodeNS()`, `setAttributeNode()`, `setAttributeNodeNS()`, and `removeAttributeNode()`), and a live `Element.attributes` `NamedNodeMap` surface with `length`, `item(index)`, `getNamedItem(name)`, `getNamedItemNS(namespace, localName)`, `setNamedItem(...)`, `setNamedItemNS(...)`, `removeNamedItem(...)`, `removeNamedItemNS(...)`, `keys()`, `values()`, `entries()`, and `forEach(callback[, thisArg])`, with `createElementNS()` currently limited to the HTML, SVG, and MathML namespaces
- direct reflected `Element` attributes `id`, `title`, `role`, `slot`, `part`, `ariaLabel`, `ariaDescription`, `ariaRoleDescription`, `ariaHidden`, `lang`, `dir`, `hidden`, `inert`, `translate`, `draggable`, `disabled`, `required`, `noValidate`, `formNoValidate`, `name`, `dirName`, `defaultValue`, `defaultChecked`, `min`, `max`, `step`, `minLength`, `maxLength`, `multiple`, `type`, `placeholder`, `pattern`, `nonce`, `autocapitalize`, `autocomplete`, `autofocus`, `spellcheck`, `inputMode`, `readOnly`, `tabIndex`, and `accessKey` are available, and the reflected `contentEditable` / `isContentEditable` state is available on `Element` as well; form submission reflection is available on `form` / submit controls through `action`, `method`, `enctype`, `encoding`, `target`, `acceptCharset`, `name`, `rel`, `relList`, `formAction`, `formMethod`, `formEnctype`, and `formTarget`, with action URLs resolving against the current location and method/enctype limited to known values, while `form.relList.add()` / `remove()` / `toggle()` / `replace()` / `supports()` are available with `supports()` limited to `noreferrer` / `noopener` / `opener`, `form.submit()` / `form.requestSubmit()` / `form.reset()` dispatch `submit` and `reset` events without modeling real network submission or navigation, and `SubmitEvent.submitter` reflects the explicit submitter on submit events from submit-button clicks and `requestSubmit(submitter)`; form owner reflection is also available on form-associated controls through `form`, with `HTMLLegendElement.form` following the same owner rule when the legend is a direct child of a `fieldset`, and with explicit `form` attributes taking precedence over the nearest owning `form` or `select` chain; `HTMLDialogElement.open` / `HTMLDialogElement.returnValue` / `HTMLDialogElement.closedBy` / `show()` / `showModal()` / `requestClose([returnValue])` / `close([returnValue])` are available on dialog elements as a deterministic close/cancel-event slice, and `HTMLDetailsElement.name` / `HTMLDetailsElement.open` are available on details elements as a deterministic toggle-event slice where the first `summary` child toggles `open`; top-layer rendering, blocking modal behavior, and details-name-group exclusivity are not modeled; file inputs also expose a minimal read-only `files` snapshot that mirrors `Harness.setFiles(...)`; supported form controls also expose a minimal `ValidityState` snapshot via `validity`, including `typeMismatch` and `patternMismatch` for supported text-like inputs
- `HTMLImageElement` exposes modern image reflection through `src`, `srcset`, `sizes`, `width`, `height`, `loading`, `decoding`, `currentSrc`, `complete`, `naturalWidth`, `naturalHeight`, `fetchPriority`, `crossOrigin`, `referrerPolicy`, `alt`, `useMap`, and `isMap`, and unsupported elements reject the properties explicitly
- `HTMLCanvasElement` exposes modern canvas size reflection through `width` / `height` plus deterministic `getContext(contextId[, options])` behavior, deterministic `toDataURL(type[, quality])` placeholder output, and deterministic `toBlob(callback[, type[, quality]])` callback delivery with a `null` blob payload; the width and height IDL attributes reflect non-negative integer content attributes with defaults of `300` and `150`, `getContext()` currently returns `null`, `toDataURL()` returns a placeholder data URL, `toBlob()` invokes the callback synchronously with `null`, and bitmap rendering / `transferControlToOffscreen()` are not modeled
- `HTMLSlotElement` exposes reflected `name` plus deterministic `assignedNodes([options])` / `assignedElements([options])` snapshots over the element's fallback-content children; shadow-tree distribution and flattening behavior are not modeled
- `HTMLMediaElement` / `HTMLAudioElement` / `HTMLVideoElement` expose modern media reflection through `src`, `currentSrc`, `currentTime`, `buffered`, `seekable`, `played`, `textTracks`, `crossOrigin`, `preload`, `autoplay`, `loop`, `controls`, `defaultMuted`, `muted`, `volume`, `defaultPlaybackRate`, `playbackRate`, `preservesPitch`, `disableRemotePlayback`, and `controlsList`, plus deterministic `load()`, `pause()`, and `canPlayType(type)` methods, while `HTMLVideoElement` also exposes `disablePictureInPicture`, `poster`, `playsInline`, `width`, and `height`; unsupported elements reject the properties explicitly
- `HTMLQuoteElement.cite` and `HTMLModElement.cite` / `HTMLModElement.dateTime` expose raw quote/mod metadata reflections on `blockquote` / `q` / `ins` / `del` elements; unsupported elements reject the surface explicitly
- `HTMLSourceElement` exposes modern source reflection through `src`, `srcset`, `sizes`, `media`, and `type`, and unsupported elements reject the properties explicitly
- `HTMLTrackElement` exposes modern track reflection through `kind`, `src`, `srclang`, `label`, `default`, `readyState`, and `track`, with `track` modeled as a minimal `TextTrack` snapshot that exposes `kind`, `label`, `language`, `mode`, and `readyState`
- `HTMLDataElement.value` and `HTMLTimeElement.dateTime` expose modern machine-readable content reflections on `data` and `time` elements; `time.dateTime` falls back to the element's text content when `datetime` is absent
- `HTMLTemplateElement.shadowRootMode` / `shadowRootDelegatesFocus` / `shadowRootClonable` / `shadowRootSerializable` / `shadowRootCustomElementRegistry` expose declarative shadow-root flags on `<template>` elements while leaving `template.content` on the existing inert `DocumentFragment` slice
- `HTMLIFrameElement` exposes modern iframe reflection through `src`, `srcdoc`, `name`, `loading`, `referrerPolicy`, `allow`, `sandbox`, `allowFullscreen`, `credentialless`, `width`, `height`, `fetchPriority`, `contentDocument`, and `contentWindow`, and the embedded browsing-context getters are modeled as `null` in this workspace
- deterministic screen-position aliases are fixed at `0` through `window.screenX`, `window.screenY`, `window.screenLeft`, and `window.screenTop`
- `window.screen` is available as a deterministic read-only host object with fixed `width`, `height`, `availWidth`, `availHeight`, `availLeft`, `availTop`, `left`, `top`, `colorDepth`, `pixelDepth`, `orientation.type`, and `orientation.angle`
- global `Math` is available as a deterministic host object with standard constants, `window.crypto` is available as a deterministic host object with `randomUUID()`, and `HarnessBuilder.randomSeed(...)` can reseed both deterministic sequences
- script-side document and window alias surfaces (`document.documentElement`, `document.head`, `document.body`, `document.scrollingElement`, `document.activeElement`, `document.defaultView`, `document.title`, `document.location`, `document.URL`, `document.documentURI`, `document.baseURI`, `document.compatMode`, `document.characterSet`, `document.charset`, `document.contentType`, `document.referrer`, `document.dir`, `document.domain`, `document.visibilityState`, `document.hidden`, `document.origin`, `document.hasFocus()`, `document.ownerDocument`, `document.parentNode`, `document.parentElement`, `document.firstElementChild`, `document.lastElementChild`, `document.childElementCount`, `Node.contains(...)`, `Node.compareDocumentPosition(...)`, `Node.isSameNode(...)`, `Node.isEqualNode(...)`, `Node.namespaceURI`, `Node.isConnected`, `Node.hasChildNodes()`, `Node.firstChild`, `Node.lastChild`, `Node.nextSibling`, `Node.previousSibling`, `Node.nextElementSibling`, `Node.previousElementSibling`, and `template.content` exposes the same detached fragment traversal helpers (`firstElementChild`, `lastElementChild`, `childElementCount`, `isConnected`, `hasChildNodes()`, `firstChild`, `lastChild`, `nextSibling`, `previousSibling`, `nextElementSibling`, and `previousElementSibling`), `window.window`, `window.self`, `window.top`, `window.parent`, `window.opener`, `window.frameElement`, `window.closed`, `window.children`, `window.frames`, `window.length`, `window.navigator` (`userAgent`, `appCodeName`, `appName`, `appVersion`, `product`, `productSub`, `vendor`, `vendorSub`, `mimeTypes` (`length`, `item(index)`, `namedItem(name)`, `keys()`, `values()`, `entries()`, `toString()`), `languages` (`length`, `item(index)`, `contains(value)`, `keys()`, `values()`, `entries()`, `toString()`), `userLanguage`, `browserLanguage`, `systemLanguage`, `oscpu`, `pdfViewerEnabled`, `doNotTrack`, `javaEnabled()`, `plugins` (`length`, `item(index)`, `namedItem(name)`, `refresh()`, `toString()`), `platform`, `language`, `cookieEnabled`, `onLine`, `webdriver`, `hardwareConcurrency`, `maxTouchPoints`), `window.performance` (`now()` / `timeOrigin`), `window.crypto` (`randomUUID()`), `window.devicePixelRatio`, `window.innerWidth`, `window.innerHeight`, `window.outerWidth`, `window.outerHeight`, `window.screenX`, `window.screenY`, `window.screenLeft`, `window.screenTop`, `window.scrollX`, `window.scrollY`, `window.pageXOffset`, `window.pageYOffset`, `window.name`, `window.title`, `window.location`, `window.origin`, and `Element.baseURI` / `Element.origin` / `Element.namespaceURI` / `Element.ownerDocument` / `Element.parentNode` / `Element.parentElement` / `Element.firstElementChild` / `Element.lastElementChild` / `Element.childElementCount` / `Element.attributes.keys()` / `Element.attributes.values()` / `Element.attributes.entries()`) are available through inline scripts and stay wired into the copied session state; the viewport metrics are deterministic constants, the screen-position aliases are fixed at `0`, `window.performance.now()` is backed by the fake clock, `window.performance.timeOrigin` is deterministic, `window.crypto.randomUUID()` is deterministic, `document.visibilityState` is `visible`, `document.hidden` is `false`, and `document.hasFocus()` is deterministic
- script-side storage surfaces (`window.localStorage` and `window.sessionStorage`) are available through inline scripts and stay wired into the copied session state through `HarnessBuilder.addLocalStorage(...)` and `HarnessBuilder.addSessionStorage(...)`; convenience constructors `fromHtmlWithSessionStorage(...)` and `fromHtmlWithUrlAndSessionStorage(...)` are also available, and storage mutations dispatch deterministic `storage` events through `window.addEventListener('storage', ...)` and `window.onstorage`
- script-side `window.open()` / `window.close()` / `window.print()` / `window.scrollTo()` / `window.scrollBy()` are wired into the deterministic mock families; `window.print()` also dispatches deterministic `beforeprint` / `afterprint` handlers; `HarnessBuilder.openFailure(...)`, `HarnessBuilder.closeFailure(...)`, `HarnessBuilder.printFailure(...)`, and `HarnessBuilder.scrollFailure(...)` can seed bootstrap failures for inline scripts
- script-side HTML serialization surfaces (`innerHTML`, `outerHTML`, `insertAdjacentHTML`, and `template.content.innerHTML`) are available through inline scripts, with bounded fragment parsing on setters, deterministic serialization on getters, position-aware fragment insertion, `DocumentFragment`-style stringification for template content, namespace-aware SVG / MathML name adjustments during serialization, browser-style mixed-quote attribute escaping on outerHTML-style getters, and basic character reference decoding in parsed HTML
- script-side navigator collection helpers expose `forEach(callback[, thisArg])` on `window.navigator.mimeTypes`, `window.navigator.languages`, and `window.navigator.plugins`, alongside the existing `keys()` / `values()` / `entries()` / `toString()` helpers and the deterministic `refresh()` helper on `plugins`
- script-side file-input selection snapshots expose `length`, `item(index)`, `keys()`, `values()`, `entries()`, and `forEach(callback[, thisArg])` on `input[type=file].files`, and they stay wired to `Harness.setFiles(...)` selections
- script-side `document.location` / `window.location` are available as `Location` host objects with `href`, `hash`, `protocol`, `host`, `hostname`, `port`, `username`, `password`, `pathname`, `search`, `assign()`, `replace()`, `reload()`, `toString()`, and `valueOf()`, and still coerce to the current URL string during inline script evaluation; fragment changes also dispatch deterministic `hashchange` events through `window.addEventListener('hashchange', ...)` and `window.onhashchange`; history traversal also dispatches deterministic `popstate` events through `window.addEventListener('popstate', ...)` and `window.onpopstate`; window focus transitions dispatch deterministic `focus` / `blur` events through `window.addEventListener('focus', ...)` / `window.onfocus` and `window.addEventListener('blur', ...)` / `window.onblur`; page lifecycle transitions dispatch deterministic `beforeunload` / `pagehide` / `unload` / `pageshow` events through `window.addEventListener('beforeunload', ...)` / `window.onbeforeunload`, `window.addEventListener('pagehide', ...)` / `window.onpagehide`, `window.addEventListener('unload', ...)` / `window.onunload`, and `window.addEventListener('pageshow', ...)` / `window.onpageshow`; scroll transitions dispatch deterministic `scroll` events through `document.onscroll` and `window.onscroll`; bootstrap completion also dispatches deterministic `readystatechange` events through `document.onreadystatechange` and deterministic `load` events through `window.addEventListener('load', ...)` and `window.onload`; `window.history` is available as a limited `History` host object with `length`, `state`, `scrollRestoration`, `back()`, `forward()`, `go(delta)`, `pushState(state, title, url)`, and `replaceState(state, title, url)`, while `history.state` keeps a minimal payload snapshot (`null` / `undefined` stay `null`, and other values are stringified); the same hyperlink elements also expose read-only URL decomposition properties (`origin`, `protocol`, `host`, `hostname`, `port`, `username`, `password`, `pathname`, `search`, `hash`)
- the selector engine also accepts the universal selector `*` in the internal DOM layer and script-side query APIs
- node wrappers expose `nodeName`, `nodeType`, `nodeValue`, `data`, `textContent`, `length`, `wholeText`, and CharacterData editing methods (`splitText()`, `substringData()`, `appendData()`, `insertData()`, `deleteData()`, and `replaceData()`), and `Element.innerText` is available as a deterministic `textContent`-like alias on Element nodes
- `Harness.assertExists(...)`, `Harness.assertValue(...)`, `Harness.assertChecked(...)`, and `Harness.dumpDom(...)` are available for inspection
- `Harness.nowMs(...)`, `Harness.advanceTime(...)`, `Harness.flush(...)`, `Harness.mocksMut(...)`, `Harness.fetch(...)`, `Harness.alert(...)`, `Harness.confirm(...)`, `Harness.prompt(...)`, `Harness.readClipboard(...)`, `Harness.writeClipboard(...)`, `Harness.captureDownload(...)`, `Harness.open(...)`, `Harness.close()`, `Harness.print()`, `Harness.scrollTo(...)`, `Harness.scrollBy(...)`, `Harness.navigate(...)`, and `Harness.setFiles(...)` are available for deterministic runtime and mock control, including fetch, dialogs, clipboard, open, close, print, scroll, location, matchMedia, downloads, file-input, and storage families; storage mutations dispatch deterministic `storage` events through `window.addEventListener('storage', ...)` and `window.onstorage`; `matchMedia` returns deterministic `MediaQueryList` objects whose `matches` state reflects the current seeded mock state, and `addListener()` / `removeListener()` plus `addEventListener('change', ...)` / `removeEventListener('change', ...)` and `onchange` are available for change observation; inline scripts can schedule microtasks with `queueMicrotask()` / `window.queueMicrotask()` and timers with `setTimeout()` / `window.setTimeout()` / `clearTimeout()` / `window.clearTimeout()` plus repeating timers with `setInterval()` / `window.setInterval()` / `clearInterval()` / `window.clearInterval()`, animation frame callbacks with `requestAnimationFrame()` / `window.requestAnimationFrame()` / `cancelAnimationFrame()` / `window.cancelAnimationFrame()`, `window.performance.now()` reads from the fake clock, `window.crypto.randomUUID()` uses the deterministic crypto sequence, `HarnessBuilder.randomSeed(...)` controls the deterministic `Math.random()` and `crypto.randomUUID()` sequences, and `advanceTime()` / `flush()` drive due timers, animation frames, plus queued microtasks
- `Harness.click(...)`, `Harness.typeText(...)`, `Harness.setChecked(...)`, `Harness.setSelectValue(...)`, `Harness.focus(...)`, `Harness.blur(...)`, `Harness.submit(...)`, and `Harness.dispatch(...)` are available for user-like actions; `click(...)` also drives anchor / area navigation, download, target-open observation, and command-button activation deterministically, and script-side `HTMLElement.click()` / `Element.focus()` / `Element.blur()` update the action / focused element state from inline scripts
- inline `<script>` bootstrapping runs during `Harness.fromHtml(...)` construction for the `document.getElementById(...).textContent = ...` slice, and script-side selector lookups can reuse the shared DOM selector engine through `querySelector`, `querySelectorAll`, `matches`, and `closest`, plus `template.content.querySelector(All)` / `template.content.getElementById()`, including sibling combinators (`A + B`, `A ~ B`), the `:scope`, `:has(...)`, `:lang(...)`, `:dir(...)`, `:not(...)`, `:is(...)`, `:where(...)`, `:focus`, `:focus-visible`, `:focus-within`, `:target`, and bounded `:nth-*` pseudo-classes including `of <selector-list>` support on the `nth-child`, `nth-last-child`, `nth-of-type`, and `nth-last-of-type` families, and the bounded structural/state pseudo-class slice (`:root`, `:empty`, `:first-child`, `:last-child`, `:only-child`, `:first-of-type`, `:last-of-type`, `:only-of-type`, `:checked`, `:disabled`, `:enabled`, `:required`, `:optional`, `:link`, `:any-link`, `:placeholder-shown`, and `:blank`), plus the `:defined` pseudo-class slice, while inline bootstrap also exposes `document.currentScript`, `document.readyState`, `document.compatMode`, `document.characterSet`, `document.charset`, `document.contentType`, `document.referrer`, `document.dir`, `document.activeElement`, `document.defaultView`, `window.window`, `window.self`, `window.top`, `window.parent`, `window.opener`, `window.closed`, and `window.children`, with minimal `NodeList` snapshots for collection queries including `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, plus live `document.scripts` / `document.anchors` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `keys()`, `values()`, and `entries()`, live `document.forms`, `form.elements` (including controls associated via `form=` outside the subtree in document order), `fieldset.elements`, `datalist.options`, `map.areas`, and `table.tBodies` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, live `element.labels` NodeList support on labelable form controls and `fieldset` with `length`, `item(index)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()` that reflects explicit `label[for]` associations and implicit ancestor labels in tree order, live `document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.applets` / `document.all` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, live `document.styleSheets` StyleSheetList support with `length`, `item(index)`, `keys()`, `values()`, and `entries()`, live `table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, live `getElementsByTagName`, `getElementsByTagNameNS`, and `getElementsByClassName` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, plus live `getElementsByName` NodeList support with `length`, `item(index)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()` that reflects descendant elements in tree order, where `form.elements.namedItem(name)` returns a `RadioNodeList` when multiple matching controls share a name, live `select.options` and `select.selectedOptions` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, and live `select.options.add()` / `select.options.remove()` helpers, and live child-element / child-node surfaces on `Element`, `Document`, and `template.content` with `length`, `item(index)`, `namedItem(name)` where applicable, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`; node wrappers expose `nodeName`, `nodeType`, `nodeValue`, `data`, `textContent`, `length`, and CharacterData editing methods (`splitText()`, `substringData()`, `appendData()`, `insertData()`, `deleteData()`, and `replaceData()`); attribute reflection methods update the shared DOM attribute store and keep selector and form-control views in sync, and `className` / `classList` / `dataset` stay aligned with the same store
- inline `<script>` bootstrapping runs during `Harness.fromHtml(...)` construction for the `document.getElementById(...).textContent = ...` slice, and script-side selector lookups can reuse the shared DOM selector engine through `querySelector`, `querySelectorAll`, `matches`, and `closest`, including sibling combinators (`A + B`, `A ~ B`), the `:scope`, `:has(...)`, `:lang(...)`, `:dir(...)`, `:not(...)`, `:is(...)`, `:where(...)`, `:focus`, `:focus-within`, `:target`, and bounded `:nth-*` pseudo-classes including `of <selector-list>` support on the `nth-child`, `nth-last-child`, `nth-of-type`, and `nth-last-of-type` families, and the bounded structural/state pseudo-class slice (`:root`, `:empty`, `:first-child`, `:last-child`, `:only-child`, `:first-of-type`, `:last-of-type`, `:only-of-type`, `:checked`, `:disabled`, `:enabled`, `:required`, `:optional`, `:link`, `:any-link`, and `:placeholder-shown`), plus the `:defined` pseudo-class slice, while inline bootstrap also exposes `document.currentScript`, `document.readyState`, `document.compatMode`, `document.characterSet`, `document.charset`, `document.contentType`, `document.referrer`, `document.dir`, `document.activeElement`, `document.defaultView`, `window.window`, `window.self`, `window.top`, `window.parent`, `window.opener`, `window.closed`, and `window.children`, with minimal `NodeList` snapshots for collection queries including `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, plus live `document.scripts` / `document.anchors` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `keys()`, `values()`, and `entries()`, live `document.forms`, `form.elements`, `fieldset.elements`, `datalist.options`, `map.areas`, and `table.tBodies` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, live `element.labels` NodeList support on labelable form controls and `fieldset` with `length`, `item(index)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()` that reflects explicit `label[for]` associations and implicit ancestor labels in tree order, live `document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.applets` / `document.all` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, live `document.styleSheets` StyleSheetList support with `length`, `item(index)`, `keys()`, `values()`, and `entries()`, live `table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, live `getElementsByTagName`, `getElementsByTagNameNS`, and `getElementsByClassName` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, plus live `getElementsByName` NodeList support with `length`, `item(index)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()` that reflects descendant elements in tree order, where `form.elements.namedItem(name)` returns a `RadioNodeList` when multiple matching controls share a name, live `select.options` and `select.selectedOptions` HTMLCollection surfaces with `length`, `item(index)`, `namedItem(name)`, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`, and live `select.options.add()` / `select.options.remove()` helpers, and live child-element / child-node surfaces on `Element`, `Document`, and `template.content` with `length`, `item(index)`, `namedItem(name)` where applicable, `forEach(callback[, thisArg])`, `keys()`, `values()`, and `entries()`; node wrappers expose `nodeName`, `nodeType`, `nodeValue`, and `textContent`; attribute reflection methods update the shared DOM attribute store and keep selector and form-control views in sync, and `className` / `classList` / `dataset` stay aligned with the same store
- `form.elements` stays live in document order and includes controls associated via `form=` outside the subtree; `form.elements.namedItem(name)` can surface `RadioNodeList` objects for multi-match groups, `RadioNodeList.forEach(callback[, thisArg])` is available, and `RadioNodeList.value` is writable; assigning a matching radio value checks the first matching radio, and assigning a missing value clears the group; `select.selectedIndex`, `select.size`, and `select.type` are available on `select` controls and stay aligned with `select.value` / `select.selectedOptions`
- `classList`, `part`, and `relList` now also expose `keys()`, `values()`, `entries()`, and `forEach(callback[, thisArg])` alongside `value`, `item(index)`, and `add()` / `remove()` / `toggle()` / `replace()`
- stylesheet-owner `relList` surfaces also expose `add()` / `remove()` / `toggle()` / `replace()` / `supports()` on the same deterministic token store
- `checkValidity()` / `reportValidity()` are available on `input`, `textarea`, `select`, `form`, and `output`; `reportValidity()` also dispatches deterministic `invalid` events on invalid controls, supported form controls also expose `setCustomValidity()` / `validationMessage` / `willValidate` / `validity`, they follow the same built-in constraint state as `:valid` / `:invalid`, including `typeMismatch` and `patternMismatch` for supported text-like inputs, `input.valueAsNumber` / `input.valueAsDate` are available on supported `number` / `range` / `date` / `datetime-local` / `time` / `month` / `week` controls, `input.list` resolves the associated `datalist` element by `id` or `null`, `input.dirName` / `textarea.dirName` reflect the directionality name field, `textarea.rows` / `textarea.cols` / `textarea.wrap` reflect the modern textarea sizing and wrap hints with positive-integer fallback defaults, `input.accept`, `input.size`, and `input.capture` are available on supported inputs as modern file/text entry reflections, `input[type=color]` also exposes `alpha` and `colorSpace`, and `input` type=image also exposes modern image-input reflection for `src`, `alt`, `useMap`, `width`, and `height`, `input.indeterminate` feeds the existing `:indeterminate` state on supported inputs, `inputMode` and `enterKeyHint` are available as modern input-modality hints, `input.showPicker()` is available on supported `input` / `select` controls as a deterministic no-op surface, `HTMLOutputElement` additionally exposes `type`, `defaultValue`, `value`, `htmlFor`, `form`, and `labels`, `HTMLObjectElement` additionally exposes `data`, `type`, `name`, `width`, `height`, `useMap`, `form`, `contentDocument`, `contentWindow`, `getSVGDocument()`, `willValidate`, `validity`, `validationMessage`, `checkValidity()`, `reportValidity()`, and `setCustomValidity()`, with embedded browsing-context getters modeled as `null` in this workspace, and unsupported elements reject the methods explicitly
- the modern popover API slice (`popover`, `showPopover()`, `hidePopover()`, `togglePopover()`, `popoverTargetElement`, `popoverTargetAction`, and `:popover-open`) is available on HTML elements, popover target activation is modeled for button and input triggers and can be driven from script-side `HTMLElement.click()`, and top-layer rendering / light-dismiss behavior is not modeled
- `stepUp()` / `stepDown()` are available on supported `number` / `range` / `date` / `datetime-local` / `time` / `month` / `week` controls, using the same reflected `min` / `max` / `step` / `value` state and rejecting unsupported controls explicitly
- the direct reflected `Element` attribute slice also includes `defaultValue` / `defaultChecked` / `multiple` / `type` for form controls, plus `selected` and `value` for option/select controls
- inline `<script>` bootstrapping also exposes the minimal `Element.style` / `CSSStyleDeclaration` surface for semicolon-aware declaration lists with comment stripping and `!important` priority handling, including `cssText`, `getPropertyValue(...)`, `getPropertyPriority(...)`, `setProperty(...)`, `removeProperty(...)`, `length`, `item(index)`, and property reflection through `style.someProperty = ...`
- `window.getComputedStyle(element[, pseudoElt])` exposes a minimal live read-only `CSSStyleDeclaration` view backed by the element's inline style attribute; it supports `cssText`, `getPropertyValue(...)`, `getPropertyPriority(...)`, `length`, and `item(index)`, while write attempts and unsupported pseudo-elements fail explicitly
- `Element.getBoundingClientRect()` exposes a minimal read-only `DOMRect` snapshot derived from the element's inline style attribute; it supports `x`, `y`, `width`, `height`, `top`, `right`, `bottom`, `left`, and `String(rect)` / `toString()`, and it does not attempt full layout or viewport geometry
- `Element.getClientRects()` exposes a minimal read-only `DOMRectList` snapshot derived from the same inline-style geometry; it supports `length`, `item(index)`, and `String(rects)` / `toString()`, and it does not attempt full layout or viewport geometry; `Element.scrollIntoView()` routes through the deterministic scroll mock and scrolls to the workspace origin instead of doing layout-aware positioning
- `HTMLLabelElement.htmlFor` / `control` / `form` are available on label elements, using the existing `for` association and descendant labelable-control resolution, and `form` follows the labeled control's form owner when one exists
- `Harness` and `HarnessBuilder` are available, and `HarnessBuilder` can capture URL, HTML, local storage, session storage, and open/close/print/scroll bootstrap failure seeds
- `Session` stays internal and owns the copied configuration state plus the DOM store, which carries focus and target selector-state snapshots, script runtime state, event listener registry, queued microtasks, fake clock state, and mock registry
- `CSSLayerBlockRule.nameText` / `CSSLayerStatementRule.nameText` and writable `CSSLayerBlockRule.cssText` / `CSSLayerStatementRule.cssText` are available on `@layer` rules and rewrite the owning block in place while preserving nested rules; `CSSLayerStatementRule.nameList` exposes the comma-separated names as a `DOMStringList`
- `CSSScopeRule.start` / `CSSScopeRule.end` are available on `@scope` rules; the getters expose the scope root and scope limit strings (or `null` when absent), and `CSSScopeRule.cssText` / `CSSScopeRule.cssRules` remain available on the same slice
- `DomStore` builds, selects, serializes, and dumps HTML trees for tests, including class selectors, descendant/child/sibling combinators, `:scope`, `:has(...)`, `:lang(...)`, `:dir(...)`, `:not(...)`, `:is(...)`, `:where(...)`, `:focus`, `:focus-within`, `:target`, bounded `:nth-*` forms, the bounded structural/state pseudo-class slice including `:blank`, and `:defined`, but it is not part of the public API
- inline scripts can also read the document/window alias surfaces (`document.documentElement`, `document.head`, `document.body`, `document.scrollingElement`, `document.activeElement`, `document.title`, `document.location`, `document.URL`, `document.documentURI`, `document.baseURI`, `document.compatMode`, `document.characterSet`, `document.charset`, `document.contentType`, `document.referrer`, `document.dir`, `document.domain`, `document.origin`, `Node.contains(...)`, `Node.compareDocumentPosition(...)`, `Node.isSameNode(...)`, `Node.isEqualNode(...)`, `window.children`, `window.frames`, `window.length`, `window.navigator` (`userAgent`, `appCodeName`, `appName`, `appVersion`, `product`, `productSub`, `vendor`, `vendorSub`, `mimeTypes` (`length`, `item(index)`, `namedItem(name)`, `keys()`, `values()`, `entries()`, `toString()`), `languages` (`length`, `item(index)`, `contains(value)`, `keys()`, `values()`, `entries()`, `toString()`), `userLanguage`, `browserLanguage`, `systemLanguage`, `oscpu`, `platform`, `language`, `cookieEnabled`, `onLine`, `webdriver`, `hardwareConcurrency`, `maxTouchPoints`, `refresh()`, `javaEnabled()`), `window.performance` (`now()` / `timeOrigin`), `window.scrollX`, `window.scrollY`, `window.pageXOffset`, `window.pageYOffset`, `window.name`, `window.title`, `window.location`, `window.origin`, `Element.baseURI`, and `Element.origin`), but those bindings remain internal to the script runtime and are not part of the public `Harness` API; `document.location` / `window.location` are `Location` host objects with `href`, `hash`, `protocol`, `host`, `hostname`, `port`, `username`, `password`, `pathname`, `search`, `assign()`, `replace()`, `reload()`, `toString()`, and `valueOf()`, `window.performance` is a deterministic clock-backed host object with `now()` / `timeOrigin`, `window.history` is a limited `History` host object with `length`, `state`, `scrollRestoration`, `back()`, `forward()`, `go(delta)`, `pushState(state, title, url)`, and `replaceState(state, title, url)`, while `history.state` keeps a minimal payload snapshot (`null` / `undefined` stay `null`, and other values are stringified), and history traversal also dispatches deterministic `popstate` events through `window.addEventListener('popstate', ...)` and `window.onpopstate`
- `window.navigator.mimeTypes`, `window.navigator.languages`, and `window.navigator.plugins` also expose `forEach(callback[, thisArg])` alongside the existing `keys()` / `values()` / `entries()` / `toString()` helpers, with `plugins` also keeping deterministic `refresh()`.
- `StyleSheetList` and `CSSRuleList` expose `keys()` / `values()` / `entries()` / `forEach(callback[, thisArg])`, matching the other list-like surfaces in this workspace.
- `document.cookie` is backed by an owned cookie jar in session state; reads return semicolon-separated pairs and assignments accept simple `name=value` strings
- nested `@keyframes` rules also expose writable `CSSKeyframeRule.keyText` / `CSSKeyframesRule.cssText`; `CSSFontFeatureValuesRule.cssText` / `CSSColorProfileRule.cssText` / `CSSStartingStyleRule.cssText` / `CSSPositionTryRule.cssText` are writable too
- phase 8 HTML serialization surfaces (`innerHTML`, `outerHTML`, `insertAdjacentHTML`, and `template.content.innerHTML`) are available, including namespace-aware serialization compatibility, browser-style mixed-quote attribute escaping, and basic character reference decoding in parsed HTML; `document.styleSheets` also exposes a minimal `CSSStyleSheet.cssRules` surface for simple qualified rules and bounded `@media` / `@supports` / `@supports-condition` / `@document` / `@container` / `@starting-style` / `@position-try` / `@scope` / `@keyframes` / `@font-face` / `@font-feature-values` / `@font-palette-values` rules with `CSSFontPaletteValuesRule` exposing `name`, `fontFamily`, `basePalette`, `overrideColors`, and writable `CSSFontPaletteValuesRule.cssText` / `@color-profile` / `@page` / `@layer` block / statement rules / `@property` block rules with writable `CSSPropertyRule.cssText` / `@counter-style` rules with `name`, `system`, `symbols`, `negative`, `prefix`, `suffix`, `range`, `pad`, `fallback`, `speakAs`, `additiveSymbols`, and writable `CSSCounterStyleRule.cssText`, plus `@charset` / `@import` / `@namespace` rules in inline `<style>` sheets, a live writable `CSSStyleRule.style` `CSSStyleDeclaration` and writable top-level `CSSStyleRule.selectorText`, `CSSStyleRule.cssText`, `CSSFontFaceRule.style, writable `CSSFontFaceRule.cssText`, and writable `CSSKeyframeRule.keyText` / `CSSKeyframesRule.cssText`, `CSSPageRule.selectorText` for top-level `@page` rules, `CSSPageRule.cssText`, `CSSMediaRule.cssText`, `CSSMediaRule.insertRule()` / `deleteRule()` on top-level `@media` rules, writable `CSSPageRule.style`, legacy `CSSRule.type` integers, `CSSRule.parentStyleSheet` / `CSSRule.parentRule`, `CSSStyleSheet.ownerNode` / `href` / `title` / `disabled` / `media`, with `CSSStyleSheet.media.mediaText` writable and `CSSStyleSheet.media.appendMedium()` / `deleteMedium()` available on stylesheet media lists, while `CSSMediaRule.media` is a writable minimal `MediaList` surface with `mediaText`, `length`, and `item(index)`, and read-only `CSSMediaRule.matches` mirrors the current seeded `window.matchMedia(...)` result, while writable `CSSMediaRule.conditionText` rewrites the owning block in place while preserving nested rules, and writable `CSSSupportsRule.cssText` / `CSSContainerRule.cssText` rewrite their owning blocks in place while preserving nested rules, with `CSSSupportsRule.insertRule()` / `deleteRule()` and `CSSContainerRule.insertRule()` / `deleteRule()` available on top-level `@supports` / `@container` rules, while `CSSImportRule.media` remains read-only with the same minimal surface, and `CSSImportRule.styleSheet` / `CSSStyleSheet.ownerRule` are exposed as null linkage surfaces, with `CSSImportRule.supportsText` / `CSSImportRule.layerName` exposed as read-only metadata, plus `CSSSupportsConditionRule.name` on `@supports-condition` rules with writable `cssText` and an empty deterministic `cssRules` slice for now, plus `HTMLStyleElement.sheet` / `HTMLLinkElement.sheet` and reflected `media` / `rel` / `relList` / `charset` / `hreflang` / `crossOrigin` / `disabled` on stylesheet owner elements, plus `keys()` / `values()` / `entries()` / `forEach(callback[, thisArg])` parity on `StyleSheetList` / `CSSRuleList` and legacy `CSSStyleSheet.rules` / `addRule()` / `removeRule()` aliases alongside `CSSStyleSheet.insertRule()` / `deleteRule()` / `replaceSync()` on inline `<style>` owners, while broader CSS parsing beyond the bounded selector engine remains deferred until a specific user-visible gap needs it; nested `@keyframes` rules also expose writable `CSSKeyframeRule.keyText`
- `CSSDocumentRule.conditionText` and `CSSDocumentRule.cssText` are writable on `@document` rules and rewrite the owning block in place while preserving nested rules.
- `CSSContainerRule` now also exposes `containerName` and `containerQuery` on `@container` rules, and `conditionText` / `cssText` are writable so they rewrite the owning block in place while preserving nested rules, alongside the existing `cssRules` surface.
- `CSSSupportsRule.conditionText` / `CSSSupportsRule.cssText` are writable on `@supports` rules and rewrite the owning block in place while preserving nested rules.

- stylesheet owner elements also expose reflected `media` / `rel` / `relList` / `relList.add()` / `relList.remove()` / `relList.toggle()` / `relList.replace()` / `relList.supports()` / `as` / `charset` / `imageSrcset` / `imageSizes` / `fetchPriority` / `hreflang` / `crossOrigin` / `referrerPolicy` / `integrity` / `type`, and `HTMLStyleElement.sheet` / `HTMLLinkElement.sheet` return the stylesheet owner linkage; `HTMLLinkElement` / `HTMLScriptElement` / `HTMLStyleElement` also expose `blocking` as a `DOMTokenList` with supported token `render`, but render-blocking behavior itself is not modeled
- HTMLScriptElement also exposes reflected `src` / `charset` / `text` / `type` / `async` / `defer` / `noModule` / `crossOrigin` / `integrity` / `referrerPolicy` / `fetchPriority` metadata on the same phase 8 script surface, and `HTMLLinkElement` / `HTMLScriptElement` / `HTMLStyleElement` also expose `blocking` as a `DOMTokenList` with supported token `render`, but render-blocking behavior itself is not modeled
- HTMLMetaElement also exposes reflected `name` / `content` / `httpEquiv` / `charset` / `media` metadata on the same modern reflection slice
- `HTMLCanvasElement` exposes modern canvas size reflection through `width` / `height` plus deterministic `getContext(contextId[, options])` behavior, deterministic `toDataURL(type[, quality])` placeholder output, and deterministic `toBlob(callback[, type[, quality]])` callback delivery with a `null` blob payload; the width and height IDL attributes reflect non-negative integer content attributes with defaults of `300` and `150`, `getContext()` currently returns `null`, `toDataURL()` returns a placeholder data URL, `toBlob()` invokes the callback synchronously with `null`, and bitmap rendering / `transferControlToOffscreen()` are not modeled
- HTMLMediaElement / HTMLAudioElement / HTMLVideoElement also expose modern reflection through `src`, `currentSrc`, `currentTime`, `buffered`, `seekable`, `played`, `textTracks`, `duration`, `paused`, `seeking`, `ended`, `readyState`, `networkState`, `crossOrigin`, `preload`, `autoplay`, `loop`, `controls`, `defaultMuted`, `muted`, `volume`, `defaultPlaybackRate`, `playbackRate`, `preservesPitch`, `disableRemotePlayback`, and `controlsList`, plus deterministic `load()`, `pause()`, and `canPlayType(type)` methods, while `HTMLVideoElement` also exposes `disablePictureInPicture`, `poster`, `playsInline`, `width`, and `height`
## Quick Start

```bash
cd zig
zig build test
```

## Minimal Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<form id='profile'><input id='name'><input id='agree' type='checkbox'><button id='submit' type='submit'>Save</button></form><div id='out'></div><script>document.getElementById('profile').addEventListener('submit', () => { document.getElementById('out').textContent = document.getElementById('name').value + ':' + String(document.getElementById('agree').checked); });</script>",
    );
    defer harness.deinit();

    try harness.typeText("#name", "Alice");
    try harness.click("#agree");
    try harness.click("#submit");
    try harness.assertChecked("#agree", true);
    try harness.assertValue("#out", "Alice:true");
}
```

## Mock Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(std.heap.page_allocator, "<main></main>");
    defer harness.deinit();

    try harness.mocksMut().fetch().respondText("https://app.local/api/message", 200, "ok");
    const response = try harness.fetch("https://app.local/api/message");
    try std.testing.expectEqualStrings("ok", response.body);
    try std.testing.expectEqual(@as(usize, 1), harness.mocksMut().fetch().calls().len);

    try harness.captureDownload("report.csv", "downloaded bytes");
    try std.testing.expectEqual(@as(usize, 1), harness.mocksMut().downloads().artifacts().len);

    try harness.open("https://app.local/popup");
    try harness.close();
    try harness.print();
    try harness.scrollTo(10, 20);
    try harness.scrollBy(-5, 3);
    try std.testing.expectEqual(@as(usize, 1), harness.mocksMut().open().calls().len);
    try std.testing.expectEqualStrings(
        "https://app.local/popup",
        harness.mocksMut().open().calls()[0].url.?,
    );
    try std.testing.expectEqual(@as(usize, 1), harness.mocksMut().close().calls().len);
    try std.testing.expectEqual(@as(usize, 1), harness.mocksMut().print().calls().len);
    try std.testing.expectEqual(@as(usize, 2), harness.mocksMut().scroll().calls().len);
}
```

Use `HarnessBuilder.openFailure(...)`, `HarnessBuilder.closeFailure(...)`, `HarnessBuilder.printFailure(...)`, and `HarnessBuilder.scrollFailure(...)` when you want inline `window.open()` / `window.close()` / `window.print()` / `window.scrollTo()` / `window.scrollBy()` calls to fail during bootstrap with `error.MockError`.

## MatchMedia Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<button id='toggle'>Toggle</button><div id='out'></div><script>document.getElementById('toggle').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); document.getElementById('out').textContent = String(mql) + ':' + String(mql.matches); });</script>",
    );
    defer harness.deinit();

    try harness.mocksMut().matchMedia().seedMatch("(max-width: 600px)", true);
    try harness.click("#toggle");
    try harness.assertValue("#out", "[object MediaQueryList]:true");
    try std.testing.expectEqual(@as(usize, 1), harness.mocksMut().matchMedia().calls().len);
}
```

## Storage Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var builder = bt.Harness.builder(std.heap.page_allocator);
    defer builder.deinit();

    _ = builder.html("<main id='out'></main><script>const local = window.localStorage; const session = window.sessionStorage; local.setItem('theme', 'dark'); session.setItem('scratch', 'xyz'); document.getElementById('out').textContent = local.getItem('token') + ':' + session.getItem('session-token') + '|' + local.getItem('theme') + ':' + session.getItem('scratch');</script>");
    try builder.addLocalStorage("token", "abc");
    try builder.addSessionStorage("session-token", "seed");

    var harness = try builder.build();
    defer harness.deinit();

    try harness.assertValue("#out", "abc:seed|dark:xyz");
    try std.testing.expectEqualStrings("dark", harness.mocksMut().storage().local().get("theme").?);
    try std.testing.expectEqualStrings("xyz", harness.mocksMut().storage().session().get("scratch").?);
}
```

## Class/Dataset Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<main id='root'><button id='button' class='base' data-kind='App'>First</button><div id='out'></div><script>document.getElementById('button').className = 'primary secondary'; document.getElementById('button').classList.add('active'); document.getElementById('button').dataset.userId = '42'; document.getElementById('out').textContent = document.getElementById('button').className + ':' + document.getElementById('button').dataset.userId;</script></main>",
    );
    defer harness.deinit();

    try harness.assertExists(".active");
    try harness.assertExists("[data-user-id]");
    try harness.assertValue("#out", "primary secondary active:42");
}
```

## Inline Style Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<main id='root'><div id='box' style='color: red; background-color: white;'></div><div id='out'></div><script>const box = document.getElementById('box'); const style = box.style; style.backgroundColor = 'blue'; style.setProperty('border-top-width', '2px'); style.removeProperty('color'); document.getElementById('out').textContent = String(style.cssText) + ':' + style.getPropertyValue('background-color') + ':' + String(style.length) + ':' + style.item(0);</script></main>",
    );
    defer harness.deinit();

    try harness.assertValue(
        "#out",
        "background-color: blue; border-top-width: 2px;:blue:2:background-color",
    );
}
```

## Tree Mutation Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<main id='root'><section id='target'></section><button id='first'>First</button><button id='second'>Second</button><button id='third'>Third</button><div id='out'></div><script>document.getElementById('target').append(document.getElementById('first'), document.getElementById('second')); document.getElementById('target').prepend(document.getElementById('third')); document.getElementById('first').remove(); document.getElementById('out').textContent = document.getElementById('target').textContent + ':' + String(document.querySelectorAll('#target > button').length);</script></main>",
    );
    defer harness.deinit();

    try harness.assertValue("#out", "ThirdSecond:2");
    try harness.assertExists("#target > #third");
    try harness.assertExists("#target > #second");
}
```

## HTML Serialization Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section><div id='out'></div><script>document.getElementById('target').insertAdjacentHTML('beforebegin', '<aside id=\"before\">Before</aside>'); document.getElementById('target').insertAdjacentHTML('afterbegin', '<span id=\"first\">One</span>'); document.getElementById('target').insertAdjacentHTML('beforeend', '<span id=\"second\">Two</span>'); document.getElementById('target').insertAdjacentHTML('afterend', '<aside id=\"after\">After</aside>'); document.getElementById('out').textContent = document.getElementById('root').innerHTML + '|' + document.getElementById('target').innerHTML + '|' + String(document.querySelectorAll('#target > span').length);</script></main>",
    );
    defer harness.deinit();

    try harness.assertValue(
        "#out",
        "<aside id=\"before\">Before</aside><section id=\"target\"><span id=\"first\">One</span><button class=\"primary\" id=\"old\">Old</button><span id=\"second\">Two</span></section><aside id=\"after\">After</aside>|<span id=\"first\">One</span><button class=\"primary\" id=\"old\">Old</button><span id=\"second\">Two</span>|2",
    );
    try harness.assertExists("#before");
    try harness.assertExists("#after");
    try harness.assertExists("#target > #first");
    try harness.assertExists("#target > #second");
}
```

## Template Content Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>document.getElementById('out').textContent = String(document.getElementById('tpl').content) + '|' + document.getElementById('tpl').content.innerHTML; document.getElementById('tpl').content.innerHTML = '<span id=\"second\">Second</span>'; document.getElementById('out').textContent += '|' + document.getElementById('tpl').content.innerHTML;</script>",
    );
    defer harness.deinit();

    try harness.assertValue(
        "#out",
        "[object DocumentFragment]|<span id=\"inner\">Inner</span>|<span id=\"second\">Second</span>",
    );
    try harness.assertExists("#second");
}
```

## Public Surface

- `Harness`
- `HarnessBuilder`
- `StorageSeed`
- `MockRegistry`
- `FetchMocks`
- `FetchResponseRule`
- `FetchErrorRule`
- `FetchCall`
- `FetchResponse`
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
- `DownloadMocks`
- `DownloadCapture`
- `FileInputMocks`
- `FileInputSelection`
- `StorageSeeds`
- `Error`
- `Result(T)`
- `Error` currently includes `InvalidUrl`, `InvalidSelector`, `AssertionFailed`, `DomError`, `EventError`, `ScriptParse`, `ScriptRuntime`, `HtmlParse`, `MockError`, `TimerError`, and `OutOfMemory`
- `Harness.assertExists(selector)`
- `Harness.assertValue(selector, expected)`
- `Harness.assertChecked(selector, expected)`
- `Harness.nowMs()`
- `Harness.advanceTime(delta_ms)`
- `Harness.flush()`
- `Harness.mocksMut()`
- `HarnessBuilder.openFailure(message)`
- `HarnessBuilder.closeFailure(message)`
- `HarnessBuilder.printFailure(message)`
- `HarnessBuilder.scrollFailure(message)`
- `Harness.fetch(url)`
- `Harness.alert(message)`
- `Harness.confirm(message)`
- `Harness.prompt(message)`
- `Harness.readClipboard()`
- `Harness.writeClipboard(text)`
- `Harness.captureDownload(file_name, bytes)`
- `Harness.open(url)`
- `Harness.close()`
- `Harness.print()`
- `Harness.scrollTo(x, y)`
- `Harness.scrollBy(x, y)`
- `Harness.navigate(url)`
- `Harness.setFiles(selector, files)`
- `Harness.click(selector)`
- `Harness.typeText(selector, text)`
- `Harness.setChecked(selector, checked)`
- `Harness.setSelectValue(selector, value)`
- `Harness.focus(selector)`
- `Harness.blur(selector)`
- `Harness.submit(selector)`
- `Harness.dispatch(selector, event_type)`
- `Harness.dumpDom(allocator)`

## Docs

- [Architecture](doc/architecture.md)
- [Capability Matrix](doc/capability-matrix.md)
- [Implementation Guide](doc/implementation-guide.md)
- [Subsystem Map](doc/subsystem-map.md)
- [Mock Guide](doc/mock-guide.md)
- [Limitations](doc/limitations.md)
- [Roadmap](doc/roadmap.md)
- [Publish Checklist](doc/publish-checklist.md)
