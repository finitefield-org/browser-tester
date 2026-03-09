# HTML Spec Conformance TODO

## Current Status

- `P0: Parsing, Tree Construction, and Serialization` is complete.
- `P1.1` through `P1.139` are complete.
- Shared reflection, URL/URLSearchParams parity, constructor/property-surface parity, DOM collection parity, form named-property parity, and the current media wrapper/source-selection sweeps are already implemented and verified.
- No new test-only mock is currently required. If a future P1 task needs a deterministic mock, document it in `/Users/kazuyoshitoshiya/Documents/GitHub/browser-tester/README.md`.

## Recently Completed

- `P1.139: Exposed element-algorithm audit beyond shared reflection`
  - locked `details` / `summary` default-action boundaries so `toggle` stays local to the owning `<details>` element and `summary` click `preventDefault()` suppresses the open-state transition
  - locked `dialog` `beforetoggle` cancellation so prevented open/close transitions do not leak `toggle` / `close` side effects before the element state actually changes
  - verified with:
  - `cargo test --lib dom_details_element -- --nocapture`
  - `cargo test --lib dom_dialog_element -- --nocapture`
  - `cargo test --lib dom_summary_element -- --nocapture`
  - `cargo test --lib dom_navigation_dialog -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib` (`2478 passed, 0 failed`)

- `P1.138: Residual reflected-attribute and fast-path audit across exposed non-form elements`
  - aligned the remaining non-form reflected-attribute DOM fast paths so explicit own data/accessor shadowing wins over reflected fallback for residual generic element, dialog, hyperlink, area, and media property surfaces
  - aligned parser-specialized direct assignment with the same own-property precedence and closed the matching generic bracket/property-path gaps for `closedBy`, `elementTiming`, plain hyperlink string/boolean properties, and media boolean properties
  - locked shared regressions around `defineProperty`, direct assignment, `Reflect.set(...)`, bracket access, and delete-to-reflection restore for the remaining audited non-form properties
  - verified with:
  - `cargo test --lib attribute_reflection_html_2_6_1_non_form_plain_property_shadow_define_property_delete_and_fast_path_parity_work -- --nocapture`
  - `cargo test --lib attribute_reflection_html_2_6_1_non_form_hyperlink_and_media_plain_property_shadow_define_property_delete_and_fast_path_parity_work -- --nocapture`
  - `cargo test --lib dom_attribute_reflection_shared`
  - `cargo test --lib dom_audio_element`
  - `cargo test --lib dom_navigation_dialog`
  - `cargo test --lib dom_area_element`
  - `cargo fmt`
  - `cargo test --lib` (`2475 passed, 0 failed`)

- `P1.137: Residual global-attribute audit across exposed element families`
  - aligned remaining global-attribute DOM fast paths so `accessKey`, `autocapitalize`, `autocorrect`, `contentEditable`, `draggable`, `enterKeyHint`, `hidden`, `inert`, `inputMode`, `nonce`, `popover`, `spellcheck`, `tabIndex`, and `translate` respect explicit own data/accessor shadowing before reflected fallback
  - aligned parser-specialized direct assignment with the same own-property precedence, closing the gap between generic node lookup and DOM fast-path setter behavior for the remaining audited global attributes
  - locked the remaining exposed-family breadth with a shared regression around `defineProperty`, direct assignment, bracket access, and delete-to-reflection restore for the audited attributes
  - verified with:
  - `cargo test --lib attribute_reflection_html_3_2_6_remaining_global_attributes_shadow_define_property_delete_and_fast_path_parity_work -- --nocapture`
  - `cargo test --lib dom_attribute_reflection_shared`
  - `cargo fmt`
  - `cargo test --lib` (`2473 passed, 0 failed`)

