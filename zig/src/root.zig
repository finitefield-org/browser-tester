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
        "<html id='html'><head><title>Example</title></head><body id='body'><main id='out'></main><script>const metadata = document.compatMode + ':' + document.characterSet + ':' + document.charset + ':' + document.contentType; const active = document.activeElement.getAttribute('id'); const documentChildren = document.children; const windowChildren = window.children; document.getElementById('out').textContent = metadata + ':' + active + ':' + String(documentChildren.length) + ':' + String(windowChildren.length) + ':' + documentChildren.item(0).getAttribute('id') + ':' + windowChildren.item(0).getAttribute('id');</script></body></html>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "CSS1Compat:UTF-8:UTF-8:text/html:body:1:1:html:html");
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

test "contract: Harness.fromHtml resolves valid, invalid, in-range, and out-of-range pseudo-classes during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><input id='filled' type='text' required value='Ada'><input id='empty' type='text' required><input id='check' type='checkbox' required><input id='check-ok' type='checkbox' required checked><input id='low' type='number' min='2' max='6' value='1'><input id='high' type='number' min='2' max='6' value='7'><input id='in-range' type='number' min='2' max='6' value='4'><textarea id='bio' required></textarea><select id='mode' required><option value='a' selected>A</option><option value='b'>B</option></select><button id='button'>Button</button></main><div id='out'></div><script>const valid = document.querySelectorAll(':valid'); const invalid = document.querySelectorAll(':invalid'); const inRange = document.querySelectorAll(':in-range'); const outOfRange = document.querySelectorAll(':out-of-range'); document.getElementById('out').textContent = String(valid.length) + ':' + valid.item(0).getAttribute('id') + ':' + valid.item(1).getAttribute('id') + ':' + valid.item(2).getAttribute('id') + ':' + valid.item(3).getAttribute('id') + ':' + String(invalid.length) + ':' + invalid.item(0).getAttribute('id') + ':' + invalid.item(1).getAttribute('id') + ':' + invalid.item(2).getAttribute('id') + ':' + invalid.item(3).getAttribute('id') + ':' + invalid.item(4).getAttribute('id') + ':' + String(inRange.length) + ':' + inRange.item(0).getAttribute('id') + ':' + String(outOfRange.length) + ':' + outOfRange.item(0).getAttribute('id') + ':' + outOfRange.item(1).getAttribute('id');</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "4:filled:check-ok:in-range:mode:5:empty:check:low:high:bio:1:in-range:2:low:high",
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

test "contract: Harness.fromHtml runs class and dataset views during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><button id='button' class='base' data-kind='App'>First</button><div id='out'></div><script>document.getElementById('button').className = 'primary secondary'; document.getElementById('button').classList.add('tertiary'); document.getElementById('button').classList.remove('secondary'); document.getElementById('button').dataset.userId = '42'; document.getElementById('out').textContent = String(document.getElementById('button').classList.length) + ':' + String(document.getElementById('button').classList.contains('primary')) + ':' + String(document.getElementById('button').classList.toggle('active')) + ':' + document.getElementById('button').className + ':' + document.getElementById('button').dataset.kind + ':' + document.getElementById('button').dataset.userId + ':' + String(document.getElementById('button').classList) + ':' + String(document.getElementById('button').dataset);</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:true:true:primary tertiary active:App:42:[object DOMTokenList]:[object DOMStringMap]");
    try subject.assertExists(".active");
    try subject.assertExists("[data-user-id]");
    try subject.assertExists("[data-kind=App]");
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

test "contract: Harness.fromHtml reports style property priorities during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><div id='box' style='color: red !important; background-color: white;'></div><div id='out'></div><script>const style = document.getElementById('box').style; document.getElementById('out').textContent = style.getPropertyPriority('color') + ':' + style.getPropertyPriority('background-color') + ':' + style.getPropertyPriority('missing');</script></main>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "important::");
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
            "<main id='root'><div id='box'></div><script>document.getElementById('box').style.getPropertyPriority('color', 'extra');</script></main>",
        ),
    );
}

test "failure: Harness.fromHtml rejects whitespace classList tokens in inline scripts" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<button id='button' class='base'></button><script>document.getElementById('button').classList.add('bad token');</script>",
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

test "contract: Harness.fromHtmlWithUrl exposes origin aliases" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtmlWithUrl(
        allocator,
        "https://example.test:8443/start?x#old",
        "<main id='root'><span id='child'></span></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = document.origin + ':' + window.origin + ':' + root.origin + ':' + child.origin;</script>",
    );
    defer subject.deinit();

    try subject.assertValue(
        "#out",
        "https://example.test:8443:https://example.test:8443:https://example.test:8443:https://example.test:8443",
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
