const std = @import("std");
const errors = @import("errors.zig");
const harness = @import("harness.zig");
const session = @import("session.zig");

pub const Error = errors.Error;
pub const Result = errors.Result;
pub const StorageSeed = session.StorageSeed;
pub const HarnessBuilder = harness.HarnessBuilder;
pub const Harness = harness.Harness;

test "contract: Harness.fromHtml keeps the default URL" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<p>Hello</p>");
    defer subject.deinit();

    try std.testing.expectEqualStrings("https://app.local/", subject.url());
    try std.testing.expectEqualStrings("<p>Hello</p>", subject.html().?);
}

test "contract: Harness.fromHtmlWithUrlAndLocalStorage keeps explicit configuration" {
    const allocator = std.testing.allocator;
    const seeds = [_]StorageSeed{
        .{
            .key = "theme",
            .value = "dark",
        },
    };

    var subject = try Harness.fromHtmlWithUrlAndLocalStorage(
        allocator,
        "https://app.local/tests",
        "<p>Ready</p>",
        &seeds,
    );
    defer subject.deinit();

    try std.testing.expectEqualStrings("https://app.local/tests", subject.url());
    try std.testing.expectEqualStrings("<p>Ready</p>", subject.html().?);
    try std.testing.expectEqual(@as(usize, 1), subject.localStorage().len);
    try std.testing.expectEqualStrings("theme", subject.localStorage()[0].key);
    try std.testing.expectEqualStrings("dark", subject.localStorage()[0].value);
}

test "failure: malformed html is rejected" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.HtmlParse,
        Harness.fromHtml(allocator, "<main><span></main>"),
    );
}