- `P1.136: File input files/value/reset and mock-determinism sweep`
  - aligned file-input FormData integration with the current mock-backed model by contributing one entry per selected file name, while keeping fakepath/value semantics and `files = null` clearing behavior deterministic
  - locked reset-driven file clearing and same-file reselection so `form.reset()` drops `files`/`value`, suppresses spurious cancel/change noise, and allows a clean second selection of the same mock file
  - verified with:
  - `cargo test --lib html_input_file_reset_clears_files_and_same_file_reselection_replays_input_change_work -- --nocapture`
  - `cargo test --lib html_input_file_form_data_tracks_multiple_files_and_files_null_clears_work -- --nocapture`
  - `cargo test --lib dom_events_input_runtime`
  - `cargo test --lib dom_input_element`
  - `cargo test --lib issue_121_127_finitefield_site_regressions`
  - `cargo test --lib window_forms_trace`
  - `cargo test --lib dom_input_file_array_buffer`
  - `cargo test --lib issue_84_89_input_file_object_url_pipeline`
  - `cargo fmt`
  - `cargo test --lib` (`2472 passed, 0 failed`)

- `P1.135: Clipboard copy/paste default-action and cancellation sweep`
  - aligned trusted paste default action with event-local `clipboardData` mutation so inherited/raw-getter `setData('text/plain', ...)` now drives the inserted text instead of falling back to the original platform clipboard snapshot
  - locked trusted copy override coverage for inherited/raw-getter call paths while preserving the existing “preventDefault without text/plain write keeps the clipboard untouched” behavior
  - verified with:
  - `cargo test --lib element_paste_event_mutated_clipboard_data_drives_default_insertion -- --nocapture`
  - `cargo test --lib element_copy_event_inherited_raw_getter_prevent_default_override_works -- --nocapture`
  - `cargo test --lib dom_element_copy_event`
  - `cargo test --lib dom_element_paste_event`
  - `cargo test --lib dom_dispatch_paste_clipboard_data`
  - `cargo test --lib issue_99_dispatch_paste_bubbles`
  - `cargo fmt`
  - `cargo test --lib` (`2470 passed, 0 failed`)

- `P1.134: Selection and selectionchange interaction sweep`
  - aligned trusted text-edit actions so `type_text`, trusted paste, and text-control `select()` focus the control first and surface `selectionchange` before the observable `input` path completes
  - locked selection/focus interplay for text-control selection updates and ensured the selection helper path no longer skips focus when selection is changed programmatically through exposed control APIs
  - verified with:
  - `cargo test --lib trusted_type_text_focuses_control_and_dispatches_selectionchange_before_input_work -- --nocapture`
  - `cargo test --lib trusted_paste_focuses_target_and_dispatches_selectionchange_before_input_work -- --nocapture`
  - `cargo test --lib text_control_select_focuses_element_and_dispatches_selectionchange_work -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib` (`2468 passed, 0 failed`)

- `P1.133: Focus/blur/focusin/focusout and activeElement parity sweep`
  - aligned focus-event flags and ordering so `focus` / `blur` are non-bubbling, `focusin` / `focusout` bubble without becoming cancelable, and blur-time `activeElement` falls back cleanly when focus is cleared
  - blocked focus from landing on disconnected nodes and hidden-attribute targets, and locked bubbling plus `document.activeElement` fallback coverage with focused regressions
  - verified with:
  - `cargo test --lib focus_in_and_focus_out_events_are_dispatched -- --nocapture`
  - `cargo test --lib focus_and_blur_do_not_bubble_but_focusin_and_focusout_do_work -- --nocapture`
  - `cargo test --lib document_active_element_ignores_hidden_and_disconnected_focus_targets -- --nocapture`
  - `cargo test --lib dom_document_active_element_property`
  - `cargo test --lib dom_events_input_runtime`
  - `cargo test --lib dom_fieldset_element`
  - `cargo test --lib dom_label_element`
  - `cargo test --lib dom_navigation_dialog`
  - `cargo fmt`
  - `cargo test --lib` (`2465 passed, 0 failed`)

