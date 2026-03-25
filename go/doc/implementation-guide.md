# Implementation Guide

This guide describes how the Go workspace should be built.
It is deliberately conservative: explicit subsystems, thin public facade, and deterministic mocks first.

## Design Rules

1. Keep `Harness` thin. Its methods should delegate into runtime or mock registry objects.
2. Keep mutable state in the owning subsystem. Do not scatter the same state across facade, runtime, and DOM.
3. Keep public and test-only APIs separate. If something exists only for tests, it belongs in a mock family or a debug view.
4. Keep builder configuration explicit. Do not encode mock seeds into unrelated state.
5. Use `../html-standard/` as the reference for any HTML/DOM slice before coding it.
6. Prefer bounded slices over broad partial compatibility. Each new slice should have a clear exit criterion.
7. Do not spend implementation budget on legacy or deprecated spec behavior. Treat those branches as out of scope unless a specific, documented user-visible need requires them.

## Suggested File Layout

The exact file names can change, but the intended ownership is:

```text
browsertester/
  harness.go
  errors.go
  debug.go
  mocks.go
internal/
  dom/
    store.go
    parser.go
    selector.go
    collections.go
    serialize.go
  runtime/
    session.go
    scheduler.go
    events.go
    history.go
    location.go
    window_name.go
    storage.go
  script/
    runtime.go
    parser.go
    evaluator.go
    bindings.go
  mocks/
    fetch.go
    dialogs.go
    clipboard.go
    location.go
    open.go
    close.go
    print.go
    scroll.go
    matchmedia.go
    downloads.go
    fileinput.go
    storage.go
```

## Build Order

### Phase 0: Scaffold

- Create the module, public package, internal packages, and a minimal build.
- Define the error taxonomy and the public facade types.
- Define `SessionConfig` with explicit fields.
- Define `MockRegistryView` and `DebugView` early so later APIs have a place to live; keep read-only seed snapshots such as random seed, matchMedia rules, builder failure readouts, fetch call traces, fetch rule traces, dialog traces, download/file-input traces, storage change traces, browser-action call traces, matchMedia query traces, matchMedia listener traces, clipboard write traces, registered event listener snapshots, cookie-jar snapshots, history cursor snapshots, visited URL snapshots, live node-count snapshots, live script-count snapshots, and pending timer / animation-frame / microtask snapshots there when they are needed for debugging.
- Keep DOM initialization state visible through `DebugView.DOMReady()` and `DebugView.DOMError()` when a caller needs to distinguish successful bootstrap from parse/runtime failure, and expose live `DebugView.NodeCount()` / `DebugView.ScriptCount()` / `DebugView.ImageCount()` / `DebugView.FormCount()` / `DebugView.SelectCount()` snapshots when a caller needs read-only size state after bootstrap.
- Keep live `DebugView.TemplateCount()` / `DebugView.TableCount()` / `DebugView.ButtonCount()` / `DebugView.TextAreaCount()` / `DebugView.InputCount()` / `DebugView.FieldsetCount()` / `DebugView.LegendCount()` / `DebugView.OutputCount()` / `DebugView.LabelCount()` / `DebugView.ProgressCount()` / `DebugView.MeterCount()` / `DebugView.AudioCount()` / `DebugView.VideoCount()` snapshots available when a caller needs read-only template-, table-, button-, textarea-, input-, fieldset-, legend-, output-, label-, progress-, meter-, audio-, or video-count reflection after bootstrap.
- Keep `DebugView.PictureCount()` available when a caller needs read-only picture-count reflection after bootstrap.
- Keep `DebugView.SourceCount()` available when a caller needs read-only source-count reflection after bootstrap.
- Keep `DebugView.DialogCount()` available when a caller needs read-only dialog-count reflection after bootstrap.
- Keep live `DebugView.OptionLabels()` / `DebugView.SelectedOptionLabels()` / `DebugView.OptionValues()` / `DebugView.SelectedOptionValues()` / `DebugView.OptionCount()` / `DebugView.SelectedOptionCount()` / `DebugView.OptgroupCount()` / `DebugView.LinkCount()` / `DebugView.AnchorCount()` / `DebugView.OptgroupLabels()` snapshots available when a caller needs read-only option label/value/count reflection after bootstrap, and keep `DebugView.AudioCount()` / `DebugView.VideoCount()` / `DebugView.IframeCount()` / `DebugView.EmbedCount()` / `DebugView.TrackCount()` available when a caller needs read-only media-count reflection after bootstrap.
- Keep the most recently executed classic inline script visible through `DebugView.LastInlineScriptHTML()` when a caller needs to inspect bootstrap trace state.
- Keep the original builder HTML visible through `DebugView.InitialHTML()` when a caller needs to compare the configured source against later document replacement or mutation output.
- Keep read-only history stack snapshots there as well when a caller needs to inspect `DebugView.HistoryEntries()` or `DebugView.HistoryIndex()` without mutating session state, and keep visited URL snapshots there when a caller needs to inspect `DebugView.VisitedURLs()` without mutating session state. Keep registered event listener snapshots there when a caller needs to inspect `DebugView.EventListeners()` without mutating session state.
- The current scaffold already compiles with `go test ./...`; later phases should extend it without widening the facade prematurely.

