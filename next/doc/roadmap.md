# Roadmap

This roadmap mirrors the staged plan from [`next.md`](../../next.md) and turns it into the working order for `next/`.

For day-to-day implementation sequencing inside each phase, also see [implementation-guide.md](implementation-guide.md).

Status note:

- Phases 0 through 7 are complete in this workspace.
- Detached HTML element / text node / comment / fragment construction via `document.createElement()`, `document.createTextNode()`, `document.createComment()`, and `document.createDocumentFragment()` is available through the same host-binding seam and remains part of the current script surface, with `before()`, `after()`, `cloneNode()`, `remove()`, `removeChild()`, `normalize()`, `importNode()`, and `replaceWith()` on existing nodes and `cloneNode()` / `textContent` on `template.content` available for detached cloning and reflection.
- Phase 8 is now scoped as DOM mutation and reflection expansion. Phase 7 is complete in this workspace; slices 1 through 4 are delivered, sibling selectors (`A + B`, `A ~ B`), selector lists (`A, B`), and a small pseudo-class slice are also delivered as backlog slices, `:scope`, `:has(...)`, `:lang(...)`, `:dir(...)`, `:link`, `any-link`, `:placeholder-shown`, `:required`, `:optional`, `:focus`, `:focus-within`, `:target`, `:defined`, `:default`, `:valid`, `:invalid`, `:in-range`, `:out-of-range`, and `:indeterminate` are delivered as selector grammar broadening slices, including bounded `:nth-child(... of <selector-list>)`, `:nth-last-child(... of <selector-list>)`, `:nth-of-type(... of <selector-list>)`, and `:nth-last-of-type(... of <selector-list>)` support, and post-Phase-7 collection slices add `querySelectorAll` with minimal `NodeList` support plus `Element.children`, `getElementsByTagName`, `getElementsByTagNameNS`, `getElementsByClassName`, `getElementsByName`, `document.forms` / `form.elements` (with `RadioNodeList` from `namedItem()` when multiple controls share a name), `select.options` / `select.selectedOptions` plus `select.options.add()` / `select.options.remove()`, `fieldset.elements` / `datalist.options`, `map.areas` / `table.tBodies`, `document.childNodes` / `document.children` / `document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.anchors`, `document.scripts`, `document.styleSheets` / `StyleSheetList.forEach()`, `document.all`, `template.content` and `template.content.querySelector(All)`, and `element.labels` on labelable form controls / fieldset collection support. Legacy named property access on those live `HTMLCollection` surfaces is also delivered in this workspace. Future work should stay backlog-driven. Phase 8 slices 1 through 5 are delivered; the phase is complete in this workspace. `document.location`, `window.location`, `document.URL`, `document.documentURI`, `document.baseURI`, `element.baseURI`, `document.currentScript`, `document.readyState`, `document.compatMode`, `document.characterSet`, `document.charset`, `document.contentType`, `document.visibilityState`, `document.hidden`, `document.activeElement`, `document.scrollingElement`, `document.hasFocus()`, `document.referrer`, `document.cookie`, `document.domain`, `document.designMode`, `form.noValidate`, `input.formNoValidate`, `button.formNoValidate`, `document.location.pathname`, `document.location.search`, `document.location.origin`, `document.location.protocol`, `document.location.host`, `document.location.hostname`, `document.location.port`, `window.location.pathname`, `window.location.search`, `window.location.origin`, `window.location.protocol`, `window.location.host`, `window.location.hostname`, `window.location.port`, `window.name`, `window.self`, `window.window`, `window.parent`, `window.top`, `window.closed`, `window.history.length` / `window.history.state` / `window.history.scrollRestoration` / `window.history.pushState()` / `window.history.replaceState()` / `window.history.back()` / `window.history.forward()` / `window.history.go()`, `window.localStorage`, `window.sessionStorage`, `window.children`, `window.frames` as a live `HTMLCollection` of descendant `iframe` / `frame` elements, `window.length` as the frame-count alias, `window.frameElement` / `window.opener` as read-only null surfaces, `form.length` / `select.length` as read-only aliases to their underlying live collections, `firstChild` / `lastChild` / `nextSibling` / `previousSibling` / `nextElementSibling` / `previousElementSibling` / `firstElementChild` / `lastElementChild` / `childElementCount` reflection helpers, `window.navigator` (`userAgent`, `appCodeName`, `appName`, `appVersion`, `product`, `productSub`, `vendor`, `vendorSub`, `pdfViewerEnabled`, `doNotTrack`, `javaEnabled()`, `plugins`, `mimeTypes`, `platform`, `language`, `userLanguage`, `browserLanguage`, `systemLanguage`, `oscpu`, `cookieEnabled`, `onLine`, `webdriver`, `hardwareConcurrency`, `maxTouchPoints`, `devicePixelRatio`, `innerWidth`, `innerHeight`, `outerWidth`, `outerHeight`, `screenX`, `screenY`, `screenLeft`, `screenTop`, `window.screen` (`availWidth`, `availHeight`, `availLeft`, `availTop`, `colorDepth`, `pixelDepth`, `orientation.type`, `orientation.angle`)), and `window.open()` / `window.close()` / `window.print()` / `window.scrollTo()` / `window.scrollBy()` / `window.scrollX` / `window.scrollY` / `window.pageXOffset` / `window.pageYOffset` during inline script bootstrap are also available as the runtime location mock / URL alias surface. `window.navigator.languages` and `window.navigator.mimeTypes` now also expose `keys()` / `values()` / `entries()` helpers through the same bounded collection layer.
- The same reflected element surface also includes `element.spellcheck` and `element.inputMode` alongside the other reflected element attributes.
- Phase 8 also exposes `Node.isConnected` / `Element.isConnected` / `Document.isConnected` and `Node.contains()` / `Element.contains()` / `Document.contains()` / `Node.compareDocumentPosition()` / `Element.compareDocumentPosition()` / `Document.compareDocumentPosition()` / `Node.hasChildNodes()` / `Element.hasChildNodes()` / `Document.hasChildNodes()` / `Node.nextSibling` / `Element.nextSibling` / `Document.nextSibling` / `Node.previousSibling` / `Element.previousSibling` / `Document.previousSibling` / `Node.nextElementSibling` / `Element.nextElementSibling` / `Node.previousElementSibling` / `Element.previousElementSibling` / `Node.firstChild` / `Element.firstChild` / `Document.firstChild` / `Node.lastChild` / `Element.lastChild` / `Document.lastChild` as read-only tree-connectivity, containment, and tree-order reflection helpers, with detached `template.content` remaining disconnected.
- The same Phase 8 reflection surface also includes `Node.isSameNode()` / `Element.isSameNode()` / `Document.isSameNode()` and `Node.isEqualNode()` / `Element.isEqualNode()` / `Document.isEqualNode()`.
- The same location mock also resolves `toString()` / `valueOf()` to the current URL.
- `document.childNodes`, `template.content`, `document.styleSheets`, `document.children`, `table.rows` / `tr.cells`, `select.selectedOptions`, `fieldset.elements`, `datalist.options`, `map.areas`, `table.tBodies`, and `element.labels` on labelable form controls / fieldset with live `NodeList` support are also implemented as additional specialized live collection slices; `template.content` now also exposes `textContent`, and direct parent `removeChild()` fits the same tree mutation family.
- `element.contentEditable` / `element.isContentEditable` / `Node.ownerDocument` / `Element.ownerDocument` / `Node.parentNode` / `Element.parentNode` / `Element.parentElement` are also implemented as bounded element reflection surfaces.

