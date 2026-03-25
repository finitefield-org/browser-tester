# Release Checklist

This checklist is the repeatable gate for the Go workspace.
Use it before declaring a slice ready for publication or promotion.

## Scope

- Confirm the change belongs on the public facade, `DebugView`, a typed mock family, or an internal subsystem.
- Confirm the capability matrix already covers the public surface that changed.
- Confirm `README.md` and `doc/README.md` describe the new behavior if the user-visible surface changed.
- Confirm `doc/subsystem-map.md` still places the implementation in the right subsystem.
- Confirm legacy or deprecated spec branches were not added unless the capability matrix explicitly called for them.

## Behavior Coverage

- Add or update internal subsystem tests for the code that changed.
- Add or update public contract tests for any public facade behavior.
- Add or update regression tests for bug fixes and edge cases.
- Add or update fuzz/property tests for parser, selector, scheduler, registry, and other boundary-heavy subsystems when those subsystems change.
- Add or update minimal usage examples whenever a test-only mock or public user-facing entry point changes.

## Determinism

- Run `gofmt` on all changed Go files.
- Run `scripts/run-go-checklist.sh` from the repository root to verify formatting and run `go test ./... -count=1` from the Go workspace root.
- If a parser, selector, scheduler, or registry boundary changed materially, run the targeted fuzz/property seeds as part of the normal test pass and confirm they still satisfy the bounded invariants.

## Mock-Specific Gate

When a test-only mock is added or changed:

- verify the public API addition or update is intentional
- verify the minimal usage example is present
- verify failure coverage exists
- verify capture or artifact semantics are documented
- verify `README.md` and `doc/mock-guide.md` were updated together

## Public API Gate

When a new public `Harness` method or public view method is added or changed:

- verify it is not better served as a debug helper or a mock-only API
- verify the capability matrix has a row for it
- verify the public contract tests and regression tests were updated
- verify the relevant docs explain the bounded behavior and any known exclusions

## Ready To Publish

A slice is ready when:

- the code is formatted
- the full test suite passes
- the public boundary is documented
- the internal boundary is tested
- the change remains bounded
- the checklist above is complete
