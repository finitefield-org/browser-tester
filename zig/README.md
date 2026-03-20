# browser-tester zig

This directory is a clean-room Zig rewrite workspace for `browser-tester`.
The rewrite follows [`next.md`](../next.md) and keeps the public surface small while the internal phases are built out.

Current state:

- phase 0 scaffold plus the internal phase 1 DOM bootstrap and selector slices, plus the phase 2 script runtime minimum slice
- `Harness.assertExists(...)` and `Harness.dumpDom(...)` are available for read-only inspection
- inline `<script>` bootstrapping runs during `Harness.fromHtml(...)` construction for the `document.getElementById(...).textContent = ...` slice
- `Harness` and `HarnessBuilder` are available
- `Session` stays internal and owns the copied configuration state plus the DOM store and script runtime state
- `DomStore` builds, selects, and dumps HTML trees for tests, but it is not part of the public API
- deterministic selector expansion, event, timer, and mock runtime pieces are still planned

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
        "<main id='out'>Before</main><script>document.getElementById('out').textContent = 'Hello';</script>",
    );
    defer harness.deinit();

    try harness.assertExists("#out");
    const dom = try harness.dumpDom(std.heap.page_allocator);
    defer std.heap.page_allocator.free(dom);

    std.debug.print("{s}\n", .{dom});
}
```

## Public Surface

- `Harness`
- `HarnessBuilder`
- `StorageSeed`
- `Error`
- `Result(T)`
- `Error` currently includes `InvalidUrl`, `InvalidSelector`, `AssertionFailed`, `ScriptParse`, `ScriptRuntime`, `HtmlParse`, and `OutOfMemory`
- `Harness.assertExists(selector)`
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
