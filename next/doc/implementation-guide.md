# Implementation Guide

The bounded script host surface also includes `document.title` and `window.title`.
It also includes `document.location`, `window.location`, `document.URL`, `document.documentURI`, `document.baseURI`, `element.baseURI`, `document.origin`, `window.origin`, and `element.origin` through the runtime location mock.

This document explains how to actually build out the `next/` rewrite from its current Phase 6-complete baseline with Phase 7 slices 1 through 4 already delivered, plus sibling selector backlog slices (`A + B`, `A ~ B`), selector lists (`A, B`), bounded attribute selectors (`[attr=value]`, `[attr^=value]`, `[attr$=value]`, `[attr*=value]`) plus optional `i` / `s` flags, bounded pseudo-classes including `:not(...)`, `:is(...)`, `:scope`, `:has(...)`, `:lang(...)`, `:dir(...)`, `:link`, `:any-link`, `:placeholder-shown`, `:required`, `:optional`, `:focus`, `:focus-within`, `:target`, `:root`, `:empty`, `:only-child`, `:only-of-type`, `:first-of-type`, `:last-of-type`, `:nth-child(<positive integer>)`, `:nth-child(odd)`, `:nth-child(even)`, `:nth-child(an+b)`, `:nth-child(... of <selector-list>)`, `:nth-last-child(<positive integer>)`, `:nth-last-child(odd)`, `:nth-last-child(even)`, `:nth-last-child(an+b)`, `:nth-last-child(... of <selector-list>)`, `:nth-of-type(<positive integer>)`, `:nth-of-type(odd)`, `:nth-of-type(even)`, `:nth-of-type(an+b)`, `:nth-last-of-type(<positive integer>)`, `:nth-last-of-type(odd)`, `:nth-last-of-type(even)`, and `:nth-last-of-type(an+b)`, and post-Phase-7 collection slices for `querySelectorAll` / minimal `NodeList`, `Element.children` / minimal `HTMLCollection`, `getElementsByTagName` / live `HTMLCollection`, `getElementsByTagNameNS` / live `HTMLCollection`, `getElementsByClassName` / live `HTMLCollection`, `getElementsByName` / live `NodeList`, `document.forms` / `form.elements` live `HTMLCollection`, `select.options` / `select.selectedOptions` live `HTMLCollection`, `fieldset.elements` / `datalist.options` live `HTMLCollection`, `map.areas` / `table.tBodies` live `HTMLCollection`, `document.documentElement` / `document.head` / `document.body` / `document.title` / `window.title`, `document.childNodes` / `document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.anchors` live `NodeList` / `HTMLCollection`, `document.scripts` live `HTMLCollection`, `document.styleSheets` live `StyleSheetList`, `document.all` live `HTMLCollection`, `template.content` live fragment-like collections, and `element.labels` on labelable form controls / fieldset live `NodeList`. Phase 8 is the next named milestone and scopes DOM mutation and reflection expansion.

Use it together with:

- [architecture.md](architecture.md) for the target shape
- [subsystem-map.md](subsystem-map.md) for ownership decisions
- [capability-matrix.md](capability-matrix.md) for support-level decisions
- [roadmap.md](roadmap.md) for the staged milestone plan

## Core Rule

Do not grow `next/` by adding scattered features opportunistically.
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

