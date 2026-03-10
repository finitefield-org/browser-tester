# HTML Spec Conformance TODO

## Current Status

- `P0: Parsing, Tree Construction, and Serialization` is complete.
- `P1: Attribute Reflection, Global Attributes, Element Algorithms, Forms, Default Actions, and Events` is complete.
- `P2: Navigation, Loading, Media, and Rendering-Tied Behavior` is complete.
- `P3: WPT-Guided Exposed-Surface Interop Hardening` is complete.
- `P4.1: Harness-reduction inventory refresh and candidate ranking` is complete.
- `P4.2: Editing, selection, focus, and clipboard harness-reduction pass` is complete.
- `P1.1` through `P1.139` are complete.
- `P2.1` through `P2.13` are complete.
- `P3.1` through `P3.13` are complete.
- The latest full verification was `cargo test --lib` with `2534 passed, 0 failed`.
- No new test-only mock is currently required. If a future task adds one, document it in `README.md`.

## Recently Completed

- `P4.2: Editing, selection, focus, and clipboard harness-reduction pass`
  - added trusted `cut` default-action support for text controls so the harness now copies the selected text into the clipboard, removes the selected range from editable controls, and preserves focus/selection visibility through the same event path
  - locked reduced regressions for `cut` default action, `preventDefault()` plus event-local `clipboardData` override, and synthetic-dispatch no-op behavior while keeping the existing `copy` / `paste` contract green
  - verification:
  - `cargo test --lib dom_element_cut_event -- --nocapture`
  - `cargo test --lib dom_element_copy_event -- --nocapture`
  - `cargo test --lib dom_element_paste_event -- --nocapture`
  - `cargo test --lib dom_dispatch_paste_clipboard_data -- --nocapture`
  - `cargo test --lib dom_events_input_runtime -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P4.1: Harness-reduction inventory refresh and candidate ranking`
  - reviewed every `Harness reduction first` and `Browser-comparison first` family in `doc/p3-wpt-audit-inventory.md`
  - ranked the next reduced-WPT/browser-comparison candidates by payoff, determinism, and harness complexity, and chose the core `P4` execution order as editing/focus/clipboard, navigation/lifecycle, media/resource loading, downloads/object URLs, and canvas/image artifacts
  - explicitly deferred worker/message-loop reduction and CSSOM View/layout reduction behind the core `P4` queue rather than dropping them silently
  - verification:
  - documentation-only update; no code or test changes were required

- `P3.13: Post-audit closure pass and roadmap refresh`
  - closed the broad `P3` exposed-surface audit and refreshed the roadmap to reflect that the remaining high-value work is now concentrated in partially modeled harness behavior rather than another horizontal public-surface sweep
  - chose a new `P4` phase focused on harness reduction and selective reduced-WPT intake, and turned the next work into a smaller backlog aimed at navigation/loading, editing/selection, media/resource loading, downloads/object URLs, and canvas/image artifacts
  - verification:
  - documentation-only update; no code or test changes were required

