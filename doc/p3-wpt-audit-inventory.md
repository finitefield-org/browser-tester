# P3 WPT Audit Inventory

## Purpose

This document is the working inventory for `P3: WPT-Guided Exposed-Surface Interop Hardening`.

It does two things:

- lists the public API surface that is already exposed by `browser_tester`
- maps each surface to feasible Web Platform Test directories or browser-comparison targets

This is not a promise to import WPT wholesale. It is a planning document for deciding which areas are ready for direct reduced-WPT auditing and which still need harness reduction first.

## Audit Readiness Labels

- `Direct reduced-WPT`: the exposed surface is deterministic enough that reduced WPT cases can usually be ported directly into `src/tests`
- `Harness reduction first`: the surface is public, but current mock or harness behavior still needs a small reduction layer before WPT-style cases can be imported safely
- `Browser-comparison first`: use real browser behavior and selected WPTs to identify expectations, then write reduced deterministic tests instead of importing directly

## Surface Inventory

| Surface family | Representative repository coverage | Feasible WPT or browser-comparison targets | Audit readiness | Notes |
| --- | --- | --- | --- | --- |
| HTML parsing and tree construction | parser fixtures, `dom_*` element tests, `innerHTML` / `outerHTML` / `insertAdjacentHTML` coverage | `html/syntax/parsing/`, `html/semantics/scripting-1/the-template-element/`, `domparsing/` | `Direct reduced-WPT` | best target for reduced parsing fixtures and serialization round-trips |
| DOM parsing and serialization APIs | `Document.parseHTMLUnsafe`, `setHTMLUnsafe`, fragment parsing and serialization tests | `domparsing/`, `html/webappapis/dynamic-markup-insertion/` | `Direct reduced-WPT` | already deterministic and spec-anchored |
| DOM mutation, traversal, collections, and reflectors | broad `dom_element_*`, `dom_named_node_map`, `dom_selection_interface`, `language_core_expressions` coverage | `dom/nodes/`, `dom/lists/`, `dom/events/`, `html/dom/`, selected `html/semantics/*` | `Direct reduced-WPT` | main residual risk is prototype/descriptor/liveness parity |
| HTML element algorithms and local default actions | `details`, `summary`, `dialog`, hyperlink, media, form-associated element tests | `html/semantics/interactive-elements/`, `html/semantics/links/`, `html/semantics/forms/` | `Direct reduced-WPT` | stay scoped to algorithms already surfaced by the crate |
| Forms, validation, focus, selection, and trusted user interaction | `dom_form_element`, `dom_events_input_runtime`, `dom_label_element`, `dom_document_active_element_property`, `dom_element_copy_event`, `dom_element_paste_event` | `html/semantics/forms/`, `uievents/`, `selection/`, `clipboard-apis/` | `Harness reduction first` | many cases are deterministic already, but some WPTs assume browser-native editing and focus behavior not modeled one-to-one |
| Navigation, history, location, and document lifecycle | `dom_navigation_dialog`, `dom_navigation_interface`, `dom_hash_change_event`, `dom_body_element` | `html/browsers/history/the-history-interface/`, `html/browsers/history/the-location-interface/`, `navigation-api/` | `Harness reduction first` | mock-page lifecycle is exposed, but direct WPT import needs a smaller mapping from harness operations to browser navigation states |
| Downloads, artifacts, and object URL default actions | `issue_74_download_artifacts`, `webapi_url_create_object_url`, area/anchor download coverage | selected `FileAPI/BlobURL/`, `html/semantics/links/`, browser-comparison for download flows | `Browser-comparison first` | download capture is harness-specific, so behavior should be reduced from browser expectations rather than copied directly |
| URL, URLSearchParams, and URL-backed reflection | `collections_url_typed_arrays`, reflection suites, media/resource URL tests | `url/`, `urlpattern/` where relevant, selected `html/semantics/links/` | `Direct reduced-WPT` | strong candidate for systematic reduced-WPT import |
| Worker, postMessage, structured clone, and blob URL integration | worker regression suites, structured-clone tests, object URL worker coverage | `workers/`, `html/webappapis/structured-clone/`, `html/webappapis/messaging/`, `FileAPI/BlobURL/` | `Harness reduction first` | worker lifecycle is deterministic, but direct import needs a clear message-loop reduction layer |
| Media elements, source selection, text tracks, and time ranges | `dom_audio_element`, `dom_video_element`, `dom_track_element`, `dom_source_element` | `html/semantics/embedded-content/media-elements/`, `html/semantics/embedded-content/the-track-element/`, selected `html/semantics/embedded-content/the-img-element/` | `Browser-comparison first` | current media model is intentionally partial; use browser expectations to drive reduced deterministic cases |
| Resource elements (`img`, `iframe`, `object`, `embed`, `track`) | `dom_img_element`, `dom_iframe_element`, `dom_object_element`, `dom_embed_element`, `dom_track_element` | `html/semantics/embedded-content/the-img-element/`, `html/semantics/embedded-content/the-iframe-element/`, related embedded-content directories | `Direct reduced-WPT` | good target for `currentSrc`, event surface, and reflective state audits |
| CSSOM View, scroll, geometry, and computed style | `window_get_computed_style`, `dom_element_client_*`, `dom_element_get_bounding_client_rect_method`, `window_forms_trace` | `css/cssom-view/`, `css/cssom/` | `Harness reduction first` | many values are deterministic, but some WPTs assume full layout/painting not present here |
| Web Animations and animation-frame timing | `dom_element_get_animations_method`, animation cases in `dom_navigation_dialog`, `issue_86_request_animation_frame_promise_callback` | `web-animations/`, `html/webappapis/animation-frames/` | `Direct reduced-WPT` | scope stays limited to currently exposed `Animation` and `requestAnimationFrame` behavior |
| Canvas, image pipeline, and artifact-producing graphics APIs | `dom_canvas_element`, `dom_canvas_rendering_context_2d`, `issue_96_canvas_to_blob_clipboard_flow`, input-file image tests | `html/canvas/`, selected image bitmap and clipboard browser-comparison targets | `Browser-comparison first` | deterministic mocks exist, but the surface is mock-shaped enough that direct import should be selective |
| Encoding, streams, iterators, and async iteration | `webapi_text_encoder*`, `webapi_text_decoder*`, `async_iterator_helpers`, stream-facing tests | `encoding/`, `streams/` | `Direct reduced-WPT` | strong candidate for reduced-WPT parity cases |
| Intl and remaining non-HTML platform builtins | Intl suites, `webapi_data_builtins`, number/date/string/regexp platform coverage | `intl402/`, selective `ecmascript`-adjacent browser-comparison targets | `Direct reduced-WPT` | keep focus on surfaces already exposed through the harness/runtime, not on full JS engine parity |

