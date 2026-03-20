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
pub const LocationMocks = mocks.LocationMocks;
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

test "contract: Harness.mocksMut exposes fetch, dialogs, clipboard, location, downloads, and storage" {
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
        try mocks_view.location().setCurrent("https://example.test/next");
        try mocks_view.location().recordNavigation("https://example.test/next");
        try mocks_view.downloads().capture("report.csv", "downloaded bytes");
        try mocks_view.fileInput().setFiles("#upload", &.{"report.csv"});
        try mocks_view.storage().seedLocal("token", "abc");
        try mocks_view.storage().seedSession("session-token", "xyz");
    }

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
    try std.testing.expectEqual(@as(?[]const u8, null), subject.mocksMut().location().currentUrl());
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().location().navigations().len);
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().downloads().artifacts().len);
    try std.testing.expectEqual(@as(usize, 0), subject.mocksMut().fileInput().selections().len);
    try std.testing.expectEqual(@as(?[]const u8, null), subject.mocksMut().storage().local().get("token"));
    try std.testing.expectEqual(@as(?[]const u8, null), subject.mocksMut().storage().session().get("session-token"));
    try std.testing.expectError(error.MockError, subject.readClipboard());
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
            "<main id='out'>Before</main><script>const value = 'x';</script>",
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

test "contract: Harness.fromHtml runs script querySelectorAll during bootstrap" {
    const allocator = std.testing.allocator;
    var subject = try Harness.fromHtml(
        allocator,
        "<main id='root'><section><button id='first' class='primary'>First</button></section><button id='second' class='secondary'>Second</button></main><div id='out'></div><script>document.querySelector('#out').textContent = document.querySelectorAll('button').length + ':' + document.querySelectorAll('button').item(0).textContent + ':' + document.querySelectorAll('button').item(1).textContent + ':' + String(document.querySelector('#root').querySelectorAll('button').length) + ':' + String(document.querySelectorAll('button').item(99));</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:First:Second:2:null");
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
            "<main id='root'></main><script>document.querySelector('main + span');</script>",
        ),
    );
}

test "failure: Harness.fromHtml rejects unsupported querySelectorAll syntax" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'></main><script>document.querySelectorAll('main + span');</script>",
        ),
    );
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

    try std.testing.expectError(error.InvalidSelector, subject.assertExists("main + span"));
    try std.testing.expectError(error.InvalidSelector, subject.assertExists("[data-state"));
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
