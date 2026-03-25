# Implementation Guide

The bounded script host surface also includes `document.title` and `window.title`.
It also includes `document.createElement()`, `document.createElementNS()`, `document.createTextNode()`, `document.createComment()`, and `document.createDocumentFragment()` for detached HTML / namespace-aware element / text node / comment / fragment construction, plus `before()`, `after()`, `cloneNode()`, `remove()`, `removeChild()`, `normalize()`, `importNode()`, and `replaceWith()` on existing nodes, and `cloneNode()` / `textContent` on `template.content` for detached cloning and reflection, and direct child mutation on `template.content`.
It also includes `document.location.username` / `document.location.password` and `window.location.username` / `window.location.password` as part of the same location mock.
It also includes `Element.click()`, `Element.focus()`, and `Element.blur()` as script-side entry points into the same deterministic event/default-action and focus-tracking paths as the harness actions.
It also includes `element.accessKey` / `element.slot` / `element.autocapitalize` / `element.translate` / `element.dir` / `element.lang` / `element.title` / `element.role` / `element.ariaLabel` / `element.ariaDescription` / `element.ariaRoleDescription` / `element.ariaHidden` / `element.tabIndex` / `element.hidden` as reflected string / boolean attributes that feed the same `:dir()` / `:lang()` selector matching surfaces, with `element.translate` inheriting the nearest `translate` state from ancestors and `element.title` / `element.role` / `element.ariaLabel` / `element.ariaDescription` / `element.ariaRoleDescription` / `element.ariaHidden` / `element.tabIndex` remaining plain attribute reflections or bounded integer reflection.
It also includes `element.spellcheck` as a boolean reflection that inherits ancestor `spellcheck` state, and `element.inputMode` as a plain string reflection on the same surface.
The same script surface also includes `Number.prototype.toFixed()`, `Number.prototype.toPrecision()`, and `Number.prototype.toExponential()` on numeric values, which is sufficient for the imported regression cases that format fixed-point, significant-digit, and scientific-notation amounts.
The bounded pseudo-class set also includes `:focus-visible`, which currently follows the same focus state as `:focus`.
It also includes `Node.isConnected` / `Element.isConnected` / `Document.isConnected` and `Node.contains()` / `Element.contains()` / `Document.contains()` / `Node.compareDocumentPosition()` / `Element.compareDocumentPosition()` / `Document.compareDocumentPosition()` / `Node.isSameNode()` / `Element.isSameNode()` / `Document.isSameNode()` / `Node.isEqualNode()` / `Element.isEqualNode()` / `Document.isEqualNode()` / `Node.hasChildNodes()` / `Element.hasChildNodes()` / `Document.hasChildNodes()` / `Node.firstChild` / `Element.firstChild` / `Document.firstChild` / `Node.lastChild` / `Element.lastChild` / `Document.lastChild` / `Node.nextSibling` / `Element.nextSibling` / `Document.nextSibling` / `Node.previousSibling` / `Element.previousSibling` / `Document.previousSibling` as read-only tree-connectivity, containment, tree-order, node-equality, and child-sibling-presence reflection helpers, with detached `template.content` remaining disconnected.
It also includes `document.location`, `document.location.href`, `document.location.hash`, `document.location.pathname`, `document.location.search`, `document.location.protocol`, `document.location.host`, `document.location.hostname`, `document.location.port`, `document.location.username`, `document.location.password`, `window.location`, `window.location.href`, `window.location.hash`, `window.location.pathname`, `window.location.search`, `window.location.protocol`, `window.location.host`, `window.location.hostname`, `window.location.port`, `window.location.username`, `window.location.password`, `document.URL`, `document.documentURI`, `document.baseURI`, `element.baseURI`, `element.tagName`, `element.localName`, `element.namespaceURI`, `document.origin`, `document.referrer`, `document.cookie`, `document.domain`, `document.designMode` (read/write bounded to `on` / `off`), `element.contentEditable` / `element.isContentEditable` (bounded to the current element's `contenteditable` attribute plus ancestor `contenteditable` inheritance), `window.name`, `window.self`, `window.window`, `window.parent`, `window.top`, `window.closed`, `window.history.length` / `window.history.state` / `window.history.scrollRestoration` / `window.history.pushState()` / `window.history.replaceState()` / `window.history.back()` / `window.history.forward()` / `window.history.go()` (bounded history navigation; `state` tracks the current history entry and is updated by `pushState` / `replaceState`; `scrollRestoration` is currently `auto` or `manual` and does not trigger viewport restoration), `window.origin`, `element.origin`, `window.navigator` metadata (`userAgent`, `appCodeName`, `appName`, `appVersion`, `product`, `productSub`, `vendor`, `vendorSub`, `pdfViewerEnabled`, `doNotTrack`, `javaEnabled()`, `plugins`, `mimeTypes`, `platform`, `language`, `userLanguage`, `browserLanguage`, `systemLanguage`, `oscpu`, `languages`, `cookieEnabled`, `onLine`, `webdriver`, `hardwareConcurrency`, `maxTouchPoints`, plus iterator helpers and `forEach()` on `navigator.languages` / `navigator.mimeTypes`, with `navigator.plugins.refresh()` exposed as a deterministic no-op refresh hook), `window.devicePixelRatio`, `window.innerWidth`, `window.innerHeight`, `window.outerWidth`, `window.outerHeight`, `window.screenX`, `window.screenY`, `window.screenLeft`, `window.screenTop`, `window.screen` (`availWidth`, `availHeight`, `availLeft`, `availTop`, `colorDepth`, `pixelDepth`, `orientation.type`, `orientation.angle`), `window.localStorage` / `window.sessionStorage` named property access, `document.currentScript`, `document.readyState`, `document.compatMode`, `document.characterSet`, `document.charset`, `document.contentType`, `document.visibilityState`, `document.hidden`, `document.activeElement`, `document.hasFocus()`, `window.children` as an alias of `document.children`, and `window.frames` / `window.length` / `window.frameElement` / `window.opener` as live `HTMLCollection` frame-count / frame-element surfaces over descendant `iframe` / `frame` elements during inline script bootstrap through the runtime location mock, storage surface, cookie jar, focus state, session bootstrap state, and document metadata surface; `form.length` and `select.length` are also exposed as read-only aliases to their underlying live collections, and window scroll capture is also wired through the same mock seam. `template.content` also exposes `textContent` alongside its live collection and query surfaces.
It also exposes `window.Node`, `window.Element`, `window.HTMLElement`, and a bounded set of HTML constructor globals, including `window.HTMLButtonElement`, `window.HTMLSelectElement`, `window.HTMLInputElement`, `window.HTMLTextAreaElement`, `window.HTMLFormElement`, `window.HTMLOptionElement`, `window.HTMLOptGroupElement`, `window.HTMLFieldSetElement`, `window.HTMLLabelElement`, `window.HTMLImageElement`, `window.HTMLAnchorElement`, `window.HTMLAreaElement`, `window.HTMLMapElement`, `window.HTMLTableElement`, `window.HTMLTableSectionElement`, `window.HTMLTableRowElement`, `window.HTMLTableCellElement`, `window.HTMLUListElement`, `window.HTMLOListElement`, `window.HTMLLIElement`, `window.HTMLObjectElement`, `window.HTMLEmbedElement`, `window.HTMLLegendElement`, `window.HTMLDListElement`, `window.HTMLScriptElement`, and `window.HTMLStyleElement`, for `instanceof` checks.
The same reflected element surface also includes `element.id` and `element.name` so the shared attribute store stays in sync with selector and named-collection lookups.
The same reflected option surface also includes `option.selected` so the shared attribute store stays in sync with `select.selectedOptions` and `:checked` selectors.
The same reflected option surface also includes `option.defaultSelected` as the selected-attribute alias used by the same state and selector surfaces.
The same reflected option surface also includes `option.disabled` as the disabled-attribute alias used by the same state and selector surfaces, and the same disabled state is now reflected on common form controls via `input.disabled` / `textarea.disabled` / `button.disabled` / `select.disabled` / `fieldset.disabled`.
The same reflected option surface also includes `option.label` as the label-attribute alias, with text-content fallback when the attribute is missing.
The same reflected optgroup surface also includes `optgroup.disabled` / `optgroup.label` as reflected read/write attributes backed by the same attribute store, and `optgroup.disabled` feeds the bounded `:disabled` selector state.
The same reflected fieldset surface also includes `fieldset.disabled` as a reflected read/write boolean disabled attribute alias, and `fieldset.disabled` feeds the same bounded `:disabled` selector state.
The same reflected option surface also includes `option.text` as the text-content alias used by the same option text surface, and form validation reflection now includes `form.noValidate` / `input.formNoValidate` / `button.formNoValidate` as boolean validation-suppression flags, while form submission metadata reflection covers `form.action` / `form.method` / `form.enctype` / `form.target` plus `input.formAction` / `button.formAction` / `input.formMethod` / `button.formMethod` / `input.formEnctype` / `button.formEnctype` / `input.formTarget` / `button.formTarget`, resolved against `document.baseURI`.
The same reflected select surface also includes `select.multiple` as the boolean multiple-selection flag on `<select>` elements.
The same reflected select surface also includes `select.type` as the read-only select-kind string (`select-one` / `select-multiple`) on `<select>` elements, and the same form-control reflection slice also includes `input.type` / `button.type` as reflected type strings on `<input>` / `<button>` controls.
The same reflected form-control surface also includes `required` as the boolean required flag on `input`, `textarea`, and `select` elements.
The same reflected form-control surface also includes `readOnly` as the boolean readonly flag on `input` and `textarea` elements.
The same reflected form-control surface also includes `indeterminate` as the checkbox-only indeterminate flag on `input[type=checkbox]`, and that state feeds the bounded `:indeterminate` selector while checkbox activation clears it.
The same reflected form-control surface also includes `defaultChecked` as the checkbox/radio default-checked state on `input` elements, and that state feeds the bounded `:default` selector.
The same reflected form-control surface also includes `accept` and `multiple` as the file-input configuration flags on `input` elements.
The same reflected form-control surface also includes `autocomplete` as the autocomplete string on `input` and `textarea` elements.
The same reflected form-control surface also includes `minLength` / `maxLength` as the reflected text-length constraints on `input` and `textarea` elements, and those constraints feed the bounded `:valid` / `:invalid` selector checks for text controls.
The same reflected form-control surface also includes `min` / `max` as the reflected string range bounds on `input` elements, `step` as the reflected step string on `input` elements, `size` as the non-negative size attribute on `input` elements defaulting to `20`, `rows` / `cols` as the non-negative textarea dimensions defaulting to `2` / `20`, and `wrap` as the textarea wrap mode defaulting to `soft`.
The same reflected form-control surface also includes `pattern` as the validation pattern on text inputs, and matching / mismatching values feed the bounded `:valid` / `:invalid` selector checks.
The same reflected form-control surface also includes `placeholder` as the placeholder string on `input` and `textarea` elements.
The same reflected form-control surface also includes `autofocus` as the boolean autofocus flag on `input`, `textarea`, `button`, and `select` elements.
The same reflected select surface also includes `select.size` as the non-negative size attribute on `<select>` elements, defaulting to `0` when absent.
The same reflected select surface also includes `select.value` as the current selected option value and `select.selectedIndex` as the active option index for that same selected state.
The same reflected option surface also includes `option.index` as the zero-based position within the owning select, and it updates when the option order changes.
The same reflected owner-form surface also includes `input.form` / `button.form` / `select.form` / `textarea.form` / `option.form` / `fieldset.form` / `output.form` / `object.form` / `embed.form` as the owning form element, and it updates when the control is moved out of its form; the current owner lookup follows ancestor forms only. `input.defaultValue` and `textarea.defaultValue` are reflected default-value surfaces on text controls, backed by the `value` attribute for `<input>` and the textarea's text content.
The same location mock also backs `document.location.href`, `document.location.hash`, `document.location.pathname`, `document.location.search`, `document.location.protocol`, `document.location.host`, `document.location.hostname`, `document.location.port`, `window.location.href`, `window.location.hash`, `window.location.pathname`, `window.location.search`, `window.location.protocol`, `window.location.host`, `window.location.hostname`, and `window.location.port`, and it resolves `toString()` / `valueOf()` to the current URL.
`document.styleSheets` is also available as a live `StyleSheetList` with `length`, `item()`, `keys()`, `values()`, `entries()`, `namedItem()`, and `forEach()`.
The same live `HTMLCollection` surfaces also expose legacy named property access on non-reserved names through the same `namedItem()` semantics, so patterns like `children.first` and `elements.mode` work in inline scripts.

This document explains how to actually build out the `` rewrite from its current Phase 6-complete baseline with Phase 7 slices 1 through 4 already delivered, plus sibling selector backlog slices (`A + B`, `A ~ B`), selector lists (`A, B`), bounded attribute selectors (`[attr=value]`, `[attr^=value]`, `[attr$=value]`, `[attr*=value]`) plus optional `i` / `s` flags, bounded pseudo-classes including `:not(...)`, `:is(...)`, `:where(...)`, `:scope`, `:has(...)`, `:lang(...)`, `:dir(...)`, `:link`, `:any-link`, `:placeholder-shown`, `:blank`, `:required`, `:optional`, `:focus`, `:focus-within`, `:target`, `:defined`, `:default`, `:valid`, `:invalid`, `:in-range`, `:out-of-range`, `:root`, `:empty`, `:only-child`, `:only-of-type`, `:first-of-type`, `:last-of-type`, `:nth-child(<positive integer>)`, `:nth-child(odd)`, `:nth-child(even)`, `:nth-child(an+b)`, `:nth-child(... of <selector-list>)`, `:nth-last-child(<positive integer>)`, `:nth-last-child(odd)`, `:nth-last-child(even)`, `:nth-last-child(an+b)`, `:nth-last-child(... of <selector-list>)`, `:nth-of-type(<positive integer>)`, `:nth-of-type(odd)`, `:nth-of-type(even)`, `:nth-of-type(an+b)`, `:nth-of-type(... of <selector-list>)`, `:nth-last-of-type(<positive integer>)`, `:nth-last-of-type(odd)`, `:nth-last-of-type(even)`, `:nth-last-of-type(an+b)`, `:nth-last-of-type(... of <selector-list>)`, and post-Phase-7 collection slices for `querySelectorAll` / minimal `NodeList`, `Element.children` / minimal `HTMLCollection`, `getElementsByTagName` / live `HTMLCollection`, `getElementsByTagNameNS` / live `HTMLCollection`, `getElementsByClassName` / live `HTMLCollection`, `getElementsByName` / live `NodeList`, `document.forms` / `form.elements` live `HTMLCollection`, `select.options` / `select.selectedOptions` live `HTMLCollection`, `fieldset.elements` / `datalist.options` live `HTMLCollection`, `map.areas` / `table.tBodies` live `HTMLCollection`, `document.documentElement` / `document.head` / `document.body` / `document.scrollingElement` / `document.title` / `window.title`, `document.childNodes` / `Node.childNodes` / `document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.anchors` live `NodeList` / `HTMLCollection`, `document.scripts` live `HTMLCollection`, `document.styleSheets` live `StyleSheetList` including `forEach()`, `document.all` live `HTMLCollection`, `template.content` live fragment-like collections, and `element.labels` on labelable form controls / fieldset live `NodeList`. Phase 8 is the next named milestone and scopes DOM mutation and reflection expansion, including child and sibling reflection helpers (`firstChild`, `lastChild`, `nextSibling`, `previousSibling`, `firstElementChild`, `lastElementChild`, `childElementCount`) plus node-equality helpers (`isSameNode()`, `isEqualNode()`). The inline-script runtime also exposes scroll position aliases (`window.scrollX`, `window.scrollY`, `window.pageXOffset`, `window.pageYOffset`) and `window.navigator` metadata plus `window.screen` (including `orientation.type` / `orientation.angle`) through the same runtime location mock and session state seam.

`Node.childNodes` is also available through `Node` values returned by collection iteration.

`:default`, `:valid`, `:invalid`, `:in-range`, `:out-of-range`, and `:indeterminate` are included in the bounded pseudo-class set used by the selector grammar broadening slices.

Use it together with:

- [architecture.md](architecture.md) for the target shape
- [subsystem-map.md](subsystem-map.md) for ownership decisions
- [capability-matrix.md](capability-matrix.md) for support-level decisions
- [roadmap.md](roadmap.md) for the staged milestone plan

## Core Rule

Do not grow `` by adding scattered features opportunistically.
Each change should add one capability through a narrow vertical slice:

1. decide the owning subsystem
2. define the contract with tests
3. implement inside the subsystem
4. connect through `Session`
5. expose through `Harness` only if it belongs in the public API
6. update docs when the capability becomes public

## Recommended Build Order

The safest way to turn the Phase 0 skeleton into a usable runtime is:

1. DOM bootstrap
2. selector subset
3. read-only assertions
4. script runtime minimum slice
5. event dispatch
6. form controls and default actions
7. deterministic mocks wired into runtime behavior
8. hardening and publication work

This order keeps the public facade thin and avoids implementing user actions before the DOM and selector layers are trustworthy.

Slices 1 through 22 are already implemented in this workspace, including public download capture. Phase 6 selector expansion slices 1 through 4, covering class selectors, compound simple selectors, descendant combinators, child combinators, and selector hardening, are also implemented in this workspace. A post-Phase-7 selector slice adds sibling combinators (`A + B`, `A ~ B`), selector lists (`A, B`), bounded attribute selectors (`[attr=value]`, `[attr^=value]`, `[attr$=value]`, `[attr*=value]`, `[attr~=value]`, `[attr|=value]`) plus optional `i` / `s` flags, and bounded pseudo-classes including `:not(...)`, `:is(...)`, `:root`, `:empty`, `:only-child`, `:only-of-type`, `:first-of-type`, `:last-of-type`, `:nth-child(<positive integer>)`, `:nth-child(odd)`, `:nth-child(even)`, `:nth-child(an+b)`, `:nth-last-child(<positive integer>)`, `:nth-last-child(odd)`, `:nth-last-child(even)`, `:nth-last-child(an+b)`, `:nth-of-type(<positive integer>)`, `:nth-of-type(odd)`, `:nth-of-type(even)`, `:nth-of-type(an+b)`, `:nth-of-type(... of <selector-list>)`, `:nth-last-of-type(<positive integer>)`, `:nth-last-of-type(odd)`, `:nth-last-of-type(even)`, and `:nth-last-of-type(an+b)`, `:nth-last-of-type(... of <selector-list>)` through the same bounded engine. Phase 7 script DOM query slices 1 through 4 (`document.querySelector`, `element.querySelector`, `Element.matches`, `Element.closest`, and selector hardening) are implemented as well. Post-Phase-7 collection slices add `querySelectorAll` with minimal `NodeList` support, `Element.children` with minimal `HTMLCollection` support, `getElementsByTagName` with live `HTMLCollection` support, `getElementsByTagNameNS` with live `HTMLCollection` support, `getElementsByClassName` with live `HTMLCollection` support, `getElementsByName` with live `NodeList` support, `document.forms` / `form.elements` live `HTMLCollection` support, `select.options` / `select.selectedOptions` live `HTMLCollection` support, `fieldset.elements` / `datalist.options` live `HTMLCollection` support, `map.areas` / `table.tBodies` live `HTMLCollection` support, `document.childNodes` / `document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.anchors` live `NodeList` / `HTMLCollection` support, `document.scripts` live `HTMLCollection` support, `document.styleSheets` live `StyleSheetList` support, `document.all` live `HTMLCollection` support, `template.content` live fragment-like collections, and `element.labels` on labelable form controls / fieldset live `NodeList` support. Additional selector work now includes `:lang(...)`, `:dir(...)`, `:link`, `:any-link`, `:placeholder-shown`, `:required`, and `:optional` in the bounded pseudo-class set. Phase 4 mock integration now also includes scroll capture through the same runtime seam.
The `document.styleSheets` slice specifically includes `namedItem()` and `forEach()` on the live `StyleSheetList` surface.

## First Vertical Slices

### Slice 1: Minimal HTML Tree Builder

Goal:

- `Harness::from_html(...)` should create a real document tree, not just store source text

Primary owner:

- `bt-dom`

Suggested scope:

- document node
- element nodes
- text nodes
- parent/child relationships
- basic attribute parsing

Tests to add first:

- `DomStore` builds a document with one child element
- nested elements preserve order
- text nodes are attached correctly
- malformed input returns an explicit parse error

Do not add yet:

- script execution
- event behavior
- form semantics

### Slice 2: Minimal Selector Subset

Goal:

- support `#id`, tag, and `[attr]` selection

Primary owner:

- `bt-dom`

Suggested scope:

- selector parsing for the guaranteed Phase 1 subset only
- explicit errors for unsupported selectors
- index-backed lookup where appropriate

Tests to add first:

- `#id` finds the expected node
- tag selectors return multiple nodes in document order
- `[attr]` matches presence
- unsupported selector syntax fails explicitly

Public outcome:

- `Harness::assert_exists(...)` can become the first real DOM-facing assertion

### Slice 3: Read-Only Debug and Assertion Layer

Goal:

- make the DOM inspectable before mutating it through user actions

Primary owner:

- `bt-dom` for the data
- `browser-tester` for the public assertion surface

Suggested scope:

- `assert_exists`
- `dump_dom`
- small DOM snippet support for assertion errors

Tests to add first:

- successful existence checks
- missing selector produces useful assertion text
- DOM dump is stable for simple fixtures

### Slice 4: Script Minimum Slice

Goal:

- make one narrow inline-script scenario work end to end

Primary owners:

- `bt-script`
- `bt-runtime`

Recommended first supported scenario:

- `document.getElementById("out").textContent = "Hello";`

Why this slice:

- it proves the binding seam
- it exercises DOM lookup and mutation
- it avoids jumping directly to broad JavaScript compatibility

Tests to add first:

- inline script mutates text content
- missing element access returns a script-side error
- unsupported syntax still fails explicitly

### Slice 5: Event Target Dispatch

Goal:

- register a listener and dispatch a simple event through target and ancestor listeners

Primary owner:

- `bt-runtime` for event orchestration
- `bt-script` for callback bridging

Suggested scope:

- listener registration
- capture/target/bubble dispatch
- deterministic callback order
- cancelable default actions

### Slice 6: Forms and User Actions

Goal:

- support the first realistic form test through `Harness`

Recommended order:

1. input value state
2. `type_text`
3. checkbox state
4. `set_checked`
5. select value state
6. `set_select_value`
7. button click default behavior
8. focus/blur behavior
9. submit behavior

Primary owners:

- `bt-dom` for control state
- `bt-runtime` for default-action orchestration
- `browser-tester` for thin public entry points

### Slice 7: Mock Integration

Goal:

- connect the typed mock registry to real runtime behavior

Recommended order:

1. dialogs
2. clipboard
3. print
4. close
5. open
6. scroll
7. fetch
8. location
9. file input
10. download capture

Why this order:

- dialogs, clipboard, print, close, open, and scroll are simpler than fetch/navigation
- fetch and location tend to widen the service surface quickly

Clipboard is exposed as a deterministic mock-backed compatibility shim through `navigator.clipboard.writeText()` / `navigator.clipboard.readText()` so tests can exercise copy and read flows without a browser permission model.

### Slice 8: Selector Expansion

Goal:

- extend the selector engine beyond the Phase 1 subset without introducing a broad CSS parser

Primary owner:

- `bt-dom`

Suggested scope:

- `.class`
- `tag.class`
- `#id.class`
- descendant combinators (`A B`)
- child combinators (`A > B`)
- adjacent sibling combinators (`A + B`)
- general sibling combinators (`A ~ B`)
- selector lists (`A, B`)
  - bounded pseudo-classes (`:not(...)`, `:is(...)`, `:lang(...)`, `:dir(...)`, `:link`, `:any-link`, `:placeholder-shown`, `:first-child`, `:last-child`, `:root`, `:empty`, `:only-child`, `:only-of-type`, `:first-of-type`, `:last-of-type`, `:nth-child(<positive integer>)`, `:nth-child(odd)`, `:nth-child(even)`, `:nth-child(an+b)`, `:nth-last-child(<positive integer>)`, `:nth-last-child(odd)`, `:nth-last-child(even)`, `:nth-last-child(an+b)`, `:nth-of-type(<positive integer>)`, `:nth-of-type(odd)`, `:nth-of-type(even)`, `:nth-of-type(an+b)`, `:nth-of-type(... of <selector-list>)`, `:nth-last-of-type(<positive integer>)`, `:nth-last-of-type(odd)`, `:nth-last-of-type(even)`, `:nth-last-of-type(an+b)`, `:nth-last-of-type(... of <selector-list>)`, `:checked`, `:disabled`, `:enabled`, `:read-only`, `:read-write`)
  - `:indeterminate`
- explicit failures for unsupported selector syntax

Tests already in place:

- class selectors match nodes in document order
- descendant combinators resolve nested nodes
- child combinators match only direct children
- adjacent sibling combinators match the immediate previous element sibling
- general sibling combinators match later element siblings in document order
- selector lists preserve document order and deduplicate results
- bounded pseudo-classes resolve against logical negation, bounded selector lists/combinators, structure, and simple form state
- public `Harness` actions and assertions continue to resolve selectors deterministically
- selector hardening remains explicit for unsupported syntax

Do not add yet:

- broader CSS parsing beyond the bounded selector grammar
- general CSS parsing

### Slice 9: Script DOM Query Expansion

Goal:

- make selector-based DOM lookup available inside inline scripts without broadening the JS grammar

Primary owners:

- `bt-script`
- `bt-runtime`

Delivered in this workspace:

- `document.querySelector(selector)`
- `element.querySelector(selector)`
- `Element.matches(selector)`
- `Element.closest(selector)`
- selector hardening and regression coverage
- document-order first match
- subtree-scoped lookup
- current-element selector checks
- ancestor-walk selector checks
- `null` on miss
- explicit errors for unsupported selector syntax

Remaining slices:

- none

Tests already in place:

- document-scoped lookup works in inline scripts
- subtree-scoped lookup works in inline scripts
- current-element selector checks work in inline scripts
- ancestor-walk selector checks work in inline scripts
- selector lists work in inline scripts
- bounded pseudo-classes work in inline scripts
- unsupported selector syntax remains explicit
- null-on-miss behavior is preserved
- invalid selector syntax fails explicitly

Do not add yet:

- broad CSS parsing
- unrelated DOM mutation APIs

### Slice 10: Query Selector Collections

Goal:

- make `querySelectorAll` available in inline scripts through a minimal collection surface

Primary owners:

- `bt-script`
- `bt-runtime`

Delivered in this workspace:

- `document.querySelectorAll(selector)`
- `element.querySelectorAll(selector)`
- `NodeList.length`
- `NodeList.item(index)`
- document-order snapshot results
- subtree-scoped lookup
- `null` on missing item

Tests already in place:

- document-scoped collection lookup works in inline scripts
- subtree-scoped collection lookup works in inline scripts
- selector list inputs work in inline scripts
- NodeList length and item access work in inline scripts
- unsupported NodeList methods fail explicitly

Do not add yet:

- broader collection APIs

### Slice 11: Element Children Collections

Goal:

- make `Element.children` available in inline scripts through a minimal live HTMLCollection surface

Primary owners:

- `bt-script`
- `bt-runtime`

Delivered in this workspace:

- `Element.children`
- `HTMLCollection.length`
- `HTMLCollection.item(index)`
- live child-element collection semantics

Tests already in place:

- child collections reflect DOM mutations in inline scripts
- `length` and `item()` work in inline scripts
- `namedItem()` works in inline scripts
- missing child items return `null`
- unsupported `HTMLCollection` methods fail explicitly

Do not add yet:

- broader collection APIs

### Slice 12: Tag-name Collections

Goal:

- make `getElementsByTagName` available in inline scripts through a live HTMLCollection surface

Primary owners:

- `bt-script`
- `bt-runtime`

Delivered in this workspace:

- `document.getElementsByTagName(tagName)`
- `element.getElementsByTagName(tagName)`
- live descendant tag-name collection semantics
- `HTMLCollection.length`
- `HTMLCollection.item(index)`
- `HTMLCollection.namedItem(name)`

Tests already in place:

- document-scoped tag-name lookup works in inline scripts
- element-scoped tag-name lookup works in inline scripts
- collections stay live after DOM mutations
- unsupported `HTMLCollection` methods fail explicitly

Do not add yet:

- broader collection APIs

### Slice 13: Class-Name and Name Collections

Goal:

- make `getElementsByClassName` and `getElementsByName` available in inline scripts through minimal live collection surfaces

Primary owners:

- `bt-script`
- `bt-runtime`

Delivered in this workspace:

- `document.getElementsByClassName(classNames)`
- `element.getElementsByClassName(classNames)`
- `document.getElementsByName(name)`
- live descendant class-name collection semantics
- live name-based `NodeList` semantics
- `HTMLCollection.length`
- `HTMLCollection.item(index)`
- `HTMLCollection.namedItem(name)`
- `NodeList.length`
- `NodeList.item(index)`

Tests already in place:

- class-name collections reflect DOM mutations in inline scripts
- name-based node lists reflect DOM mutations in inline scripts
- `length`, `item()`, and `namedItem()` work for class-name collections
- `length` and `item()` work for name-based node lists
- unsupported `Element.getElementsByName` fails explicitly

Do not add yet:

- broader collection APIs

### Slice 14: Form Collections

Goal:

- make `document.forms` and `form.elements` available in inline scripts through minimal live HTMLCollection surfaces

Primary owners:

- `bt-script`
- `bt-runtime`

Delivered in this workspace:

- `document.forms`
- `form.elements`
- live forms collection semantics
- live descendant form-control collection semantics
- `HTMLCollection.length`
- `HTMLCollection.item(index)`
- `HTMLCollection.namedItem(name)`
- `form.elements.namedItem(name)` returns `RadioNodeList` when multiple matching controls share the same name
- `RadioNodeList.keys()` / `RadioNodeList.values()` / `RadioNodeList.entries()` / `RadioNodeList.forEach()` expose snapshot iterator and callback helpers for multi-match groups
- `RadioNodeList.value = ...` updates the checked radio group state and clears the group when no radio matches

Tests already in place:

- document-scoped forms collection works in inline scripts
- form-scoped elements collection works in inline scripts
- collections stay live after DOM mutations
- `namedItem()` works for forms and form controls
- `namedItem()` returns `RadioNodeList` for multi-match form control groups
- `entries()` works on `RadioNodeList`
- assigning to `RadioNodeList.value` updates the checked radio group state
- non-form `elements` access fails explicitly

Do not add yet:

- broader collection APIs

### Slice 15: Select Options

Goal:

- make `select.options` available in inline scripts through a minimal live HTMLCollection surface with iterator helpers

Primary owners:

- `bt-script`
- `bt-runtime`

Delivered in this workspace:

- `select.options`
- live option collection semantics
- `HTMLCollection.length`
- `HTMLCollection.item(index)`
- `HTMLCollection.namedItem(name)`
- `HTMLCollection.keys()`
- `HTMLCollection.values()`
- `HTMLCollection.entries()`
- `HTMLCollection.forEach()`

Tests already in place:

- select option collections reflect DOM mutations in inline scripts
- `length`, `item()`, and `namedItem()` work for select options
- non-select `options` access fails explicitly

Do not add yet:

- broader collection APIs

### Slice 16: Specialized Document Live Collections

Goal:

- make `document.children`, `document.images`, `document.links`, `document.embeds` / `document.plugins`, and `document.anchors` available in inline scripts through minimal live HTMLCollection surfaces with iterator helpers

Primary owners:

- `bt-script`
- `bt-runtime`

Delivered in this workspace:

- `document.children`
- live child-element collection semantics on the document node
- `document.images`
- `document.links`
- `document.embeds` / `document.plugins`
- `document.anchors`
- live image collection semantics
- live link collection semantics filtered to `a[href]` and `area[href]`
- live anchor collection semantics filtered to `a[name]`
- `HTMLCollection.length`
- `HTMLCollection.item(index)`
- `HTMLCollection.namedItem(name)`

Tests already in place:

- `document.children` reflects DOM mutations in inline scripts
- `length`, `item()`, and `namedItem()` work for document children
- image, link, embed, and anchor collections reflect DOM mutations in inline scripts
- `length`, `item()`, and `namedItem()` work for document images, links, embeds, and anchors
- `document.all` reflects DOM mutations in inline scripts
- `length`, `item()`, and `namedItem()` work for document.all
- non-window `children` access fails explicitly
- non-document `images` access fails explicitly
- non-document `anchors` access fails explicitly

### Slice 17: Document All

Goal:

- make `document.all` available in inline scripts through a minimal live HTMLCollection surface

Primary owners:

- `bt-script`
- `bt-runtime`

Delivered in this workspace:

- `document.all`
- live all-elements collection semantics
- `HTMLCollection.length`
- `HTMLCollection.item(index)`
- `HTMLCollection.namedItem(name)`

Tests already in place:

- `document.all` reflects DOM mutations in inline scripts
- `length`, `item()`, and `namedItem()` work for document.all
- non-document `all` access fails explicitly

### Slice 17b: Template Content Live Collections

Goal:

- make `template.content.childNodes`, `template.content.children`, and fragment-scoped `querySelector(All)` available in inline scripts through a fragment-like live collection surface

Primary owners:

- `bt-script`
- `bt-runtime`

Delivered in this workspace:

- `template.content`
- `template.content.childNodes`
- `template.content.children`
- `template.content.getElementById(...)`
- `template.content.querySelector(...)`
- `template.content.querySelectorAll(...)`
- live fragment-like collection semantics linked to the template element's current content
- `NodeList.length`
- `NodeList.item(index)`
- `HTMLCollection.length`
- `HTMLCollection.item(index)`
- `HTMLCollection.namedItem(name)`

Tests already in place:

- `template.content` reflects DOM mutations in inline scripts
- `template.content.getElementById(...)`, `template.content.querySelector(...)`, and `template.content.querySelectorAll(...)` resolve within the template fragment
- `length`, `item()`, and `namedItem()` work for template content collections
- non-template `content` access fails explicitly

### Slice 18: Namespace-aware Tag-name Collections

Goal:

- make `getElementsByTagNameNS` available in inline scripts through a minimal live `HTMLCollection` surface

Primary owners:

- `bt-dom`
- `bt-script`
- `bt-runtime`

Delivered in this workspace:

- `document.getElementsByTagNameNS(namespaceUri, localName)`
- `element.getElementsByTagNameNS(namespaceUri, localName)`
- live namespace-aware descendant collection semantics
- bounded namespace support for HTML, SVG, and MathML URIs plus `*`
- `HTMLCollection.length`
- `HTMLCollection.item(index)`
- `HTMLCollection.namedItem(name)`

Tests already in place:

- namespace-aware collections resolve in inline scripts for document and element scope
- `length`, `item()`, and `namedItem()` work for namespace-aware collections
- detached scoped subtree behavior stays explicit through regression coverage
- arity mismatches fail explicitly

### Slice 19: Specialized Table Live Collections

Goal:

- make `table.rows`, `tbody.rows`, `thead.rows`, `tfoot.rows`, and `tr.cells` available in inline scripts through minimal live HTMLCollection surfaces

Primary owners:

- `bt-script`
- `bt-runtime`

Delivered in this workspace:

- `table.rows`
- `tbody.rows`
- `thead.rows`
- `tfoot.rows`
- `tr.cells`
- live table row collection semantics spanning table and section rows
- live cell collection semantics spanning direct row children
- `HTMLCollection.length`
- `HTMLCollection.item(index)`
- `HTMLCollection.namedItem(name)`

Tests already in place:

- table row and cell collections reflect DOM mutations in inline scripts
- `length`, `item()`, and `namedItem()` work for table rows and row cells
- non-table rows and non-row cells access fail explicitly

### Slice 20: Select Selected Options

Goal:

- make `select.selectedOptions` available in inline scripts through a minimal live HTMLCollection surface with iterator helpers

Primary owners:

- `bt-script`
- `bt-runtime`

Delivered in this workspace:

- `select.selectedOptions`
- live selected-option collection semantics
- `HTMLCollection.length`
- `HTMLCollection.item(index)`
- `HTMLCollection.namedItem(name)`
- `HTMLCollection.keys()`
- `HTMLCollection.values()`
- `HTMLCollection.entries()`
- `HTMLCollection.forEach()`

Tests already in place:

- selected-option collections reflect DOM mutations in inline scripts
- `length`, `item()`, and `namedItem()` work for selected options
- non-select `selectedOptions` access fails explicitly

### Slice 21: Additional Specialized Live Collections

Goal:

- make the remaining specialized live collection surfaces available in inline scripts without broadening the public `Harness`

Primary owners:

- `bt-script`
- `bt-runtime`

Delivered in this workspace:

- `fieldset.elements`
- `datalist.options`
- `map.areas`
- `table.tBodies`
- live `HTMLCollection` semantics through the same bounded collection surface

Tests already in place:

- specialized live collections reflect DOM mutations in inline scripts
- `length`, `item()`, and `namedItem()` work for the new collection surfaces
- unsupported host elements fail explicitly

### Slice 22: Labelable Element Labels

Goal:

- expose `element.labels` as a live `NodeList` on labelable form controls and `fieldset` without broadening the public `Harness`

Primary owners:

- `bt-script`
- `bt-runtime`

Delivered in this workspace:

- `element.labels`
- live `NodeList` semantics through the same bounded collection surface
- explicit `label[for]` associations and implicit ancestor-label associations

Tests already in place:

- labels reflect DOM mutations in inline scripts
- `length` and `item()` work for labelable-element labels
- non-labelable element access fails explicitly

### Phase 8: DOM Mutation and Reflection Expansion

Goal:

- make common DOM mutation and reflection flows work from inline script without broadening the public surface unnecessarily
- keep selectors, collections, and event/default-action behavior deterministic after mutation

Primary owners:

- `bt-dom`
- `bt-script`
- `bt-runtime`

Delivered slices:

1. attribute reflection (delivered in this workspace)
   - `getAttribute`, `setAttribute`, `removeAttribute`, `hasAttribute`, `toggleAttribute`
   - `Element.attributes` as a live `NamedNodeMap`
   - `document.createAttribute()` / `document.createAttributeNS()`
   - `getAttributeNode()` / `getAttributeNodeNS()` / `setAttributeNode()` / `setAttributeNodeNS()` / `removeAttributeNode()`
   - `NamedNodeMap.setNamedItem()` / `NamedNodeMap.setNamedItemNS()` / `NamedNodeMap.removeNamedItem()` / `NamedNodeMap.removeNamedItemNS()` / `NamedNodeMap.keys()` / `NamedNodeMap.values()` / `NamedNodeMap.entries()` / `NamedNodeMap.forEach()`
   - `Attr.specified` / `Attr.isId`
   - reflected ID, class, name, checked, disabled, selected, required, read-only, autocomplete, placeholder, autofocus, value, and range-bounds state
2. class and dataset views (delivered in this workspace)
   - `className`
   - `classList`
   - `classList.value`
   - `classList.toString()`
   - `classList.replace()`
   - `classList.item()`
   - `classList.forEach()`
   - `classList.keys()` / `classList.values()` / `classList.entries()`
   - `dataset`
3. tree mutation primitives (delivered in this workspace, including `before()`, `after()`, and `replaceWith()`)
   - `append`, `prepend`, `before`, `after`, `remove`
   - `appendChild`, `insertBefore`, `replaceChild`, `replaceChildren`
4. HTML serialization surfaces (delivered in this workspace)
   - `innerHTML`
   - `outerHTML`
   - bounded fragment insertion paths that reuse the existing HTML parser
   - `document.open()`
   - `document.write()`
   - `document.writeln()`
   - `document.close()`
5. mutation hardening and regression coverage (delivered)
   - selectors and collections stay consistent after mutation
   - unsupported mutation semantics fail explicitly, and mixed-quote attribute values serialize browser-style instead of failing

Tests already in place:

- tree mutation preserves document order, parent/child links, and explicit errors on invalid moves
- HTML serialization surfaces round-trip the bounded HTML subset, including `document.open()` / `document.write()` / `document.writeln()` / `document.close()`; `document.open()` / `document.close()` return `document` for chainable compatibility, and `document.write()` can append into the open document tree after the current root has been cleared

Tests in place for the delivered slices:

- serialization surfaces round-trip the bounded HTML subset
- failure-path tests cover unsupported mutation semantics and mutation hardening regressions, while browser-style mixed-quote attribute escaping and basic character reference decoding, including common named references such as `&nbsp;`, `&copy;`, and `&reg;` plus safe semicolonless forms like `&nbsp` / `&amp` / `&lt` / `&gt` / `&copy` / `&reg`, legacy uppercase variants like `&AMP` / `&LT` / `&GT` / `&QUOT` / `&NBSP` / `&COPY` / `&REG`, are covered by round-trip serialization tests

Do not add yet:

- broad browser compatibility
- unrelated rendering or layout behavior
- broad collection or CSS parser expansion just because mutation exists

### Post-Phase-8 Backlog Slices

Use these when the next user-visible gap lands:

| Track | Implementation order | Owner | Typical tests |
| --- | --- | --- | --- |
| Collection API slice 1 (`NodeList.forEach`, `HTMLCollection.forEach`) | 1. `NodeList.forEach` / `HTMLCollection.forEach` (delivered) | `bt-script` | public contract, owning-crate regression, callback execution smoke test |
| Collection API slice 2 (`document.scripts`) | 1. `document.scripts` (delivered) | `bt-script` + `bt-runtime` | public contract, owning-crate regression, live collection hardening |
| Collection API slice 3 (`document.anchors`) | 1. `document.anchors` (delivered) | `bt-script` + `bt-runtime` | public contract, owning-crate regression, live collection hardening |
| Collection API slice 4 (`NodeList.keys()`, `NodeList.values()`, `NodeList.entries()`, `HTMLCollection.keys()`, `HTMLCollection.values()`, `HTMLCollection.entries()`) | 1. iterator-style helpers (delivered) | `bt-script` | public contract, owning-crate regression, snapshot iterator hardening |
| Collection API slice 5 (`document.applets`) | 1. `document.applets` (delivered) | `bt-script` + `bt-runtime` | public contract, owning-crate regression, live collection hardening |
| Collection API slice 6 (`document.children`) | 1. `document.children` (delivered) | `bt-script` + `bt-runtime` | public contract, owning-crate regression, live collection hardening |
| Collection API slice 7 (`table.rows`, `tbody.rows`, `thead.rows`, `tfoot.rows`, `tr.cells`) | 1. `table.rows` / `tr.cells` (delivered) | `bt-script` + `bt-runtime` | public contract, owning-crate regression, live collection hardening |
| Collection API slice 8 (`document.childNodes`) | 1. `document.childNodes` (delivered) | `bt-script` + `bt-runtime` | public contract, owning-crate regression, live `NodeList` hardening |
| Collection API slice 13 (`template.content.childNodes`, `template.content.children`) | 1. `template.content` live fragment-like collection surface (delivered), including `template.content.getElementById(...)` / `template.content.querySelector(...)` / `template.content.querySelectorAll(...)` and direct child mutation primitives, including `removeChild(...)` | `bt-script` + `bt-runtime` | public contract, owning-crate regression, template-content hardening |
| Collection API broadening remainder (specialized live collections) | 1. additional specialized live collections bundle (`fieldset.elements`, `datalist.options`, `map.areas`, `table.tBodies`) is delivered; 2. `element.labels` on labelable form controls / fieldset is delivered; 3. `document.styleSheets` is delivered; 4. `document.childNodes` is delivered; 5. `template.content` is delivered, including `getElementById(...)` / `querySelector(All)` on the fragment surface and direct child mutation primitives; 6. `select.options.add()` / `select.options.remove()` is delivered; the remaining collection API broadening slices are further specialized live collections beyond the current bounded set | `bt-script` + `bt-runtime` | public contract, owning-crate regression, unsupported-method failure |
| Selector grammar broadening | 1. `:scope` (delivered) 2. `:has(...)` (delivered) 3. structural selector expansion with richer nested selector handling (delivered) 4. `:lang(...)` / `:dir(...)` / `:link` / `:any-link` / `:placeholder-shown` (delivered) 5. bounded `:nth-of-type(... of <selector-list>)` / `:nth-last-of-type(... of <selector-list>)` (delivered) | `bt-dom` | public contract, DOM matcher regression, explicit unsupported syntax |
| HTML serialization broadening | 1. `insertAdjacentHTML` (delivered) 2. `insertAdjacentElement()` / `insertAdjacentText()` (delivered) 3. `template.content.innerHTML` / `DocumentFragment` serialization (delivered) 4. namespace-aware serialization compatibility (delivered) 5. browser-style mixed-quote attribute escaping and basic character reference decoding, including common named references such as `&nbsp;`, `&copy;`, and `&reg;` plus safe semicolonless forms like `&nbsp` / `&amp` / `&lt` / `&gt` / `&copy` / `&reg`, legacy uppercase variants like `&AMP` / `&LT` / `&GT` / `&QUOT` / `&NBSP` / `&COPY` / `&REG`, and semicolonless numeric forms like `&#160` / `&#xA0` (delivered) 6. `document.open()` / `document.write()` / `document.writeln()` / `document.close()` helpers (delivered) | `bt-dom` + `bt-script` + `bt-runtime` | round-trip success, malformed failure, runtime regression |

## Decision Flow for Each Change

Before implementing anything, answer these questions:

1. Which crate owns the long-lived state?
2. Is this a stable public API, a test-only mock, a debug helper, or internal-only machinery?
3. Can it be expressed as a smaller capability slice?
4. What is the explicit failure behavior for unsupported input?
5. Which test layer should lock this in?

If the answers are unclear, the change is probably too wide.

## Test Strategy by Stage

### Early Phases

Prefer:

- subsystem tests
- small public contract tests
- explicit failure-path tests

Avoid:

- large issue-driven regression fixtures before a capability stabilizes
- browser-comparison work before the public contract is defined

### Once a Capability Is Public

Require:

- a public contract test
- a subsystem test in the owning crate
- at least one failure-path test
- docs that match the actual API

### For New Test-Only Mocks

Require:

- response injection or seed-state coverage
- failure-path coverage
- call capture or artifact capture coverage
- README update
- `doc/mock-guide.md` update

## Update Rules

Update these files when the corresponding decisions change:

- `doc/capability-matrix.md`: support level or public guarantee changed
- `doc/mock-guide.md`: a new public mock or new capture mode exists
- `doc/subsystem-map.md`: ownership guidance changed
- `README.md`: user-visible public entry point changed

## Practical Working Loop

Use this loop for daily implementation work:

1. pick one slice from the current phase
2. decide the owning subsystem
3. write or extend tests
4. implement the subsystem behavior
5. wire through `Session`
6. expose through `Harness` only if needed
7. update docs
8. run `cargo test`

## Recommended Immediate Next Task

The next highest-value task is:

1. pick the next user-visible gap or regression cluster
2. decide the owning subsystem first
3. add contract, subsystem, and failure-path tests before implementing

That is the general post-Phase-7 working mode after the script-side selector query family is complete.
