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

- event dispatch with bubbling and capture listeners
- default actions
- form controls
- text selection state on supported `input` / `textarea` controls (`selectionStart`, `selectionEnd`, `selectionDirection`, `setSelectionRange(...)`, and `select()`)
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
- tree mutation primitives
- HTML serialization surfaces
- HTML serialization broadening slice 1 (`insertAdjacentHTML`)
- HTML serialization broadening slice 2 (`template.content.innerHTML` / `DocumentFragment` serialization)

The attribute reflection (`getAttribute`, `getAttributeNS`, `setAttribute`, `setAttributeNS`, `removeAttribute`, `removeAttributeNS`, `hasAttribute`, `hasAttributeNS`, `hasAttributes()`, `getAttributeNames()`, and `toggleAttribute`), including direct reflected `id`, `title`, `lang`, `dir`, `hidden`, `inert`, `translate`, `draggable`, `disabled`, `required`, `nonce`, `autocapitalize`, `autocomplete`, `autofocus`, `spellcheck`, `inputMode`, `readOnly`, `tabIndex`, `accessKey`, `contentEditable`, and `isContentEditable` state, class/dataset view, tree mutation, HTML serialization, and namespace-aware serialization slices are implemented in this workspace now, and collection API broadening slices 1 (`NodeList.forEach`), 2 (`document.scripts`), 3 (`document.anchors`), 4 (`NodeList.keys()` / `NodeList.values()` / `HTMLCollection.keys()` / `HTMLCollection.values()` / `entries()`), 5 (`Element.children` / `document.children` / `document.childNodes` / `element.childNodes` / `template.content.childNodes` / `template.content.children`), 6 (`document.forms`), 7 (`form.elements` / `form.length`), 8 (`select.options` / `select.length`), 9 (`select.selectedOptions`), 10 (`fieldset.elements`), 11 (`datalist.options`), 12 (`map.areas`), 13 (`table.tBodies`), 14 (`element.labels`), 15 (`document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.applets` / `document.all`), 16 (`document.styleSheets`), 17 (`table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells`), 18 (`getElementsByTagName` / `getElementsByTagNameNS` / `getElementsByClassName` / `getElementsByName`), 19 (`entries()` helpers across `NodeList`, `HTMLCollection`, `StyleSheetList`, and `RadioNodeList`), and 20 (`select.options.add()` / `select.options.remove()`) are implemented as well. The `StyleSheetList`, `CSSRuleList`, and `RadioNodeList` `forEach(callback[, thisArg])` helper parity is implemented too, and `CSSStyleSheet.insertRule()` / `deleteRule()` are available on inline `<style>` sheets.
The direct reflected attribute set also includes `name`.
The direct reflected attribute set also includes `placeholder`.
The direct reflected attribute set also includes `selected` and `value`, and `select.selectedIndex` is available as the select-side index mirror.
The detached construction slice (`document.createElement()`, `document.createTextNode()`, `document.createComment()`, and `document.createDocumentFragment()`) is also delivered, and the tree mutation slice now also includes `normalize()` and `document.importNode(...)`.
The tree mutation slice also includes `removeChild()`.
The minimal inline style declaration slice, including comment stripping, `!important` priority handling, and `getPropertyPriority(...)`, is implemented; the minimal `CSSStyleSheet.cssRules` slice for inline `<style>` sheets is implemented too, including bounded `@media` / `@supports` / `@document` / `@container` / `@starting-style` / `@position-try` / `@scope` / `@keyframes` / `@font-face` / `@font-feature-values` / `@font-palette-values` / `@color-profile` / `@page` / `@layer` block / statement rules / `@property` block rules and `@counter-style` rules and `@charset` / `@import` / `@namespace` statements, and `CSSStyleRule.style` is available as a read-only snapshot `CSSStyleDeclaration`, and `CSSRule.type` exposes the legacy CSSOM integer mapping for classic rule kinds (with newer at-rules returning `0`), and `CSSRule.parentStyleSheet` and `CSSRule.parentRule` return the owning stylesheet and owning rule on rule objects, and `CSSStyleSheet.ownerNode` returns the owner element, while `CSSStyleSheet.href` / `CSSStyleSheet.title` / `CSSStyleSheet.disabled` expose owner metadata, and `CSSStyleSheet.media.appendMedium()` / `deleteMedium()` are available on stylesheet media lists, while `CSSMediaRule.media` / `CSSImportRule.media` remain read-only minimal `MediaList` surfaces and `CSSImportRule.styleSheet` / `CSSStyleSheet.ownerRule` stay null linkage surfaces, with `CSSImportRule.supportsText` / `CSSImportRule.layerName` as read-only metadata; the next named work is broader CSS parsing beyond the bounded selector engine, if a specific user-visible gap needs it.
`RadioNodeList.value` is writable in the form-elements slice, and unmatched assignments clear the checked radio group in this workspace.