Exit criteria:

- `go test ./...` passes with skeleton tests.

### Phase 1: DOM

- Implement HTML parsing into a tree store.
- Implement selector matching for the first bounded slice.
- Implement DOM dump helpers and the initial assertion helpers. The current Go workspace already has the initial assertion slice; later work should keep it bounded rather than widening the facade.

Exit criteria:

- Parsed HTML round-trips through the DOM dump in tests.
- Selector behavior is covered by contract and regression tests.

### Phase 2: Script Runtime Minimum

- Implement the lexer/parser/evaluator slice needed for inline bootstrap.
- Add host bindings for the initial DOM and document/window accessors, including bounded `innerHTML` / `outerHTML` / `textContent` helpers, a bounded `documentCurrentScript` helper for classic inline script execution, and the explicit `expr(...)` wrapper for nested host expressions.
- Keep the runtime deterministic and explicit about unsupported syntax.

Exit criteria:

- Inline scripts can mutate the DOM during bootstrap.
- Missing features fail explicitly, not silently.

### Phase 3: Events and User-Like Actions

- Implement bounded capture/target/bubble listener dispatch for click/input/change/submit, plus target-only focus/blur/reset behavior, `preventDefault`/`stopPropagation`-style event control, bounded listener removal, bounded `once` listeners, default actions, and form-control state updates.
- Add `Click`, `TypeText`, `SetChecked`, `SetSelectValue`, `Focus`, `Blur`, `Dispatch`, `DispatchKeyboard`, and `Submit`.

Exit criteria:

- Listener ordering and default actions are deterministic.
- `preventDefault` only suppresses bounded default actions that exist in the current slice, `stopPropagation` only suppresses later propagation within the current slice, `removeEventListener` only unregisters exact listener registrations that were added in the same bounded slice, and `once` listeners remove themselves after a single invocation.
- Later event semantics stay bounded unless explicitly added to the matrix.
- Default actions and form updates are covered by tests.

### Phase 4: Runtime Services and Mock Registry

- Implement the deterministic clock, a bounded microtask queue, bounded timers, bounded animation-frame callbacks, bounded history entries/state/scroll restoration, bounded cookie state, bounded `window.name` state, bounded web-storage state (`localStorage` / `sessionStorage`), bounded location read helpers, and scheduler.
- Implement typed mock families for fetch, dialogs, clipboard, location, open, close, print, scroll, matchMedia, downloads, file input, and storage.
- Add public mock actions on `Harness` as thin wrappers.

Exit criteria:

- Every family supports seed state, capture, and reset.
- Public actions remain thin and do not duplicate registry logic.
- Time control remains deterministic through `Harness.AdvanceTime()` and the bounded time/history/cookie/web-storage queues.

### Phase 5: Hardening

