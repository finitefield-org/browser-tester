# Test Taxonomy

This document defines the current testing roles in the repository.

The goal is not to redesign the whole test tree at once.
The goal is to make it obvious where a new test should go, and what kind of
stability that test is protecting.

## Current Layers

### 1. Public Contract Tests

Files:

- `tests/contract_harness_core.rs`
- runner: `scripts/run-test-layer.sh contract`

Purpose:

- protect the smallest stable public `Harness` contract
- verify the APIs listed as `Stable Core` and `Stable Test Mocks`
- stay readable and intentionally small

What belongs here:

- constructors such as `from_html`
- core actions such as `click`, `type_text`, `set_checked`, `submit`
- timer control such as `advance_time`
- core assertions such as `assert_*`
- representative direct mocks such as `fetch`, `clipboard`, `location`, `file input`

What does not belong here:

- deep runtime corner cases
- browser-compatibility details
- large issue-specific regressions

## 2. Subsystem Tests

Files:

- `src/tests/mod.rs`
- modules under `src/tests/`
- runner: `scripts/run-test-layer.sh subsystem`

Purpose:

- protect internal DOM, runtime, parser, event, and Web API behavior
- allow dense, implementation-aware regression coverage without making the public contract suite noisy

Typical examples:

- DOM interface details
- runtime value behavior
- event ordering specifics
- parser edge cases
- helper-level regressions

## 3. Integration Regression Tests

Files:

- `tests/integration_suite.rs`
- modules under `tests/integration_cases/`
- runner: `scripts/run-test-layer.sh integration`

Purpose:

- keep larger real-world or cross-subsystem regressions together
- reduce Cargo overhead by using one grouped integration entrypoint
- preserve issue-driven repro coverage without inflating the public contract suite

Typical examples:

- real-world HTML fixtures
- issue-numbered reproductions
- parser/runtime state regressions that span multiple subsystems

## 4. Property And Fuzz Tests

Files:

- `tests/parser_property_fuzz_test.rs`
- `tests/runtime_property_fuzz_test.rs`
- `tests/proptest-regressions/`
- runner: `scripts/run-test-layer.sh fuzz`

Purpose:

- explore many parser/runtime inputs automatically
- keep shrunk failing seeds for repeatable reproduction

Use these when:

- the space of valid inputs is broad
- a bug is better expressed as an invariant than as a single example

## Placement Rules

When adding a new test, choose the narrowest layer that still protects the real contract.

### Add a public contract test when:

- a stable `Harness` API is added
- a stable `Harness` API changes behavior intentionally
- a bug fix affects the documented `Stable Core` or `Stable Test Mocks` surface

### Add a subsystem test when:

- the behavior is internal or implementation-shaped
- the scenario is too detailed for the public contract suite
- the fix targets a specific DOM/runtime/parser helper path

### Add an integration regression when:

- the failure crosses subsystem boundaries
- the repro is tied to a real page, fixture, or issue report
- the scenario is too large for `src/tests`

### Add a property or fuzz test when:

- the bug is best protected by an invariant over many generated inputs
- example-based coverage would miss too much of the space

## Practical Workflow

For most feature or bug-fix changes, prefer this order:

1. Add or update a narrow regression test closest to the implementation.
2. If the change affects stable public behavior, also add or update a public contract test.
3. If the bug came from a real multi-subsystem scenario, add or update an integration regression too.

This keeps internal coverage deep without letting the public contract suite turn into a full compatibility matrix.