## Phase 9: Document and Window Surface Expansion

- `document.documentElement`, `document.head`, `document.body`, `document.scrollingElement`, `document.activeElement`, `document.referrer`, `document.dir`, `document.visibilityState`, `document.hidden`, `document.hasFocus()`, `document.ownerDocument`, `document.parentNode`, `document.parentElement`, `document.firstElementChild`, `document.lastElementChild`, `document.childElementCount`, `window.children`, `window.frames`, `window.length`, `window.navigator` (`userAgent`, `appCodeName`, `appName`, `appVersion`, `product`, `productSub`, `vendor`, `vendorSub`, `pdfViewerEnabled`, `doNotTrack`, `javaEnabled()`, `plugins`, `platform`, `language`, `cookieEnabled`, `onLine`, `webdriver`, `hardwareConcurrency`, `maxTouchPoints`, `refresh()`), `window.performance` (`now()` / `timeOrigin`), `window.devicePixelRatio`, `window.innerWidth`, `window.innerHeight`, `window.outerWidth`, `window.outerHeight`, `window.scrollX`, `window.scrollY`, `window.pageXOffset`, `window.pageYOffset`, and `window.name`
- `document.title` and `window.title`
- `document.location` and `window.location` as a `Location` host object with `href`, `hash`, `protocol`, `host`, `hostname`, `port`, `username`, `password`, `pathname`, `search`, `assign()`, `replace()`, `reload()`, `toString()`, and `valueOf()`, plus deterministic `hashchange` events, deterministic `popstate` events on history traversal, `window.onhashchange`, `window.onpopstate`, `window.onfocus`, and `window.onblur`, plus bootstrap completion `load` events through `window.onload`
- `document.URL`, `document.documentURI`, `document.baseURI`, `document.compatMode`, `document.characterSet`, `document.charset`, and `document.contentType`
- `document.origin`, `window.origin`, `Element.baseURI`, and `Element.origin`
- `document.domain` as a deterministic read-only host-derived alias
- `document.cookie` as a deterministic session-owned cookie jar

The document and window alias slice is implemented in this workspace now, including the document metadata, `document.defaultView`, `document.referrer`, `document.dir`, `document.domain`, `document.cookie`, the node reflection helpers (`ownerDocument`, `parentNode`, `parentElement`, `nodeValue`, `data`, `firstElementChild`, `lastElementChild`, `childElementCount`, `isConnected`, `hasChildNodes()`, `firstChild`, `lastChild`, `nextSibling`, `previousSibling`, `nextElementSibling`, and `previousElementSibling`), `document.scrollingElement`, `document.visibilityState`, `document.hidden`, `document.hasFocus()`, `window.window`, `window.self`, `window.top`, `window.parent`, `window.opener`, `window.closed`, `window.children`, `window.frames`, `window.length`, `window.navigator` (`userAgent`, `appCodeName`, `appName`, `appVersion`, `product`, `productSub`, `vendor`, `vendorSub`, `platform`, `language`, `cookieEnabled`, `onLine`, `webdriver`, `hardwareConcurrency`, `maxTouchPoints`, `javaEnabled()`), `window.performance` (`now()` / `timeOrigin`), `window.devicePixelRatio`, `window.innerWidth`, `window.innerHeight`, `window.outerWidth`, `window.outerHeight`, `window.scrollX`, `window.scrollY`, `window.pageXOffset`, `window.pageYOffset`, and `window.name` aliases used during inline script bootstrap.

`window.screen` is also implemented as a deterministic read-only geometry object, including the fixed `orientation.type` / `orientation.angle` pair, `window.Math` is available as a deterministic global host object with constants and `Math.random()`, and the screen-position quartet (`window.screenX`, `window.screenY`, `window.screenLeft`, and `window.screenTop`) is already implemented in Zig as deterministic constants.

## Phase 10: Limited Navigation Model

- `window.history`
- `history.length`, `history.state`, and `history.scrollRestoration`
- `back()`, `forward()`, and `go(delta)`
- `pushState(...)` and `replaceState(...)`

The limited history navigation slice is implemented in this workspace now, and `history.scrollRestoration` is also exposed as a deterministic read/write alias with `auto` / `manual`; history traversal now also dispatches deterministic `popstate` events through `window.onpopstate`, the window focus/blur alias surface dispatches deterministic `focus` / `blur` events through `window.onfocus` and `window.onblur`, and bootstrap completion dispatches deterministic `load` events through `window.onload`.

The deterministic mock phase also includes `HarnessBuilder.randomSeed(...)` for seeding the deterministic `Math.random()` and `crypto.randomUUID()` sequences before inline scripts run.
`window.navigator.languages` is implemented as a minimal `DOMStringList`-like surface with `length`, `item(index)`, `contains(value)`, and `toString()`, and the legacy aliases `userLanguage`, `browserLanguage`, `systemLanguage`, and `oscpu` are also part of the implemented slice.
