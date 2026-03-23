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

test "failure: malformed html is rejected" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.HtmlParse,
        Harness.fromHtml(allocator, "<main><span></main>"),
    );
}

test "contract: Harness.assertExists and Harness.dumpDom expose read-only DOM inspection" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<main id='app' data-state='Ready'><span>Hello</span><input disabled></main>");
    defer subject.deinit();

    try subject.assertExists("#app");
    try subject.assertExists("main");
    try subject.assertExists("[data-state=ready i]");
    try subject.assertExists("[disabled]");

    const dump = try subject.dumpDom(allocator);
    defer allocator.free(dump);

    try std.testing.expectEqualStrings(
        "#document\n  <main id=\"app\" data-state=\"Ready\">\n    <span>\n      \"Hello\"\n    </span>\n    <input disabled />\n  </main>\n",
        dump,
    );
}

test "contract: Harness.assertExists resolves class selectors and combinators" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='app' class='shell'><section class='panel'><button id='save' class='primary'>Save</button></section><section class='panel'><button id='cancel' class='secondary'>Cancel</button></section></main>",
    );
    defer subject.deinit();

    try subject.assertExists(".shell");
    try subject.assertExists("main.shell");
    try subject.assertExists("#app.shell");
    try subject.assertExists("main section.panel");
    try subject.assertExists("main > section.panel");
    try subject.assertExists("main section.panel > button.primary");
    try subject.assertExists("main > section.panel > button.secondary");
}

test "contract: Harness.assertExists resolves universal selectors" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='app'><section id='child'></section></main>",
    );
    defer subject.deinit();

    try subject.assertExists("*");
    try subject.assertExists("main *");
    try subject.assertExists("#app > *");
}

test "contract: Harness.fromHtml runs inline scripts during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='out'>Before</main><script>document.getElementById('out').textContent = 'Hello';</script>",
    );
    defer subject.deinit();

    const dump = try subject.dumpDom(allocator);
    defer allocator.free(dump);

    try std.testing.expectEqualStrings(
        "#document\n  <main id=\"out\">\n    \"Hello\"\n  </main>\n  <script>\n    \"document.getElementById('out').textContent = 'Hello';\"\n  </script>\n",
        dump,
    );
}

test "contract: Harness.fromHtml exposes currentScript and readyState during inline bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='out'></main><script id='first'>document.getElementById('out').textContent = document.currentScript.getAttribute('id') + ':' + document.readyState;</script><script id='second'>document.getElementById('out').textContent += ':' + document.currentScript.getAttribute('id') + ':' + document.readyState;</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "first:loading:second:loading");
}

test "contract: Harness.fromHtml exposes document metadata and window.children during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<html id='html'><head><title>Example</title></head><body id='body'><main id='out'></main><script>const metadata = document.compatMode + ':' + document.characterSet + ':' + document.charset + ':' + document.contentType; const active = document.activeElement.getAttribute('id'); const documentChildren = document.children; const windowChildren = window.children; document.getElementById('out').textContent = metadata + ':' + active + ':' + String(documentChildren.length) + ':' + String(windowChildren.length) + ':' + String(window.frameElement) + ':' + documentChildren.item(0).getAttribute('id') + ':' + windowChildren.item(0).getAttribute('id');</script></body></html>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "CSS1Compat:UTF-8:UTF-8:text/html:body:1:1:null:html:html");
}

test "failure: Harness.fromHtml rejects window.frameElement assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='out'></div><script>window.frameElement = document.documentElement;</script></main>",
        ),
    );
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

test "contract: Harness.fromHtml runs script querySelectorAll during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section><button id='first' class='primary'>First</button></section><button id='second' class='secondary'>Second</button></main><div id='out'></div><script>document.querySelector('#out').textContent = document.querySelectorAll('button').length + ':' + document.querySelectorAll('button').item(0).textContent + ':' + document.querySelectorAll('button').item(1).textContent + ':' + String(document.querySelector('#root').querySelectorAll('button').length) + ':' + String(document.querySelectorAll('button').item(99));</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:First:Second:2:null");
}

test "contract: Harness.fromHtmlWithUrl resolves target and nth pseudo-classes during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://app.local/#named",
        "<main id='root'><a id='fallback' name='named'>Named</a><section id='list'><span id='a' class='match'>A</span><div id='b'>B</div><span id='c'>C</span><div id='d' class='match'>D</div></section><div id='out'></div><script>document.getElementById('out').textContent = document.querySelector(':target').getAttribute('id') + ':' + document.querySelector('#list > span:nth-child(1)').getAttribute('id') + ':' + document.querySelector('#list > div:nth-of-type(2)').getAttribute('id') + ':' + document.querySelector('#list > .match:nth-child(1 of .match)').getAttribute('id') + ':' + document.querySelector('#list > .match:nth-last-child(1 of .match)').getAttribute('id');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "fallback:a:d:a:d");
    try subject.navigate("https://app.local/#d");
    try subject.assertExists("#d:target");
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

test "contract: Harness.fromHtml resolves read-only and read-write pseudo-classes during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada'><input id='readonly' value='Bee' readonly><textarea id='bio'>Hello</textarea><div id='editable' contenteditable='true'>Edit</div><select id='mode'><option id='option' value='a'>A</option></select><button id='button'>Button</button></main><div id='out'></div><script>const readWrite = document.querySelectorAll(':read-write'); document.getElementById('out').textContent = String(readWrite.length) + ':' + readWrite.item(0).getAttribute('id') + ':' + readWrite.item(1).getAttribute('id') + ':' + readWrite.item(2).getAttribute('id') + ':' + String(document.getElementById('readonly').matches(':read-only')) + ':' + String(document.getElementById('mode').matches(':read-only')) + ':' + String(document.getElementById('option').matches(':read-only')) + ':' + String(document.getElementById('button').matches(':read-only'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "3:name:bio:editable:true:true:true:true",
    );
}

test "contract: Harness.fromHtml resolves blank pseudo-classes during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='blank-input' value='   '><textarea id='blank-textarea'>   </textarea><div id='blank-editable' contenteditable='true'>   </div><input id='filled' value='Ada'></main><div id='out'></div><script>const blankInput = document.getElementById('blank-input'); const blankTextarea = document.getElementById('blank-textarea'); const blankEditable = document.getElementById('blank-editable'); const filled = document.getElementById('filled'); document.getElementById('out').textContent = String(blankInput.matches(':blank')) + ':' + String(blankTextarea.matches(':blank')) + ':' + String(blankEditable.matches(':blank')) + ':' + String(filled.matches(':blank')) + ':' + String(document.querySelectorAll('#blank-input:blank').length) + ':' + String(document.querySelectorAll('#blank-textarea:blank').length) + ':' + String(document.querySelectorAll('#blank-editable:blank').length) + ':' + String(document.querySelectorAll('#filled:blank').length);</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "true:true:true:false:1:1:1:0");
}

test "contract: Harness.fromHtml resolves valid, invalid, in-range, and out-of-range pseudo-classes during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='filled' type='text' required value='Ada'><input id='empty' type='text' required><input id='check' type='checkbox' required><input id='check-ok' type='checkbox' required checked><input id='low' type='number' min='2' max='6' value='1'><input id='high' type='number' min='2' max='6' value='7'><input id='in-range' type='number' min='2' max='6' value='4'><textarea id='bio' required></textarea><input id='short' type='text' minlength='4' value='abc'><textarea id='long' maxlength='3'>abcd</textarea><select id='mode' required><option value='a' selected>A</option><option value='b'>B</option></select><button id='button'>Button</button></main><div id='out'></div><script>const valid = document.querySelectorAll(':valid'); const invalid = document.querySelectorAll(':invalid'); const inRange = document.querySelectorAll(':in-range'); const outOfRange = document.querySelectorAll(':out-of-range'); document.getElementById('out').textContent = String(valid.length) + ':' + valid.item(0).getAttribute('id') + ':' + valid.item(1).getAttribute('id') + ':' + valid.item(2).getAttribute('id') + ':' + valid.item(3).getAttribute('id') + ':' + String(invalid.length) + ':' + invalid.item(0).getAttribute('id') + ':' + invalid.item(1).getAttribute('id') + ':' + invalid.item(2).getAttribute('id') + ':' + invalid.item(3).getAttribute('id') + ':' + invalid.item(4).getAttribute('id') + ':' + invalid.item(5).getAttribute('id') + ':' + invalid.item(6).getAttribute('id') + ':' + String(inRange.length) + ':' + inRange.item(0).getAttribute('id') + ':' + String(outOfRange.length) + ':' + outOfRange.item(0).getAttribute('id') + ':' + outOfRange.item(1).getAttribute('id');</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "4:filled:check-ok:in-range:mode:7:empty:check:low:high:bio:short:long:1:in-range:2:low:high",
    );
}

test "contract: Harness.fromHtml runs sibling combinator queries during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><button id='first'>First</button><span id='gap'>Gap</span><button id='second'>Second</button><button id='third'>Third</button></main><div id='out'></div><script>document.getElementById('out').textContent = document.querySelector('#first + span').textContent + ':' + document.querySelector('#first ~ button').textContent + ':' + String(document.querySelectorAll('#first ~ button').length) + ':' + String(document.querySelector('#first + button'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "Gap:Second:2:null");
}

test "contract: Harness.fromHtml runs NodeList.forEach during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><button id='first'>First</button><button id='second'>Second</button></main><div id='out'></div><script>document.querySelectorAll('button').forEach((item, index, list) => { document.getElementById('out').textContent += String(index) + ':' + item.textContent + ':' + String(list.length) + ';'; item.remove(); }, null);</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "0:First:2;1:Second:2;");
}

test "contract: Harness.fromHtml runs document.scripts during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><script id='first-script'></script><script name='named-script'></script></main><div id='out'></div><script>document.getElementById('out').textContent = String(document.scripts.length) + ':' + String(document.scripts.item(0)) + ':' + String(document.scripts.namedItem('first-script')) + ':' + String(document.scripts.namedItem('named-script')) + ':' + String(document.scripts.namedItem('missing')) + ':'; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent += String(document.scripts.length) + ':' + String(document.scripts.namedItem('first-script')) + ':' + String(document.scripts.namedItem('named-script')) + ':' + String(document.scripts.namedItem('missing'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "3:[object Element]:[object Element]:[object Element]:null:1:null:null:null",
    );
}

test "contract: Harness.fromHtml runs document.anchors during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><a name='first'>First</a><a id='ignored'>Ignored</a></main><div id='out'></div><script>document.getElementById('out').textContent = String(document.anchors.length) + ':' + document.anchors.item(0).textContent + ':' + String(document.anchors.namedItem('ignored')) + ':' + document.anchors.namedItem('first').textContent + ':' + String(document.anchors.namedItem('missing')); document.getElementById('root').innerHTML = document.getElementById('root').innerHTML + '<a name=\"second\">Second</a>'; document.getElementById('out').textContent += ':' + String(document.anchors.length) + ':' + document.anchors.namedItem('second').textContent + ':' + String(document.anchors.namedItem('missing'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "1:First:null:First:null:2:Second:null",
    );
}

test "contract: Harness.fromHtml runs document.forms during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='root'><form id='signup' name='signup'>Signup</form><form id='login' name='login'>Login</form></div><div id='out'></div><script>const forms = document.forms; const first = forms.item(0); const named = forms.namedItem('signup'); const before = forms.length; const firstText = first.textContent; const namedText = named.textContent; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(forms.length) + ':' + firstText + ':' + namedText + ':' + String(forms.namedItem('missing'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:0:Signup:Signup:null");
}

test "contract: Harness.fromHtml runs form.elements during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='root'><form id='signup'><input name='first' value='Ada'><textarea name='bio'>Bio</textarea></form></div><div id='out'></div><script>const elements = document.getElementById('signup').elements; const first = elements.item(0); const named = elements.namedItem('first'); const before = elements.length; const firstValue = first.value; const namedValue = named.value; document.getElementById('signup').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(elements.length) + ':' + firstValue + ':' + namedValue + ':' + String(elements.namedItem('missing'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:0:Ada:Ada:null");
}

test "contract: Harness.fromHtml runs form.elements external associations during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='root'><input id='before' form='signup' name='before' value='Before'><form id='signup'><input id='inside' name='inside' value='Inside'><textarea id='bio' name='bio'>Bio</textarea></form><input id='after' form='signup' name='after' value='After'></div><div id='out'></div><script>const form = document.getElementById('signup'); const elements = form.elements; const before = form.length; const first = elements.item(0); const second = elements.item(1); const third = elements.item(2); const fourth = elements.item(3); const beforeNamed = elements.namedItem('before'); const afterNamed = elements.namedItem('after'); form.innerHTML += '<input id=\"extra\" name=\"extra\" value=\"Grace\">'; document.getElementById('out').textContent = String(before) + ':' + String(form.length) + ':' + first.id + ':' + second.id + ':' + third.id + ':' + fourth.id + ':' + beforeNamed.value + ':' + afterNamed.value + ':' + elements.namedItem('extra').value;</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "4:5:before:inside:bio:after:Before:After:Grace");
}

test "contract: Harness.fromHtml runs form.elements RadioNodeList during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='root'><form id='signup'><input type='radio' name='mode' id='mode-a' value='a'><input type='radio' name='mode' id='mode-b' value='b'><textarea name='bio'>Bio</textarea></form></div><div id='out'></div><script>const elements = document.getElementById('signup').elements; const named = elements.namedItem('mode'); const entries = named.entries(); const before = named.length; const firstEntry = entries.next(); const secondEntry = entries.next(); document.getElementById('signup').innerHTML += '<input type=\"radio\" name=\"mode\" id=\"mode-c\" value=\"c\" checked>'; document.getElementById('out').textContent = String(before) + ':' + String(named.length) + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.value + ':' + String(secondEntry.value.index) + ':' + secondEntry.value.value.value + ':' + named.value + ':' + String(named);</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:3:0:a:1:b:c:[object RadioNodeList]");
}

test "contract: Harness.fromHtml runs RadioNodeList value assignment during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='root'><form id='signup'><input type='radio' name='mode' id='mode-a' checked><input type='radio' name='mode' id='mode-b' value='b'><input type='radio' name='mode' id='mode-c' value='c'></form></div><div id='out'></div><script>const named = document.getElementById('signup').elements.namedItem('mode'); const initial = named.value; named.value = 'b'; const afterMatch = named.value; named.value = 'on'; const afterOn = named.value; const onA = String(document.getElementById('mode-a').checked); const onB = String(document.getElementById('mode-b').checked); named.value = 'missing'; document.getElementById('out').textContent = initial + ':' + afterMatch + ':' + afterOn + ':' + onA + ':' + onB + ':' + named.value + ':' + String(document.getElementById('mode-a').checked) + ':' + String(document.getElementById('mode-b').checked) + ':' + String(document.getElementById('mode-c').checked);</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "on:b:on:true:false::false:false:false");
}

