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
