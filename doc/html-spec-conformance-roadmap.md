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
- Use external compatibility checks, including WPT, only as a spot-check tool for ambiguous or high-risk algorithms. They should inform decisions, not replace the repository's deterministic acceptance tests.

Minimum verification categories for conformance work:

- parser edge cases
- fragment parsing and serialization round-trips
- boolean, enumerated, numeric, and URL attribute reflection
- form controls, submit flows, and validation
- DOM mutation APIs
- event ordering, cancellation, and default actions

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