- `P3.12: Intl, Encoding, Streams, and remaining non-HTML platform surface audit`
  - moved the exposed text codec surface onto branded prototype-backed constructors so `TextEncoder`, `TextDecoder`, `TextEncoderStream`, and `TextDecoderStream` now expose stable constructor/prototype linkage, `Symbol.toStringTag`, and `Object.prototype.toString` branding while preserving the existing deterministic codec behavior
  - hardened receiver validation for text codec methods and getters, and locked reduced regressions for prototype descriptors, extracted calls, readonly surface, and stream wrapper branding across the existing codec and builtins suites
  - verified with:
  - `cargo test --lib webapi_text_encoder -- --nocapture`
  - `cargo test --lib webapi_text_decoder -- --nocapture`
  - `cargo test --lib webapi_text_encoder_stream -- --nocapture`
  - `cargo test --lib webapi_text_decoder_stream -- --nocapture`
  - `cargo test --lib issue_93_text_codec_web_compat -- --nocapture`
  - `cargo test --lib webapi_data_builtins -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P3.11: URL, URLSearchParams, fetch-adjacent, and storage/cache exposed-surface audit`
  - added branded constructor/prototype surfaces for `Storage`, `CookieStore`, `CacheStorage`, and `Cache` so `localStorage`, `cookieStore`, `caches`, and named caches now expose stable constructor identity, prototype linkage, secure-context visibility, and `Object.prototype.toString` branding without regressing the existing own-method surface
  - locked reduced regressions for secure/insecure constructor visibility, illegal constructor behavior, and cache/cookie/storage instance branding while re-running the existing URL/storage/cache suites to keep the broader audit slice green
  - verified with:
  - `cargo test --lib storage_cache_and_cookie_store_constructor_surface_and_branding_work -- --nocapture`
  - `cargo test --lib secure_storage_like_constructors_are_hidden_in_insecure_contexts_work -- --nocapture`
  - `cargo test --lib webapi_data_builtins -- --nocapture`
  - `cargo test --lib collections_url_typed_arrays -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P3.10: Canvas, image pipeline, and deterministic artifact interop audit`
  - exposed `ImageBitmap` as a branded constructor/prototype surface so `createImageBitmap(...)` results now have stable constructor identity, prototype linkage, readonly `width`/`height` accessors, branded `Object.prototype.toString`, and receiver-aware `close()`
  - locked reduced regressions for extracted getter/method calls, incompatible receiver errors, shadow/delete restore, and object-copy behavior on `ImageBitmap` instances while keeping the existing deterministic canvas/image pipeline model
  - verified with:
  - `cargo test --lib create_image_bitmap_exposes_image_bitmap_constructor_surface_and_branding -- --nocapture`
  - `cargo test --lib image_bitmap_close_and_reflective_surface_support_extracted_calls_and_shadow_restore -- --nocapture`
  - `cargo test --lib dom_input_file_create_image_bitmap -- --nocapture`
  - `cargo test --lib dom_canvas_element -- --nocapture`
  - `cargo test --lib issue_96_canvas_to_blob_clipboard_flow -- --nocapture`
  - `cargo test --lib issue_121_127_finitefield_site_regressions -- --nocapture`
  - `cargo test --lib dom_image_data -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P3.9: Worker, postMessage, structured-clone, and blob-URL interop audit`
  - aligned `Worker` with a branded `Worker.prototype` surface so constructor/prototype chaining and `Object.prototype.toString` results are stable while keeping direct instance methods working through the existing bound path
  - made `worker.postMessage(...)` and worker-global `postMessage(...)` structured-clone their payloads before delivery, so worker message handlers no longer observe caller-owned object mutations through shared references
  - verified with:
  - `cargo test --lib worker_constructor_surface_and_prototype_branding_work -- --nocapture`
  - `cargo test --lib worker_post_message_structured_clones_payloads_work -- --nocapture`
  - `cargo test --lib issue_102_worker_regex_exec -- --nocapture`
  - `cargo test --lib issue_121_127_finitefield_site_regressions -- --nocapture`
  - `cargo test --lib dom_error_event -- --nocapture`
  - `cargo test --lib webapi_data_builtins -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P3.8: Clipboard, DataTransfer, download, and object-URL interop audit`
  - aligned `DataTransfer` and event-local `clipboardData` with a branded `DataTransfer.prototype` surface so constructor/prototype chaining, method metadata, and `Object.prototype.toString` results are stable across direct construction and trusted clipboard events
  - updated reduced regressions to treat `delete` on clipboard/data-transfer method shadows as restoring prototype-backed methods instead of collapsing to `undefined`, matching the new prototype-linked surface
  - verified with:
  - `cargo test --lib data_transfer_constructor_surface_and_prototype_branding_work -- --nocapture`
  - `cargo test --lib element_copy_event_clipboard_data_uses_data_transfer_prototype_and_branding_work -- --nocapture`
  - `cargo test --lib dom_data_transfer -- --nocapture`
  - `cargo test --lib dom_element_copy_event -- --nocapture`
  - `cargo test --lib dom_dispatch_paste_clipboard_data -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P3.7: Media/resource element and wrapper interop audit`
  - audited the exposed media/resource wrapper surface and locked `TextTrack` constructor/prototype/instance reflective behavior so wrapper branding, descriptor visibility, shadow/delete restore, and object-copy paths are fixed by reduced regressions
  - confirmed the existing generic object machinery already preserves `TextTrack` instance accessor fallback and expando semantics, so no runtime change was required for this audit slice
  - verified with:
  - `cargo test --lib text_track_reflective_surface_and_shadow_restore_work -- --nocapture`
  - `cargo test --lib dom_track_element -- --nocapture`
  - `cargo test --lib language_core_expressions -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P3.6: Web Animations and rendering-tied object-surface audit`
  - aligned the exposed `Animation` object surface so method properties such as `play`, `pause`, `finish`, `cancel`, `reverse`, `updatePlaybackRate`, `commitStyles`, and `persist` are non-enumerable while keeping raw getter identity and callable behavior stable
  - locked reduced regressions for `Object.keys(...)`, `Reflect.ownKeys(...)`, `Object.assign(...)`, object spread, and descriptor visibility on `element.animate(...)` results without changing the existing deterministic animation state model
  - verified with:
  - `cargo test --lib dom_element_get_animations_method -- --nocapture`
  - `cargo test --lib dom_navigation_dialog -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P3.5: CSSOM View, scroll, geometry, and computed-style interop audit`
  - upgraded `getBoundingClientRect()` to return branded readonly `DOMRect`-like objects and `getClientRects()` to return `DOMRectList`-like arrays with `item(...)` callable parity, so geometry objects expose the expected object/callable surface instead of plain ad hoc records
  - locked reduced regressions for readonly descriptor behavior, extracted `item.call(...)`, and branded `Object.prototype.toString` results without regressing existing deterministic geometry behavior
  - verified with:
  - `cargo test --lib dom_element_get_bounding_client_rect_method -- --nocapture`
  - `cargo test --lib dom_element_get_client_rects_method -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P3.4: Navigation, history, and document lifecycle interop audit`
  - aligned the exposed `navigation.currententrychange` surface so legacy navigation paths such as `history.pushState(...)`, `history.replaceState(...)`, `history.back()/forward()/go(...)`, and hash-only `location` navigations notify the `navigation` object when the current entry changes
  - locked reduced regressions so `currententrychange` handlers observe updated `navigation.currentEntry` and `history.state` on both state updates and same-document traversals without regressing existing lifecycle/hashchange/popstate behavior
  - verified with:
  - `cargo test --lib navigation_currententrychange_fires_for_history_push_replace_and_traverse_work -- --nocapture`
  - `cargo test --lib navigation_currententrychange_fires_for_hash_only_location_navigation_work -- --nocapture`
  - `cargo test --lib dom_navigation_interface -- --nocapture`
  - `cargo fmt`
  - `cargo test --lib`

