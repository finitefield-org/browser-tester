# Mock Guide

`Harness.mocksMut()` returns the typed test-only `MockRegistry`. Use it when a test needs deterministic network, dialogs, clipboard, location, open/close/print/scroll, matchMedia, download, file-input, or storage behavior, including seeding `window.localStorage`, `window.sessionStorage`, `window.open()`, `window.close()`, `window.print()`, `window.scrollTo()`, and `window.scrollBy()` before inline scripts run; file-input selections also flow through to the script-side `input.files` snapshot; storage mutations also dispatch deterministic `storage` events through `window.addEventListener('storage', ...)` and `window.onstorage`.

The registry is intentionally narrow:

- it exposes families, not a bag of `set_*` helpers
- each family carries its own capture and reset semantics
- `resetAll()` clears every family between scenarios

## Current Mock Families

- fetch
- dialogs
- clipboard
- location
- open
- close
- print
- scroll
- matchMedia
- downloads
- file_input
- storage

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

## MatchMedia Listener Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<button id='toggle'>Toggle</button><div id='out'></div><script>document.getElementById('toggle').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); mql.addListener(() => { document.getElementById('out').textContent = 'changed'; }); document.getElementById('out').textContent = String(mql.matches); });</script>",
    );
    defer harness.deinit();

    try harness.mocksMut().matchMedia().seedMatch("(max-width: 600px)", false);
    try harness.click("#toggle");
    try harness.assertValue("#out", "false");

    try harness.mocksMut().matchMedia().seedMatch("(max-width: 600px)", true);
    try harness.flush();
    try harness.assertValue("#out", "changed");
}
```

## MatchMedia Change Event Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<button id='toggle'>Toggle</button><div id='out'></div><script>document.getElementById('toggle').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); mql.addEventListener('change', () => { document.getElementById('out').textContent = 'changed'; }); document.getElementById('out').textContent = String(mql.matches); });</script>",
    );
    defer harness.deinit();

    try harness.mocksMut().matchMedia().seedMatch("(max-width: 600px)", false);
    try harness.click("#toggle");
    try harness.assertValue("#out", "false");

    try harness.mocksMut().matchMedia().seedMatch("(max-width: 600px)", true);
    try harness.flush();
    try harness.assertValue("#out", "changed");
}
```

## MatchMedia OnChange Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<button id='toggle'>Toggle</button><div id='out'></div><script>document.getElementById('toggle').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); mql.onchange = () => { document.getElementById('out').textContent = 'changed'; }; document.getElementById('out').textContent = String(mql.matches); });</script>",
    );
    defer harness.deinit();

    try harness.mocksMut().matchMedia().seedMatch("(max-width: 600px)", false);
    try harness.click("#toggle");
    try harness.assertValue("#out", "false");

    try harness.mocksMut().matchMedia().seedMatch("(max-width: 600px)", true);
    try harness.flush();
    try harness.assertValue("#out", "changed");
}
```

## Open/Print Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<main id='out'></main><script>window.open('https://app.local/popup', '_blank', 'noopener'); window.print(); document.getElementById('out').textContent = 'booted';</script>",
    );
    defer harness.deinit();

    try harness.open("https://app.local/settings");
    try harness.print();
    try std.testing.expectEqual(@as(usize, 2), harness.mocksMut().open().calls().len);
    try std.testing.expectEqualStrings(
        "https://app.local/popup",
        harness.mocksMut().open().calls()[0].url.?,
    );
    try std.testing.expectEqualStrings(
        "https://app.local/settings",
        harness.mocksMut().open().calls()[1].url.?,
    );
    try std.testing.expectEqual(@as(usize, 2), harness.mocksMut().print().calls().len);
}
```