test "contract: Harness.fromHtml runs select.options during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='root'><select id='mode'><option name='alpha' value='a'>A</option><option id='second' value='b'>B</option></select></div><div id='out'></div><script>const options = document.getElementById('mode').options; const first = options.item(0); const named = options.namedItem('second'); const before = options.length; const firstText = first.textContent; const namedText = named.textContent; document.getElementById('mode').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(options.length) + ':' + firstText + ':' + namedText + ':' + String(options.namedItem('missing'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:0:A:B:null");
}

test "contract: Harness.fromHtml runs form.length and select.length during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='root'><form id='signup'><input name='first' value='Ada'><textarea name='bio'>Bio</textarea></form><select id='mode'><option id='first-option' value='a'>A</option><option id='second-option' value='b'>B</option></select></div><div id='out'></div><script>const form = document.getElementById('signup'); const select = document.getElementById('mode'); const beforeForm = form.length; const beforeSelect = select.length; form.innerHTML += '<input name=\"extra\" value=\"Grace\">'; select.innerHTML += '<option id=\"third-option\" value=\"c\">C</option>'; document.getElementById('out').textContent = String(beforeForm) + ':' + String(beforeSelect) + ':' + String(form.length) + ':' + String(select.length) + ':' + String(form.elements.length) + ':' + String(select.options.length);</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:2:3:3:3:3");
}

test "contract: Harness.fromHtml runs select.options add and remove during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='root'><select id='mode'><option id='first' value='a'>A</option></select><option id='extra' value='b'>B</option></div><div id='out'></div><script>const select = document.getElementById('mode'); const extra = document.getElementById('extra'); const before = select.options.length; select.options.add(extra); const afterAdd = select.options.length; const entries = select.options.entries(); const firstEntry = entries.next(); select.options.remove(0); document.getElementById('out').textContent = String(before) + ':' + String(afterAdd) + ':' + String(select.options.length) + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + String(select.options.item(0).getAttribute('id')) + ':' + String(select.options.namedItem('first'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "1:2:1:0:first:extra:null");
    try subject.assertExists("#mode > #extra");
    try std.testing.expectError(error.AssertionFailed, subject.assertExists("#first"));
}

test "contract: Harness.fromHtml runs select.selectedOptions during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='root'><select id='mode'><option id='first' value='a' selected>A</option><option id='second' value='b'>B</option></select></div><div id='out'></div><script>const select = document.getElementById('mode'); const selected = select.selectedOptions; const before = selected.length; const first = selected.item(0); select.innerHTML = '<option id=\"third\" value=\"c\" selected>C</option><option id=\"fourth\" value=\"d\" selected>D</option>'; document.getElementById('out').textContent = String(before) + ':' + String(selected.length) + ':' + first.textContent + ':' + selected.item(0).textContent + ':' + selected.item(1).textContent + ':' + String(selected.namedItem('third')) + ':' + String(selected.namedItem('missing'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "1:2:A:C:D:[object Element]:null");
}

test "contract: Harness.fromHtml runs fieldset.elements and datalist.options during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><fieldset id='fieldset'><input name='first' value='Ada'><textarea name='bio'>Bio</textarea></fieldset><datalist id='list'><option name='alpha' value='a'>A</option><option id='second' value='b'>B</option></datalist><div id='out'></div><script>const elements = document.getElementById('fieldset').elements; const options = document.getElementById('list').options; const beforeElements = elements.length; const beforeOptions = options.length; const first = elements.item(0); const namedElement = elements.namedItem('first'); const namedOption = options.namedItem('second'); document.getElementById('fieldset').textContent = 'gone'; document.getElementById('list').textContent = 'gone'; document.getElementById('out').textContent = String(beforeElements) + ':' + String(elements.length) + ':' + String(beforeOptions) + ':' + String(options.length) + ':' + first.value + ':' + namedElement.value + ':' + namedOption.textContent + ':' + String(options.namedItem('missing'));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:0:2:0:Ada:Ada:B:null");
    try subject.assertExists("#fieldset");
    try subject.assertExists("#list");
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

test "failure: Harness.fromHtml rejects non-document all access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='not-doc'></div></main><script>document.getElementById('not-doc').all.length;</script>",
        ),
    );
}

test "contract: Harness.fromHtml runs document.styleSheets during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='root'><style id='first-style'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'><link id='ignored-link' rel='preload' href='b.css'></div><div id='out'></div><script>const sheets = document.styleSheets; const before = sheets.length; const first = sheets.item(0); const second = sheets.item(1); const keys = sheets.keys(); const values = sheets.values(); const entries = sheets.entries(); document.getElementById('root').textContent = 'gone'; const key = keys.next(); const value = values.next(); const entry = entries.next(); document.getElementById('out').textContent = String(before) + ':' + String(sheets.length) + ':' + String(first) + ':' + String(second) + ':' + String(key.value) + ':' + String(value.value) + ':' + String(entry.value.index) + ':' + String(entry.value.value);</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:0:[object CSSStyleSheet]:[object CSSStyleSheet]:0:[object CSSStyleSheet]:0:[object CSSStyleSheet]");
}

test "failure: Harness.fromHtml rejects non-document styleSheets access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='wrapper'><div id='not-doc'></div></div><script>document.getElementById('not-doc').styleSheets.length;</script>",
        ),
    );
}

test "contract: Harness.fromHtml runs document.styleSheets ownerNode during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='root'><style id='first-style'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'><link id='ignored-link' rel='preload' href='b.css'></div><div id='out'></div><script>const sheets = document.styleSheets; const first = sheets.item(0); const second = sheets.item(1); document.getElementById('out').textContent = String(first.ownerNode) + ':' + first.ownerNode.getAttribute('id') + ':' + String(second.ownerNode) + ':' + second.ownerNode.getAttribute('id');</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object Element]:first-style:[object Element]:first-link",
    );
}

test "failure: Harness.fromHtml rejects document.styleSheets ownerNode assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>.primary { color: red; }</style><script>document.styleSheets.item(0).ownerNode = null;</script>",
        ),
    );
}

test "contract: Harness.fromHtml runs document.styleSheets href/title during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='root'><style id='first-style'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' title='theme' href='a.css'></div><div id='out'></div><script>const sheets = document.styleSheets; const first = sheets.item(0); const second = sheets.item(1); document.getElementById('out').textContent = String(first.href) + ':' + String(first.title) + ':' + String(second.href) + ':' + String(second.title);</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "null:null:a.css:theme",
    );
}

test "failure: Harness.fromHtml rejects document.styleSheets href assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>.primary { color: red; }</style><script>document.styleSheets.item(0).href = 'x.css';</script>",
        ),
    );
}

test "contract: Harness.fromHtml runs document.styleSheets disabled during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='root'><style id='first-style' disabled>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css' disabled title='theme'></div><div id='out'></div><script>const sheets = document.styleSheets; const first = sheets.item(0); const second = sheets.item(1); document.getElementById('out').textContent = String(first.disabled) + ':' + String(second.disabled);</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "true:true",
    );
}

test "contract: Harness.fromHtml runs document.styleSheets disabled assignment during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='root'><style id='first-style' disabled>.primary { color: red; }</style></div><div id='out'></div><script>const sheet = document.styleSheets.item(0); const before = sheet.disabled; sheet.disabled = false; document.getElementById('out').textContent = String(before) + ':' + String(sheet.disabled) + ':' + String(document.getElementById('first-style').hasAttribute('disabled'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "true:false:false",
    );
}

test "contract: Harness.fromHtml runs document.styleSheets media during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='root'><style id='first-style' media='print'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' media='screen and (min-width: 1px), print' href='a.css'></div><div id='out'></div><script>const sheets = document.styleSheets; const first = sheets.item(0).media; const second = sheets.item(1).media; document.getElementById('out').textContent = String(first) + ':' + first.mediaText + ':' + String(first.length) + ':' + first.item(0) + ':' + String(second) + ':' + second.mediaText + ':' + String(second.length) + ':' + second.item(0) + ':' + second.item(1);</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "print:print:1:print:screen and (min-width: 1px), print:screen and (min-width: 1px), print:2:screen and (min-width: 1px):print",
    );
}

test "contract: Harness.fromHtml mutates stylesheet media lists" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='root'><style id='first-style' media='print'>.primary { color: red; }</style></div><div id='out'></div><script>const style = document.getElementById('first-style'); const media = document.styleSheets.item(0).media; media.appendMedium('screen'); media.deleteMedium('print'); document.getElementById('out').textContent = String(media) + ':' + media.mediaText + ':' + String(media.length) + ':' + media.item(0) + ':' + style.getAttribute('media');</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "screen:screen:1:screen:screen");
}

test "contract: Harness.fromHtml mutates stylesheet mediaText during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='root'><style id='first-style' media='print'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' media='screen and (min-width: 1px), print' href='a.css'></div><div id='out'></div><script>const styleMedia = document.styleSheets.item(0).media; const linkMedia = document.styleSheets.item(1).media; const before = String(styleMedia) + ':' + String(linkMedia) + ':' + document.getElementById('first-style').getAttribute('media') + ':' + document.getElementById('first-link').getAttribute('media'); styleMedia.mediaText = 'tv, speech'; linkMedia.mediaText = 'print'; document.getElementById('out').textContent = before + ':' + String(styleMedia) + ':' + String(linkMedia) + ':' + document.getElementById('first-style').getAttribute('media') + ':' + document.getElementById('first-link').getAttribute('media');</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "print:screen and (min-width: 1px), print:print:screen and (min-width: 1px), print:tv, speech:print:tv, speech:print",
    );
}

test "failure: Harness.fromHtml rejects document.styleSheets media assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style media='print'>.primary { color: red; }</style><script>document.styleSheets.item(0).media = null;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects read-only stylesheet rule mediaText assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@media screen {.primary { color: red; }}</style><script>document.styleSheets.item(0).cssRules.item(0).media.mediaText = 'print';</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@import url(x.css) screen and (min-width: 1px);</style><script>document.styleSheets.item(0).cssRules.item(0).media.mediaText = 'print';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects CSS rule media list mutation" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@media screen {.primary { color: red; }}</style><script>document.styleSheets.item(0).cssRules.item(0).media.appendMedium('print');</script>",
        ),
    );
}

test "contract: Harness.fromHtml exposes stylesheet owner element sheet, media, rel, hreflang, href, and type" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style id='first-style' type='text/css' media='screen'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' hreflang='en' type='text/css' href='a.css' media='print'><div id='out'></div><script>const style = document.getElementById('first-style'); const link = document.getElementById('first-link'); const styleSheet = style.sheet; const linkSheet = link.sheet; const before = String(styleSheet) + ':' + String(linkSheet) + ':' + style.media + ':' + style.type + ':' + styleSheet.media.mediaText + ':' + link.media + ':' + link.hreflang + ':' + link.type + ':' + linkSheet.media.mediaText + ':' + link.rel + ':' + link.href + ':' + styleSheet.cssRules.item(0).selectorText; style.media = 'tv'; link.media = 'speech'; style.type = 'text/plain'; link.hreflang = 'fr'; link.type = 'application/css'; link.rel = 'preload'; link.href = 'b.css'; document.getElementById('out').textContent = before + ':' + style.media + ':' + style.type + ':' + styleSheet.media.mediaText + ':' + link.media + ':' + link.hreflang + ':' + link.type + ':' + linkSheet.media.mediaText + ':' + link.rel + ':' + link.href + ':' + String(link.sheet) + ':' + String(document.styleSheets.length);</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSStyleSheet]:[object CSSStyleSheet]:screen:text/css:screen:print:en:text/css:print:stylesheet:a.css:.primary:tv:text/plain:tv:speech:fr:application/css:speech:preload:b.css:null:1",
    );
}

test "contract: Harness.fromHtml exposes stylesheet owner element relList" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'><div id='out'></div><script>const link = document.getElementById('first-link'); const relList = link.relList; const sheets = document.styleSheets; const before = String(relList.length) + ':' + String(relList.contains('stylesheet')) + ':' + String(relList.supports('stylesheet')) + ':' + String(relList.supports('preload')) + ':' + String(relList.supports('bogus')) + ':' + String(sheets.length) + ':' + String(link.sheet) + ':' + link.href; relList.add('preload'); relList.remove('stylesheet'); link.href = 'b.css'; document.getElementById('out').textContent = before + ':' + String(relList.length) + ':' + String(relList.contains('stylesheet')) + ':' + String(relList.contains('preload')) + ':' + String(relList.supports('stylesheet')) + ':' + String(relList.supports('preload')) + ':' + String(relList.supports('bogus')) + ':' + String(sheets.length) + ':' + String(link.sheet) + ':' + link.href;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "1:true:true:true:false:2:[object CSSStyleSheet]:a.css:1:false:true:true:true:false:1:null:b.css",
    );
}

test "contract: Harness.fromHtml exposes stylesheet owner element disabled reflection" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style id='first-style' disabled>.primary { color: red; }</style><link id='first-link' rel='stylesheet' disabled href='a.css'><div id='out'></div><script>const style = document.getElementById('first-style'); const link = document.getElementById('first-link'); const styleSheet = style.sheet; const linkSheet = link.sheet; const before = String(style.disabled) + ':' + String(link.disabled) + ':' + String(styleSheet.disabled) + ':' + String(linkSheet.disabled); style.disabled = false; link.disabled = false; document.getElementById('out').textContent = before + ':' + String(style.disabled) + ':' + String(link.disabled) + ':' + String(styleSheet.disabled) + ':' + String(linkSheet.disabled) + ':' + String(style.hasAttribute('disabled')) + ':' + String(link.hasAttribute('disabled'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "true:true:true:true:false:false:false:false:false:false",
    );
}

test "failure: Harness.fromHtml rejects non-stylesheet owner element sheet access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').sheet;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-stylesheet owner element media assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').media = 'print';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects relList.supports on unsupported token lists" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<button id='button' class='base'></button><script>document.getElementById('button').classList.supports('stylesheet');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-style-link disabled access and assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').disabled;</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').disabled = true;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-link owner element rel access and assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').rel;</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').rel = 'preload';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-link owner element hreflang access and assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').hreflang;</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').hreflang = 'en';</script>",
        ),
    );
}