## First P3 Audit Order

Start with surfaces that are both high-value and ready for direct reduced-WPT import:

1. URL and URLSearchParams
2. DOM parsing and serialization
3. DOM mutation/collection/descriptor surfaces
4. Web Animations and `requestAnimationFrame`
5. Encoding and streams
6. Intl exposed surfaces

Then move to surfaces that need a small harness reduction:

1. forms/focus/selection/default actions
2. navigation/history/lifecycle
3. CSSOM View and computed style
4. workers and structured clone

Finally, audit browser-comparison-first surfaces:

1. downloads and artifact capture
2. media/source-selection state
3. canvas and image-pipeline artifacts

## Reduction Rules for WPT Findings

When a WPT or browser comparison reveals a gap:

1. identify the smallest observable API contract that differs
2. write a deterministic in-repo regression for that contract
3. only then implement the fix or harness reduction needed to satisfy it

Do not import tests that depend on:

- real network loading
- full layout or painting
- nondeterministic timing
- browser process isolation
- cross-origin policies not already modeled by the harness

Reduce those cases to the minimal observable behavior already exposed by `browser_tester`.

## P4 Candidate Ranking

After closing `P3`, the remaining high-value intake candidates are the surfaces still labeled `Harness reduction first` or `Browser-comparison first`.

The ranking below uses a simple planning rubric:

- `Payoff`: how much public-surface risk the work removes
- `Determinism`: how feasible it is to reduce browser/WPT behavior into stable in-repo tests
- `Harness complexity`: how invasive the required harness/model reduction is

Scoring is `1` to `5`, where higher `Payoff`/`Determinism` is better and lower `Harness complexity` is better.

| Rank | Candidate family | Source inventory label | Payoff | Determinism | Harness complexity | Decision |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Editing, selection, focus, and clipboard default actions | `Harness reduction first` | 5 | 4 | 3 | `P4 core queue` |
| 2 | Navigation, history, loading, and lifecycle | `Harness reduction first` | 5 | 4 | 4 | `P4 core queue` |
| 3 | Media/resource loading and source selection | `Harness reduction first` plus `Browser-comparison first` | 5 | 3 | 4 | `P4 core queue` |
| 4 | Download artifacts and object-URL capture flows | `Browser-comparison first` | 4 | 3 | 4 | `P4 core queue` |
| 5 | Canvas/image-pipeline artifact behavior | `Browser-comparison first` | 3 | 3 | 4 | `P4 core queue` |
| 6 | Worker/message-loop and structured-clone harness reduction | `Harness reduction first` | 3 | 4 | 5 | `Deferred after core P4 queue` |
| 7 | CSSOM View/computed-style/layout reduction | `Harness reduction first` | 3 | 2 | 5 | `Deferred after core P4 queue` |

## P4 Execution Order

The ranked intake order for the next phase is:

1. editing, selection, focus, and clipboard
2. navigation, history, loading, and lifecycle
3. media/resource loading and candidate selection
4. download/object-URL/artifact capture
5. canvas/image-pipeline artifacts

Two audit families stay out of the core `P4` queue for now:

- worker/message-loop reduction remains useful, but the current deterministic worker surface is already strong enough that it does not outrank the editing/navigation/media gaps above
- CSSOM View/computed-style reduction still depends on a narrower deterministic layout contract, so it should stay on a deferred rolling backlog until a smaller reduction target is chosen
