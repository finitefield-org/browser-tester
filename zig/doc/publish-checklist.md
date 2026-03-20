# Publish Checklist

Use this checklist before treating a Zig workspace state as a candidate for sharing or tagging.

## Test Commands

- `zig fmt src/*.zig build.zig`
- `zig build test`

## Release Checks

- verify `README.md` matches the public surface
- verify `doc/capability-matrix.md` matches the supported capability set
- verify `doc/mock-guide.md` documents any public test-only mock behavior
- verify public API changes have contract and regression coverage
- verify property tests remain green
- check `git status` for unintended changes

## Notes

- Public `Harness` API additions should always be paired with documentation updates and a regression test.
- Mock families should never be added without documenting their capture and reset behavior.