Slices 1 through 22 are already implemented in this workspace, including public download capture. Phase 6 selector expansion slices 1 through 4, covering class selectors, compound simple selectors, descendant combinators, child combinators, and selector hardening, are also implemented in this workspace. A post-Phase-7 selector slice adds sibling combinators (`A + B`, `A ~ B`), selector lists (`A, B`), bounded attribute selectors (`[attr=value]`, `[attr^=value]`, `[attr$=value]`, `[attr*=value]`, `[attr~=value]`, `[attr|=value]`) plus optional `i` / `s` flags, and bounded pseudo-classes including `:not(...)`, `:is(...)`, `:root`, `:empty`, `:only-child`, `:only-of-type`, `:first-of-type`, `:last-of-type`, `:nth-child(<positive integer>)`, `:nth-child(odd)`, `:nth-child(even)`, `:nth-child(an+b)`, `:nth-last-child(<positive integer>)`, `:nth-last-child(odd)`, `:nth-last-child(even)`, `:nth-last-child(an+b)`, `:nth-of-type(<positive integer>)`, `:nth-of-type(odd)`, `:nth-of-type(even)`, `:nth-of-type(an+b)`, `:nth-last-of-type(<positive integer>)`, `:nth-last-of-type(odd)`, `:nth-last-of-type(even)`, and `:nth-last-of-type(an+b)` through the same bounded engine. Phase 7 script DOM query slices 1 through 4 (`document.querySelector`, `element.querySelector`, `Element.matches`, `Element.closest`, and selector hardening) are implemented as well. Post-Phase-7 collection slices add `querySelectorAll` with minimal `NodeList` support, `Element.children` with minimal `HTMLCollection` support, `getElementsByTagName` with live `HTMLCollection` support, `getElementsByTagNameNS` with live `HTMLCollection` support, `getElementsByClassName` with live `HTMLCollection` support, `getElementsByName` with live `NodeList` support, `document.forms` / `form.elements` live `HTMLCollection` support, `select.options` / `select.selectedOptions` live `HTMLCollection` support, `fieldset.elements` / `datalist.options` live `HTMLCollection` support, `map.areas` / `table.tBodies` live `HTMLCollection` support, `document.childNodes` / `document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.anchors` live `NodeList` / `HTMLCollection` support, `document.scripts` live `HTMLCollection` support, `document.styleSheets` live `StyleSheetList` support, `document.all` live `HTMLCollection` support, `template.content` live fragment-like collections, and `element.labels` on labelable form controls / fieldset live `NodeList` support. Additional selector work now includes `:lang(...)`, `:dir(...)`, `:link`, `:any-link`, `:placeholder-shown`, `:required`, and `:optional` in the bounded pseudo-class set.

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
3. fetch
4. location
5. file input
6. download capture

Why this order:

- dialogs and clipboard are simpler than fetch/navigation
- fetch and location tend to widen the service surface quickly

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
  - bounded pseudo-classes (`:not(...)`, `:is(...)`, `:lang(...)`, `:dir(...)`, `:link`, `:any-link`, `:placeholder-shown`, `:first-child`, `:last-child`, `:root`, `:empty`, `:only-child`, `:only-of-type`, `:first-of-type`, `:last-of-type`, `:nth-child(<positive integer>)`, `:nth-child(odd)`, `:nth-child(even)`, `:nth-child(an+b)`, `:nth-last-child(<positive integer>)`, `:nth-last-child(odd)`, `:nth-last-child(even)`, `:nth-last-child(an+b)`, `:nth-of-type(<positive integer>)`, `:nth-of-type(odd)`, `:nth-of-type(even)`, `:nth-of-type(an+b)`, `:nth-last-of-type(<positive integer>)`, `:nth-last-of-type(odd)`, `:nth-last-of-type(even)`, `:nth-last-of-type(an+b)`, `:checked`, `:disabled`, `:enabled`)
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

- broader CSS parsing beyond the bounded selector grammar, including malformed or unknown attribute selector flags such as `[attr=value x]`
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

Tests already in place:

- document-scoped forms collection works in inline scripts
- form-scoped elements collection works in inline scripts
- collections stay live after DOM mutations
- `namedItem()` works for forms and form controls
- `namedItem()` returns `RadioNodeList` for multi-match form control groups
- non-form `elements` access fails explicitly

Do not add yet:

- broader collection APIs

### Slice 15: Select Options

Goal:

- make `select.options` available in inline scripts through a minimal live HTMLCollection surface

Primary owners:

- `bt-script`
- `bt-runtime`

Delivered in this workspace:

- `select.options`
- live option collection semantics
- `HTMLCollection.length`
- `HTMLCollection.item(index)`
- `HTMLCollection.namedItem(name)`

Tests already in place:

- select option collections reflect DOM mutations in inline scripts
- `length`, `item()`, and `namedItem()` work for select options
- non-select `options` access fails explicitly

Do not add yet:

- broader collection APIs

### Slice 16: Specialized Document Live Collections

Goal:

- make `document.children`, `document.images`, `document.links`, `document.embeds` / `document.plugins`, and `document.anchors` available in inline scripts through minimal live HTMLCollection surfaces

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

- make `template.content.childNodes` and `template.content.children` available in inline scripts through a fragment-like live collection surface

Primary owners:

- `bt-script`
- `bt-runtime`

Delivered in this workspace:

