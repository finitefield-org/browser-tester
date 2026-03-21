# browser-tester next

This directory is a clean-room rewrite workspace for `browser-tester`.

It is intentionally organized around the staged plan from [`next.md`](../next.md):

- split the runtime into explicit subsystems
- keep `Harness` as a thin public facade
- treat deterministic mocks as first-class APIs
- keep the initial surface small and documented

Current status:

- a compilable Rust workspace exists under `next/crates/`
- `HarnessBuilder`, `Session`, `DomStore`, scheduler, mock registry, and error taxonomy are in place
- Phase 1 DOM parsing, selector subset support, Phase 6 selector expansion slices 1 through 4, and sibling selectors (`A + B`, `A ~ B`) are implemented
- Phase 2 inline script bootstrapping, `document.getElementById(...).textContent = ...`, and listener registration are implemented
- Phase 3 event dispatch, ancestor bubbling, cancelable click default actions, form controls, `click`, `type_text`, `set_checked`, `set_select_value`, `focus`, `blur`, `submit`, `dispatch`, `assert_value`, and `assert_checked` are implemented
- Phase 4 fake clock hardening, microtask semantics, and runtime mock wiring for fetch, dialogs, clipboard, location aliases (`document.location`, `window.location`, `document.URL`, `document.documentURI`, `document.baseURI`, `element.baseURI`, `document.origin`, `document.referrer`, `window.name`, `window.origin`, `element.origin`), `window.navigator` metadata (`userAgent`, `platform`, `language`, `cookieEnabled`, `onLine`), `window.devicePixelRatio`, `window.localStorage` / `window.sessionStorage`, `window.matchMedia`, `window.open()`, `window.close()`, `window.print()`, `window.scrollTo()` / `window.scrollBy()` and scroll position aliases (`window.scrollX`, `window.scrollY`, `window.pageXOffset`, `window.pageYOffset`), `document.currentScript` / `document.readyState` / `document.compatMode` / `document.characterSet` / `document.charset` / `document.contentType` / `document.visibilityState` / `document.hidden` / `document.dir` / `document.activeElement` / `document.hasFocus()` during inline script bootstrap and focus tracking, `window.children` as an alias of `document.children`, file input, and download capture are implemented
- Phase 5 hardening adds contract tests, subsystem tests, regression coverage, property tests, and a publication checklist
- Phase 7 script DOM query slices 1 through 4, selector lists, bounded attribute selectors (`[attr=value]`, `[attr^=value]`, `[attr$=value]`, `[attr*=value]`, `[attr~=value]`, `[attr|=value]`) plus optional `i` / `s` attribute selector flags, bounded pseudo-classes including `:not(...)`, `:is(...)`, `:scope`, `:has(...)`, `:lang(...)`, `:dir(...)`, `:link`, `:any-link`, `:placeholder-shown`, `:required`, `:optional`, `:focus`, `:focus-within`, `:target`, `:defined`, `:root`, `:empty`, `:only-child`, `:only-of-type`, `:first-of-type`, `:last-of-type`, `:nth-child(<positive integer>)`, `:nth-child(odd)`, `:nth-child(even)`, `:nth-child(an+b)`, `:nth-child(... of <selector-list>)`, `:nth-last-child(<positive integer>)`, `:nth-last-child(odd)`, `:nth-last-child(even)`, `:nth-last-child(an+b)`, `:nth-last-child(... of <selector-list>)`, `:nth-of-type(<positive integer>)`, `:nth-of-type(odd)`, `:nth-of-type(even)`, `:nth-of-type(an+b)`, `:nth-of-type(... of <selector-list>)`, `:nth-last-of-type(<positive integer>)`, `:nth-last-of-type(odd)`, `:nth-last-of-type(even)`, `:nth-last-of-type(an+b)`, `:nth-last-of-type(... of <selector-list>)`, `:checked`, `:disabled`, `:enabled`, `:default`, `:valid`, `:invalid`, `:in-range`, `:out-of-range`, `:read-only`, `:read-write`, and `:indeterminate` are implemented; bounded selector lists and combinators are available inside `:not(...)` and `:is(...)`, while broader CSS parsing and malformed or unknown attribute selector flags outside this bounded slice still fail explicitly; `querySelectorAll` with minimal `NodeList` support, `Element.children` with minimal `HTMLCollection` support, `getElementsByTagName(...)` with live `HTMLCollection` support, `getElementsByTagNameNS(...)` with live `HTMLCollection` support, `getElementsByClassName(...)` with live `HTMLCollection` support, `getElementsByName(...)` with live `NodeList` support, `document.forms` / `form.elements` live `HTMLCollection` support, including `form.elements.namedItem()` returning `RadioNodeList` when multiple matching controls share a name, `select.options` / `select.selectedOptions` live `HTMLCollection` support, including `select.options.add()` / `select.options.remove()` mutation helpers, `fieldset.elements` / `datalist.options` live `HTMLCollection` support, `map.areas` / `table.tBodies` live `HTMLCollection` support, `document.documentElement` / `document.head` / `document.body` / `document.title` / `window.title` / `document.location` / `window.location` / `document.URL` / `document.documentURI`, `document.images` / `document.links` / `document.embeds` / `document.plugins` / `document.anchors` / `document.applets` live `HTMLCollection` support, `document.scripts` live `HTMLCollection` support, `document.styleSheets` live `StyleSheetList` support, `document.all` live `HTMLCollection` support, and `element.labels` on labelable form controls / fieldset live `NodeList` support are implemented, including `namedItem()` on HTMLCollection; `NodeList.forEach` / `HTMLCollection.forEach` and `NodeList.keys()` / `NodeList.values()` / `NodeList.entries()` / `HTMLCollection.keys()` / `HTMLCollection.values()` / `HTMLCollection.entries()` are implemented too
- `document.childNodes` / `Node.childNodes`, `template.content` childNodes / children, `document.styleSheets`, `document.children`, `table.rows` / `tr.cells`, `select.selectedOptions`, `fieldset.elements`, `datalist.options`, `map.areas`, `table.tBodies`, and `element.labels` on labelable form controls / fieldset with live `NodeList` support are implemented as additional specialized live collection slices; `form.elements.namedItem()` can return `RadioNodeList` for multi-match groups, `RadioNodeList` now exposes `entries()` alongside `forEach()`, `keys()`, `values()`, `item()`, `length`, and `value`, and `RadioNodeList.value` assignment updates the checked radio group state, while `document.styleSheets` exposes `keys()`, `values()`, `entries()`, `namedItem()`, `length`, and `item()`
- Phase 8 slice 1 attribute reflection is implemented, covering `getAttribute`, `setAttribute`, `removeAttribute`, `hasAttribute`, and `toggleAttribute` with reflected ID, class, name, checked, disabled, selected, and value state
- Phase 8 slice 2 class and dataset views are implemented, covering `className`, `classList`, and `dataset`
- Phase 8 slice 3 tree mutation primitives are implemented, covering `append`, `prepend`, `before`, `after`, `remove`, `appendChild`, `insertBefore`, `replaceChild`, and `replaceChildren`
- Phase 8 slice 4 HTML serialization surfaces are implemented, covering `innerHTML` and `outerHTML` with bounded fragment parsing and serialization
- HTML serialization broadening slice 1 `insertAdjacentHTML`, slice 2 `template.content.innerHTML`, and slice 3 namespace-aware serialization compatibility are implemented with bounded fragment insertion / serialization on `<template>` content and adjusted SVG / MathML name handling
- Phase 8 slice 5 mutation hardening and regression coverage are implemented, covering selector and collection consistency after mutation plus explicit failures for lossy mutation semantics
- backlog-driven work now focuses on additional specialized live collections beyond `document.childNodes`, `template.content`, `document.styleSheets`, `document.children`, `table.rows` / `tr.cells`, `select.selectedOptions`, `fieldset.elements`, `datalist.options`, `map.areas`, `table.tBodies`, and `element.labels`, while broader CSS parsing beyond the bounded selector grammar remains deferred until needed by a specific user-visible gap

Workspace layout:

```text
next/
  crates/
    browser-tester/   # public facade crate (`browser_tester_next`)
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
cd next
cargo test
```

Minimal Phase 3 example:

```rust
use browser_tester_next::Harness;

fn main() -> browser_tester_next::Result<()> {
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
use browser_tester_next::Harness;

fn main() -> browser_tester_next::Result<()> {
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
