## Summary

- describe the user-visible or maintenance-visible change

## Public API Checklist

If this PR adds or changes a public API, confirm the following before merge.

- [ ] I decided whether this should be public at all instead of adding to `Harness` by default.
- [ ] I chose a support level in `doc/capability-matrix.md`.
- [ ] I chose an owning subsystem using `doc/subsystem-map.md`.
- [ ] I updated `README.md` if the change belongs to `Stable Core` or `Stable Test Mocks`.
- [ ] I added or updated a public contract test when stable public behavior changed.
- [ ] I added or updated a narrow regression test near the implementation.

## `Harness` Gate

If this PR adds a new `Harness` method, confirm the following.

- [ ] Existing API combinations were not sufficient.
- [ ] This is not better modeled as a test-only mock.
- [ ] This is not better modeled as trace/debug inspection.
- [ ] The method fits an existing `Harness` category.

## Mock Rule

If this PR adds a new test-only mock, confirm the following.

- [ ] I added or updated the public API.
- [ ] I added a minimal usage example.
- [ ] I added failure-path coverage.
- [ ] I documented call capture or artifact capture behavior.
- [ ] I updated `README.md`.
- [ ] I updated `doc/mock-guide.md`.