## Phase 0: Skeleton

Delivered in this workspace:

- independent Rust workspace under `next/`
- `HarnessBuilder`
- `Session`
- `DomStore`
- scheduler and mock registry skeletons
- error taxonomy
- design-document set

Exit criteria:

- build an empty session without HTML
- keep URL and storage seed configuration
- compile and test the workspace

## Phase 1: DOM Core

Delivered in this workspace:

- HTML parser
- DOM tree construction
- selector subset
- `assert_exists`
- `dump_dom`

Exit criteria:

- build DOM from HTML text
- resolve `#id`, tag, and attribute selectors

## Phase 2: Script Runtime

Delivered in this workspace:

- lexer
- parser
- evaluator
- host bindings for `window`, `document`, and `Element`
- inline script execution

Exit criteria:

- simple DOM mutation through script works
- event handlers can be registered

## Phase 3: Events and Forms

Delivered in this workspace:

- event dispatch with ancestor bubbling and capture listeners
- default action registry for checkbox toggles and submit buttons
- form controls for text input, textarea, checkbox, and select state
- `click`, `type_text`, `set_checked`, `set_select_value`, `focus`, `blur`, `submit`, `dispatch`
- `assert_value` and `assert_checked`

Exit criteria:

- common form-oriented UI tests pass

