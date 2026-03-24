const std = @import("std");
const errors = @import("errors.zig");
const harness = @import("harness.zig");
const mocks = @import("mocks.zig");
const session = @import("session.zig");

pub const Error = errors.Error;
pub const Result = errors.Result;
pub const StorageSeed = session.StorageSeed;
pub const HarnessBuilder = harness.HarnessBuilder;
pub const Harness = harness.Harness;
pub const MockRegistry = mocks.MockRegistry;
pub const FetchMocks = mocks.FetchMocks;
pub const FetchResponseRule = mocks.FetchResponseRule;
pub const FetchErrorRule = mocks.FetchErrorRule;
pub const FetchCall = mocks.FetchCall;
pub const FetchResponse = mocks.FetchResponse;
pub const DialogMocks = mocks.DialogMocks;
pub const ClipboardMocks = mocks.ClipboardMocks;
pub const OpenCall = mocks.OpenCall;
pub const OpenMocks = mocks.OpenMocks;
pub const CloseCall = mocks.CloseCall;
pub const CloseMocks = mocks.CloseMocks;
pub const PrintCall = mocks.PrintCall;
pub const PrintMocks = mocks.PrintMocks;
pub const ScrollMethod = mocks.ScrollMethod;
pub const ScrollCall = mocks.ScrollCall;
pub const ScrollMocks = mocks.ScrollMocks;
pub const LocationMocks = mocks.LocationMocks;
pub const MatchMediaMocks = mocks.MatchMediaMocks;
pub const MatchMediaRule = mocks.MatchMediaRule;
pub const MatchMediaCall = mocks.MatchMediaCall;
pub const DownloadMocks = mocks.DownloadMocks;
pub const DownloadCapture = mocks.DownloadCapture;
pub const FileInputMocks = mocks.FileInputMocks;
pub const FileInputSelection = mocks.FileInputSelection;
pub const StorageSeeds = mocks.StorageSeeds;

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
    try std.testing.expectEqualStrings(
        "https://app.local/tests",
        subject.mocksMut().location().currentUrl().?,
    );
    try std.testing.expectEqualStrings(
        "dark",
        subject.mocksMut().storage().local().get("theme").?,
    );
}

test "contract: Harness.fromHtmlWithSessionStorage keeps explicit configuration" {
    const allocator = std.testing.allocator;
    const seeds = [_]StorageSeed{
        .{
            .key = "session-token",
            .value = "seed",
        },
    };

    var subject = try Harness.fromHtmlWithSessionStorage(
        allocator,
        "<main id='out'></main><script>const session = window.sessionStorage; document.getElementById('out').textContent = String(session) + ':' + String(session.length) + ':' + session.getItem('session-token') + ':' + session.key(0);</script>",
        &seeds,
    );
    defer subject.deinit();

    try std.testing.expectEqualStrings("https://app.local/", subject.url());
    try subject.assertValue("#out", "[object Storage]:1:seed:session-token");
    try std.testing.expectEqualStrings(
        "seed",
        subject.mocksMut().storage().session().get("session-token").?,
    );
}

test "contract: Harness.fromHtmlWithUrlAndSessionStorage keeps explicit configuration" {
    const allocator = std.testing.allocator;
    const seeds = [_]StorageSeed{
        .{
            .key = "session-token",
            .value = "seed",
        },
    };

    var subject = try Harness.fromHtmlWithUrlAndSessionStorage(
        allocator,
        "https://app.local/tests",
        "<main id='out'></main><script>const session = window.sessionStorage; session.setItem('scratch', 'xyz'); document.getElementById('out').textContent = session.getItem('session-token') + ':' + session.getItem('scratch') + ':' + String(session.length) + ':' + session.key(1);</script>",
        &seeds,
    );
    defer subject.deinit();

    try std.testing.expectEqualStrings("https://app.local/tests", subject.url());
    try subject.assertValue("#out", "seed:xyz:2:scratch");
    try std.testing.expectEqualStrings(
        "seed",
        subject.mocksMut().storage().session().get("session-token").?,
    );
    try std.testing.expectEqualStrings(
        "xyz",
        subject.mocksMut().storage().session().get("scratch").?,
    );
}

test "contract: HarnessBuilder.addSessionStorage keeps explicit configuration" {
    const allocator = std.testing.allocator;
    var builder = Harness.builder(allocator);
    defer builder.deinit();

    _ = builder.url("https://app.local/tests");
    _ = builder.html("<main id='out'></main><script>const local = window.localStorage; const session = window.sessionStorage; const before = String(local) + ':' + String(session) + ':' + String(local.length) + ':' + String(session.length); const token = local.getItem('token'); const sessionToken = session.getItem('session-token'); local.setItem('theme', 'dark'); local.removeItem('token'); session.setItem('scratch', 'xyz'); const sessionKey = session.key(1); session.clear(); document.getElementById('out').textContent = before + '|' + token + ':' + sessionToken + ':' + local.getItem('theme') + ':' + String(local.length) + ':' + String(local.key(0)) + ':' + String(session.length) + ':' + String(sessionKey);</script>");
    try builder.addLocalStorage("token", "abc");
    try builder.addSessionStorage("session-token", "xyz");

    var subject = try builder.build();
    defer subject.deinit();

    try std.testing.expectEqualStrings("https://app.local/tests", subject.url());
    try subject.assertValue("#out", "[object Storage]:[object Storage]:1:1|abc:xyz:dark:1:theme:0:scratch");
    try std.testing.expectEqualStrings(
        "dark",
        subject.mocksMut().storage().local().get("theme").?,
    );
    try std.testing.expectEqual(@as(?[]const u8, null), subject.mocksMut().storage().local().get("token"));
    try std.testing.expectEqual(@as(?[]const u8, null), subject.mocksMut().storage().session().get("session-token"));
}

test "contract: Harness.nowMs and Harness.advanceTime expose fake clock" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<main></main>");
    defer subject.deinit();

    try std.testing.expectEqual(@as(i64, 0), subject.nowMs());
    try subject.advanceTime(25);
    try std.testing.expectEqual(@as(i64, 25), subject.nowMs());
    try subject.flush();
}

test "contract: queueMicrotask drains during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='out'></main><script>document.getElementById('out').textContent = 'start'; queueMicrotask(() => { document.getElementById('out').textContent = 'done'; });</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "done");
}

test "contract: setTimeout and clearTimeout drive the timer queue" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='out'></main><script>document.getElementById('out').textContent = 'start'; const cancelled = setTimeout(() => { document.getElementById('out').textContent = 'cancelled'; }, 10); clearTimeout(cancelled); window.setTimeout(() => { document.getElementById('out').textContent = 'timeout'; queueMicrotask(() => { document.getElementById('out').textContent = 'drained'; }); }, 5);</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "start");
    try subject.advanceTime(4);
    try subject.assertValue("#out", "start");
    try subject.advanceTime(1);
    try subject.assertValue("#out", "drained");
}

test "contract: setInterval and clearInterval drive the repeating timer queue" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='out'></main><script>document.getElementById('out').textContent = 'start'; const repeating = setInterval(() => { document.getElementById('out').textContent = document.getElementById('out').textContent + 'x'; }, 5); const cancelledByGlobal = setInterval(() => { document.getElementById('out').textContent = 'global'; }, 7); clearInterval(cancelledByGlobal); const cancelledByWindow = window.setInterval(() => { document.getElementById('out').textContent = 'window'; }, 9); window.clearInterval(cancelledByWindow);</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "start");
    try subject.advanceTime(5);
    try subject.assertValue("#out", "startx");
    try subject.advanceTime(5);
    try subject.assertValue("#out", "startxx");
}

test "contract: requestAnimationFrame and cancelAnimationFrame drive the frame queue" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='out'></main><script>document.getElementById('out').textContent = 'start'; const cancelled = requestAnimationFrame(() => { document.getElementById('out').textContent = 'cancelled'; }); cancelAnimationFrame(cancelled); window.requestAnimationFrame(() => { document.getElementById('out').textContent = document.getElementById('out').textContent + ':raf'; });</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "start");
    try subject.advanceTime(15);
    try subject.assertValue("#out", "start");
    try subject.advanceTime(1);
    try subject.assertValue("#out", "start:raf");
}

test "failure: Harness.advanceTime rejects negative deltas" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<main></main>");
    defer subject.deinit();

    try std.testing.expectError(error.TimerError, subject.advanceTime(-1));
    try std.testing.expectEqual(@as(i64, 0), subject.nowMs());
}

test "failure: window.localStorage.setItem rejects missing arguments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.localStorage.setItem('theme')</script>"),
    );
}

test "failure: queueMicrotask rejects non-function callbacks" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>queueMicrotask(1)</script>"),
    );
}

test "failure: window.setTimeout rejects non-function callbacks" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.setTimeout(1, 0)</script>"),
    );
}

test "failure: window.setInterval rejects non-function callbacks" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.setInterval(1, 0)</script>"),
    );
}

test "failure: requestAnimationFrame rejects missing callbacks" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>requestAnimationFrame()</script>"),
    );
}

test "contract: Harness.mocksMut exposes fetch, dialogs, clipboard, open, close, print, scroll, location, downloads, and storage" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<main></main>");
    defer subject.deinit();

    {
        const mocks_view = subject.mocksMut();
        try mocks_view.fetch().respondText("https://example.test/api/message", 201, "ok");
        try mocks_view.dialogs().pushConfirm(true);
        try mocks_view.dialogs().pushPrompt("Ada");
        try mocks_view.clipboard().seedText("seeded");
        try mocks_view.storage().seedLocal("token", "abc");
        try mocks_view.storage().seedSession("session-token", "xyz");
    }

    const response = try subject.fetch("https://example.test/api/message");
    try std.testing.expectEqualStrings("https://example.test/api/message", response.url);
    try std.testing.expectEqual(@as(u16, 201), response.status);
    try std.testing.expectEqualStrings("ok", response.body);
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().fetch().calls().len);
    try std.testing.expectEqualStrings(
        "https://example.test/api/message",
        subject.mocksMut().fetch().calls()[0].url,
    );

    try subject.alert("Notice");
    try std.testing.expect(try subject.confirm("Continue?"));
    const prompt = try subject.prompt("Name?");
    try std.testing.expectEqualStrings("Ada", prompt.?);
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().dialogs().alertMessages().len);
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().dialogs().confirmMessages().len);
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().dialogs().promptMessages().len);

    try std.testing.expectEqualStrings("seeded", try subject.readClipboard());
    try subject.writeClipboard("copied");
    try std.testing.expectEqualStrings("copied", try subject.readClipboard());
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().clipboard().writes().len);
    try std.testing.expectEqualStrings("copied", subject.mocksMut().clipboard().writes()[0]);

    try subject.navigate(" https://example.test/next ");
    try std.testing.expectEqualStrings(
        "https://example.test/next",
        subject.mocksMut().location().currentUrl().?,
    );
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().location().navigations().len);
    try std.testing.expectEqualStrings(
        "https://example.test/next",
        subject.mocksMut().location().navigations()[0],
    );

    try subject.captureDownload("report.csv", "downloaded bytes");
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().downloads().artifacts().len);
    try std.testing.expectEqualStrings(
        "report.csv",
        subject.mocksMut().downloads().artifacts()[0].file_name,
    );
    try std.testing.expectEqualStrings(
        "downloaded bytes",
        subject.mocksMut().downloads().artifacts()[0].bytes,
    );

    try std.testing.expectEqualStrings(
        "abc",
        subject.mocksMut().storage().local().get("token").?,
    );
    try std.testing.expectEqualStrings(
        "xyz",
        subject.mocksMut().storage().session().get("session-token").?,
    );
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().open().calls().len);
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().close().calls().len);
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().print().calls().len);
}

test "contract: Harness.mocksMut.resetAll clears every family" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<main></main>");
    defer subject.deinit();

    {
        const mocks_view = subject.mocksMut();
        try mocks_view.fetch().respondText("https://example.test/api/message", 201, "ok");
        try mocks_view.dialogs().pushConfirm(true);
        try mocks_view.dialogs().pushPrompt("Ada");
        try mocks_view.dialogs().recordAlert("Notice");
        try mocks_view.clipboard().seedText("seeded");
        try mocks_view.clipboard().recordWrite("copied");
        try mocks_view.open().fail("popup blocked");
        try mocks_view.close().fail("window closed");
        try mocks_view.print().fail("print blocked");
        try mocks_view.scroll().fail("scroll blocked");
        try mocks_view.location().setCurrent("https://example.test/next");
        try mocks_view.location().recordNavigation("https://example.test/next");
        try mocks_view.downloads().capture("report.csv", "downloaded bytes");
        try mocks_view.fileInput().setFiles("#upload", &.{"report.csv"});
        try mocks_view.storage().seedLocal("token", "abc");
        try mocks_view.storage().seedSession("session-token", "xyz");
    }

    try std.testing.expectError(
        error.MockError,
        subject.open("https://example.test/popup"),
    );
    try std.testing.expectError(error.MockError, subject.close());
    try std.testing.expectError(error.MockError, subject.print());
    try std.testing.expectError(error.MockError, subject.scrollTo(10, 20));

    subject.mocksMut().resetAll();

    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().fetch().responses().len);
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().fetch().errors().len);
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().fetch().calls().len);
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().dialogs().confirmQueue().len);
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().dialogs().promptQueue().len);
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().dialogs().alertMessages().len);
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().dialogs().confirmMessages().len);
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().dialogs().promptMessages().len);
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().clipboard().writes().len);
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().open().calls().len);
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().close().calls().len);
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().print().calls().len);
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().scroll().calls().len);
    try std.testing.expectEqual(@as(?[]const u8, null), subject.mocksMut().location().currentUrl());
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().location().navigations().len);
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().downloads().artifacts().len);
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().fileInput().selections().len);
    try std.testing.expectEqual(@as(?[]const u8, null), subject.mocksMut().storage().local().get("token"));
    try std.testing.expectEqual(@as(?[]const u8, null), subject.mocksMut().storage().session().get("session-token"));
    try std.testing.expectError(error.MockError, subject.readClipboard());
}

test "contract: Harness.open, Harness.close, and Harness.print record calls through the registry" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<main></main>");
    defer subject.deinit();

    try subject.open("https://example.test/popup");
    try subject.print();

    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().open().calls().len);
    try std.testing.expectEqualStrings(
        "https://example.test/popup",
        subject.mocksMut().open().calls()[0].url.?,
    );
    try std.testing.expectEqual(@as(?[]const u8, null), subject.mocksMut().open().calls()[0].target);
    try std.testing.expectEqual(@as(?[]const u8, null), subject.mocksMut().open().calls()[0].features);
    try subject.close();
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().close().calls().len);
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().print().calls().len);
}

test "contract: Harness.print dispatches beforeprint and afterprint handlers" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='out'></main><script>window.onbeforeprint = () => { document.getElementById('out').textContent += 'before|'; }; window.onafterprint = () => { document.getElementById('out').textContent += 'after|'; };</script>",
    );
    defer subject.deinit();

    try subject.print();
    try subject.assertValue("#out", "before|after|");
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().print().calls().len);
}

test "contract: Harness.scrollTo and Harness.scrollBy record calls through the registry" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<main></main>");
    defer subject.deinit();

    try subject.scrollTo(10, 20);
    try subject.scrollBy(-5, 3);

    try std.testing.expectEqual(@as(usize, 2), subject.mocksMut().scroll().calls().len);
    try std.testing.expectEqual(.To, subject.mocksMut().scroll().calls()[0].method);
    try std.testing.expectEqual(@as(i64, 10), subject.mocksMut().scroll().calls()[0].x);
    try std.testing.expectEqual(@as(i64, 20), subject.mocksMut().scroll().calls()[0].y);
    try std.testing.expectEqual(.By, subject.mocksMut().scroll().calls()[1].method);
    try std.testing.expectEqual(@as(i64, -5), subject.mocksMut().scroll().calls()[1].x);
    try std.testing.expectEqual(@as(i64, 3), subject.mocksMut().scroll().calls()[1].y);
}

test "contract: Harness.scrollTo dispatches window and document scroll handlers" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='doc'></main><main id='win'></main><script>document.onscroll = () => { document.getElementById('doc').textContent = String(window.scrollX) + ':' + String(window.scrollY); }; window.onscroll = () => { document.getElementById('win').textContent = String(window.scrollX) + ':' + String(window.scrollY); };</script>",
    );
    defer subject.deinit();

    try subject.scrollTo(10, 20);

    try subject.assertValue("#doc", "10:20");
    try subject.assertValue("#win", "10:20");
}

test "contract: Harness.fromHtml exposes document referrer and dir during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<html id='html' dir='ltr'><body id='body'><main id='out'></main><script>const referrer = '[' + document.referrer + ']'; const before = document.dir; document.dir = 'rtl'; document.getElementById('out').textContent = referrer + ':' + before + ':' + document.dir + ':' + document.documentElement.getAttribute('dir');</script></body></html>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "[]:ltr:rtl:rtl");
}

test "contract: Harness.fromHtml exposes document.cookie during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='out'></main><script>document.cookie = 'theme=dark'; document.cookie = 'theme=light'; document.getElementById('out').textContent = document.cookie;</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "theme=light");
}

test "contract: Harness.fromHtml exposes window.name during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='out'></main><script>const before = window.name; window.name = 'updated'; document.getElementById('out').textContent = before + ':' + document.defaultView.name;</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", ":updated");
}

test "failure: Harness.fromHtml reports missing element access in inline scripts" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'>Before</main><script>document.getElementById('missing').textContent = 'Hello';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects unsupported script syntax" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptParse,
        Harness.fromHtml(
            allocator,
            "<main id='out'>Before</main><script>document.getElementById('out').textContent = ;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects property writes on window.name" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>window.name.length = 1;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects read-only document metadata assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<html id='html'><body id='body'><script>document.compatMode = 'BackCompat';</script></body></html>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<html id='html'><body id='body'><script>document.referrer = 'https://example.test/source';</script></body></html>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<html id='html'><body id='body'><script>document.domain = 'example.test';</script></body></html>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed document.cookie assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>document.cookie = 'badcookie';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects read-only node traversal assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<!--pre--><main id='root'><span id='first'>One</span></main><script>document.getElementById('root').firstChild = null;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-character-data data assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>document.getElementById('out').data = 'ignored';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects splitText on non-text nodes" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>document.getElementById('out').splitText(1);</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects splitText out of range" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='host'>Hello</main><script>document.getElementById('host').childNodes.item(0).splitText(99);</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects CharacterData methods on non-character-data nodes" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>document.getElementById('out').appendData('ignored');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects wholeText on non-text nodes" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>document.getElementById('out').wholeText;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects CharacterData offset out of range" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='host'>Hello</main><script>document.getElementById('host').childNodes.item(0).insertData(99, 'ignored');</script>",
        ),
    );
}

test "contract: Harness.fromHtml runs script querySelector methods during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root' class='app'><section class='panel'><span id='marker'>panel</span><button id='first' class='primary'>First</button><button id='second' class='secondary'>Second</button></section></main><div id='out'></div><script>document.getElementById('out').textContent = document.querySelector('button').textContent + ':' + document.getElementById('root').querySelector('button.secondary').textContent + ':' + String(document.getElementById('root').querySelector('main'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "First:Second:null");
}

test "contract: Harness.fromHtml runs script matches during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root' class='app'><section class='panel'><span id='marker'>panel</span><button id='first' class='primary'>First</button><button id='second' class='secondary'>Second</button></section></main><div id='out'></div><script>document.getElementById('out').textContent = String(document.querySelector('#second').matches('button.secondary')) + ':' + String(document.querySelector('#second').matches('button.primary'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "true:false");
}

test "contract: Harness.fromHtml runs script closest during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root' class='app'><section class='panel'><span id='marker'>panel</span><button id='first' class='primary'>First</button><button id='second' class='secondary'>Second</button></section></main><div id='out'></div><script>document.getElementById('out').textContent = document.querySelector('#second').closest('section.panel').querySelector('#marker').textContent;</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "panel");
}

test "contract: Harness.fromHtml resolves defined pseudo-class selectors during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><x-widget id='widget'></x-widget><svg id='svg'><text id='svg-text'>Hi</text></svg></main><div id='out'></div><script>const defined = document.querySelectorAll(':defined'); const widget = document.getElementById('widget'); const svg = document.getElementById('svg'); document.getElementById('out').textContent = defined.item(0).getAttribute('id') + ':' + defined.item(1).getAttribute('id') + ':' + defined.item(2).getAttribute('id') + ':' + String(widget.matches(':defined')) + ':' + String(svg.matches(':defined'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "root:svg:svg-text:false:true");
}

test "contract: Harness.fromHtml resolves :not, :is, and :where selectors during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='buttons'><button id='first' class='primary'>First</button><button id='second' class='secondary'>Second</button></section><div id='out'></div><script>document.getElementById('out').textContent = document.querySelector('#buttons > button:not(.missing, .secondary)').getAttribute('id') + ':' + String(document.querySelectorAll('#buttons > button:is(.primary, .secondary)').length) + ':' + String(document.querySelectorAll('#buttons > button:where(.primary, .secondary)').length) + ':' + document.querySelector('#buttons > button:where(.missing, .secondary)').getAttribute('id');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "first:2:2:second");
}

test "contract: Harness.fromHtml resolves default and indeterminate pseudo-classes during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><progress id='loading'></progress><form id='signup'><input type='radio' name='mode' id='mode-a'><input type='radio' name='mode' id='mode-b'></form><form id='chosen'><input type='radio' name='picked' id='picked-a' checked><input type='radio' name='picked' id='picked-b'></form><form id='form'><input id='submit' type='submit'><input id='agree' type='checkbox' checked><input id='mode-c' type='radio' name='mode2'><input id='mode-d' type='radio' name='mode2' checked><select id='select'><option id='first' value='a'>A</option><option id='selected' value='b' selected>B</option></select></form></main><div id='out'></div><script>const defaults = document.querySelectorAll(':default'); const indeterminate = document.querySelectorAll(':indeterminate'); document.getElementById('out').textContent = String(defaults.length) + ':' + defaults.item(0).getAttribute('id') + ':' + defaults.item(1).getAttribute('id') + ':' + defaults.item(2).getAttribute('id') + ':' + defaults.item(3).getAttribute('id') + ':' + defaults.item(4).getAttribute('id') + ':' + String(indeterminate.length) + ':' + indeterminate.item(0).getAttribute('id') + ':' + indeterminate.item(1).getAttribute('id') + ':' + indeterminate.item(2).getAttribute('id');</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "5:picked-a:submit:agree:mode-d:selected:3:loading:mode-a:mode-b");
}

test "contract: Harness.fromHtml runs map.areas and table.tBodies during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><map id='map'><area id='first-area' name='first' href='/first'><area id='second-area' name='second' href='/second'></map><table id='table'><tbody id='first-body'><tr><td>One</td></tr></tbody></table><div id='out'></div><script>const areas = document.getElementById('map').areas; const bodies = document.getElementById('table').tBodies; const beforeAreas = areas.length; const beforeBodies = bodies.length; const firstArea = areas.item(0); const firstBody = bodies.item(0); document.getElementById('map').innerHTML += '<area id=\"third-area\" name=\"third\" href=\"/third\">'; document.getElementById('table').innerHTML += '<tbody id=\"second-body\"></tbody>'; document.getElementById('out').textContent = String(beforeAreas) + ':' + String(areas.length) + ':' + String(beforeBodies) + ':' + String(bodies.length) + ':' + String(firstArea.getAttribute('id')) + ':' + String(firstBody.getAttribute('id')) + ':' + String(areas.namedItem('third-area')) + ':' + String(bodies.namedItem('second-body')) + ':' + String(areas.namedItem('missing'));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:3:1:2:first-area:first-body:[object Element]:[object Element]:null");
    try subject.assertExists("#third-area");
    try subject.assertExists("#second-body");
}

test "contract: Harness.fromHtml exposes table body creation during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<table id='table'><caption id='caption'></caption><colgroup id='group'></colgroup><tbody id='body-1'></tbody></table><div id='out'></div><script>const table = document.getElementById('table'); const before = String(table.children.length) + ':' + String(table.children.item(0).getAttribute('id')) + ':' + String(table.children.item(1).getAttribute('id')) + ':' + String(table.children.item(2).getAttribute('id')) + ':' + String(table.tBodies.length); const head = table.createTHead(); head.id = 'head'; const body2 = table.createTBody(); body2.id = 'body-2'; const foot = table.createTFoot(); foot.id = 'foot'; document.getElementById('out').textContent = before + '|' + String(table.children.length) + ':' + String(table.children.item(0).getAttribute('id')) + ':' + String(table.children.item(1).getAttribute('id')) + ':' + String(table.children.item(2).getAttribute('id')) + ':' + String(table.children.item(3).getAttribute('id')) + ':' + String(table.children.item(4).getAttribute('id')) + ':' + String(table.children.item(5).getAttribute('id')) + ':' + String(table.tBodies.length) + ':' + String(table.tHead.getAttribute('id')) + ':' + String(table.tFoot.getAttribute('id'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "3:caption:group:body-1:1|6:caption:group:head:body-1:body-2:foot:2:head:foot");
}