test "contract: Harness.fromHtml exposes stylesheet owner element crossOrigin reflection" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' crossorigin='anonymous' href='a.css'><div id='out'></div><script>const link = document.getElementById('first-link'); const sheets = document.styleSheets; const before = String(link.crossOrigin) + ':' + String(sheets.length) + ':' + String(link.sheet); link.crossOrigin = 'use-credentials'; document.getElementById('out').textContent = before + ':' + link.crossOrigin + ':' + String(sheets.length) + ':' + String(link.sheet) + ':' + link.href;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "anonymous:2:[object CSSStyleSheet]:use-credentials:2:[object CSSStyleSheet]:a.css",
    );
}

test "failure: Harness.fromHtml rejects non-link owner element crossOrigin access and assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').crossOrigin;</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').crossOrigin = 'anonymous';</script>",
        ),
    );
}

test "contract: Harness.fromHtml exposes stylesheet owner element referrerPolicy reflection" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' referrerpolicy='no-referrer' href='a.css'><div id='out'></div><script>const link = document.getElementById('first-link'); const sheets = document.styleSheets; const before = String(link.referrerPolicy) + ':' + String(sheets.length) + ':' + String(link.sheet); link.referrerPolicy = 'same-origin'; document.getElementById('out').textContent = before + ':' + link.referrerPolicy + ':' + String(sheets.length) + ':' + String(link.sheet) + ':' + link.href;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "no-referrer:2:[object CSSStyleSheet]:same-origin:2:[object CSSStyleSheet]:a.css",
    );
}

test "failure: Harness.fromHtml rejects non-link owner element referrerPolicy access and assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').referrerPolicy;</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').referrerPolicy = 'origin';</script>",
        ),
    );
}

test "contract: Harness.fromHtml exposes stylesheet owner element integrity reflection" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' integrity='sha384-abc' href='a.css'><div id='out'></div><script>const link = document.getElementById('first-link'); const sheets = document.styleSheets; const before = String(link.integrity) + ':' + String(sheets.length) + ':' + String(link.sheet); link.integrity = 'sha384-def'; document.getElementById('out').textContent = before + ':' + link.integrity + ':' + String(sheets.length) + ':' + String(link.sheet) + ':' + link.href;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "sha384-abc:2:[object CSSStyleSheet]:sha384-def:2:[object CSSStyleSheet]:a.css",
    );
}

test "failure: Harness.fromHtml rejects non-link owner element integrity access and assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').integrity;</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').integrity = 'sha384-def';</script>",
        ),
    );
}

test "contract: Harness.fromHtml exposes stylesheet owner element as reflection" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' as='style' href='a.css'><div id='out'></div><script>const link = document.getElementById('first-link'); const sheets = document.styleSheets; const before = String(link.as) + ':' + String(sheets.length) + ':' + String(link.sheet); link.as = 'script'; document.getElementById('out').textContent = before + ':' + link.as + ':' + String(sheets.length) + ':' + String(link.sheet) + ':' + link.href;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "style:2:[object CSSStyleSheet]:script:2:[object CSSStyleSheet]:a.css",
    );
}

test "contract: Harness.fromHtml exposes stylesheet owner element charset reflection" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' charset='utf-8' href='a.css'><div id='out'></div><script>const link = document.getElementById('first-link'); const sheets = document.styleSheets; const before = String(link.charset) + ':' + String(sheets.length) + ':' + String(link.sheet); link.charset = 'windows-1252'; document.getElementById('out').textContent = before + ':' + link.charset + ':' + String(sheets.length) + ':' + String(link.sheet) + ':' + link.href;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "utf-8:2:[object CSSStyleSheet]:windows-1252:2:[object CSSStyleSheet]:a.css",
    );
}

test "contract: Harness.fromHtml exposes stylesheet owner element responsive image metadata reflection" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' imagesrcset='a-1x.css 1x, a-2x.css 2x' imagesizes='100vw' href='a.css'><div id='out'></div><script>const link = document.getElementById('first-link'); const sheets = document.styleSheets; const before = String(link.imageSrcset) + ':' + String(link.imageSizes) + ':' + String(sheets.length) + ':' + String(link.sheet); link.imageSrcset = 'b-1x.css 1x'; link.imageSizes = '50vw'; document.getElementById('out').textContent = before + ':' + link.imageSrcset + ':' + link.imageSizes + ':' + String(sheets.length) + ':' + String(link.sheet) + ':' + link.href;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "a-1x.css 1x, a-2x.css 2x:100vw:2:[object CSSStyleSheet]:b-1x.css 1x:50vw:2:[object CSSStyleSheet]:a.css",
    );
}

test "contract: Harness.fromHtml exposes stylesheet owner element fetchPriority reflection" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' fetchpriority='low' href='a.css'><div id='out'></div><script>const link = document.getElementById('first-link'); const sheets = document.styleSheets; const before = String(link.fetchPriority) + ':' + String(sheets.length) + ':' + String(link.sheet); link.fetchPriority = 'high'; document.getElementById('out').textContent = before + ':' + link.fetchPriority + ':' + String(sheets.length) + ':' + String(link.sheet) + ':' + link.href;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "low:2:[object CSSStyleSheet]:high:2:[object CSSStyleSheet]:a.css",
    );
}

test "failure: Harness.fromHtml rejects non-link owner element as access and assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').as;</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').as = 'script';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-link owner element charset access and assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').charset;</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').charset = 'utf-8';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-link owner element responsive image metadata access and assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').imageSrcset;</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').imageSrcset = 'a.css';</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').imageSizes;</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').imageSizes = '100vw';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-link owner element fetchPriority access and assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').fetchPriority;</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').fetchPriority = 'high';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-link owner element relList access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').relList;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-link owner element href access and assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').href;</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').href = 'a.css';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-stylesheet owner element type access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').type;</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').type = 'text/plain';</script>",
        ),
    );
}

test "contract: Harness.fromHtml runs document.styleSheets cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='root'><style id='first-style'>.primary { color: red; } .secondary { color: blue; }</style><link id='first-link' rel='stylesheet' href='a.css'></div><div id='out'></div><script>const sheets = document.styleSheets; const first = sheets.item(0); const second = sheets.item(1); const rules = first.cssRules; const ruleKeys = rules.keys(); const ruleValues = rules.values(); const ruleEntries = rules.entries(); const rule = rules.item(0); const linkRules = second.cssRules; document.getElementById('root').textContent = 'gone'; const key = ruleKeys.next(); const value = ruleValues.next(); const entry = ruleEntries.next(); document.getElementById('out').textContent = String(rules.length) + ':' + String(linkRules.length) + ':' + String(rule) + ':' + rule.selectorText + ':' + rule.cssText + ':' + String(key.value) + ':' + String(value.value) + ':' + String(entry.value.index) + ':' + String(entry.value.value);</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "2:0:[object CSSStyleRule]:.primary:.primary { color: red; }:0:[object CSSStyleRule]:0:[object CSSStyleRule]",
    );
}

test "contract: Harness.fromHtml runs CSSStyleRule.style during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>.primary { color: red; font-weight: bold; }</style><div id='out'></div><script>const rule = document.styleSheets.item(0).cssRules.item(0); const style = rule.style; document.getElementById('out').textContent = String(rule) + ':' + String(style) + ':' + style.cssText + ':' + String(style.length) + ':' + style.item(0) + ':' + style.item(1) + ':' + style.getPropertyValue('color') + ':' + style.getPropertyValue('font-weight') + ':' + style.getPropertyPriority('color');</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSStyleRule]:color: red; font-weight: bold;:color: red; font-weight: bold;:2:color:font-weight:red:bold:",
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @media cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@media screen { .primary { color: red; } .secondary { color: blue; } }</style><div id='out'></div><script>const media = document.styleSheets.item(0).cssRules.item(0); const nested = media.cssRules; const list = media.media; document.getElementById('out').textContent = String(media) + ':' + media.conditionText + ':' + String(list) + ':' + String(list.length) + ':' + list.item(0) + ':' + String(nested.length) + ':' + nested.item(0).selectorText + ':' + nested.item(1).selectorText + ':' + nested.item(0).cssText;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSMediaRule]:screen:screen:1:screen:2:.primary:.secondary:.primary { color: red; }",
    );
}

test "failure: Harness.fromHtml rejects non-media media access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@supports (display: grid) { .primary { color: red; } }</style><script>document.styleSheets.item(0).cssRules.item(0).media.length;</script>",
        ),
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @supports cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@supports (display: grid) { .primary { color: red; } .secondary { color: blue; } }</style><div id='out'></div><script>const supports = document.styleSheets.item(0).cssRules.item(0); const nested = supports.cssRules; document.getElementById('out').textContent = String(supports) + ':' + supports.conditionText + ':' + String(nested.length) + ':' + nested.item(0).selectorText + ':' + nested.item(1).selectorText + ':' + nested.item(0).cssText;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSSupportsRule]:(display: grid):2:.primary:.secondary:.primary { color: red; }",
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @container cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@container card (min-width: 1px) { .primary { color: red; } .secondary { color: blue; } }</style><div id='out'></div><script>const rule = document.styleSheets.item(0).cssRules.item(0); const nested = rule.cssRules; document.getElementById('out').textContent = String(rule) + ':' + rule.conditionText + ':' + String(nested.length) + ':' + nested.item(0).selectorText + ':' + nested.item(1).selectorText + ':' + nested.item(0).cssText + ':' + rule.cssText;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSContainerRule]:card (min-width: 1px):2:.primary:.secondary:.primary { color: red; }:@container card (min-width: 1px) { .primary { color: red; } .secondary { color: blue; } }",
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @starting-style cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@starting-style { .primary { color: red; } .secondary { color: blue; } }</style><div id='out'></div><script>const rule = document.styleSheets.item(0).cssRules.item(0); const nested = rule.cssRules; document.getElementById('out').textContent = String(rule) + ':' + String(nested.length) + ':' + nested.item(0).selectorText + ':' + nested.item(1).selectorText + ':' + nested.item(0).cssText + ':' + rule.cssText;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSStartingStyleRule]:2:.primary:.secondary:.primary { color: red; }:@starting-style { .primary { color: red; } .secondary { color: blue; } }",
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @position-try cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@position-try card { .primary { color: red; } .secondary { color: blue; } }</style><div id='out'></div><script>const rule = document.styleSheets.item(0).cssRules.item(0); document.getElementById('out').textContent = String(rule) + ':' + rule.name + ':' + rule.cssText;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSPositionTryRule]:card:@position-try card { .primary { color: red; } .secondary { color: blue; } }",
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @keyframes cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@keyframes pulse { from { opacity: 0; } to { opacity: 1; } }</style><div id='out'></div><script>const keyframes = document.styleSheets.item(0).cssRules.item(0); const nested = keyframes.cssRules; document.getElementById('out').textContent = String(keyframes) + ':' + keyframes.name + ':' + String(nested.length) + ':' + nested.item(0).keyText + ':' + nested.item(1).keyText + ':' + nested.item(0).cssText;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSKeyframesRule]:pulse:2:from:to:from { opacity: 0; }",
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @font-face cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@font-face { font-family: x; src: url(x.woff); }</style><div id='out'></div><script>const rule = document.styleSheets.item(0).cssRules.item(0); const style = rule.style; document.getElementById('out').textContent = String(rule) + ':' + rule.cssText + ':' + String(style) + ':' + style.cssText + ':' + style.getPropertyValue('font-family') + ':' + style.getPropertyValue('src');</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSFontFaceRule]:@font-face { font-family: x; src: url(x.woff); }:font-family: x; src: url(x.woff);:font-family: x; src: url(x.woff);:x:url(x.woff)",
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @font-feature-values cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@font-feature-values test { .x { color: red; } }</style><div id='out'></div><script>const rule = document.styleSheets.item(0).cssRules.item(0); document.getElementById('out').textContent = String(rule) + ':' + rule.fontFamily + ':' + rule.cssText;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSFontFeatureValuesRule]:test:@font-feature-values test { .x { color: red; } }",
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @font-palette-values cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@font-palette-values --palette { font-family: Bungee Spice; base-palette: light; override-colors: 0 red; }</style><div id='out'></div><script>const rule = document.styleSheets.item(0).cssRules.item(0); document.getElementById('out').textContent = String(rule) + ':' + rule.name + ':' + rule.fontFamily + ':' + rule.basePalette + ':' + rule.overrideColors + ':' + rule.cssText;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSFontPaletteValuesRule]:--palette:Bungee Spice:light:0 red:@font-palette-values --palette { font-family: Bungee Spice; base-palette: light; override-colors: 0 red; }",
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @color-profile cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@color-profile --swopc { src: url(http://example.org/swop-coated.icc); rendering-intent: perceptual; components: cyan, magenta, yellow, black; }</style><div id='out'></div><script>const rule = document.styleSheets.item(0).cssRules.item(0); document.getElementById('out').textContent = String(rule) + ':' + rule.name + ':' + rule.src + ':' + rule.renderingIntent + ':' + rule.components + ':' + rule.cssText;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSColorProfileRule]:--swopc:url(http://example.org/swop-coated.icc):perceptual:cyan, magenta, yellow, black:@color-profile --swopc { src: url(http://example.org/swop-coated.icc); rendering-intent: perceptual; components: cyan, magenta, yellow, black; }",
    );
}

test "contract: Harness.fromHtml exposes CSSRule.type during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>.primary { color: red; } @color-profile --swopc { src: url(http://example.org/swop-coated.icc); rendering-intent: perceptual; components: cyan, magenta, yellow, black; }</style><div id='out'></div><script>const rules = document.styleSheets.item(0).cssRules; const styleRule = rules.item(0); const profileRule = rules.item(1); document.getElementById('out').textContent = String(styleRule.type) + ':' + String(profileRule.type) + ':' + String(styleRule) + ':' + String(profileRule);</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "1:0:[object CSSStyleRule]:[object CSSColorProfileRule]",
    );
}

test "contract: Harness.fromHtml exposes CSSRule.parentStyleSheet during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>.primary { color: red; } @media screen { .secondary { color: blue; } }</style><div id='out'></div><script>const rules = document.styleSheets.item(0).cssRules; const styleRule = rules.item(0); const mediaRule = rules.item(1); const nestedRule = mediaRule.cssRules.item(0); document.getElementById('out').textContent = String(styleRule.parentStyleSheet) + ':' + String(mediaRule.parentStyleSheet) + ':' + String(nestedRule.parentStyleSheet);</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSStyleSheet]:[object CSSStyleSheet]:[object CSSStyleSheet]",
    );
}

