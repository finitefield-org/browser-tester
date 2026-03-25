# browser-tester

This repository is a clean-room rewrite workspace for `browser-tester`.

It is intentionally organized around the staged plan from [`next.md`](next.md):

- split the runtime into explicit subsystems
- keep `Harness` as a thin public facade
- treat deterministic mocks as first-class APIs
- keep the initial surface small and documented

Current status:

- a compilable Rust workspace exists under `crates/`
- `HarnessBuilder`, `Session`, `DomStore`, scheduler, mock registry, and error taxonomy are in place
- Phase 1 DOM parsing, selector subset support, Phase 6 selector expansion slices 1 through 4, and sibling selectors (`A + B`, `A ~ B`) are implemented
- Phase 2 inline script bootstrapping, `document.getElementById(...).textContent = ...`, listener registration, and `Number.prototype.toFixed()` / `Number.prototype.toPrecision()` / `Number.prototype.toExponential()` on numeric values are implemented
- Phase 3 event dispatch, ancestor bubbling, cancelable click default actions, form controls, `click`, `type_text`, `set_checked`, `set_select_value`, `focus`, `blur`, `submit`, `dispatch`, `assert_value`, and `assert_checked` are implemented
- Phase 4 fake clock hardening, microtask semantics, and runtime mock wiring for fetch, dialogs (including script-accessible `window.alert()`, `window.confirm()`, and `window.prompt()`), clipboard, location aliases (`document.location`, `document.location.href`, `document.location.hash`, `document.location.pathname`, `document.location.search`, `document.location.origin`, `window.location`, `window.location.href`, `window.location.hash`, `window.location.pathname`, `window.location.search`, `window.location.origin`, `document.URL`, `document.documentURI`, `document.baseURI`, `element.baseURI`, `element.tagName`, `element.localName`, `element.namespaceURI`, `document.origin`, `document.referrer`, `document.cookie`, `document.domain`, `document.designMode`, `element.accessKey`, `element.slot`, `element.autocapitalize`, `element.translate`, `element.dir`, `element.lang`, `element.title`, `element.role`, `element.ariaLabel`, `element.ariaHidden`, `element.tabIndex`, `element.hidden`, `element.contentEditable`, `element.isContentEditable`, `Node.ownerDocument`, `Element.ownerDocument`, `Node.parentNode`, `Element.parentNode`, `Element.parentElement`, `nextSibling` / `previousSibling` / `nextElementSibling` / `previousElementSibling` / `firstChild` / `lastChild` / `firstElementChild` / `lastElementChild` / `childElementCount` reflection helpers, `window.name`, `window.self`, `window.window`, `window.parent`, `window.top`, `window.closed`, `window.history.length` / `window.history.state` / `window.history.scrollRestoration` / `window.history.pushState()` / `window.history.replaceState()` / `window.history.back()` / `window.history.forward()` / `window.history.go()`, `window.origin`, `element.origin`, and script-side `window.location.assign()`, `window.location.replace()`, `window.location.reload()` routed through the location mock, plus script-side `Element.click()`, `Element.focus()`, and `Element.blur()` routed through the same event/default-action seam, plus `window.navigator.clipboard.writeText()` / `window.navigator.clipboard.readText()` routed through the clipboard mock), `window.navigator` metadata (`userAgent`, `appCodeName`, `appName`, `appVersion`, `product`, `productSub`, `vendor`, `vendorSub`, `pdfViewerEnabled`, `doNotTrack`, `javaEnabled()`, `plugins`, `mimeTypes`, `platform`, `language`, `userLanguage`, `browserLanguage`, `systemLanguage`, `oscpu`, `languages`, `cookieEnabled`, `onLine`, `webdriver`, `hardwareConcurrency`, `maxTouchPoints`), `window.devicePixelRatio`, `window.innerWidth` / `window.innerHeight`, `window.outerWidth` / `window.outerHeight`, `window.screenX` / `window.screenY` / `window.screenLeft` / `window.screenTop` / `window.screen` (including `availWidth`, `availHeight`, `availLeft`, `availTop`, `colorDepth`, `pixelDepth`, `orientation.type`, `orientation.angle`), `window.localStorage` / `window.sessionStorage` named property access, `window.matchMedia` (with `MediaQueryList.addListener()` / `removeListener()` listener hooks and registry-backed listener call capture), `window.open()`, `window.close()`, `window.print()`, `window.scrollTo()` / `window.scrollBy()` and scroll position aliases (`window.scrollX`, `window.scrollY`, `window.pageXOffset`, `window.pageYOffset`), `document.currentScript` / `document.readyState` / `document.compatMode` / `document.characterSet` / `document.charset` / `document.contentType` / `document.visibilityState` / `document.hidden` / `document.dir` / `document.activeElement` / `document.scrollingElement` / `document.hasFocus()` during inline script bootstrap and focus tracking, `window.children` as an alias of `document.children`, and `window.frames` / `window.length` / `window.frameElement` / `window.opener` as live `HTMLCollection` frame-count / frame-element surfaces over descendant `iframe` / `frame` elements, with `form.length` and `select.length` as read-only aliases to their live collection sizes, file input, and download capture are implemented; `window.Node`, `window.Element`, `window.HTMLElement`, and a bounded set of HTML element constructor globals, including `window.HTMLButtonElement`, `window.HTMLSelectElement`, `window.HTMLInputElement`, `window.HTMLTextAreaElement`, `window.HTMLFormElement`, `window.HTMLOptionElement`, `window.HTMLOptGroupElement`, `window.HTMLFieldSetElement`, `window.HTMLLabelElement`, `window.HTMLImageElement`, `window.HTMLAnchorElement`, `window.HTMLAreaElement`, `window.HTMLMapElement`, `window.HTMLTableElement`, `window.HTMLTableSectionElement`, `window.HTMLTableRowElement`, `window.HTMLTableCellElement`, `window.HTMLUListElement`, `window.HTMLOListElement`, `window.HTMLLIElement`, `window.HTMLObjectElement`, `window.HTMLEmbedElement`, `window.HTMLLegendElement`, `window.HTMLDListElement`, `window.HTMLScriptElement`, and `window.HTMLStyleElement`, are also exposed as constructor globals for `instanceof` checks
- Deterministic test controls are available on `Harness` for `Math.random()` seeding (`set_random_seed`), trace capture (`enable_trace`, `set_trace_stderr`, `set_trace_events`, `set_trace_timers`, `set_trace_log_limit`, `take_trace_logs`), and bounded timer stepping (`set_timer_step_limit`), so regression tests can pin runtime output without reaching for private internals.
- `Element.attributes` is exposed as a live `NamedNodeMap`, and `document.createAttribute()` / `document.createAttributeNS()` plus `getAttributeNode()` / `getAttributeNodeNS()` / `setAttributeNode()` / `setAttributeNodeNS()` / `removeAttributeNode()` / `NamedNodeMap.setNamedItem()` / `NamedNodeMap.setNamedItemNS()` / `NamedNodeMap.removeNamedItem()` / `NamedNodeMap.removeNamedItemNS()` / `NamedNodeMap.keys()` / `NamedNodeMap.values()` / `NamedNodeMap.entries()` / `NamedNodeMap.forEach()` expose detached `Attr` nodes through the same attribute store; `Attr.specified` and `Attr.isId` are exposed as read-only booleans on those attribute nodes.
- `window.navigator.languages`, `window.navigator.mimeTypes`, and `window.navigator.plugins` also expose `keys()`, `values()`, `entries()`, and `forEach()` iterator helpers through the same bounded collection seam; `window.navigator.plugins.refresh()` is exposed as a deterministic no-op refresh hook.
- The live `HTMLCollection` surfaces used for `document.children`, `window.children`, `window.frames`, `document.images`, `document.links`, `document.embeds`, `document.plugins`, `document.anchors`, `document.applets`, `document.scripts`, and `document.all` also expose `keys()`, `values()`, `entries()`, and `forEach()` iterator helpers.
- Phase 4 reflection also covers `element.spellcheck` and `element.inputMode` as reflected element properties on the same attribute store as `getAttribute` / `setAttribute`.
- `element.id` and `element.name` are exposed as reflected, read/write element properties that route through the same attribute store as `getAttribute` / `setAttribute`.
- `element.accessKey`, `element.slot`, `element.autocapitalize`, `element.spellcheck`, `element.inputMode`, `element.translate`, `element.dir`, `element.lang`, `element.title`, `element.role`, `element.ariaLabel`, `element.ariaDescription`, `element.ariaRoleDescription`, `element.ariaHidden`, `element.tabIndex`, and `element.hidden` are exposed as reflected, read/write element properties that route through the same attribute store as `getAttribute` / `setAttribute`, and `element.translate` inherits the nearest `translate` state from ancestors while `element.dir` / `element.lang` feed `:dir()` / `:lang()` selector matching through the same element attribute store.
- `input.indeterminate` is exposed as the reflected, read/write checkbox-only indeterminate flag on `<input type="checkbox">` and stays in sync with `:indeterminate`; checkbox activation clears the state.
- `input.defaultChecked` is exposed as the reflected, read/write checkbox/radio default-checked flag on `<input type="checkbox">` / `<input type="radio">` and stays in sync with `:default`.
- `input.accept` and `input.multiple` are exposed as reflected, read/write file-input configuration on `<input>` elements.
- `option.selected` is exposed as a reflected, read/write option property that keeps `select.selectedOptions` and `:checked` selectors in sync through the same attribute store.
- `option.defaultSelected` is exposed as the reflected, read/write option-state alias for the same selected attribute.
- `option.disabled` is exposed as the reflected, read/write option-state alias for the same disabled attribute, and `input.disabled` / `textarea.disabled` / `button.disabled` / `select.disabled` / `fieldset.disabled` expose the same reflected boolean disabled flag on common form controls.
- `option.label` is exposed as the reflected, read/write option label, using the `label` attribute when present and the option's text content as the fallback.
- `option.text` is exposed as the reflected, read/write option text content alias.
- `optgroup.disabled` / `optgroup.label` are exposed as the reflected, read/write optgroup disabled and label attributes, and `optgroup.disabled` stays in sync with `:disabled`.
- `select.multiple` is exposed as the reflected, read/write boolean option-group flag on `<select>` elements.
- `select.type` is exposed as the read-only select-kind string (`select-one` / `select-multiple`) on `<select>` elements.
- `input.type` and `button.type` are exposed as the reflected type strings on their respective form controls, using browser-style defaults when the `type` attribute is missing or invalid.
- `select.required` is exposed as the reflected, read/write boolean required flag on `input`, `textarea`, and `select` form controls, `fieldset.disabled` is exposed as the reflected, read/write boolean disabled flag on `fieldset`, `form.noValidate` / `input.formNoValidate` / `button.formNoValidate` are exposed as reflected boolean validation-suppression flags, and `form.action` / `form.method` / `form.enctype` / `form.target` plus `input.formAction` / `button.formAction` / `input.formMethod` / `button.formMethod` / `input.formEnctype` / `button.formEnctype` / `input.formTarget` / `button.formTarget` expose reflected form submission metadata resolved against `document.baseURI`.
- `input.readOnly` and `textarea.readOnly` are exposed as the reflected, read/write boolean readonly flag on text controls.
- `input.autocomplete` and `textarea.autocomplete` are exposed as the reflected, read/write autocomplete string on text controls.
- `input.defaultValue` and `textarea.defaultValue` are exposed as the reflected, read/write default-value surfaces on text controls; `input.defaultValue` maps to the `value` attribute and `textarea.defaultValue` maps to the textarea text content.
- `input.minLength` and `textarea.minLength`, plus `input.maxLength` and `textarea.maxLength`, are exposed as the reflected length constraints on text controls and keep the bounded `:valid` / `:invalid` selectors in sync.
- `input.min` and `input.max` are exposed as reflected string bounds on `<input>` elements and keep the bounded `:in-range` / `:out-of-range` selectors in sync for number inputs; `input.step` remains a plain reflected step string, `input.size` is exposed as the reflected non-negative size attribute on `<input>` elements and defaults to `20` when absent, `textarea.rows` / `textarea.cols` are exposed as reflected non-negative dimensions on `<textarea>` elements and default to `2` / `20` when absent, and `textarea.wrap` is exposed as the reflected textarea wrap mode and defaults to `soft` when absent.
- `input.pattern` is exposed as the reflected validation pattern on text inputs and keeps the bounded `:invalid` selector in sync when the current value does not match.
- `input.placeholder` and `textarea.placeholder` are exposed as the reflected, read/write placeholder string on text controls.
- `input.autofocus`, `textarea.autofocus`, `button.autofocus`, and `select.autofocus` are exposed as the reflected, read/write boolean autofocus flag on common interactive controls.
- `select.size` is exposed as the reflected, read/write non-negative size attribute on `<select>` elements, defaulting to `0` when absent.
- `select.value` is exposed as the reflected, read/write current selected option value for the same selected state, and unmatched assignments clear the selection.
- `select.selectedIndex` is exposed as the reflected, read/write active-option index for the same selected state.
- `option.index` is exposed as a read-only option position property that updates when the owning select's option order changes.
- `input.form` / `button.form` / `select.form` / `textarea.form` / `option.form` / `fieldset.form` / `output.form` / `object.form` / `embed.form` are exposed as read-only owner properties that update when the control is moved out of its form.
- The location mock also exposes `document.location.protocol` / `window.location.protocol`, `document.location.host` / `window.location.host`, `document.location.hostname` / `window.location.hostname`, `document.location.port` / `window.location.port`, `document.location.username` / `window.location.username`, and `document.location.password` / `window.location.password` as deterministic aliases on the same runtime seam.
- Phase 4 also exposes `Node.isConnected` / `Element.isConnected` / `Document.isConnected` and `Node.contains()` / `Element.contains()` / `Document.contains()` / `Node.hasChildNodes()` / `Element.hasChildNodes()` / `Document.hasChildNodes()` / `Node.nextSibling` / `Element.nextSibling` / `Document.nextSibling` / `Node.previousSibling` / `Element.previousSibling` / `Document.previousSibling` / `Node.nextElementSibling` / `Element.nextElementSibling` / `Node.previousElementSibling` / `Element.previousElementSibling` / `Node.firstChild` / `Element.firstChild` / `Document.firstChild` / `Node.lastChild` / `Element.lastChild` / `Document.lastChild` / `Node.compareDocumentPosition()` / `Element.compareDocumentPosition()` / `Document.compareDocumentPosition()` / `Node.isSameNode()` / `Element.isSameNode()` / `Document.isSameNode()` / `Node.isEqualNode()` / `Element.isEqualNode()` / `Document.isEqualNode()` as read-only tree-connectivity, containment, child-presence, tree-order, and node-equality reflection helpers, with detached `template.content` remaining disconnected.
- The same location mock also supports `document.location.toString()` / `window.location.toString()` and `document.location.valueOf()` / `window.location.valueOf()` stringification helpers for the current URL.
- Phase 5 hardening adds contract tests, subsystem tests, regression coverage, property tests, and a publication checklist
- Phase 7 script DOM query slices 1 through 4, selector lists, bounded attribute selectors (`[attr=value]`, `[attr^=value]`, `[attr$=value]`, `[attr*=value]`, `[attr~=value]`, `[attr|=value]`) plus optional `i` / `s` attribute selector flags, bounded pseudo-classes including `:not(...)`, `:is(...)`, `:where(...)`, `:scope`, `:has(...)`, `:lang(...)`, `:dir(...)`, `:link`, `:any-link`, `:placeholder-shown`, `:blank`, `:required`, `:optional`, `:focus`, `:focus-visible`, `:focus-within`, `:target`, `:defined`, `:root`, `:empty`, `:only-child`, `:only-of-type`, `:first-of-type`, `:last-of-type`, `:nth-child(<positive integer>)`, `:nth-child(odd)`, `:nth-child(even)`, `:nth-child(an+b)`, `:nth-child(... of <selector-list>)`, `:nth-last-child(<positive integer>)`, `:nth-last-child(odd)`, `:nth-last-child(even)`, `:nth-last-child(an+b)`, `:nth-last-child(... of <selector-list>)`, `:nth-of-type(<positive integer>)`, `:nth-of-type(odd)`, `:nth-of-type(even)`, `:nth-of-type(an+b)`, `:nth-of-type(... of <selector-list>)`, `:nth-last-of-type(<positive integer>)`, `:nth-last-of-type(odd)`, `:nth-last-of-type(even)`, `:nth-last-of-type(an+b)`, `:nth-last-of-type(... of <selector-list>)`, `:checked`, `:disabled`, `:enabled`, `:default`, `:valid`, `:invalid`, `:in-range`, `:out-of-range`, `:read-only`, `:read-write`, and `:indeterminate` are implemented; bounded selector lists and combinators are available inside `:not(...)` and `:is(...)`, while broader CSS parsing and malformed or unknown attribute selector flags outside this bounded slice still fail explicitly; `querySelectorAll` with minimal `NodeList` support, `Element.children` with minimal `HTMLCollection` support, `getElementsByTagName(...)` with live `HTMLCollection` support, `getElementsByTagNameNS(...)` with live `HTMLCollection` support, `getElementsByClassName(...)` with live `HTMLCollection` support, `getElementsByName(...)` with live `NodeList` support, `document.forms` / `form.elements` live `HTMLCollection` support, including `form.elements.namedItem()` returning `RadioNodeList` when multiple matching controls share a name, `select.options` / `select.selectedOptions` live `HTMLCollection` support, including `select.options.add()` / `select.options.remove()` mutation helpers, `fieldset.elements` / `datalist.options` live `HTMLCollection` support, `map.areas` / `table.tBodies` live `HTMLCollection` support, `document.documentElement` / `document.head` / `document.body` / `document.title` / `window.title` / `document.location` / `window.location` / `document.location.href` / `window.location.href` / `document.location.hash` / `window.location.hash` / `document.URL` / `document.documentURI`, `document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.anchors` / `document.applets` live `HTMLCollection` support, `document.scripts` live `HTMLCollection` support, `document.styleSheets` live `StyleSheetList` support including `keys()`, `values()`, `entries()`, `namedItem()`, and `forEach()`, `document.all` live `HTMLCollection` support, and `element.labels` on labelable form controls / fieldset live `NodeList` support are implemented, including `namedItem()` on HTMLCollection; `NodeList.forEach` / `HTMLCollection.forEach` and `NodeList.keys()` / `NodeList.values()` / `NodeList.entries()` / `HTMLCollection.keys()` / `HTMLCollection.values()` / `HTMLCollection.entries()` are implemented too
- `document.childNodes` / `Node.childNodes`, `template.content` childNodes / children / `textContent` / `getElementById()` / `querySelector(All)` / `cloneNode()` / direct child mutation (`appendChild()`, `insertBefore()`, `replaceChild()`, `replaceChildren()`, `removeChild()`, `replaceWith()`, `append()`, `prepend()`), `document.styleSheets`, `document.children`, `window.children`, `table.rows` / `tr.cells` (with iterator helpers), `select.options`, `select.selectedOptions`, `fieldset.elements`, `datalist.options`, `map.areas`, `table.tBodies` (with iterator helpers), and `element.labels` on labelable form controls / fieldset with live `NodeList` support are implemented as additional specialized live collection slices; `form.elements.namedItem()` can return `RadioNodeList` for multi-match groups, legacy named property access on live `HTMLCollection` surfaces resolves through the same `namedItem()` semantics, `RadioNodeList` now exposes `keys()`, `values()`, `entries()`, and `forEach()` alongside `item()`, `length`, and `value`, and `RadioNodeList.value` assignment updates the checked radio group state, while `document.styleSheets` exposes `keys()`, `values()`, `entries()`, `namedItem()`, `forEach()`, `length`, and `item()`
- Phase 8 slice 1 attribute reflection is implemented, covering `getAttribute`, `setAttribute`, `removeAttribute`, `hasAttribute`, and `toggleAttribute` with reflected ID, class, name, accessKey, slot, autocapitalize, spellcheck, inputMode, translate, checked, disabled, selected, required, read-only, autocomplete, placeholder, defaultValue, autofocus, form validation, value, step, size, and range-bounds state
- Phase 8 slice 2 class and dataset views are implemented, covering `className`, `classList` (including `value`, `toString()`, `replace()`, and iterator helpers), and `dataset`
- Phase 8 slice 3 tree mutation primitives are implemented, covering `append`, `prepend`, `before`, `after`, `remove`, `removeChild`, `normalize`, `appendChild`, `insertBefore`, `replaceChild`, `replaceChildren`, `replaceWith`, and `cloneNode`
- Phase 8 slice 4 HTML serialization / insertion surfaces are implemented, covering `innerHTML`, `outerHTML`, `insertAdjacentHTML()`, `insertAdjacentElement()`, `insertAdjacentText()`, `document.open()`, `document.write()`, `document.writeln()`, and `document.close()` with bounded fragment parsing and serialization; `document.open()` / `document.close()` return `document`
- HTML serialization broadening slice 1 `insertAdjacentHTML`, slice 2 `insertAdjacentElement()` / `insertAdjacentText()`, slice 3 `template.content.innerHTML`, slice 4 namespace-aware serialization compatibility, and slice 5 `document.open()` / `document.write()` / `document.writeln()` / `document.close()` are implemented with bounded fragment insertion / serialization on `<template>` content, adjusted SVG / MathML name handling, browser-style escaping for mixed-quote attribute values, basic character reference decoding during fragment parse, including common named references such as `&nbsp;`, `&copy;`, and `&reg;` plus safe semicolonless forms like `&nbsp` / `&amp` / `&lt` / `&gt` / `&copy` / `&reg`, legacy uppercase variants like `&AMP` / `&LT` / `&GT` / `&QUOT` / `&NBSP` / `&COPY` / `&REG`, and semicolonless numeric forms like `&#160` / `&#xA0`, append-style document write helpers that append to the open document tree, and chainable `document.open()` / `document.close()` returns
- Phase 8 slice 5 mutation hardening and regression coverage are implemented, covering selector and collection consistency after mutation plus explicit failures for unsupported mutation semantics
- The script host surface also includes `document.createElement()`, `document.createElementNS()`, `document.createTextNode()`, `document.createComment()`, and `document.createDocumentFragment()` for detached HTML / namespace-aware element / text node / comment / fragment construction, plus `before()`, `after()`, `cloneNode()`, `remove()`, `removeChild()`, `normalize()`, `replaceWith()`, and `importNode()` on existing nodes for detached cloning and replacement.
- backlog-driven work now focuses on additional specialized live collections beyond `document.childNodes`, `template.content`, `document.styleSheets`, `document.children`, `table.rows` / `tr.cells`, `select.selectedOptions`, `fieldset.elements`, `datalist.options`, `map.areas`, `table.tBodies`, and `element.labels`, while broader CSS parsing beyond the bounded selector grammar remains deferred until needed by a specific user-visible gap

