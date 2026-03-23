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
5. event dispatch and script-side focus helpers
6. forms and user actions
7. deterministic mocks
8. hardening and publication work

The detached construction slice (`document.createElement()`, `document.createElementNS()`, `document.createAttribute()`, `document.createAttributeNS()`, `document.createTextNode()`, `document.createComment()`, and `document.createDocumentFragment()`) is also landed. Attribute node accessors (`getAttributeNode()`, `getAttributeNodeNS()`, `setAttributeNode()`, `setAttributeNodeNS()`, and `removeAttributeNode()`) are available too, and `Element.attributes` exposes a minimal read-only `NamedNodeMap` snapshot with `keys()`, `values()`, and `entries()`. `Element.innerText` is available as a deterministic `textContent`-like alias on Element nodes. The tree mutation slice also includes `removeChild()`, `insertAdjacentElement()`, and `insertAdjacentText()`, while `createElementNS()` is still limited to the HTML, SVG, and MathML namespaces. The phase 3 selection slice also dispatches document-level `selectionchange` handlers from the supported selection APIs and exposes minimal `window.getSelection()` / `document.getSelection()` snapshots for the active text control with `collapseToStart()` / `collapseToEnd()` / `containsNode(...)` / `removeAllRanges()` / `collapse(node, offset)` / `extend(node, offset)` / `setBaseAndExtent(...)` / `deleteFromDocument()` / `setPosition(node, offset)` / `selectAllChildren(node)` / `addRange(range)` / `removeRange(range)` / `getRangeAt(0)` / `Range.cloneRange()` / `Range.cloneContents()` / `Range.deleteContents()` / `Range.isPointInRange(...)` / `Range.comparePoint(...)` / `Range.compareBoundaryPoints(...)` / `Range.extractContents()` helpers.