test "contract: Harness.fromHtml exposes CSSRule.parentRule during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>.primary { color: red; } @media screen { .secondary { color: blue; } }</style><div id='out'></div><script>const rules = document.styleSheets.item(0).cssRules; const styleRule = rules.item(0); const mediaRule = rules.item(1); const nestedRule = mediaRule.cssRules.item(0); document.getElementById('out').textContent = String(styleRule.parentRule) + ':' + String(mediaRule.parentRule) + ':' + String(nestedRule.parentRule);</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "null:null:[object CSSMediaRule]",
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @import cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@import url(x.css) screen and (min-width: 1px), print;</style><div id='out'></div><script>const rule = document.styleSheets.item(0).cssRules.item(0); const media = rule.media; document.getElementById('out').textContent = String(rule) + ':' + rule.href + ':' + rule.mediaText + ':' + String(media) + ':' + media.mediaText + ':' + String(media.length) + ':' + media.item(0) + ':' + media.item(1) + ':' + rule.cssText;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSImportRule]:x.css:screen and (min-width: 1px), print:screen and (min-width: 1px), print:screen and (min-width: 1px), print:2:screen and (min-width: 1px):print:@import url(x.css) screen and (min-width: 1px), print;",
    );
}

test "contract: Harness.fromHtml exposes location ancestorOrigins as empty DOMStringList" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='out'></main><script>const location = window.location; const origins = location.ancestorOrigins; document.getElementById('out').textContent = String(origins) + ':' + origins.toString() + ':' + String(origins.length) + ':' + String(origins.item(0));</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object DOMStringList]:[object DOMStringList]:0:null",
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @import supports/layer metadata during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@import url(x.css) layer(foo) supports(display: grid) screen and (min-width: 1px), print;</style><div id='out'></div><script>const rule = document.styleSheets.item(0).cssRules.item(0); const media = rule.media; document.getElementById('out').textContent = String(rule) + ':' + rule.href + ':' + rule.layerName + ':' + rule.supportsText + ':' + rule.mediaText + ':' + String(media) + ':' + media.mediaText + ':' + String(media.length) + ':' + media.item(0) + ':' + media.item(1) + ':' + rule.cssText;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSImportRule]:x.css:foo:display: grid:screen and (min-width: 1px), print:screen and (min-width: 1px), print:screen and (min-width: 1px), print:2:screen and (min-width: 1px):print:@import url(x.css) layer(foo) supports(display: grid) screen and (min-width: 1px), print;",
    );
}

test "contract: Harness.fromHtml exposes document.styleSheets @import styleSheet and stylesheet ownerRule as null" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@import url(x.css) screen and (min-width: 1px);</style><div id='out'></div><script>const rule = document.styleSheets.item(0).cssRules.item(0); const sheet = document.styleSheets.item(0); document.getElementById('out').textContent = String(rule.styleSheet) + ':' + String(sheet.ownerRule);</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "null:null",
    );
}

test "failure: Harness.fromHtml rejects document.styleSheets @import media assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@import url(x.css) screen and (min-width: 1px);</style><script>document.styleSheets.item(0).cssRules.item(0).media = null;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects document.styleSheets @import styleSheet assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@import url(x.css) screen and (min-width: 1px);</style><script>document.styleSheets.item(0).cssRules.item(0).styleSheet = null;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects location ancestorOrigins assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<script>window.location.ancestorOrigins = null;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects document.styleSheets @import supportsText assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@import url(x.css) layer(foo) supports(display: grid) screen and (min-width: 1px);</style><script>document.styleSheets.item(0).cssRules.item(0).supportsText = null;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects document.styleSheets @import layerName assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@import url(x.css) layer(foo) supports(display: grid) screen and (min-width: 1px);</style><script>document.styleSheets.item(0).cssRules.item(0).layerName = null;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects document.styleSheets ownerRule assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@import url(x.css) screen and (min-width: 1px);</style><script>document.styleSheets.item(0).ownerRule = null;</script>",
        ),
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @charset cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@charset \"UTF-8\"; .primary { color: red; }</style><div id='out'></div><script>const rules = document.styleSheets.item(0).cssRules; const charsetRule = rules.item(0); const styleRule = rules.item(1); document.getElementById('out').textContent = String(charsetRule) + ':' + charsetRule.encoding + ':' + charsetRule.cssText + ':' + String(charsetRule.type) + ':' + String(styleRule.type);</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSCharsetRule]:UTF-8:@charset \"UTF-8\";:2:1",
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @namespace cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@namespace svg url(http://www.w3.org/2000/svg);</style><div id='out'></div><script>const rule = document.styleSheets.item(0).cssRules.item(0); document.getElementById('out').textContent = String(rule) + ':' + rule.prefix + ':' + rule.namespaceURI + ':' + rule.cssText;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSNamespaceRule]:svg:http://www.w3.org/2000/svg:@namespace svg url(http://www.w3.org/2000/svg);",
    );
}

test "contract: Harness.fromHtml runs StyleSheetList and RadioNodeList forEach during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<div id='root'><style>.primary { color: red; } .secondary { color: blue; }</style><link rel='stylesheet' href='a.css'><form id='signup'><input type='radio' name='mode' id='mode-a' value='a'><input type='radio' name='mode' id='mode-b' value='b'></form></div><div id='out'></div><script>const sheets = document.styleSheets; sheets.forEach((sheet, index, list) => { document.getElementById('out').textContent += String(index) + ':' + String(sheet) + ':' + String(list) + '|'; }); const rules = sheets.item(0).cssRules; rules.forEach((rule, index, list) => { document.getElementById('out').textContent += String(index) + ':' + rule.selectorText + ':' + String(list) + '|'; }); const named = document.getElementById('signup').elements.namedItem('mode'); named.forEach((control, index, list) => { document.getElementById('out').textContent += String(index) + ':' + control.getAttribute('id') + ':' + String(list) + '|'; });</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "0:[object CSSStyleSheet]:[object StyleSheetList]|1:[object CSSStyleSheet]:[object StyleSheetList]|0:.primary:[object CSSRuleList]|1:.secondary:[object CSSRuleList]|0:mode-a:[object RadioNodeList]|1:mode-b:[object RadioNodeList]|",
    );
}

test "contract: Harness.fromHtml runs table.rows and tr.cells during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<table id='table'><thead id='head'><tr id='head-row'><th id='head-cell'>H</th></tr></thead><tbody id='body'><tr id='first-row'><td id='first-cell'>A</td></tr></tbody><tfoot id='foot'><tr id='foot-row'><td id='foot-cell'>F</td></tr></tfoot></table><div id='out'></div><script>const table = document.getElementById('table'); const body = document.getElementById('body'); const row = document.getElementById('first-row'); const rows = table.rows; const bodyRows = body.rows; const cells = row.cells; const before = String(rows.length) + ':' + String(bodyRows.length) + ':' + String(cells.length) + ':' + String(rows.namedItem('first-row')) + ':' + String(cells.namedItem('first-cell')); body.innerHTML = body.innerHTML + '<tr id=\"second-row\"><td id=\"second-cell\">B</td><td id=\"third-cell\">C</td></tr>'; row.append(document.getElementById('third-cell')); document.getElementById('out').textContent = before + '|' + String(rows.length) + ':' + String(bodyRows.length) + ':' + String(cells.length) + ':' + String(rows.namedItem('second-row')) + ':' + String(bodyRows.namedItem('second-row')) + ':' + String(cells.namedItem('third-cell'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "3:1:1:[object Element]:[object Element]|4:2:2:[object Element]:[object Element]:[object Element]",
    );
    try subject.assertExists("#second-row");
}

test "failure: Harness.fromHtml rejects non-table rows access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='bad'></div><script>document.getElementById('bad').rows.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed document.styleSheets cssRules access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>.broken { color: red;</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @page cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@page :first { margin: 1cm; }</style><div id='out'></div><script>const rule = document.styleSheets.item(0).cssRules.item(0); const style = rule.style; document.getElementById('out').textContent = String(rule) + ':' + rule.selectorText + ':' + String(style) + ':' + style.cssText + ':' + style.getPropertyValue('margin');</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSPageRule]::first:margin: 1cm;:margin: 1cm;:1cm",
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @layer cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@layer base { .primary { color: red; } .secondary { color: blue; } }</style><div id='out'></div><script>const rule = document.styleSheets.item(0).cssRules.item(0); const nested = rule.cssRules; document.getElementById('out').textContent = String(rule) + ':' + rule.nameText + ':' + String(nested.length) + ':' + nested.item(0).selectorText + ':' + nested.item(1).selectorText + ':' + nested.item(0).cssText;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSLayerBlockRule]:base:2:.primary:.secondary:.primary { color: red; }",
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @layer statement cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@layer base, theme;</style><div id='out'></div><script>const rule = document.styleSheets.item(0).cssRules.item(0); document.getElementById('out').textContent = String(rule) + ':' + rule.nameText + ':' + rule.cssText;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSLayerStatementRule]:base, theme:@layer base, theme;",
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @counter-style cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@counter-style thumbs { system: cyclic; symbols: a b; negative: '-' '+'; prefix: pre; suffix: post; range: 1 3; pad: 2 0; fallback: decimal; speak-as: bullets; additive-symbols: 1 '*' 2 '**'; }</style><div id='out'></div><script>const rule = document.styleSheets.item(0).cssRules.item(0); document.getElementById('out').textContent = String(rule) + ':' + rule.name + ':' + rule.system + ':' + rule.symbols + ':' + rule.negative + ':' + rule.prefix + ':' + rule.suffix + ':' + rule.range + ':' + rule.pad + ':' + rule.fallback + ':' + rule.speakAs + ':' + rule.additiveSymbols + ':' + rule.cssText;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSCounterStyleRule]:thumbs:cyclic:a b:'-' '+':pre:post:1 3:2 0:decimal:bullets:1 '*' 2 '**':@counter-style thumbs { system: cyclic; symbols: a b; negative: '-' '+'; prefix: pre; suffix: post; range: 1 3; pad: 2 0; fallback: decimal; speak-as: bullets; additive-symbols: 1 '*' 2 '**'; }",
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @property cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@property --accent { syntax: \"<color>\"; inherits: false; initial-value: red; }</style><div id='out'></div><script>const rule = document.styleSheets.item(0).cssRules.item(0); document.getElementById('out').textContent = String(rule) + ':' + rule.name + ':' + rule.syntax + ':' + String(rule.inherits) + ':' + rule.initialValue + ':' + rule.cssText;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSPropertyRule]:--accent:\"<color>\":false:red:@property --accent { syntax: \"<color>\"; inherits: false; initial-value: red; }",
    );
}

test "contract: Harness.fromHtml runs document.styleSheets @scope cssRules during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>@scope (.root) to (.leaf) { .primary { color: red; } .secondary { color: blue; } }</style><div id='out'></div><script>const rule = document.styleSheets.item(0).cssRules.item(0); const nested = rule.cssRules; document.getElementById('out').textContent = String(rule) + ':' + rule.start + ':' + rule.end + ':' + String(nested.length) + ':' + nested.item(0).selectorText + ':' + nested.item(1).selectorText + ':' + nested.item(0).cssText + ':' + rule.cssText;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object CSSScopeRule]:.root:.leaf:2:.primary:.secondary:.primary { color: red; }:@scope (.root) to (.leaf) { .primary { color: red; } .secondary { color: blue; } }",
    );
}

test "failure: Harness.fromHtml rejects malformed document.styleSheets @page access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@page :first { margin: 1cm;</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed document.styleSheets @scope access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@scope (.root) to (.leaf) { .primary { color: red; } .secondary { color: blue; }</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed document.styleSheets @layer access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@layer base { .primary { color: red; } .secondary { color: blue; }</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed document.styleSheets @layer statement access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@layer ;</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed document.styleSheets @counter-style access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@counter-style thumbs { system: cyclic; symbols: '*'; suffix: ' ';</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed document.styleSheets @property access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@property --accent { inherits: false; initial-value: red; }</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed document.styleSheets @font-palette-values access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@font-palette-values palette { base-palette: light; }</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed document.styleSheets @color-profile access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@color-profile swopc { src: url(http://example.org/swop-coated.icc); }</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects CSSRule.type mutations" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>.primary { color: red; }</style><script>document.styleSheets.item(0).cssRules.item(0).type = 2;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects CSSRule.parentStyleSheet mutations" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>.primary { color: red; }</style><script>document.styleSheets.item(0).cssRules.item(0).parentStyleSheet = null;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects CSSRule.parentRule mutations" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>.primary { color: red; }</style><script>document.styleSheets.item(0).cssRules.item(0).parentRule = null;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects CSSStyleRule.style mutations" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>.primary { color: red; }</style><script>document.styleSheets.item(0).cssRules.item(0).style.cssText = 'color: blue;';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects unsupported document.styleSheets at-rules" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@unknown-rule card { .x { color: red; } }</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed document.styleSheets @supports access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@supports (display: grid) { .broken { color: red; }</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed document.styleSheets @container access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@container card (min-width: 1px) { .broken { color: red; }</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed document.styleSheets @starting-style access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@starting-style { .broken { color: red; }</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed document.styleSheets @position-try access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@position-try card { .broken { color: red; }</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed document.styleSheets @keyframes access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@keyframes pulse { from { opacity: 0; }</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed document.styleSheets @font-face access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@font-face { font-family: x; src: url(x.woff);</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed document.styleSheets @font-feature-values access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@font-feature-values test { .x { color: red; }</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed document.styleSheets @import access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@import url(x.css</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed document.styleSheets @charset access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@charset UTF-8; .primary { color: red; }</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed document.styleSheets @namespace access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>@namespace svg url(http://www.w3.org/2000/svg</style><script>document.styleSheets.item(0).cssRules.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects invalid StyleSheetList forEach callback" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>.primary { color: red; }</style><script>document.styleSheets.forEach(1);</script>",
        ),
    );
}

test "contract: Harness.fromHtml runs CSSStyleSheet insertRule and deleteRule during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>.primary { color: red; }</style><div id='out'></div><script>const sheet = document.styleSheets.item(0); const before = sheet.cssRules.length; const inserted = sheet.insertRule('.secondary { color: blue; }'); const afterInsert = String(sheet.cssRules.length) + ':' + sheet.cssRules.item(0).selectorText + ':' + sheet.cssRules.item(1).selectorText + ':' + sheet.cssRules.item(1).cssText; sheet.deleteRule(0); document.getElementById('out').textContent = String(before) + ':' + String(inserted) + ':' + afterInsert + ':' + String(sheet.cssRules.length) + ':' + sheet.cssRules.item(0).selectorText;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "1:0:2:.secondary:.primary:.primary { color: red; }:1:.primary",
    );
}

test "contract: Harness.fromHtml runs CSSStyleSheet.replaceSync during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style id='sheet'>.primary { color: red; }</style><div id='out'></div><script>const sheet = document.styleSheets.item(0); const before = sheet.cssRules.item(0).cssText; sheet.replaceSync('.secondary { color: blue; }'); document.getElementById('out').textContent = before + '|' + String(sheet.cssRules.length) + ':' + sheet.cssRules.item(0).selectorText + ':' + sheet.cssRules.item(0).cssText + ':' + document.getElementById('sheet').textContent;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        ".primary { color: red; }|1:.secondary:.secondary { color: blue; }:.secondary { color: blue; }",
    );
}

test "contract: Harness.fromHtml runs CSSStyleSheet rules alias and addRule/removeRule during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<style>.primary { color: red; }</style><div id='out'></div><script>const sheet = document.styleSheets.item(0); const rules = sheet.rules; const before = String(rules.length) + ':' + rules.item(0).selectorText + ':' + String(rules); const inserted = sheet.addRule('.secondary', 'color: blue;'); const afterAdd = String(inserted) + ':' + String(rules.length) + ':' + rules.item(1).selectorText + ':' + rules.item(1).cssText; sheet.removeRule(0); document.getElementById('out').textContent = before + ':' + afterAdd + ':' + String(rules.length) + ':' + rules.item(0).selectorText;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "1:.primary:[object CSSRuleList]:1:2:.secondary:.secondary { color: blue; }:1:.secondary",
    );
}