test "contract: Harness.fromHtml runs element.labels during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><label id='explicit-label' for='control'>Explicit</label><input id='control' value='A'><label id='implicit-label'><input id='inner-control' value='B'>Implicit</label><fieldset id='group'></fieldset><label id='group-label' for='group'>Group</label><div id='wrapper'></div><div id='out'></div><script>const control = document.getElementById('control'); const labels = control.labels; const inner = document.getElementById('inner-control').labels; const fieldset = document.getElementById('group'); const fieldsetLabels = fieldset.labels; const before = labels.length; const fieldsetBefore = fieldsetLabels.length; document.getElementById('wrapper').innerHTML = '<label id=\"second-label\" for=\"control\">Second</label><label id=\"group-second\" for=\"group\">Second Group</label>'; document.getElementById('out').textContent = String(before) + ':' + String(labels.length) + ':' + labels.item(0).getAttribute('id') + ':' + labels.item(1).textContent + ':' + String(inner.length) + ':' + inner.item(0).getAttribute('id') + ':' + String(fieldsetBefore) + ':' + String(fieldsetLabels.length) + ':' + fieldsetLabels.item(0).getAttribute('id') + ':' + fieldsetLabels.item(1).getAttribute('id');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "1:2:explicit-label:Second:1:implicit-label:1:2:group-label:group-second");
    try subject.assertExists("#second-label");
    try subject.assertExists("#group-second");
}

test "contract: Harness.fromHtml runs label.control and htmlFor during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><label id='explicit-label' for='control'>Explicit</label><input id='control' value='A'><label id='implicit-label'><input id='inner-control' value='B'>Implicit</label><div id='out'></div><script>const explicit = document.getElementById('explicit-label'); const implicit = document.getElementById('implicit-label'); const before = explicit.htmlFor + ':' + explicit.control.getAttribute('id') + ':' + String(implicit.htmlFor) + ':' + implicit.control.getAttribute('id'); explicit.htmlFor = 'inner-control'; document.getElementById('out').textContent = before + ':' + explicit.htmlFor + ':' + explicit.control.getAttribute('id');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "control:control::inner-control:inner-control:inner-control");
    try subject.assertExists("#control");
    try subject.assertExists("#inner-control");
}

test "contract: Harness.fromHtml runs HTMLLabelElement.form during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><form id='owner'></form><input id='control' form='owner' value='A'><label id='explicit' for='control'>Explicit</label><label id='implicit'><input id='inner' form='owner' value='B'>Implicit</label><label id='empty'>Empty</label><div id='out'></div><script>const explicit = document.getElementById('explicit'); const implicit = document.getElementById('implicit'); const empty = document.getElementById('empty'); document.getElementById('out').textContent = explicit.form.id + ':' + implicit.form.id + ':' + String(empty.form);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "owner:owner:null");
}

test "contract: Harness.fromHtml runs document.images and document.links during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><img id='hero' name='hero' alt='Hero'><img name='thumb' alt='Thumb'><a id='docs' href='/docs'>Docs</a><a id='plain'>Plain</a><area id='map' name='map' href='/map'><div id='out'></div><script>const images = document.images; const links = document.links; const beforeImages = images.length; const beforeLinks = links.length; const hero = images.namedItem('hero'); const thumb = images.namedItem('thumb'); const docs = links.namedItem('docs'); const map = links.namedItem('map'); const plain = links.namedItem('plain'); document.getElementById('root').innerHTML += '<img id=\"third\" name=\"third\" alt=\"Third\"><a id=\"more\" href=\"/more\">More</a>'; document.getElementById('out').textContent = String(beforeImages) + ':' + String(images.length) + ':' + String(beforeLinks) + ':' + String(links.length) + ':' + String(hero) + ':' + String(thumb) + ':' + String(docs) + ':' + String(map) + ':' + String(plain);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:3:2:3:[object Element]:[object Element]:[object Element]:[object Element]:null");
    try subject.assertExists("#third");
    try subject.assertExists("#more");
}

test "contract: Harness.fromHtml runs document.embeds, document.plugins, document.applets, and document.all during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><embed id='first-embed' name='first-embed'><embed name='second-embed'><applet id='first-applet' name='first-applet'>First</applet><div id='first'>First</div><div id='second' name='second'>Second</div><div id='out'></div><script>const embeds = document.embeds; const plugins = document.plugins; const applets = document.applets; const all = document.all; const beforeEmbeds = embeds.length; const beforePlugins = plugins.length; const beforeApplets = applets.length; const beforeAll = all.length; const firstEmbed = embeds.namedItem('first-embed'); const firstPlugin = plugins.namedItem('first-embed'); const firstApplet = applets.namedItem('first-applet'); const second = all.namedItem('second'); document.getElementById('root').innerHTML += '<embed id=\"third-embed\" name=\"third-embed\"><applet id=\"second-applet\" name=\"second-applet\">Second</applet>'; document.getElementById('out').textContent = String(beforeEmbeds) + ':' + String(embeds.length) + ':' + String(beforePlugins) + ':' + String(plugins.length) + ':' + String(beforeApplets) + ':' + String(applets.length) + ':' + String(beforeAll) + ':' + String(all.length) + ':' + String(firstEmbed) + ':' + String(firstPlugin) + ':' + String(firstApplet) + ':' + String(second);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:3:2:3:1:2:8:10:[object Element]:[object Element]:[object Element]:[object Element]");
    try subject.assertExists("#third-embed");
    try subject.assertExists("#second-applet");
}

test "failure: Harness.fromHtml rejects non-document images access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='not-doc'></div></main><script>document.getElementById('not-doc').images.length;</script>",
        ),
    );
}

test "contract: Harness.fromHtml runs defaultValue and defaultChecked during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada'><input id='agree' type='checkbox' checked><textarea id='bio'>Hello</textarea></main><div id='out'></div><script>const name = document.getElementById('name'); const agree = document.getElementById('agree'); const bio = document.getElementById('bio'); const before = name.defaultValue + ':' + String(agree.defaultChecked) + ':' + bio.defaultValue; name.defaultValue = 'Bea'; agree.defaultChecked = false; bio.defaultValue = 'World'; document.getElementById('out').textContent = before + ':' + name.defaultValue + ':' + name.value + ':' + String(name.getAttribute('value')) + ':' + String(agree.checked) + ':' + String(agree.defaultChecked) + ':' + bio.defaultValue + ':' + bio.textContent;</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "Ada:true:Hello:Bea:Bea:Bea:false:false:World:World");
}

test "contract: Harness.fromHtml runs form submission reflection during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><form id='form'><button id='button' type='submit'></button><input id='input' type='submit'></form><div id='before'></div><div id='after'></div><script>const form = document.getElementById('form'); const button = document.getElementById('button'); const input = document.getElementById('input'); document.getElementById('before').textContent = 'form=' + form.action + ':' + form.method + ':' + form.enctype + ':' + form.encoding + ':' + form.target + ':' + form.acceptCharset + '|button=' + button.formAction + ':' + button.formMethod + ':' + button.formEnctype + ':' + button.formTarget + '|input=' + input.formAction + ':' + input.formMethod + ':' + input.formEnctype + ':' + input.formTarget; form.action = '/submit'; form.method = 'POST'; form.enctype = 'multipart/form-data'; form.target = '_blank'; form.acceptCharset = 'utf-8'; button.formAction = '/button-submit'; button.formMethod = 'Dialog'; button.formEnctype = 'text/plain'; button.formTarget = '_self'; input.formAction = '/input-submit'; input.formMethod = 'POST'; input.formEnctype = 'multipart/form-data'; input.formTarget = '_parent'; document.getElementById('after').textContent = 'form=' + form.action + ':' + form.method + ':' + form.enctype + ':' + form.encoding + ':' + form.target + ':' + form.acceptCharset + ':' + form.getAttribute('action') + ':' + form.getAttribute('method') + ':' + form.getAttribute('enctype') + ':' + form.getAttribute('target') + ':' + form.getAttribute('accept-charset') + '|button=' + button.formAction + ':' + button.formMethod + ':' + button.formEnctype + ':' + button.formTarget + ':' + button.getAttribute('formaction') + ':' + button.getAttribute('formmethod') + ':' + button.getAttribute('formenctype') + ':' + button.getAttribute('formtarget') + '|input=' + input.formAction + ':' + input.formMethod + ':' + input.formEnctype + ':' + input.formTarget + ':' + input.getAttribute('formaction') + ':' + input.getAttribute('formmethod') + ':' + input.getAttribute('formenctype') + ':' + input.getAttribute('formtarget');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#before", "form=https://app.local/:get:application/x-www-form-urlencoded:application/x-www-form-urlencoded::|button=https://app.local/:get:application/x-www-form-urlencoded:|input=https://app.local/:get:application/x-www-form-urlencoded:");
    try subject.assertValue("#after", "form=https://app.local/submit:post:multipart/form-data:multipart/form-data:_blank:utf-8:/submit:post:multipart/form-data:_blank:utf-8|button=https://app.local/button-submit:dialog:text/plain:_self:/button-submit:dialog:text/plain:_self|input=https://app.local/input-submit:post:multipart/form-data:_parent:/input-submit:post:multipart/form-data:_parent");
}

test "contract: Harness.fromHtml runs form.submit and form.requestSubmit during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><form id='form'><input id='name'><button id='button' type='submit'></button></form><div id='out'></div><script>const form = document.getElementById('form'); const name = document.getElementById('name'); const button = document.getElementById('button'); form.addEventListener('submit', (event) => { event.preventDefault(); document.getElementById('out').textContent += document.getElementById('name').value + '|'; }); name.value = 'Ada'; form.submit(); form.requestSubmit(button);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "Ada|Ada|");
}

test "contract: Harness.fromHtml runs FormData and formdata during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><form id='form'><input id='name' name='name' value='Ada'><select id='choice' name='choice'><option value='alpha' selected>Alpha</option></select><button id='submit' type='submit' name='submit' value='go'></button></form><div id='out'></div><script>const form = document.getElementById('form'); const button = document.getElementById('submit'); const data = new FormData(form); const before = String(data) + ':' + data.get('name') + ':' + data.get('choice') + ':' + String(data.getAll('choice').item(0)) + ':' + String(data.has('submit')) + ':' + String(data.getAll('choice').length); data.append('extra', '1'); data.set('name', 'Bea'); data.delete('choice'); const after = String(data) + ':' + data.get('name') + ':' + String(data.has('choice')) + ':' + String(data.get('extra')) + ':' + String(data.getAll('extra').length); form.addEventListener('submit', (event) => { document.getElementById('out').textContent += 'submit:' + event.submitter.id + '|'; }); form.addEventListener('formdata', (event) => { const formData = event.formData; document.getElementById('out').textContent += String(event) + ':' + formData.get('name') + ':' + formData.get('choice') + ':' + formData.get('submit') + ':' + String(formData.has('listener')) + '|'; formData.append('listener', 'yes'); document.getElementById('out').textContent += formData.get('listener'); }); form.requestSubmit(button); document.getElementById('out').textContent += '|' + before + '|' + after;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "submit:submit|[object FormDataEvent]:Ada:alpha:go:false|yes|[object FormData]:Ada:alpha:alpha:false:1|[object FormData]:Bea:false:1:1");
}

test "contract: Harness.fromHtml runs FormData iterator parity during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const data = new FormData(); data.append('alpha', '1'); data.append('beta', '2'); data.append('alpha', '3'); const keys = data.keys(); const values = data.values(); const entries = data.entries(); const firstKey = keys.next(); const secondKey = keys.next(); const thirdKey = keys.next(); const fourthKey = keys.next(); const firstValue = values.next(); const secondValue = values.next(); const thirdValue = values.next(); const fourthValue = values.next(); const firstEntry = entries.next(); const secondEntry = entries.next(); const thirdEntry = entries.next(); const fourthEntry = entries.next(); document.getElementById('out').textContent = String(firstKey.value) + ':' + String(secondKey.value) + ':' + String(thirdKey.value) + ':' + String(fourthKey.done) + '|' + firstValue.value + ':' + secondValue.value + ':' + thirdValue.value + ':' + String(fourthValue.done) + '|' + firstEntry.value.name + ':' + firstEntry.value.value + ':' + secondEntry.value.name + ':' + secondEntry.value.value + ':' + thirdEntry.value.name + ':' + thirdEntry.value.value + ':' + String(fourthEntry.done) + '|'; data.forEach((value, name, formData) => { document.getElementById('out').textContent += name + ':' + value + ':' + formData.get(name) + ';'; }, null);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "alpha:beta:alpha:true|1:2:3:true|alpha:1:beta:2:alpha:3:true|alpha:1:1;beta:2:2;alpha:3:1;");
}

test "contract: Harness.fromHtml skips formdata when submit is canceled during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><form id='form'><input id='name' name='name' value='Ada'><button id='submit' type='submit' name='submit' value='go'></button></form><div id='out'></div><script>const form = document.getElementById('form'); const button = document.getElementById('submit'); form.addEventListener('submit', (event) => { event.preventDefault(); document.getElementById('out').textContent += 'submit|'; }); form.addEventListener('formdata', () => { document.getElementById('out').textContent += 'formdata|'; }); form.requestSubmit(button);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "submit|");
}

test "failure: Harness.fromHtml rejects FormData.forEach with a non-function callback" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='out'></div><script>const data = new FormData(); data.forEach(1);</script></main>",
        ),
    );
}

test "contract: Harness.fromHtml runs form.reset during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><form id='form'><input id='name' value='Ada'></form><div id='out'></div><script>const form = document.getElementById('form'); form.addEventListener('reset', (event) => { event.preventDefault(); document.getElementById('out').textContent = 'reset:' + String(event.bubbles) + ':' + String(event.cancelable); }); form.reset();</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "reset:true:true");
}

test "contract: Harness.fromHtml runs HTMLDialogElement.showModal and close during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><dialog id='dlg'></dialog><div id='out'></div><script>const dialog = document.getElementById('dlg'); dialog.closedBy = 'none'; dialog.addEventListener('cancel', () => { document.getElementById('out').textContent = 'cancel:' + String(document.getElementById('dlg').open) + ':' + document.getElementById('dlg').returnValue; }); dialog.addEventListener('close', () => { document.getElementById('out').textContent += '|close:' + String(document.getElementById('dlg').open) + ':' + document.getElementById('dlg').returnValue; }); dialog.showModal(); dialog.requestClose('done'); document.getElementById('out').textContent += '|after:' + String(dialog.open) + ':' + dialog.returnValue + ':' + dialog.closedBy;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "cancel:true:|close:false:done|after:false:done:none");
}

test "contract: Harness.fromHtml runs HTMLButtonElement.command activation during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><dialog id='dlg'></dialog><button id='open' type='button'>Open</button><button id='close' type='button'>Close</button><div id='out'></div><script>const dialog = document.getElementById('dlg'); const open = document.getElementById('open'); const close = document.getElementById('close'); open.command = 'SHOW-MODAL'; open.commandForElement = dialog; close.command = 'request-close'; close.commandForElement = dialog; dialog.addEventListener('command', (event) => { document.getElementById('out').textContent += event.command + ':' + event.source.id + ':' + String(document.getElementById('dlg').open) + ';'; }); open.click(); document.getElementById('out').textContent += '|after-open:' + String(document.getElementById('dlg').open) + ':' + open.type + ':' + open.command + ':' + open.commandForElement.id + ':' + close.type + ':' + close.command + ':' + close.commandForElement.id;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "show-modal:open:false;|after-open:true:button:show-modal:dlg:button:request-close:dlg");
}

test "contract: Harness.fromHtml runs HTMLButtonElement.requestClose command cancellation during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><dialog id='dlg'></dialog><button id='close' type='button'>Close</button><div id='out'></div><script>const dialog = document.getElementById('dlg'); const close = document.getElementById('close'); close.command = 'request-close'; close.commandForElement = dialog; dialog.addEventListener('command', (event) => { document.getElementById('out').textContent += event.command + ':' + event.source.id + ':' + String(document.getElementById('dlg').open) + ';'; event.preventDefault(); }); dialog.showModal(); close.click(); document.getElementById('out').textContent += '|after-close:' + String(document.getElementById('dlg').open) + ':' + document.getElementById('dlg').returnValue + ':' + close.command + ':' + close.commandForElement.id;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "request-close:close:true;|after-close:true::request-close:dlg");
}

test "contract: Harness.fromHtml runs HTMLDetailsElement open and name reflection during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><details id='details'><summary id='summary'>Title</summary><div>Body</div></details><div id='out'></div><script>const details = document.getElementById('details'); const out = document.getElementById('out'); details.name = 'accordion'; out.textContent = String(details.open) + ':' + details.name; details.addEventListener('toggle', () => { const current = document.getElementById('details'); const output = document.getElementById('out'); output.textContent += '|' + String(current.open) + ':' + current.name; }); details.open = true;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "false:accordion|true:accordion");
}

test "contract: Harness.fromHtml exposes form owner reflection during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><form id='owner'></form><div id='host'><input id='input' form='owner'><button id='button' form='owner'></button><fieldset id='fieldset' form='owner'></fieldset><select id='select' form='owner'><optgroup id='group'><option id='option'>A</option></optgroup></select><output id='output' form='owner'></output><meter id='meter' form='owner'></meter><progress id='progress' form='owner'></progress></div><div id='out'></div><script>const input = document.getElementById('input'); const button = document.getElementById('button'); const fieldset = document.getElementById('fieldset'); const select = document.getElementById('select'); const group = document.getElementById('group'); const option = document.getElementById('option'); const output = document.getElementById('output'); const meter = document.getElementById('meter'); const progress = document.getElementById('progress'); const detached = document.createElement('input'); document.getElementById('out').textContent = input.form.id + ':' + button.form.id + ':' + fieldset.form.id + ':' + fieldset.type + ':' + select.form.id + ':' + group.form.id + ':' + option.form.id + ':' + output.form.id + ':' + meter.form.id + ':' + progress.form.id + ':' + String(detached.form);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "owner:owner:owner:fieldset:owner:owner:owner:owner:owner:owner:null");
}

test "contract: Harness.fromHtml exposes HTMLLegendElement form reflection during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><form id='owner'></form><fieldset id='group' form='owner'><legend id='legend'>Title</legend></fieldset><legend id='detached'>Loose</legend><div id='out'></div><script>const legend = document.getElementById('legend'); const detached = document.getElementById('detached'); const fieldset = document.getElementById('group'); document.getElementById('out').textContent = legend.form.id + ':' + String(detached.form) + ':' + fieldset.form.id;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "owner:null:owner");
}

test "failure: Harness.fromHtml rejects HTMLFieldSetElement.type on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><fieldset id='group'></fieldset><script>document.getElementById('group').type = 'group';</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects HTMLLegendElement form reflection on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='host'></div><script>document.getElementById('host').form;</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects HTMLButtonElement.command on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='host'></div><script>document.getElementById('host').command;</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects HTMLButtonElement.commandForElement on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='host'></div><script>document.getElementById('host').commandForElement;</script></main>",
        ),
    );
}

test "contract: Harness.fromHtml runs HTMLOutputElement reflection during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><form id='owner'></form><label id='label' for='result'>Answer</label><output id='result' form='owner' for='alpha alpha beta'>Hello</output><div id='out'></div><script>const result = document.getElementById('result'); const before = result.type + ':' + result.defaultValue + ':' + result.value + ':' + result.form.id + ':' + String(result.labels.length) + ':' + result.htmlFor.value + ':' + String(result.htmlFor.length) + ':' + String(result.htmlFor.contains('alpha')) + ':' + String(result.willValidate) + ':' + result.validationMessage + ':' + String(result.checkValidity()) + ':' + String(result.reportValidity()) + ':' + String(result.validity.customError); result.htmlFor.value = 'gamma gamma delta'; result.defaultValue = 'Reset'; result.value = 'Computed'; result.setCustomValidity('bad'); document.getElementById('out').textContent = before + '|' + result.defaultValue + ':' + result.value + ':' + result.htmlFor.value + ':' + String(result.validity.customError) + ':' + result.validationMessage + ':' + String(result.checkValidity()) + ':' + String(result.reportValidity()) + ':' + String(result.labels.length);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "output:Hello:Hello:owner:1:alpha beta:2:true:true::true:true:false|Reset:Computed:gamma delta:true:bad:false:false:1",
    );
}

test "contract: Harness.fromHtml runs HTMLProgressElement and HTMLMeterElement reflection during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><form id='owner'></form><label id='progress-label' for='progress'>Progress</label><label id='meter-label' for='meter'>Meter</label><progress id='progress' form='owner' max='4' value='3'>75%</progress><meter id='meter' form='owner' min='2' max='10' value='12' low='3' high='8' optimum='9'>12</meter><div id='out'></div><script>const progress = document.getElementById('progress'); const meter = document.getElementById('meter'); const before = String(progress.value) + ':' + String(progress.max) + ':' + String(progress.position) + ':' + String(progress.labels.length) + ':' + progress.form.id + '|' + String(meter.value) + ':' + String(meter.min) + ':' + String(meter.max) + ':' + String(meter.low) + ':' + String(meter.high) + ':' + String(meter.optimum) + ':' + String(meter.labels.length) + ':' + meter.form.id; progress.max = 2; progress.value = 5; meter.min = 1; meter.max = 5; meter.value = 4; meter.low = 2; meter.high = 3; meter.optimum = 2.5; const after = progress.getAttribute('max') + ':' + progress.getAttribute('value') + ':' + String(progress.value) + ':' + String(progress.max) + ':' + String(progress.position) + '|' + meter.getAttribute('min') + ':' + String(meter.min) + ':' + meter.getAttribute('max') + ':' + String(meter.max) + ':' + meter.getAttribute('value') + ':' + String(meter.value) + ':' + meter.getAttribute('low') + ':' + String(meter.low) + ':' + meter.getAttribute('high') + ':' + String(meter.high) + ':' + meter.getAttribute('optimum') + ':' + String(meter.optimum); document.getElementById('out').textContent = before + '|' + after;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "3:4:0.75:1:owner|10:2:10:3:8:9:1:owner|2:5:2:2:1|1:1:5:5:4:4:2:2:3:3:2.5:2.5",
    );
}

test "contract: Harness.fromHtml runs Element.multiple during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='mail' type='email' multiple><select id='mode' multiple><option value='a' selected>A</option><option value='b'>B</option></select><div id='out'></div><script>const mail = document.getElementById('mail'); const mode = document.getElementById('mode'); const before = String(mail.multiple) + ':' + String(mode.multiple); mail.multiple = false; mode.multiple = false; document.getElementById('out').textContent = before + ':' + String(mail.multiple) + ':' + String(mode.multiple) + ':' + String(mail.hasAttribute('multiple')) + ':' + String(mode.hasAttribute('multiple'));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "true:true:false:false:false:false");
}

test "contract: Harness.fromHtml runs Element.type during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='mail'><button id='action'></button><div id='out'></div><script>const mail = document.getElementById('mail'); const action = document.getElementById('action'); const before = mail.type + ':' + action.type; mail.type = 'email'; action.type = 'reset'; document.getElementById('out').textContent = before + ':' + mail.type + ':' + action.type + ':' + mail.getAttribute('type') + ':' + action.getAttribute('type');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "text:submit:email:reset:email:reset");
}

test "contract: Harness.fromHtml runs Element.minLength and maxLength during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada'><textarea id='bio'>Hello</textarea><div id='out'></div><script>const name = document.getElementById('name'); const bio = document.getElementById('bio'); const before = String(name.minLength) + ':' + String(name.maxLength) + ':' + String(bio.minLength) + ':' + String(bio.maxLength); name.minLength = 2; name.maxLength = 5; bio.minLength = 3; bio.maxLength = 7; document.getElementById('out').textContent = before + ':' + String(name.minLength) + ':' + String(name.maxLength) + ':' + String(bio.minLength) + ':' + String(bio.maxLength) + ':' + String(name.getAttribute('minlength')) + ':' + String(name.getAttribute('maxlength')) + ':' + String(bio.getAttribute('minlength')) + ':' + String(bio.getAttribute('maxlength'));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "-1:-1:-1:-1:2:5:3:7:2:5:3:7");
}

test "contract: Harness.fromHtml runs Element.hasAttributes during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><button id='filled' class='base' data-kind='App'></button><div id='out'></div><script>const empty = document.createElement('button'); const filled = document.getElementById('filled'); document.getElementById('out').textContent = String(empty.hasAttributes()) + ':' + String(filled.hasAttributes()); empty.setAttribute('data-flag', ''); document.getElementById('out').textContent += ':' + String(empty.hasAttributes());</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "false:true:true");
}

test "contract: Harness.fromHtml runs Element.getAttributeNames during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const button = document.createElement('button'); button.setAttribute('id', 'button'); button.setAttribute('data-kind', 'App'); const names = button.getAttributeNames(); document.getElementById('out').textContent = String(names.length) + ':' + names.item(0) + ':' + names.item(1) + ':' + String(names.contains('id')) + ':' + String(names.contains('missing'));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:id:data-kind:true:false");
}

test "contract: Harness.fromHtml runs Element.id during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const button = document.createElement('button'); button.id = 'button'; const before = button.id; button.id = 'updated'; document.getElementById('out').textContent = before + ':' + button.id + ':' + button.getAttribute('id');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "button:updated:updated");
}

test "contract: Harness.fromHtml runs Element.hidden during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const box = document.createElement('section'); const before = box.hidden; box.hidden = true; const during = box.hidden; box.hidden = false; document.getElementById('out').textContent = String(before) + ':' + String(during) + ':' + String(box.hidden) + ':' + String(box.hasAttribute('hidden'));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "false:true:false:false");
}

