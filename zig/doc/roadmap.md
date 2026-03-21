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
- user-facing `Harness` actions

Phase 3 is complete in this workspace.

## Phase 4: Determinism and Mocks

Delivered in this workspace:

- fake clock helpers (`nowMs`, `advanceTime`, `flush`)
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

The attribute reflection, class/dataset view, tree mutation, HTML serialization, and namespace-aware serialization slices are implemented in this workspace now, and collection API broadening slices 1 (`NodeList.forEach`), 2 (`document.scripts`), 3 (`document.anchors`), 4 (`NodeList.keys()` / `NodeList.values()` / `HTMLCollection.keys()` / `HTMLCollection.values()` / `entries()`), 5 (`Element.children` / `document.children` / `document.childNodes` / `element.childNodes` / `template.content.childNodes` / `template.content.children`), 6 (`document.forms`), 7 (`form.elements`), 8 (`select.options`), 9 (`select.selectedOptions`), 10 (`fieldset.elements`), 11 (`datalist.options`), 12 (`map.areas`), 13 (`table.tBodies`), 14 (`element.labels`), 15 (`document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.applets` / `document.all`), 16 (`document.styleSheets`), 17 (`table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells`), 18 (`getElementsByTagName` / `getElementsByTagNameNS` / `getElementsByClassName` / `getElementsByName`), 19 (`entries()` helpers across `NodeList`, `HTMLCollection`, `StyleSheetList`, and `RadioNodeList`), and 20 (`select.options.add()` / `select.options.remove()`) are implemented as well.
The minimal inline style declaration slice, including comment stripping, `!important` priority handling, and `getPropertyPriority(...)`, is implemented; the next named work is broader CSS parsing beyond the bounded selector engine, if a specific user-visible gap needs it.
`RadioNodeList.value` is writable in the form-elements slice, and unmatched assignments clear the checked radio group in this workspace.

## Phase 9: Document and Window Surface Expansion

- `document.documentElement`, `document.head`, `document.body`, `document.activeElement`, `document.referrer`, `document.dir`, `window.children`, `window.scrollX`, `window.scrollY`, `window.pageXOffset`, `window.pageYOffset`, and `window.name`
- `document.title` and `window.title`
- `document.location` and `window.location` as a limited `Location` host object (`href`, `assign()`, `replace()`, `reload()`)
- `document.URL`, `document.documentURI`, `document.baseURI`, `document.compatMode`, `document.characterSet`, `document.charset`, and `document.contentType`
- `document.origin`, `window.origin`, `Element.baseURI`, and `Element.origin`

The document and window alias slice is implemented in this workspace now, including the document metadata, `document.referrer`, `document.dir`, `window.children`, `window.scrollX`, `window.scrollY`, `window.pageXOffset`, `window.pageYOffset`, and `window.name` aliases used during inline script bootstrap.

## Phase 10: Limited Navigation Model

- `window.history`
- `history.length` and `history.state`
- `back()`, `forward()`, and `go(delta)`
- `pushState(...)` and `replaceState(...)`

The limited history navigation slice is implemented in this workspace now.