test "failure: Harness.fromHtml rejects malformed CSSStyleSheet.insertRule syntax" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>.primary { color: red; }</style><script>document.styleSheets.item(0).insertRule('.broken { color: red;');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed CSSStyleSheet.replaceSync syntax" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>.primary { color: red; }</style><script>document.styleSheets.item(0).replaceSync('@unknown-rule card { color: red; }');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed CSSStyleSheet.addRule syntax" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>.primary { color: red; }</style><script>document.styleSheets.item(0).addRule('@unknown-rule card', 'color: red;');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-style CSSStyleSheet.replaceSync" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>.primary { color: red; }</style><link rel='stylesheet' href='a.css'><script>document.styleSheets.item(1).replaceSync('.secondary { color: blue; }');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-style CSSStyleSheet.addRule" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<style>.primary { color: red; }</style><link rel='stylesheet' href='a.css'><script>document.styleSheets.item(1).addRule('.secondary', 'color: blue;');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-row cells access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='bad'></div><script>document.getElementById('bad').cells.length;</script>",
        ),
    );
}

test "contract: Harness.fromHtml runs getElementsByTagName family during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='scope'><span id='first' class='alpha'>One</span><div id='class-target' class='alpha'>Two</div><svg id='icon'><foreignobject id='foreign'><div id='html' class='alpha beta'>Svg</div></foreignobject></svg><input id='named' name='search'></main><div id='before'></div><div id='after'></div><script>const scope = document.getElementById('scope'); const tags = scope.getElementsByTagName('span'); const classes = scope.getElementsByClassName('alpha beta'); const ns = scope.getElementsByTagNameNS('http://www.w3.org/2000/svg', '*'); const names = document.getElementsByName('search'); document.getElementById('before').textContent = String(tags.length) + ':' + String(classes.length) + ':' + String(ns.length) + ':' + String(names.length) + ':' + tags.item(0).getAttribute('id') + ':' + classes.item(0).getAttribute('id') + ':' + ns.item(0).getAttribute('id') + ':' + ns.namedItem('foreign').getAttribute('id') + ':' + names.item(0).getAttribute('id'); scope.innerHTML = scope.innerHTML + '<span id=\"second\" class=\"alpha beta\">Two</span><input id=\"second-named\" name=\"search\">'; document.getElementById('class-target').className = 'alpha beta'; document.getElementById('after').textContent = String(tags.length) + ':' + String(classes.length) + ':' + String(ns.length) + ':' + String(names.length) + ':' + String(tags.namedItem('second')) + ':' + String(classes.namedItem('class-target')) + ':' + names.item(1).getAttribute('id');</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#before", "1:1:2:1:first:html:icon:foreign:named");
    try subject.assertValue("#after", "2:3:2:2:[object Element]:[object Element]:second-named");
    try subject.assertExists("#second");
    try subject.assertExists("#second-named");
}

test "failure: Harness.fromHtml rejects getElementsByTagName argument counts" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='scope'></main><script>document.getElementsByTagName();</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects getElementsByTagNameNS argument counts" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='scope'></main><script>document.getElementsByTagNameNS('http://www.w3.org/1999/xhtml');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects getElementsByClassName argument counts" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='scope'></main><script>document.getElementsByClassName();</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects getElementsByName argument counts" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='scope'></main><script>document.getElementsByName();</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects getElementsByName on element receivers" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='scope'></main><script>document.getElementById('scope').getElementsByName('search');</script>",
        ),
    );
}

test "contract: Harness.fromHtml runs collection iterator helpers during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><span class='item'>One</span><span class='item'>Two</span><script id='first-script'></script><script name='named-script'></script></main><div id='out'></div><script id='trailing-script'>const nodes = document.querySelectorAll('.item'); const nodeValues = nodes.values(); const nodeKeys = nodes.keys(); const scripts = document.scripts; const scriptValues = scripts.values(); const scriptKeys = scripts.keys(); document.getElementById('root').textContent = 'gone'; const firstNode = nodeValues.next(); const secondNode = nodeValues.next(); const thirdNode = nodeValues.next(); const firstNodeKey = nodeKeys.next(); const secondNodeKey = nodeKeys.next(); const thirdNodeKey = nodeKeys.next(); const firstScript = scriptValues.next(); const secondScript = scriptValues.next(); const thirdScript = scriptValues.next(); const firstScriptKey = scriptKeys.next(); const secondScriptKey = scriptKeys.next(); const thirdScriptKey = scriptKeys.next(); document.getElementById('out').textContent = String(nodes.length) + ':' + String(scripts.length) + ':' + firstNode.value.textContent + ':' + String(firstNode.done) + ':' + secondNode.value.textContent + ':' + String(secondNode.done) + ':' + String(thirdNode.done) + ':' + String(firstNodeKey.value) + ':' + String(secondNodeKey.value) + ':' + String(thirdNodeKey.done) + ':' + firstScript.value.getAttribute('id') + ':' + String(firstScript.done) + ':' + secondScript.value.getAttribute('name') + ':' + String(secondScript.done) + ':' + thirdScript.value.getAttribute('id') + ':' + String(thirdScript.done) + ':' + String(firstScriptKey.value) + ':' + String(secondScriptKey.value) + ':' + String(thirdScriptKey.value) + ':' + String(thirdScriptKey.done);</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "2:1:One:false:Two:false:true:0:1:true:first-script:false:named-script:false:trailing-script:false:0:1:2:false",
    );
}

test "contract: Harness.fromHtml runs document.children during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><span>First</span></main><div id='out'></div><script>const children = document.children; const before = children.length; const first = children.item(0); const root = children.namedItem('root'); document.getElementById('root').remove(); document.getElementById('out').textContent = String(before) + ':' + String(children.length) + ':' + String(first) + ':' + String(root) + ':' + String(children.namedItem('root'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "3:2:[object Element]:[object Element]:null");
}

test "contract: Harness.fromHtml runs Element.children iterator helpers during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><span id='first'>First</span><span id='second'>Second</span></main><div id='out'></div><script>const children = document.getElementById('root').children; const childValues = children.values(); const childKeys = children.keys(); children.forEach((child, index, list) => { document.getElementById('out').textContent += 'F' + String(index) + ':' + child.textContent + ':' + String(list.length) + ';'; }, null); document.getElementById('root').textContent = 'gone'; const firstChild = childValues.next(); const secondChild = childValues.next(); const thirdChild = childValues.next(); const firstKey = childKeys.next(); const secondKey = childKeys.next(); const thirdKey = childKeys.next(); document.getElementById('out').textContent += '|' + firstChild.value.textContent + ':' + String(firstChild.done) + ':' + secondChild.value.textContent + ':' + String(secondChild.done) + ':' + String(thirdChild.done) + ':' + String(firstKey.value) + ':' + String(secondKey.value) + ':' + String(thirdKey.done) + ':' + String(children.length) + ':' + String(children.item(0));</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "F0:First:2;F1:Second:2;|First:false:Second:false:true:0:1:true:0:null",
    );
}

test "contract: Harness.fromHtml runs document.childNodes during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<!--pre--><main id='root'>Hello<span>World</span><!--tail--></main><div id='out'></div><script>const docNodes = document.childNodes; const rootNodes = document.getElementById('root').childNodes; const root = document.getElementById('root'); const docFirst = docNodes.item(0); const docSecond = docNodes.item(1); const rootValues = rootNodes.values(); const firstRoot = rootValues.next(); const secondRoot = rootValues.next(); const thirdRoot = rootValues.next(); root.innerHTML += '<span id=\"second\">Second</span>'; document.getElementById('out').textContent = String(docNodes.length) + ':' + docFirst.nodeName + ':' + String(docFirst.nodeType) + ':' + String(docFirst) + ':' + docSecond.nodeName + ':' + String(docSecond.nodeType) + ':' + firstRoot.value.nodeName + ':' + String(firstRoot.value.nodeType) + ':' + firstRoot.value.textContent + ':' + secondRoot.value.nodeName + ':' + String(secondRoot.value.nodeType) + ':' + secondRoot.value.textContent + ':' + thirdRoot.value.nodeName + ':' + String(thirdRoot.value.nodeType) + ':' + thirdRoot.value.textContent + ':' + String(rootNodes.length) + ':' + String(root.children.length) + ':' + root.children.item(1).textContent + ':' + root.children.namedItem('second').textContent;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "4:#comment:8:[object Node]:main:1:#text:3:Hello:span:1:World:#comment:8:tail:4:2:Second:Second",
    );
}

test "contract: Harness.fromHtml runs template.content childNodes during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const tpl = document.getElementById('tpl'); const content = tpl.content; const nodes = content.childNodes; const children = content.children; const before = nodes.length; tpl.innerHTML += '<!--tail--><span id=\"second\">Second</span>'; document.getElementById('out').textContent = String(content) + ':' + String(before) + ':' + String(nodes.length) + ':' + nodes.item(1).nodeName + ':' + String(children.length) + ':' + String(children.namedItem('second').textContent);</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "[object DocumentFragment]:1:3:#comment:2:Second");
}

test "contract: Harness.fromHtml runs template.content query methods during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<template id='tpl'><span id='inner'>Inner</span><span id='second' class='hit'>Second</span></template><div id='out'></div><script>const content = document.getElementById('tpl').content; const inner = content.getElementById('inner'); const missing = content.getElementById('tpl'); const query = content.querySelector('.hit'); const all = content.querySelectorAll('span'); document.getElementById('out').textContent = String(content) + ':' + String(inner) + ':' + inner.textContent + ':' + String(missing) + ':' + String(query) + ':' + query.textContent + ':' + String(all.length) + ':' + all.item(1).textContent;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object DocumentFragment]:[object Element]:Inner:null:[object Element]:Second:2:Second",
    );
    try subject.assertExists("#inner");
    try subject.assertExists("#second");
}

test "contract: Harness.fromHtml runs attribute reflection methods during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><button id='button'>First</button><input id='name'><input id='agree' type='checkbox'><select id='mode'><option value='a'>A</option><option id='selected' value='b'>B</option></select><div id='out'></div><script>document.getElementById('button').setAttribute('class', 'primary'); document.getElementById('out').textContent = String(document.querySelectorAll('.primary').length) + ':' + String(document.getElementById('button').hasAttribute('data-flag')) + ':'; document.getElementById('out').textContent += String(document.getElementById('button').toggleAttribute('data-flag')) + ':' + String(document.querySelectorAll('[data-flag]').length) + ':'; document.getElementById('out').textContent += String(document.getElementById('button').toggleAttribute('data-flag', false)) + ':' + String(document.querySelectorAll('[data-flag]').length) + ':'; document.getElementById('button').setAttribute('data-label', 'Hello'); document.getElementById('out').textContent += String(document.getElementById('button').getAttribute('data-label')) + ':'; document.getElementById('button').removeAttribute('data-label'); document.getElementById('out').textContent += String(document.getElementById('button').getAttribute('data-label')) + ':'; document.getElementById('name').setAttribute('value', 'Alice'); document.getElementById('agree').setAttribute('checked', ''); document.getElementById('selected').setAttribute('selected', ''); document.getElementById('out').textContent += document.getElementById('name').value + ':' + String(document.getElementById('agree').checked) + ':' + document.getElementById('mode').value;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "1:false:true:1:false:0:Hello:null:Alice:true:b");
    try subject.assertExists(".primary");
    try subject.assertValue("#name", "Alice");
    try subject.assertChecked("#agree", true);
    try subject.assertValue("#mode", "b");
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

test "contract: Harness.fromHtml runs form.noValidate and formNoValidate during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const form = document.createElement('form'); const button = document.createElement('button'); const input = document.createElement('input'); button.type = 'submit'; input.type = 'submit'; form.appendChild(button); form.appendChild(input); document.getElementById('root').appendChild(form); const before = String(form.noValidate) + ':' + String(button.formNoValidate) + ':' + String(input.formNoValidate); form.noValidate = true; button.formNoValidate = true; input.formNoValidate = true; const during = String(form.noValidate) + ':' + String(button.formNoValidate) + ':' + String(input.formNoValidate) + ':' + String(form.getAttribute('novalidate')) + ':' + String(button.getAttribute('formnovalidate')) + ':' + String(input.getAttribute('formnovalidate')); form.noValidate = false; button.formNoValidate = false; input.formNoValidate = false; const after = String(form.noValidate) + ':' + String(button.formNoValidate) + ':' + String(input.formNoValidate) + ':' + String(form.hasAttribute('novalidate')) + ':' + String(button.hasAttribute('formnovalidate')) + ':' + String(input.hasAttribute('formnovalidate')); document.getElementById('out').textContent = before + '|' + during + '|' + after;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "false:false:false|true:true:true:::|false:false:false:false:false:false");
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

test "contract: Harness.fromHtml runs form.reset during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><form id='form'><input id='name' value='Ada'></form><div id='out'></div><script>const form = document.getElementById('form'); form.addEventListener('reset', (event) => { event.preventDefault(); document.getElementById('out').textContent = 'reset:' + String(event.bubbles) + ':' + String(event.cancelable); }); form.reset();</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "reset:true:true");
}