The minimal `CSSStyleSheet.cssRules` slice for inline `<style>` sheets is landed too, including bounded `@media` / `@supports` / `@document` / `@container` / `@starting-style` / `@position-try` / `@scope` / `@keyframes` / `@font-face` / `@font-feature-values` / `@font-palette-values` rules with `CSSFontPaletteValuesRule` exposing `name`, `fontFamily`, `basePalette`, `overrideColors`, and writable `CSSFontPaletteValuesRule.cssText` / `@color-profile` / `@page` / `@layer` / `@property` block rules with writable `CSSPropertyRule.cssText` and `@counter-style` rules with writable `CSSCounterStyleRule.cssText`, plus `@import` / `@namespace` statements, and `CSSStyleRule.style` is exposed as a live writable `CSSStyleDeclaration` with writable top-level `CSSStyleRule.selectorText`, `CSSStyleRule.cssText`, `CSSPageRule.selectorText` for top-level `@page` rules, `CSSPageRule.cssText`, writable `CSSPageRule.style`, and `CSSRule.type` exposes the legacy CSSOM integer mapping for classic rule kinds (with newer at-rules returning `0`), and `CSSRule.parentStyleSheet` and `CSSRule.parentRule` return the owning stylesheet and owning rule on rule objects, with `CSSStyleSheet.media.mediaText` writable and `CSSStyleSheet.media.appendMedium()` / `deleteMedium()` available on stylesheet media lists, `CSSMediaRule.media` is writable and read-only `CSSMediaRule.matches` mirrors the seeded `window.matchMedia(...)` result, and `CSSImportRule.media` is read-only minimal `MediaList` surfaces, `CSSImportRule.supportsText` / `CSSImportRule.layerName` exposed as read-only metadata, and legacy `CSSStyleSheet.rules` / `addRule()` / `removeRule()` aliases are available alongside `CSSStyleSheet.insertRule()` / `deleteRule()` / `replaceSync()` on inline `<style>` owners. Stylesheet owner elements also expose reflected `media`, `rel`, `relList`, `relList.supports()` / `relList.replace()`, `as`, `charset`, `imageSrcset`, `imageSizes`, `fetchPriority`, `hreflang`, `crossOrigin`, `referrerPolicy`, `integrity`, and `disabled`. The CSSOM View computed-style slice (`window.getComputedStyle(element[, pseudoElt])`) is also landed; it returns a minimal live read-only `CSSStyleDeclaration` view backed by the element's inline style attribute, while unsupported pseudo-elements and write attempts fail explicitly. The CSSOM View geometry slice is landed too; `Element.getBoundingClientRect()` returns a minimal read-only `DOMRect` snapshot derived from inline style and does not attempt full layout or viewport geometry. A buffered `document.open()` / `document.write()` / `document.writeln()` / `document.close()` slice is landed too, browser-style mixed-quote attribute escaping is used by `outerHTML`-style getters, and basic character reference decoding is applied when parsing HTML text and attributes; replay flushes accumulated markup on `close()`. If you need the content to appear earlier in the pipeline, this slice belongs immediately after the HTML serialization surfaces and before the detached construction slice. Nested @keyframes rules also expose writable `CSSKeyframeRule.keyText`, plus `CSSKeyframesRule.appendRule()`, `deleteRule()`, and `findRule()`. `CSSFontFeatureValuesRule.cssText` / `CSSColorProfileRule.cssText` are writable too.
`CSSContainerRule` also exposes `containerName` and `containerQuery`, with `containerQuery` returning the specified query text without logical simplification, and `CSSContainerRule.conditionText` is writable on `@container` rules while preserving nested rules, and `CSSContainerRule.cssText` is writable through the same bounded rewrite path. `CSSSupportsConditionRule` exposes `name` and an empty deterministic `cssRules` slice for `@supports-condition` rules, but the inner support-condition block is not semantically interpreted yet. `CSSSupportsRule.conditionText` is writable on `@supports` rules and rewrites the owning block in place while preserving nested rules, and `CSSSupportsRule.cssText` is writable through the same bounded rewrite path.
`HTMLAnchorElement.href` / `HTMLAnchorElement.rel` / `HTMLAnchorElement.relList` / `HTMLAnchorElement.download` / `HTMLAnchorElement.target` / `HTMLAnchorElement.ping` / `HTMLAnchorElement.hreflang` / `HTMLAnchorElement.referrerPolicy` / `HTMLAnchorElement.type` / `HTMLAreaElement.href` / `HTMLAreaElement.rel` / `HTMLAreaElement.relList` / `HTMLAreaElement.download` / `HTMLAreaElement.target` / `HTMLAreaElement.ping` / `HTMLAreaElement.alt` / `HTMLAreaElement.coords` / `HTMLAreaElement.shape` / `HTMLAreaElement.noHref` / `HTMLAreaElement.hreflang` / `HTMLAreaElement.referrerPolicy` / `HTMLAreaElement.type` are reflected string attributes that feed the deterministic download/open click slice, and `HTMLAnchorElement.text` is exposed as a `textContent` alias on anchors, while `click()` honors the area target/open observation slice.
The stylesheet owner element slice also includes reflected `type` and `referrerPolicy` on `HTMLLinkElement`, and `HTMLScriptElement` also exposes reflected `src`, `charset`, `text`, `type`, `async`, `defer`, `noModule`, `crossOrigin`, `integrity`, `referrerPolicy`, and `fetchPriority` metadata.

The form submission reflection slice (`form.action`, `form.method`, `form.enctype`, `form.encoding`, `form.target`, `form.acceptCharset`, `formAction`, `formMethod`, `formEnctype`, and `formTarget`) is also landed; action URLs resolve against the current location and method/enctype stay limited to known values, `form.submit()` / `form.requestSubmit()` / `form.reset()` dispatch `submit` and `reset` events without real navigation, form-associated controls also expose read-only `form` owner reflection through the explicit `form` attribute or the nearest owning `form` / `select` chain, `form.elements` stays in document order while including controls associated via `form=` outside the subtree, and minimal `checkValidity()` / `reportValidity()` support is available on `input`, `textarea`, `select`, and `form`, with `reportValidity()` dispatching deterministic `invalid` events on invalid controls, while supported `number` / `range` / `date` / `datetime-local` / `time` / `month` / `week` controls also expose `valueAsNumber` getters and setters and `valueAsDate` getters and setters, but the workspace still does not model real network submission or navigation.
Those same controls also expose `stepUp()` / `stepDown()` using the workspace's step-aware date/time/month/week handling.

