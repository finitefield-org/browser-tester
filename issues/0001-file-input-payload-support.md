# [P1] Add payload-aware file input and drop support

## Problem
`browser_tester 2.0.0` can seed selected file names, but the runtime only stores filenames in `FileInputSelection.files`. The script side does not expose a `File` or `FileList` model with payload data, so code that reads `input.files[0].text()` or `event.dataTransfer.files[0]` cannot be exercised.

## Evidence
- `crates/bt-dom/src/html_dom.rs` stores file inputs as `Vec<String>` and `value_for_node()` joins the names.
- `crates/bt-script/src/evaluator.rs` has no `File` / `FileList` value type or `files` property handling.
- The current contract coverage only verifies that `set_files()` updates the input value and the captured file names.

## Impact
This blocks common upload flows in real tools:
- SVG importers that call `await file.text()`
- CSV and text importers that use `FileReader` or `file.text()`
- drag-and-drop upload paths that read `event.dataTransfer.files`

## Expected behavior
- `Harness::set_files()` should accept a payload-aware file spec, or an equivalent API should be added.
- `input.files` and `dataTransfer.files` should expose file-like objects with at least `name`, `type`, `size`, and `text()`.
- Existing filename-only test cases should keep working.

## Suggested regression tests
- Seed a text payload and assert `await input.files[0].text()` returns the seeded content.
- Seed an SVG payload and assert the importer reads the text and updates the DOM.
- Trigger a drop handler and assert `event.dataTransfer.files[0].name` and `text()` are available.