test "contract: Harness.fromHtml exposes form owner reflection during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><form id='owner'></form><div id='host'><input id='input' form='owner'><button id='button' form='owner'></button><fieldset id='fieldset' form='owner'></fieldset><select id='select' form='owner'><optgroup id='group'><option id='option'>A</option></optgroup></select><output id='output' form='owner'></output><meter id='meter' form='owner'></meter><progress id='progress' form='owner'></progress></div><div id='out'></div><script>const input = document.getElementById('input'); const button = document.getElementById('button'); const fieldset = document.getElementById('fieldset'); const select = document.getElementById('select'); const group = document.getElementById('group'); const option = document.getElementById('option'); const output = document.getElementById('output'); const meter = document.getElementById('meter'); const progress = document.getElementById('progress'); const detached = document.createElement('input'); document.getElementById('out').textContent = input.form.id + ':' + button.form.id + ':' + fieldset.form.id + ':' + select.form.id + ':' + group.form.id + ':' + option.form.id + ':' + output.form.id + ':' + meter.form.id + ':' + progress.form.id + ':' + String(detached.form);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "owner:owner:owner:owner:owner:owner:owner:owner:owner:null");
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

test "contract: Harness.fromHtml runs Element.willValidate during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const field = document.createElement('input'); field.required = true; const hidden = document.createElement('input'); hidden.type = 'hidden'; hidden.required = true; const area = document.createElement('textarea'); area.required = true; area.readOnly = true; const select = document.createElement('select'); select.required = true; const option = document.createElement('option'); option.value = 'a'; option.selected = true; select.appendChild(option); root.appendChild(field); root.appendChild(hidden); root.appendChild(area); root.appendChild(select); document.getElementById('out').textContent = String(field.willValidate) + ':' + String(hidden.willValidate) + ':' + String(area.willValidate) + ':' + String(select.willValidate) + ':' + String(field.checkValidity()) + ':' + String(hidden.checkValidity()) + ':' + String(area.checkValidity()) + ':' + String(select.checkValidity());</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "true:false:false:true:false:true:true:true");
}

test "contract: Harness.fromHtml runs Element.validity during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const required = document.createElement('input'); required.required = true; const short = document.createElement('input'); short.minLength = 4; short.value = 'abc'; const long = document.createElement('textarea'); long.maxLength = 3; long.textContent = 'abcd'; const low = document.createElement('input'); low.type = 'number'; low.min = '2'; low.value = '1'; const high = document.createElement('input'); high.type = 'number'; high.min = '2'; high.max = '6'; high.value = '7'; const stepper = document.createElement('input'); stepper.type = 'number'; stepper.step = '2'; stepper.value = '3'; const bad = document.createElement('input'); bad.type = 'number'; bad.value = 'abc'; const check = document.createElement('input'); check.type = 'checkbox'; check.required = true; const select = document.createElement('select'); select.required = true; const hidden = document.createElement('input'); hidden.type = 'hidden'; hidden.required = true; const custom = document.createElement('input'); custom.value = 'ok'; custom.setCustomValidity('bad'); root.appendChild(required); root.appendChild(short); root.appendChild(long); root.appendChild(low); root.appendChild(high); root.appendChild(stepper); root.appendChild(bad); root.appendChild(check); root.appendChild(select); root.appendChild(hidden); root.appendChild(custom); document.getElementById('out').textContent = String(required.validity.valid) + ':' + String(required.validity.valueMissing) + ':' + String(short.validity.tooShort) + ':' + String(long.validity.tooLong) + ':' + String(low.validity.rangeUnderflow) + ':' + String(high.validity.rangeOverflow) + ':' + String(stepper.validity.stepMismatch) + ':' + String(bad.validity.badInput) + ':' + String(check.validity.valueMissing) + ':' + String(select.validity.valueMissing) + ':' + String(hidden.validity.valid) + ':' + String(custom.validity.customError) + ':' + String(custom.validity.valid) + ':' + stepper.step + ':' + String(stepper.validity.valid) + ':' + low.min + ':' + high.max + ':' + String(required.validity);</script></main>",
    );
    defer subject.deinit();
    try subject.assertValue("#out", "false:true:true:true:true:true:true:true:true:true:true:true:false:2:false:2:6:[object ValidityState]");
}

test "contract: Harness.fromHtml runs Element.validity typeMismatch during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const email = document.createElement('input'); email.type = 'email'; email.value = 'not-an-email'; const validEmail = document.createElement('input'); validEmail.type = 'email'; validEmail.multiple = true; validEmail.value = 'ada@example.com, grace@example.com'; const url = document.createElement('input'); url.type = 'url'; url.value = 'https://example.com/path'; const badUrl = document.createElement('input'); badUrl.type = 'url'; badUrl.value = 'not a url'; root.appendChild(email); root.appendChild(validEmail); root.appendChild(url); root.appendChild(badUrl); document.getElementById('out').textContent = String(email.validity.typeMismatch) + ':' + String(email.checkValidity()) + ':' + String(validEmail.validity.typeMismatch) + ':' + String(validEmail.checkValidity()) + ':' + String(url.validity.typeMismatch) + ':' + String(url.checkValidity()) + ':' + String(badUrl.validity.typeMismatch) + ':' + String(badUrl.checkValidity());</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "true:false:false:true:false:true:true:false");
}

test "contract: Harness.fromHtml runs Element.disabled and required during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const button = document.createElement('button'); button.id = 'button'; const input = document.createElement('input'); input.id = 'field'; root.appendChild(button); root.appendChild(input); const beforeDisabled = button.disabled; const beforeRequired = input.required; button.disabled = true; input.required = true; const duringDisabled = button.disabled; const duringRequired = input.required; const disabledMatches = document.querySelectorAll(':disabled').length; const requiredMatches = document.querySelectorAll(':required').length; button.disabled = false; input.required = false; document.getElementById('out').textContent = String(beforeDisabled) + ':' + String(beforeRequired) + ':' + String(duringDisabled) + ':' + String(duringRequired) + ':' + String(disabledMatches) + ':' + String(requiredMatches) + ':' + String(button.disabled) + ':' + String(input.required) + ':' + String(document.querySelectorAll(':disabled').length) + ':' + String(document.querySelectorAll(':required').length);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "false:false:true:true:1:1:false:false:0:0");
}

test "contract: Harness.fromHtml runs Element.autocomplete during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const input = document.createElement('input'); const before = input.autocomplete; input.autocomplete = 'email'; const during = input.autocomplete; document.getElementById('out').textContent = before + ':' + during + ':' + input.autocomplete + ':' + input.getAttribute('autocomplete');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", ":email:email:email");
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

test "contract: Harness.fromHtml runs setRangeText on selection controls during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.setSelectionRange(4, 12); name.setRangeText('Byron', 4, 12, 'select'); document.getElementById('out').textContent = name.value + ':' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "Ada Byron:4:9:none");
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

test "failure: Harness.fromHtml rejects selection setters on unsupported controls" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><input id='check' type='checkbox'></main><script>document.getElementById('check').setSelectionRange(0, 1);</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects selectionchange handlers on unsupported targets" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.onselectionchange = 1;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects setRangeText on unsupported controls" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').setRangeText('x');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects valueAsNumber setters on unsupported controls" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><input id='text' type='text'></main><script>document.getElementById('text').valueAsNumber = 1;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects valueAsDate setters on unsupported controls" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><input id='text' type='text'></main><script>document.getElementById('text').valueAsDate = new Date(0);</script>",
        ),
    );
}

test "contract: Harness.fromHtml runs tree mutation append, prepend, and remove during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='target'></section><button id='first'>First</button><button id='second'>Second</button><button id='third'>Third</button><div id='out'></div><script>document.getElementById('target').append(document.getElementById('first'), document.getElementById('second')); document.getElementById('target').prepend(document.getElementById('third')); document.getElementById('first').remove(); document.getElementById('out').textContent = document.getElementById('target').textContent + ':' + String(document.querySelectorAll('#target > button').length) + ':' + document.querySelector('#target > #third').textContent + ':' + document.querySelector('#target > #second').textContent;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "ThirdSecond:2:Third:Second");
    try subject.assertExists("#target > #third");
    try subject.assertExists("#target > #second");
    try std.testing.expectError(error.AssertionFailed, subject.assertExists("#first"));
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

test "failure: Harness.fromHtml rejects element.attributes keys arguments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>document.documentElement.attributes.keys(1);</script>"),
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

test "contract: Harness.fromHtml runs Text.wholeText during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='host'>Hello</div><div id='out'></div><script>const text = document.getElementById('host').childNodes.item(0); const split = text.splitText(3); document.getElementById('out').textContent = text.wholeText + ':' + split.wholeText + ':' + text.data + ':' + split.data + ':' + String(text.length) + ':' + String(split.length);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "Hello:Hello:Hel:lo:3:2");
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

test "contract: Harness.fromHtml runs insertAdjacentHTML during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section></main><div id='out'></div><script>document.getElementById('target').insertAdjacentHTML('beforebegin', '<aside id=\"before\">Before</aside>'); document.getElementById('target').insertAdjacentHTML('afterbegin', '<span id=\"first\">First</span>'); document.getElementById('target').insertAdjacentHTML('beforeend', '<span id=\"last\">Last</span>'); document.getElementById('target').insertAdjacentHTML('afterend', '<aside id=\"after\">After</aside>'); document.getElementById('out').textContent = document.getElementById('root').innerHTML + '|' + document.getElementById('target').innerHTML + '|' + String(document.querySelectorAll('#target > span').length) + ':' + String(document.querySelector('#before')) + ':' + String(document.querySelector('#after'));</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "<aside id=\"before\">Before</aside><section id=\"target\"><span id=\"first\">First</span><button class=\"primary\" id=\"old\">Old</button><span id=\"last\">Last</span></section><aside id=\"after\">After</aside>|<span id=\"first\">First</span><button class=\"primary\" id=\"old\">Old</button><span id=\"last\">Last</span>|2:[object Element]:[object Element]",
    );
    try subject.assertExists("#before");
    try subject.assertExists("#after");
    try subject.assertExists("#target > #first");
    try subject.assertExists("#target > #last");
}

test "contract: Harness.fromHtml runs template.content.innerHTML during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>document.getElementById('out').textContent = String(document.getElementById('tpl').content) + '|' + document.getElementById('tpl').content.innerHTML; document.getElementById('tpl').content.innerHTML = '<!--tail--><span id=\"second\">Second</span>'; document.getElementById('out').textContent += '|' + String(document.getElementById('tpl').content) + '|' + document.getElementById('tpl').content.innerHTML;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object DocumentFragment]|<span id=\"inner\">Inner</span>|[object DocumentFragment]|<!--tail--><span id=\"second\">Second</span>",
    );
    try subject.assertExists("#second");
    try std.testing.expectError(error.AssertionFailed, subject.assertExists("#inner"));
}

test "contract: Harness.fromHtml runs template.content element boundaries during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<template id='tpl'><span id='first'>First</span><em id='middle'>Middle</em>text<b id='last'>Last</b></template><div id='out'></div><script>const content = document.getElementById('tpl').content; const before = content.firstElementChild.id + ':' + content.lastElementChild.id + ':' + String(content.childElementCount) + ':' + String(content.children.length); content.innerHTML = '<span id=\"second\">Second</span>text<b id=\"third\">Third</b>'; document.getElementById('out').textContent = before + '|' + content.firstElementChild.id + ':' + content.lastElementChild.id + ':' + String(content.childElementCount) + ':' + String(content.children.length) + '|' + content.innerHTML;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "first:last:3:3|second:third:2:2|<span id=\"second\">Second</span>text<b id=\"third\">Third</b>",
    );
}

test "contract: Harness.fromHtml runs document.open write writeln and close during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='out'></div><script>const opened = document.open(); document.write('<main id=\"root\"><div id=\"out\"></div><span id=\"name\">Ada</span>'); document.writeln('</main>'); document.close(); document.getElementById('out').textContent = String(opened) + ':' + document.getElementById('name').textContent + ':' + String(document.getElementById('root').nextSibling.nodeType);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "[object Document]:Ada:3",
    );
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

test "failure: Harness.fromHtml rejects unsupported insertAdjacentElement positions" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><section id='target'></section></main><script>document.getElementById('target').insertAdjacentElement('middle', document.createElement('span'));</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects insertAdjacentText on void elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><img id='image'></main><script>document.getElementById('image').insertAdjacentText('beforeend', 'Bad');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects template.content on non-template elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').content;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed template.content.innerHTML fragments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.HtmlParse,
        Harness.fromHtml(
            allocator,
            "<template id='tpl'></template><script>document.getElementById('tpl').content.innerHTML = '<span></main>';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects document.open arity mismatches" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><script>document.open(1);</script></main>",
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

test "failure: Harness.fromHtml rejects insertAdjacentHTML on void elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><img id='image'></main><script>document.getElementById('image').insertAdjacentHTML('beforeend', '<span id=\"bad\">Bad</span>');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects detached insertAdjacentHTML" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><section id='target'><span id='old'>Old</span></section><section id='replacement'></section></main><script>document.getElementById('target').replaceChild(document.getElementById('replacement'), document.getElementById('old')).insertAdjacentHTML('beforebegin', '<aside id=\"before\">Before</aside>');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects document.createElement invalid names" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('bad tag');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects document.createTextNode arity mismatches" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode();</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects document.createComment arity mismatches" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createComment();</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects document.createDocumentFragment arity mismatches" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createDocumentFragment('extra');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects unsupported document.createElementNS namespaces" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElementNS('urn:test', 'div');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects invalid document.createAttributeNS qualified names" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createAttributeNS(null, 'svg:stroke');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects invalid document.createAttribute names" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createAttribute('svg:stroke');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects document.attributes on Document" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.attributes;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects invalid element attribute node removal" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.getElementById('root').removeAttributeNode(document.createAttribute('data-state'));</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects document.cloneNode" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.cloneNode();</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects normalize and importNode mismatches" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.normalize(1);</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><template id='tpl'></template><script>document.importNode(document.getElementById('tpl').ownerDocument, true);</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Node.contains arity and type mismatches" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><section id='outer'><span id='inside'>Inside</span></section><script>document.contains();</script></main>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><section id='outer'><span id='inside'>Inside</span></section><script>document.getElementById('outer').contains('nope');</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Node.compareDocumentPosition arity and type mismatches" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><section id='outer'><span id='inside'>Inside</span></section><script>document.compareDocumentPosition();</script></main>",
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
            "<main id='root'><section id='outer'><span id='inside'>Inside</span></section><script>document.isSameNode();</script></main>",
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
            "<main id='root'><section id='target'><button id='first'>First</button></section><script>document.getElementById('target').removeChild();</script></main>",
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

test "failure: Harness.fromHtml rejects unsupported querySelectorAll syntax" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.querySelectorAll('main::before');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed nth pseudo-class selectors" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><button id='first'>First</button></main><script>document.querySelector('button:nth-child(2 of )');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed :not, :is, and :where selectors" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.querySelector('button:not()');</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.querySelector('button:is()');</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.querySelector('button:where()');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed :has selectors" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.querySelector('main:has(>)');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed state pseudo-class selectors" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.querySelector('main:default()');</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.querySelector('main:indeterminate()');</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.querySelector('main:read-only()');</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.querySelector('main:read-write()');</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.querySelector('main:valid()');</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.querySelector('main:invalid()');</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.querySelector('main:in-range()');</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.querySelector('main:out-of-range()');</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.querySelector('main:defined()');</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.querySelector('main:blank()');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed :lang selectors" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.querySelector('main:lang()');</script>",
        ),
    );
}