- `P1.132: Input/change/click event ordering and cancellation sweep`
  - aligned trusted control interactions so checkbox/radio pre-activation happens before `click`, canceled `click` restores state, and committed paths dispatch non-cancelable `input` then `change`
  - committed text-control `change` on blur for user edits only, while keeping script-only `.value` mutation from scheduling blur-time `change`, and locked submit/reset/file/select ordering regressions
  - verified with:
  - `cargo test --lib trusted_checkbox_click_orders_click_before_input_change_and_canceled_click_restores_state_work -- --nocapture`
  - `cargo test --lib trusted_radio_click_orders_click_before_input_change_and_canceled_click_restores_group_work -- --nocapture`
  - `cargo test --lib text_input_change_commits_on_blur_only_for_user_input_work -- --nocapture`
  - `cargo test --lib textarea_change_commits_on_blur_only_for_user_input_work -- --nocapture`
  - `cargo test --lib option_click_orders_click_before_input_change_and_prevent_default_skips_selection_work -- --nocapture`
  - `cargo test --lib trusted_submit_and_reset_click_ordering_and_click_prevent_default_work -- --nocapture`
  - `cargo test --lib html_input_file_selection_orders_input_then_change_and_cancel_work -- --nocapture`
  - `cargo test --lib dom_events_input_runtime`
  - `cargo test --lib dom_select_element`
  - `cargo test --lib dom_form_element`
  - `cargo test --lib dom_label_element`
  - `cargo fmt`
  - `cargo test --lib` (`2463 passed, 0 failed`)

- `P1.131: Checked/value/selectedness synchronization sweep`
  - separated current and default state for option selectedness plus text/checked defaults so `defaultValue`, `defaultChecked`, `defaultSelected`, and current live state no longer collapse into one storage path
  - aligned radio-group synchronization with external `form` ownership changes and locked reset/dirty-state coverage across input, textarea, select, and option behavior
  - verified with:
  - `cargo test --lib dom_input_element -- --nocapture`
  - `cargo test --lib dom_select_element -- --nocapture`
  - `cargo test --lib dom_textarea_element -- --nocapture`
  - `cargo test --lib html_input_radio_external_form_owner_mutation_keeps_group_sync_and_validity_work -- --nocapture`
  - `cargo test --lib dom_events_input_runtime -- --nocapture`
  - `cargo test --lib dom_form_element -- --nocapture`
  - `cargo test --lib dom_option_element -- --nocapture`
  - `cargo test --lib operators_advanced_selectors -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib` (`2456 passed, 0 failed`)

- `P1.130: Label activation and implicit-control default-action sweep`
  - added descendant-label activation retargeting so clicks inside a `<label>` activate and focus the associated control unless an intervening labelable descendant should consume the activation
  - locked checkbox/select/file/button activation through labels, plus nested interactive-descendant non-retarget behavior, with focused regressions
  - verified with:
  - `cargo test --lib dom_label_element -- --nocapture`
  - `cargo test --lib html_input_radio_required_group_and_label_click_work -- --nocapture`
  - `cargo test --lib dom_button_element -- --nocapture`
  - `cargo test --lib dom_events_input_runtime -- --nocapture`
  - `cargo test --lib dom_form_element -- --nocapture`
  - `cargo test --lib dom_select_element -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib` (`2452 passed, 0 failed`)

- `P1.129: Form reset algorithm and dirty-state restoration sweep`
  - separated current/default restoration for text inputs, checkboxes/radios, textareas, selects/options, and outputs so `form.reset()` plus reset-button default action restore default state instead of the latest live value
  - added focused regressions for dirty `value` / `checked` restoration, single-select fallback selection, textarea reset, and output reset across both `form.reset()` and reset-button click paths
  - verified with:
  - `cargo test --lib dom_form_element -- --nocapture`
  - `cargo test --lib dom_events_input_runtime -- --nocapture`
  - `cargo test --lib dom_select_element -- --nocapture`
  - `cargo test --lib dom_textarea_element -- --nocapture`
  - `cargo test --lib dom_navigation_dialog -- --nocapture`
  - `cargo test --lib dom_output_element -- --nocapture`
  - `cargo test --lib operators_advanced_selectors -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib` (`2450 passed, 0 failed`)

- `P1.128: Validation ordering, invalid-event dispatch, and prevented-default sweep`
  - routed trusted submission validation through shared invalid-event dispatch so blocked submit flows fire non-bubbling, cancelable `invalid` events before submission is suppressed
  - aligned direct form/control `checkValidity()` and `reportValidity()` fast paths with the same `invalid` semantics and locked dialog-submit prevented-default behavior
  - verified with:
  - `cargo test --lib input_and_form_validity_methods_dispatch_non_bubbling_cancelable_invalid_events_work -- --nocapture`
  - `cargo test --lib dialog_invalid_submit_dispatches_non_bubbling_invalid_and_keeps_dialog_open_work -- --nocapture`
  - `cargo test --lib dom_form_element`
  - `cargo test --lib dom_navigation_dialog`
  - `cargo test --lib dom_events_input_runtime`
  - `cargo test --lib dom_button_element`
  - `cargo test --lib dom_select_element`
  - `cargo fmt`
  - `cargo test --lib` (`2448 passed, 0 failed`)