test "contract: Harness.fromHtml runs Element.inert during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const box = document.createElement('section'); const before = box.inert; box.inert = true; const during = box.inert; box.inert = false; document.getElementById('out').textContent = String(before) + ':' + String(during) + ':' + String(box.inert) + ':' + String(box.hasAttribute('inert'));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "false:true:false:false");
}

test "contract: Harness.fromHtml runs Element.translate during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const outer = document.createElement('div'); const inner = document.createElement('span'); outer.appendChild(inner); const before = inner.translate; outer.translate = false; const inherited = inner.translate; inner.translate = true; const overridden = inner.translate; document.getElementById('out').textContent = String(before) + ':' + String(inherited) + ':' + String(overridden) + ':' + outer.getAttribute('translate') + ':' + inner.getAttribute('translate');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "true:false:true:no:yes");
}

test "contract: Harness.fromHtml runs Element.spellcheck during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const outer = document.createElement('div'); const inner = document.createElement('span'); outer.appendChild(inner); const before = inner.spellcheck; outer.spellcheck = false; const inherited = inner.spellcheck; inner.spellcheck = true; const overridden = inner.spellcheck; document.getElementById('out').textContent = String(before) + ':' + String(inherited) + ':' + String(overridden) + ':' + outer.getAttribute('spellcheck') + ':' + inner.getAttribute('spellcheck');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "true:false:true:false:true");
}

test "contract: Harness.fromHtml runs Element.draggable during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const box = document.createElement('section'); const before = box.draggable; box.draggable = true; const during = box.draggable; box.draggable = false; document.getElementById('out').textContent = String(before) + ':' + String(during) + ':' + String(box.draggable) + ':' + String(box.hasAttribute('draggable'));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "false:true:false:false");
}

test "contract: Harness.fromHtml runs Element.nonce during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const script = document.createElement('script'); const before = script.nonce; script.nonce = 'abc123'; const during = script.nonce; document.getElementById('out').textContent = before + ':' + during + ':' + script.nonce + ':' + script.getAttribute('nonce');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", ":abc123:abc123:abc123");
}

test "contract: Harness.fromHtml runs HTMLScriptElement metadata reflection during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const script = document.createElement('script'); const before = '[' + script.src + ']:[' + String(script.async) + ']:[' + String(script.defer) + ']:[' + String(script.noModule) + ']:[' + script.type + ']:[' + script.crossOrigin + ']:[' + script.integrity + ']:[' + script.referrerPolicy + ']:[' + script.fetchPriority + ']:[' + script.charset + ']:[' + script.text + ']'; script.src = 'https://example.test/app.js'; script.async = true; script.defer = true; script.noModule = true; script.type = 'module'; script.crossOrigin = 'anonymous'; script.integrity = 'sha384-abc'; script.referrerPolicy = 'no-referrer'; script.fetchPriority = 'high'; script.charset = 'utf-8'; script.text = 'inline text'; document.getElementById('out').textContent = before + '|' + '[' + script.src + ']:[' + String(script.async) + ']:[' + String(script.defer) + ']:[' + String(script.noModule) + ']:[' + script.type + ']:[' + script.crossOrigin + ']:[' + script.integrity + ']:[' + script.referrerPolicy + ']:[' + script.fetchPriority + ']:[' + script.charset + ']:[' + script.text + ']:[' + script.getAttribute('src') + ']:[' + script.getAttribute('async') + ']:[' + script.getAttribute('defer') + ']:[' + script.getAttribute('nomodule') + ']:[' + script.getAttribute('type') + ']:[' + script.getAttribute('crossorigin') + ']:[' + script.getAttribute('integrity') + ']:[' + script.getAttribute('referrerpolicy') + ']:[' + script.getAttribute('fetchpriority') + ']:[' + script.getAttribute('charset') + ']';</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[]:[false]:[false]:[false]:[]:[]:[]:[]:[]:[]:[]|[https://example.test/app.js]:[true]:[true]:[true]:[module]:[anonymous]:[sha384-abc]:[no-referrer]:[high]:[utf-8]:[inline text]:[https://example.test/app.js]:[]:[]:[]:[module]:[anonymous]:[sha384-abc]:[no-referrer]:[high]:[utf-8]",
    );
}

test "contract: Harness.fromHtml runs Element.autocapitalize during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const box = document.createElement('textarea'); const before = box.autocapitalize; box.autocapitalize = 'words'; const during = box.autocapitalize; document.getElementById('out').textContent = before + ':' + during + ':' + box.autocapitalize + ':' + box.getAttribute('autocapitalize');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", ":words:words:words");
}

test "contract: Harness.fromHtml runs Element.autofocus during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const input = document.createElement('input'); const before = input.autofocus; input.autofocus = true; const during = input.autofocus; input.autofocus = false; document.getElementById('out').textContent = String(before) + ':' + String(during) + ':' + String(input.autofocus) + ':' + String(input.hasAttribute('autofocus'));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "false:true:false:false");
}

test "contract: Harness.fromHtml runs Element.placeholder during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const input = document.createElement('input'); const area = document.createElement('textarea'); const beforeInput = input.placeholder; const beforeArea = area.placeholder; root.appendChild(input); root.appendChild(area); input.placeholder = 'Name'; area.placeholder = 'Bio'; const duringInput = input.placeholder; const duringArea = area.placeholder; const placeholderShown = document.querySelectorAll(':placeholder-shown').length; document.getElementById('out').textContent = beforeInput + ':' + beforeArea + ':' + duringInput + ':' + duringArea + ':' + String(placeholderShown) + ':' + input.getAttribute('placeholder') + ':' + area.getAttribute('placeholder');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "::Name:Bio:2:Name:Bio");
}

test "contract: Harness.fromHtml runs Element.name during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const form = document.createElement('form'); const input = document.createElement('input'); const area = document.createElement('textarea'); const beforeForm = form.name; const beforeInput = input.name; const beforeArea = area.name; form.name = 'signup'; input.name = 'first'; input.id = 'first-input'; area.name = 'bio'; root.appendChild(form); form.appendChild(input); form.appendChild(area); const duringForm = form.name; const duringInput = input.name; const duringArea = area.name; const formNamed = document.forms.namedItem('signup').name; const elementNamed = form.elements.namedItem('first').getAttribute('name'); const namedElements = document.getElementsByName('bio').length; document.getElementById('out').textContent = beforeForm + ':' + beforeInput + ':' + beforeArea + ':' + duringForm + ':' + duringInput + ':' + duringArea + ':' + formNamed + ':' + elementNamed + ':' + String(namedElements) + ':' + form.getAttribute('name') + ':' + input.getAttribute('name') + ':' + area.getAttribute('name');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", ":::signup:first:bio:signup:first:1:signup:first:bio");
}

test "contract: Harness.fromHtml runs option.selected and select.value during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const select = document.createElement('select'); const first = document.createElement('option'); const second = document.createElement('option'); first.id = 'first'; first.value = 'a'; first.textContent = 'A'; second.id = 'second'; second.value = 'b'; second.textContent = 'B'; second.selected = true; select.appendChild(first); select.appendChild(second); root.appendChild(select); const before = select.value + ':' + first.value + ':' + second.value + ':' + String(first.selected) + ':' + String(second.selected) + ':' + String(select.selectedOptions.length); first.value = 'z'; select.value = 'z'; const after = select.value + ':' + first.value + ':' + second.value + ':' + String(first.selected) + ':' + String(second.selected) + ':' + String(select.selectedOptions.length); document.getElementById('out').textContent = before + '|' + after;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "b:a:b:false:true:1|z:z:b:true:false:1");
}

test "contract: Harness.fromHtml runs select.selectedIndex during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const select = document.createElement('select'); const first = document.createElement('option'); const second = document.createElement('option'); first.id = 'first'; first.value = 'a'; first.textContent = 'A'; second.id = 'second'; second.value = 'b'; second.textContent = 'B'; second.selected = true; select.appendChild(first); select.appendChild(second); root.appendChild(select); const before = String(select.selectedIndex) + ':' + select.value + ':' + String(first.selected) + ':' + String(second.selected) + ':' + String(select.selectedOptions.length); select.selectedIndex = 0; const after = String(select.selectedIndex) + ':' + select.value + ':' + String(first.selected) + ':' + String(second.selected) + ':' + String(select.selectedOptions.length); document.getElementById('out').textContent = before + '|' + after;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "1:b:false:true:1|0:a:true:false:1");
}

test "contract: Harness.fromHtml runs option and optgroup reflection during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><select id='select'><optgroup id='group' label='Group'><option id='first' value='a'>Alpha</option><option id='second' value='b' label='Bravo'>Beta</option></optgroup></select><div id='out'></div><script>const select = document.getElementById('select'); const group = document.getElementById('group'); const first = document.getElementById('first'); const second = document.getElementById('second'); const before = String(group.label) + ':' + String(first.label) + ':' + String(first.defaultSelected) + ':' + String(first.text) + ':' + String(first.index) + ':' + String(second.label) + ':' + String(second.defaultSelected) + ':' + String(second.text) + ':' + String(second.index); group.label = 'Updated group'; first.label = 'Alpha label'; first.defaultSelected = true; first.text = 'Alpha!'; document.getElementById('out').textContent = before + '|' + group.label + ':' + first.label + ':' + String(first.defaultSelected) + ':' + first.text + ':' + String(first.index) + ':' + second.label + ':' + String(second.defaultSelected) + ':' + second.text + ':' + String(second.index) + ':' + select.options.item(0).label;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "Group:Alpha:false:Alpha:0:Bravo:false:Beta:1|Updated group:Alpha label:true:Alpha!:0:Bravo:false:Beta:1:Alpha label");
}

test "contract: Harness.fromHtml runs checkValidity and reportValidity during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><form id='form'><input id='short' minlength='4' value='abc'><textarea id='area' maxlength='3'>abcd</textarea><input id='hidden' type='hidden' required></form><div id='out'></div><script>const form = document.getElementById('form'); const short = document.getElementById('short'); const area = document.getElementById('area'); const hidden = document.getElementById('hidden'); const before = String(short.checkValidity()) + ':' + String(short.reportValidity()) + ':' + String(area.checkValidity()) + ':' + String(area.reportValidity()) + ':' + String(hidden.checkValidity()) + ':' + String(hidden.reportValidity()) + ':' + String(form.checkValidity()) + ':' + String(form.reportValidity()); short.value = 'abcd'; area.textContent = 'abc'; const after = String(short.checkValidity()) + ':' + String(short.reportValidity()) + ':' + String(area.checkValidity()) + ':' + String(area.reportValidity()) + ':' + String(hidden.checkValidity()) + ':' + String(hidden.reportValidity()) + ':' + String(form.checkValidity()) + ':' + String(form.reportValidity()); document.getElementById('out').textContent = before + '|' + after;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "false:false:false:false:true:true:false:false|true:true:true:true:true:true:true:true");
}

test "contract: Harness.fromHtml dispatches invalid during reportValidity" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><form id='form'><input id='short' minlength='4' value='abc'></form><div id='out'></div><script>const form = document.getElementById('form'); const input = document.getElementById('short'); form.addEventListener('invalid', () => { document.getElementById('out').textContent += 'form|'; }, true); input.addEventListener('invalid', () => { document.getElementById('out').textContent += 'input|'; }); const inputResult = String(input.reportValidity()); const inputEvents = document.getElementById('out').textContent; document.getElementById('out').textContent = ''; const formResult = String(form.reportValidity()); const formEvents = document.getElementById('out').textContent; document.getElementById('out').textContent = inputResult + ':' + inputEvents + '|' + formResult + ':' + formEvents;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "false:form|input||false:form|input|");
}

test "contract: Harness.fromHtml runs Element.setCustomValidity and validationMessage during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const form = document.createElement('form'); const input = document.createElement('input'); input.value = 'abc'; form.appendChild(input); document.getElementById('root').appendChild(form); const before = String(input.checkValidity()) + ':' + input.validationMessage + ':' + String(form.checkValidity()); input.setCustomValidity('bad'); const during = String(input.checkValidity()) + ':' + input.validationMessage + ':' + String(input.reportValidity()) + ':' + String(form.checkValidity()); input.setCustomValidity(''); const after = String(input.checkValidity()) + ':' + input.validationMessage + ':' + String(form.checkValidity()); document.getElementById('out').textContent = before + '|' + during + '|' + after;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "true::true|false:bad:false:false|true::true");
}

test "contract: Harness.fromHtml mutates CSSMediaRule insertRule and deleteRule during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style id='sheet'>@media screen { .primary { color: red; } }</style><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const before = String(rule) + ':' + String(rule.cssRules.length) + ':' + rule.cssRules.item(0).selectorText; const inserted = rule.insertRule('.secondary { color: blue; }', 1); const afterInsert = document.styleSheets.item(0).cssRules.item(0); const snapshot = String(afterInsert) + ':' + String(afterInsert.cssRules.length) + ':' + afterInsert.cssRules.item(0).selectorText + ':' + afterInsert.cssRules.item(1).selectorText + ':' + afterInsert.cssText; afterInsert.deleteRule(0); const refreshed = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(inserted) + ':' + snapshot + '|' + String(refreshed.cssRules.length) + ':' + refreshed.cssRules.item(0).selectorText + ':' + refreshed.cssText + ':' + document.getElementById('sheet').textContent;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSMediaRule]:1:.primary|1:[object CSSMediaRule]:2:.primary:.secondary:@media screen { .primary { color: red; }\n.secondary { color: blue; } }|1:.secondary:@media screen { .secondary { color: blue; } }:@media screen { .secondary { color: blue; } }",
    );
}

test "contract: Harness.fromHtml mutates CSSSupportsRule insertRule and deleteRule during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style id='sheet'>@supports (display: grid) { .primary { color: red; } }</style><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const before = String(rule) + ':' + rule.conditionText + ':' + String(rule.cssRules.length) + ':' + rule.cssRules.item(0).selectorText; const inserted = rule.insertRule('.secondary { color: blue; }', 1); const afterInsert = document.styleSheets.item(0).cssRules.item(0); const snapshot = String(afterInsert) + ':' + afterInsert.conditionText + ':' + String(afterInsert.cssRules.length) + ':' + afterInsert.cssRules.item(0).selectorText + ':' + afterInsert.cssRules.item(1).selectorText + ':' + afterInsert.cssText; afterInsert.deleteRule(0); const refreshed = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(inserted) + ':' + snapshot + '|' + String(refreshed) + ':' + refreshed.conditionText + ':' + String(refreshed.cssRules.length) + ':' + refreshed.cssRules.item(0).selectorText + ':' + refreshed.cssText + ':' + document.getElementById('sheet').textContent;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSSupportsRule]:(display: grid):1:.primary|1:[object CSSSupportsRule]:(display: grid):2:.primary:.secondary:@supports (display: grid) { .primary { color: red; }\n.secondary { color: blue; } }|[object CSSSupportsRule]:(display: grid):1:.secondary:@supports (display: grid) { .secondary { color: blue; } }:@supports (display: grid) { .secondary { color: blue; } }",
    );
}

test "contract: Harness.fromHtml mutates CSSContainerRule insertRule and deleteRule during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style id='sheet'>@container card (min-width: 1px) { .primary { color: red; } }</style><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const before = String(rule) + ':' + rule.containerName + ':' + rule.containerQuery + ':' + String(rule.cssRules.length) + ':' + rule.cssRules.item(0).selectorText; const inserted = rule.insertRule('.secondary { color: blue; }', 1); const afterInsert = document.styleSheets.item(0).cssRules.item(0); const snapshot = String(afterInsert) + ':' + afterInsert.containerName + ':' + afterInsert.containerQuery + ':' + String(afterInsert.cssRules.length) + ':' + afterInsert.cssRules.item(0).selectorText + ':' + afterInsert.cssRules.item(1).selectorText + ':' + afterInsert.cssText; afterInsert.deleteRule(0); const refreshed = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(inserted) + ':' + snapshot + '|' + String(refreshed) + ':' + refreshed.containerName + ':' + refreshed.containerQuery + ':' + String(refreshed.cssRules.length) + ':' + refreshed.cssRules.item(0).selectorText + ':' + refreshed.cssText + ':' + document.getElementById('sheet').textContent;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSContainerRule]:card:(min-width: 1px):1:.primary|1:[object CSSContainerRule]:card:(min-width: 1px):2:.primary:.secondary:@container card (min-width: 1px) { .primary { color: red; }\n.secondary { color: blue; } }|[object CSSContainerRule]:card:(min-width: 1px):1:.secondary:@container card (min-width: 1px) { .secondary { color: blue; } }:@container card (min-width: 1px) { .secondary { color: blue; } }",
    );
}

test "contract: Harness.fromHtml runs Element.pattern during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const field = document.createElement('input'); field.pattern = '[0-9]{3}'; field.value = '12'; const good = document.createElement('input'); good.pattern = '[0-9]{3}'; good.value = '123'; root.appendChild(field); root.appendChild(good); document.getElementById('out').textContent = field.pattern + ':' + String(field.validity.patternMismatch) + ':' + String(field.checkValidity()) + ':' + good.pattern + ':' + String(good.validity.patternMismatch) + ':' + String(good.checkValidity()) + ':' + String(document.querySelectorAll(':invalid').length);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "[0-9]{3}:true:false:[0-9]{3}:false:true:1");
}

test "contract: Harness.fromHtml runs Element.inputMode during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const box = document.createElement('input'); const before = box.inputMode; box.inputMode = 'numeric'; const during = box.inputMode; document.getElementById('out').textContent = before + ':' + during + ':' + box.inputMode + ':' + box.getAttribute('inputmode');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", ":numeric:numeric:numeric");
}

test "contract: Harness.fromHtml runs Element.readOnly during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const box = document.createElement('input'); const before = box.readOnly; box.readOnly = true; const during = box.readOnly; box.readOnly = false; document.getElementById('out').textContent = String(before) + ':' + String(during) + ':' + String(box.readOnly) + ':' + String(box.hasAttribute('readonly'));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "false:true:false:false");
}

test "contract: Harness.fromHtml runs Element.accessKey during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const button = document.createElement('button'); const before = button.accessKey; button.accessKey = 'k'; const during = button.accessKey; document.getElementById('out').textContent = before + ':' + during + ':' + button.accessKey + ':' + button.getAttribute('accesskey');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", ":k:k:k");
}

test "contract: Harness.fromHtml runs Element aria and role reflection during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const box = document.createElement('div'); const beforeRole = box.role; const beforeHidden = box.ariaHidden; box.role = 'button'; box.ariaLabel = 'Menu'; box.ariaDescription = 'Opens menu'; box.ariaRoleDescription = 'toggle button'; box.ariaHidden = 'true'; document.getElementById('out').textContent = beforeRole + ':' + beforeHidden + ':' + box.role + ':' + box.ariaLabel + ':' + box.getAttribute('aria-label') + ':' + box.ariaDescription + ':' + box.getAttribute('aria-description') + ':' + box.ariaRoleDescription + ':' + box.getAttribute('aria-roledescription') + ':' + box.ariaHidden + ':' + box.getAttribute('aria-hidden');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "::button:Menu:Menu:Opens menu:Opens menu:toggle button:toggle button:true:true");
}

test "contract: Harness.fromHtml runs Element.slot during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const box = document.createElement('div'); const before = box.slot; box.slot = 'primary'; const during = box.slot; document.getElementById('out').textContent = before + ':' + during + ':' + box.slot + ':' + box.getAttribute('slot');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", ":primary:primary:primary");
}

test "contract: Harness.fromHtml runs Element.part during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const box = document.createElement('div'); const part = box.part; const before = part.length; part.add('primary'); part.add('secondary'); part.remove('secondary'); const replaced = part.replace('primary', 'accent'); const missing = part.replace('missing', 'other'); document.getElementById('out').textContent = String(before) + ':' + box.getAttribute('part') + ':' + String(part.length) + ':' + String(part.contains('accent')) + ':' + String(replaced) + ':' + String(missing) + ':' + String(part);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "0:accent:1:true:true:false:[object DOMTokenList]");
}

test "contract: Harness.fromHtml runs Element.contentEditable during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const box = document.createElement('section'); const child = document.createElement('span'); box.appendChild(child); const before = box.contentEditable; const childBefore = child.isContentEditable; box.contentEditable = 'true'; const during = box.contentEditable; const childDuring = child.isContentEditable; box.contentEditable = 'false'; document.getElementById('out').textContent = before + ':' + String(childBefore) + ':' + during + ':' + String(childDuring) + ':' + String(box.isContentEditable) + ':' + box.getAttribute('contenteditable');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "inherit:false:true:true:false:false");
}

test "contract: Harness.fromHtml runs Element.tabIndex during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const button = document.createElement('button'); const panel = document.createElement('div'); const buttonBefore = button.tabIndex; const panelBefore = panel.tabIndex; panel.tabIndex = 3; const panelDuring = panel.tabIndex; panel.tabIndex = -1; document.getElementById('out').textContent = String(buttonBefore) + ':' + String(panelBefore) + ':' + String(panelDuring) + ':' + String(panel.tabIndex) + ':' + panel.getAttribute('tabindex');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "0:-1:3:-1:-1");
}

test "contract: Harness.fromHtml runs Element.title lang and dir during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='localized'></section><div id='out'></div><script>const localized = document.getElementById('localized'); localized.title = 'Greeting'; localized.lang = 'en-US'; localized.dir = 'rtl'; document.getElementById('out').textContent = localized.title + ':' + localized.lang + ':' + localized.dir + ':' + document.querySelector(':lang(en)').id + ':' + document.querySelector(':dir(rtl)').id;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "Greeting:en-US:rtl:localized:localized");
}

test "contract: Harness.fromHtml runs namespace-aware attribute reflection during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const button = document.createElement('button'); button.setAttributeNS('urn:test', 'id', 'button'); button.setAttributeNS('urn:test', 'data-kind', 'App'); const before = button.getAttributeNS('urn:test', 'data-kind'); const present = button.hasAttributeNS('urn:test', 'id'); button.removeAttributeNS('urn:test', 'id'); document.getElementById('out').textContent = before + ':' + String(present) + ':' + String(button.hasAttributeNS('urn:test', 'id')) + ':' + String(button.getAttributeNS('urn:test', 'missing'));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "App:true:false:null");
}

test "contract: Harness.fromHtml runs class and dataset views during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><button id='button' class='base' data-kind='App'>First</button><div id='out'></div><script>document.getElementById('button').className = 'primary secondary'; document.getElementById('button').classList.add('tertiary'); document.getElementById('button').classList.remove('secondary'); const replaced = document.getElementById('button').classList.replace('primary', 'accent'); const missing = document.getElementById('button').classList.replace('missing', 'other'); document.getElementById('button').dataset.userId = '42'; document.getElementById('out').textContent = String(document.getElementById('button').classList.length) + ':' + String(document.getElementById('button').classList.contains('accent')) + ':' + String(replaced) + ':' + String(missing) + ':' + String(document.getElementById('button').classList.toggle('active')) + ':' + document.getElementById('button').className + ':' + document.getElementById('button').dataset.kind + ':' + document.getElementById('button').dataset.userId + ':' + String(document.getElementById('button').classList) + ':' + String(document.getElementById('button').dataset);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:true:true:false:true:accent tertiary active:App:42:[object DOMTokenList]:[object DOMStringMap]");
    try subject.assertExists(".active");
    try subject.assertExists("[data-user-id]");
    try subject.assertExists("[data-kind=App]");
}

test "contract: Harness.fromHtml runs DOMTokenList value and item during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><button id='button' class='base primary base'>First</button><div id='parted' part='primary secondary primary'></div><link id='link' rel='stylesheet preload stylesheet' href='a.css'><div id='out'></div><script>const button = document.getElementById('button'); const parted = document.getElementById('parted'); const link = document.getElementById('link'); const before = button.classList.value + ':' + parted.part.value + ':' + link.relList.value + ':' + String(button.classList.item(0)) + ':' + String(button.classList.item(9)) + ':' + String(parted.part.item(0)) + ':' + String(link.relList.item(0)) + ':' + String(link.relList.item(9)); button.classList.value = 'alpha  beta alpha'; parted.part.value = 'accent accent tertiary'; link.relList.value = 'stylesheet preload stylesheet'; document.getElementById('out').textContent = before + '|' + button.className + ':' + button.classList.value + ':' + String(button.classList.item(0)) + ':' + String(button.classList.item(1)) + ':' + parted.getAttribute('part') + ':' + parted.part.value + ':' + String(parted.part.item(0)) + ':' + link.rel + ':' + link.relList.value + ':' + String(link.sheet) + ':' + String(link.relList.contains('stylesheet')) + ':' + String(link.relList.supports('preload')) + ':' + String(link.relList.item(0)) + ':' + String(link.relList.item(1));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "base primary:primary secondary:stylesheet preload:base:null:primary:stylesheet:null|alpha beta:alpha beta:alpha:beta:accent tertiary:accent tertiary:accent:stylesheet preload:stylesheet preload:[object CSSStyleSheet]:true:true:stylesheet:preload");
}