test "failure: Harness.assertExists rejects malformed :dir selectors" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<main id='app'><span>Hello</span></main>");
    defer subject.deinit();

    try std.testing.expectError(error.InvalidSelector, subject.assertExists("main:dir(up)"));
}

test "failure: Harness.fromHtmlWithUrl surfaces invalid Location.href navigation" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.MockError,
        Harness.fromHtmlWithUrl(
            allocator,
            "https://example.test:8443/start?x#old",
            "<script>window.location.href = '   ';</script>",
        ),
    );
}

test "contract: Harness.fromHtml resolves scope pseudo-class selectors during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='section'><div id='child'>Child</div></section></main><div id='out'></div><script>const docScope = document.querySelector(':scope'); const root = document.getElementById('root'); const section = root.querySelector(':scope > section'); const missing = root.querySelector(':scope'); const matches = root.matches(':scope'); const closest = document.getElementById('child').closest(':scope'); document.getElementById('out').textContent = docScope.getAttribute('id') + ':' + section.getAttribute('id') + ':' + String(missing) + ':' + String(matches) + ':' + closest.getAttribute('id');</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "root:section:null:true:child");
}

test "contract: Harness.fromHtml resolves focus-visible pseudo-class selectors during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='panel'><input id='field'></section><div id='out'></div><script>const field = document.getElementById('field'); field.focus(); document.getElementById('out').textContent = String(field.matches(':focus')) + ':' + String(field.matches(':focus-visible')) + ':' + String(document.querySelectorAll(':focus-visible').length) + ':' + String(document.querySelector('#panel:focus-visible')) + ':' + String(document.querySelector('#root:focus-visible'));</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "true:true:1:null:null");
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
            "<main id='root'><div id='out'></div><script>document.blur();</script></main>",
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

test "contract: Harness.fromHtml resolves :lang and :dir pseudo-class selectors during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root' lang='en-US' dir='rtl'><section id='section'><span id='leaf' xml:lang='fr'>Bonjour</span></section></main><div id='out'></div><script>const langMatch = document.querySelector('main:lang(en)'); const inheritedLang = document.querySelector('section:lang(en-us)'); const dirMatch = document.querySelector('section:dir(rtl)'); const closest = document.getElementById('leaf').closest('main:dir(rtl)'); const matches = document.getElementById('section').matches(':dir(rtl)'); const leafLang = document.getElementById('leaf').matches(':lang(fr)'); document.getElementById('out').textContent = langMatch.getAttribute('id') + ':' + inheritedLang.getAttribute('id') + ':' + dirMatch.getAttribute('id') + ':' + closest.getAttribute('id') + ':' + String(matches) + ':' + String(leafLang);</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "root:section:section:root:true:true");
}

test "failure: Harness.fromHtml rejects non-function NodeList.forEach callbacks" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><button>First</button></main><script>document.querySelectorAll('button').forEach(123);</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects invalid document.scripts item indices" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><script id='first-script'></script></main><script>document.scripts.item('bad');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects children access on non-element nodes" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<!--pre--><main id='root'></main><script>document.childNodes.item(0).children.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects firstElementChild access on non-element nodes" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<!--pre--><main id='root'></main><script>document.childNodes.item(0).firstElementChild;</script>",
        ),
    );
}

test "contract: Harness.fromHtml runs collection entries during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><span class='item'>One</span><span class='item'>Two</span></main><div id='out'></div><script>const nodes = document.querySelectorAll('.item'); const nodeEntries = nodes.entries(); const children = document.getElementById('root').children; const childEntries = children.entries(); const firstNode = nodeEntries.next(); const secondNode = nodeEntries.next(); const firstChild = childEntries.next(); const secondChild = childEntries.next(); document.getElementById('out').textContent = String(firstNode.value.index) + ':' + firstNode.value.value.textContent + ':' + String(secondNode.value.index) + ':' + secondNode.value.value.textContent + ':' + String(firstChild.value.index) + ':' + firstChild.value.value.textContent + ':' + String(secondChild.value.index) + ':' + secondChild.value.value.textContent;</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "0:One:1:Two:0:One:1:Two");
}

test "failure: Harness.fromHtml rejects non-document anchors access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><a name='first'>First</a></main><script>document.getElementById('root').anchors.length;</script>",
        ),
    );
}

test "contract: Harness.fromHtml exposes anchor and area download reflection" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><a id='anchor' href='https://example.test/files/report.csv'>Anchor</a><map name='map'><area id='area' download='area.bin' href='https://example.test/files/diagram.png'></map><div id='out'></div><script>const anchor = document.getElementById('anchor'); const before = String(anchor.download); anchor.download = 'anchor.txt'; document.getElementById('out').textContent = before + '|' + anchor.download;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "|anchor.txt");
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().downloads().artifacts().len);

    try subject.click("#anchor");
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().downloads().artifacts().len);
    try std.testing.expectEqualStrings(
        "anchor.txt",
        subject.mocksMut().downloads().artifacts()[0].file_name,
    );
    try std.testing.expectEqualStrings(
        "https://example.test/files/report.csv",
        subject.mocksMut().downloads().artifacts()[0].bytes,
    );

    try subject.click("#area");
    try std.testing.expectEqual(@as(usize, 2), subject.mocksMut().downloads().artifacts().len);
    try std.testing.expectEqualStrings(
        "area.bin",
        subject.mocksMut().downloads().artifacts()[1].file_name,
    );
    try std.testing.expectEqualStrings(
        "https://example.test/files/diagram.png",
        subject.mocksMut().downloads().artifacts()[1].bytes,
    );
}

test "contract: Harness.fromHtml exposes anchor target reflection and area target click observation" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><a id='anchor' href='https://example.test/next' target='_blank'>Anchor</a><map name='map'><area id='area' target='popup' href='https://example.test/files/diagram.png'></map><div id='out'></div><script>const anchor = document.getElementById('anchor'); const before = String(anchor.target); anchor.target = 'reports'; document.getElementById('out').textContent = before + '|' + anchor.target;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "_blank|reports");
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
        "popup",
        subject.mocksMut().open().calls()[1].target.?,
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

test "failure: Harness.fromHtml rejects non-anchor download access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').download;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-anchor target access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='box'></div><script>document.getElementById('box').target;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-form elements access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='wrapper'><div id='not-form'></div></div><script>document.getElementById('wrapper').elements.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-select options access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='wrapper'><div id='not-select'></div></div><script>document.getElementById('not-select').options.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects datalist options add" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><select id='mode'><option id='first' value='a'>A</option></select><datalist id='list'><option id='extra' value='b'>B</option></datalist></main><script>document.getElementById('list').options.add(document.getElementById('extra'));</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-select selectedOptions access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='wrapper'><div id='not-select'></div></div><script>document.getElementById('not-select').selectedOptions.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-form length access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='wrapper'><div id='not-form'></div></div><script>document.getElementById('not-form').length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-select length access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='wrapper'><div id='not-select'></div></div><script>document.getElementById('not-select').length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects form.length assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<form id='signup'><input name='first' value='Ada'><textarea name='bio'>Bio</textarea></form><script>document.getElementById('signup').length = 0;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects select.length assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<select id='mode'><option>A</option><option>B</option></select><script>document.getElementById('mode').length = 0;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-fieldset elements access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='wrapper'><div id='not-fieldset'></div></div><script>document.getElementById('not-fieldset').elements.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-datalist options access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='wrapper'><div id='not-datalist'></div></div><script>document.getElementById('not-datalist').options.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-map areas access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='wrapper'><div id='not-map'></div></div><script>document.getElementById('not-map').areas.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-table tBodies access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='wrapper'><div id='not-table'></div></div><script>document.getElementById('not-table').tBodies.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-labelable labels access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='wrapper'><div id='not-labelable'></div></div><script>document.getElementById('not-labelable').labels.length;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-label control access" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='wrapper'><div id='not-label'></div></div><script>document.getElementById('not-label').control;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects non-label htmlFor assignment" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='wrapper'><div id='not-label'></div></div><script>document.getElementById('not-label').htmlFor = 'control';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed innerHTML fragments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.HtmlParse,
        Harness.fromHtml(
            allocator,
            "<main id='root'><section id='target'></section><script>document.getElementById('target').innerHTML = '<span></main>';</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects lossy outerHTML serialization" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.DomError,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='target'></div><script>document.getElementById('target').setAttribute('data-label', 'a\\'b\"c'); document.getElementById('target').outerHTML;</script></main>",
        ),
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

test "failure: Harness.fromHtml rejects empty attribute names in inline scripts" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><button id='button'></button><script>document.getElementById('button').setAttribute('', 'x');</script></main>",
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

test "failure: Harness.fromHtml rejects Element.id on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').id = 'text';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.title lang and dir on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').title = 'Tip';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.hidden on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').hidden = true;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.inert on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').inert = true;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.translate on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').translate = false;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.spellcheck on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').spellcheck = false;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.draggable on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').draggable = true;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.nonce on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').nonce = 'abc123';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.autocapitalize on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').autocapitalize = 'words';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.autofocus on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').autofocus = true;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.placeholder on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('button').placeholder = 'Name';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.name on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').name = 'first';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.selected on non-option elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').selected = true;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.defaultValue on non-form controls" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='out'></div><script>document.createElement('div').defaultValue = 'x';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.defaultChecked on non-form controls" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<div id='out'></div><script>document.createElement('div').defaultChecked = true;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects form.noValidate on non-forms" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').noValidate = true;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects formNoValidate on non-submit controls" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').formNoValidate = true;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects form action reflection on non-forms and non-submit controls" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').action = '/submit';</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').formAction = '/submit';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects form owner reflection on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').form;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects form owner assignment on supported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('input').form = null;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects requestSubmit on invalid submitters" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>const form = document.createElement('form'); const button = document.createElement('button'); button.type = 'button'; form.requestSubmit(button);</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects form submit methods on non-forms" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').submit();</script>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').requestSubmit();</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects form.reset on non-forms" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').reset();</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects select.value assignments that do not match an option" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>const select = document.createElement('select'); select.innerHTML = '<option value=\"a\">A</option>'; select.value = 'missing';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects select.selectedIndex on non-select elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').selectedIndex = 0;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.disabled on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').disabled = true;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.required on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').required = true;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.autocomplete on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').autocomplete = 'email';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.pattern on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').pattern = '[0-9]{3}';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.inputMode on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').inputMode = 'numeric';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.multiple on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').multiple = true;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.type on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').type = 'email';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.minLength on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').minLength = 2;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects checkValidity on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').checkValidity();</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects setCustomValidity on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').setCustomValidity('bad');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects validationMessage on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').validationMessage;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects willValidate on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').willValidate;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects validity on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').validity.valid;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects reportValidity on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').reportValidity();</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects min on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').min = '1';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects step on unsupported elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createElement('div').step = '1';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.readOnly on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').readOnly = true;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.accessKey on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').accessKey = 'k';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element ariaLabel on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').ariaLabel = 'Menu';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.slot on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').slot = 'primary';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.part on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').part;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.contentEditable on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').contentEditable = 'true';</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects Element.tabIndex on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').tabIndex = 1;</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects namespace-aware attribute reflection arity and non-element mismatches" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.createTextNode('x').setAttributeNS('urn:test', 'id', 'button');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects hasAttributes on non-elements" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.hasAttributes();</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects malformed style declaration syntax" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='box' style='color: red;'></div><script>document.getElementById('box').style.cssText = 'color red';</script></main>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='box'></div><script>document.getElementById('box').style.setProperty('bad name', 'x');</script></main>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='box'></div><script>document.getElementById('box').style.setProperty('color', 'red', 'urgent');</script></main>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='box'></div><script>document.getElementById('box').style.setProperty('background-image', 'red; background: blue');</script></main>",
        ),
    );
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><div id='box'></div><script>document.getElementById('box').style.getPropertyPriority('color', 'extra');</script></main>",
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
        "<form id='profile'><input id='name'><button id='submit' type='submit'>Save</button></form><div id='out'></div><script>document.getElementById('profile').addEventListener('submit', () => { document.getElementById('out').textContent = document.getElementById('name').value; });</script>",
    );
    defer subject.deinit();

    try subject.typeText("#name", "Alice");
    try subject.click("#submit");
    try subject.assertValue("#out", "Alice");
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

test "contract: Harness.submit dispatches submit directly" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<form id='profile'><input id='name'><button id='submit' type='submit'>Save</button></form><div id='out'></div><script>document.getElementById('profile').addEventListener('submit', () => { document.getElementById('out').textContent = document.getElementById('name').value; });</script>",
    );
    defer subject.deinit();

    try subject.typeText("#name", "Bob");
    try subject.submit("#profile");
    try subject.assertValue("#out", "Bob");
}

test "contract: Harness.dispatch runs custom listeners without default actions" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<button id='run'></button><div id='out'></div><script>document.getElementById('run').addEventListener('custom', () => { document.getElementById('out').textContent = 'custom'; });</script>",
    );
    defer subject.deinit();

    try subject.dispatch("#run", "custom");
    try subject.assertValue("#out", "custom");
}