- `P1.127: Form-associated override-attribute and owner/default-action sweep`
  - routed submit default-action through effective submitter override handling for `formmethod` and `formnovalidate`
  - locked `method="dialog"` suppression via submitter override, plus external submitter owner reassociation with override attributes and dialog close behavior
  - verified with:
  - `cargo test --lib dialog_submitter_formmethod_override_can_suppress_close_default_action -- --nocapture`
  - `cargo test --lib external_submitter_override_attributes_and_owner_reassociation_drive_dialog_default_action -- --nocapture`
  - `cargo test --lib dom_navigation_dialog`
  - `cargo test --lib dom_form_element`
  - `cargo test --lib dom_events_input_runtime`
  - `cargo test --lib dom_button_element`
  - `cargo fmt`
  - `cargo test --lib` (`2446 passed, 0 failed`)

- `P1.126: SubmitEvent submitter and trusted submission-path matrix sweep`
  - added `submit` event submitter payload propagation for trusted click, `requestSubmit(...)`, and implicit Enter-triggered submission
  - locked image submitters, external submitter owner reassociation, and `form.submit()` bypass behavior with focused regressions
  - verified with:
  - `cargo test --lib form_submitter_property_tracks_request_submit_image_and_submit_bypass_work -- --nocapture`
  - `cargo test --lib trusted_click_and_implicit_enter_choose_default_submitter_work -- --nocapture`
  - `cargo test --lib external_submitter_request_submit_and_trusted_click_follow_owner_reassociation_work -- --nocapture`
  - `cargo test --lib dom_form_element`
  - `cargo test --lib dom_navigation_dialog`
  - `cargo test --lib dom_button_element`
  - `cargo fmt`
  - `cargo test --lib` (`2444 passed, 0 failed`)

- `P1.125: HTMLMediaElement load-triggered direct/nested precedence matrix and media wrapper callable alias-path residual sweep`
  - locked explicit `load()`-driven precedence flips between direct `src` and nested `<source>` candidates
  - locked alias-object plus borrowed `call` / `apply` stability on cached `TextTrackList` and `TimeRanges`
  - verified with:
  - `cargo test --lib audio_load_triggered_direct_nested_precedence_matrix_stays_aligned_work -- --nocapture`
  - `cargo test --lib video_media_wrapper_callable_alias_paths_stay_live_across_load_triggered_precedence_churn_work -- --nocapture`
  - `cargo test --lib dom_audio_element`
  - `cargo test --lib dom_video_element`
  - `cargo fmt`
  - `cargo test --lib` (`2441 passed, 0 failed`)

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

- [ ] `P2.1: Media/network/navigation harness-surface audit kickoff`
  - inventory the currently exposed loading, navigation, and media-state APIs where the harness already produces observable results without requiring full browser loading machinery
  - turn the first concrete `P2` gap into a spec-anchored acceptance test, preferring deterministic source-selection, media-state, and harness-navigation behaviors already modeled in-repo

## P1 Gaps From `/doc/html-spec-conformance-roadmap.md`

### Forms, Default Actions, and Events

- [x] `P1.126: SubmitEvent submitter and trusted submission-path matrix sweep`
  - deepen `4.10 Forms` coverage for trusted click, keyboard-triggered implicit submission, `requestSubmit(...)`, and `form.submit()` so submitter selection stays aligned across direct and trusted paths
  - verify `SubmitEvent.submitter`, external submit controls, image submitters, and owner reassociation churn under `form` attribute changes

- [x] `P1.127: Form-associated override-attribute and owner/default-action sweep`
  - audit `formAction`, `formMethod`, `formEnctype`, `formTarget`, and `formNoValidate` when submitter ownership changes dynamically or when external controls target a form
  - verify default-action parity between reflected submitter overrides, navigation-harness behavior, and validation bypass rules

