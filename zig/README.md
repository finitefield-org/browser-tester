# browser-tester zig

This directory is a clean-room Zig rewrite workspace for `browser-tester`.
The rewrite follows [`next.md`](../next.md) and keeps the public surface small while the internal phases are built out.

Current state:

- phase 0 scaffold plus the internal phase 1 DOM bootstrap slice, the phase 2 script runtime minimum slice, the phase 3 event/default-action and form-control slice, the phase 4 deterministic mock and fake-clock slice, and the phase 6 selector expansion slice
- `Harness.assertExists(...)`, `Harness.assertValue(...)`, `Harness.assertChecked(...)`, and `Harness.dumpDom(...)` are available for inspection
- `Harness.nowMs(...)`, `Harness.advanceTime(...)`, `Harness.flush(...)`, `Harness.mocksMut(...)`, `Harness.fetch(...)`, `Harness.alert(...)`, `Harness.confirm(...)`, `Harness.prompt(...)`, `Harness.readClipboard(...)`, `Harness.writeClipboard(...)`, `Harness.captureDownload(...)`, `Harness.navigate(...)`, and `Harness.setFiles(...)` are available for deterministic runtime and mock control
- `Harness.click(...)`, `Harness.typeText(...)`, `Harness.setChecked(...)`, `Harness.setSelectValue(...)`, `Harness.focus(...)`, `Harness.blur(...)`, `Harness.submit(...)`, and `Harness.dispatch(...)` are available for user-like actions
- inline `<script>` bootstrapping runs during `Harness.fromHtml(...)` construction for the `document.getElementById(...).textContent = ...` slice
- `Harness` and `HarnessBuilder` are available
- `Session` stays internal and owns the copied configuration state plus the DOM store, script runtime state, event listener registry, focus state, fake clock state, and mock registry
- `DomStore` builds, selects, and dumps HTML trees for tests, including class selectors and descendant/child combinators, but it is not part of the public API
- event hardening and later script expansion pieces are still planned

## Quick Start

```bash
cd zig
zig build test
```

## Minimal Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<form id='profile'><input id='name'><input id='agree' type='checkbox'><button id='submit' type='submit'>Save</button></form><div id='out'></div><script>document.getElementById('profile').addEventListener('submit', () => { document.getElementById('out').textContent = document.getElementById('name').value + ':' + String(document.getElementById('agree').checked); });</script>",
    );
    defer harness.deinit();

    try harness.typeText("#name", "Alice");
    try harness.click("#agree");
    try harness.click("#submit");
    try harness.assertChecked("#agree", true);
    try harness.assertValue("#out", "Alice:true");
}
```

## Mock Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(std.heap.page_allocator, "<main></main>");
    defer harness.deinit();

    try harness.mocksMut().fetch().respondText("https://app.local/api/message", 200, "ok");
    const response = try harness.fetch("https://app.local/api/message");
    try std.testing.expectEqualStrings("ok", response.body);
    try std.testing.expectEqual(@as(usize, 1), harness.mocksMut().fetch().calls().len);

    try harness.captureDownload("report.csv", "downloaded bytes");
    try std.testing.expectEqual(@as(usize, 1), harness.mocksMut().downloads().artifacts().len);
}
```

## Public Surface

- `Harness`
- `HarnessBuilder`
- `StorageSeed`
- `MockRegistry`
- `FetchMocks`
- `FetchResponseRule`
- `FetchErrorRule`
- `FetchCall`
- `FetchResponse`
- `DialogMocks`
- `ClipboardMocks`
- `LocationMocks`
- `DownloadMocks`
- `DownloadCapture`
- `FileInputMocks`
- `FileInputSelection`
- `StorageSeeds`
- `Error`
- `Result(T)`
- `Error` currently includes `InvalidUrl`, `InvalidSelector`, `AssertionFailed`, `DomError`, `EventError`, `ScriptParse`, `ScriptRuntime`, `HtmlParse`, `MockError`, `TimerError`, and `OutOfMemory`
- `Harness.assertExists(selector)`
- `Harness.assertValue(selector, expected)`
- `Harness.assertChecked(selector, expected)`
- `Harness.nowMs()`
- `Harness.advanceTime(delta_ms)`
- `Harness.flush()`
- `Harness.mocksMut()`
- `Harness.fetch(url)`
- `Harness.alert(message)`
- `Harness.confirm(message)`
- `Harness.prompt(message)`
- `Harness.readClipboard()`
- `Harness.writeClipboard(text)`
- `Harness.captureDownload(file_name, bytes)`
- `Harness.navigate(url)`
- `Harness.setFiles(selector, files)`
- `Harness.click(selector)`
- `Harness.typeText(selector, text)`
- `Harness.setChecked(selector, checked)`
- `Harness.setSelectValue(selector, value)`
- `Harness.focus(selector)`
- `Harness.blur(selector)`
- `Harness.submit(selector)`
- `Harness.dispatch(selector, event_type)`
- `Harness.dumpDom(allocator)`

## Docs

- [Architecture](doc/architecture.md)
- [Capability Matrix](doc/capability-matrix.md)
- [Implementation Guide](doc/implementation-guide.md)
- [Subsystem Map](doc/subsystem-map.md)
- [Mock Guide](doc/mock-guide.md)
- [Limitations](doc/limitations.md)
- [Roadmap](doc/roadmap.md)
- [Publish Checklist](doc/publish-checklist.md)
