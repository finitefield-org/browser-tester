# browser-tester Capability Matrix

## Purpose

This document classifies the current public surface of `browser-tester` by support level.

The goal is to make maintenance decisions easier:

- what is part of the stable public contract
- what is a stable test-only mock surface
- what is exposed but broader or more browser-like
- what should remain internal and free to change

This matrix is not a claim of full browser compatibility.
It is a maintenance and release document for the crate's own public surface.

## Support Levels

## Stable Core

These are the APIs and behaviors users should be able to rely on as the main contract of the crate.

Characteristics:

- central to typical `Harness`-based tests
- documented in the top-level README
- changes should be treated as high-risk
- regressions should be prevented with direct contract tests

## Stable Test Mocks

These are test-only browser-like surfaces that are intentionally exposed as deterministic control points.

Characteristics:

- not general browser features for their own sake
- directly support predictable testing
- expected to remain public and documented
- mock behavior and artifact capture are part of the intended contract

## Extended Browser-Like Surface

These are exposed and supported, but they are broader than the minimum core test-harness contract.

Characteristics:

- useful and real, but not the narrowest essential surface
- more likely to grow, narrow, or need stronger caveats
- should keep regression coverage, but should not silently expand into "full browser compatibility"

## Internal Only

These are implementation details.

Characteristics:

- not part of the intended public contract
- may change freely to support refactors or bug fixes
- should not be documented as user-facing guarantees

## Classification Rules

When deciding where a capability belongs:

1. If typical users need it to write basic deterministic DOM tests, it belongs in `Stable Core`.
2. If it exists mainly to inject or inspect deterministic browser-like test state, it belongs in `Stable Test Mocks`.
3. If it exposes broader browser-like behavior beyond the narrow test harness core, it belongs in `Extended Browser-Like Surface`.
4. If it is a helper, runtime primitive, or implementation detail, it belongs in `Internal Only`.

## Current Public Matrix

## Stable Core

Constructors:

- `Harness::from_html`
- `Harness::from_html_with_url`
- `Harness::from_html_with_local_storage`
- `Harness::from_html_with_url_and_local_storage`

User actions:

- `type_text`
- `set_select_value`
- `set_checked`
- `click`
- `focus`
- `blur`
- `press_enter`
- `submit`
- `dispatch`
- `dispatch_keyboard`

Clipboard-style user actions:

- `copy`
- `paste`
- `cut`

Assertions and debug:

- `assert_text`
- `assert_value`
- `assert_checked`
- `assert_exists`
- `dump_dom`

Time and scheduling:

- `now_ms`
- `advance_time`
- `advance_time_to`
- `flush`
- `run_due_timers`
- `run_next_timer`
- `run_next_due_timer`
- `pending_timers`
- `clear_timer`
- `clear_all_timers`
- `set_timer_step_limit`
- `set_random_seed`

Trace and diagnostics:

- `enable_trace`
- `take_trace_logs`
- `set_trace_stderr`
- `set_trace_events`
- `set_trace_timers`
- `set_trace_log_limit`

Why these are core:

- they define the primary `Harness` workflow
- they are directly documented in the main README
- they represent the crate's testing identity more than any browser-specific extra

## Stable Test Mocks

Network-like mocks:

- `set_fetch_mock`
- `set_fetch_mock_response`
- `clear_fetch_mocks`
- `take_fetch_calls`

Clipboard mocks:

- `set_clipboard_text`
- `clipboard_text`
- `set_clipboard_read_error`
- `set_clipboard_write_error`
- `clear_clipboard_errors`
- `take_clipboard_writes`

Dialog and print mocks:

- `enqueue_confirm_response`
- `set_default_confirm_response`
- `enqueue_prompt_response`
- `set_default_prompt_response`
- `take_alert_messages`
- `take_print_call_count`

Navigation mocks:

- `set_location_mock_page`
- `clear_location_mock_pages`
- `take_location_navigations`
- `location_reload_count`

Media-query mocks:

