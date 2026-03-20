# browser-tester zig

This directory is a clean-room Zig rewrite workspace for `browser-tester`.
The rewrite follows [`next.md`](../next.md) and keeps the public surface small while the internal phases are built out.

Current state:

- phase 0 scaffold plus the internal phase 1 DOM bootstrap and selector slices
- `Harness` and `HarnessBuilder` are available
- `Session` stays internal and owns the copied configuration state plus the DOM store
- `DomStore` builds, selects, and dumps HTML trees for tests, but it is not part of the public API
- deterministic selector, script, event, timer, and mock runtime pieces are still planned

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
    var harness = try bt.Harness.fromHtml(std.heap.page_allocator, "<p>Hello</p>");
    defer harness.deinit();

    std.debug.print("url={s}\n", .{harness.url()});
}
```

## Public Surface

- `Harness`
- `HarnessBuilder`
- `StorageSeed`
- `Error`
- `Result(T)`

## Docs

- [Architecture](doc/architecture.md)
- [Capability Matrix](doc/capability-matrix.md)
- [Implementation Guide](doc/implementation-guide.md)
- [Subsystem Map](doc/subsystem-map.md)
- [Mock Guide](doc/mock-guide.md)
- [Limitations](doc/limitations.md)
- [Roadmap](doc/roadmap.md)
- [Publish Checklist](doc/publish-checklist.md)