## Close Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(
        std.heap.page_allocator,
        "<main id='out'></main><script>window.close(); document.getElementById('out').textContent = 'closed';</script>",
    );
    defer harness.deinit();

    try harness.close();
    try harness.assertValue("#out", "closed");
    try std.testing.expectEqual(@as(usize, 2), harness.mocksMut().close().calls().len);
}
```

## Scroll Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var harness = try bt.Harness.fromHtml(std.heap.page_allocator, "<main></main>");
    defer harness.deinit();

    try harness.scrollTo(10, 20);
    try harness.scrollBy(-5, 3);
    try std.testing.expectEqual(@as(usize, 2), harness.mocksMut().scroll().calls().len);
    try std.testing.expectEqual(.To, harness.mocksMut().scroll().calls()[0].method);
    try std.testing.expectEqual(@as(i64, 10), harness.mocksMut().scroll().calls()[0].x);
    try std.testing.expectEqual(@as(i64, 20), harness.mocksMut().scroll().calls()[0].y);
    try std.testing.expectEqual(.By, harness.mocksMut().scroll().calls()[1].method);
    try std.testing.expectEqual(@as(i64, -5), harness.mocksMut().scroll().calls()[1].x);
    try std.testing.expectEqual(@as(i64, 3), harness.mocksMut().scroll().calls()[1].y);
}
```

## Open/Print Failure Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var builder = bt.Harness.builder(std.heap.page_allocator);
    defer builder.deinit();

    _ = builder.html("<script>window.open('https://app.local/popup', '_blank', 'noopener');</script>");
    builder.openFailure("popup blocked");

    try std.testing.expectError(error.MockError, builder.build());
}
```

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var builder = bt.Harness.builder(std.heap.page_allocator);
    defer builder.deinit();

    _ = builder.html("<script>window.print();</script>");
    builder.printFailure("print blocked");

    try std.testing.expectError(error.MockError, builder.build());
}
```

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var builder = bt.Harness.builder(std.heap.page_allocator);
    defer builder.deinit();

    _ = builder.html("<script>window.close();</script>");
    builder.closeFailure("window closed");

    try std.testing.expectError(error.MockError, builder.build());
}
```

## Scroll Failure Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var builder = bt.Harness.builder(std.heap.page_allocator);
    defer builder.deinit();

    _ = builder.html("<script>window.scrollTo(10, 20);</script>");
    builder.scrollFailure("scroll blocked");

    try std.testing.expectError(error.MockError, builder.build());
}
```

## Storage Example

```zig
const std = @import("std");
const bt = @import("browser_tester_zig");

pub fn main() !void {
    var builder = bt.Harness.builder(std.heap.page_allocator);
    defer builder.deinit();

    _ = builder.html("<main id='out'></main><script>const local = window.localStorage; const session = window.sessionStorage; local.setItem('theme', 'dark'); session.setItem('scratch', 'xyz'); document.getElementById('out').textContent = local.getItem('token') + ':' + session.getItem('session-token') + '|' + local.getItem('theme') + ':' + session.getItem('scratch');</script>");
    try builder.addLocalStorage("token", "abc");
    try builder.addSessionStorage("session-token", "seed");

    var harness = try builder.build();
    defer harness.deinit();

    try harness.assertValue("#out", "abc:seed|dark:xyz");
    try std.testing.expectEqualStrings("dark", harness.mocksMut().storage().local().get("theme").?);
    try std.testing.expectEqualStrings("xyz", harness.mocksMut().storage().session().get("scratch").?);
}
```

## Capture Model

Call capture records the inputs requested by the test:

- `fetch.calls()` records requested URLs
- `dialogs.alertMessages()`, `confirmMessages()`, and `promptMessages()` record dialog text
- `location.navigations()` records navigated URLs
- `open.calls()` records requested popup URLs, targets, and feature strings from `Harness.open(...)` and inline `window.open(...)`
- `close.calls()` records close invocations from `Harness.close()` and inline `window.close()`
- `print.calls()` records print invocations from `Harness.print()` and inline `window.print()`
- `scroll.calls()` records scroll method and coordinate pairs from `Harness.scrollTo(...)`, `Harness.scrollBy(...)`, and inline `window.scrollTo(...)` / `window.scrollBy(...)`
- `fileInput.selections()` records selector/file lists