test "contract: Harness.focus and Harness.blur dispatch in order" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<input id='first'><input id='second'><div id='out'></div><script>document.getElementById('first').addEventListener('blur', () => { document.getElementById('second').textContent = 'after-blur'; }); document.getElementById('second').addEventListener('focus', () => { document.getElementById('out').textContent = document.getElementById('second').textContent; });</script>",
    );
    defer subject.deinit();

    try subject.focus("#first");
    try subject.focus("#second");
    try subject.assertValue("#out", "after-blur");
}

test "contract: Harness.focus and Harness.blur dispatch focusin and focusout events" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='out'></main><section id='panel'><input id='first'><input id='second'></section><script>const panel = document.getElementById('panel'); panel.addEventListener('focusin', (event) => { document.getElementById('out').textContent += 'focusin|'; }); panel.addEventListener('focusout', (event) => { document.getElementById('out').textContent += 'focusout|'; }); document.getElementById('first').addEventListener('focus', (event) => { document.getElementById('out').textContent += 'focus:first|'; }); document.getElementById('first').addEventListener('blur', (event) => { document.getElementById('out').textContent += 'blur:first|'; }); document.getElementById('second').addEventListener('focus', (event) => { document.getElementById('out').textContent += 'focus:second|'; }); document.getElementById('second').addEventListener('blur', (event) => { document.getElementById('out').textContent += 'blur:second'; });</script>",
    );
    defer subject.deinit();

    try subject.focus("#first");
    try subject.focus("#second");
    try subject.assertValue(
        "#out",
        "focusin|focus:first|focusout|blur:first|focusin|focus:second|",
    );
}

test "contract: Harness.focus and Harness.blur dispatch window focus and blur events" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='out'></main><section id='panel'><input id='first'><input id='second'></section><script>window.addEventListener('focus', () => { document.getElementById('out').textContent += 'focus|'; }); window.onfocus = () => { document.getElementById('out').textContent += 'property-focus|'; }; window.addEventListener('blur', () => { document.getElementById('out').textContent += 'blur|'; }); window.onblur = () => { document.getElementById('out').textContent += 'property-blur'; };</script>",
    );
    defer subject.deinit();

    try subject.focus("#first");
    try subject.focus("#second");
    try subject.blur("#second");
    try subject.assertValue("#out", "focus|property-focus|blur|property-blur");
}

test "contract: Harness.fromHtml dispatches window load events after bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='out'></main><script>window.addEventListener('load', () => { document.getElementById('out').textContent += 'load|'; }); window.onload = () => { document.getElementById('out').textContent += 'property-load|'; }; window.addEventListener('pageshow', () => { document.getElementById('out').textContent += 'pageshow|'; }); window.onpageshow = () => { document.getElementById('out').textContent += 'property-pageshow'; };</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "load|property-load|pageshow|property-pageshow");
}

test "contract: Harness.focus and Harness.blur sync focus pseudo-classes" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='panel'><input id='field'></section></main>",
    );
    defer subject.deinit();

    try subject.focus("#field");
    try subject.assertExists("#field:focus");
    try subject.assertExists("#panel:focus-within");
    try subject.assertExists("#root:focus-within");

    try subject.blur("#field");
    try std.testing.expectError(error.AssertionFailed, subject.assertExists(":focus"));
}

test "contract: Harness.setSelectValue updates selection and fires change" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<select id='mode'><option value='a'>A</option><option value='b'>B</option></select><div id='out'></div><script>document.getElementById('mode').addEventListener('change', () => { document.getElementById('out').textContent = document.getElementById('mode').value; });</script>",
    );
    defer subject.deinit();

    try subject.setSelectValue("#mode", "b");
    try subject.assertValue("#mode", "b");
    try subject.assertValue("#out", "b");
}

test "failure: Harness.typeText rejects non-text form controls" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<input id='agree' type='checkbox'>");
    defer subject.deinit();

    try std.testing.expectError(error.DomError, subject.typeText("#agree", "Alice"));
}

test "failure: Harness.fetch rejects missing mocks" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<main></main>");
    defer subject.deinit();

    try std.testing.expectError(
        error.MockError,
        subject.fetch("https://example.test/api/missing"),
    );
}

test "failure: Harness.fetch honors injected fetch errors" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<main></main>");
    defer subject.deinit();

    try subject.mocksMut().fetch().fail("https://example.test/api/fail", "network disabled");

    try std.testing.expectError(
        error.MockError,
        subject.fetch("https://example.test/api/fail"),
    );
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().fetch().calls().len);
    try std.testing.expectEqualStrings(
        "https://example.test/api/fail",
        subject.mocksMut().fetch().calls()[0].url,
    );
}

test "failure: dialogs and clipboard reads require seeded values" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<main></main>");
    defer subject.deinit();

    try std.testing.expectError(error.MockError, subject.confirm("Continue?"));
    try std.testing.expectError(error.MockError, subject.prompt("Name?"));
    try std.testing.expectError(error.MockError, subject.readClipboard());
}

test "failure: Harness.captureDownload rejects blank file names" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<main></main>");
    defer subject.deinit();

    try std.testing.expectError(
        error.MockError,
        subject.captureDownload("   ", "downloaded bytes"),
    );
}

test "failure: HarnessBuilder.openFailure rejects bootstrap window.open" {
    const allocator = std.testing.allocator;
    var builder = Harness.builder(allocator);
    defer builder.deinit();

    _ = builder.html("<main id='out'></main><script>window.open('https://example.test/popup');</script>");
    _ = builder.openFailure("popup blocked");

    try std.testing.expectError(error.MockError, builder.build());
}

test "failure: HarnessBuilder.closeFailure rejects bootstrap window.close" {
    const allocator = std.testing.allocator;
    var builder = Harness.builder(allocator);
    defer builder.deinit();

    _ = builder.html("<main id='out'></main><script>window.close();</script>");
    _ = builder.closeFailure("window closed");

    try std.testing.expectError(error.MockError, builder.build());
}

test "failure: HarnessBuilder.printFailure rejects bootstrap window.print" {
    const allocator = std.testing.allocator;
    var builder = Harness.builder(allocator);
    defer builder.deinit();

    _ = builder.html("<main id='out'></main><script>window.print();</script>");
    _ = builder.printFailure("print blocked");

    try std.testing.expectError(error.MockError, builder.build());
}

test "failure: window.onbeforeprint rejects non-function assignments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.onbeforeprint = 1;</script>"),
    );
}

test "failure: HarnessBuilder.scrollFailure rejects bootstrap window.scrollTo" {
    const allocator = std.testing.allocator;
    var builder = Harness.builder(allocator);
    defer builder.deinit();

    _ = builder.html("<main id='out'></main><script>window.scrollTo(10, 20);</script>");
    _ = builder.scrollFailure("scroll blocked");

    try std.testing.expectError(error.MockError, builder.build());
}

test "failure: window.scrollTo rejects non-integer coordinates" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.scrollTo(1.5, 20);</script>"),
    );
}

test "failure: window.navigator.toString rejects arguments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.navigator.toString(1);</script>"),
    );
}

test "failure: window.performance.now rejects arguments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.performance.now(1);</script>"),
    );
}

test "failure: window.navigator appCodeName is read-only" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.navigator.appCodeName = 'x';</script>"),
    );
}

test "failure: window.navigator appName is read-only" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.navigator.appName = 'x';</script>"),
    );
}

test "failure: window.navigator appVersion is read-only" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.navigator.appVersion = 'x';</script>"),
    );
}

test "failure: window.navigator product is read-only" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.navigator.product = 'x';</script>"),
    );
}

test "failure: window.navigator productSub is read-only" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.navigator.productSub = 'x';</script>"),
    );
}

test "failure: window.navigator hardwareConcurrency is read-only" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.navigator.hardwareConcurrency = 16;</script>"),
    );
}

test "failure: window.navigator vendor is read-only" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.navigator.vendor = 'x';</script>"),
    );
}

test "failure: window.navigator vendorSub is read-only" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.navigator.vendorSub = 'x';</script>"),
    );
}

test "failure: window.navigator pdfViewerEnabled is read-only" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.navigator.pdfViewerEnabled = true;</script>"),
    );
}

test "failure: window.navigator doNotTrack is read-only" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.navigator.doNotTrack = '1';</script>"),
    );
}

test "failure: window.navigator plugins is read-only" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.navigator.plugins = null;</script>"),
    );
}

test "failure: window.navigator.plugins.refresh rejects extra arguments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.navigator.plugins.refresh(1, 2);</script>"),
    );
}

test "failure: window.navigator mimeTypes is read-only" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.navigator.mimeTypes = null;</script>"),
    );
}

test "failure: window.navigator languages is read-only" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.navigator.languages = null;</script>"),
    );
}

test "failure: window.navigator languages contains rejects arguments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.navigator.languages.contains();</script>"),
    );
}

test "failure: window.navigator languages keys rejects arguments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.navigator.languages.keys(1);</script>"),
    );
}

test "failure: window.navigator javaEnabled rejects arguments" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.navigator.javaEnabled(1);</script>"),
    );
}

test "failure: window.devicePixelRatio is read-only" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.devicePixelRatio = 2;</script>"),
    );
}

test "failure: window.top is read-only" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.top = null;</script>"),
    );
}

test "failure: window.screenX is read-only" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>window.screenX = 1;</script>"),
    );
}

test "contract: Harness.setFiles updates selection and fires change" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<input id='upload' type='file'><div id='out'></div><script>document.getElementById('upload').addEventListener('change', () => { document.getElementById('out').textContent = document.getElementById('upload').value; });</script>",
    );
    defer subject.deinit();

    try subject.setFiles("#upload", &.{"report.csv"});
    try subject.assertValue("#upload", "report.csv");
    try subject.assertValue("#out", "report.csv");
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().fileInput().selections().len);
    try std.testing.expectEqualStrings("#upload", subject.mocksMut().fileInput().selections()[0].selector);
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().fileInput().selections()[0].files.len);
    try std.testing.expectEqualStrings("report.csv", subject.mocksMut().fileInput().selections()[0].files[0]);
}

test "failure: Harness.setFiles rejects non-file inputs" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(allocator, "<input id='name'>");
    defer subject.deinit();

    try std.testing.expectError(
        error.DomError,
        subject.setFiles("#name", &.{"report.csv"}),
    );
}

test "contract: Harness.fromHtml exposes input.files on file inputs" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<input id='upload' type='file'><div id='out'></div><script>const upload = document.getElementById('upload'); document.getElementById('out').textContent = String(upload.files.length) + ':' + String(upload.files.item(0)); upload.addEventListener('change', () => { const current = document.getElementById('upload'); document.getElementById('out').textContent = String(current.files.length) + ':' + current.files.item(0) + ':' + current.files.item(1) + ':' + String(current.files); });</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "0:null");
    try subject.setFiles("#upload", &.{ "report.csv", "logs.txt" });
    try subject.assertValue("#out", "2:report.csv:logs.txt:[object FileList]");
}

test "failure: Harness.fromHtml rejects input.files on unsupported inputs" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>document.createElement('input').files;</script>"),
    );
}

test "failure: Harness.fromHtml rejects document.innerText on Document" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>document.innerText;</script>"),
    );
}

test "failure: Harness.fromHtml rejects document.outerText on Document" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(allocator, "<script>document.outerText = 'x';</script>"),
    );
}

test "contract: Harness.fromHtml exposes document.documentElement, head, body, and title aliases" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<html id='html'><head id='head'><title>Initial</title></head><body id='body'><main id='out'></main><script>document.title = 'Updated'; const html = document.documentElement; const head = document.head; const body = document.body; document.getElementById('out').textContent = html.getAttribute('id') + ':' + head.getAttribute('id') + ':' + body.getAttribute('id') + ':' + document.title;</script></body></html>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "html:head:body:Updated",
    );
}

test "contract: Harness.fromHtml exposes node and element reflection helpers" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<!--pre--><main id='root'><span id='child'></span><!--tail--></main><div id='out'></div><script>const doc = document; const root = document.getElementById('root'); const child = document.getElementById('child'); const comment = document.childNodes.item(0); document.getElementById('out').textContent = String(doc.ownerDocument) + ':' + String(doc.parentNode) + ':' + String(doc.parentElement) + ':' + String(doc.firstElementChild) + ':' + String(doc.lastElementChild) + ':' + String(doc.childElementCount) + ':' + String(root.ownerDocument) + ':' + String(root.parentNode) + ':' + String(root.parentElement) + ':' + String(root.firstElementChild) + ':' + String(root.lastElementChild) + ':' + String(root.childElementCount) + ':' + String(child.ownerDocument) + ':' + String(child.parentNode) + ':' + String(child.parentElement) + ':' + String(comment.ownerDocument) + ':' + String(comment.parentNode) + ':' + String(comment.parentElement);</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "null:null:null:[object Element]:[object Element]:3:[object Document]:[object Document]:null:[object Element]:[object Element]:1:[object Document]:[object Element]:[object Element]:[object Document]:[object Document]:null",
    );
}

test "contract: Harness.fromHtml exposes element.innerText as a text alias" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section id='panel'><span>One</span><span>Two</span></section><div id='out'></div><script>const panel = document.getElementById('panel'); const before = panel.innerText; panel.innerText = 'Reset'; document.getElementById('out').textContent = before + ':' + panel.innerText + ':' + panel.textContent;</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "OneTwo:Reset:Reset");
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

test "contract: Harness.fromHtmlWithUrl exposes window.navigator languages and legacy aliases" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='out'></main><script>const languages = window.navigator.languages; const keys = languages.keys(); const values = languages.values(); const entries = languages.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); document.getElementById('out').textContent = window.navigator.userLanguage + ':' + window.navigator.browserLanguage + ':' + window.navigator.systemLanguage + ':' + window.navigator.oscpu + ':' + String(languages.length) + ':' + languages.item(0) + ':' + languages.toString() + ':' + String(languages.contains('en-US')) + ':' + String(languages.contains('fr-FR')) + ':' + String(firstKey.value) + ':' + firstValue.value + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "en-US:en-US:en-US:unknown:1:en-US:[object DOMStringList]:true:false:0:en-US:0:en-US",
    );
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
        "<main id='out'></main><script>const mimeTypes = window.navigator.mimeTypes; const keys = mimeTypes.keys(); const values = mimeTypes.values(); const entries = mimeTypes.entries(); document.getElementById('out').textContent = String(mimeTypes) + ':' + mimeTypes.toString() + ':' + String(mimeTypes.length) + ':' + String(mimeTypes.item(0)) + ':' + String(mimeTypes.namedItem('text/plain')) + ':' + String(keys.next().done) + ':' + String(values.next().done) + ':' + String(entries.next().done);</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "[object MimeTypeArray]:[object MimeTypeArray]:0:null:null:true:true:true");
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
