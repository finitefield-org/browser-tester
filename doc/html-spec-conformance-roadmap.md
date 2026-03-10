# HTML Spec Conformance Roadmap

## Purpose

This document defines how `browser_tester` should move closer to the HTML Living Standard in a controlled, test-first way.

It is not a claim of full browser compatibility. The goal is to make conformance work repeatable: identify a spec section, reproduce current behavior, add spec-anchored tests, implement the minimum change, and keep the result deterministic.

## Scope and Non-Goals

Priority scope:

- HTML parsing and tree construction
- DOM construction and DOM parsing/serialization APIs
- Attribute reflection and common microsyntax parsing
- Core element APIs
- Forms, validation, default actions, and related events
- User-interaction behaviors that affect current public APIs

Deferred unless they materially affect the current public surface:

- Full rendering behavior
- Broad media loading behavior
- General-purpose navigation and loading machinery beyond existing harness APIs
- Web APIs whose behavior is not already exposed by `browser_tester`

## Sources of Truth

- Use `html-standard.txt` as the local index for locating relevant HTML Living Standard sections quickly.
- Use chapter numbers as the stable reference point in implementation notes, tests, and issue descriptions.
- Use the WHATWG HTML Living Standard as the normative source for final behavior decisions:
  - [Multipage version](https://html.spec.whatwg.org/multipage/)
  - [One-page version](https://html.spec.whatwg.org/)

High-priority sections to anchor work against:

- `2.3 Common microsyntaxes`
- `2.6.1 Reflecting content attributes in IDL attributes`
- `3.2.4` to `3.2.6` element definitions, attributes, content models, and global attributes
- `4.x The elements of HTML`, especially the currently exposed element families
- `4.10 Forms`
- `6 User interaction`
- `8.5 DOM parsing and serialization APIs`
- `13.2 Parsing HTML documents`

## Current Baseline

The current repository is already structured in a way that supports spec-driven hardening:

- HTML parsing, selector logic, and script/runtime behavior are implemented in-repo, so spec changes do not need cross-language coordination.
- The DOM and runtime layers are separated enough to trace many gaps back to a specific subsystem.
- The test suite already includes broad element-level coverage in `src/tests`, with more than one hundred `dom_*` modules.
- Parser/runtime property tests already run in CI via `.github/workflows/property-fuzz.yml`.

The main weakness is traceability: existing tests cover a lot of surface area, but many are organized by API or element name rather than by HTML chapter and algorithm. The roadmap below fixes that by making every conformance change map back to a specific spec section.

## Roadmap Status After P2

The original roadmap phases are now complete:

- `P0: Parsing, Tree Construction, and Serialization`
- `P1: Attribute Reflection, Global Attributes, Element Algorithms, Forms, Default Actions, and Events`
- `P2: Navigation, Loading, Media, and Rendering-Tied Behavior`

That changes the remaining work materially.

`browser_tester` now exposes a wider deterministic public API surface than the original HTML-centered roadmap assumed. In addition to core HTML parsing and DOM behavior, the repository already exposes and regression-tests:

- DOM mutation, reflection, collections, and event dispatch across a broad set of HTML elements
- forms, validation, focus, selection, clipboard/data-transfer, and default-action flows
- harness-backed `location`, `history`, `navigation`, and document lifecycle behavior
- media elements, `TextTrackList`, `TextTrack`, `TimeRanges`, and resource-selection state
- scroll, geometry, and computed-style reads
- `element.animate(...)` and related rendering-tied object surfaces already visible to scripts
- URL/object-URL/download, worker, structured-clone, encoding, streams, canvas, and Intl APIs that are already public in the crate

Because of that broadened surface, the next step should not be another HTML chapter expansion. The highest-value next phase is an interoperability audit across already exposed APIs.

## Decision After P2

The next roadmap step is:

- create a new phase, `P3`
- run it as a WPT-guided audit over APIs that are already exposed publicly
- reduce every finding to deterministic in-repo regressions and the smallest possible implementation change

This is the right tradeoff after `P2` because:

- the main remaining risk is edge-case parity and cross-surface interaction bugs, not missing major algorithms from the original HTML backlog
- recent regressions have clustered around receiver validation, descriptor visibility, cached-wrapper identity, event/promise ordering, and resource-selection churn
- those gaps are easier to discover with browser/WPT comparison over the current public API surface than with continued chapter-by-chapter backlog growth

The starting inventory and WPT mapping for this phase should be maintained in a separate working document so the backlog can evolve without rewriting the roadmap itself. The current inventory lives in `doc/p3-wpt-audit-inventory.md`.

## Roadmap Status After P3

`P3: WPT-Guided Exposed-Surface Interop Hardening` is now complete.

The repository has now completed a full horizontal audit over the currently exposed public surface, including:

- DOM/HTML mutation, collection, reflection, and descriptor surfaces
- forms, focus, selection, validation, and default actions
- navigation, history, and document lifecycle
- CSSOM View, geometry, scroll, and computed style
- Web Animations and rendering-tied object surfaces
- media/resource elements and wrapper objects
- clipboard/data-transfer/download/object-URL behavior
- workers, structured clone, blob URLs, encoding, streams, Intl, and other non-HTML builtins

That changes the remaining risk profile again.

The main risk is no longer broad exposed-surface incoherence on already public APIs. The remaining gaps are concentrated in areas where browser parity still depends on harness reduction, selective browser comparison, or narrower follow-up intake rather than another wide audit sweep.

## Decision After P3

The next roadmap step is:

- create a new phase, `P4`
- focus it on harness reduction and selective reduced-WPT intake for partially modeled behavior
- keep newly exposed APIs on a rolling targeted backlog instead of opening another broad exposed-surface audit immediately

This is the right tradeoff after `P3` because:

- another horizontal audit phase would mostly revisit areas whose branding, descriptor, and receiver-validation surfaces are now already covered
- the highest-value remaining mismatches sit in surfaces that were still `Harness reduction first` or `Browser-comparison first` in `doc/p3-wpt-audit-inventory.md`
- newly exposed APIs should be absorbed incrementally through inventory refreshes and focused backlog entries, not by restarting a full audit phase

The next work should therefore prioritize:

- editing-adjacent focus/selection/default-action behavior that still depends on simplified harness editing semantics
- navigation/loading lifecycle paths whose current deterministic model is narrower than browser state machines
- media/resource loading and candidate-selection behavior that remains intentionally partial
- download/object-URL/captured-artifact behavior that is harness-shaped by design
- canvas/image-pipeline artifact behavior that still needs browser-comparison-first reduction

## Priority Workstreams

### P0: Parsing, Tree Construction, and Serialization

Anchor sections:

- `13.2.1` overview of the parsing model
- `13.2.4.1` insertion mode
- `13.2.6.4.x` token processing by insertion mode
- `8.5 DOM parsing and serialization APIs`
- `8.5.2 Unsafe HTML parsing methods`

Focus:

- Parser insertion modes and tree-construction edge cases before adding more per-element behavior
- HTML fragment parsing used by `innerHTML`, `outerHTML`, `insertAdjacentHTML`, and `setHTMLUnsafe`
- Round-trip expectations for parsing plus serialization where the crate already exposes those operations

Typical gaps to look for:

- Table-related insertion-mode behavior
- Foster parenting and misplaced content recovery
- Head/body/template handling
- Detached-node and document-child restrictions in serialization/mutation APIs

Primary repo surfaces:

- `src/core_impl/parser`
- `src/core_impl/dom/text_html_content.rs`
- `src/tests`

### P1: Attribute Reflection, Global Attributes, and Element Algorithms

Anchor sections:

- `2.3 Common microsyntaxes`
- `2.6.1 Reflecting content attributes in IDL attributes`
- `3.2.4.1 Attributes`
- `3.2.6 Global attributes`
- Relevant `4.x` element definitions

Focus:

- Normalize boolean, enumerated, numeric, URL, and token-list parsing behavior before expanding feature breadth
- Audit reflection rules element-by-element only after the shared coercion rules are correct
- Prefer shared helper logic over one-off element fixes when multiple APIs depend on the same microsyntax

Typical gaps to look for:

- Boolean attribute presence semantics
- Enumerated-attribute invalid-value and missing-value defaults
- Numeric parsing edge cases
- URL resolution and serialization mismatches
- Global attribute reflection inconsistencies across elements

### P1: Forms, Default Actions, and Events

Anchor sections:

- `4.10 Forms`
- Relevant parts of `6 User interaction`
- Element-specific sections under `4.x` for `input`, `button`, `select`, `option`, `textarea`, `label`, `form`, and related elements

Focus:

- Form submission algorithms and validation ordering
- Default actions for trusted user-style interactions exposed by the harness
- Event ordering and cancellation behavior around input, change, click, submit, copy/paste, and focus flows

Typical gaps to look for:

- Validation timing and prevented-default behavior
- Submitter selection rules
- Checked/value synchronization rules
- Selection/file-input behavior that depends on deterministic mocks

### P2: Navigation, Loading, Media, and Rendering-Tied Behavior

Anchor sections:

- Relevant `4.x`, `7`, `8`, and `15` sections that are already surfaced through current harness APIs

Focus:

- Only standardize behavior that materially affects existing public APIs, mocks, or deterministic test flows
- Keep broader browser-loading and rendering work deferred until a concrete harness requirement exists

Examples:

- `location` transitions already modeled by harness mocks
- Download-triggering behaviors already exposed through captured artifacts
- Limited media-element behavior where current APIs already assert it

### P3: WPT-Guided Exposed-Surface Interop Hardening

Anchor sources:

- Relevant WHATWG HTML, DOM, URL, Fetch, File API, Encoding, Streams, Web Animations, Clipboard, CSSOM View, and Workers specs only where the API is already public in `browser_tester`
- Web Platform Tests as a discovery and prioritization input for existing surfaces

Focus:

- audit only the APIs that `browser_tester` already exposes publicly
- use WPT and browser comparison to discover mismatches and missing edge cases
- convert each finding into deterministic repository tests and then fix the smallest viable implementation gap
- avoid broad new feature work unless an already exposed API cannot be made coherent without it

Typical gaps to look for:

- branded object, prototype, descriptor, and callable-surface mismatches
- promise/event/lifecycle ordering mismatches
- cached live-wrapper identity and liveness regressions
- resource selection and current-state restoration edge cases
- worker/message/object-URL/structured-clone integration mismatches
- geometry/style/animation surfaces whose values or readonly behavior drift from browser expectations

Primary repo surfaces:

- `src/tests`
- runtime/property-access layers under `src/core_impl/runtime`
- harness-backed navigation/loading/media logic
- any test-only deterministic mocks already documented in `README.md`

### P4: Harness Reduction and Selective WPT Intake

Anchor sources:

- relevant WHATWG HTML, DOM, URL, Fetch, File API, Clipboard, CSSOM View, Web Animations, Encoding, Streams, and Workers specs
- Web Platform Tests, but only after the affected behavior is reduced into the deterministic harness model used by `browser_tester`

Focus:

- make a small set of partially modeled high-value surfaces deterministic enough for reduced-WPT intake
- prioritize harness shaping and browser-comparison reduction over broad new API exposure
- keep scope narrow and test-first, preferring a few high-confidence reductions over speculative feature breadth

Typical gaps to look for:

- editing, selection, focus, and clipboard behavior that depends on browser-native text editing semantics
- navigation/loading transitions whose lifecycle ordering is only partially modeled today
- media source selection and load-state transitions that still rely on simplified heuristics
- download and artifact-capture behavior that needs explicit browser-comparison reduction
- canvas/image-pipeline output contracts that are deterministic today but still mock-shaped

Primary repo surfaces:

- `src/tests`
- harness-backed action/navigation logic under `src/core_impl/runtime/runtime_platform`
- media/resource selection helpers
- deterministic mock/documentation surfaces described in `README.md`

### Post-P4 Direction

`P4` is now complete.

Decision:

- do not open a broad `P5` roadmap phase yet
- move to a smaller rolling maintenance backlog instead

Rationale:

- the highest-payoff partially modeled surfaces targeted by `P4` are now reduced enough for selective reduced-WPT intake
- the remaining work is narrower and less phase-shaped: deferred worker/message-loop reduction, CSSOM View/layout reduction, and steady reduced-WPT/browser-comparison intake over the contracts stabilized in `P4`
- a new named phase should be justified only if these maintenance items grow into another cross-cutting implementation campaign rather than a steady audit-and-intake loop

Rolling maintenance priorities after `P4`:

1. deferred worker/message-loop and structured-clone harness reduction
2. CSSOM View/layout reduction target selection
3. selective reduced-WPT intake for the navigation/loading contracts stabilized in `P4`
4. selective reduced-WPT intake for the media/download/canvas contracts stabilized in `P4`
5. periodic public-API delta audit for newly exposed surfaces or regression clusters

`M2` outcome:

- do not open a broad CSS/layout phase
- keep CSSOM View follow-up work limited to scroll aliases, geometry object surface, computed-style object surface, and layout-derived properties whose values are already deterministic in the harness
- keep full layout, painting, visual viewport, transforms, and reflow-dependent geometry on the rolling backlog until a future public API need justifies a larger effort

`M5` outcome:

- the first rolling public-API delta audit did not reveal any newly exposed surface family that falls outside the existing roadmap and inventory buckets
- recent regressions still fit the current rolling backlog: deferred worker/message-loop work, deferred CSSOM View/layout reduction, and selective reduced-WPT/browser-comparison intake over already stabilized contracts
- do not open a new named roadmap phase yet; keep proceeding through the rolling maintenance backlog until a future public API expansion or regression cluster becomes meaningfully cross-cutting again

Post-`M5` triage refresh:

- the narrowed CSSOM View subset is now stabilized enough that it no longer needs to be the first deferred maintenance target
- the next smallest high-confidence intake target is selective worker/message-loop/browser-comparison coverage over the existing end-of-task delivery and structured-clone contract
- keep CSSOM View/layout follow-up limited to reopening cases only if a future public API need expands beyond the already stabilized scroll/geometry/computed-style subset

Post-worker/browser-comparison triage refresh:

- the selective intake slices over navigation/loading, media/download/canvas, CSSOM View, and worker/message-loop are now complete enough that there is no obvious next reduced browser-comparison family with similar payoff
- do not open another named phase and do not force a new selective intake slice immediately
- move the roadmap into a dormant/on-demand maintenance posture gated by either:
  - newly exposed public APIs
  - a fresh browser-comparison-backed regression cluster
  - a future harness change that broadens one of the currently stabilized contracts
- keep the default next step as a periodic public-API delta audit rather than another standing implementation campaign

Periodic public-API delta audit outcome:

- the current public constructor/prototype surface still fits the existing roadmap and inventory buckets
- recent regressions since the worker/message-loop intake do not form a new browser-comparison-backed cluster that would justify reopening selective intake immediately
- keep maintenance in a dormant/on-demand state until one of the existing reactivation triggers is hit

Dormant backlog watch outcome:

- no concrete reactivation trigger is currently present
- keep the roadmap in dormant/on-demand maintenance mode
- the next implementation task should only be created when a trigger justifies reopening the smallest selective intake slice

Trigger-driven reopening outcome:

- the current check does not justify reopening any selective intake slice
- there is no active roadmap implementation task until a new trigger appears

## Standard Workflow for Each Gap

Every conformance task should follow the same sequence:

1. Identify the exact HTML section number in `html-standard.txt`.
2. Reproduce current behavior with the smallest failing test or fixture.
3. Add or update a test that cites the relevant spec section in its name or comments.
4. If the algorithm depends on external I/O or browser state, add a deterministic mock first.
5. Implement the smallest change that satisfies the spec-backed test.
6. Run targeted tests for the touched area.
7. Run the full `cargo test` suite.
8. Run property/fuzz coverage relevant to parser or runtime behavior.

This order matters. Do not start with an implementation guess when the algorithm can be pinned down by a targeted, spec-labeled test.

## Traceability Template

Track each gap using a row with the following fields:

| Spec section | Repo surface | Current coverage | Missing behavior | Required mock | Acceptance test |
| --- | --- | --- | --- | --- | --- |
| `13.2.6.4.9 in table` | `src/core_impl/parser` | element smoke tests exist | incorrect table insertion recovery | none | parser fixture plus DOM assertion |
| `2.6.1 reflect boolean attrs` | shared DOM/runtime property layer | scattered element tests | inconsistent IDL/content-attribute sync | none | focused reflection tests across representative elements |
| `4.10 form submission` | form runtime and user actions | submit tests exist | ordering or prevented-default mismatch | existing form/navigation mocks | targeted submit/validation tests |
| `8.5.2 unsafe HTML parsing methods` | DOM mutation/serialization layer | API tests exist | fragment parsing or document restrictions differ | none | round-trip plus error-path tests |

Use this table format in issues, TODO tracking, or future spec-coverage documents. The key requirement is that each row is decision-ready and testable.

## Areas That Need Algorithm-First Work

Do not treat the following as isolated element chores. They should be hardened as shared algorithms first:

- parser insertion modes
- fragment parsing
- attribute reflection
- form submission and validation
- `innerHTML`
- `outerHTML`
- `insertAdjacentHTML`
- `setHTMLUnsafe`

If these are fixed piecemeal through element-specific patches, regressions will reappear when new element APIs are added.

## Verification Strategy

Internal tests are the primary enforcement mechanism.

- Expand `src/tests` with targeted, spec-anchored behavior tests.
- Keep parser/runtime property tests as a regression net for high-churn logic.
- Use the existing CI profiles in `.github/workflows/property-fuzz.yml` to keep lightweight coverage on PRs and deeper runs on scheduled jobs.
- Use external compatibility checks, including WPT, as a discovery and spot-check tool for ambiguous or high-risk algorithms. They should inform decisions, not replace the repository's deterministic acceptance tests.

Minimum verification categories for conformance work:

- parser edge cases
- fragment parsing and serialization round-trips
- boolean, enumerated, numeric, and URL attribute reflection
- form controls, submit flows, and validation
- DOM mutation APIs
- event ordering, cancellation, and default actions

For `P3`, add one more rule:

- if a gap is discovered through WPT or browser comparison, land a reduced deterministic repository regression for it before or together with the fix

## Rules for Mock APIs and Documentation

When spec-conformance work requires new test-only mocks or extensions to existing mocks:

- keep them deterministic and narrowly scoped to the algorithm being tested
- prefer extending existing harness/mock patterns over inventing parallel APIs
- document the mock usage in `README.md` at the same time the public API is added

This keeps the crate aligned with its stated testing model and avoids hidden testing-only behavior.

## Definition of Done

A conformance task is complete only when all of the following are true:

- the relevant HTML section number is recorded
- the missing behavior is expressed as a deterministic test
- any required mock is documented and justified
- targeted tests pass
- `cargo test` passes
- relevant property/fuzz coverage passes when parser or runtime logic changed

Roadmap progress should be measured by closed spec-backed gaps, not by raw counts of added element types or methods.