- `P3.3: Forms, focus, selection, and default-action interop audit`
  - aligned `HTMLElement.click()` so script-triggered clicks dispatch untrusted `click` events while still running activation/default-action behavior for submit controls, checkboxes, and label-driven activation paths
  - locked reduced regressions for script-side `.click()` submitter plumbing and checkbox activation so public form/default-action behavior stays browser-like without regressing trusted harness click paths
  - verified with:
  - `cargo test --lib script_submit_click_is_untrusted_but_still_submits_with_submitter_work`
  - `cargo test --lib script_checkbox_click_is_untrusted_but_still_runs_activation_behavior_work`
  - `cargo fmt`
  - `cargo test --lib`

- `P3.2: DOM/HTML residual exposed-surface audit`
  - exposed receiver-aware raw getter call surfaces for residual `Node` / `Element` / `ParentNode` / `ChildNode` methods so extracted calls such as `const append = fragment.append; append.call(fragment, ...)` follow the same runtime behavior as direct DOM method entrypoints
  - locked shadow/delete/restore behavior for those node-method raw getters and added reduced regressions for extracted-call and reflective-surface parity
  - verified with:
  - `cargo test --lib dom_node_interface`
  - `cargo fmt`
  - `cargo test --lib`

- `P3.1: Exposed-surface inventory and WPT mapping kickoff`
  - inventoried the currently exposed public API surface by spec family and mapped each group to feasible WPT directories or browser-comparison targets
  - recorded which surfaces are ready for direct reduced-WPT auditing and which still need harness reduction or browser-comparison-first reduction before importing regressions
  - wrote the working inventory in `doc/p3-wpt-audit-inventory.md` and linked the roadmap to that document
  - verification:
  - documentation-only update; no code or test changes were required

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

