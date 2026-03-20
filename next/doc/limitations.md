# Limitations

The `next/` workspace has reached Phase 4 for DOM, script, events, forms, fake time, and deterministic mock wiring, Phase 5 for hardening and publication checks, Phase 6 selector slices 1 through 4 plus sibling combinators, and Phase 7 script DOM query slices 1 through 4. Phase 8 DOM mutation and reflection expansion is now complete in this workspace, and its five slices, attribute reflection, class / dataset views, tree mutation primitives, bounded HTML serialization surfaces, and mutation hardening / regression coverage, are implemented. Post-Phase-7 collection slices add `querySelectorAll` with minimal `NodeList` support, `Element.children` with minimal `HTMLCollection` support, including `namedItem()`, `getElementsByTagName` with live `HTMLCollection` support, `getElementsByTagNameNS` with live `HTMLCollection` support, `getElementsByClassName` with live `HTMLCollection` support, `getElementsByName` with live `NodeList` support, `document.forms` / `form.elements` with live `HTMLCollection` support, `select.options` / `select.selectedOptions` with live `HTMLCollection` support, `fieldset.elements` / `datalist.options` with live `HTMLCollection` support, `map.areas` / `table.tBodies` with live `HTMLCollection` support, `document.childNodes` / `document.children` / `document.images` / `document.links` / `document.embeds` / `document.anchors` / `document.applets` / `document.scripts` / `document.styleSheets` / `document.all` with live `HTMLCollection` / `NodeList` support, and `element.labels` on labelable form controls / fieldset with live `NodeList` support. Selector lists, bounded attribute selectors (`[attr=value]`, `[attr^=value]`, `[attr$=value]`, `[attr*=value]`, `[attr~=value]`, `[attr|=value]`) plus optional `i` / `s` flags, and a bounded pseudo-class slice, including `:not(...)`, `:is(...)`, `:root`, `:empty`, `:only-child`, `:only-of-type`, `:first-of-type`, `:last-of-type`, `:nth-child(<positive integer>)`, `:nth-child(odd)`, `:nth-child(even)`, `:nth-child(an+b)`, `:nth-last-child(<positive integer>)`, `:nth-last-child(odd)`, `:nth-last-child(even)`, `:nth-last-child(an+b)`, `:nth-of-type(<positive integer>)`, `:nth-of-type(odd)`, `:nth-of-type(even)`, `:nth-of-type(an+b)`, `:nth-last-of-type(<positive integer>)`, `:nth-last-of-type(odd)`, `:nth-last-of-type(even)`, and `:nth-last-of-type(an+b)`, are supported by the bounded selector engine.

What exists today:

- a compilable crate graph
- public and internal type skeletons
- deterministic session bootstrap
- real HTML parsing and DOM tree construction for the Phase 1 subset
- selector matching for `#id`, tag, `[attr]`, `[attr=value]`, `[attr^=value]`, `[attr$=value]`, `[attr*=value]`, `[attr~=value]`, `[attr|=value]`, optional `i` / `s` flags, `.class`, `tag.class`, `#id.class`, selector lists, bounded pseudo-classes, descendant combinators, child combinators, adjacent sibling combinators, and general sibling combinators
- `assert_exists` and DOM dump helpers
- inline `<script>` bootstrapping with a minimal script parser/evaluator
- listener registration through the script host seam
- target-phase event dispatch and form-control state updates
- `click`, `type_text`, `set_checked`, `submit`, `dispatch`, `assert_value`, and `assert_checked`
- explicit mock families, debug hooks, and public mock actions for `fetch`, dialogs, clipboard, location, file input, and download capture
- contract, regression, and property test suites
- quick and hardening test profiles plus a publication checklist
- script-side `document.querySelector` and `element.querySelector` are available
- script-side `querySelectorAll` is available with minimal `NodeList` support (`length`, `item`)
- script-side `document.childNodes` is available with live `NodeList` support (`length`, `item`)
- script-side `Element.children` is available with minimal `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `document.children` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `table.rows` and `tr.cells` are available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `getElementsByTagName` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `getElementsByTagNameNS` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`) for the bounded HTML, SVG, and MathML namespace set
- script-side `getElementsByClassName` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `getElementsByName` is available with live `NodeList` support (`length`, `item`)
- script-side `document.forms` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `form.elements` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`); `namedItem()` can return `RadioNodeList` when multiple controls share the same name
- script-side `fieldset.elements` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `select.options` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `select.selectedOptions` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `datalist.options` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `map.areas` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `table.tBodies` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `element.labels` is available on labelable form controls / fieldset with live `NodeList` support (`length`, `item`)
- script-side `document.images` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `document.links` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `document.embeds` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `document.anchors` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `document.all` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `document.styleSheets` is available with live `StyleSheetList` support (`length`, `item`)
- script-side `className`, `classList`, and `dataset` are available on `Element`
- script-side tree mutation primitives are available on `Element`
- script-side `innerHTML` and `outerHTML` are available on `Element` with bounded HTML serialization and fragment parsing
- script-side `Element.matches` is available
- script-side `Element.closest` is available
- script-side `getAttribute`, `setAttribute`, `removeAttribute`, `hasAttribute`, and `toggleAttribute` are available on `Element`
- selector hardening and regression coverage are available

What does not exist yet:

- broader collection APIs beyond the current bounded DOM collection set, including further specialized live collections beyond `document.childNodes`, `document.styleSheets`, `document.children`, `table.rows` / `tr.cells`, `select.selectedOptions`, `fieldset.elements`, `datalist.options`, `map.areas`, `table.tBodies`, and `element.labels`; backlog slice: collection API broadening
- broader browser-compatible HTML serialization beyond the bounded `innerHTML` and `outerHTML` surfaces; backlog slice: HTML serialization broadening
- broader CSS parsing beyond the bounded selector grammar, including malformed or unknown attribute selector flags such as `[attr=value x]`; backlog slice: selector grammar broadening

## Important Consequence

Public methods such as `focus`, `blur`, `set_select_value`, `fetch`, `confirm`, `prompt`, `read_clipboard`, `write_clipboard`, `navigate`, `set_files`, and `capture_download` are now supported and dispatch or capture through the same deterministic runtime.

## Still Out of Scope

Even after later phases, this rewrite does not aim to provide:

- real rendering
- CSS layout
- arbitrary browser compatibility
- real network access
- service worker support
- exhaustive Web API coverage
