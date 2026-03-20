const std = @import("std");
const errors = @import("errors.zig");
const session = @import("session.zig");

const default_url = "https://app.local/";

fn isBlank(text: []const u8) bool {
    return std.mem.trim(u8, text, " \t\r\n").len == 0;
}

pub const HarnessBuilder = struct {
    allocator: std.mem.Allocator,
    url_value: ?[]const u8 = null,
    html_value: ?[]const u8 = null,
    local_storage: std.ArrayListUnmanaged(session.StorageSeed) = .{},

    pub fn init(allocator: std.mem.Allocator) HarnessBuilder {
        return .{ .allocator = allocator };
    }

    pub fn deinit(self: *HarnessBuilder) void {
        self.local_storage.deinit(self.allocator);
    }

    pub fn url(self: *HarnessBuilder, value: []const u8) *HarnessBuilder {
        self.url_value = value;
        return self;
    }

    pub fn html(self: *HarnessBuilder, value: []const u8) *HarnessBuilder {
        self.html_value = value;
        return self;
    }

    pub fn addLocalStorage(
        self: *HarnessBuilder,
        key: []const u8,
        value: []const u8,
    ) errors.Result(void) {
        try self.local_storage.append(self.allocator, .{
            .key = key,
            .value = value,
        });
    }

    pub fn build(self: *HarnessBuilder) errors.Result(Harness) {
        const url_source = self.url_value orelse default_url;
        if (isBlank(url_source)) {
            return error.InvalidUrl;
        }

        const session_instance = try session.Session.init(
            self.allocator,
            .{
                .url = url_source,
                .html = self.html_value,
                .local_storage = self.local_storage.items,
            },
        );
        return Harness{
            .session = session_instance,
        };
    }
};

pub const Harness = struct {
    session: session.Session,

    pub fn builder(allocator: std.mem.Allocator) HarnessBuilder {
        return HarnessBuilder.init(allocator);
    }

    pub fn fromHtml(
        allocator: std.mem.Allocator,
        html_source: []const u8,
    ) errors.Result(Harness) {
        var subject_builder = HarnessBuilder.init(allocator);
        defer subject_builder.deinit();
        _ = subject_builder.html(html_source);
        return try subject_builder.build();
    }

    pub fn fromHtmlWithUrl(
        allocator: std.mem.Allocator,
        url_source: []const u8,
        html_source: []const u8,
    ) errors.Result(Harness) {
        var subject_builder = HarnessBuilder.init(allocator);
        defer subject_builder.deinit();
        _ = subject_builder.url(url_source);
        _ = subject_builder.html(html_source);
        return try subject_builder.build();
    }

    pub fn fromHtmlWithLocalStorage(
        allocator: std.mem.Allocator,
        html_source: []const u8,
        seeds: []const session.StorageSeed,
    ) errors.Result(Harness) {
        var subject_builder = HarnessBuilder.init(allocator);
        defer subject_builder.deinit();
        _ = subject_builder.html(html_source);
        for (seeds) |seed| {
            try subject_builder.addLocalStorage(seed.key, seed.value);
        }
        return try subject_builder.build();
    }

    pub fn fromHtmlWithUrlAndLocalStorage(
        allocator: std.mem.Allocator,
        url_source: []const u8,
        html_source: []const u8,
        seeds: []const session.StorageSeed,
    ) errors.Result(Harness) {
        var subject_builder = HarnessBuilder.init(allocator);
        defer subject_builder.deinit();
        _ = subject_builder.url(url_source);
        _ = subject_builder.html(html_source);
        for (seeds) |seed| {
            try subject_builder.addLocalStorage(seed.key, seed.value);
        }
        return try subject_builder.build();
    }

    pub fn deinit(self: *Harness) void {
        self.session.deinit();
    }

    pub fn url(self: Harness) []const u8 {
        return self.session.url();
    }

    pub fn html(self: Harness) ?[]const u8 {
        return self.session.html();
    }

    pub fn localStorage(self: Harness) []const session.StorageSeed {
        return self.session.localStorage();
    }

    pub fn assertExists(self: *const Harness, selector: []const u8) errors.Result(void) {
        const matches = self.session.domStore().select(std.heap.page_allocator, selector) catch |err| switch (err) {
            error.HtmlParse => return error.InvalidSelector,
            error.OutOfMemory => return error.OutOfMemory,
            else => unreachable,
        };
        defer std.heap.page_allocator.free(matches);

        if (matches.len == 0) {
            return error.AssertionFailed;
        }
    }

    pub fn dumpDom(
        self: *const Harness,
        allocator: std.mem.Allocator,
    ) errors.Result([]u8) {
        return self.session.domStore().dumpDom(allocator);
    }
};