The document/window alias slice also includes `document.scrollingElement`, `window.frames`, `window.length`, `window.frameElement`, and `window.history.scrollRestoration`, and the scroll alias surface dispatches deterministic `scroll` events through `document.onscroll` and `window.onscroll`; page lifecycle transitions dispatch deterministic `beforeunload` / `pagehide` / `unload` / `pageshow` events through `window.onbeforeunload`, `window.onpagehide`, `window.onunload`, and `window.onpageshow`.
Bootstrap completion also dispatches deterministic `DOMContentLoaded` events before `readystatechange`.
The document/window alias surface also includes the deterministic `window.screen` geometry object with a fixed `orientation.type` / `orientation.angle` pair, `window.Math` exposes the standard constants plus deterministic `Math.random()`, and the screen-position quartet (`window.screenX`, `window.screenY`, `window.screenLeft`, and `window.screenTop`) is implemented as deterministic constants.
`document.domain` should follow the same URL-derived read-only model: keep the host parser small, deterministic, and explicit about rejected assignments.
`document.cookie` should follow the same session-owned cookie jar model: keep the parser small, deterministic, and explicit about malformed assignments.
The open/close/print/scroll mock slice is part of the same phase 4 work, and `HarnessBuilder.openFailure(...)` / `HarnessBuilder.closeFailure(...)` / `HarnessBuilder.printFailure(...)` / `HarnessBuilder.scrollFailure(...)` keep those calls deterministic during bootstrap; `window.print()` also dispatches deterministic `beforeprint` / `afterprint` lifecycle handlers.
`RadioNodeList.value` is writable in the form-elements slice, and unmatched assignments clear the checked radio group in this workspace.

This order keeps the public facade thin and avoids implementing user actions before the DOM and selector layers are trustworthy.

The selector expansion slice now also includes the bounded structural/state pseudo-classes, including `:blank`, and the focus-related pseudo-classes, including `:focus-visible`, so new selector work should keep that vertical slice narrow and test-driven.

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

If a slice needs a different deterministic `Math.random()` or `crypto.randomUUID()` sequence, seed it through `HarnessBuilder.randomSeed(...)` before `build()`.
The document/window alias slice now also includes `Node.contains(...)`, `Node.compareDocumentPosition(...)`, `Node.isSameNode(...)`, `Node.isEqualNode(...)`, `Node.nodeValue`, `Node.data`, `document.onreadystatechange`, `window.onbeforeunload`, `window.onpagehide`, `window.onunload`, `window.onpageshow`, and the minimal `window.navigator.mimeTypes` `PluginArray`-like surface plus `window.navigator.languages` `DOMStringList`-like surface with `keys()`, `values()`, `entries()`, and `forEach(callback[, thisArg])`, plus `window.navigator.plugins` with `refresh()` and the deterministic legacy aliases `userLanguage`, `browserLanguage`, `systemLanguage`, and `oscpu`.
The CSSOM View geometry slice now also includes `Element.getBoundingClientRect()` as a minimal read-only `DOMRect` snapshot and `Element.getClientRects()` as a one-rect read-only `DOMRectList` snapshot, both derived from inline style and both intentionally short of full layout or viewport geometry, and `Element.scrollIntoView()` is available as a deterministic scroll-to-origin alias over the same mock-driven scroll path.
`CSSPageRule.selectorText` is writable for top-level `@page` rules and uses the same bounded stylesheet rewrite path as `CSSStyleRule.selectorText`; `CSSPageRule.cssText` is writable through the same bounded stylesheet rewrite path as `CSSStyleRule.cssText`.