test "contract: Harness.fromHtml runs DOMTokenList iterators and forEach during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><button id='button' class='base primary base'>First</button><div id='parted' part='primary secondary primary'></div><link id='link' rel='stylesheet preload stylesheet' href='a.css'><div id='out'></div><script>const button = document.getElementById('button'); const parted = document.getElementById('parted'); const link = document.getElementById('link'); const classKeys = button.classList.keys(); const classValues = button.classList.values(); const classEntries = button.classList.entries(); const classKey0 = classKeys.next(); const classKey1 = classKeys.next(); const classKey2 = classKeys.next(); const classValue0 = classValues.next(); const classValue1 = classValues.next(); const classValue2 = classValues.next(); const classEntry0 = classEntries.next(); const classEntry1 = classEntries.next(); const classEntry2 = classEntries.next(); document.getElementById('out').textContent = String(classKey0.value) + ':' + String(classKey1.value) + ':' + String(classKey2.done) + '|' + classValue0.value + ':' + classValue1.value + ':' + String(classValue2.done) + '|' + String(classEntry0.value.index) + ':' + classEntry0.value.value + ':' + String(classEntry1.value.index) + ':' + classEntry1.value.value + ':' + String(classEntry2.done) + '|'; button.classList.forEach((token, index, list) => { document.getElementById('out').textContent += String(index) + ':' + token + ':' + String(list.length) + ';'; }, null); document.getElementById('out').textContent += '|'; parted.part.forEach((token, index, list) => { document.getElementById('out').textContent += String(index) + ':' + token + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent += '|'; link.relList.forEach((token, index, list) => { document.getElementById('out').textContent += String(index) + ':' + token + ':' + String(list.length) + ';'; }, null);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "0:1:true|base:primary:true|0:base:1:primary:true|0:base:2;1:primary:2;|0:primary:2;1:secondary:2;|0:stylesheet:2;1:preload:2;",
    );
}

test "contract: Harness.fromHtml runs inline style declaration surface during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='box' style='color: red; background-color: white;'></div><div id='out'></div><script>const box = document.getElementById('box'); const style = box.style; const before = style.cssText; const color = style.color; const length = style.length; const first = style.item(0); const background = style.getPropertyValue('background-color'); const removed = style.removeProperty('color'); style.backgroundColor = 'blue'; style.setProperty('border-top-width', '2px'); document.getElementById('out').textContent = before + '|' + color + '|' + String(length) + '|' + first + '|' + background + '|' + removed + '|' + box.getAttribute('style') + '|' + String(style);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "color: red; background-color: white;|red|2|color|white|red|background-color: blue; border-top-width: 2px;|background-color: blue; border-top-width: 2px;",
    );
}

test "contract: Harness.fromHtml runs window.getComputedStyle during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='box' style='color: red; background-color: white;'></div><div id='out'></div><script>const box = document.getElementById('box'); const style = window.getComputedStyle(box); const styleNull = String(window.getComputedStyle(box, null)); const styleEmpty = String(window.getComputedStyle(box, '')); document.getElementById('out').textContent = String(style) + ':' + style.cssText + ':' + String(style.length) + ':' + style.item(0) + ':' + style.item(1) + ':' + style.getPropertyValue('color') + ':' + style.getPropertyValue('background-color') + ':' + styleNull + ':' + styleEmpty;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "color: red; background-color: white;:color: red; background-color: white;:2:color:background-color:red:white:color: red; background-color: white;:color: red; background-color: white;",
    );
}

test "contract: Harness.fromHtml runs Element.getBoundingClientRect during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='box' style='left: 10px; top: 20px; width: 30px; height: 40px;'></div><div id='out'></div><script>const box = document.getElementById('box'); const rect = box.getBoundingClientRect(); document.getElementById('out').textContent = String(rect) + ':' + String(rect.x) + ':' + String(rect.y) + ':' + String(rect.width) + ':' + String(rect.height) + ':' + String(rect.top) + ':' + String(rect.right) + ':' + String(rect.bottom) + ':' + String(rect.left);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object DOMRect]:10:20:30:40:20:40:60:10",
    );
}

test "contract: Harness.fromHtml runs Element.getClientRects during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='box' style='left: 10px; top: 20px; width: 30px; height: 40px;'></div><div id='out'></div><script>const box = document.getElementById('box'); const rects = box.getClientRects(); document.getElementById('out').textContent = String(rects) + ':' + String(rects.length) + ':' + String(rects.item(0)) + ':' + String(rects.item(0).width) + ':' + String(rects.item(0).right);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object DOMRectList]:1:[object DOMRect]:30:40",
    );
}

test "contract: Harness.fromHtml accepts style comments and important priority during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='box' style='/* lead */ color: red !important; background-color: white; /* tail */'></div><div id='out'></div><script>const box = document.getElementById('box'); const style = box.style; const before = style.cssText; const color = style.getPropertyValue('color'); const length = style.length; const first = style.item(0); style.setProperty('border-top-width', '2px', 'important'); document.getElementById('out').textContent = before + '|' + color + '|' + String(length) + '|' + first + '|' + box.getAttribute('style') + '|' + String(style);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "color: red !important; background-color: white;|red|2|color|color: red !important; background-color: white; border-top-width: 2px !important;|color: red !important; background-color: white; border-top-width: 2px !important;",
    );
}

test "contract: Harness.fromHtml accepts semicolon-aware style declaration values during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='box'></div><div id='out'></div><script>const box = document.getElementById('box'); const style = box.style; style.cssText = \"content: 'A;B'; background-image: url(data:image/svg+xml;utf8,foo);\"; style.setProperty('border-image-source', 'url(data:image/svg+xml;utf8,bar)'); document.getElementById('out').textContent = style.cssText + '|' + style.getPropertyValue('content') + '|' + style.getPropertyValue('background-image') + '|' + style.getPropertyValue('border-image-source') + '|' + String(style.length);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "content: 'A;B'; background-image: url(data:image/svg+xml;utf8,foo); border-image-source: url(data:image/svg+xml;utf8,bar);|'A;B'|url(data:image/svg+xml;utf8,foo)|url(data:image/svg+xml;utf8,bar)|3",
    );
}

test "contract: Harness.fromHtml reports style property priorities during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='box' style='color: red !important; background-color: white;'></div><div id='out'></div><script>const style = document.getElementById('box').style; document.getElementById('out').textContent = style.getPropertyPriority('color') + ':' + style.getPropertyPriority('background-color') + ':' + style.getPropertyPriority('missing');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "important::");
}

test "contract: Harness.fromHtml runs selection state on inputs and textareas during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada'><textarea id='bio'>Hello</textarea><input id='check' type='checkbox'><div id='out'></div><script>const name = document.getElementById('name'); const bio = document.getElementById('bio'); const check = document.getElementById('check'); const before = String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + ':' + String(bio.selectionStart) + ':' + String(bio.selectionEnd) + ':' + bio.selectionDirection + ':' + String(check.selectionStart) + ':' + String(check.selectionEnd) + ':' + String(check.selectionDirection); name.setSelectionRange(1, 3, 'backward'); bio.select(); document.getElementById('out').textContent = before + '|' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + '|' + String(bio.selectionStart) + ':' + String(bio.selectionEnd) + ':' + bio.selectionDirection + '|' + String(check.selectionStart) + ':' + String(check.selectionEnd) + ':' + String(check.selectionDirection);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "3:3:none:5:5:none:null:null:null|1:3:backward|0:5:none|null:null:null");
}

test "contract: Harness.fromHtml exposes selection snapshots through getSelection" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(1, 3, 'backward'); const selection = window.getSelection(); const docSelection = document.getSelection(); document.getElementById('out').textContent = String(selection) + '|' + String(docSelection) + '|' + String(selection.rangeCount) + '|' + String(selection.isCollapsed) + '|' + selection.type + '|' + selection.anchorNode.id + '|' + selection.focusNode.id + '|' + String(selection.anchorOffset) + '|' + String(selection.focusOffset) + '|' + String(selection.containsNode(name)) + '|' + String(selection.containsNode(document.body));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "da|da|1|false|Range|name|name|3|1|true|false");
}

test "contract: Harness.fromHtml collapses selection snapshots during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const selection = document.getSelection(); selection.collapseToStart(); const start = String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + ':' + String(document.getSelection()); name.setSelectionRange(4, 12, 'backward'); selection.collapseToEnd(); const end = String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + ':' + String(document.getSelection()); document.getElementById('out').textContent = start + '|' + end;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "4:4:none:|12:12:none:");
}

test "contract: Harness.fromHtml collapses selection snapshots to a node and offset during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const selection = document.getSelection(); selection.collapse(name, 2); document.getElementById('out').textContent = String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + ':' + String(document.getSelection()) + ':' + String(document.getSelection().rangeCount) + ':' + document.getSelection().type;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:2:none::1:Caret");
}

test "contract: Harness.fromHtml extends selection snapshots during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const selection = document.getSelection(); selection.extend(name, 2); const current = document.getSelection(); document.getElementById('out').textContent = String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + ':' + String(current) + ':' + String(current.rangeCount) + ':' + current.type + ':' + String(current.anchorOffset) + ':' + String(current.focusOffset);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:12:backward:a Lovelace:1:Range:12:2");
}

test "contract: Harness.fromHtml sets selection base and extent during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); const selection = document.getSelection(); selection.setBaseAndExtent(name, 12, name, 4); const current = document.getSelection(); document.getElementById('out').textContent = String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + ':' + String(current) + ':' + String(current.rangeCount) + ':' + current.type + ':' + String(current.anchorOffset) + ':' + String(current.focusOffset);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "4:12:backward:Lovelace:1:Range:12:4");
}

test "contract: Harness.fromHtml sets selection position during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); const selection = document.getSelection(); selection.setPosition(name, 2); const current = document.getSelection(); document.getElementById('out').textContent = String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + ':' + String(current) + ':' + String(current.rangeCount) + ':' + current.type + ':' + String(current.anchorOffset) + ':' + String(current.focusOffset);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:2:none::1:Caret:2:2");
}

test "contract: Harness.fromHtml selects all children during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); const selection = document.getSelection(); selection.selectAllChildren(name); const current = document.getSelection(); document.getElementById('out').textContent = String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + ':' + String(current) + ':' + String(current.rangeCount) + ':' + current.type + ':' + String(current.anchorOffset) + ':' + String(current.focusOffset);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "0:3:none:Ada:1:Range:0:3");
}

test "contract: Harness.fromHtml exposes selection ranges during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(1, 3, 'backward'); const range = document.getSelection().getRangeAt(0); document.getElementById('out').textContent = String(range) + '|' + range.startContainer.id + ':' + range.endContainer.id + '|' + String(range.startOffset) + ':' + String(range.endOffset) + '|' + String(range.collapsed) + '|' + range.commonAncestorContainer.id;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "da|name:name|1:3|false|name");
}

test "contract: Harness.fromHtml exposes document.createRange during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const range = document.createRange(); document.getElementById('out').textContent = String(range) + ':' + String(range.collapsed) + ':' + String(range.startContainer) + ':' + String(range.endContainer) + ':' + String(range.commonAncestorContainer) + ':' + String(range.startOffset) + ':' + String(range.endOffset);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", ":true:null:null:null:0:0");
}

test "contract: Harness.fromHtml updates range boundaries during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='box'>Hello</div><div id='out'></div><script>const box = document.getElementById('box'); const start = document.createRange().selectNodeContents(box).setEnd(box, 2).setStart(box, 4); const end = document.createRange().selectNodeContents(box).setStart(box, 4).setEnd(box, 2); document.getElementById('out').textContent = String(start) + '|' + String(start.startOffset) + ':' + String(start.endOffset) + ':' + String(start.collapsed) + '|' + String(end) + '|' + String(end.startOffset) + ':' + String(end.endOffset) + ':' + String(end.collapsed);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "|4:4:true||2:2:true");
}

test "contract: Harness.fromHtml collapses range snapshots during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='box'>Hello</div><div id='out'></div><script>const box = document.getElementById('box'); const range = document.createRange().selectNodeContents(box); const start = range.collapse(true); const end = range.collapse(); document.getElementById('out').textContent = String(start) + '|' + String(start.startOffset) + ':' + String(start.endOffset) + ':' + String(start.collapsed) + '|' + String(end) + '|' + String(end.startOffset) + ':' + String(end.endOffset) + ':' + String(end.collapsed);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "|0:0:true||5:5:true");
}

test "contract: Harness.fromHtml clones selection ranges during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const range = document.getSelection().getRangeAt(0); const clone = range.cloneRange(); document.getElementById('out').textContent = String(range) + '|' + String(clone) + '|' + clone.startContainer.id + ':' + clone.endContainer.id + '|' + String(clone.startOffset) + ':' + String(clone.endOffset) + '|' + String(clone.collapsed) + '|' + clone.commonAncestorContainer.id;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "Lovelace|Lovelace|name:name|4:12|false|name");
}

test "contract: Harness.fromHtml creates contextual fragments during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(1, 3, 'backward'); const range = document.getSelection().getRangeAt(0); const fragment = range.createContextualFragment('<strong>Byron</strong>'); document.getElementById('out').textContent = fragment.innerHTML + '|' + fragment.textContent + '|' + name.value + '|' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + '|' + String(document.getSelection().rangeCount) + ':' + String(document.getSelection().type);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "<strong>Byron</strong>|Byron|Ada|1:3:backward|1:Range");
}

test "contract: Harness.fromHtml inserts nodes at the start of selected ranges during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='box'><span>Hello</span></div><div id='out'></div><script>const box = document.getElementById('box'); const range = document.createRange().selectNodeContents(box); const node = document.createElement('em'); node.textContent = 'Byron'; range.insertNode(node); document.getElementById('out').textContent = box.innerHTML + '|' + String(range) + '|' + String(range.startOffset) + ':' + String(range.endOffset) + ':' + String(range.collapsed);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "<em>Byron</em><span>Hello</span>|Hello|0:5:false");
}

test "contract: Harness.fromHtml surrounds selected range contents during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='box'><span>Hello</span><em>!</em></div><div id='out'></div><script>const box = document.getElementById('box'); const range = document.createRange().selectNodeContents(box); const wrapper = document.createElement('strong'); range.surroundContents(wrapper); document.getElementById('out').textContent = box.innerHTML + '|' + wrapper.innerHTML;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "<strong><span>Hello</span><em>!</em></strong>|<span>Hello</span><em>!</em>");
}

test "contract: Harness.fromHtml selects range contents during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='box'><span>Hello</span><em>!</em></div><div id='out'></div><script>const box = document.getElementById('box'); const contents = document.createRange().selectNodeContents(box); const node = document.createRange().selectNode(box); document.getElementById('out').textContent = String(contents) + '|' + String(contents.startOffset) + ':' + String(contents.endOffset) + ':' + String(contents.collapsed) + '|' + String(node) + '|' + String(node.startOffset) + ':' + String(node.endOffset) + ':' + String(node.collapsed);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "Hello!|0:6:false|Hello!|0:6:false");
}

test "contract: Harness.fromHtml checks points in selection ranges during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const range = document.getSelection().getRangeAt(0); document.getElementById('out').textContent = String(range.isPointInRange(name, 4)) + ':' + String(range.isPointInRange(name, 9)) + ':' + String(range.isPointInRange(name, 12)) + ':' + String(range.isPointInRange(name, 3));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "true:true:true:false");
}

test "contract: Harness.fromHtml intersects selection ranges during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const range = document.getSelection().getRangeAt(0); document.getElementById('out').textContent = String(range.intersectsNode(name));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "true");
}

test "contract: Harness.fromHtml compares points in selection ranges during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const range = document.getSelection().getRangeAt(0); document.getElementById('out').textContent = String(range.comparePoint(name, 4)) + ':' + String(range.comparePoint(name, 9)) + ':' + String(range.comparePoint(name, 12)) + ':' + String(range.comparePoint(name, 3));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "0:0:0:-1");
}

test "contract: Harness.fromHtml clones selection range contents during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const range = document.getSelection().getRangeAt(0); const fragment = range.cloneContents(); document.getElementById('out').textContent = fragment.textContent + '|' + String(fragment) + '|' + name.value + '|' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + '|' + String(document.getSelection().rangeCount) + ':' + String(document.getSelection().type);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "Lovelace|[object DocumentFragment]|Ada Lovelace|4:12:backward|1:Range");
}

test "contract: Harness.fromHtml compares selection range boundaries during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const first = document.getSelection().getRangeAt(0); name.setSelectionRange(1, 3, 'backward'); const second = document.getSelection().getRangeAt(0); document.getElementById('out').textContent = String(first.compareBoundaryPoints(0, second)) + ':' + String(first.compareBoundaryPoints(1, second)) + ':' + String(first.compareBoundaryPoints(2, second)) + ':' + String(first.compareBoundaryPoints(3, second));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "1:1:1:1");
}

test "contract: Harness.fromHtml extracts selection ranges during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const range = document.getSelection().getRangeAt(0); const fragment = range.extractContents(); document.getElementById('out').textContent = fragment.textContent + '|' + String(fragment) + '|' + name.value + '|' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + '|' + String(document.getSelection().rangeCount) + ':' + String(document.getSelection().type) + ':' + String(document.getSelection().anchorOffset) + ':' + String(document.getSelection().focusOffset);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "Lovelace|[object DocumentFragment]|Ada |4:4:none|1:Caret:4:4");
}

test "contract: Harness.fromHtml deletes selection range contents during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const range = document.getSelection().getRangeAt(0); range.deleteContents(); document.getElementById('out').textContent = name.value + '|' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + '|' + String(document.getSelection().rangeCount) + ':' + String(document.getSelection().type) + ':' + String(document.getSelection().anchorOffset) + ':' + String(document.getSelection().focusOffset);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "Ada |4:4:none|1:Caret:4:4");
}

test "contract: Harness.fromHtml adds and removes selection ranges during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada'><div id='count'></div><div id='out'></div><script>const name = document.getElementById('name'); const count = document.getElementById('count'); document.onselectionchange = () => { document.getElementById('count').textContent += '1'; }; name.focus(); name.setSelectionRange(1, 3, 'backward'); const selection = document.getSelection(); const range = selection.getRangeAt(0); selection.removeRange(range); const removedSelection = document.getSelection(); const removed = String(count.textContent) + ':' + String(removedSelection.rangeCount) + ':' + removedSelection.type + ':' + String(removedSelection.anchorNode) + ':' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection; selection.addRange(range); const restoredSelection = document.getSelection(); const restored = String(count.textContent) + ':' + String(restoredSelection.rangeCount) + ':' + restoredSelection.type + ':' + String(restoredSelection) + ':' + String(restoredSelection.anchorOffset) + ':' + String(restoredSelection.focusOffset) + ':' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection; selection.removeAllRanges(); selection.removeRange(range); const clearedSelection = document.getSelection(); const cleared = String(count.textContent) + ':' + String(clearedSelection.rangeCount) + ':' + clearedSelection.type + ':' + String(clearedSelection.anchorNode); document.getElementById('out').textContent = removed + '|' + restored + '|' + cleared;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "11:0:None:null:1:3:backward|111:1:Range:da:1:3:1:3:none|1111:0:None:null");
}

test "contract: Harness.fromHtml deletes selection snapshots from the document during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const selection = document.getSelection(); selection.deleteFromDocument(); const current = document.getSelection(); document.getElementById('out').textContent = name.value + '|' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + '|' + String(current.rangeCount) + ':' + current.type + ':' + String(current.anchorOffset) + ':' + String(current.focusOffset);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "Ada |4:4:none|1:Caret:4:4");
}

test "contract: Harness.fromHtml removes selection snapshots during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada'><div id='out'></div><script>const name = document.getElementById('name'); document.getElementById('out').textContent = ''; document.onselectionchange = () => { document.getElementById('out').textContent += '1'; }; name.focus(); name.setSelectionRange(1, 3, 'backward'); const selection = document.getSelection(); selection.removeAllRanges(); const current = document.getSelection(); document.getElementById('out').textContent += '|' + String(current.rangeCount) + '|' + String(current.isCollapsed) + '|' + current.type + '|' + String(current.anchorNode);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "11|0|true|None|null");
}

test "contract: Harness.fromHtml runs selectionchange handlers during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); document.onselectionchange = () => { document.getElementById('out').textContent += '1'; }; name.setSelectionRange(4, 12); name.setRangeText('Byron', 4, 12, 'select'); document.getElementById('out').textContent += ':' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "11:4:9:none");
}

test "contract: Harness.fromHtml runs readystatechange handlers during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>document.onreadystatechange = () => { document.getElementById('out').textContent += ':' + document.readyState; }; document.getElementById('out').textContent = document.readyState;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "loading:complete");
}

test "contract: Harness.fromHtml dispatches DOMContentLoaded during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>document.addEventListener('DOMContentLoaded', () => { document.getElementById('out').textContent += ':dom:' + document.readyState; }); document.onreadystatechange = () => { document.getElementById('out').textContent += ':ready:' + document.readyState; }; document.getElementById('out').textContent = document.readyState;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "loading:dom:loading:ready:complete");
}

test "failure: Harness.fromHtml rejects readystatechange handlers on unsupported targets" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.onreadystatechange = 1;</script>",
        ),
    );
}

test "contract: Harness.fromHtml runs stepUp and stepDown on numeric inputs during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='count' type='number' min='2' max='6' step='2' value='2'><div id='out'></div><script>const count = document.getElementById('count'); const before = count.value + ':' + String(count.validity.stepMismatch); count.stepUp(); const afterUp = count.value + ':' + String(count.validity.stepMismatch); count.stepDown(2); const afterDown = count.value + ':' + String(count.validity.stepMismatch); document.getElementById('out').textContent = before + '|' + afterUp + '|' + afterDown;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:false|4:false|2:false");
}

test "contract: Harness.fromHtml runs stepUp and stepDown on date and month controls during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='date' type='date' value='2017-06-01'><input id='month' type='month' step='2' value='2017-06'><input id='time' type='time' step='1800' value='09:00'><input id='week' type='week' value='2017-W01'><div id='out'></div><script>const date = document.getElementById('date'); const month = document.getElementById('month'); const time = document.getElementById('time'); const week = document.getElementById('week'); date.stepUp(2); month.stepUp(); time.stepDown(); week.stepUp(); document.getElementById('out').textContent = date.value + '|' + month.value + '|' + time.value + '|' + week.value;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2017-06-03|2017-08|08:30|2017-W02");
}

test "contract: Harness.fromHtml runs input.valueAsNumber getters and setters during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='num' type='number' value='42.5'><input id='date' type='date' value='2017-06-01'><input id='dt' type='datetime-local' value='2017-06-01T08:30'><input id='time' type='time' value='15:30:05.006'><input id='range' type='range' min='2' max='10' step='2' value='9'><input id='text' type='text' value='5'><div id='out'></div><script>const num = document.getElementById('num'); const date = document.getElementById('date'); const dt = document.getElementById('dt'); const time = document.getElementById('time'); const range = document.getElementById('range'); const text = document.getElementById('text'); const notANumber = text.valueAsNumber; const first = num.valueAsNumber + ':' + date.valueAsNumber + ':' + dt.valueAsNumber + ':' + time.valueAsNumber + ':' + range.valueAsNumber + ':' + String(notANumber); num.valueAsNumber = 10; date.valueAsNumber = 1496275200000; dt.valueAsNumber = 1496305805006; time.valueAsNumber = 32405006; range.valueAsNumber = 9; const second = num.value + ':' + num.valueAsNumber + '|' + date.value + ':' + date.valueAsNumber + '|' + dt.value + ':' + dt.valueAsNumber + '|' + time.value + ':' + time.valueAsNumber + '|' + range.value + ':' + range.valueAsNumber; num.valueAsNumber = notANumber; date.valueAsNumber = notANumber; dt.valueAsNumber = notANumber; time.valueAsNumber = notANumber; range.valueAsNumber = notANumber; const third = '[' + num.value + ']:' + String(num.valueAsNumber) + '|[' + date.value + ']:' + String(date.valueAsNumber) + '|[' + dt.value + ']:' + String(dt.valueAsNumber) + '|[' + time.value + ']:' + String(time.valueAsNumber) + '|' + range.value + ':' + range.valueAsNumber; document.getElementById('out').textContent = first + '|' + second + '|' + third;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "42.5:1496275200000:1496305800000:55805006:10:NaN|10:10|2017-06-01:1496275200000|2017-06-01T08:30:05.006:1496305805006|09:00:05.006:32405006|10:10|[]:NaN|[]:NaN|[]:NaN|[]:NaN|6:6",
    );
}

test "contract: Harness.fromHtml runs input.valueAsDate getters and setters during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='date' type='date' value='2017-06-01'><input id='dt' type='datetime-local' value='2017-06-01T08:30:05.006'><input id='time' type='time' value='09:00:05.006'><input id='month' type='month' value='2017-06'><input id='text' type='text' value='ignored'><div id='out'></div><script>const date = document.getElementById('date'); const dt = document.getElementById('dt'); const time = document.getElementById('time'); const month = document.getElementById('month'); const text = document.getElementById('text'); const dateObj = new Date(1496275200000); const dtObj = new Date(1496305805006); const timeObj = new Date(32405006); const first = date.valueAsDate.toISOString() + ':' + String(date.valueAsDate.valueOf()) + '|' + dt.valueAsDate.toISOString() + ':' + String(dt.valueAsDate.valueOf()) + '|' + time.valueAsDate.toISOString() + ':' + String(time.valueAsDate.valueOf()) + '|' + month.valueAsDate.toISOString() + ':' + String(month.valueAsDate.valueOf()) + '|' + String(text.valueAsDate); date.valueAsDate = dateObj; dt.valueAsDate = dtObj; time.valueAsDate = timeObj; month.valueAsDate = dateObj; const second = date.value + ':' + date.valueAsDate.toISOString() + '|' + dt.value + ':' + dt.valueAsDate.toISOString() + '|' + time.value + ':' + time.valueAsDate.toISOString() + '|' + month.value + ':' + month.valueAsDate.toISOString(); date.valueAsDate = null; dt.valueAsDate = null; time.valueAsDate = null; month.valueAsDate = null; const third = '[' + date.value + ']:' + String(date.valueAsDate) + '|[' + dt.value + ']:' + String(dt.valueAsDate) + '|[' + time.value + ']:' + String(time.valueAsDate) + '|[' + month.value + ']:' + String(month.valueAsDate); document.getElementById('out').textContent = first + '|' + second + '|' + third;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "2017-06-01T00:00:00.000Z:1496275200000|2017-06-01T08:30:05.006Z:1496305805006|1970-01-01T09:00:05.006Z:32405006|2017-06-01T00:00:00.000Z:1496275200000|null|2017-06-01:2017-06-01T00:00:00.000Z|2017-06-01T08:30:05.006:2017-06-01T08:30:05.006Z|09:00:05.006:1970-01-01T09:00:05.006Z|2017-06:2017-06-01T00:00:00.000Z|[]:null|[]:null|[]:null|[]:null",
    );
}

