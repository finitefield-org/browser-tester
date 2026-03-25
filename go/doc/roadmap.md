# Roadmap

This is the recommended build order for the Go workspace.
The phases are intentionally sequential at the start so the public facade stays small and the implementation stays testable.

## Phase 0: Scaffold

- Create the module, package root, internal packages, and CI-friendly tests.
- Define the public facade, error taxonomy, and explicit builder config.
- Land `Subsystem Map`, `Capability Matrix`, `Implementation Guide`, and `Mock Guide` before adding behavior.
- The scaffold is present now; later phases should extend it behind the thin facade rather than widening the API early.

Exit criteria:

- the package compiles
- the facade is thin
- the docs are in place

## Phase 1: DOM Core

- Parse HTML into the internal DOM store.
- Implement the initial selector subset, bounded attribute selectors, and a bounded combinator slice.
- Implement DOM dump and assertion helpers. The current workspace already has the initial assertion slice; later work should expand it only when a bounded user-visible gap appears.

Exit criteria:

- HTML round-trips deterministically in tests
- selectors work for the first supported slice

## Phase 2: Script Core

- Implement the minimum script parser/evaluator slice.
- Add host bindings needed for inline bootstrap, including bounded `innerHTML` / `outerHTML` / `textContent` helpers, a bounded `documentCurrentScript` helper for classic inline scripts, and an explicit `expr(...)` wrapper for nested host expressions.
- Keep unsupported syntax explicit.

Exit criteria:

- inline scripts can mutate the DOM
- errors are classified and repeatable

## Phase 3: Events and Form Controls

- Add bounded capture/target/bubble event dispatch for click/input/change/submit, target-only focus/blur/reset behavior, `preventDefault`/`stopPropagation`-style event control, bounded listener removal, bounded `once` listeners, default actions, and user-like actions.
- Add input, checkbox, select, focus, blur, and submit behavior.
- Keep broader event semantics out of scope until a bounded need appears.

Exit criteria:

- deterministic event order
- deterministic form-control state updates

## Phase 4: Deterministic Runtime and Mocks

- Add fake clock, scheduler, bounded timers, bounded animation-frame callbacks, a bounded microtask queue, bounded history entries/state/scroll restoration, a bounded cookie jar, bounded web-storage state, and bounded `window.name` state. The current workspace already has `host:queueMicrotask()` draining at script/action boundaries.
- Add typed mock families and thin `Harness` actions.
- Keep capture and failure injection explicit.

Exit criteria:

- mock families are inspectable
- time-based behavior is deterministic
- timer-driven behavior is driven through `AdvanceTime()` and stays bounded
- history-driven behavior is deterministic and bounded
- cookie-state behavior is deterministic and bounded
- web-storage behavior is deterministic and bounded
- window-name behavior is deterministic and bounded

## Phase 5: Hardening

- Add subsystem tests, public contract tests, regression tests, and property tests.
- The current workspace already has seeded fuzz/property tests for the script, selector, timer/scheduler, location/history, location getters, cookie/window.name, and mock registry boundaries, plus read-only history stack snapshots through `DebugView.HistoryEntries()`, pending timer / animation-frame snapshots through `DebugView.PendingTimers()` / `DebugView.PendingAnimationFrames()`, pending microtask snapshots through `DebugView.PendingMicrotasks()`, cookie-jar snapshots through `DebugView.CookieJar()`, and registered event listener snapshots through `DebugView.EventListeners()`; keep extending those before widening parser surfaces.
- Read-only size snapshots such as `DebugView.SelectCount()`, `DebugView.TemplateCount()`, `DebugView.TableCount()`, `DebugView.ButtonCount()`, `DebugView.TextAreaCount()`, `DebugView.InputCount()`, `DebugView.FieldsetCount()`, `DebugView.LegendCount()`, `DebugView.OutputCount()`, `DebugView.LabelCount()`, `DebugView.ProgressCount()`, `DebugView.MeterCount()`, `DebugView.AudioCount()`, `DebugView.VideoCount()`, `DebugView.IframeCount()`, `DebugView.EmbedCount()`, and `DebugView.TrackCount()` should be kept in sync with the same DOM slices when they help explain parser or collection regressions.
- `DebugView.PictureCount()` should be kept in sync with the same DOM slice when it helps explain picture-collection regressions.
- `DebugView.SourceCount()` should be kept in sync with the same DOM slice when it helps explain source-collection regressions.
- `DebugView.DialogCount()` / `DebugView.DetailsCount()` / `DebugView.SummaryCount()` / `DebugView.SectionCount()` / `DebugView.MainCount()` / `DebugView.ArticleCount()` / `DebugView.NavCount()` / `DebugView.AsideCount()` / `DebugView.FigureCount()` / `DebugView.FigcaptionCount()` / `DebugView.HeaderCount()` / `DebugView.FooterCount()` / `DebugView.AddressCount()` / `DebugView.BlockquoteCount()` / `DebugView.ParagraphCount()` / `DebugView.PreCount()` / `DebugView.MarkCount()` / `DebugView.QCount()` / `DebugView.CiteCount()` / `DebugView.AbbrCount()` / `DebugView.StrongCount()` / `DebugView.SpanCount()` / `DebugView.DataCount()` / `DebugView.DfnCount()` / `DebugView.KbdCount()` / `DebugView.VarCount()` / `DebugView.CodeCount()` / `DebugView.SmallCount()` / `DebugView.TimeCount()` should be kept in sync with the same DOM slice when they help explain dialog/details/summary/section/main/article/nav/aside/figure/figcaption/header/footer/address/blockquote/paragraph/pre/mark/q/cite/abbr/strong/span/data/dfn/kbd/var/code/small/time collection regressions.
- `DebugView.SampCount()` should be kept in sync with the same DOM slice when it helps explain samp collection regressions.
- `DebugView.RubyCount()` should be kept in sync with the same DOM slice when it helps explain ruby collection regressions.
- `DebugView.RtCount()` should be kept in sync with the same DOM slice when it helps explain rt collection regressions.
- Define the release checklist. The checklist now lives in `release-checklist.md`.

