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
            "<main id='out'>Before</main><script>document.getElementById('out').textContent = ;</script>",
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
        "<div id='root'><form id='signup'><input type='radio' name='mode' id='mode-a' value='a'><input type='radio' name='mode' id='mode-b' value='b'><textarea name='bio'>Bio</textarea></form></div><div id='out'></div><script>const elements = document.getElementById('signup').elements; const named = elements.namedItem('mode'); const before = named.length; document.getElementById('signup').innerHTML += '<input type=\"radio\" name=\"mode\" id=\"mode-c\" value=\"c\" checked>'; document.getElementById('out').textContent = String(before) + ':' + String(named.length) + ':' + named.item(0).value + ':' + named.item(1).value + ':' + named.value + ':' + String(named);</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:3:a:b:c:[object RadioNodeList]");
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
        "<div id='root'><style id='first-style'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'><link id='ignored-link' rel='preload' href='b.css'></div><div id='out'></div><script>const sheets = document.styleSheets; const before = sheets.length; const first = sheets.item(0); const second = sheets.item(1); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(sheets.length) + ':' + String(first) + ':' + String(second) + ':' + String(sheets.item(2));</script>",
    );
    defer subject.deinit();

    try subject.assertValue("#out", "2:0:[object CSSStyleSheet]:[object CSSStyleSheet]:null");
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

test "failure: Harness.fromHtml rejects unsupported collection entries helpers" {
    const allocator = std.testing.allocator;
    try std.testing.expectError(
        error.ScriptRuntime,
        Harness.fromHtml(
            allocator,
            "<main id='root'><span class='item'>One</span></main><script>document.querySelectorAll('.item').entries();</script>",
        ),
    );
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
