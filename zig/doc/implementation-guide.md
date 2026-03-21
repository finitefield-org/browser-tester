# Implementation Guide

This document explains how to grow the Zig rewrite without turning `Harness` into a catch-all facade.

## Core Rule

Do not grow the workspace by adding scattered features opportunistically.
Each change should add one capability through a narrow vertical slice:

1. decide the owning subsystem
2. define the contract with tests
3. implement inside the subsystem
4. connect through `Session`
5. expose through `Harness` only if it belongs in the public API
6. update docs when the capability becomes public

## Recommended Build Order

The safest order for turning the phase 0 scaffold into a usable runtime is:

1. DOM bootstrap
2. selector subset
3. read-only assertions
4. script runtime minimum slice
5. event dispatch
6. forms and user actions
7. deterministic mocks
8. hardening and publication work

The DOM bootstrap, selector expansion, read-only inspection slices, the phase 2 script runtime minimum slice, the phase 3 event/default-action and form-control slice, the phase 4 deterministic mock slice, the phase 5 hardening suite, the phase 7 query selector and collection slices, the phase 8 attribute reflection, class/dataset view, inline style declaration (with comment stripping, `!important` priority handling, and `getPropertyPriority(...)`), tree mutation, HTML serialization, and namespace-aware serialization slices, the phase 9 document/window alias slice, the phase 10 limited navigation slice, the limited `Location` host object navigation slice, and collection API broadening slices 1 (`NodeList.forEach`), 2 (`document.scripts`), 3 (`document.anchors`), 4 (`NodeList.keys()` / `NodeList.values()` / `HTMLCollection.keys()` / `HTMLCollection.values()` / `entries()`), 5 (`Element.children` / `document.children` / `document.childNodes` / `element.childNodes` / `template.content.childNodes` / `template.content.children`), 6 (`document.forms`), 7 (`form.elements`), 8 (`select.options`), 9 (`select.selectedOptions`), 10 (`fieldset.elements`), 11 (`datalist.options`), 12 (`map.areas`), 13 (`table.tBodies`), 14 (`element.labels`), 15 (`document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.applets` / `document.all`), 16 (`document.styleSheets`), 17 (`table.rows` / `tbody.rows` / `thead.rows` / `tfoot.rows` / `tr.cells`), 18 (`getElementsByTagName` / `getElementsByTagNameNS` / `getElementsByClassName` / `getElementsByName`), 19 (`entries()` helpers across `NodeList`, `HTMLCollection`, `StyleSheetList`, and `RadioNodeList`), and 20 (`select.options.add()` / `select.options.remove()`) are landed as well. The `:scope` pseudo-class slice is also landed; the `:has(...)` pseudo-class slice is also landed; the `:lang(...)` / `:dir(...)` pseudo-class slices are also landed; the `:not(...)` / `:is(...)` / `:where(...)` selector-list pseudo-class slice, the bounded structural/state pseudo-class slice, the `:defined` pseudo-class slice, and the focus/target/nth pseudo-class slice are also landed, including `of <selector-list>` support on the `nth-child`, `nth-last-child`, `nth-of-type`, and `nth-last-of-type` families; broader CSS parsing beyond the bounded selector engine remains deferred until a specific user-visible gap needs it. The inline style declaration slice now also strips CSS comments and preserves `!important` priority during serialization. The document/window alias slice also includes `document.compatMode`, `document.characterSet`, `document.charset`, `document.contentType`, `document.referrer`, `document.dir`, `document.activeElement`, `window.scrollX`, `window.scrollY`, `window.pageXOffset`, `window.pageYOffset`, `window.name`, and `window.children`. The phase 4 deterministic mock slice now also includes script-side `window.localStorage` and `window.sessionStorage` surfaces backed by seedable storage maps.
The open/close/print/scroll mock slice is part of the same phase 4 work, and `HarnessBuilder.openFailure(...)` / `HarnessBuilder.closeFailure(...)` / `HarnessBuilder.printFailure(...)` / `HarnessBuilder.scrollFailure(...)` keep those calls deterministic during bootstrap.
`RadioNodeList.value` is writable in the form-elements slice, and unmatched assignments clear the checked radio group in this workspace.

This order keeps the public facade thin and avoids implementing user actions before the DOM and selector layers are trustworthy.

## Public API Rules

- Ask whether a new method really belongs on `Harness`.
- Prefer internal subsystem APIs when they can stay hidden.
- Use a mock family instead of growing a pile of one-off setters.
- Keep unsupported behavior explicit; do not add silent fallback paths.

## Test Strategy

When a capability becomes public, aim to add:

- a public contract test
- a failure-path test
- a subsystem-level test
- documentation updates in the same change

Large regression fixtures should wait until the capability is stable enough to benefit from them.