- [x] `P1.128: Validation ordering, invalid-event dispatch, and prevented-default sweep`
  - tighten ordering around `checkValidity()`, `reportValidity()`, invalid-event dispatch, and prevented-default behavior during trusted submission flows
  - cover dialog-form interactions, validation bypass cases, and the point at which submission/navigation is suppressed

- [x] `P1.129: Form reset algorithm and dirty-state restoration sweep`
  - audit `form.reset()` plus reset-button default action across `input`, `textarea`, `select`, `option`, `output`, radio groups, and checkbox groups
  - verify dirty value/checkedness restoration, default-value/default-checked interactions, and prevented-default reset behavior

- [x] `P1.130: Label activation and implicit-control default-action sweep`
  - broaden coverage for label-to-control activation, nested label edge cases, disabled controls, and focus transfer through trusted clicks and keyboard activation
  - verify checkbox/radio/select/button/file-input default actions triggered through `<label>` association

- [x] `P1.131: Checked/value/selectedness synchronization sweep`
  - broaden checkedness, selectedness, and dirty-value synchronization coverage across radios, checkboxes, selects, options, text inputs, and textareas
  - verify synchronization under trusted user-style interaction, programmatic mutation, and reset flows

- [x] `P1.132: Input/change/click event ordering and cancellation sweep`
  - audit ordering and cancellation behavior around `click`, `input`, `change`, and related control events for trusted interactions versus script-triggered mutations
  - cover text controls, checkbox/radio toggles, select changes, submit/reset controls, and file-input selection updates

- [x] `P1.133: Focus/blur/focusin/focusout and activeElement parity sweep`
  - tighten `6 User interaction` coverage for focus transitions, bubbling/non-bubbling focus events, and `document.activeElement` updates across trusted pointer/keyboard flows
  - verify disabled/hidden/disconnected control behavior and interaction with form/label activation

- [x] `P1.134: Selection and selectionchange interaction sweep`
  - verify `selectionchange` ordering, `Selection` / `Range` liveness, and selection/focus interplay across text controls, contenteditable hosts, and document text selection flows
  - cover trusted edit-style actions that should update selection state before observable events/default actions complete

- [x] `P1.135: Clipboard copy/paste default-action and cancellation sweep`
  - deepen trusted `copy` / `paste` default-action coverage so clipboard mutation and DOM/text insertion align with cancellation, bubbling, and `clipboardData` mutation semantics
  - verify remaining differences between trusted dispatch, synthetic dispatch, and inherited/raw-getter clipboard paths

- [x] `P1.136: File input files/value/reset and mock-determinism sweep`
  - audit exposed file-input behavior including `files`, fakepath/value semantics, same-file reselection, reset rules, cancel/default-action ordering, and FormData integration
  - if new deterministic picker-style mocks are required, add them narrowly and document them in `/Users/kazuyoshitoshiya/Documents/GitHub/browser-tester/README.md`

- [x] `P1.137: Residual global-attribute audit across exposed element families`
  - finished the roadmap’s `3.2.4.1` / `3.2.6` residual audit for the remaining specialized global-attribute DOM fast paths and direct setter routes
  - verified own-shadow precedence across the remaining audited global attributes with shared reflection regressions on generic and parser-specialized access paths

- [x] `P1.138: Residual reflected-attribute and fast-path audit across exposed non-form elements`
  - finished the remaining non-form `2.6.1` sweep for specialized getter/setter and parser fast paths outside the shared reflection pipeline
  - verified alias properties, plain hyperlink/media reflected surfaces, and delete/defineProperty shadow parity across generic and parser-specialized access paths

- [x] `P1.139: Exposed element-algorithm audit beyond shared reflection`
  - reviewed the remaining exposed element-local algorithms and locked `details` / `summary` toggle locality plus `dialog` `beforetoggle` cancellation/default-action boundaries with focused regressions
  - verified public event/default-action parity for element-specific behavior that is outside the shared reflection pipeline

### Attribute Reflection, Global Attributes, and Element Algorithms

## Verification Rule

- Every P1 task should end with:
  - targeted regressions for the touched behavior
  - relevant focused suites
  - `cargo fmt`
  - `cargo test --lib`

- If a task introduces a new test-only mock, update `/Users/kazuyoshitoshiya/Documents/GitHub/browser-tester/README.md` in the same change.