Workspace layout:

```text
crates/
  browser-tester/   # public facade crate (`browser_tester`)
  bt-dom/           # DOM store, HTML parser, selector subset
  bt-runtime/       # session, scheduler, mocks, debug state
  bt-script/        # script runtime and host-binding seam
doc/
  architecture.md
  capability-matrix.md
  implementation-guide.md
  mock-guide.md
  limitations.md
  subsystem-map.md
  roadmap.md
  adr/
```

Quick start:

```bash
cargo test
```

Minimal Phase 3 example:

```rust
use browser_tester::Harness;

fn main() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<form id='profile'><input id='name'><input id='agree' type='checkbox'><button id='submit' type='submit'>Save</button></form><div id='out'></div><script>document.getElementById('profile').addEventListener('submit', () => { document.getElementById('out').textContent = document.getElementById('name').value + ':' + String(document.getElementById('agree').checked); });</script>",
    )?;

    harness.type_text("#name", "Alice")?;
    harness.click("#agree")?;
    harness.click("#submit")?;
    harness.assert_checked("#agree", true)?;
    harness.assert_text("#out", "Alice:true")?;
    Ok(())
}
```

Minimal Phase 4 mock example:

```rust
use browser_tester::Harness;

fn main() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html("<input id='upload' type='file'>")?;

    harness
        .mocks_mut()
        .fetch()
        .respond_text("https://app.local/api/message", 200, "ok");
    harness.mocks_mut().dialogs().push_confirm(true);
    harness.mocks_mut().clipboard().seed_text("seeded");

    let response = harness.fetch("https://app.local/api/message")?;
    assert_eq!(response.body, "ok");
    assert!(harness.confirm("Continue?")?);
    assert_eq!(harness.read_clipboard()?, "seeded");

    harness.set_files("#upload", ["report.csv"])?;
    harness.capture_download("report.csv", b"downloaded bytes".to_vec())?;
    harness.scroll_to(0, 120)?;
    harness.close()?;
    harness.navigate("https://app.local/next")?;
    {
        let downloads = harness.mocks_mut().downloads();
        assert_eq!(downloads.artifacts().len(), 1);
        assert_eq!(downloads.artifacts()[0].file_name, "report.csv");
        assert_eq!(downloads.artifacts()[0].bytes, b"downloaded bytes".to_vec());
    }
    assert_eq!(harness.mocks_mut().close().calls().len(), 1);
    assert_eq!(harness.mocks_mut().scroll().calls().len(), 1);
    Ok(())
}
```

Design docs:

- [Architecture](doc/architecture.md)
- [Capability Matrix](doc/capability-matrix.md)
- [Implementation Guide](doc/implementation-guide.md)
- [Mock Guide](doc/mock-guide.md)
- [Publication Checklist](doc/publish-checklist.md)
- [Limitations](doc/limitations.md)
- [Subsystem Map](doc/subsystem-map.md)
- [Roadmap](doc/roadmap.md)
