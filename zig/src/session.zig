const std = @import("std");

const dom = @import("dom.zig");
const errors = @import("errors.zig");

pub const StorageSeed = struct {
    key: []const u8,
    value: []const u8,
};

pub const SessionConfig = struct {
    url: []const u8,
    html: ?[]const u8 = null,
    local_storage: []const StorageSeed = &.{},
};

pub const Session = struct {
    arena: std.heap.ArenaAllocator,
    config: SessionConfig,
    dom_store: dom.DomStore,

    pub fn init(allocator: std.mem.Allocator, config: SessionConfig) errors.Result(Session) {
        var arena = std.heap.ArenaAllocator.init(allocator);
        errdefer arena.deinit();

        const arena_alloc = arena.allocator();
        const url_copy = try arena_alloc.dupe(u8, config.url);
        const html_copy = if (config.html) |html_source|
            try arena_alloc.dupe(u8, html_source)
        else
            null;

        const storage_copy = try arena_alloc.alloc(StorageSeed, config.local_storage.len);
        for (config.local_storage, 0..) |seed, index| {
            storage_copy[index] = .{
                .key = try arena_alloc.dupe(u8, seed.key),
                .value = try arena_alloc.dupe(u8, seed.value),
            };
        }

        var dom_store = try dom.DomStore.init(allocator);
        errdefer dom_store.deinit();
        if (html_copy) |html_source| {
            try dom_store.bootstrapHtml(html_source);
        }

        return .{
            .arena = arena,
            .config = .{
                .url = url_copy,
                .html = html_copy,
                .local_storage = storage_copy,
            },
            .dom_store = dom_store,
        };
    }

    pub fn deinit(self: *Session) void {
        self.dom_store.deinit();
        self.arena.deinit();
    }

    pub fn url(self: Session) []const u8 {
        return self.config.url;
    }

    pub fn html(self: Session) ?[]const u8 {
        return self.config.html;
    }

    pub fn localStorage(self: Session) []const StorageSeed {
        return self.config.local_storage;
    }

    pub fn domStore(self: *Session) *dom.DomStore {
        return &self.dom_store;
    }
};

test "session boots html into the dom store" {
    const allocator = std.testing.allocator;
    var subject = try Session.init(allocator, .{
        .url = "https://app.local/",
        .html = "<main id='app'><span>Hello</span></main>",
        .local_storage = &.{},
    });
    defer subject.deinit();

    try std.testing.expectEqual(@as(usize, 4), subject.domStore().nodeCount());

    const dump = try subject.domStore().dumpDom(allocator);
    defer allocator.free(dump);

    try std.testing.expectEqualStrings(
        "#document\n  <main id=\"app\">\n    <span>\n      \"Hello\"\n    </span>\n  </main>\n",
        dump,
    );
}