test "contract: Harness.fromHtml runs input.list datalist association during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='search' list='suggestions'><input id='orphan' list='missing'><datalist id='suggestions'><option id='first' value='alpha'>Alpha</option><option value='beta'>Beta</option></datalist><div id='out'></div><script>const search = document.getElementById('search'); const orphan = document.getElementById('orphan'); const list = search.list; document.getElementById('out').textContent = list.id + ':' + String(list.options.length) + ':' + list.options.item(0).value + ':' + String(orphan.list);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "suggestions:2:alpha:null");
}

test "contract: Harness.fromHtml runs showPicker on supported controls during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='date' type='date' value='2017-06-01'><select id='mode'><option value='a'>A</option><option value='b' selected>B</option></select><div id='out'></div><script>const date = document.getElementById('date'); const mode = document.getElementById('mode'); document.getElementById('out').textContent = String(date.showPicker()) + ':' + String(mode.showPicker()) + ':' + date.value + ':' + String(mode.selectedIndex);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "undefined:undefined:2017-06-01:1");
}

test "contract: Harness.fromHtml runs popover controls during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><article id='popover' popover='manual'></article><div id='count'></div><div id='out'></div><script>const popover = document.getElementById('popover'); const count = document.getElementById('count'); popover.addEventListener('toggle', () => { document.getElementById('count').textContent += '1'; }); const before = popover.popover + ':' + String(popover.matches(':popover-open')) + ':' + String(document.querySelectorAll(':popover-open').length); popover.showPopover(); const afterShow = String(popover.matches(':popover-open')) + ':' + count.textContent + ':' + String(document.querySelectorAll(':popover-open').length); popover.hidePopover(); const afterHide = String(popover.matches(':popover-open')) + ':' + count.textContent + ':' + String(document.querySelectorAll(':popover-open').length); const forcedHide = String(popover.togglePopover(false)) + ':' + String(popover.matches(':popover-open')) + ':' + count.textContent + ':' + String(document.querySelectorAll(':popover-open').length); const toggled = String(popover.togglePopover()) + ':' + String(popover.matches(':popover-open')) + ':' + count.textContent + ':' + String(document.querySelectorAll(':popover-open').length); document.getElementById('out').textContent = before + '|' + afterShow + '|' + afterHide + '|' + forcedHide + '|' + toggled;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "manual:false:0|true:1:1|false:11:0|false:false:11:0|true:true:111:1");
}

test "contract: Harness.fromHtml runs HTMLElement.click popover target activation during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><article id='popover' popover='manual'></article><button id='trigger' popovertarget='popover' popovertargetaction='show'></button><div id='out'></div><script>const popover = document.getElementById('popover'); const trigger = document.getElementById('trigger'); trigger.click(); document.getElementById('out').textContent = popover.popover + ':' + String(popover.matches(':popover-open')) + ':' + String(document.querySelectorAll(':popover-open').length);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "manual:true:1");
}

test "contract: Harness.fromHtml runs input.indeterminate during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><progress id='loading'></progress><input id='check' type='checkbox'><div id='out'></div><script>const check = document.getElementById('check'); const before = String(check.indeterminate) + ':' + String(document.querySelectorAll(':indeterminate').length); check.indeterminate = true; const after = String(check.indeterminate) + ':' + String(document.querySelectorAll(':indeterminate').length); check.indeterminate = false; const cleared = String(check.indeterminate) + ':' + String(document.querySelectorAll(':indeterminate').length); document.getElementById('out').textContent = before + '|' + after + '|' + cleared;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "false:1|true:2|false:1");
}

test "contract: Harness.fromHtml runs input.accept and input.size during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='upload' type='file' accept='.png,image/*'><input id='plain' type='text'><input id='fixed' type='text' size='12'><div id='out'></div><script>const upload = document.getElementById('upload'); const plain = document.getElementById('plain'); const fixed = document.getElementById('fixed'); const before = upload.accept + ':' + String(plain.size) + ':' + String(fixed.size); upload.accept = 'audio/*'; plain.size = 18; fixed.size = 6; document.getElementById('out').textContent = before + '|' + upload.accept + ':' + String(plain.size) + ':' + String(fixed.size) + ':' + upload.getAttribute('accept') + ':' + plain.getAttribute('size') + ':' + fixed.getAttribute('size');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", ".png,image/*:20:12|audio/*:18:6:audio/*:18:6");
}

test "contract: Harness.fromHtml runs input.files iterator helpers during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='upload' type='file'><div id='out'></div><script>const upload = document.getElementById('upload'); const keys = upload.files.keys(); const values = upload.files.values(); const entries = upload.files.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); const before = String(upload.files.length) + ':' + String(firstKey.done) + ':' + String(firstValue.done) + ':' + String(firstEntry.done) + ':' + String(upload.files.item(0)); document.getElementById('out').textContent = ''; upload.files.forEach((value, index, list) => { document.getElementById('out').textContent += String(index) + ':' + value + ':' + String(list.length) + ';'; }, null); document.getElementById('out').textContent = before + '|' + document.getElementById('out').textContent;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "0:true:true:true:null|");
}

test "contract: Harness.fromHtml exposes HTMLInputElement image reflection" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='photo' type='image' src='/photo.png' alt='Photo' usemap='#map' width='320' height='240'><div id='out'></div><script>const photo = document.getElementById('photo'); const before = photo.src + ':' + photo.alt + ':' + photo.useMap + ':' + String(photo.width) + ':' + String(photo.height); photo.src = '/next.png'; photo.alt = 'Next'; photo.useMap = '#next-map'; photo.width = 640; photo.height = 480; document.getElementById('out').textContent = before + '|' + photo.src + ':' + photo.alt + ':' + photo.useMap + ':' + String(photo.width) + ':' + String(photo.height) + ':' + photo.getAttribute('src') + ':' + photo.getAttribute('alt') + ':' + photo.getAttribute('usemap') + ':' + photo.getAttribute('width') + ':' + photo.getAttribute('height');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "/photo.png:Photo:#map:320:240|/next.png:Next:#next-map:640:480:/next.png:Next:#next-map:640:480",
    );
}

test "contract: Harness.fromHtml runs HTMLCanvasElement reflection during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const canvas = document.createElement('canvas'); const before = String(canvas.width) + ':' + String(canvas.height) + ':' + String(canvas.getContext('2d')); canvas.width = 0; canvas.height = 1; const during = String(canvas.width) + ':' + String(canvas.height) + ':' + canvas.getAttribute('width') + ':' + canvas.getAttribute('height') + ':' + String(canvas.getContext('bitmaprenderer')); canvas.width = 640; canvas.height = 480; document.getElementById('out').textContent = before + '|' + during + '|' + String(canvas.width) + ':' + String(canvas.height) + ':' + canvas.getAttribute('width') + ':' + canvas.getAttribute('height');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "300:150:null|0:1:0:1:null|640:480:640:480");
}

test "contract: Harness.fromHtml runs HTMLSlotElement reflection during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><slot id='slot' name='main'><span id='fallback'>Fallback</span>text</slot><div id='out'></div><script>const slot = document.getElementById('slot'); const before = slot.name + ':' + String(slot.assignedNodes().length) + ':' + String(slot.assignedElements().length) + ':' + slot.assignedNodes().item(0).nodeName + ':' + slot.assignedNodes().item(1).nodeValue + ':' + slot.assignedElements().item(0).id; slot.name = 'next'; document.getElementById('out').textContent = before + '|' + slot.name + ':' + slot.getAttribute('name') + ':' + String(slot.assignedNodes().length) + ':' + String(slot.assignedElements().length);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "main:2:1:span:text:fallback|next:next:2:1");
}

test "contract: Harness.fromHtml runs HTMLInputElement color reflection during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='color' type='color' colorspace='display-p3' alpha><div id='out'></div><script>const color = document.getElementById('color'); const before = String(color.alpha) + ':' + color.colorSpace + ':' + color.getAttribute('colorspace') + ':' + String(color.hasAttribute('alpha')); color.alpha = false; color.colorSpace = 'bogus'; document.getElementById('out').textContent = before + '|' + String(color.alpha) + ':' + color.colorSpace + ':' + color.getAttribute('colorspace') + ':' + String(color.hasAttribute('alpha'));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "true:display-p3:display-p3:true|false:limited-srgb:limited-srgb:false");
}

test "contract: Harness.fromHtml runs HTMLInputElement capture reflection during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='file' type='file' capture='environment'><div id='out'></div><script>const file = document.getElementById('file'); const before = file.capture + ':' + file.getAttribute('capture'); file.capture = 'user'; const during = file.capture + ':' + file.getAttribute('capture'); file.capture = 'bogus'; const after = file.capture + ':' + file.getAttribute('capture'); document.getElementById('out').textContent = before + '|' + during + '|' + after;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "environment:environment|user:user|:bogus");
}

test "contract: Harness.fromHtml runs blocking attribute reflection during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><style id='style' blocking='render'></style><link id='link' rel='stylesheet' blocking='render' href='a.css'><div id='out'></div><script>const style = document.getElementById('style'); const link = document.getElementById('link'); const script = document.createElement('script'); const before = style.blocking.value + ':' + link.blocking.value + ':' + script.blocking.value + ':' + String(style.blocking.length) + ':' + String(link.blocking.contains('render')) + ':' + String(script.blocking.supports('render')) + ':' + String(script.blocking.supports('layout')); script.blocking.add('render'); style.blocking.remove('render'); link.blocking.remove('render'); script.blocking.remove('render'); document.getElementById('out').textContent = before + '|' + style.blocking.value + ':' + link.blocking.value + ':' + script.blocking.value + ':' + String(style.getAttribute('blocking')) + ':' + String(link.getAttribute('blocking')) + ':' + String(script.getAttribute('blocking')) + ':' + String(style.blocking.length) + ':' + String(link.blocking.length) + ':' + String(script.blocking.length);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "render:render::1:true:true:false|:::null:null:null:0:0:0");
}

test "contract: Harness.fromHtml exposes HTMLFormElement rel reflection during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><form id='form' rel='noopener noreferrer opener noopener'></form><div id='out'></div><script>const form = document.getElementById('form'); const before = form.rel + ':' + String(form.relList.length) + ':' + String(form.relList.contains('noopener')) + ':' + String(form.relList.supports('noopener')) + ':' + String(form.relList.supports('opener')) + ':' + String(form.relList.supports('stylesheet')); form.rel = 'opener'; const during = form.rel + ':' + form.relList.value + ':' + form.getAttribute('rel') + ':' + String(form.relList.supports('noopener')) + ':' + String(form.relList.supports('stylesheet')); form.relList.value = 'noreferrer noopener'; const after = form.rel + ':' + form.relList.value + ':' + form.getAttribute('rel') + ':' + String(form.relList.supports('noopener')) + ':' + String(form.relList.supports('stylesheet')); document.getElementById('out').textContent = before + '|' + during + '|' + after;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "noopener noreferrer opener noopener:3:true:true:true:false|opener:opener:opener:true:false|noreferrer noopener:noreferrer noopener:noreferrer noopener:true:false",
    );
}

test "contract: Harness.fromHtml runs enterKeyHint during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='box' enterkeyhint='search'></div><textarea id='area'></textarea><div id='out'></div><script>const box = document.getElementById('box'); const area = document.getElementById('area'); const before = box.enterKeyHint + ':' + area.enterKeyHint; box.enterKeyHint = 'send'; area.enterKeyHint = 'next'; document.getElementById('out').textContent = before + '|' + box.enterKeyHint + ':' + area.enterKeyHint + ':' + box.getAttribute('enterkeyhint') + ':' + area.getAttribute('enterkeyhint');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "search:|send:next:send:next");
}

test "contract: Harness.fromHtml runs dirName during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='input' dirname='input.dir'><textarea id='area' dirname='area.dir'></textarea><div id='out'></div><script>const input = document.getElementById('input'); const area = document.getElementById('area'); const before = input.dirName + ':' + area.dirName; input.dirName = 'input.next'; area.dirName = 'area.next'; document.getElementById('out').textContent = before + '|' + input.dirName + ':' + area.dirName + ':' + input.getAttribute('dirname') + ':' + area.getAttribute('dirname');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "input.dir:area.dir|input.next:area.next:input.next:area.next");
}

test "contract: Harness.fromHtml runs HTMLMediaElement playback state during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><video id='clip' muted></video><div id='out'></div><script>const clip = document.getElementById('clip'); const before = String(clip.defaultMuted) + ':' + String(clip.muted) + ':' + String(clip.volume) + ':' + String(clip.defaultPlaybackRate) + ':' + String(clip.playbackRate) + ':' + String(clip.preservesPitch); clip.muted = false; clip.volume = 0.25; clip.defaultPlaybackRate = 1.5; clip.playbackRate = 0.5; clip.preservesPitch = false; clip.disablePictureInPicture = true; clip.disableRemotePlayback = true; clip.controlsList.add('nodownload', 'nofullscreen', 'noremoteplayback'); document.getElementById('out').textContent = before + '|' + String(clip.defaultMuted) + ':' + String(clip.muted) + ':' + String(clip.volume) + ':' + String(clip.defaultPlaybackRate) + ':' + String(clip.playbackRate) + ':' + String(clip.preservesPitch) + ':' + String(clip.disablePictureInPicture) + ':' + String(clip.disableRemotePlayback) + ':' + String(clip.controlsList.value) + ':' + String(clip.controlsList.length) + ':' + String(clip.controlsList.item(0)) + ':' + String(clip.controlsList.supports('noremoteplayback')) + ':' + String(clip.controlsList.contains('nodownload')) + ':' + String(clip.hasAttribute('disablepictureinpicture')) + ':' + String(clip.hasAttribute('disableremoteplayback')) + ':' + String(clip.hasAttribute('controlslist')) + ':' + clip.getAttribute('muted');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "true:true:1:1:1:true|true:false:0.25:1.5:0.5:false:true:true:nodownload nofullscreen noremoteplayback:3:nodownload:true:true:true:true:true:");
}

test "contract: Harness.fromHtml runs HTMLAudioElement playback state during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><audio id='clip' src='track.ogg' muted controls preload='metadata' crossorigin='anonymous'></audio><div id='out'></div><script>const clip = document.getElementById('clip'); const before = String(clip.currentSrc) + ':' + String(clip.defaultMuted) + ':' + String(clip.muted) + ':' + String(clip.volume) + ':' + String(clip.defaultPlaybackRate) + ':' + String(clip.playbackRate) + ':' + String(clip.preservesPitch) + ':' + String(clip.disableRemotePlayback) + ':' + String(clip.controlsList.value); clip.muted = false; clip.volume = 0.25; clip.defaultPlaybackRate = 1.5; clip.playbackRate = 0.5; clip.preservesPitch = false; clip.disableRemotePlayback = true; clip.controlsList.add('nodownload', 'noremoteplayback'); document.getElementById('out').textContent = before + '|' + String(clip.currentSrc) + ':' + String(clip.defaultMuted) + ':' + String(clip.muted) + ':' + String(clip.volume) + ':' + String(clip.defaultPlaybackRate) + ':' + String(clip.playbackRate) + ':' + String(clip.preservesPitch) + ':' + String(clip.disableRemotePlayback) + ':' + String(clip.controlsList.value) + ':' + String(clip.controlsList.length) + ':' + String(clip.controlsList.item(0)) + ':' + String(clip.controlsList.supports('noremoteplayback')) + ':' + String(clip.controlsList.contains('nodownload')) + ':' + clip.getAttribute('crossorigin');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "track.ogg:true:true:1:1:1:true:false:|track.ogg:true:false:0.25:1.5:0.5:false:true:nodownload noremoteplayback:2:nodownload:true:true:anonymous");
}

test "contract: Harness.fromHtml runs HTMLTrackElement reflection during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><track id='cue' kind='subtitles' src='captions.vtt' srclang='en' label='English' default><div id='out'></div><script>const cue = document.getElementById('cue'); const before = cue.kind + ':' + cue.src + ':' + cue.srclang + ':' + cue.label + ':' + String(cue.default) + ':' + String(cue.readyState) + ':' + String(cue.track) + ':' + cue.track.kind + ':' + cue.track.label + ':' + cue.track.language + ':' + cue.track.mode + ':' + String(cue.track.readyState); cue.kind = 'captions'; cue.src = 'subtitles.vtt'; cue.srclang = 'fr'; cue.label = 'Français'; cue.default = false; cue.track.mode = 'showing'; document.getElementById('out').textContent = before + '|' + cue.kind + ':' + cue.src + ':' + cue.srclang + ':' + cue.label + ':' + String(cue.default) + ':' + String(cue.readyState) + ':' + String(cue.track) + ':' + cue.track.kind + ':' + cue.track.label + ':' + cue.track.language + ':' + cue.track.mode + ':' + String(cue.track.readyState);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "subtitles:captions.vtt:en:English:true:0:[object TextTrack]:subtitles:English:en:disabled:0|captions:subtitles.vtt:fr:Français:false:0:[object TextTrack]:captions:Français:fr:showing:0");
}

test "contract: Harness.fromHtml runs HTMLDataElement and HTMLTimeElement reflection during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><data id='datum' value='UPC:022014640201'>North Coast Organic Apple Cider</data><time id='stamp'>2011-11-18</time><div id='out'></div><script>const datum = document.getElementById('datum'); const stamp = document.getElementById('stamp'); const before = datum.value + ':' + stamp.dateTime + ':' + String(stamp.getAttribute('datetime')); datum.value = 'UPC:022014640202'; stamp.dateTime = '2012-05-30T10:00'; document.getElementById('out').textContent = before + '|' + datum.value + ':' + stamp.dateTime + ':' + datum.getAttribute('value') + ':' + stamp.getAttribute('datetime') + ':' + stamp.textContent;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "UPC:022014640201:2011-11-18:null|UPC:022014640202:2012-05-30T10:00:UPC:022014640202:2012-05-30T10:00:2011-11-18",
    );
}

test "contract: Harness.fromHtml runs HTMLTemplateElement declarative shadow root flags during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><template id='tpl' shadowrootmode='open' shadowrootdelegatesfocus shadowrootclonable shadowrootserializable shadowrootcustomelementregistry='registry'><span>Hidden</span></template><div id='out'></div><script>const tpl = document.getElementById('tpl'); const before = tpl.shadowRootMode + ':' + String(tpl.shadowRootDelegatesFocus) + ':' + String(tpl.shadowRootClonable) + ':' + String(tpl.shadowRootSerializable) + ':' + tpl.shadowRootCustomElementRegistry; tpl.shadowRootMode = 'closed'; tpl.shadowRootDelegatesFocus = false; tpl.shadowRootClonable = false; tpl.shadowRootSerializable = true; tpl.shadowRootCustomElementRegistry = 'registry-2'; document.getElementById('out').textContent = before + '|' + tpl.shadowRootMode + ':' + String(tpl.shadowRootDelegatesFocus) + ':' + String(tpl.shadowRootClonable) + ':' + String(tpl.shadowRootSerializable) + ':' + tpl.shadowRootCustomElementRegistry + ':' + String(tpl.hasAttribute('shadowrootmode')) + ':' + String(tpl.hasAttribute('shadowrootdelegatesfocus')) + ':' + String(tpl.hasAttribute('shadowrootclonable')) + ':' + String(tpl.hasAttribute('shadowrootserializable'));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "open:true:true:true:registry|closed:false:false:true:registry-2:true:false:false:true",
    );
}

test "failure: Harness.fromHtml rejects stepUp on unsupported controls" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').stepUp();</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects HTMLTimeElement.dateTime on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='host'></div><script>document.getElementById('host').dateTime = '2012-05-30';</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects HTMLMediaElement volume on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').volume = 0.5;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects HTMLTrackElement mode on unsupported values" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><track id='cue'><script>document.getElementById('cue').track.mode = 'loud';</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects HTMLInputElement image reflection on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('input').src;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects HTMLInputElement color reflection on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('input').colorSpace = 'display-p3';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects HTMLInputElement capture reflection on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').capture = 'environment';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects HTMLCanvasElement getContext on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').getContext('2d');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects HTMLSlotElement assignedNodes on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').assignedNodes();</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects HTMLFormElement rel reflection on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='out'></div><script>document.createElement('div').rel;</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects out of range HTMLMediaElement volume" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><video id='clip'></video><script>document.getElementById('clip').volume = 1.5;</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects HTMLMediaElement disableRemotePlayback on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').disableRemotePlayback = true;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects HTMLVideoElement disablePictureInPicture on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('audio').disablePictureInPicture = true;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects HTMLMediaElement controlsList on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').controlsList;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects blocking reflection on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').blocking;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects selection extend on unsupported targets" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='out'></div><script>const selection = document.getSelection(); selection.extend(document.body, 1);</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects selection base and extent across nodes" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><input id='name' value='Ada'><textarea id='bio'>Hello</textarea></main><script>const selection = document.getSelection(); selection.setBaseAndExtent(document.getElementById('name'), 1, document.getElementById('bio'), 2);</script>",
        ),
    );
}

test "contract: Harness.fromHtml runs tree mutation insertBefore and replaceChild during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='target'><button id='second'>Second</button><button id='third'>Third</button></section><button id='first'>First</button><button id='extra'>Extra</button><div id='out'></div><script>document.getElementById('target').insertBefore(document.getElementById('first'), document.getElementById('second')); document.getElementById('target').replaceChild(document.getElementById('extra'), document.getElementById('second')); document.getElementById('first').remove(); document.getElementById('out').textContent = document.getElementById('target').textContent + ':' + String(document.querySelectorAll('#target > button').length) + ':' + document.querySelector('#target > #extra').textContent + ':' + document.querySelector('#target > #third').textContent;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "ExtraThird:2:Extra:Third");
    try subject.assertExists("#target > #extra");
    try subject.assertExists("#target > #third");
    try std.testing.expectError(error.AssertionFailed, subject.assertExists("#second"));
    try std.testing.expectError(error.AssertionFailed, subject.assertExists("#first"));
}

test "contract: Harness.fromHtml runs tree mutation replaceChildren with existing children during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='target'><span id='placeholder'>Placeholder</span></section><button id='first'>First</button><button id='second'>Second</button><div id='out'></div><script>document.getElementById('target').replaceChildren(document.getElementById('first'), document.getElementById('placeholder'), document.getElementById('second')); document.getElementById('out').textContent = document.getElementById('target').textContent + ':' + String(document.querySelectorAll('#target > button').length) + ':' + document.querySelector('#target > #placeholder').textContent;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "FirstPlaceholderSecond:2:Placeholder");
    try subject.assertExists("#target > #first");
    try subject.assertExists("#target > #placeholder");
    try subject.assertExists("#target > #second");
}

test "contract: Harness.fromHtml runs tree mutation before and after during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='source'><button id='second'>Second</button><button id='third'>Third</button></section><button id='first'>First</button><div id='out'></div><script>document.getElementById('second').before(document.getElementById('first')); document.getElementById('second').after(document.getElementById('third')); document.getElementById('out').textContent = document.getElementById('source').textContent + ':' + String(document.querySelectorAll('#source > button').length) + ':' + document.querySelector('#first').textContent + ':' + document.querySelector('#third').textContent;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "FirstSecondThird:3:First:Third");
    try subject.assertExists("#source > #first");
    try subject.assertExists("#source > #second");
    try subject.assertExists("#source > #third");
}

test "contract: Harness.fromHtml runs tree mutation replaceWith during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='source'><button id='old'>Old</button><span id='tail'>Tail</span></section><button id='replacement'>Replacement</button><div id='out'></div><script>document.getElementById('old').replaceWith(document.getElementById('replacement')); document.getElementById('out').textContent = document.getElementById('source').textContent + ':' + String(document.querySelectorAll('#source > button').length) + ':' + document.querySelector('#source > #replacement').textContent + ':' + String(document.querySelector('#old'));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "ReplacementTail:1:Replacement:null");
    try subject.assertExists("#source > #replacement");
    try subject.assertExists("#source > #tail");
    try std.testing.expectError(error.AssertionFailed, subject.assertExists("#old"));
}