test "failure: blank url is rejected" {
    const allocator = std.testing.allocator;
    var builder = HarnessBuilder.init(allocator);
    defer builder.deinit();

    _ = builder.url("   ");

    try std.testing.expectError(error.InvalidUrl, builder.build());
}

test "regression: builder copies caller-provided html" {
    const allocator = std.testing.allocator;
    var html_bytes = [_]u8{ '<', 'p', '>', 'A', '<', '/', 'p', '>' };

    var subject = try Harness.fromHtml(allocator, html_bytes[0..]);
    defer subject.deinit();

    html_bytes[3] = 'B';

    try std.testing.expectEqualStrings("<p>A</p>", subject.html().?);
}

test "regression: read-only inspection uses the copied html snapshot" {
    const allocator = std.testing.allocator;
    var html_bytes = [_]u8{ '<', 'm', 'a', 'i', 'n', ' ', 'i', 'd', '=', '\'', 'a', 'p', 'p', '\'', '>', '<', 's', 'p', 'a', 'n', '>', 'H', 'i', '<', '/', 's', 'p', 'a', 'n', '>', '<', '/', 'm', 'a', 'i', 'n', '>' };

    var subject = try Harness.fromHtml(allocator, html_bytes[0..]);
    defer subject.deinit();

    html_bytes[10] = 'z';

    try subject.assertExists("#app");
    const dump = try subject.dumpDom(allocator);
    defer allocator.free(dump);

    try std.testing.expectEqualStrings(
        "#document\n  <main id=\"app\">\n    <span>\n      \"Hi\"\n    </span>\n  </main>\n",
        dump,
    );
    try std.testing.expectEqualStrings("<main id='app'><span>Hi</span></main>", subject.html().?);
}

test "regression: inline scripts execute against the copied html snapshot" {
    const allocator = std.testing.allocator;
    var html_bytes = [_]u8{ '<', 'm', 'a', 'i', 'n', ' ', 'i', 'd', '=', '\'', 'o', 'u', 't', '\'', '>', 'B', 'e', 'f', 'o', 'r', 'e', '<', '/', 'm', 'a', 'i', 'n', '>', '<', 's', 'c', 'r', 'i', 'p', 't', '>', 'd', 'o', 'c', 'u', 'm', 'e', 'n', 't', '.', 'g', 'e', 't', 'E', 'l', 'e', 'm', 'e', 'n', 't', 'B', 'y', 'I', 'd', '(', '\'', 'o', 'u', 't', '\'', ')', '.', 't', 'e', 'x', 't', 'C', 'o', 'n', 't', 'e', 'n', 't', ' ', '=', ' ', '\'', 'H', 'e', 'l', 'l', 'o', '\'', ';', '<', '/', 's', 'c', 'r', 'i', 'p', 't', '>' };

    var subject = try Harness.fromHtml(allocator, html_bytes[0..]);
    defer subject.deinit();

    html_bytes[16] = 'Z';

    const dump = try subject.dumpDom(allocator);
    defer allocator.free(dump);

    try std.testing.expectEqualStrings(
        "#document\n  <main id=\"out\">\n    \"Hello\"\n  </main>\n  <script>\n    \"document.getElementById('out').textContent = 'Hello';\"\n  </script>\n",
        dump,
    );
    try std.testing.expectEqualStrings("<main id='out'>Before</main><script>document.getElementById('out').textContent = 'Hello';</script>", subject.html().?);
}
