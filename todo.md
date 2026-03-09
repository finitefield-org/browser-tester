# HTML Spec Conformance TODO

## Current Status

- `P0: Parsing, Tree Construction, and Serialization` is complete.
- `P1.1` through `P1.124` are complete.
- Shared reflection, URL/URLSearchParams parity, constructor/property-surface parity, DOM collection parity, form named-property parity, and the current media wrapper/source-selection sweeps are already implemented and verified.
- No new test-only mock is currently required. If a future P1 task needs a deterministic mock, document it in `/Users/kazuyoshitoshiya/Documents/GitHub/browser-tester/README.md`.

## Recently Completed

- `P1.121: HTMLMediaElement source batch-reset matrix and media wrapper descriptor/copy residual sweep`
  - locked source-list batch-reset coverage and cached media-wrapper descriptor/object-copy behavior
  - verified with targeted audio/video regressions, `cargo fmt`, and `cargo test --lib`

- `P1.122: HTMLMediaElement source string-rebuild matrix and media wrapper expando/prototype residual sweep`
  - locked string-based source rebuild coverage and cached media-wrapper expando/prototype persistence
  - verified with targeted audio/video regressions, `cargo fmt`, and `cargo test --lib`

- `P1.123: HTMLMediaElement mixed rebuild churn and media wrapper borrowed object-surface residual sweep`
  - locked mixed string/DOM rebuild coverage and borrowed object-surface behavior on cached media wrappers
  - verified with targeted audio/video regressions, `cargo fmt`, and `cargo test --lib`

- `P1.124: HTMLMediaElement source reset/direct-property interaction and media wrapper callable-identity residual sweep`
  - locked direct `src` assignment/removal plus `load()` churn against nested source resets
  - locked raw-getter identity, extracted method reuse, and borrowed callable stability on cached `TextTrackList` and `TimeRanges`
  - verified with:
  - `cargo test --lib audio_direct_src_reset_and_load_churn_keeps_current_src_aligned_work -- --nocapture`
  - `cargo test --lib video_cached_media_wrapper_callables_stay_stable_across_direct_src_reset_and_load_churn_work -- --nocapture`
  - `cargo test --lib dom_audio_element`
  - `cargo test --lib dom_video_element`
  - `cargo fmt`
  - `cargo test --lib` (`2439 passed, 0 failed`)

## Active Task

- [ ] `P1.125: HTMLMediaElement load-triggered direct/nested precedence matrix and media wrapper callable alias-path residual sweep`
  - verify `currentSrc`, `networkState`, and `readyState` across explicit `load()` churn that repeatedly flips precedence between direct `src` and nested `<source>` candidates
  - cover alias-object, bracket-path, and borrowed `call`/`apply` stability on cached `TextTrackList` and `TimeRanges` while `load()` keeps re-resolving candidates
  - verify with targeted regressions, relevant media suites, and `cargo test --lib`

## P1 Gaps From `/doc/html-spec-conformance-roadmap.md`

### Forms, Default Actions, and Events

- [ ] `P1.126: SubmitEvent submitter and trusted submission-path matrix sweep`
  - deepen `4.10 Forms` coverage for trusted click, keyboard-triggered implicit submission, `requestSubmit(...)`, and `form.submit()` so submitter selection stays aligned across direct and trusted paths
  - verify `SubmitEvent.submitter`, external submit controls, image submitters, and owner reassociation churn under `form` attribute changes

- [ ] `P1.127: Form-associated override-attribute and owner/default-action sweep`
  - audit `formAction`, `formMethod`, `formEnctype`, `formTarget`, and `formNoValidate` when submitter ownership changes dynamically or when external controls target a form
  - verify default-action parity between reflected submitter overrides, navigation-harness behavior, and validation bypass rules

- [ ] `P1.128: Validation ordering, invalid-event dispatch, and prevented-default sweep`
  - tighten ordering around `checkValidity()`, `reportValidity()`, invalid-event dispatch, and prevented-default behavior during trusted submission flows
  - cover dialog-form interactions, validation bypass cases, and the point at which submission/navigation is suppressed