- `template.content`
- `template.content.childNodes`
- `template.content.children`
- live fragment-like collection semantics linked to the template element's current content
- `NodeList.length`
- `NodeList.item(index)`
- `HTMLCollection.length`
- `HTMLCollection.item(index)`
- `HTMLCollection.namedItem(name)`

Tests already in place:

- `template.content` reflects DOM mutations in inline scripts
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

- make `select.selectedOptions` available in inline scripts through a minimal live HTMLCollection surface

Primary owners:

- `bt-script`
- `bt-runtime`

Delivered in this workspace:

- `select.selectedOptions`
- live selected-option collection semantics
- `HTMLCollection.length`
- `HTMLCollection.item(index)`
- `HTMLCollection.namedItem(name)`

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
   - reflected ID, class, name, checked, disabled, selected, and value state
2. class and dataset views (delivered in this workspace)
   - `className`
   - `classList`
   - `dataset`
3. tree mutation primitives (delivered in this workspace)
   - `append`, `prepend`, `before`, `after`, `remove`
   - `appendChild`, `insertBefore`, `replaceChild`, `replaceChildren`
4. HTML serialization surfaces (delivered in this workspace)
   - `innerHTML`
   - `outerHTML`
   - bounded fragment insertion paths that reuse the existing HTML parser
5. mutation hardening and regression coverage (delivered)
   - selectors and collections stay consistent after mutation
   - unsupported or lossy mutation semantics fail explicitly

Tests already in place:

- tree mutation preserves document order, parent/child links, and explicit errors on invalid moves
- HTML serialization surfaces round-trip the bounded HTML subset

Tests in place for the delivered slices:

- serialization surfaces round-trip the bounded HTML subset
- failure-path tests cover unsupported or lossy mutation semantics and mutation hardening regressions

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
| Collection API slice 4 (`NodeList.keys()`, `NodeList.values()`, `HTMLCollection.keys()`, `HTMLCollection.values()`) | 1. iterator-style helpers (delivered) | `bt-script` | public contract, owning-crate regression, snapshot iterator hardening |
| Collection API slice 5 (`document.applets`) | 1. `document.applets` (delivered) | `bt-script` + `bt-runtime` | public contract, owning-crate regression, live collection hardening |
| Collection API slice 6 (`document.children`) | 1. `document.children` (delivered) | `bt-script` + `bt-runtime` | public contract, owning-crate regression, live collection hardening |
| Collection API slice 7 (`table.rows`, `tbody.rows`, `thead.rows`, `tfoot.rows`, `tr.cells`) | 1. `table.rows` / `tr.cells` (delivered) | `bt-script` + `bt-runtime` | public contract, owning-crate regression, live collection hardening |
| Collection API slice 8 (`document.childNodes`) | 1. `document.childNodes` (delivered) | `bt-script` + `bt-runtime` | public contract, owning-crate regression, live `NodeList` hardening |
| Collection API slice 13 (`template.content.childNodes`, `template.content.children`) | 1. `template.content` live fragment-like collection surface (delivered) | `bt-script` + `bt-runtime` | public contract, owning-crate regression, template-content hardening |
| Collection API broadening remainder (specialized live collections) | 1. additional specialized live collections bundle (`fieldset.elements`, `datalist.options`, `map.areas`, `table.tBodies`) is delivered; 2. `element.labels` on labelable form controls / fieldset is delivered; 3. `document.styleSheets` is delivered; 4. `document.childNodes` is delivered; 5. `template.content` is delivered; the remaining collection API broadening slices are further specialized live collections beyond the current bounded set | `bt-script` + `bt-runtime` | public contract, owning-crate regression, unsupported-method failure |
| Selector grammar broadening | 1. `:scope` (delivered) 2. `:has(...)` (delivered) 3. structural selector expansion with richer nested selector handling (delivered) 4. `:lang(...)` / `:dir(...)` / `:link` / `:any-link` / `:placeholder-shown` (delivered) | `bt-dom` | public contract, DOM matcher regression, explicit unsupported syntax |
| HTML serialization broadening | 1. `insertAdjacentHTML` (delivered) 2. `template.content.innerHTML` / `DocumentFragment` serialization (delivered) 3. namespace-aware serialization compatibility (delivered) | `bt-dom` + `bt-script` + `bt-runtime` | round-trip success, lossy / malformed failure, runtime regression |

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
