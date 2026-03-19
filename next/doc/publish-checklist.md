# Publication Checklist

This checklist is the Phase 5 release gate for `next/`.
Use the quick profile for fast feedback and the hardening profile before publishing or tagging a workspace state.

## Test Profiles

- Quick: `./scripts/test-quick.sh`
- Hardening: `./scripts/test-hardening.sh`

## Release Checklist

- run `cargo fmt --all`
- run the quick test profile
- run the hardening test profile
- verify `README.md` matches the public surface
- verify `doc/capability-matrix.md` matches the supported capability set
- verify `doc/mock-guide.md` documents any new test-only mock behavior
- verify public API changes have contract and regression coverage
- verify property tests are still green
- check `git status` for unintended changes

## Notes

- The hardening profile includes workspace tests and doc tests.
- Public `Harness` API additions should always be paired with documentation updates and a regression test.