- Add subsystem tests for internal packages.
- Add public contract tests for the facade.
- Add regression tests for issue reproductions.
- Add fuzz/property tests for parser and scheduler boundaries.

Exit criteria:

- The implementation can be guarded by a repeatable publication checklist, documented in `release-checklist.md`.

### Phase 6: Selector and Query Expansion

- Expand selectors in bounded slices, starting with bounded attribute selectors such as `[attr]`, `[attr=value]`, `[attr~=value]`, `[attr|=value]`, `[attr^=value]`, `[attr$=value]`, and `[attr*=value]`, bounded descendant, child, sibling combinators, and a bounded pseudo-class slice for `:root`, `:scope`, `:defined`, `:state(identifier)`, `:active`, `:hover`, `:empty`, `:checked`, `:indeterminate`, `:autofill`, `:-webkit-autofill`, `:default`, `:enabled`, `:disabled`, `:required`, `:optional`, `:read-only`, `:read-write`, `:valid`, `:invalid`, `:user-valid`, `:user-invalid`, `:in-range`, `:out-of-range`, `:first-child`, `:last-child`, `:first-of-type`, `:last-of-type`, `:only-child`, `:only-of-type`, `:nth-child()`, `:nth-of-type()`, `:nth-last-child()`, `:nth-last-of-type()`, `:link`, `:any-link`, `:visited`, `:local-link`, `:lang()`, `:dir()`, `:placeholder-shown`, `:blank`, `:heading`, `:heading(integer#)`, `:playing`, `:paused`, `:seeking`, `:buffering`, `:stalled`, `:muted`, `:volume-locked`, `:modal`, `:popover-open`, `:open`, `:focus`, `:focus-visible`, `:focus-within`, `:target`, `:target-within`, `:is()`, `:where()`, `:not()`, and `:has()`.
- Add script-side `querySelector`, `querySelectorAll`, `matches`, and `closest`.
- Add live collection slices only when a user-visible gap demands them; the current workspace already has bounded live `HTMLCollection` slices for `children`, `document.images`, `document.forms`, `form.elements`, `fieldset.elements`, `select.options`, `select.selectedOptions`, `datalist.options`, `table.rows`, `table.tBodies`, `HTMLTableSectionElement.rows`, `tr.cells`, `document.scripts`, `document.links`, and `document.anchors`, plus bounded live `NodeList` slices for `childNodes` and `template.content.childNodes`, so later work should keep any additional collection slices similarly bounded.

Exit criteria:

- Query APIs reuse the same selector engine as the DOM layer.

### Phase 7: Reflection, Mutation, and Serialization

- Add attribute reflection, class/dataset views, selector-based tree mutation helpers, and HTML serialization/insertion helpers.
- The current workspace already has public `ClassList` / `Dataset` views on top of the internal helpers, plus public tree-mutation wrappers including `TextContent()` / `SetTextContent()`, `ReplaceChildren()`, `CloneNode()`, and `WriteHTML()` for bounded document-write-style replay; on `textarea`, content mutations that change its contents keep the reset default value in sync, and failed `WriteHTML()` replays roll back DOM and session state; later work should keep the slice bounded rather than widening the facade.
- Keep the supported slice bounded and documented.

Exit criteria:

- Mutation updates selectors and live collections deterministically.

## Test Policy

- Use `*_contract_test.go` for public facade behavior.
- Use `*_test.go` under `internal/...` for subsystem behavior.
- Use `*_regression_test.go` for issue reproductions.
- Use fuzz tests for parser, selector, and scheduler boundaries.
- Keep tests close to the behavior they protect.

## Change Rules

- If a new behavior belongs only to tests, add it as a mock family or a debug view.
- If a new behavior is user-facing, update the capability matrix and `README.md` before or with the code.
- If a new mock family is added, update the mock guide and add a minimal example plus failure coverage.
- Do not let `Harness` become a bag of setter methods.
- Legacy and deprecated spec paths are not implementation targets unless the capability matrix explicitly lists them for a concrete compatibility reason.