## Phase 4: Determinism and Mocks

Delivered in this workspace:

- fake clock hardening
- microtask queue semantics
- fetch mock
- dialogs
- clipboard
- open
- print
- location mock, including `window.location.href`, `document.location.href`, `window.location.hash`, `document.location.hash`, `window.location.pathname`, `document.location.pathname`, `window.location.search`, `document.location.search`, `window.location.origin`, `document.location.origin`, `window.location.assign()`, `window.location.replace()`, and `window.location.reload()`
- file input mock
- download capture

Exit criteria:

- realistic mock-heavy tests can be expressed through the public facade

## Phase 5: Hardening

Delivered in this workspace:

- public contract tests
- subsystem tests
- regression suite
- property tests
- documentation polish
- publish checklist
- quick and hardening test profiles

Exit criteria:

- docs and code agree
- quick and hardening test profiles both exist

## Phase 6: Selector Expansion

Delivered slices 1 through 4:

- class selectors and compound simple selectors
  - `.class`, `tag.class`, `#id.class`
  - `DomStore::select`, `assert_exists`, and action resolution now share the selector engine

- descendant combinators
  - `A B`
  - nested DOM matching and document-order behavior now work through the same selector engine

- child combinators
  - `A > B`
  - direct-child matching and false-positive avoidance now work through the same selector engine

Phase 6 complete:

- selector hardening and regression coverage are delivered

Exit criteria:

- `DomStore::select` resolves the bounded selector set deterministically
- existing `Harness` assertions and actions work with the new selector forms
- unsupported syntax still fails explicitly
- quick and hardening profiles stay green

## Phase 7: Script DOM Query Expansion

Delivered slices 1 through 4:

- `document.querySelector(...)` and `element.querySelector(...)`
  - document-order first match, subtree-scoped lookup, `null` on miss

- `Element.matches(...)`
  - current-element only, boolean return

- `Element.closest(...)`
  - self-inclusive ancestor walk, `null` on miss

Post-Phase-7 slices:

- query selector collections
  - `querySelectorAll` with minimal `NodeList` support
  - `length` and `item()` only
- element child collections
  - `Element.children` with minimal `HTMLCollection` support
  - `length`, `item()`, and `namedItem()`
- tag-name collections
  - `getElementsByTagName` with minimal `HTMLCollection` support
  - `length`, `item()`, and `namedItem()`
- namespace-aware tag-name collections
  - `getElementsByTagNameNS` with minimal `HTMLCollection` support
  - `length`, `item()`, and `namedItem()`
  - bounded to the HTML, SVG, and MathML namespace URIs plus `*`
- class-name collections
  - `getElementsByClassName` with live `HTMLCollection` support
  - `length`, `item()`, and `namedItem()`
- name collections
  - `getElementsByName` with live `NodeList` support
  - `length` and `item()`
- form collections
  - `document.forms` with live `HTMLCollection` support
  - `form.elements` with live `HTMLCollection` support
  - `form.elements.namedItem()` can return `RadioNodeList` when multiple matching controls share a name, and `RadioNodeList.entries()` is available on those groups
  - `length`, `item()`, and `namedItem()`
- select collections
  - `select.options` with live `HTMLCollection` support
  - `length`, `item()`, and `namedItem()`
- image, link, and embed collections
  - `document.images` with live `HTMLCollection` support
  - `document.links` with live `HTMLCollection` support
  - `document.embeds` / `document.plugins` with live `HTMLCollection` support
  - `length`, `item()`, and `namedItem()`
- anchors collection
  - `document.anchors` with live `HTMLCollection` support
  - `length`, `item()`, and `namedItem()`
- all-elements collection
  - `document.all` with live `HTMLCollection` support
  - `length`, `item()`, and `namedItem()`
- stylesheet collection
  - `document.styleSheets` with live `StyleSheetList` support
  - `length` and `item()`
- bounded attribute selectors
  - `[attr=value]`, `[attr^=value]`, `[attr$=value]`, `[attr*=value]`, `[attr~=value]`, and `[attr|=value]` are available through the same bounded engine
  - unquoted and quoted value forms are supported
  - optional `i` / `s` attribute selector flags are available through the same bounded engine
