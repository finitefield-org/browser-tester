# Mock Guide

`Harness.mocksMut()` returns the typed test-only `MockRegistry`. Use it when a test needs deterministic network, dialogs, clipboard, location, matchMedia, download, file-input, or storage behavior.

The registry is intentionally narrow:

- it exposes families, not a bag of `set_*` helpers
- each family carries its own capture and reset semantics
- `resetAll()` clears every family between scenarios

## Minimal Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(std.heap.page_allocator, "<main></main>");
    defer harness.deinit();

    var mocks = harness.mocksMut();
    try mocks.fetch().respondText("https://app.local/api/message", 201, "ok");
    try mocks.dialogs().pushConfirm(true);
    try mocks.clipboard().seedText("seeded");

    const response = try harness.fetch("https://app.local/api/message");
    try std.testing.expectEqualStrings("ok", response.body);
    try std.testing.expectEqual(@as(usize, 1), mocks.fetch().calls().len);

    try std.testing.expect(try harness.confirm("Continue?"));
    try std.testing.expectEqualStrings("seeded", try harness.readClipboard());

    try harness.captureDownload("report.csv", "downloaded bytes");
    try std.testing.expectEqual(@as(usize, 1), mocks.downloads().artifacts().len);
}
```

## MatchMedia Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<button id='toggle'>Toggle</button><div id='out'></div><script>document.getElementById('toggle').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); document.getElementById('out').textContent = String(mql) + ':' + String(mql.matches); });</script>",
    );
    defer harness.deinit();

    try harness.mocksMut().matchMedia().seedMatch("(max-width: 600px)", true);
    try harness.click("#toggle");
    try harness.assertValue("#out", "[object MediaQueryList]:true");
    try std.testing.expectEqual(@as(usize, 1), harness.mocksMut().matchMedia().calls().len);
}
```

## Capture Model

Call capture records the inputs requested by the test:

- `fetch.calls()` records requested URLs
- `dialogs.alertMessages()`, `confirmMessages()`, and `promptMessages()` record dialog text
- `location.navigations()` records navigated URLs
- `fileInput.selections()` records selector/file lists

Artifact capture records the side effects a test needs to inspect:

- `fetch.respondText(...)` injects a deterministic response
- `fetch.fail(...)` injects a deterministic failure
- `clipboard.writes()` records written clipboard values and keeps the latest value available for subsequent reads
- `downloads.artifacts()` records captured file names and bytes
- `storage.local()` and `storage.session()` hold seeded key/value pairs for deterministic reads

The same capture model is what keeps the mock families predictable without exposing browser internals.

`matchMedia()` is seeded by exact query string:

- `matchMedia.seedMatch(query, matches)` injects the query result
- `matchMedia.fail(query)` injects an explicit failure for the query
- `matchMedia.calls()` records requested queries in order

## Failure Semantics

The public mock API fails explicitly when the test has not seeded the required state:

- `Harness.fetch()` returns `error.MockError` if no matching response or failure rule exists
- `Harness.confirm()` and `Harness.prompt()` return `error.MockError` when the queue is empty
- `Harness.readClipboard()` returns `error.MockError` when clipboard text has not been seeded
- `Harness.captureDownload()` returns `error.MockError` for blank file names
- `window.matchMedia()` returns `error.MockError` when no matching rule exists or a failure rule was seeded
- `Harness.advanceTime(-1)` returns `error.TimerError`
- `Harness.setFiles()` returns `error.DomError` when the target is not a file input

## Reset Semantics

`MockRegistry.resetAll()` clears every family:

- fetch response rules, error rules, and call capture
- dialog queues and capture logs
- clipboard seed and write capture
- location current URL and navigation capture
- matchMedia query rules and call capture
- download artifacts
- file-input selections
- storage seeds

That makes it safe to reuse the same harness in a test loop without carrying mock state across scenarios.