test "contract: Harness.fromHtml runs tree mutation removeChild during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='target'><button id='first'>First</button><button id='second'>Second</button></section><div id='out'></div><script>const target = document.getElementById('target'); const second = document.getElementById('second'); const removed = target.removeChild(second); document.getElementById('out').textContent = String(removed) + ':' + removed.textContent + ':' + String(document.querySelector('#second')) + ':' + String(target.childNodes.length);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "[object Element]:Second:null:1");
    try subject.assertExists("#target > #first");
    try std.testing.expectError(error.AssertionFailed, subject.assertExists("#second"));
}

test "contract: Harness.fromHtml runs tree mutation cloneNode during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='source' data-kind='orig'><button id='button'>One</button><span id='tail'>Tail</span></section><template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const clone = document.getElementById('source').cloneNode(true); const fragment = document.getElementById('tpl').content.cloneNode(); document.getElementById('out').textContent = clone.getAttribute('id') + ':' + clone.getAttribute('data-kind') + ':' + String(clone.parentNode) + ':' + clone.textContent + ':' + String(clone.querySelectorAll('button').length) + ':' + String(document.querySelectorAll('#source').length) + '|' + String(fragment) + ':' + fragment.innerHTML + ':' + String(fragment.childNodes.length) + ':' + String(fragment.querySelector('#inner')) + ':' + document.getElementById('tpl').content.textContent;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "source:orig:null:OneTail:1:1|[object DocumentFragment]::0:null:Inner",
    );
    try subject.assertExists("#source");
    try subject.assertExists("#source > #button");
    try subject.assertExists("#source > #tail");
    try subject.assertExists("#tpl");
    try subject.assertExists("#tpl > #inner");
}

test "contract: Harness.fromHtml runs normalize and importNode during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='target'>a</div><template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const text = document.getElementById('target').childNodes.item(0); text.replaceWith(document.createTextNode('a'), document.createTextNode(''), document.createTextNode('b')); document.getElementById('target').normalize(); const fragment = document.importNode(document.getElementById('tpl').content, true); document.getElementById('out').textContent = String(document.getElementById('target').childNodes.length) + ':' + document.getElementById('target').textContent + '|' + String(fragment) + ':' + fragment.innerHTML + ':' + String(fragment.childNodes.length) + ':' + String(fragment.querySelector('#inner'));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "1:ab|[object DocumentFragment]:<span id=\"inner\">Inner</span>:1:[object Element]");
    try subject.assertExists("#target");
    try subject.assertExists("#tpl > #inner");
}

test "contract: Harness.fromHtml runs Node.contains during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='outer'><span id='inside'>Inside</span></section><template id='tpl'><span id='frag'>Frag</span></template><div id='out'></div><script>const outer = document.getElementById('outer'); const inside = document.getElementById('inside'); const fragment = document.getElementById('tpl').content; const fragmentChild = fragment.querySelector('#frag'); document.getElementById('out').textContent = String(document.contains(document)) + ':' + String(document.contains(outer)) + ':' + String(document.contains(inside)) + ':' + String(outer.contains(inside)) + ':' + String(inside.contains(outer)) + ':' + String(fragment.contains(fragmentChild)) + ':' + String(fragment.contains(document)) + ':' + String(document.contains(null)) + ':' + String(document.contains(fragmentChild));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "true:true:true:true:false:true:false:false:false");
    try subject.assertExists("#outer > #inside");
    try subject.assertExists("#tpl > #frag");
}

test "contract: Harness.fromHtml runs Node.compareDocumentPosition during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='outer'><span id='inside'>Inside</span></section><template id='tpl'><span id='frag'>Frag</span></template><div id='out'></div><script>const outer = document.getElementById('outer'); const inside = document.getElementById('inside'); const fragment = document.getElementById('tpl').content; const fragmentChild = fragment.querySelector('#frag'); document.getElementById('out').textContent = String(document.compareDocumentPosition(outer)) + ':' + String(outer.compareDocumentPosition(document)) + ':' + String(outer.compareDocumentPosition(inside)) + ':' + String(inside.compareDocumentPosition(outer)) + ':' + String(outer.compareDocumentPosition(fragmentChild)) + ':' + String(fragmentChild.compareDocumentPosition(outer)) + ':' + String(fragment.compareDocumentPosition(fragmentChild)) + ':' + String(fragmentChild.compareDocumentPosition(fragment)) + ':' + String(document.compareDocumentPosition(document));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "20:10:20:10:37:35:20:10:0");
    try subject.assertExists("#outer > #inside");
    try subject.assertExists("#tpl > #frag");
}

test "contract: Harness.fromHtml runs Node.isSameNode and Node.isEqualNode during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='out'></main><script>const left = document.createElement('div'); left.appendChild(document.createTextNode('Hello')); const right = document.createElement('div'); right.appendChild(document.createTextNode('Hello')); const fragLeft = document.createDocumentFragment(); fragLeft.appendChild(document.createTextNode('Hello')); const fragRight = document.createDocumentFragment(); fragRight.appendChild(document.createTextNode('Hello')); document.getElementById('out').textContent = String(document.isSameNode(document)) + ':' + String(document.isEqualNode(document)) + ':' + String(left.isSameNode(right)) + ':' + String(left.isEqualNode(right)) + ':' + String(fragLeft.isSameNode(fragRight)) + ':' + String(fragLeft.isEqualNode(fragRight));</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "true:true:false:true:false:true");
}

test "contract: Harness.fromHtml runs detached node construction during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='target'><span id='text-source'>Old</span><span id='comment-source'>Keep</span></section><div id='out'></div><script>const article = document.createElement('ARTICLE'); article.setAttribute('id', 'created'); article.textContent = 'Body'; const text = document.createTextNode('Replaced'); const comment = document.createComment('note'); const fragment = document.createDocumentFragment(); fragment.innerHTML = '<span id=\"fragment-child\">Fragment</span>'; const before = String(article.parentNode) + ':' + String(text.parentNode) + ':' + String(comment.parentNode) + ':' + String(fragment) + ':' + fragment.innerHTML + ':' + String(fragment.childNodes.length) + ':' + String(fragment.hasChildNodes()) + ':' + String(fragment.isConnected) + ':' + String(fragment.firstChild) + ':' + String(fragment.lastChild) + ':' + String(fragment.nextSibling) + ':' + String(fragment.previousSibling) + ':' + String(fragment.ownerDocument) + ':' + String(fragment.parentNode) + ':' + String(fragment.parentElement) + ':' + String(fragment.querySelector('#fragment-child')) + ':' + String(text) + ':' + String(comment) + ':' + text.textContent + ':' + comment.textContent; document.getElementById('target').appendChild(article); document.getElementById('text-source').childNodes.item(0).replaceWith(text); document.getElementById('comment-source').childNodes.item(0).replaceWith(comment); document.getElementById('out').textContent = before + '|' + document.getElementById('target').innerHTML + '|' + document.getElementById('text-source').innerHTML + '|' + document.getElementById('comment-source').innerHTML + '|' + String(document.querySelector('#created'));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "null:null:null:[object DocumentFragment]:<span id=\"fragment-child\">Fragment</span>:1:true:false:[object Element]:[object Element]:null:null:[object Document]:null:null:[object Element]:[object Node]:[object Node]:Replaced:note|<span id=\"text-source\">Replaced</span><span id=\"comment-source\"><!--note--></span><article id=\"created\">Body</article>|Replaced|<!--note-->|[object Element]",
    );
    try subject.assertExists("#target > #text-source");
    try subject.assertExists("#target > #comment-source");
    try subject.assertExists("#target > #created");
}

test "contract: Harness.fromHtml runs document.createElementNS during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg'); const gradient = document.createElementNS('http://www.w3.org/2000/svg', 'linearGradient'); const html = document.createElementNS('http://www.w3.org/1999/xhtml', 'DIV'); const fragment = document.createDocumentFragment(); svg.appendChild(gradient); document.getElementById('root').appendChild(svg); document.getElementById('root').appendChild(html); document.getElementById('out').textContent = svg.namespaceURI + ':' + gradient.namespaceURI + ':' + html.namespaceURI + ':' + String(fragment.namespaceURI) + ':' + svg.nodeName + ':' + gradient.nodeName + ':' + html.nodeName + ':' + svg.outerHTML + '|' + html.outerHTML;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "http://www.w3.org/2000/svg:http://www.w3.org/2000/svg:http://www.w3.org/1999/xhtml:null:svg:linearGradient:div:<svg><linearGradient></linearGradient></svg>|<div></div>",
    );
    try subject.assertExists("#root > svg");
    try subject.assertExists("#root > div");
}

test "contract: Harness.fromHtml runs document.createAttributeNS during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='out'></div><script>const namespaced = document.createAttributeNS('urn:test', 'svg:stroke'); namespaced.nodeValue = 'azure'; const plain = document.createAttributeNS(null, 'data-role'); plain.value = 'dialog'; document.getElementById('out').textContent = String(namespaced) + ':' + namespaced.name + ':' + String(namespaced.namespaceURI) + ':' + namespaced.localName + ':' + String(namespaced.prefix) + ':' + namespaced.nodeName + ':' + String(namespaced.nodeType) + ':' + namespaced.value + ':' + namespaced.data + ':' + namespaced.textContent + ':' + String(namespaced.ownerDocument) + ':' + String(namespaced.parentNode) + ':' + String(namespaced.parentElement) + ':' + String(namespaced.ownerElement) + ':' + String(plain.namespaceURI) + ':' + plain.name + ':' + plain.localName + ':' + String(plain.prefix) + ':' + plain.value;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object Attr]:svg:stroke:urn:test:stroke:svg:svg:stroke:2:azure:azure:azure:[object Document]:null:null:null:null:data-role:data-role:null:dialog",
    );
}

test "contract: Harness.fromHtml runs element.attributes during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='host' data-role='menu' aria-label='Label'></div><div id='out'></div><script>const attrs = document.getElementById('host').attributes; const keys = attrs.keys(); const values = attrs.values(); const entries = attrs.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); document.getElementById('out').textContent = String(attrs.length) + ':' + String(attrs) + ':' + String(firstKey.value) + ':' + String(firstValue.value) + ':' + firstValue.value.name + ':' + firstValue.value.value + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.name + ':' + firstEntry.value.value.value + ':' + String(attrs.getNamedItem('data-role')) + ':' + attrs.getNamedItem('data-role').value + ':' + String(attrs.getNamedItemNS(null, 'aria-label')) + ':' + attrs.getNamedItemNS(null, 'aria-label').value + ':' + String(attrs.item(99));</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "3:[object NamedNodeMap]:0:[object Attr]:id:host:0:id:host:[object Attr]:menu:[object Attr]:Label:null",
    );
}

test "contract: Harness.fromHtml runs element.attributes namespace lookups during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='host' data-role='menu'></div><div id='out'></div><script>const host = document.getElementById('host'); host.setAttributeNS('urn:test', 'svg:stroke', 'azure'); const attrs = host.attributes; document.getElementById('out').textContent = String(attrs.length) + ':' + String(attrs.getNamedItemNS('urn:test', 'stroke')) + ':' + attrs.getNamedItemNS('urn:test', 'stroke').namespaceURI + ':' + attrs.getNamedItemNS('urn:test', 'stroke').prefix + ':' + String(attrs.getNamedItemNS(null, 'stroke')) + ':' + String(attrs.item(2)) + ':' + attrs.item(2).name + ':' + attrs.item(2).prefix;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "3:[object Attr]:urn:test:svg:null:[object Attr]:svg:stroke:svg",
    );
}

test "contract: Harness.fromHtml runs element.attributes mutators during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='host' data-role='menu' aria-label='Label'></div><div id='out'></div><script>const host = document.getElementById('host'); const attrs = host.attributes; const replacement = document.createAttribute('data-role'); replacement.value = 'dialog'; const previous = attrs.setNamedItem(replacement); const namespaced = document.createAttributeNS('urn:test', 'svg:stroke'); namespaced.value = 'azure'; const nsPrevious = attrs.setNamedItemNS(namespaced); const before = String(previous) + ':' + previous.name + ':' + previous.value + ':' + String(previous.ownerElement) + ':' + String(replacement.ownerElement) + ':' + String(nsPrevious) + ':' + String(namespaced.ownerElement) + ':' + String(attrs.length); document.getElementById('out').textContent = before + ':'; attrs.forEach((attribute, index, list) => { document.getElementById('out').textContent += String(index) + ':' + attribute.name + ':' + attribute.value + ':' + String(list.length) + ';'; }, null); const removed = attrs.removeNamedItem('data-role'); const removedNS = attrs.removeNamedItemNS('urn:test', 'stroke'); document.getElementById('out').textContent += '|' + String(removed) + ':' + removed.value + ':' + String(removed.ownerElement) + ':' + String(removedNS) + ':' + removedNS.name + ':' + removedNS.prefix + ':' + String(removedNS.ownerElement) + ':' + String(attrs.length) + ':' + String(replacement.ownerElement) + ':' + String(namespaced.ownerElement) + ':' + String(host.getAttributeNode('data-role')) + ':' + String(host.getAttributeNodeNS('urn:test', 'stroke'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object Attr]:data-role:menu:null:[object Element]:null:[object Element]:4:0:id:host:4;1:data-role:dialog:4;2:aria-label:Label:4;3:svg:stroke:azure:4;|[object Attr]:dialog:null:[object Attr]:svg:stroke:svg:null:2:[object Element]:[object Element]:null:null",
    );
}

test "failure: Harness.fromHtml rejects element.attributes keys arguments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>document.documentElement.attributes.keys(1);</script>"),
    );
}

test "failure: Harness.fromHtml rejects element.attributes setNamedItem non-Attr argument" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>document.documentElement.attributes.setNamedItem('data-role');</script>"),
    );
}

test "failure: Harness.fromHtml rejects element.attributes setNamedItemNS owner mismatch" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='left'></div><div id='right'></div><script>const attr = document.createAttributeNS('urn:test', 'svg:stroke'); document.getElementById('right').attributes.setNamedItemNS(attr); document.getElementById('left').attributes.setNamedItemNS(attr);</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects element.attributes removeNamedItem missing name" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>document.documentElement.attributes.removeNamedItem('missing');</script>"),
    );
}

test "failure: Harness.fromHtml rejects element.attributes removeNamedItemNS missing name" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>document.documentElement.attributes.removeNamedItemNS('urn:test', 'missing');</script>"),
    );
}

test "failure: Harness.fromHtml rejects element.attributes forEach arguments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>document.documentElement.attributes.forEach(123);</script>"),
    );
}

test "contract: Harness.fromHtml runs document.createAttribute during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='out'></div><script>const attr = document.createAttribute('data-role'); attr.value = 'dialog'; document.getElementById('out').textContent = String(attr) + ':' + attr.name + ':' + String(attr.namespaceURI) + ':' + attr.localName + ':' + String(attr.prefix) + ':' + attr.nodeName + ':' + String(attr.nodeType) + ':' + attr.value + ':' + attr.data + ':' + attr.textContent + ':' + String(attr.ownerDocument) + ':' + String(attr.parentNode) + ':' + String(attr.parentElement) + ':' + String(attr.ownerElement);</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object Attr]:data-role:null:data-role:null:data-role:2:dialog:dialog:dialog:[object Document]:null:null:null",
    );
}

test "contract: Harness.fromHtml runs element attribute node APIs during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='host' data-role='menu'></div><div id='out'></div><script>const host = document.getElementById('host'); const created = document.createAttribute('data-state'); created.value = 'open'; const previous = host.setAttributeNode(created); const snapshot = host.getAttributeNode('data-state'); const removed = host.removeAttributeNode(created); const attached = host.getAttributeNode('data-role'); document.getElementById('out').textContent = String(previous) + ':' + String(snapshot) + ':' + snapshot.name + ':' + snapshot.value + ':' + String(snapshot.ownerElement) + ':' + String(created.ownerElement) + ':' + String(removed) + ':' + String(host.getAttributeNode('data-state')) + ':' + String(attached) + ':' + attached.value + ':' + String(attached.ownerElement);</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "null:[object Attr]:data-state:open:[object Element]:null:[object Attr]:null:[object Attr]:menu:[object Element]",
    );
}

test "contract: Harness.fromHtml runs element attribute node NS APIs during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='host'></div><div id='out'></div><script>const host = document.getElementById('host'); const created = document.createAttributeNS('urn:test', 'svg:stroke'); created.value = 'azure'; const previous = host.setAttributeNodeNS(created); const snapshot = host.getAttributeNodeNS('urn:test', 'stroke'); document.getElementById('out').textContent = String(previous) + ':' + String(snapshot) + ':' + snapshot.name + ':' + String(snapshot.namespaceURI) + ':' + snapshot.localName + ':' + String(snapshot.prefix) + ':' + snapshot.value + ':' + String(snapshot.ownerElement) + ':' + String(created.ownerElement);</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "null:[object Attr]:svg:stroke:urn:test:stroke:svg:azure:[object Element]:[object Element]",
    );
}

test "contract: Harness.fromHtml runs Node.nodeValue during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='out'></main><script>const element = document.getElementById('out'); const fragment = document.createDocumentFragment(); const text = document.createTextNode('Hello'); const comment = document.createComment('note'); element.nodeValue = 'ignored'; fragment.nodeValue = 'ignored'; document.nodeValue = 'ignored'; text.data = 'World'; comment.data = 'updated'; document.getElementById('out').textContent = String(document.nodeValue) + ':' + String(element.nodeValue) + ':' + String(fragment.nodeValue) + ':' + text.data + ':' + comment.data + ':' + text.nodeValue + ':' + comment.nodeValue + ':' + text.textContent + ':' + comment.textContent;</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "null:null:null:World:updated:World:updated:World:updated");
}

test "contract: Harness.fromHtml runs Text.splitText during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='host'>Hello</div></main><div id='out'></div><script>const text = document.getElementById('host').childNodes.item(0); const split = text.splitText(2); document.getElementById('out').textContent = text.data + ':' + split.data + ':' + text.nextSibling.data + ':' + document.getElementById('host').textContent + ':' + String(text.length) + ':' + String(split.parentNode) + ':' + String(document.getElementById('host').childNodes.length);</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "He:llo:llo:Hello:2:[object Element]:2");
    try subject.assertExists("#host");
}

test "contract: Harness.fromHtml runs CharacterData methods during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='host'>Hello</div><div id='note'><!--note--></div><div id='out'></div><script>const text = document.getElementById('host').childNodes.item(0); const comment = document.getElementById('note').childNodes.item(0); const substring = text.substringData(1, 3); text.appendData('!'); text.insertData(1, 'X'); text.deleteData(2, 2); text.replaceData(1, 2, 'Q'); comment.appendData('!'); comment.insertData(0, '['); comment.deleteData(1, 1); comment.replaceData(0, 1, '('); document.getElementById('out').textContent = substring + ':' + text.data + ':' + String(text.length) + ':' + comment.data + ':' + String(comment.length) + ':' + text.substringData(0, 2) + ':' + comment.substringData(0, 2);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "ell:HQo!:4:(ote!:5:HQ:(o");
}

test "contract: Harness.fromHtml runs innerHTML and outerHTML serialization during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section><div id='out'></div><script>document.getElementById('out').textContent = document.getElementById('target').innerHTML + '|' + document.getElementById('target').outerHTML + '|'; document.getElementById('target').innerHTML = '<span id=\"first\">One</span><span id=\"second\">Two</span>'; document.getElementById('out').textContent += document.getElementById('target').innerHTML + '|' + document.getElementById('target').outerHTML + '|' + String(document.querySelector('#old'));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "<button class=\"primary\" id=\"old\">Old</button>|<section id=\"target\"><button class=\"primary\" id=\"old\">Old</button></section>|<span id=\"first\">One</span><span id=\"second\">Two</span>|<section id=\"target\"><span id=\"first\">One</span><span id=\"second\">Two</span></section>|null",
    );
    try subject.assertExists("#target > #first");
    try subject.assertExists("#target > #second");
    try std.testing.expectError(error.AssertionFailed, subject.assertExists("#old"));
}

test "contract: Harness.fromHtml runs outerHTML replacement during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='target'><span id='old'>Old</span></section><div id='out'></div><script>document.getElementById('target').outerHTML = '<article id=\"replacement\"><em id=\"inner\">Inner</em></article>'; document.getElementById('out').textContent = String(document.querySelector('#target')) + '|' + document.getElementById('replacement').outerHTML + '|' + document.getElementById('inner').textContent;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "null|<article id=\"replacement\"><em id=\"inner\">Inner</em></article>|Inner");
    try subject.assertExists("#replacement");
    try subject.assertExists("#replacement > #inner");
    try std.testing.expectError(error.AssertionFailed, subject.assertExists("#target"));
}

test "contract: Harness.fromHtml decodes character references during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='attr' data-label='a&amp;b&copy;&reg;&nbsp;&#160;&#xA0;&AMP;&LT;&GT;&QUOT;&NBSP;&COPY;&REG'></div><div id='text'>a&amp;b&copy;&reg;&nbsp;&#160;&#xA0;&AMP;&LT;&GT;&QUOT;&NBSP;&COPY;&REG</div><div id='out'></div><script>const attr = document.getElementById('attr').getAttribute('data-label'); const text = document.getElementById('text').textContent; const html = document.getElementById('attr').outerHTML; document.getElementById('out').textContent = attr + '|' + text + '|' + html;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "a&b\xc2\xa9\xc2\xae\xc2\xa0\xc2\xa0\xc2\xa0&<>\"\xc2\xa0\xc2\xa9\xc2\xae|a&b\xc2\xa9\xc2\xae\xc2\xa0\xc2\xa0\xc2\xa0&<>\"\xc2\xa0\xc2\xa9\xc2\xae|<div data-label='a&amp;b\xc2\xa9\xc2\xae\xc2\xa0\xc2\xa0\xc2\xa0&amp;&lt;&gt;\"\xc2\xa0\xc2\xa9\xc2\xae' id=\"attr\"></div>",
    );
}

test "contract: Harness.fromHtml runs insertAdjacentElement and insertAdjacentText during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section></main><div id='out'></div><script>const target = document.getElementById('target'); const before = document.createElement('aside'); before.id = 'before'; const inserted = target.insertAdjacentElement('beforebegin', before); const text = target.insertAdjacentText('afterbegin', 'First'); target.insertAdjacentText('beforeend', 'Last'); const after = document.createElement('aside'); after.id = 'after'; target.insertAdjacentElement('afterend', after); document.getElementById('out').textContent = String(inserted) + ':' + String(text) + ':' + document.getElementById('root').innerHTML + ':' + document.getElementById('target').innerHTML;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object Element]:undefined:<aside id=\"before\"></aside><section id=\"target\">First<button class=\"primary\" id=\"old\">Old</button>Last</section><aside id=\"after\"></aside>:First<button class=\"primary\" id=\"old\">Old</button>Last",
    );
    try subject.assertExists("#before");
    try subject.assertExists("#after");
    try subject.assertExists("#target > #old");
}

test "contract: Harness.fromHtml runs namespace-aware serialization during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><svg id='icon' viewbox='0 0 10 10'><foreignobject id='foreign'><div id='html'>Text</div></foreignobject></svg><math id='formula' definitionurl='https://example.com'><mi id='symbol'>x</mi></math><div id='out'></div><script>document.getElementById('out').textContent = document.getElementById('icon').outerHTML + '|' + document.getElementById('formula').outerHTML;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "<svg id=\"icon\" viewBox=\"0 0 10 10\"><foreignObject id=\"foreign\"><div id=\"html\">Text</div></foreignObject></svg>|<math definitionURL=\"https://example.com\" id=\"formula\"><mi id=\"symbol\">x</mi></math>",
    );
    try subject.assertExists("#foreign");
    try subject.assertExists("#symbol");
}

test "failure: Harness.fromHtml rejects unsupported insertAdjacentHTML positions" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><section id='target'></section></main><script>document.getElementById('target').insertAdjacentHTML('middle', '<span id=\"bad\">Bad</span>');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects template.content firstElementChild assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<template id='tpl'><span id='first'>First</span></template><script>document.getElementById('tpl').content.firstElementChild = null;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects template.content.outerHTML access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<template id='tpl'></template><script>document.getElementById('tpl').content.outerHTML;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects template.content querySelector arity mismatches" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<template id='tpl'><span id='inner'>Inner</span></template><script>document.getElementById('tpl').content.querySelector();</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects HTMLTemplateElement.shadowRootMode on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').shadowRootMode = 'open';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects insertAdjacentHTML on void elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><img id='image'></main><script>document.getElementById('image').insertAdjacentHTML('beforeend', '<span id=\"bad\">Bad</span>');</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><img id='image'></main><script>document.getElementById('image').insertAdjacentHTML('beforeend', '<span id=\"bad\">Bad</span>');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Node.compareDocumentPosition arity and type mismatches" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><section id='outer'><span id='inside'>Inside</span></section><script>document.getElementById('outer').compareDocumentPosition('nope');</script></main>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><section id='outer'><span id='inside'>Inside</span></section><script>document.getElementById('outer').compareDocumentPosition('nope');</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Node.isSameNode and Node.isEqualNode arity and type mismatches" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><section id='outer'><span id='inside'>Inside</span></section><script>document.isEqualNode('unexpected');</script></main>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><section id='outer'><span id='inside'>Inside</span></section><script>document.isEqualNode('unexpected');</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects tree mutation replaceWith type mismatches" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><section id='target'><button id='old'>Old</button></section><div id='replacement'>Replacement</div><script>document.getElementById('old').replaceWith('Replacement');</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Node.removeChild arity and parent mismatches" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><section id='target'><button id='first'>First</button></section><button id='other'>Other</button><script>document.getElementById('target').removeChild(document.getElementById('other'));</script></main>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><section id='target'><button id='first'>First</button></section><button id='other'>Other</button><script>document.getElementById('target').removeChild(document.getElementById('other'));</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects unsupported script query selectors" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.querySelector('main::before');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects document.focus and document.blur calls" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='out'></div><script>document.focus();</script></main>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='out'></div><script>document.focus();</script></main>",
        ),
    );
}