Artifact capture records the side effects a test needs to inspect:

- `fetch.respondText(...)` injects a deterministic response
- `fetch.fail(...)` injects a deterministic failure
- `clipboard.writes()` records written clipboard values and keeps the latest value available for subsequent reads
- `downloads.artifacts()` records captured file names and bytes
- `storage.local()` and `storage.session()` hold seeded key/value pairs for deterministic reads and reflect `setItem(...)`, `removeItem(...)`, and `clear()` mutations made through the script-side storage objects; those mutations also dispatch `storage` events to the window

The same capture model is what keeps the mock families predictable without exposing browser internals.

`matchMedia()` is seeded by exact query string:

- `matchMedia.seedMatch(query, matches)` injects the query result
- `matchMedia.fail(query)` injects an explicit failure for the query
- `matchMedia.calls()` records requested queries in order
- existing `MediaQueryList.matches` reads the current seeded state when it is read again later in the same harness
- `MediaQueryList.addListener(callback)` and `MediaQueryList.removeListener(callback)` are available for legacy change observation on seeded queries
- `MediaQueryList.addEventListener('change', callback)` and `MediaQueryList.removeEventListener('change', callback)` are available for event-target style change observation on seeded queries
- assigning `MediaQueryList.onchange = callback` installs a single change handler for that query; assigning `null` clears it

## Failure Semantics

The public mock API fails explicitly when the test has not seeded the required state:

- `Harness.fetch()` returns `error.MockError` if no matching response or failure rule exists
- `Harness.confirm()` and `Harness.prompt()` return `error.MockError` when the queue is empty
- `Harness.readClipboard()` returns `error.MockError` when clipboard text has not been seeded
- `Harness.captureDownload()` returns `error.MockError` for blank file names
- `Harness.open()` / `Harness.close()` / `Harness.print()` return `error.MockError` when the corresponding mock family was seeded to fail
- `Harness.scrollTo()` / `Harness.scrollBy()` return `error.MockError` when the corresponding mock family was seeded to fail
- `window.open()` / `window.close()` / `window.print()` / `window.scrollTo()` / `window.scrollBy()` return `error.MockError` during bootstrap when `HarnessBuilder.openFailure(...)` / `HarnessBuilder.closeFailure(...)` / `HarnessBuilder.printFailure(...)` / `HarnessBuilder.scrollFailure(...)` were used
- `input.files` returns `error.ScriptRuntime` when read from a non-file input
- `window.matchMedia()` returns `error.MockError` when no matching rule exists or a failure rule was seeded
- `MediaQueryList.addListener(...)`, `MediaQueryList.removeListener(...)`, `MediaQueryList.addEventListener(...)`, `MediaQueryList.removeEventListener(...)`, and `MediaQueryList.onchange = ...` return `error.ScriptRuntime` on wrong arity or non-callable arguments
- `window.localStorage.setItem(...)`, `window.localStorage.removeItem(...)`, `window.localStorage.clear()`, `window.sessionStorage.setItem(...)`, `window.sessionStorage.removeItem(...)`, and `window.sessionStorage.clear()` return `error.ScriptRuntime` when called with the wrong arity or on unsupported members
- `window.localStorage.key(index)` and `window.sessionStorage.key(index)` return `null` when the index is out of range
- `Harness.advanceTime(-1)` returns `error.TimerError`
- `Harness.setFiles()` returns `error.DomError` when the target is not a file input

## Reset Semantics

`MockRegistry.resetAll()` clears every family:

- fetch response rules, error rules, and call capture
- dialog queues and capture logs
- clipboard seed and write capture
- location current URL and navigation capture
- open call capture and failure state
- close call capture and failure state
- print call capture and failure state
- scroll call capture and failure state
- matchMedia query rules and call capture
- download artifacts
- file-input selections
- storage seeds and the resulting script-side storage state

That makes it safe to reuse the same harness in a test loop without carrying mock state across scenarios.
