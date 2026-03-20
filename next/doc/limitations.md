# Limitations

The `next/` workspace has reached Phase 4 for DOM, script, events, forms, fake time, and deterministic mock wiring, Phase 5 for hardening and publication checks, Phase 6 selector slices 1 through 4 plus sibling combinators, and Phase 7 script DOM query slices 1 through 4. Post-Phase-7 collection slices add `querySelectorAll` with minimal `NodeList` support, `Element.children` with minimal `HTMLCollection` support, including `namedItem()`, `getElementsByTagName` with live `HTMLCollection` support, `getElementsByClassName` with live `HTMLCollection` support, `getElementsByName` with live `NodeList` support, `document.forms` / `form.elements` with live `HTMLCollection` support, `select.options` with live `HTMLCollection` support, and `document.images` / `document.links` with live `HTMLCollection` support. Selector lists and a small pseudo-class slice are supported by the bounded selector engine; broader collection APIs such as `document.all` and namespace-aware `getElementsByTagNameNS` remain deferred by design.

What exists today:

- a compilable crate graph
- public and internal type skeletons
- deterministic session bootstrap
- real HTML parsing and DOM tree construction for the Phase 1 subset
- selector matching for `#id`, tag, `[attr]`, `.class`, `tag.class`, `#id.class`, selector lists, simple pseudo-classes, descendant combinators, child combinators, adjacent sibling combinators, and general sibling combinators
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
- script-side `Element.children` is available with minimal `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `getElementsByTagName` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `getElementsByClassName` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `getElementsByName` is available with live `NodeList` support (`length`, `item`)
- script-side `document.forms` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `form.elements` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `select.options` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `document.images` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `document.links` is available with live `HTMLCollection` support (`length`, `item`, `namedItem`)
- script-side `Element.matches` is available
- script-side `Element.closest` is available
- selector hardening and regression coverage are available

What does not exist yet:

- broader collection APIs such as `document.all` and namespace-aware `getElementsByTagNameNS`
- broad pseudo-classes such as `:nth-child`, `:not`, and `:is`

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