Exit criteria:

- behavior is covered at the public boundary and the internal boundary

## Phase 6: Selector and Query Expansion

- Expand selector support in bounded slices, including bounded attribute selectors such as `[attr]`, `[attr=value]`, `[attr~=value]`, `[attr|=value]`, `[attr^=value]`, `[attr$=value]`, and `[attr*=value]`, descendant, child, sibling combinators, and a bounded pseudo-class slice for `:root`, `:scope`, `:defined`, `:state(identifier)`, `:active`, `:hover`, `:empty`, `:checked`, `:indeterminate`, `:autofill`, `:-webkit-autofill`, `:default`, `:enabled`, `:disabled`, `:required`, `:optional`, `:read-only`, `:read-write`, `:valid`, `:invalid`, `:user-valid`, `:user-invalid`, `:in-range`, `:out-of-range`, `:first-child`, `:last-child`, `:first-of-type`, `:last-of-type`, `:only-child`, `:only-of-type`, `:nth-child()`, `:nth-of-type()`, `:nth-last-child()`, `:nth-last-of-type()`, `:link`, `:any-link`, `:visited`, `:local-link`, `:lang()`, `:dir()`, `:placeholder-shown`, `:blank`, `:heading`, `:heading(integer#)`, `:playing`, `:paused`, `:seeking`, `:buffering`, `:stalled`, `:muted`, `:volume-locked`, `:modal`, `:popover-open`, `:open`, `:focus`, `:focus-visible`, `:focus-within`, `:target`, `:target-within`, `:is()`, `:where()`, `:not()`, and `:has()` first.
- Add script-side query APIs that reuse the same selector engine, plus a minimal snapshot `NodeList` and bounded live `HTMLCollection` / `NodeList` slices for `children`, `document.images`, `document.forms`, `form.elements`, `fieldset.elements`, `select.options`, `select.selectedOptions`, `datalist.options`, `table.rows`, `table.tBodies`, `HTMLTableSectionElement.rows`, `tr.cells`, `document.scripts`, `document.links`, `document.anchors`, `childNodes`, `template.content.childNodes`, and script/image/form-count inspection.
- Expand live collections only as needed by user-visible gaps.

Exit criteria:

- DOM and script querying use the same core selector logic

## Phase 7: Reflection, Mutation, Serialization

- Add bounded attribute reflection helpers, then bounded `classList` / `dataset` views, tree mutation primitives, and HTML serialization/insertion surfaces.
- The current workspace already has public `classList` / `dataset` views plus the public selector-based tree-mutation wrappers, including `TextContent()` / `SetTextContent()`, `ReplaceChildren()`, `CloneNode()`, and `WriteHTML()` for bounded document-write-style replay; on `textarea`, content mutations that change its contents keep the reset default value in sync; keep expanding only the slices that are still missing.
- Keep each slice bounded and documented.

Exit criteria:

- mutation updates the DOM deterministically
- live collections stay coherent after mutation

## Working Rules

- Do not move to a later phase until the earlier phase is covered by tests.
- Do not add a new public `Harness` method until the capability matrix has a row for it.
- Use `../html-standard/` when adding or changing HTML behavior.
- Prefer small slices over large parity pushes.