test "contract: Harness.fromHtml resolves :has pseudo-class selectors during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='first' class='child'>First</section><section id='child' class='child'><div id='grandchild' class='grandchild'>Grand</div></section></main><div id='out'></div><script>const docMatch = document.querySelector('main:has(#missing, #child)'); const directMatch = document.querySelector('main:has(> .child)'); const root = document.getElementById('root'); const section = document.getElementById('child'); const nested = document.querySelector('main:has(section .grandchild)'); const closest = section.closest('main:has(> .child)'); document.getElementById('out').textContent = docMatch.getAttribute('id') + ':' + directMatch.getAttribute('id') + ':' + String(root.matches('main:has(> .child)')) + ':' + String(section.matches(':has(.grandchild)')) + ':' + closest.getAttribute('id') + ':' + nested.getAttribute('id');</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "root:root:true:true:root:root");
}

test "contract: Harness.fromHtml exposes anchor and area target reflection and click observation" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><a id='anchor' href='https://example.test/next' target='_blank'>Anchor</a><map name='map'><area id='area' target='popup' href='https://example.test/files/diagram.png'></map><div id='out'></div><script>const anchor = document.getElementById('anchor'); const area = document.querySelector('#area'); const before = String(anchor.target) + ':' + String(area.target); anchor.target = 'reports'; area.target = 'diagram'; document.getElementById('out').textContent = before + '|' + anchor.target + ':' + area.target;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "_blank:popup|reports:diagram");
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().open().calls().len);

    try subject.click("#anchor");
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().open().calls().len);
    try std.testing.expectEqualStrings(
        "https://example.test/next",
        subject.mocksMut().open().calls()[0].url.?,
    );
    try std.testing.expectEqualStrings(
        "reports",
        subject.mocksMut().open().calls()[0].target.?,
    );

    try subject.click("#area");
    try std.testing.expectEqual(@as(usize, 2), subject.mocksMut().open().calls().len);
    try std.testing.expectEqualStrings(
        "https://example.test/files/diagram.png",
        subject.mocksMut().open().calls()[1].url.?,
    );
    try std.testing.expectEqualStrings(
        "diagram",
        subject.mocksMut().open().calls()[1].target.?,
    );
}

test "contract: Harness.fromHtml exposes anchor and area href reflection" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><a id='anchor' href='https://example.test/next'>Anchor</a><map name='map'><area id='area' href='https://example.test/files/diagram.png'></map><div id='out'></div><script>const anchor = document.getElementById('anchor'); const area = document.querySelector('#area'); const before = String(anchor.href) + ':' + String(area.href); anchor.href = 'https://example.test/other'; area.href = 'https://example.test/files/diagram-2.png'; document.getElementById('out').textContent = before + '|' + anchor.href + ':' + area.href;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "https://example.test/next:https://example.test/files/diagram.png|https://example.test/other:https://example.test/files/diagram-2.png",
    );

    try subject.click("#anchor");
    try std.testing.expectEqualStrings(
        "https://example.test/other",
        subject.mocksMut().location().currentUrl().?,
    );

    try subject.click("#area");
    try std.testing.expectEqualStrings(
        "https://example.test/files/diagram-2.png",
        subject.mocksMut().location().currentUrl().?,
    );
}

test "contract: Harness.fromHtml exposes anchor and area rel reflection" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><a id='anchor' rel='noopener noreferrer noopener' href='https://example.test/next'>Anchor</a><map name='map'><area id='area' rel='preload preload' href='https://example.test/files/diagram.png'></map><div id='out'></div><script>const anchor = document.getElementById('anchor'); const area = document.querySelector('#area'); const before = String(anchor.rel) + ':' + String(anchor.relList.length) + ':' + String(anchor.relList.contains('noopener')) + ':' + String(anchor.relList.supports('noopener')) + ':' + String(anchor.relList.supports('bogus')) + ':' + String(area.rel) + ':' + String(area.relList.length) + ':' + String(area.relList.contains('preload')) + ':' + String(area.relList.supports('preload')); anchor.rel = 'noreferrer'; area.relList.value = 'preload modulepreload'; document.getElementById('out').textContent = before + '|' + anchor.rel + ':' + anchor.relList.value + ':' + area.rel + ':' + area.relList.value;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "noopener noreferrer noopener:2:true:true:false:preload preload:1:true:true|noreferrer:noreferrer:preload modulepreload:preload modulepreload",
    );
}

test "contract: Harness.fromHtml exposes anchor and area ping reflection" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><a id='anchor' ping='https://example.test/ping-a https://example.test/ping-b' href='https://example.test/next'>Anchor</a><map name='map'><area id='area' ping='https://example.test/ping-c' href='https://example.test/files/diagram.png'></map><div id='out'></div><script>const anchor = document.getElementById('anchor'); const area = document.querySelector('#area'); const before = String(anchor.ping) + ':' + String(area.ping); anchor.ping = 'https://example.test/ping-d'; area.ping = 'https://example.test/ping-e https://example.test/ping-f'; document.getElementById('out').textContent = before + '|' + anchor.ping + ':' + area.ping + ':' + anchor.getAttribute('ping') + ':' + area.getAttribute('ping');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "https://example.test/ping-a https://example.test/ping-b:https://example.test/ping-c|https://example.test/ping-d:https://example.test/ping-e https://example.test/ping-f:https://example.test/ping-d:https://example.test/ping-e https://example.test/ping-f",
    );
}

test "contract: Harness.fromHtml exposes anchor text reflection" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><a id='anchor' href='https://example.test/next'>Anchor</a><div id='out'></div><script>const anchor = document.getElementById('anchor'); const before = anchor.text + ':' + anchor.textContent; anchor.text = 'Updated'; document.getElementById('out').textContent = before + '|' + anchor.text + ':' + anchor.textContent + ':' + anchor.innerHTML + ':' + anchor.outerHTML;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "Anchor:Anchor|Updated:Updated:Updated:<a href=\"https://example.test/next\" id=\"anchor\">Updated</a>",
    );
}

test "contract: Harness.fromHtml exposes area alt coords and shape reflection" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><map name='map'><area id='area' alt='Site map entry' coords='0,0,10,10' shape='rect' href='https://example.test/files/diagram.png'></map><div id='out'></div><script>const area = document.querySelector('#area'); const before = String(area.alt) + ':' + String(area.coords) + ':' + String(area.shape); area.alt = 'Updated'; area.coords = '1,2,3,4'; area.shape = 'circle'; document.getElementById('out').textContent = before + '|' + area.alt + ':' + area.coords + ':' + area.shape + ':' + area.getAttribute('alt') + ':' + area.getAttribute('coords') + ':' + area.getAttribute('shape');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "Site map entry:0,0,10,10:rect|Updated:1,2,3,4:circle:Updated:1,2,3,4:circle",
    );
}

test "contract: Harness.fromHtml exposes area noHref reflection and click suppression" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><map name='map'><area id='area' nohref href='https://example.test/files/diagram.png'></map><div id='out'></div><script>const area = document.querySelector('#area'); const before = String(area.noHref) + ':' + String(area.getAttribute('nohref')); area.noHref = false; const during = String(area.noHref) + ':' + String(area.getAttribute('nohref')); area.noHref = true; document.getElementById('out').textContent = before + '|' + during + '|' + String(area.noHref) + ':' + String(area.getAttribute('nohref'));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "true:|false:null|true:",
    );
    try subject.click("#area");
    try std.testing.expectEqualStrings(
        "https://app.local/",
        subject.mocksMut().location().currentUrl().?,
    );
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().open().calls().len);
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().downloads().artifacts().len);
}

test "contract: Harness.fromHtml exposes anchor and area hyperlink metadata reflection" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><a id='anchor' hreflang='en' referrerpolicy='no-referrer' type='text/html' href='https://example.test/next'>Anchor</a><map name='map'><area id='area' hreflang='de' referrerpolicy='origin' type='text/html' href='https://example.test/files/diagram.png'></map><div id='out'></div><script>const anchor = document.getElementById('anchor'); const area = document.querySelector('#area'); const before = String(anchor.hreflang) + ':' + String(anchor.referrerPolicy) + ':' + String(anchor.type) + ':' + String(area.hreflang) + ':' + String(area.referrerPolicy) + ':' + String(area.type); anchor.hreflang = 'fr'; anchor.referrerPolicy = 'same-origin'; anchor.type = 'application/xhtml+xml'; area.hreflang = 'it'; area.referrerPolicy = 'strict-origin'; area.type = 'text/plain'; document.getElementById('out').textContent = before + '|' + anchor.hreflang + ':' + anchor.referrerPolicy + ':' + anchor.type + ':' + area.hreflang + ':' + area.referrerPolicy + ':' + area.type + ':' + anchor.getAttribute('hreflang') + ':' + anchor.getAttribute('referrerpolicy') + ':' + anchor.getAttribute('type') + ':' + area.getAttribute('hreflang') + ':' + area.getAttribute('referrerpolicy') + ':' + area.getAttribute('type');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "en:no-referrer:text/html:de:origin:text/html|fr:same-origin:application/xhtml+xml:it:strict-origin:text/plain:fr:same-origin:application/xhtml+xml:it:strict-origin:text/plain",
    );
}

test "contract: Harness.fromHtml exposes anchor and area URL decomposition" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><a id='anchor' href='https://user:pass@example.test:8443/next/path?x=1#frag'>Anchor</a><map name='map'><area id='area' href='https://user:pass@example.test:8443/files/diagram.png?y=2#section'></map><link id='link' href='https://user:pass@example.test:8443/assets/style.css?z=3#sheet'><div id='out'></div><script>const anchor = document.getElementById('anchor'); const area = document.querySelector('#area'); const link = document.getElementById('link'); const before = String(anchor.origin) + '|' + anchor.protocol + '|' + anchor.host + '|' + anchor.hostname + '|' + anchor.port + '|' + anchor.username + '|' + anchor.password + '|' + anchor.pathname + '|' + anchor.search + '|' + anchor.hash + '||' + String(area.origin) + '|' + area.protocol + '|' + area.host + '|' + area.hostname + '|' + area.port + '|' + area.username + '|' + area.password + '|' + area.pathname + '|' + area.search + '|' + area.hash + '||' + String(link.origin) + '|' + link.protocol + '|' + link.host + '|' + link.hostname + '|' + link.port + '|' + link.username + '|' + link.password + '|' + link.pathname + '|' + link.search + '|' + link.hash; document.getElementById('out').textContent = before;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "https://example.test:8443|https:|example.test:8443|example.test|8443|user|pass|/next/path|?x=1|#frag||https://example.test:8443|https:|example.test:8443|example.test|8443|user|pass|/files/diagram.png|?y=2|#section||https://example.test:8443|https:|example.test:8443|example.test|8443|user|pass|/assets/style.css|?z=3|#sheet",
    );
}

test "failure: Harness.fromHtml rejects non-document forms access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><form name='signup'>Signup</form></main><script>document.getElementById('root').forms.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects HTMLProgressElement min assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><progress id='progress'></progress><script>document.getElementById('progress').min = 1;</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects HTMLMeterElement position access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><meter id='meter' value='1'></meter><script>document.getElementById('meter').position;</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects input.list assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><input id='search' list='suggestions'><datalist id='suggestions'></datalist></main><script>document.getElementById('search').list = null;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects input.files forEach with a non-function callback" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><input id='search' type='file' value='report.csv'></main><script>document.getElementById('search').files.forEach(123);</script>",
        ),
    );
}

test "contract: Harness.fromHtml serializes mixed-quote attribute values during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='target'></div><div id='out'></div><script>document.getElementById('target').setAttribute('data-label', 'a\\'b\"c&d<e>'); document.getElementById('out').textContent = document.getElementById('target').outerHTML;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "<div data-label=\"a'b&quot;c&amp;d&lt;e&gt;\" id=\"target\"></div>",
    );
}

test "failure: Harness.fromHtml rejects tree mutation ancestor cycles" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><section id='child'><span id='grandchild'>x</span></section></main><script>document.getElementById('child').appendChild(document.getElementById('root'));</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects getAttributeNames arity and non-node mismatches" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').getAttributeNames('extra');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects window.getComputedStyle writes and pseudo-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='box' style='color: red;'></div><script>window.getComputedStyle(document.getElementById('box')).setProperty('color', 'blue');</script></main>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='box' style='color: red;'></div><script>window.getComputedStyle(document.getElementById('box')).setProperty('color', 'blue');</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.getBoundingClientRect writes" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='box' style='left: 10px; top: 20px; width: 30px; height: 40px;'></div><script>document.getElementById('box').getBoundingClientRect().width = 1;</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.getClientRects writes" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='box' style='left: 10px; top: 20px; width: 30px; height: 40px;'></div><script>document.getElementById('box').getClientRects().length = 2;</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects whitespace classList.replace tokens in inline scripts" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<button id='button' class='base'></button><script>document.getElementById('button').classList.replace('base', 'bad token');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects token-list item with a non-numeric index" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<button id='button' class='base primary'></button><script>document.getElementById('button').classList.item('bad');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects token-list forEach with a non-function callback" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<button id='button' class='base primary'></button><script>document.getElementById('button').classList.forEach(123);</script>",
        ),
    );
}

test "failure: Harness.assertExists rejects malformed selectors" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<main id='app'><span>Hello</span></main>");
    defer subject.deinit();

    try std.testing.expectError(error.InvalidSelector, subject.assertExists("main::before"));
    try std.testing.expectError(error.InvalidSelector, subject.assertExists("[data-state"));
    try std.testing.expectError(error.InvalidSelector, subject.assertExists("main*"));
}

test "failure: Harness.assertExists rejects malformed nth pseudo-class selectors" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<main id='app'><button id='first'>First</button></main>");
    defer subject.deinit();

    try std.testing.expectError(error.InvalidSelector, subject.assertExists("button:nth-last-of-type(2 of )"));
}

test "failure: Harness.assertExists rejects malformed :has selectors" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<main id='app'><button id='first'>First</button></main>");
    defer subject.deinit();

    try std.testing.expectError(error.InvalidSelector, subject.assertExists("button:has(>)"));
}

test "failure: Harness.assertExists rejects malformed state pseudo-class selectors" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<main id='app'><button id='button'>Button</button></main>");
    defer subject.deinit();

    try std.testing.expectError(error.InvalidSelector, subject.assertExists("button:default()"));
    try std.testing.expectError(error.InvalidSelector, subject.assertExists("button:indeterminate()"));
    try std.testing.expectError(error.InvalidSelector, subject.assertExists("button:read-only()"));
    try std.testing.expectError(error.InvalidSelector, subject.assertExists("button:read-write()"));
    try std.testing.expectError(error.InvalidSelector, subject.assertExists("button:valid()"));
    try std.testing.expectError(error.InvalidSelector, subject.assertExists("button:invalid()"));
    try std.testing.expectError(error.InvalidSelector, subject.assertExists("button:in-range()"));
    try std.testing.expectError(error.InvalidSelector, subject.assertExists("button:out-of-range()"));
    try std.testing.expectError(error.InvalidSelector, subject.assertExists("button:defined()"));
}

test "failure: Harness.assertExists reports missing matches" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<main id='app'><span>Hello</span></main>");
    defer subject.deinit();

    try std.testing.expectError(error.AssertionFailed, subject.assertExists("#missing"));
}

test "contract: Harness.typeText updates value and dispatches input listeners" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<input id='name' class='field'><div id='out'></div><script>document.getElementById('name').addEventListener('input', () => { document.getElementById('out').textContent = document.getElementById('name').value; });</script>",
    );
    defer subject.deinit();

    try subject.typeText("input.field", "Alice");
    try subject.assertValue("#name", "Alice");
    try subject.assertValue("#out", "Alice");
}

test "contract: Harness.click dispatches capture and bubble listeners in order" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='parent'><div id='child'></div></div><div id='out'></div><script>window.addEventListener('click', () => { document.getElementById('out').textContent += ':window-bubble'; }); window.addEventListener('click', () => { document.getElementById('out').textContent += 'window-capture:'; }, true); document.addEventListener('click', () => { document.getElementById('out').textContent += 'document-capture:'; }, true); document.addEventListener('click', () => { document.getElementById('out').textContent += ':document-bubble'; }); document.getElementById('parent').addEventListener('click', () => { document.getElementById('out').textContent += 'parent-capture:'; }, true); document.getElementById('parent').addEventListener('click', () => { document.getElementById('out').textContent += ':parent-bubble'; }); document.getElementById('child').addEventListener('click', () => { document.getElementById('out').textContent += 'target'; });</script>",
    );
    defer subject.deinit();

    try subject.click("#parent > #child");
    try subject.assertValue(
        "#out",
        "window-capture:document-capture:parent-capture:target:parent-bubble:document-bubble:window-bubble",
    );
}

test "contract: preventDefault cancels click default action" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<input id='agree' type='checkbox'><div id='out'></div><script>document.getElementById('agree').addEventListener('click', (event) => { event.preventDefault(); }); document.getElementById('agree').addEventListener('change', () => { document.getElementById('out').textContent = String(document.getElementById('agree').checked); });</script>",
    );
    defer subject.deinit();

    try subject.click("#agree");
    try subject.assertChecked("#agree", false);
    try subject.assertValue("#out", "");
}

test "contract: Harness.click on a submit button dispatches form submit default action" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<form id='profile'><input id='name'><button id='submit' type='submit'>Save</button></form><div id='out'></div><script>document.getElementById('profile').addEventListener('submit', (event) => { document.getElementById('out').textContent = document.getElementById('name').value + ':' + event.submitter.id; });</script>",
    );
    defer subject.deinit();

    try subject.typeText("#name", "Alice");
    try subject.click("#submit");
    try subject.assertValue("#out", "Alice:submit");
}

test "failure: Harness.fromHtml rejects requestSubmit with a non-submit control" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><form id='form'><button id='button' type='button'></button></form><script>document.getElementById('form').requestSubmit(document.getElementById('button'));</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects FormData construction from a non-form element" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='not-form'></div><script>new FormData(document.getElementById('not-form'));</script></main>",
        ),
    );
}

test "contract: Harness.click on a reset button dispatches form reset default action" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<form id='profile'><input id='name' value='Ada'><button id='reset' type='reset'>Reset</button></form><div id='out'></div><script>document.getElementById('profile').addEventListener('reset', () => { document.getElementById('out').textContent = document.getElementById('name').value; });</script>",
    );
    defer subject.deinit();

    try subject.typeText("#name", "Alice");
    try subject.click("#reset");
    try subject.assertValue("#out", "Alice");
}

test "contract: Harness.click on anchors navigates and captures downloads" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://app.local/start",
        "<main id='root'><a id='nav' href='https://example.test/next'>Go</a><a id='download' download='report.csv' href='https://example.test/files/report.csv'>Download</a></main>",
    );
    defer subject.deinit();

    try subject.click("#nav");
    try std.testing.expectEqualStrings(
        "https://example.test/next",
        subject.mocksMut().location().currentUrl().?,
    );

    try subject.click("#download");
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().downloads().artifacts().len);
    try std.testing.expectEqualStrings(
        "report.csv",
        subject.mocksMut().downloads().artifacts()[0].file_name,
    );
    try std.testing.expectEqualStrings(
        "https://example.test/files/report.csv",
        subject.mocksMut().downloads().artifacts()[0].bytes,
    );
    try std.testing.expectEqualStrings(
        "https://example.test/next",
        subject.mocksMut().location().currentUrl().?,
    );
}

test "contract: Harness.fromHtml exposes element.outerText as a text replacement alias" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='panel'><span>One</span><span>Two</span></section></main><div id='out'></div><script>const panel = document.getElementById('panel'); const before = panel.outerText; panel.outerText = 'Reset'; document.getElementById('out').textContent = before + ':' + document.getElementById('root').textContent + ':' + String(document.getElementById('panel'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "OneTwo:Reset:null");
}

test "contract: Harness.fromHtml exposes node traversal helpers" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<!--pre--><main id='root'><span id='first'>One</span><span id='second'>Two</span><!--tail--></main><div id='after'></div><div id='out'></div><script>const doc = document; const root = document.getElementById('root'); const first = document.getElementById('first'); const second = document.getElementById('second'); const tail = root.lastChild; document.getElementById('out').textContent = String(doc.isConnected) + ':' + String(doc.hasChildNodes()) + ':' + String(doc.firstChild) + ':' + String(doc.lastChild) + ':' + String(doc.nextSibling) + ':' + String(doc.previousSibling) + ':' + String(root.isConnected) + ':' + String(root.hasChildNodes()) + ':' + String(root.firstChild) + ':' + String(root.lastChild) + ':' + String(root.nextSibling) + ':' + String(root.previousSibling) + ':' + String(first.nextSibling) + ':' + String(first.previousSibling) + ':' + String(first.nextElementSibling) + ':' + String(second.nextSibling) + ':' + String(second.previousSibling) + ':' + String(second.previousElementSibling) + ':' + String(tail.previousSibling) + ':' + String(tail.nextSibling);</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "true:true:[object Node]:[object Element]:null:null:true:true:[object Element]:[object Node]:[object Element]:[object Node]:[object Element]:null:[object Element]:[object Node]:[object Element]:[object Element]:[object Element]:null",
    );
}

test "contract: Harness.fromHtmlWithUrl resolves document.location getter/setter and window alias" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>const before = document.location; document.location = 'https://example.test:8443/next'; const after = window.location; document.getElementById('out').textContent = before + ':' + after;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "https://example.test:8443/start?x#old:https://example.test:8443/next",
    );
}

test "contract: Harness.fromHtmlWithUrl exposes location, URL, documentURI, and window.location aliases" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>const beforeLocation = document.location; const beforeUrl = document.URL; const beforeDocumentUri = document.documentURI; const beforeWindowLocation = window.location; document.getElementById('out').textContent = beforeLocation + ':' + beforeUrl + ':' + beforeDocumentUri + ':' + beforeWindowLocation;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "https://example.test:8443/start?x#old:https://example.test:8443/start?x#old:https://example.test:8443/start?x#old:https://example.test:8443/start?x#old",
    );
}

test "contract: Harness.fromHtmlWithUrl exposes Location href and navigation methods" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>const location = window.location; const before = location.href; location.assign('https://example.test:8443/assign'); const afterAssign = location.href; location.href = 'https://example.test:8443/href'; const afterHref = location.href; location.replace('https://example.test:8443/replace'); const afterReplace = location.href; location.reload(); const afterReload = location.href; document.getElementById('out').textContent = before + ':' + afterAssign + ':' + afterHref + ':' + afterReplace + ':' + afterReload;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "https://example.test:8443/start?x#old:https://example.test:8443/assign:https://example.test:8443/href:https://example.test:8443/replace:https://example.test:8443/replace",
    );
    try std.testing.expectEqualStrings(
        "https://example.test:8443/replace",
        subject.mocksMut().location().currentUrl().?,
    );
    try std.testing.expectEqual(@as(usize, 4), subject.mocksMut().location().navigations().len);
    try std.testing.expectEqualStrings("https://example.test:8443/assign", subject.mocksMut().location().navigations()[0]);
    try std.testing.expectEqualStrings("https://example.test:8443/href", subject.mocksMut().location().navigations()[1]);
    try std.testing.expectEqualStrings("https://example.test:8443/replace", subject.mocksMut().location().navigations()[2]);
    try std.testing.expectEqualStrings("https://example.test:8443/replace", subject.mocksMut().location().navigations()[3]);
}

test "contract: Harness.fromHtmlWithUrl exposes Location.hash getter and setter" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>const location = document.location; const before = location.hash; location.hash = 'next-section'; const after = window.location.hash; document.getElementById('out').textContent = before + ':' + after + ':' + location.href;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "#old:#next-section:https://example.test:8443/start?x#next-section",
    );
    try std.testing.expectEqualStrings(
        "https://example.test:8443/start?x#next-section",
        subject.mocksMut().location().currentUrl().?,
    );
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().location().navigations().len);
    try std.testing.expectEqualStrings(
        "https://example.test:8443/start?x#next-section",
        subject.mocksMut().location().navigations()[0],
    );
}

test "contract: Harness.fromHtmlWithUrl dispatches hashchange listeners and onhashchange handlers" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>window.addEventListener('hashchange', () => { document.getElementById('out').textContent += 'listener:' + window.location.hash; }); window.onhashchange = () => { document.getElementById('out').textContent += '|property:' + window.location.hash; }; window.location.hash = 'next-section';</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "listener:#next-section|property:#next-section");
    try std.testing.expectEqualStrings(
        "https://example.test:8443/start?x#next-section",
        subject.mocksMut().location().currentUrl().?,
    );
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().location().navigations().len);
}

