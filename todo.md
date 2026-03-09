# HTML Spec Conformance TODO

## Current Status

- `P0: Parsing, Tree Construction, and Serialization` is complete.
- `P1: Attribute Reflection, Global Attributes, Element Algorithms, Forms, Default Actions, and Events` is complete.
- `P2: Navigation, Loading, Media, and Rendering-Tied Behavior` is complete.
- `P1.1` through `P1.139` are complete.
- `P2.1` through `P2.13` are complete.
- The latest full verification was `cargo test --lib` with `2509 passed, 0 failed`.
- No new test-only mock is currently required. If a future task adds one, document it in `README.md`.

## Recently Completed

- `Post-P2: Refresh the conformance roadmap and define the next exposed-surface backlog`
  - reviewed the now-complete `P0` / `P1` / `P2` roadmap against the current public API surface and recent regression patterns
  - decided that the next step is a new `P3` phase run as a WPT-guided audit over already exposed APIs, with every finding reduced to deterministic in-repo regressions before or together with a fix
  - split the next backlog into exposed-surface audit passes covering DOM/HTML residuals, forms, navigation, geometry/style, animation, media, clipboard/download, workers, canvas, URL, and remaining non-HTML platform APIs
  - verification:
  - documentation-only update; no code or test changes were required

- `P2.13: Animation and rendering-tied exposed API sweep`
  - replaced the exposed `Animation` object method surface returned by `element.animate(...)` with receiver-aware builtins so extracted call paths, callable metadata, and incompatible-receiver behavior are deterministic
  - added minimal animation state transitions for `cancel`, `finish`, `pause`, `play`, `reverse`, and `updatePlaybackRate` on the already exposed animation objects without expanding into broader rendering or timeline behavior
  - verified with:
  - `cargo test --lib element_animate_methods_are_receiver_aware_and_update_state -- --nocapture`
  - `cargo test --lib element_animate_methods_reject_incompatible_receivers -- --nocapture`
  - `cargo test --lib dom_navigation_dialog -- --nocapture`
  - `cargo test --lib dom_element_get_animations_method -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P2.11: Scroll, viewport, and geometry deterministic sweep`
  - finished the exposed window/document scroll position surface by making `window.scrollX`, `window.scrollY`, `window.pageXOffset`, and `window.pageYOffset` readonly aliases backed by the deterministic document scroll state
  - synchronized those aliases before `scroll` / `scrollend` dispatch on both window-level and element-level scroll entrypoints, and locked their relationship to `getBoundingClientRect()` under the current deterministic geometry model
  - verified with:
  - `cargo test --lib window_forms_trace -- --nocapture`
  - `cargo test --lib dom_navigation_dialog -- --nocapture`
  - `cargo test --lib dom_element_get_bounding_client_rect_method -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P2.12: Computed-style and layout-derived property sweep`
  - aligned layout-derived node properties such as `clientWidth`, `clientHeight`, `clientLeft`, `clientTop`, `currentCSSZoom`, `offset*`, and `scroll*` so explicit own data/accessor shadows are honored consistently on the generic node path and the parser-specialized DOM fast path
  - extended the currently exposed `getComputedStyle(...)` object surface with deterministic `item(...)` behavior while locking readonly aliases, live mutation visibility, and computed-style property-path expectations under the crate's layout model
  - verified with:
  - `cargo test --lib window_get_computed_style -- --nocapture`
  - `cargo test --lib dom_element_client_width_property -- --nocapture`
  - `cargo test --lib dom_element_client_height_property -- --nocapture`
  - `cargo test --lib dom_element_get_bounding_client_rect_method -- --nocapture`
  - `cargo test --lib dom_element_get_client_rects_method -- --nocapture`
  - `cargo test --lib language_core_expressions -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P2.10: Resource element load/error/currentSrc parity sweep`
  - added deterministic `img.complete` / `naturalWidth` / `naturalHeight` load-facing state tied to the currently resolved resource candidate, and locked picture/source-driven `currentSrc` transitions around supported, filtered, and empty candidates
  - locked manual `load` / `error` event-handler surface for `iframe`, `object`, `embed`, and `track` so their currently exposed resource-facing properties stay aligned while using the existing dispatch harness
  - verified with:
  - `cargo test --lib dom_img_element -- --nocapture`
  - `cargo test --lib dom_iframe_element -- --nocapture`
  - `cargo test --lib dom_object_element -- --nocapture`
  - `cargo test --lib dom_embed_element -- --nocapture`
  - `cargo test --lib dom_track_element -- --nocapture`
  - `cargo test --lib dom_source_element -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P2.9: TextTrackList/TimeRanges/TextTrack surface breadth sweep`
  - added branded `TextTrack` wrapper objects with stable constructor/prototype exposure, receiver-aware accessors, and live cached identity across `HTMLTrackElement.track` and `TextTrackList` indexed/item/iterator paths
  - kept `TextTrackList` / `TimeRanges` wrapper behavior aligned while locking raw getter, descriptor, iterator, and object-surface expectations for the exposed media wrapper APIs already implemented in-repo
  - verified with:
  - `cargo test --lib html_track_element_track_returns_stable_text_track_wrappers_work -- --nocapture`
  - `cargo test --lib text_track_constructor_surface_and_prototype_accessors_work -- --nocapture`
  - `cargo test --lib dom_track_element -- --nocapture`
  - `cargo test --lib dom_video_element -- --nocapture`
  - `cargo test --lib dom_audio_element -- --nocapture`
  - `cargo test --lib language_core_expressions -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P2.8: HTMLMediaElement playback algorithm and promise/event ordering sweep`
  - added trusted `play` / `playing` / `pause` / `emptied` / `loadstart` / `seeking` / `seeked` / `ratechange` dispatch for the existing `HTMLMediaElement` method and setter surface, while keeping play-promise resolution after synchronous media events
  - verified with:
  - `cargo test --lib video_media_methods_dispatch_trusted_events_and_resolve_play_promise_after_sync_events_work -- --nocapture`
  - `cargo test --lib video_media_methods_update_cached_time_ranges_and_receiver_parity_work -- --nocapture`
  - `cargo test --lib dom_audio_element -- --nocapture`
  - `cargo test --lib dom_video_element -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P2.7: HTMLMediaElement source selection and load-state residual sweep`
  - locked remaining source-selection and load-state deltas around media-query-driven `<source>` candidates so direct `src` precedence, nested fallback, and `currentSrc` / `networkState` / `readyState` stay aligned across viewport and color-scheme changes
  - verified with:
  - `cargo test --lib audio_media_query_source_selection_and_load_state_follow_direct_src_precedence_work -- --nocapture`
  - `cargo test --lib video_media_query_source_selection_and_load_state_follow_nested_candidate_changes_work -- --nocapture`
  - `cargo test --lib dom_audio_element -- --nocapture`
  - `cargo test --lib dom_video_element -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P2.6: Object URL lifetime and artifact integration sweep`
  - locked object URL lifetime around the current consumers so revoked blob URLs stop producing captured downloads while independently created blob URLs remain usable
  - locked worker integration so blob URLs are resolved at construction time, survive post-construction revocation, and throw deterministic not-found errors once revoked before construction
  - verified with:
  - `cargo test --lib issue_74_download_artifacts -- --nocapture`
  - `cargo test --lib issue_102_worker_regex_exec -- --nocapture`
  - `cargo test --lib dom_error_event -- --nocapture`
  - `cargo test --lib webapi_url_create_object_url -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P2.5: Download-triggering default-action and captured-artifact sweep`
  - normalized captured download artifacts so empty `download` filenames are represented as unspecified while keeping blob/object-URL download capture deterministic
  - locked `_blank` download handling and `click.preventDefault()` suppression so captured artifacts stay aligned with default-action boundaries and no navigation leaks through
  - verified with:
  - `cargo test --lib issue_74_download_artifacts -- --nocapture`
  - `cargo test --lib dom_navigation_dialog -- --nocapture`
  - `cargo test --lib dom_area_element -- --nocapture`
  - `cargo test --lib webapi_url_create_object_url -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P2.4: Navigation API and location/history logging parity sweep`
  - aligned `navigation.navigate(...)` mock-page commit order with `location.assign(...)` / `replace(...)` so source-document `pagehide` still observes the source URL, history state, and current entry before the target entry is committed
  - aligned `navigation.reload({ state })` with reload logging and mock-page lifecycle so the reloaded document sees the final overridden history state during `pageshow`
  - verified with:
  - `cargo test --lib navigation_navigate_mock_page_preserves_source_state_until_pagehide_and_logs_replace_work -- --nocapture`
  - `cargo test --lib navigation_reload_state_override_commits_before_mock_pageshow_and_logs_reload_work -- --nocapture`
  - `cargo test --lib dom_navigation_interface -- --nocapture`
  - `cargo test --lib dom_navigation_dialog -- --nocapture`
  - `cargo test --lib dom_body_element -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P2.3: Cross-document mock-page navigation and history restoration sweep`
  - fixed mock-page cross-document navigation so source-document `pagehide` still observes the source URL, history state, and current history entry before the new entry is committed
  - locked back/forward restoration so the restored mock page sees the correct `document` / `location` / `navigation.currentEntry` state and stable entry identity after history traversal
  - verified with:
  - `cargo test --lib mock_page_assign_preserves_source_document_state_until_pagehide_and_syncs_new_document_work -- --nocapture`
  - `cargo test --lib cross_document_history_back_restores_mock_page_url_state_and_entry_identity_work -- --nocapture`
  - `cargo test --lib dom_navigation_dialog -- --nocapture`
  - `cargo test --lib dom_navigation_interface -- --nocapture`
  - `cargo test --lib dom_body_element -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P2.2: Same-document navigation lifecycle ordering sweep`
  - locked same-document history traversal so `popstate` fires before `hashchange` while `pagehide`, `pageshow`, and `visibilitychange` stay silent
  - verified that same-URL writes and out-of-bounds history no-op paths do not leak lifecycle events and preserve the current document phase
  - verified with:
  - `cargo test --lib same_document_history_traversal_orders_popstate_before_hashchange_without_lifecycle_work -- --nocapture`
  - `cargo test --lib same_document_noop_navigation_paths_do_not_emit_lifecycle_events_work -- --nocapture`
  - `cargo test --lib dom_navigation_dialog -- --nocapture`
  - `cargo test --lib dom_hash_change_event -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P2.1: Navigation/loading harness-surface audit kickoff`
  - inventoried the currently exposed harness-backed navigation/loading surface around `location`, `history`, `navigation`, mock-page swaps, document lifecycle, and media-adjacent loading state
  - chose `pagehide` / `pageshow` dispatch across mock-page navigation and reload as the first concrete `P2` gap, then implemented and locked it with deterministic regressions
  - verified with:
  - `cargo test --lib dom_navigation_dialog -- --nocapture`
  - `cargo test --lib dom_body_element -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P1.139: Exposed element-algorithm audit beyond shared reflection`
  - locked `details` / `summary` default-action boundaries so `toggle` stays local to the owning `<details>` element and `summary` click `preventDefault()` suppresses the open-state transition
  - locked `dialog` `beforetoggle` cancellation so prevented open/close transitions do not leak `toggle` / `close` side effects before the element state actually changes
  - verified with:
  - `cargo test --lib dom_details_element -- --nocapture`
  - `cargo test --lib dom_dialog_element -- --nocapture`
  - `cargo test --lib dom_summary_element -- --nocapture`
  - `cargo test --lib dom_navigation_dialog -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

## Next Task

- [ ] `P3.1: Exposed-surface inventory and WPT mapping kickoff`
  - inventory the currently public API surface by spec family and map it to feasible WPT directories or browser-comparison targets
  - record which surfaces are already deterministic enough for direct audit and which still require harness reduction before importing regressions

## P3 Backlog

- [ ] `P3.1: Exposed-surface inventory and WPT mapping kickoff`
  - inventory the currently public API surface by spec family and map it to feasible WPT directories or browser-comparison targets
  - record which surfaces are already deterministic enough for direct audit and which still require harness reduction before importing regressions

- [ ] `P3.2: DOM/HTML residual exposed-surface audit`
  - audit the already exposed DOM parsing, mutation, collection, reflection, and element-algorithm surfaces for residual browser mismatches
  - prioritize gaps that show up as descriptor, prototype, liveness, or default-action inconsistencies

- [ ] `P3.3: Forms, focus, selection, and default-action interop audit`
  - audit the already exposed form submission, validation, reset, label activation, focus, blur, selection, and clipboard-triggered default actions
  - import reduced regressions for remaining ordering, cancellation, and dirty-state mismatches

- [ ] `P3.4: Navigation, history, and document lifecycle interop audit`
  - audit current `location`, `history`, `navigation`, hashchange/popstate, lifecycle, and mock-page restoration behavior against browser expectations
  - reduce any remaining ordering or state-visibility gaps to deterministic harness regressions

- [ ] `P3.5: CSSOM View, scroll, geometry, and computed-style interop audit`
  - audit the already exposed scroll aliases, geometry APIs, client/offset/scroll metrics, and computed-style reads
  - focus on readonly/alias behavior, object-surface parity, and event/value ordering

- [ ] `P3.6: Web Animations and rendering-tied object-surface audit`
  - audit `element.animate(...)`, `Animation`, `requestAnimationFrame`, and related rendering-tied surfaces that are already exposed
  - keep scope limited to current public APIs rather than broad rendering or painting behavior

- [ ] `P3.7: Media/resource element and wrapper interop audit`
  - audit the already exposed audio/video/source/img/track/object/embed/iframe behavior, including wrapper identity and current-state restoration
  - focus on source selection, event ordering, reflective surface, and cached-wrapper parity

- [ ] `P3.8: Clipboard, DataTransfer, download, and object-URL interop audit`
  - audit copy/paste, drag-and-drop-facing surfaces, download artifacts, blob/object URL lifetime, and default-action boundaries already modeled by the harness
  - reduce remaining event-local versus global-state mismatches to deterministic tests

- [ ] `P3.9: Worker, postMessage, structured-clone, and blob-URL interop audit`
  - audit the existing worker construction, message delivery, structured clone, and blob URL integration surfaces
  - focus on ordering, transfer behavior, error reporting, and deterministic lifetime semantics

- [ ] `P3.10: Canvas, image pipeline, and deterministic artifact interop audit`
  - audit the already exposed canvas, image bitmap, toBlob/toDataURL, and clipboard/image pipeline behavior
  - prioritize observable output shape, callback ordering, and object-surface parity

- [ ] `P3.11: URL, URLSearchParams, fetch-adjacent, and storage/cache exposed-surface audit`
  - audit the already exposed URL, URLSearchParams, fetch-adjacent mocks, CacheStorage/Cache, cookieStore, and storage-facing behavior
  - focus on descriptor/callable surface, live sync, and remaining parsing/serialization residuals

- [ ] `P3.12: Intl, Encoding, Streams, and remaining non-HTML platform surface audit`
  - audit the already exposed Intl, TextEncoder/TextDecoder, streams, iterators, and other non-HTML builtins that are public in the crate
  - prioritize receiver validation, branding, readonly surface, and argument-coercion mismatches

- [ ] `P3.13: Post-audit closure pass and roadmap refresh`
  - summarize the gaps closed during `P3`, collapse completed audit tracks, and identify whether any new roadmap phase is justified
  - convert any remaining uncovered public-surface mismatches into a smaller follow-up backlog

## Verification Rule

- Every roadmap task should end with:
  - targeted regressions for the touched behavior
  - relevant focused suites
  - `cargo fmt`
  - `cargo test --lib`

- If a task is documentation-only, note explicitly that no tests were run.

- If a task introduces a new test-only mock, update `README.md` in the same change.