- [ ] `P4.3: Navigation/loading lifecycle harness-reduction pass`
  - narrow the remaining gap between harness-backed lifecycle transitions and browser navigation state machines
  - focus on replacement, reload, traversal, and lifecycle visibility paths that still require reduction before wider WPT intake

## P3 Backlog

- [x] `P3.1: Exposed-surface inventory and WPT mapping kickoff`
  - inventory the currently public API surface by spec family and map it to feasible WPT directories or browser-comparison targets
  - record which surfaces are already deterministic enough for direct audit and which still require harness reduction before importing regressions
  - output: `doc/p3-wpt-audit-inventory.md`

- [x] `P3.2: DOM/HTML residual exposed-surface audit`
  - audit the already exposed DOM parsing, mutation, collection, reflection, and element-algorithm surfaces for residual browser mismatches
  - prioritize gaps that show up as descriptor, prototype, liveness, or default-action inconsistencies

- [x] `P3.3: Forms, focus, selection, and default-action interop audit`
  - audit the already exposed form submission, validation, reset, label activation, focus, blur, selection, and clipboard-triggered default actions
  - import reduced regressions for remaining ordering, cancellation, and dirty-state mismatches

- [x] `P3.4: Navigation, history, and document lifecycle interop audit`
  - audit current `location`, `history`, `navigation`, hashchange/popstate, lifecycle, and mock-page restoration behavior against browser expectations
  - reduce any remaining ordering or state-visibility gaps to deterministic harness regressions

- [x] `P3.5: CSSOM View, scroll, geometry, and computed-style interop audit`
  - audit the already exposed scroll aliases, geometry APIs, client/offset/scroll metrics, and computed-style reads
  - focus on readonly/alias behavior, object-surface parity, and event/value ordering

- [x] `P3.6: Web Animations and rendering-tied object-surface audit`
  - audit `element.animate(...)`, `Animation`, `requestAnimationFrame`, and related rendering-tied surfaces that are already exposed
  - keep scope limited to current public APIs rather than broad rendering or painting behavior

- [x] `P3.7: Media/resource element and wrapper interop audit`
  - audit the already exposed audio/video/source/img/track/object/embed/iframe behavior, including wrapper identity and current-state restoration
  - focus on source selection, event ordering, reflective surface, and cached-wrapper parity

- [x] `P3.8: Clipboard, DataTransfer, download, and object-URL interop audit`
  - audit copy/paste, drag-and-drop-facing surfaces, download artifacts, blob/object URL lifetime, and default-action boundaries already modeled by the harness
  - reduce remaining event-local versus global-state mismatches to deterministic tests

- [x] `P3.9: Worker, postMessage, structured-clone, and blob-URL interop audit`
  - audit the existing worker construction, message delivery, structured clone, and blob URL integration surfaces
  - focus on ordering, transfer behavior, error reporting, and deterministic lifetime semantics

- [x] `P3.10: Canvas, image pipeline, and deterministic artifact interop audit`
  - audit the already exposed canvas, image bitmap, toBlob/toDataURL, and clipboard/image pipeline behavior
  - prioritize observable output shape, callback ordering, and object-surface parity

