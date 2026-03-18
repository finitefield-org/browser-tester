# Subsystem Map

This document gives a practical answer to one maintenance question:

"Before adding code, which subsystem should own it?"

It is intentionally operational, not exhaustive.
Use it to choose an edit location before writing the implementation.

## Core Subsystems

### DOM

Use this subsystem for:

- node tree state
- attributes and element state
- form control semantics
- element relationships and tree mutation

Typical file areas:

- `src/core_impl/dom/`
- `src/core_dom_utils.rs`
- `src/core_impl/html.rs`
- `src/core_impl/runtime/runtime_exec/member_calls_ops/`

Choose DOM when the question is mostly:

- "what does this element/node/property mean?"
- "how should a DOM mutation update state?"

### Parser

Use this subsystem for:

- JavaScript parsing
- statement/expression grammar handling
- parse-time normalization
- syntax error behavior

Typical file areas:

- `src/core_impl/parser/`
- `src/script_ast.rs`
- `src/script_ast_expr.rs`
- `src/script_ast_expr_enums.rs`
- `src/script_ast_stmt.rs`
- `src/js_regex.rs`

Choose Parser when the question is mostly:

- "can this source text be parsed?"
- "what AST shape should this syntax produce?"

### Script Runtime

Use this subsystem for:

- expression evaluation
- statement execution
- callable execution
- runtime values and execution state

Typical file areas:

- `src/core_impl/runtime/runtime_exec/`
- `src/core_impl/runtime/runtime_platform/script_runtime/`
- `src/runtime_state.rs`
- `src/runtime_values.rs`

Choose Script Runtime when the question is mostly:

- "how should this script execute?"
- "how should values, calls, or control flow behave?"

### Event / User Actions

Use this subsystem for:

- trusted user-like interactions
- DOM action default behavior
- event dispatch entrypoints tied to user actions
- assertions that validate user-facing DOM outcomes

Typical file areas:

- `src/core_impl/runtime/runtime_platform/dom_actions/`

Choose Event / User Actions when the question is mostly:

- "what should happen when a user clicks, types, pastes, submits, or dispatches an event?"

### Timer / Scheduler

Use this subsystem for:

- fake time
- queued timers and microtasks
- scheduler safety limits
- deterministic time advancement

Typical file areas:

- `src/core_impl/runtime/runtime_platform/dom_actions/timer_controls_execution.rs`
- scheduler state in `src/runtime_state.rs`

Choose Timer / Scheduler when the question is mostly:

- "when should this callback run?"
- "how should fake time move?"

### Mocks / Trace

Use this subsystem for:

- deterministic browser-like mocks
- artifact capture
- call capture
- trace and diagnostics controls

Typical file areas:

- `src/core_impl/runtime/runtime_platform/dom_actions/platform_mock_controls.rs`
- `src/core_impl/runtime/runtime_platform/dom_actions/trace_determinism_controls.rs`

Choose Mocks / Trace when the question is mostly:

- "how can tests inject deterministic platform behavior?"
- "how can tests inspect what happened?"

## Placement Rules

When a change appears to touch multiple subsystems:

1. Put the main behavior where the state transition belongs.
2. Keep public entrypoints thin and delegate inward.
3. Add narrow helpers in the owning subsystem instead of growing unrelated files.
4. If the answer is still unclear, prefer the subsystem that owns the long-term state, not the call site.

## Common Decisions

If you are adding:

- a new stable `Harness` method: start from public API docs, then choose the owning subsystem before implementation
- a new deterministic browser mock: put the public control under `Mocks / Trace`, then connect it to the runtime path that consumes it
- a new DOM property or element behavior: start in `DOM`, then wire `Script Runtime` lookup/eval only as needed
- a new parser form: start in `Parser`, not in runtime fallback code
- a new timer-facing behavior: start in `Timer / Scheduler`, not in unrelated DOM action modules

## Related Docs

- support-level classification: `doc/capability-matrix.md`
- test placement: `doc/test-taxonomy.md`
- file-size guard: `doc/file-size-guard.md`