- [ ] `P1.129: Form reset algorithm and dirty-state restoration sweep`
  - audit `form.reset()` plus reset-button default action across `input`, `textarea`, `select`, `option`, `output`, radio groups, and checkbox groups
  - verify dirty value/checkedness restoration, default-value/default-checked interactions, and prevented-default reset behavior

- [ ] `P1.130: Label activation and implicit-control default-action sweep`
  - broaden coverage for label-to-control activation, nested label edge cases, disabled controls, and focus transfer through trusted clicks and keyboard activation
  - verify checkbox/radio/select/button/file-input default actions triggered through `<label>` association

- [ ] `P1.131: Checked/value/selectedness synchronization sweep`
  - broaden checkedness, selectedness, and dirty-value synchronization coverage across radios, checkboxes, selects, options, text inputs, and textareas
  - verify synchronization under trusted user-style interaction, programmatic mutation, and reset flows

- [ ] `P1.132: Input/change/click event ordering and cancellation sweep`
  - audit ordering and cancellation behavior around `click`, `input`, `change`, and related control events for trusted interactions versus script-triggered mutations
  - cover text controls, checkbox/radio toggles, select changes, submit/reset controls, and file-input selection updates

- [ ] `P1.133: Focus/blur/focusin/focusout and activeElement parity sweep`
  - tighten `6 User interaction` coverage for focus transitions, bubbling/non-bubbling focus events, and `document.activeElement` updates across trusted pointer/keyboard flows
  - verify disabled/hidden/disconnected control behavior and interaction with form/label activation

- [ ] `P1.134: Selection and selectionchange interaction sweep`
  - verify `selectionchange` ordering, `Selection` / `Range` liveness, and selection/focus interplay across text controls, contenteditable hosts, and document text selection flows
  - cover trusted edit-style actions that should update selection state before observable events/default actions complete

- [ ] `P1.135: Clipboard copy/paste default-action and cancellation sweep`
  - deepen trusted `copy` / `paste` default-action coverage so clipboard mutation and DOM/text insertion align with cancellation, bubbling, and `clipboardData` mutation semantics
  - verify remaining differences between trusted dispatch, synthetic dispatch, and inherited/raw-getter clipboard paths

- [ ] `P1.136: File input files/value/reset and mock-determinism sweep`
  - audit exposed file-input behavior including `files`, fakepath/value semantics, same-file reselection, reset rules, cancel/default-action ordering, and FormData integration
  - if new deterministic picker-style mocks are required, add them narrowly and document them in `/Users/kazuyoshitoshiya/Documents/GitHub/browser-tester/README.md`

### Attribute Reflection, Global Attributes, and Element Algorithms

- [ ] `P1.137: Residual global-attribute audit across exposed element families`
  - finish the roadmap’s `3.2.4.1` / `3.2.6` audit for remaining exposed elements, prioritizing global attributes that still rely on specialized DOM fast paths or element-local setters
  - verify role/tabindex/hidden/title/lang/dir/draggable/contenteditable-style parity where shared reflection exists but element-family breadth is not yet fully pinned down

- [ ] `P1.138: Residual reflected-attribute and fast-path audit across exposed non-form elements`
  - continue the per-element `2.6.1` sweep for exposed element families that still have specialized getter/setter or parser fast paths outside the shared reflection pipeline
  - prioritize alias properties, URL-backed attributes, numeric/string/boolean reflected properties, and delete/defineProperty shadow parity on non-form surfaces

- [ ] `P1.139: Exposed element-algorithm audit beyond shared reflection`
  - review the roadmap’s `4.x` “core element APIs” residuals for element-specific algorithms already surfaced by the harness, especially where behavior is not just attribute reflection
  - target owner/default resolution, activation behavior, liveness guarantees, and method/default-action parity on exposed element families that still depend on element-local runtime code

## Verification Rule

- Every P1 task should end with:
  - targeted regressions for the touched behavior
  - relevant focused suites
  - `cargo fmt`
  - `cargo test --lib`

- If a task introduces a new test-only mock, update `/Users/kazuyoshitoshiya/Documents/GitHub/browser-tester/README.md` in the same change.