- [x] `P3.11: URL, URLSearchParams, fetch-adjacent, and storage/cache exposed-surface audit`
  - audit the already exposed URL, URLSearchParams, fetch-adjacent mocks, CacheStorage/Cache, cookieStore, and storage-facing behavior
  - focus on descriptor/callable surface, live sync, and remaining parsing/serialization residuals

- [x] `P3.12: Intl, Encoding, Streams, and remaining non-HTML platform surface audit`
  - audit the already exposed Intl, TextEncoder/TextDecoder, streams, iterators, and other non-HTML builtins that are public in the crate
  - prioritize receiver validation, branding, readonly surface, and argument-coercion mismatches

- [x] `P3.13: Post-audit closure pass and roadmap refresh`
  - summarize the gaps closed during `P3`, collapse completed audit tracks, and identify whether any new roadmap phase is justified
  - convert any remaining uncovered public-surface mismatches into a smaller follow-up backlog

## P4 Backlog

- [x] `P4.1: Harness-reduction inventory refresh and candidate ranking`
  - review the `Harness reduction first` and `Browser-comparison first` entries in `doc/p3-wpt-audit-inventory.md`
  - rank the next reduced-WPT candidates by payoff, determinism, and harness complexity before touching implementation

- [x] `P4.2: Editing, selection, focus, and clipboard harness-reduction pass`
  - tighten the deterministic editing model around text controls, selection mutation, focus transfer, and clipboard default actions where browser-native editing semantics still leak through
  - reduce the next high-value WPT/browser-comparison cases into stable in-repo regressions

- [ ] `P4.3: Navigation/loading lifecycle harness-reduction pass`
  - narrow the remaining gap between harness-backed lifecycle transitions and browser navigation state machines
  - focus on replacement, reload, traversal, and lifecycle visibility paths that still require reduction before wider WPT intake

- [ ] `P4.4: Media/resource loading and candidate-selection harness-reduction pass`
  - reduce the partial media/resource model into a smaller deterministic contract suitable for broader reduced-WPT coverage
  - focus on source selection, readiness/network state, wrapper persistence, and related event ordering

- [ ] `P4.5: Download, object-URL, and artifact-capture browser-comparison reduction`
  - use browser comparison to define the stable contract for harness-captured downloads and blob/object URL interactions
  - convert the chosen contract into deterministic reduced regressions without broadening the harness unnecessarily

- [ ] `P4.6: Canvas/image-pipeline artifact reduction pass`
  - reduce mock-shaped canvas/image output behavior into a clearer deterministic contract for image bitmap, toBlob/toDataURL, and clipboard/image artifact flows
  - prefer a small number of high-confidence browser-comparison-backed regressions

- [ ] `P4.7: Post-P4 closure pass and roadmap refresh`
  - summarize which partially modeled surfaces were made intake-ready during `P4`
  - decide whether the next step is a small rolling maintenance backlog or a justified new roadmap phase

## Deferred After Core P4 Queue

- [ ] `Deferred: Worker/message-loop and structured-clone harness reduction`
  - revisit reduced-WPT intake for worker lifecycle, message delivery, and structured-clone semantics after the higher-payoff `P4` queue is complete
  - treat this as a rolling backlog item unless a new exposed-surface regression makes it urgent earlier

- [ ] `Deferred: CSSOM View/layout and computed-style harness reduction`
  - revisit reduced-WPT intake for layout-derived metrics and computed-style behavior after defining a narrower deterministic layout contract
  - keep this out of the core `P4` queue until a smaller high-confidence reduction target is chosen

## Verification Rule

- Every roadmap task should end with:
  - targeted regressions for the touched behavior
  - relevant focused suites
  - `cargo fmt`
  - `cargo test --lib`

- If a task is documentation-only, note explicitly that no tests were run.

- If a task introduces a new test-only mock, update `README.md` in the same change.