- selector lists
  - comma-separated selector lists are available through the same bounded engine
  - document-order union and deduplication
  - bounded pseudo-classes
  - `:not(...)`, `:is(...)`, `:focus`, `:focus-within`, `:target`, `:defined`, `:first-child`, `:last-child`, `:root`, `:empty`, `:only-child`, `:only-of-type`, `:first-of-type`, `:last-of-type`, `:nth-child(<positive integer>)`, `:nth-child(odd)`, `:nth-child(even)`, `:nth-child(an+b)`, `:nth-last-child(<positive integer>)`, `:nth-last-child(odd)`, `:nth-last-child(even)`, `:nth-last-child(an+b)`, `:nth-of-type(<positive integer>)`, `:nth-of-type(odd)`, `:nth-of-type(even)`, `:nth-of-type(an+b)`, `:nth-last-of-type(<positive integer>)`, `:nth-last-of-type(odd)`, `:nth-last-of-type(even)`, `:nth-last-of-type(an+b)`, `:checked`, `:disabled`, `:enabled`, `:default`, `:valid`, `:invalid`, `:in-range`, `:out-of-range`, `:read-only`, `:read-write`, and `:indeterminate` are available through the same bounded engine
  - bounded selector lists and combinators inside `:not(...)` and `:is(...)` are available; broader CSS parsing stays deferred, while a single trailing unknown attribute selector flag token such as `[attr=value x]` is tolerated as a bounded compatibility fallback

Exit criteria:

- inline scripts can use selector-based lookup without a new `Harness` API
- selector grammar stays bounded and deterministic
- unsupported syntax still fails explicitly
- docs, contract tests, and regression tests agree

## Phase 8: DOM Mutation and Reflection Expansion

Delivered in this workspace:

- attribute reflection (delivered)
- class and dataset views (delivered)
- tree mutation primitives, including `before()` / `after()` / `removeChild()` / `replaceWith()` (delivered)
- HTML serialization surfaces (delivered)
- mutation hardening and regression coverage (delivered)

Purpose:

- make common DOM mutation flows usable from inline script without broadening the public `Harness`
- keep selectors, collections, and event/default-action surfaces deterministic after mutations
- avoid turning the rewrite into a broad browser-compatibility project

Ownership:

- `bt-dom` owns DOM mutation state, attribute reflection, and tree/index updates
- `bt-script` owns method dispatch and return-value wrapping for the mutation methods
- `bt-runtime` owns runtime wiring and regression visibility for side effects

Suggested slices:

1. attribute reflection (delivered)
   - `getAttribute`, `setAttribute`, `removeAttribute`, `hasAttribute`, `toggleAttribute`
   - reflected ID, class, name, checked, disabled, selected, and value state
2. class and dataset views (delivered)
   - `className`
   - `classList`
   - `classList.replace()`
   - `dataset`
3. tree mutation primitives (delivered, including `before()` / `after()` / `removeChild()` / `replaceWith()`)
   - `append`, `prepend`, `before`, `after`, `remove`, `removeChild`
   - `appendChild`, `insertBefore`, `replaceChild`, `replaceChildren`
4. HTML serialization surfaces (delivered)
   - `innerHTML`
   - `outerHTML`
   - bounded fragment insertion paths that reuse the existing HTML parser
5. mutation hardening and regression coverage (delivered)
   - selectors and collections stay consistent after mutation
   - unsupported mutation semantics fail explicitly, and mixed-quote attribute values serialize browser-style instead of failing

Exit criteria:

- common DOM mutation flows work in inline scripts
- selectors and collections observe mutations deterministically
- unsupported mutation semantics fail explicitly, and mixed-quote attribute values serialize browser-style instead of failing
- no new `Harness` API is added unless the scenario cannot already be expressed
- docs, contract tests, and regression tests agree

## After Phase 8

Operating rule:

