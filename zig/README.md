# browser-tester zig

This directory is a clean-room Zig rewrite workspace for `browser-tester`.
The rewrite follows [`next.md`](../next.md) and keeps the public surface small while the internal phases are built out.

Current state:

- phase 0 scaffold only
- `Harness` and `HarnessBuilder` are available
- `Session` stays internal and owns the copied configuration state
- deterministic DOM, script, event, timer, and mock runtime pieces are planned but not yet implemented

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

