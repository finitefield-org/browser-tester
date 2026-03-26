# [P2] Expose `DataTransfer.files` for drag-and-drop uploads

## Problem
Some import flows use `drop` handlers and read `event.dataTransfer.files[0]`, but the runtime has no `DataTransfer` model. That blocks drag-and-drop file import paths even when file input selection is available.

## Evidence
- `tests/fixtures/csv-deduplicator-inline-script.html` reads `event.dataTransfer.files[0]` in its drop handler.
- The runtime source has no `DataTransfer` / `FileList` / drag-drop payload model.

## Expected behavior
- Drop events expose a deterministic `DataTransfer` object.
- `dataTransfer.files` contains file-like entries seeded by the test.
- The same payload model can be reused for both input selection and drag-and-drop.

## Suggested regression tests
- Seed a file payload, dispatch a drop event, and assert the handler reads `event.dataTransfer.files[0].name`.
- If `FileReader` support exists, assert the payload text can be read from the dropped file.
- Ensure the existing `set_files` path keeps working.