- future work should stay backlog-driven and narrow
- the next slices are:
  1. collection API broadening slice 1 (`NodeList.forEach` / `HTMLCollection.forEach`) is delivered
  2. collection API broadening slice 2 (`document.scripts`, including iterator helpers) is delivered
  3. collection API broadening slice 3 (`document.anchors`) is delivered
  4. collection API broadening slice 4 (`NodeList.keys()` / `NodeList.values()` / `NodeList.entries()` / `HTMLCollection.keys()` / `HTMLCollection.values()` / `HTMLCollection.entries()`) is delivered; the remaining collection API broadening slices are further specialized live collections beyond the current bounded set
  5. collection API broadening slice 5 (`document.applets`) is delivered; collection API broadening slice 6 (`document.children`) is delivered; collection API broadening slice 7 (`table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells`) is delivered; collection API broadening slice 8 (`select.selectedOptions`) is delivered; collection API broadening slice 9 (`fieldset.elements` / `datalist.options` / `map.areas` / `table.tBodies`) is delivered; collection API broadening slice 10 (`element.labels` on labelable form controls / fieldset) is delivered; collection API broadening slice 11 (`document.styleSheets`) is delivered; collection API broadening slice 12 (`document.childNodes`) is delivered; collection API broadening slice 13 (`template.content.childNodes` / `template.content.children`) is delivered; collection API broadening slice 14 (`select.options.add()` / `select.options.remove()`) is delivered; collection-like `toString()` helpers are also available; the remaining collection API broadening slices are further specialized live collections beyond the current bounded set
  6. selector grammar broadening, with `:scope`, `:has(...)`, `:lang(...)`, `:dir(...)`, `:link`, `:any-link`, `:placeholder-shown`, `:required`, `:optional`, `:focus`, `:focus-within`, `:target`, `:defined`, structural selector expansion, and bounded `:nth-child(... of <selector-list>)` / `:nth-last-child(... of <selector-list>)` / `:nth-of-type(... of <selector-list>)` / `:nth-last-of-type(... of <selector-list>)` delivered
  7. HTML serialization broadening, starting with `insertAdjacentHTML` (delivered), then `insertAdjacentElement()` / `insertAdjacentText()` (delivered), then `template.content.innerHTML` / `DocumentFragment` serialization (delivered), then namespace-aware serialization compatibility (delivered), then browser-style mixed-quote attribute escaping and basic character reference decoding, including common named references such as `&nbsp;`, `&copy;`, and `&reg;` plus safe semicolonless forms like `&nbsp` / `&amp` / `&lt` / `&gt` / `&copy` / `&reg`, legacy uppercase variants like `&AMP` / `&LT` / `&GT` / `&QUOT` / `&NBSP` / `&COPY` / `&REG`, and semicolonless numeric forms like `&#160` / `&#xA0` (delivered), then `document.open()` / `document.write()` / `document.writeln()` / `document.close()` (delivered, with `document.open()` / `document.close()` returning `document` and `document.write()` appending into the open document tree after the current root is cleared)
- `querySelectorAll` with minimal `NodeList` support is available, `Element.children`, `document.childNodes`, `document.children`, `table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells`, `getElementsByTagName`, `getElementsByTagNameNS`, `getElementsByClassName`, `getElementsByName`, `document.forms` / `form.elements` (including `RadioNodeList` from `namedItem()` when multiple controls share a name, with `RadioNodeList.entries()` available on those groups), `select.options` / `select.selectedOptions`, `fieldset.elements` / `datalist.options`, `map.areas` / `table.tBodies`, `element.labels` on labelable form controls / fieldset, `document.images`, `document.links`, `document.embeds`, `document.plugins`, `document.anchors`, `document.applets`, `document.scripts` (including iterator helpers), `document.styleSheets` (including `keys()` / `values()` / `entries()`), and `document.all` collection support are also available; `NodeList.forEach` / `HTMLCollection.forEach` and `NodeList.keys()` / `NodeList.values()` / `NodeList.entries()` / `HTMLCollection.keys()` / `HTMLCollection.values()` / `HTMLCollection.entries()` are now available too, collection-like `toString()` helpers are also available, and selector lists plus the bounded pseudo-class subset are also available through the bounded engine

## After Phase 8: Rolling Capability Delivery

Operating rule:

1. pick one user-visible gap or regression cluster
2. decide the owning subsystem first
3. lock the change in with public contract, subsystem, and failure-path tests
4. implement inside the owning subsystem
5. expose through `Harness` only if the scenario cannot already be expressed
6. update public docs in the same change when the supported surface changes

Open a new named phase only when:

- multiple upcoming slices share one cross-cutting milestone
- that milestone needs its own exit criteria and user-facing status boundary

Until then, keep shipping backlog-driven capability slices under this post-Phase-7 mode.