- `set_match_media_mock`
- `clear_match_media_mocks`
- `set_default_match_media_matches`
- `take_match_media_calls`

File and storage setup:

- `set_input_files`
- `MockFile`
- localStorage seed via `from_html_with_local_storage`
- localStorage seed via `from_html_with_url_and_local_storage`

Why these are stable mocks:

- they exist specifically to give tests deterministic setup and inspection points
- they are part of the package's practical value proposition
- they should remain documented with concrete examples

## Extended Browser-Like Surface

This section covers exposed behavior that is real and useful, but broader than the narrow `Harness` core contract.

Representative areas:

- `location`, `history`, and navigation object behavior beyond mock-page injection
- download capture flows tied to `Blob`, object URLs, and anchor activation
- assignable browser-like globals such as `navigator.clipboard` and `window.localStorage`
- `matchMedia` behavior beyond simple mock injection
- `cookieStore`
- `cacheStorage`
- `URL`, `URLSearchParams`, object URL behavior
- clipboard binary payload handling
- file-like APIs available from mocked file input state
- broader DOM element API surfaces, geometry reads, style reads, animation-like behavior
- canvas, media, worker, stream, encoding, and Intl surfaces already exposed to scripts
- `MockWindow` multi-page wrapper behavior

Why this category exists:

- these surfaces are useful and already public
- they often require more nuanced browser-compatibility judgment
- they should be maintained carefully without being mistaken for the crate's narrowest guaranteed core

Maintenance posture for this category:

- keep regression coverage when bugs are found
- document notable caveats
- avoid expanding promises casually

## Internal Only

Representative internal areas:

- `src/core_impl/**`
- `src/runtime_state.rs`
- `src/runtime_values.rs`
- `src/script_ast.rs`
- `src/selector.rs`
- DOM helper internals
- parser helper internals
- runtime execution helpers
- value-object internals
- internal storage keys and object metadata

Examples of internals that should not be treated as user contract:

- exact AST shapes
- exact internal runtime value representations
- specific helper-module boundaries
- exact file layout inside `src/core_impl`
- exact internal event or scheduler helper names

## Recommended Test Mapping

Each support level should map to tests differently.

Stable Core:

- direct contract tests
- regression tests when bugs are found
- README examples should remain valid
- the current minimal contract suite lives in `tests/contract_harness_core.rs`

Stable Test Mocks:

- direct contract tests for each mock family
- example-based tests where practical
- regression tests for failure injection and artifact capture
- the current minimal contract suite lives in `tests/contract_harness_core.rs`

Extended Browser-Like Surface:

- targeted regression tests
- reduced compatibility tests when practical
- explicit caveat documentation

Internal Only:

- unit tests and subsystem tests as needed
- no promise of user-visible stability

## File Pointers For Current Classification Work

Public API entry points are mainly defined across:

- `src/harness_api.rs`
- `src/core_impl/runtime/runtime_platform/bootstrap/environment_global_init.rs`
- `src/core_impl/runtime/runtime_platform/dom_actions/user_actions_forms.rs`
- `src/core_impl/runtime/runtime_platform/dom_actions/platform_mock_controls.rs`
- `src/core_impl/runtime/runtime_platform/dom_actions/trace_determinism_controls.rs`
- `src/core_impl/runtime/runtime_platform/dom_actions/timer_controls_execution.rs`
- `src/core_impl/runtime/runtime_platform/dom_actions/assertions_form_helpers.rs`
- `src/core_impl/runtime/runtime_platform/bootstrap/anchor_url_properties.rs`

These are the first files to revisit whenever the public surface changes.

## How To Use This Document

When adding a new public capability:

1. Decide which support level it belongs to before implementing it.
2. Decide which subsystem owns it before implementing it.
3. Update this matrix in the same change.
4. Add or update the corresponding tests.
5. Update the README if the capability belongs in `Stable Core` or `Stable Test Mocks`.

For subsystem placement, use `doc/subsystem-map.md`.

This is meant to keep the crate's public contract intentional instead of incidental.