test "contract: Harness.click dispatches pagehide and pageshow handlers on navigation" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='out'></main><button id='nav'>Go</button><script>window.addEventListener('pagehide', () => { document.getElementById('out').textContent += 'hide|'; }); window.onpagehide = () => { document.getElementById('out').textContent += 'property-hide|'; }; window.addEventListener('pageshow', () => { document.getElementById('out').textContent += 'show|'; }); window.onpageshow = () => { document.getElementById('out').textContent += 'property-show|'; }; document.getElementById('nav').addEventListener('click', () => { document.getElementById('out').textContent = ''; document.location = 'https://example.test:8443/next'; });</script>",
    );
    defer subject.deinit();

    try subject.click("#nav");
    try subject.assertValue("#out", "hide|property-hide|show|property-show|");
    try std.testing.expectEqualStrings(
        "https://example.test:8443/next",
        subject.mocksMut().location().currentUrl().?,
    );
}

test "contract: Harness.click dispatches beforeunload, pagehide, unload, and pageshow handlers on navigation" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='out'></main><button id='nav'>Go</button><script>window.addEventListener('beforeunload', () => { document.getElementById('out').textContent += 'before|'; }); window.onbeforeunload = () => { document.getElementById('out').textContent += 'property-before|'; }; window.addEventListener('pagehide', () => { document.getElementById('out').textContent += 'hide|'; }); window.onpagehide = () => { document.getElementById('out').textContent += 'property-hide|'; }; window.addEventListener('unload', () => { document.getElementById('out').textContent += 'unload|'; }); window.onunload = () => { document.getElementById('out').textContent += 'property-unload|'; }; window.addEventListener('pageshow', () => { document.getElementById('out').textContent += 'show|'; }); window.onpageshow = () => { document.getElementById('out').textContent += 'property-show|'; }; document.getElementById('nav').addEventListener('click', () => { document.getElementById('out').textContent = ''; document.location = 'https://example.test:8443/next'; });</script>",
    );
    defer subject.deinit();

    try subject.click("#nav");
    try subject.assertValue("#out", "before|property-before|hide|property-hide|unload|property-unload|show|property-show|");
    try std.testing.expectEqualStrings(
        "https://example.test:8443/next",
        subject.mocksMut().location().currentUrl().?,
    );
}

test "contract: Harness.fromHtmlWithUrl dispatches popstate listeners and onpopstate handlers" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>window.addEventListener('popstate', () => { document.getElementById('out').textContent += 'listener:' + window.history.state + '|'; }); window.onpopstate = () => { document.getElementById('out').textContent += 'property:' + window.history.state; }; window.history.pushState('seed', '', 'https://example.test:8443/seed'); window.history.pushState('next', '', 'https://example.test:8443/next'); window.history.back();</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "listener:seed|property:seed");
    try std.testing.expectEqualStrings(
        "https://example.test:8443/seed",
        subject.mocksMut().location().currentUrl().?,
    );
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().location().navigations().len);
    try std.testing.expectEqualStrings(
        "https://example.test:8443/seed",
        subject.mocksMut().location().navigations()[0],
    );
}

test "contract: Harness.fromHtmlWithUrl exposes Location.search getter and setter" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>const location = document.location; const before = location.search; location.search = 'copy'; const after = window.location.search; document.getElementById('out').textContent = before + ':' + after + ':' + location.href;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "?x:?copy:https://example.test:8443/start?copy#old",
    );
    try std.testing.expectEqualStrings(
        "https://example.test:8443/start?copy#old",
        subject.mocksMut().location().currentUrl().?,
    );
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().location().navigations().len);
    try std.testing.expectEqualStrings(
        "https://example.test:8443/start?copy#old",
        subject.mocksMut().location().navigations()[0],
    );
}

test "contract: Harness.click clears window.onhashchange when assigned null" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><button id='toggle'>Toggle</button><script>document.getElementById('toggle').addEventListener('click', () => { window.onhashchange = () => { document.getElementById('out').textContent += '|property:' + window.location.hash; }; window.location.hash = 'first'; window.onhashchange = null; window.location.hash = 'second'; });</script>",
    );
    defer subject.deinit();

    try subject.click("#toggle");
    try subject.assertValue("#out", "|property:#first");
    try std.testing.expectEqualStrings(
        "https://example.test:8443/start?x#second",
        subject.mocksMut().location().currentUrl().?,
    );
    try std.testing.expectEqual(@as(usize, 2), subject.mocksMut().location().navigations().len);
}

test "failure: Harness.fromHtml rejects non-callable window.onhashchange assignments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>window.onhashchange = 123;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-callable window.onpopstate assignments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>window.onpopstate = 123;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-callable window.onfocus assignments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>window.onfocus = 123;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-callable window.onblur assignments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>window.onblur = 123;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-callable window.onload assignments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>window.onload = 123;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-callable window.onpageshow assignments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>window.onpageshow = 123;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-callable window.onpagehide assignments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>window.onpagehide = 123;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-callable window.onbeforeunload assignments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>window.onbeforeunload = 123;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-callable window.onunload assignments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>window.onunload = 123;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-callable window.onstorage assignments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>window.onstorage = 123;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-callable window.onscroll assignments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>window.onscroll = 123;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-callable document.onscroll assignments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>document.onscroll = 123;</script>",
        ),
    );
}

test "contract: Harness.fromHtmlWithUrl exposes window scroll aliases and resets them on navigation" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>const before = String(window.scrollX) + ':' + String(window.scrollY) + ':' + String(window.pageXOffset) + ':' + String(window.pageYOffset); window.scrollTo(10, 20); window.scrollBy(-3, 5); const afterScroll = String(window.scrollX) + ':' + String(window.scrollY) + ':' + String(window.pageXOffset) + ':' + String(window.pageYOffset); document.location = 'https://example.test:8443/next'; const afterNavigation = String(window.scrollX) + ':' + String(window.scrollY) + ':' + String(window.pageXOffset) + ':' + String(window.pageYOffset) + ':' + window.location.href; document.getElementById('out').textContent = before + '|' + afterScroll + '|' + afterNavigation;</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "0:0:0:0|7:25:7:25|0:0:0:0:https://example.test:8443/next");
    try std.testing.expectEqualStrings(
        "https://example.test:8443/next",
        subject.mocksMut().location().currentUrl().?,
    );
}

test "contract: Harness.fromHtmlWithUrl exposes window.navigator aliases" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<embed id='first-embed'><embed name='second-embed'><main id='out'></main><script>const navigator = window.navigator; document.getElementById('out').textContent = String(navigator) + ':' + navigator.userAgent + ':' + navigator.appCodeName + ':' + navigator.appName + ':' + navigator.appVersion + ':' + navigator.product + ':' + navigator.productSub + ':' + navigator.vendor + ':[' + navigator.vendorSub + ']:' + String(navigator.pdfViewerEnabled) + ':' + navigator.doNotTrack + ':' + String(navigator.javaEnabled()) + ':' + String(navigator.plugins) + ':' + String(navigator.plugins.length) + ':' + navigator.platform + ':' + navigator.language + ':' + String(navigator.cookieEnabled) + ':' + String(navigator.onLine) + ':' + String(navigator.webdriver) + ':' + String(navigator.hardwareConcurrency) + ':' + String(navigator.maxTouchPoints);</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object Navigator]:browser-tester-next:browser-tester-next:browser-tester-next:browser-tester-next:browser-tester-next:browser-tester-next:browser-tester-next:[]:false:unspecified:false:[object PluginArray]:2:unknown:en-US:true:true:false:8:0",
    );
}

test "contract: Harness.fromHtmlWithUrl exposes window.navigator.plugins.refresh" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<embed><main id='out'></main><script>const plugins = window.navigator.plugins; document.getElementById('out').textContent = String(plugins.refresh()) + ':' + String(plugins.length) + ':' + String(plugins);</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "undefined:1:[object PluginArray]");
}

test "contract: Harness.fromHtmlWithUrl exposes window.navigator.plugins collection helpers" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<embed id='first-embed' name='first-embed'><embed name='second-embed'><main id='out'></main><script>const plugins = window.navigator.plugins; const keys = plugins.keys(); const values = plugins.values(); const entries = plugins.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); const secondValue = values.next(); const secondEntry = entries.next(); const before = String(plugins.length) + ':' + String(firstKey.value) + ':' + firstValue.value.id + ':' + firstValue.value.getAttribute('name') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.id + ':' + firstEntry.value.value.getAttribute('name') + ':' + secondValue.value.getAttribute('name') + ':' + String(secondEntry.value.index) + ':' + secondEntry.value.value.getAttribute('name'); plugins.forEach(() => { document.getElementById('out').textContent = 'called'; }, null); document.getElementById('out').textContent = before + ':' + document.getElementById('out').textContent;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "2:0:first-embed:first-embed:0:first-embed:first-embed:second-embed:1:second-embed:called",
    );
}

test "contract: Harness.fromHtmlWithUrl exposes window.navigator.refresh" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>const navigator = window.navigator; document.getElementById('out').textContent = String(navigator.refresh()) + ':' + String(navigator.plugins.refresh()) + ':' + String(navigator.plugins.length);</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "undefined:undefined:0");
}

test "contract: Harness.fromHtmlWithUrl exposes window.navigator languages and legacy aliases" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>const languages = window.navigator.languages; const keys = languages.keys(); const values = languages.values(); const entries = languages.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); document.getElementById('out').textContent = window.navigator.userLanguage + ':' + window.navigator.browserLanguage + ':' + window.navigator.systemLanguage + ':' + window.navigator.oscpu + ':' + String(languages.length) + ':' + languages.item(0) + ':' + languages.toString() + ':' + String(languages.contains('en-US')) + ':' + String(languages.contains('fr-FR')) + ':' + String(firstKey.value) + ':' + firstValue.value + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value; languages.forEach(() => { document.getElementById('out').textContent = 'called'; }, null); document.getElementById('out').textContent = document.getElementById('out').textContent + ':' + 'called';</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "called:called");
}

test "contract: Harness.fromHtmlWithUrl exposes window.performance aliases" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>const performance = window.performance; document.getElementById('out').textContent = String(performance) + ':' + String(window.performance) + ':' + String(performance.timeOrigin) + ':' + String(performance.now()); window.setTimeout(() => { document.getElementById('out').textContent = document.getElementById('out').textContent + ':' + String(performance.now()) + ':' + String(window.performance.now()); }, 5);</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "[object Performance]:[object Performance]:0:0");
    try subject.advanceTime(5);
    try subject.assertValue("#out", "[object Performance]:[object Performance]:0:0:5:5");
}

test "contract: Harness.fromHtmlWithUrl exposes window.navigator toString" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>const navigator = window.navigator; document.getElementById('out').textContent = navigator.toString();</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "[object Navigator]");
}

test "contract: Harness.fromHtmlWithUrl exposes window.navigator.mimeTypes" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>const mimeTypes = window.navigator.mimeTypes; const keys = mimeTypes.keys(); const values = mimeTypes.values(); const entries = mimeTypes.entries(); document.getElementById('out').textContent = 'before'; mimeTypes.forEach(() => { document.getElementById('out').textContent = 'called'; }, null); document.getElementById('out').textContent = String(mimeTypes) + ':' + mimeTypes.toString() + ':' + String(mimeTypes.length) + ':' + String(mimeTypes.item(0)) + ':' + String(mimeTypes.namedItem('text/plain')) + ':' + String(keys.next().done) + ':' + String(values.next().done) + ':' + String(entries.next().done) + ':' + document.getElementById('out').textContent;</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "[object MimeTypeArray]:[object MimeTypeArray]:0:null:null:true:true:true:before");
}

test "contract: Harness.fromHtmlWithUrl exposes window identity aliases" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>const view = document.defaultView; document.getElementById('out').textContent = String(view) + ':' + String(window.window) + ':' + String(window.self) + ':' + String(window.top) + ':' + String(window.parent) + ':' + String(window.opener) + ':' + String(window.closed);</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object Window]:[object Window]:[object Window]:[object Window]:[object Window]:null:false",
    );
}

test "contract: Harness.fromHtmlWithUrl exposes viewport and visibility aliases" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>document.getElementById('out').textContent = String(window.devicePixelRatio) + ':' + String(window.innerWidth) + ':' + String(window.innerHeight) + ':' + String(window.outerWidth) + ':' + String(window.outerHeight) + ':' + document.visibilityState + ':' + String(document.hidden) + ':' + String(document.hasFocus());</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "1:1024:768:1280:800:visible:false:true",
    );
}

test "contract: Harness.fromHtmlWithUrl exposes screen aliases" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>document.getElementById('out').textContent = String(window.screenX) + ':' + String(window.screenY) + ':' + String(window.screenLeft) + ':' + String(window.screenTop);</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "0:0:0:0");
}

test "contract: Harness.fromHtmlWithUrl exposes screen orientation aliases" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>const orientation = window.screen.orientation; document.getElementById('out').textContent = orientation.type + ':' + String(orientation.angle) + ':' + String(orientation);</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "landscape-primary:0:[object ScreenOrientation]");
}

test "contract: Harness.fromHtml exposes Math constants and Math.random" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='out'></main><script>document.getElementById('out').textContent = String(Math) + ':' + String(window.Math) + ':' + String(Math.PI) + ':' + String(window.Math.PI) + ':' + String(Math.random()) + ':' + String(window.Math.random());</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "[object Math]:[object Math]:3.141592653589793:3.141592653589793:0.114:0.363");
}

test "contract: HarnessBuilder.randomSeed seeds Math.random" {
    const allocator = std.testing.allocator;
    var builder = Harness.builder(allocator);
    defer builder.deinit();

    _ = builder.randomSeed(0);
    _ = builder.html("<main id='out'></main><script>document.getElementById('out').textContent = String(Math.random()) + ':' + String(window.Math.random());</script>");

    var subject = try builder.build();
    defer subject.deinit();

    try subject.assertValue("#out", "0.041:0.034");
}

test "contract: HarnessBuilder.randomSeed seeds crypto.randomUUID" {
    const allocator = std.testing.allocator;
    var builder = Harness.builder(allocator);
    defer builder.deinit();

    _ = builder.randomSeed(0);
    _ = builder.html("<main id='out'></main><script>document.getElementById('out').textContent = String(window.crypto) + ':' + window.crypto.randomUUID();</script>");

    var subject = try builder.build();
    defer subject.deinit();

    try subject.assertValue("#out", "[object Crypto]:29da53d4-9dee-4728-9182-3bfc0596ef50");
}

test "contract: Harness.fromHtmlWithUrl exposes window.history navigation methods" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>const history = window.history; const beforeLength = history.length; history.replaceState(null, '', 'https://example.test:8443/replaced'); history.pushState(null, '', 'https://example.test:8443/pushed'); history.back(); history.forward(); history.go(-1); document.getElementById('out').textContent = String(beforeLength) + ':' + String(history.length) + ':' + String(history.state) + ':' + window.location.href;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "1:2:null:https://example.test:8443/replaced",
    );
    try std.testing.expectEqual(@as(usize, 3), subject.mocksMut().location().navigations().len);
    try std.testing.expectEqualStrings("https://example.test:8443/replaced", subject.mocksMut().location().navigations()[0]);
    try std.testing.expectEqualStrings("https://example.test:8443/pushed", subject.mocksMut().location().navigations()[1]);
    try std.testing.expectEqualStrings("https://example.test:8443/replaced", subject.mocksMut().location().navigations()[2]);
}

test "contract: Harness.fromHtmlWithUrl tracks limited history.state payloads" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>const history = window.history; const before = String(history.state); history.replaceState('seed', '', 'https://example.test:8443/replaced'); const afterReplace = String(history.state); history.pushState('pushed', '', 'https://example.test:8443/pushed'); const afterPush = String(history.state); history.back(); const afterBack = String(history.state); document.getElementById('out').textContent = before + ':' + afterReplace + ':' + afterPush + ':' + afterBack + ':' + window.location.href;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "null:seed:pushed:seed:https://example.test:8443/replaced",
    );
}

test "contract: Harness.fromHtmlWithUrl exposes document.scrollingElement, window.frames, window.length, and history.scrollRestoration" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>const history = window.history; const scrolling = document.scrollingElement; history.scrollRestoration = 'manual'; document.getElementById('out').textContent = String(scrolling) + ':' + String(window.frames) + ':' + String(window.length) + ':' + history.scrollRestoration;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object Element]:[object Window]:0:manual",
    );
}

test "failure: Harness.fromHtml rejects invalid history.scrollRestoration values" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>window.history.scrollRestoration = 'sideways';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects screen orientation assignments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>window.screen.orientation.type = 'portrait-primary';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects invalid Math.random arity" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>Math.random(1);</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects invalid crypto.randomUUID arity" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>window.crypto.randomUUID(1);</script>",
        ),
    );
}

test "contract: Harness.fromHtmlWithUrl exposes origin and domain aliases" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='root'><span id='child'></span></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = document.domain + ':' + document.origin + ':' + window.origin + ':' + root.origin + ':' + child.origin;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "example.test:https://example.test:8443:https://example.test:8443:https://example.test:8443:https://example.test:8443",
    );
}

test "contract: Harness.click uses seeded matchMedia state in event handlers" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<button id='toggle'>Toggle</button><div id='out'></div><script>document.getElementById('toggle').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); document.getElementById('out').textContent = String(mql) + ':' + mql.media + ':' + String(mql.matches); });</script>",
    );
    defer subject.deinit();

    try subject.mocksMut().matchMedia().seedMatch("(max-width: 600px)", true);

    try subject.click("#toggle");
    try subject.assertValue("#out", "[object MediaQueryList]:(max-width: 600px):true");
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().matchMedia().calls().len);
    try std.testing.expectEqualStrings(
        "(max-width: 600px)",
        subject.mocksMut().matchMedia().calls()[0].query,
    );
}

test "contract: Harness.click dispatches matchMedia listeners after reseeding" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<button id='toggle'>Toggle</button><div id='out'></div><script>document.getElementById('toggle').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); mql.addListener(() => { document.getElementById('out').textContent = 'changed'; }); document.getElementById('out').textContent = String(mql.matches); });</script>",
    );
    defer subject.deinit();

    try subject.mocksMut().matchMedia().seedMatch("(max-width: 600px)", false);

    try subject.click("#toggle");
    try subject.assertValue("#out", "false");

    try subject.mocksMut().matchMedia().seedMatch("(max-width: 600px)", true);
    try subject.flush();
    try subject.assertValue("#out", "changed");
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().matchMedia().calls().len);
    try std.testing.expectEqualStrings(
        "(max-width: 600px)",
        subject.mocksMut().matchMedia().calls()[0].query,
    );
}

test "contract: Harness.click dispatches matchMedia change event listeners" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<button id='toggle'>Toggle</button><div id='out'></div><script>document.getElementById('toggle').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); mql.addEventListener('change', () => { document.getElementById('out').textContent = 'changed'; }); document.getElementById('out').textContent = String(mql.matches); });</script>",
    );
    defer subject.deinit();

    try subject.mocksMut().matchMedia().seedMatch("(max-width: 600px)", false);

    try subject.click("#toggle");
    try subject.assertValue("#out", "false");

    try subject.mocksMut().matchMedia().seedMatch("(max-width: 600px)", true);
    try subject.flush();
    try subject.assertValue("#out", "changed");
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().matchMedia().calls().len);
    try std.testing.expectEqualStrings(
        "(max-width: 600px)",
        subject.mocksMut().matchMedia().calls()[0].query,
    );
}

test "contract: Harness.click can remove matchMedia listeners before reseeding" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<button id='add'>Add</button><button id='remove'>Remove</button><div id='out'></div><script>document.getElementById('add').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); mql.addListener(() => { document.getElementById('out').textContent = 'changed'; }); document.getElementById('out').textContent = String(mql.matches); }); document.getElementById('remove').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); mql.removeListener(() => { document.getElementById('out').textContent = 'changed'; }); });</script>",
    );
    defer subject.deinit();

    try subject.mocksMut().matchMedia().seedMatch("(max-width: 600px)", false);

    try subject.click("#add");
    try subject.assertValue("#out", "false");
    try subject.click("#remove");

    try subject.mocksMut().matchMedia().seedMatch("(max-width: 600px)", true);
    try subject.flush();
    try subject.assertValue("#out", "false");
    try std.testing.expectEqual(@as(usize, 2), subject.mocksMut().matchMedia().calls().len);
}

test "contract: Harness.click can remove matchMedia change event listeners before reseeding" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<button id='add'>Add</button><button id='remove'>Remove</button><div id='out'></div><script>document.getElementById('add').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); mql.addEventListener('change', () => { document.getElementById('out').textContent = 'changed'; }); document.getElementById('out').textContent = String(mql.matches); }); document.getElementById('remove').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); mql.removeEventListener('change', () => { document.getElementById('out').textContent = 'changed'; }); });</script>",
    );
    defer subject.deinit();

    try subject.mocksMut().matchMedia().seedMatch("(max-width: 600px)", false);

    try subject.click("#add");
    try subject.assertValue("#out", "false");
    try subject.click("#remove");

    try subject.mocksMut().matchMedia().seedMatch("(max-width: 600px)", true);
    try subject.flush();
    try subject.assertValue("#out", "false");
    try std.testing.expectEqual(@as(usize, 2), subject.mocksMut().matchMedia().calls().len);
}

test "contract: Harness.click dispatches matchMedia onchange callbacks after reseeding" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<button id='toggle'>Toggle</button><div id='out'></div><script>document.getElementById('toggle').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); mql.onchange = () => { document.getElementById('out').textContent = 'changed'; }; document.getElementById('out').textContent = String(mql.matches); });</script>",
    );
    defer subject.deinit();

    try subject.mocksMut().matchMedia().seedMatch("(max-width: 600px)", false);

    try subject.click("#toggle");
    try subject.assertValue("#out", "false");

    try subject.mocksMut().matchMedia().seedMatch("(max-width: 600px)", true);
    try subject.flush();
    try subject.assertValue("#out", "changed");
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().matchMedia().calls().len);
    try std.testing.expectEqualStrings(
        "(max-width: 600px)",
        subject.mocksMut().matchMedia().calls()[0].query,
    );
}

test "contract: Harness.click clears matchMedia onchange callbacks when assigned null" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<button id='add'>Add</button><button id='remove'>Remove</button><div id='out'></div><script>document.getElementById('add').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); mql.onchange = () => { document.getElementById('out').textContent = 'changed'; }; document.getElementById('out').textContent = String(mql.matches); }); document.getElementById('remove').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); mql.onchange = null; });</script>",
    );
    defer subject.deinit();

    try subject.mocksMut().matchMedia().seedMatch("(max-width: 600px)", false);

    try subject.click("#add");
    try subject.assertValue("#out", "false");
    try subject.click("#remove");

    try subject.mocksMut().matchMedia().seedMatch("(max-width: 600px)", true);
    try subject.flush();
    try subject.assertValue("#out", "false");
    try std.testing.expectEqual(@as(usize, 2), subject.mocksMut().matchMedia().calls().len);
}

test "failure: Harness.click surfaces matchMedia mock failures" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<button id='toggle'>Toggle</button><div id='out'></div><script>document.getElementById('toggle').addEventListener('click', () => { document.getElementById('out').textContent = String(window.matchMedia('(max-width: 600px)').matches); });</script>",
    );
    defer subject.deinit();

    try subject.mocksMut().matchMedia().fail("(max-width: 600px)");

    try std.testing.expectError(error.MockError, subject.click("#toggle"));
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().matchMedia().calls().len);
    try std.testing.expectEqualStrings(
        "(max-width: 600px)",
        subject.mocksMut().matchMedia().calls()[0].query,
    );
}

test "failure: Harness.click rejects non-callable matchMedia onchange assignments" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<button id='toggle'>Toggle</button><script>document.getElementById('toggle').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); mql.onchange = 123; });</script>",
    );
    defer subject.deinit();

    try subject.mocksMut().matchMedia().seedMatch("(max-width: 600px)", true);
    try std.testing.expectError(error.ScriptRuntime, subject.click("#toggle"));
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().matchMedia().calls().len);
    try std.testing.expectEqualStrings(
        "(max-width: 600px)",
        subject.mocksMut().matchMedia().calls()[0].query,
    );
}

test "failure: Harness.click rejects malformed matchMedia change event listeners" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<button id='toggle'>Toggle</button><script>document.getElementById('toggle').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); mql.addEventListener('change'); });</script>",
    );
    defer subject.deinit();

    try subject.mocksMut().matchMedia().seedMatch("(max-width: 600px)", true);

    try std.testing.expectError(error.ScriptRuntime, subject.click("#toggle"));
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().matchMedia().calls().len);
    try std.testing.expectEqualStrings(
        "(max-width: 600px)",
        subject.mocksMut().matchMedia().calls()[0].query,
    );
}

test "failure: Harness.click rejects non-callable matchMedia change event listeners" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<button id='toggle'>Toggle</button><script>document.getElementById('toggle').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); mql.addEventListener('change', 123); });</script>",
    );
    defer subject.deinit();

    try subject.mocksMut().matchMedia().seedMatch("(max-width: 600px)", true);

    try std.testing.expectError(error.ScriptRuntime, subject.click("#toggle"));
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().matchMedia().calls().len);
    try std.testing.expectEqualStrings(
        "(max-width: 600px)",
        subject.mocksMut().matchMedia().calls()[0].query,
    );
}

test "failure: Harness.fromHtml rejects assignments to read-only document URL aliases" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='out'></main><script>document.URL = 'https://example.test/next';</script>",
        ),
    );
}

test "failure: Harness.fromHtmlWithUrl surfaces invalid window.history URLs" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.MockError,
        Harness.fromHtmlWithUrl(
            allocator,
            "https://example.test:8443/start?x#old",
            "<script>window.history.replaceState(null, '', '   ');</script>",
        ),
    );
}
