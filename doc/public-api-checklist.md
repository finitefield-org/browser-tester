# Public API Checklist

Use this checklist whenever a change adds or materially changes a public API.

The main target is `Harness`, but the same rule applies to other public types and methods.

## 1. Decide Whether It Should Be Public At All

Before adding a new public API, confirm all of the following:

- this really needs to be a user-facing API
- the behavior cannot be expressed clearly enough by combining existing APIs
- this is not better modeled as a test-only mock
- this is not better modeled as trace/debug inspection

If the answer is unclear, do not add to `Harness` first and rationalize later.
Decide the category first.

## 2. Choose The Support Level

Classify the API in `doc/capability-matrix.md` before or during implementation:

- `Stable Core`
- `Stable Test Mocks`
- `Extended Browser-Like Surface`
- `Internal Only`

If it is not intended to be public contract, keep it internal.

## 3. Choose The Owning Subsystem

Pick the subsystem before implementation using `doc/subsystem-map.md`:

- DOM
- parser
- script runtime
- event / user actions
- timer / scheduler
- mocks / trace

Public entrypoints should stay thin.
Put the real behavior in the owning subsystem.

## 4. Update Docs In The Same Change

If the change affects `Stable Core` or `Stable Test Mocks`, update:

- `README.md`
- `doc/capability-matrix.md`
- any relevant focused doc such as `doc/mock-guide.md`

If the API is intentionally not part of the main contract, document caveats where appropriate instead of expanding the README casually.

## 5. Update Tests In The Same Change

Required test posture:

- add or update a public contract test when stable public behavior changes
- add or update a narrow regression test near the implementation
- add an integration regression if the bug or feature crosses subsystem boundaries

Use `doc/test-taxonomy.md` to choose the right layer.

## 6. Special Rule For New Mocks

If the public API is a new test-only mock, also satisfy the mock maintenance rule:

- add or update the public API
- add a minimal usage example
- add failure-path coverage
- document call capture or artifact capture behavior
- update `README.md`
- update `doc/mock-guide.md`

## 7. Special Rule For `Harness`

Before adding a new `Harness` method, answer these questions explicitly:

1. Is this a stable user contract, or just an implementation convenience?
2. Why is an existing method combination not enough?
3. Should this live under mocks/trace instead of the main action surface?
4. Which existing `Harness` category does it belong to?

If those answers are weak, keep the API internal.
