const std = @import("std");
const dom = @import("dom.zig");
const errors = @import("errors.zig");
const mocks = @import("mocks.zig");
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

    pub fn url(self: *const Harness) []const u8 {
        return self.session.url();
    }

    pub fn html(self: *const Harness) ?[]const u8 {
        return self.session.html();
    }

    pub fn localStorage(self: *const Harness) []const session.StorageSeed {
        return self.session.localStorage();
    }

    pub fn nowMs(self: *const Harness) i64 {
        return self.session.nowMs();
    }

    pub fn advanceTime(self: *Harness, delta_ms: i64) errors.Result(void) {
        return self.session.advanceTime(delta_ms);
    }

    pub fn flush(self: *Harness) errors.Result(void) {
        return self.session.flush();
    }

    pub fn mocksMut(self: *Harness) *mocks.MockRegistry {
        return self.session.mocksMut();
    }

    pub fn alert(self: *Harness, message: []const u8) errors.Result(void) {
        return self.session.alert(message);
    }

    pub fn confirm(self: *Harness, message: []const u8) errors.Result(bool) {
        return self.session.confirm(message);
    }

    pub fn prompt(self: *Harness, message: []const u8) errors.Result(?[]const u8) {
        return self.session.prompt(message);
    }

    pub fn readClipboard(self: *Harness) errors.Result([]const u8) {
        return self.session.readClipboard();
    }

    pub fn writeClipboard(self: *Harness, text: []const u8) errors.Result(void) {
        return self.session.writeClipboard(text);
    }

    pub fn captureDownload(
        self: *Harness,
        file_name: []const u8,
        bytes: []const u8,
    ) errors.Result(void) {
        return self.session.captureDownload(file_name, bytes);
    }

    pub fn fetch(self: *Harness, url_source: []const u8) errors.Result(mocks.FetchResponse) {
        return self.session.fetch(url_source);
    }

    pub fn navigate(self: *Harness, url_source: []const u8) errors.Result(void) {
        return self.session.navigate(url_source);
    }

    pub fn setFiles(
        self: *Harness,
        selector: []const u8,
        files: []const []const u8,
    ) errors.Result(void) {
        const node_id = try self.resolveActionTarget(selector);
        try self.session.setFilesNode(node_id, selector, files);
        return;
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

    pub fn click(self: *Harness, selector: []const u8) errors.Result(void) {
        const node_id = try self.resolveActionTarget(selector);
        try self.session.clickNode(node_id);
        return;
    }

    pub fn typeText(self: *Harness, selector: []const u8, text: []const u8) errors.Result(void) {
        const node_id = try self.resolveActionTarget(selector);
        try self.session.typeTextNode(node_id, text);
        return;
    }

    pub fn setChecked(self: *Harness, selector: []const u8, checked: bool) errors.Result(void) {
        const node_id = try self.resolveActionTarget(selector);
        try self.session.setCheckedNode(node_id, checked);
        return;
    }

    pub fn setSelectValue(self: *Harness, selector: []const u8, value: []const u8) errors.Result(void) {
        const node_id = try self.resolveActionTarget(selector);
        try self.session.setSelectValueNode(node_id, value);
        return;
    }

    pub fn focus(self: *Harness, selector: []const u8) errors.Result(void) {
        const node_id = try self.resolveActionTarget(selector);
        try self.session.focusNode(node_id);
        return;
    }

    pub fn blur(self: *Harness, selector: []const u8) errors.Result(void) {
        const node_id = try self.resolveActionTarget(selector);
        try self.session.blurNode(node_id);
        return;
    }

    pub fn submit(self: *Harness, selector: []const u8) errors.Result(void) {
        const node_id = try self.resolveActionTarget(selector);
        try self.session.submitNode(node_id);
        return;
    }

    pub fn dispatch(self: *Harness, selector: []const u8, event_type: []const u8) errors.Result(void) {
        const node_id = try self.resolveActionTarget(selector);
        try self.session.dispatchNode(node_id, event_type);
        return;
    }

    pub fn assertValue(
        self: *const Harness,
        selector: []const u8,
        expected: []const u8,
    ) errors.Result(void) {
        const node_id = try self.resolveAssertionTarget(selector);
        const actual = try self.session.domStore().valueForNode(std.heap.page_allocator, node_id);
        defer std.heap.page_allocator.free(actual);

        if (!std.mem.eql(u8, actual, expected)) {
            return error.AssertionFailed;
        }

        return;
    }

    pub fn assertChecked(
        self: *const Harness,
        selector: []const u8,
        expected: bool,
    ) errors.Result(void) {
        const node_id = try self.resolveAssertionTarget(selector);
        const actual = self.session.domStore().checkedForNode(node_id) orelse return error.AssertionFailed;
        if (actual != expected) {
            return error.AssertionFailed;
        }

        return;
    }

    fn resolveActionTarget(self: *const Harness, selector: []const u8) errors.Result(dom.NodeId) {
        const node_id = try self.selectFirstMatch(selector);
        if (node_id) |id| return id;
        return error.DomError;
    }

    fn resolveAssertionTarget(self: *const Harness, selector: []const u8) errors.Result(dom.NodeId) {
        const node_id = try self.selectFirstMatch(selector);
        if (node_id) |id| return id;
        return error.AssertionFailed;
    }

    fn selectFirstMatch(self: *const Harness, selector: []const u8) errors.Result(?dom.NodeId) {
        const matches = self.session.domStore().select(std.heap.page_allocator, selector) catch |err| switch (err) {
            error.HtmlParse => return error.InvalidSelector,
            error.OutOfMemory => return error.OutOfMemory,
            else => unreachable,
        };
        defer std.heap.page_allocator.free(matches);

        if (matches.len == 0) {
            return null;
        }

        return matches[0];
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

test "regression: phase 7 query selector methods resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root' class='app'><section class='panel'><span id='marker'>panel</span><button id='first' class='primary'>First</button><button id='second' class='secondary'>Second</button></section></main><div id='out'></div><script>document.getElementById('out').textContent = document.querySelector('button').textContent + ':' + document.getElementById('root').querySelector('button.secondary').textContent + ':' + String(document.querySelector('#second').matches('button.secondary')) + ':' + document.querySelector('#second').closest('section.panel').querySelector('#marker').textContent + ':' + String(document.getElementById('root').querySelector('main'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "First:Second:true:panel:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 7 querySelectorAll methods resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section><button id='first' class='primary'>First</button></section><button id='second' class='secondary'>Second</button></main><div id='out'></div><script>document.querySelector('#out').textContent = document.querySelectorAll('button').length + ':' + document.querySelectorAll('button').item(0).textContent + ':' + document.querySelectorAll('button').item(1).textContent + ':' + String(document.querySelector('#root').querySelectorAll('button').length) + ':' + String(document.querySelectorAll('button').item(99));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2:First:Second:2:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 9 NodeList.forEach resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='first'>First</button><button id='second'>Second</button></main><div id='out'></div><script>document.querySelectorAll('button').forEach((item, index, list) => { document.getElementById('out').textContent += String(index) + ':' + item.textContent + ':' + String(list.length) + ';'; item.remove(); }, null);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "0:First:2;1:Second:2;");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 9 document.scripts resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><script id='first-script'></script><script name='named-script'></script></main><div id='out'></div><script>document.getElementById('out').textContent = String(document.scripts.length) + ':' + String(document.scripts.item(0)) + ':' + String(document.scripts.namedItem('first-script')) + ':' + String(document.scripts.namedItem('named-script')) + ':' + String(document.scripts.namedItem('missing')) + ':'; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent += String(document.scripts.length) + ':' + String(document.scripts.namedItem('first-script')) + ':' + String(document.scripts.namedItem('named-script')) + ':' + String(document.scripts.namedItem('missing'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "3:[object Element]:[object Element]:[object Element]:null:1:null:null:null",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 9 document.anchors resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><a name='first'>First</a><a id='ignored'>Ignored</a></main><div id='out'></div><script>document.getElementById('out').textContent = String(document.anchors.length) + ':' + document.anchors.item(0).textContent + ':' + String(document.anchors.namedItem('ignored')) + ':' + document.anchors.namedItem('first').textContent + ':' + String(document.anchors.namedItem('missing')); document.getElementById('root').innerHTML = document.getElementById('root').innerHTML + '<a name=\"second\">Second</a>'; document.getElementById('out').textContent += ':' + String(document.anchors.length) + ':' + document.anchors.namedItem('second').textContent + ':' + String(document.anchors.namedItem('missing'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:First:null:First:null:2:Second:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 11 document.forms resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='signup' name='signup'>Signup</form><form id='login' name='login'>Login</form></main><div id='out'></div><script>const forms = document.forms; const first = forms.item(0); const named = forms.namedItem('signup'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(forms.length) + ':' + String(first) + ':' + String(named) + ':' + String(forms.namedItem('missing'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "0:[object Element]:[object Element]:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 11 form.elements resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='signup'><input type='radio' name='mode' id='mode-a' value='a'><input type='radio' name='mode' id='mode-b' value='b'><textarea name='bio'>Bio</textarea></form></main><div id='out'></div><script>const elements = document.getElementById('signup').elements; const named = elements.namedItem('mode'); const before = named.length; document.getElementById('signup').innerHTML += '<input type=\"radio\" name=\"mode\" id=\"mode-c\" value=\"c\" checked>'; document.getElementById('out').textContent = String(before) + ':' + String(named.length) + ':' + named.item(0).value + ':' + named.item(1).value + ':' + named.value + ':' + String(named);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2:3:a:b:c:[object RadioNodeList]");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 11 select.options resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><select id='mode'><option name='alpha' value='a'>A</option><option id='second' value='b'>B</option></select></main><div id='out'></div><script>const options = document.getElementById('mode').options; const first = options.item(0); const named = options.namedItem('second'); document.getElementById('mode').textContent = 'gone'; document.getElementById('out').textContent = String(options.length) + ':' + String(first) + ':' + String(named) + ':' + String(options.namedItem('missing'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "0:[object Element]:[object Element]:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 11 select.selectedOptions resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><select id='mode'><option id='first' value='a' selected>A</option><option id='second' value='b'>B</option></select></main><div id='out'></div><script>const select = document.getElementById('mode'); const selected = select.selectedOptions; const before = selected.length; const first = selected.item(0); select.innerHTML = '<option id=\"third\" value=\"c\" selected>C</option><option id=\"fourth\" value=\"d\" selected>D</option>'; document.getElementById('out').textContent = String(before) + ':' + String(selected.length) + ':' + first.textContent + ':' + selected.item(0).textContent + ':' + selected.item(1).textContent + ':' + String(selected.namedItem('third')) + ':' + String(selected.namedItem('missing'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:2:A:C:D:[object Element]:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 12 fieldset.elements and datalist.options resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><fieldset id='fieldset'><input name='first' value='Ada'><textarea name='bio'>Bio</textarea></fieldset><datalist id='list'><option name='alpha' value='a'>A</option><option id='second' value='b'>B</option></datalist><div id='out'></div><script>const elements = document.getElementById('fieldset').elements; const options = document.getElementById('list').options; const beforeElements = elements.length; const beforeOptions = options.length; const first = elements.item(0); const namedElement = elements.namedItem('first'); const namedOption = options.namedItem('second'); document.getElementById('fieldset').textContent = 'gone'; document.getElementById('list').textContent = 'gone'; document.getElementById('out').textContent = String(beforeElements) + ':' + String(elements.length) + ':' + String(beforeOptions) + ':' + String(options.length) + ':' + first.value + ':' + namedElement.value + ':' + namedOption.textContent + ':' + String(options.namedItem('missing'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2:0:2:0:Ada:Ada:B:null");
    try subject.assertExists("#fieldset");
    try subject.assertExists("#list");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 13 map.areas and table.tBodies resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><map id='map'><area id='first-area' name='first' href='/first'><area id='second-area' name='second' href='/second'></map><table id='table'><tbody id='first-body'><tr><td>One</td></tr></tbody></table><div id='out'></div><script>const areas = document.getElementById('map').areas; const bodies = document.getElementById('table').tBodies; const beforeAreas = areas.length; const beforeBodies = bodies.length; const firstArea = areas.item(0); const firstBody = bodies.item(0); document.getElementById('map').innerHTML += '<area id=\"third-area\" name=\"third\" href=\"/third\">'; document.getElementById('table').innerHTML += '<tbody id=\"second-body\"></tbody>'; document.getElementById('out').textContent = String(beforeAreas) + ':' + String(areas.length) + ':' + String(beforeBodies) + ':' + String(bodies.length) + ':' + String(firstArea.getAttribute('id')) + ':' + String(firstBody.getAttribute('id')) + ':' + String(areas.namedItem('third-area')) + ':' + String(bodies.namedItem('second-body')) + ':' + String(areas.namedItem('missing'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2:3:1:2:first-area:first-body:[object Element]:[object Element]:null");
    try subject.assertExists("#third-area");
    try subject.assertExists("#second-body");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 14 element.labels resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><label id='explicit-label' for='control'>Explicit</label><input id='control' value='A'><label id='implicit-label'><input id='inner-control' value='B'>Implicit</label><fieldset id='group'></fieldset><label id='group-label' for='group'>Group</label><div id='wrapper'></div><div id='out'></div><script>const control = document.getElementById('control'); const labels = control.labels; const inner = document.getElementById('inner-control').labels; const fieldset = document.getElementById('group'); const fieldsetLabels = fieldset.labels; const before = labels.length; const fieldsetBefore = fieldsetLabels.length; document.getElementById('wrapper').innerHTML = '<label id=\"second-label\" for=\"control\">Second</label><label id=\"group-second\" for=\"group\">Second Group</label>'; document.getElementById('out').textContent = String(before) + ':' + String(labels.length) + ':' + labels.item(0).getAttribute('id') + ':' + labels.item(1).textContent + ':' + String(inner.length) + ':' + inner.item(0).getAttribute('id') + ':' + String(fieldsetBefore) + ':' + String(fieldsetLabels.length) + ':' + fieldsetLabels.item(0).getAttribute('id') + ':' + fieldsetLabels.item(1).getAttribute('id');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:2:explicit-label:Second:1:implicit-label:1:2:group-label:group-second");
    try subject.assertExists("#second-label");
    try subject.assertExists("#group-second");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15 document.images and document.links resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><img id='hero' name='hero' alt='Hero'><img name='thumb' alt='Thumb'><a id='docs' href='/docs'>Docs</a><a id='plain'>Plain</a><area id='map' name='map' href='/map'><div id='out'></div><script>const images = document.images; const links = document.links; const beforeImages = images.length; const beforeLinks = links.length; const hero = images.namedItem('hero'); const thumb = images.namedItem('thumb'); const docs = links.namedItem('docs'); const map = links.namedItem('map'); const plain = links.namedItem('plain'); document.getElementById('root').innerHTML += '<img id=\"third\" name=\"third\" alt=\"Third\"><a id=\"more\" href=\"/more\">More</a>'; document.getElementById('out').textContent = String(beforeImages) + ':' + String(images.length) + ':' + String(beforeLinks) + ':' + String(links.length) + ':' + String(hero) + ':' + String(thumb) + ':' + String(docs) + ':' + String(map) + ':' + String(plain);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2:3:2:3:[object Element]:[object Element]:[object Element]:[object Element]:null");
    try subject.assertExists("#third");
    try subject.assertExists("#more");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 16 document.embeds, document.plugins, document.applets, and document.all resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><embed id='first-embed' name='first-embed'><embed name='second-embed'><applet id='first-applet' name='first-applet'>First</applet><div id='first'>First</div><div id='second' name='second'>Second</div><div id='out'></div><script>const embeds = document.embeds; const plugins = document.plugins; const applets = document.applets; const all = document.all; const beforeEmbeds = embeds.length; const beforePlugins = plugins.length; const beforeApplets = applets.length; const beforeAll = all.length; const firstEmbed = embeds.namedItem('first-embed'); const firstPlugin = plugins.namedItem('first-embed'); const firstApplet = applets.namedItem('first-applet'); const second = all.namedItem('second'); document.getElementById('root').innerHTML += '<embed id=\"third-embed\" name=\"third-embed\"><applet id=\"second-applet\" name=\"second-applet\">Second</applet>'; document.getElementById('out').textContent = String(beforeEmbeds) + ':' + String(embeds.length) + ':' + String(beforePlugins) + ':' + String(plugins.length) + ':' + String(beforeApplets) + ':' + String(applets.length) + ':' + String(beforeAll) + ':' + String(all.length) + ':' + String(firstEmbed) + ':' + String(firstPlugin) + ':' + String(firstApplet) + ':' + String(second);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2:3:2:3:1:2:8:10:[object Element]:[object Element]:[object Element]:[object Element]");
    try subject.assertExists("#third-embed");
    try subject.assertExists("#second-applet");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17 document.styleSheets resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='first-style'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'><link id='ignored-link' rel='preload' href='b.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const sheets = document.styleSheets; const before = sheets.length; document.getElementById('first-link').setAttribute('rel', 'preload'); out.textContent = String(before) + ':' + String(sheets.length) + ':' + String(sheets.item(0)) + ':' + String(sheets.item(1));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2:1:[object CSSStyleSheet]:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 18 table.rows and tr.cells resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<table id='table'><thead id='head'><tr id='head-row'><th id='head-cell'>H</th></tr></thead><tbody id='body'><tr id='first-row'><td id='first-cell'>A</td></tr></tbody><tfoot id='foot'><tr id='foot-row'><td id='foot-cell'>F</td></tr></tfoot></table><div id='out'></div><script>const table = document.getElementById('table'); const body = document.getElementById('body'); const row = document.getElementById('first-row'); const rows = table.rows; const bodyRows = body.rows; const cells = row.cells; const before = String(rows.length) + ':' + String(bodyRows.length) + ':' + String(cells.length) + ':' + String(rows.namedItem('first-row')) + ':' + String(cells.namedItem('first-cell')); body.innerHTML = body.innerHTML + '<tr id=\"second-row\"><td id=\"second-cell\">B</td><td id=\"third-cell\">C</td></tr>'; row.append(document.getElementById('third-cell')); document.getElementById('out').textContent = before + '|' + String(rows.length) + ':' + String(bodyRows.length) + ':' + String(cells.length) + ':' + String(rows.namedItem('second-row')) + ':' + String(bodyRows.namedItem('second-row')) + ':' + String(cells.namedItem('third-cell'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "3:1:1:[object Element]:[object Element]|4:2:2:[object Element]:[object Element]:[object Element]",
    );
    try subject.assertExists("#second-row");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 19 getElementsByTagName family resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='scope'><span id='first' class='alpha'>One</span><div id='class-target' class='alpha'>Two</div><svg id='icon'><foreignobject id='foreign'><div id='html' class='alpha beta'>Svg</div></foreignobject></svg><input id='named' name='search'></main><div id='before'></div><div id='after'></div><script>const scope = document.getElementById('scope'); const tags = scope.getElementsByTagName('span'); const classes = scope.getElementsByClassName('alpha beta'); const ns = scope.getElementsByTagNameNS('http://www.w3.org/2000/svg', '*'); const names = document.getElementsByName('search'); document.getElementById('before').textContent = String(tags.length) + ':' + String(classes.length) + ':' + String(ns.length) + ':' + String(names.length) + ':' + tags.item(0).getAttribute('id') + ':' + classes.item(0).getAttribute('id') + ':' + ns.item(0).getAttribute('id') + ':' + ns.namedItem('foreign').getAttribute('id') + ':' + names.item(0).getAttribute('id'); scope.innerHTML = scope.innerHTML + '<span id=\"second\" class=\"alpha beta\">Two</span><input id=\"second-named\" name=\"search\">'; document.getElementById('class-target').className = 'alpha beta'; document.getElementById('after').textContent = String(tags.length) + ':' + String(classes.length) + ':' + String(ns.length) + ':' + String(names.length) + ':' + String(tags.namedItem('second')) + ':' + String(classes.namedItem('class-target')) + ':' + names.item(1).getAttribute('id');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#before", "1:1:2:1:first:html:icon:foreign:named");
    try subject.assertValue("#after", "2:3:2:2:[object Element]:[object Element]:second-named");
    try subject.assertExists("#second");
    try subject.assertExists("#second-named");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 20 sibling combinators resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='first'>First</button><span id='gap'>Gap</span><button id='second'>Second</button><button id='third'>Third</button><div id='out'></div><script>document.querySelector('#first + span').addEventListener('click', () => { document.getElementById('out').textContent = document.querySelector('#first ~ button').textContent + ':' + String(document.querySelectorAll('#first ~ button').length); });</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.click("#first + span");
    try subject.assertValue("#out", "Second:2");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 9 collection iterator helpers resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><span class='item'>One</span><span class='item'>Two</span><a name='first'>Alpha</a><a name='second'>Beta</a></main><div id='out'></div><script id='trailing-script'>const nodes = document.querySelectorAll('.item'); const nodeValues = nodes.values(); const nodeKeys = nodes.keys(); const anchors = document.anchors; const anchorValues = anchors.values(); const anchorKeys = anchors.keys(); document.getElementById('root').textContent = 'gone'; const firstNode = nodeValues.next(); const secondNode = nodeValues.next(); const thirdNode = nodeValues.next(); const firstNodeKey = nodeKeys.next(); const secondNodeKey = nodeKeys.next(); const thirdNodeKey = nodeKeys.next(); const firstAnchor = anchorValues.next(); const secondAnchor = anchorValues.next(); const thirdAnchor = anchorValues.next(); const firstAnchorKey = anchorKeys.next(); const secondAnchorKey = anchorKeys.next(); const thirdAnchorKey = anchorKeys.next(); document.getElementById('out').textContent = String(nodes.length) + ':' + String(anchors.length) + ':' + firstNode.value.textContent + ':' + String(firstNode.done) + ':' + secondNode.value.textContent + ':' + String(secondNode.done) + ':' + String(thirdNode.done) + ':' + String(firstNodeKey.value) + ':' + String(secondNodeKey.value) + ':' + String(thirdNodeKey.done) + ':' + firstAnchor.value.textContent + ':' + String(firstAnchor.done) + ':' + secondAnchor.value.textContent + ':' + String(secondAnchor.done) + ':' + String(thirdAnchor.done) + ':' + String(firstAnchorKey.value) + ':' + String(secondAnchorKey.value) + ':' + String(thirdAnchorKey.value) + ':' + String(thirdAnchorKey.done);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "2:0:One:false:Two:false:true:0:1:true:Alpha:false:Beta:false:true:0:1:undefined:true",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 10 document.children resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><span>First</span></main><div id='out'></div><script>const children = document.children; const before = children.length; const first = children.item(0); const root = children.namedItem('root'); document.getElementById('root').remove(); document.getElementById('out').textContent = String(before) + ':' + String(children.length) + ':' + String(first) + ':' + String(root) + ':' + String(children.namedItem('root'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "3:2:[object Element]:[object Element]:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 10 document.childNodes and Element.children resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<!--pre--><main id='root'>Hello<span>World</span><!--tail--></main><div id='out'></div><script>const docNodes = document.childNodes; const rootNodes = document.getElementById('root').childNodes; const root = document.getElementById('root'); const docFirst = docNodes.item(0); const docSecond = docNodes.item(1); const rootValues = rootNodes.values(); const firstRoot = rootValues.next(); const secondRoot = rootValues.next(); const thirdRoot = rootValues.next(); root.innerHTML += '<span id=\"second\">Second</span>'; document.getElementById('out').textContent = String(docNodes.length) + ':' + docFirst.nodeName + ':' + String(docFirst.nodeType) + ':' + String(docFirst) + ':' + docSecond.nodeName + ':' + String(docSecond.nodeType) + ':' + firstRoot.value.nodeName + ':' + String(firstRoot.value.nodeType) + ':' + firstRoot.value.textContent + ':' + secondRoot.value.nodeName + ':' + String(secondRoot.value.nodeType) + ':' + secondRoot.value.textContent + ':' + thirdRoot.value.nodeName + ':' + String(thirdRoot.value.nodeType) + ':' + thirdRoot.value.textContent + ':' + String(rootNodes.length) + ':' + String(root.children.length) + ':' + root.children.item(1).textContent + ':' + root.children.namedItem('second').textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "4:#comment:8:[object Node]:main:1:#text:3:Hello:span:1:World:#comment:8:tail:4:2:Second:Second",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 attribute reflection methods resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='button'>First</button><input id='name'><input id='agree' type='checkbox'><select id='mode'><option value='a'>A</option><option id='selected' value='b'>B</option></select><div id='out'></div><script>document.getElementById('button').setAttribute('class', 'primary'); document.getElementById('out').textContent = String(document.querySelectorAll('.primary').length) + ':' + String(document.getElementById('button').hasAttribute('data-flag')) + ':'; document.getElementById('out').textContent += String(document.getElementById('button').toggleAttribute('data-flag')) + ':' + String(document.querySelectorAll('[data-flag]').length) + ':'; document.getElementById('out').textContent += String(document.getElementById('button').toggleAttribute('data-flag', false)) + ':' + String(document.querySelectorAll('[data-flag]').length) + ':'; document.getElementById('button').setAttribute('data-label', 'Hello'); document.getElementById('out').textContent += String(document.getElementById('button').getAttribute('data-label')) + ':'; document.getElementById('button').removeAttribute('data-label'); document.getElementById('out').textContent += String(document.getElementById('button').getAttribute('data-label')) + ':'; document.getElementById('name').setAttribute('value', 'Alice'); document.getElementById('agree').setAttribute('checked', ''); document.getElementById('selected').setAttribute('selected', ''); document.getElementById('out').textContent += document.getElementById('name').value + ':' + String(document.getElementById('agree').checked) + ':' + document.getElementById('mode').value;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:false:true:1:false:0:Hello:null:Alice:true:b");
    try subject.assertExists(".primary");
    try subject.assertValue("#name", "Alice");
    try subject.assertChecked("#agree", true);
    try subject.assertValue("#mode", "b");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 class and dataset views resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='button' class='base' data-kind='App'>First</button><div id='out'></div><script>document.getElementById('button').className = 'primary secondary'; document.getElementById('button').classList.add('tertiary'); document.getElementById('button').classList.remove('secondary'); document.getElementById('button').dataset.userId = '42'; document.getElementById('out').textContent = String(document.getElementById('button').classList.length) + ':' + String(document.getElementById('button').classList.contains('primary')) + ':' + String(document.getElementById('button').classList.toggle('active')) + ':' + document.getElementById('button').className + ':' + document.getElementById('button').dataset.kind + ':' + document.getElementById('button').dataset.userId + ':' + String(document.getElementById('button').classList) + ':' + String(document.getElementById('button').dataset);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2:true:true:primary tertiary active:App:42:[object DOMTokenList]:[object DOMStringMap]");
    try subject.assertExists(".active");
    try subject.assertExists("[data-user-id]");
    try subject.assertExists("[data-kind=App]");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 tree mutation primitives resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='source'><button id='second'>Second</button><button id='third'>Third</button></section><button id='first'>First</button><div id='out'></div><script>document.getElementById('second').before(document.getElementById('first')); document.getElementById('second').after(document.getElementById('third')); document.getElementById('out').textContent = document.getElementById('source').textContent + ':' + String(document.querySelectorAll('#source > button').length) + ':' + document.querySelector('#first').textContent + ':' + document.querySelector('#third').textContent;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "FirstSecondThird:3:First:Third");
    try subject.assertExists("#source > #first");
    try subject.assertExists("#source > #second");
    try subject.assertExists("#source > #third");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 HTML serialization surfaces resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section><div id='out'></div><script>document.getElementById('target').innerHTML = '<span id=\"first\">One</span><span id=\"second\">Two</span>'; document.getElementById('out').textContent = document.getElementById('target').innerHTML + '|' + document.getElementById('target').outerHTML + '|' + String(document.querySelector('#old'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "<span id=\"first\">One</span><span id=\"second\">Two</span>|<section id=\"target\"><span id=\"first\">One</span><span id=\"second\">Two</span></section>|null",
    );
    try subject.assertExists("#target > #first");
    try subject.assertExists("#target > #second");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 insertAdjacentHTML surfaces resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section></main><div id='out'></div><script>document.getElementById('target').insertAdjacentHTML('beforebegin', '<aside id=\"before\">Before</aside>'); document.getElementById('target').insertAdjacentHTML('afterbegin', '<span id=\"first\">First</span>'); document.getElementById('target').insertAdjacentHTML('beforeend', '<span id=\"last\">Last</span>'); document.getElementById('target').insertAdjacentHTML('afterend', '<aside id=\"after\">After</aside>'); document.getElementById('out').textContent = document.getElementById('root').innerHTML + '|' + document.getElementById('target').innerHTML + '|' + String(document.querySelectorAll('#target > span').length) + ':' + String(document.querySelector('#before')) + ':' + String(document.querySelector('#after'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "<aside id=\"before\">Before</aside><section id=\"target\"><span id=\"first\">First</span><button class=\"primary\" id=\"old\">Old</button><span id=\"last\">Last</span></section><aside id=\"after\">After</aside>|<span id=\"first\">First</span><button class=\"primary\" id=\"old\">Old</button><span id=\"last\">Last</span>|2:[object Element]:[object Element]",
    );
    try subject.assertExists("#before");
    try subject.assertExists("#after");
    try subject.assertExists("#target > #first");
    try subject.assertExists("#target > #last");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 template.content surfaces resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>document.getElementById('out').textContent = String(document.getElementById('tpl').content) + '|' + document.getElementById('tpl').content.innerHTML; document.getElementById('tpl').content.innerHTML = '<!--tail--><span id=\"second\">Second</span>'; document.getElementById('out').textContent += '|' + String(document.getElementById('tpl').content) + '|' + document.getElementById('tpl').content.innerHTML;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object DocumentFragment]|<span id=\"inner\">Inner</span>|[object DocumentFragment]|<!--tail--><span id=\"second\">Second</span>",
    );
    try subject.assertExists("#second");
    try std.testing.expectError(error.AssertionFailed, subject.assertExists("#inner"));
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 namespace-aware serialization surfaces resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><svg id='icon' viewbox='0 0 10 10'><foreignobject id='foreign'><div id='html'>Text</div></foreignobject></svg><math id='formula' definitionurl='https://example.com'><mi id='symbol'>x</mi></math><div id='out'></div><script>document.getElementById('out').textContent = document.getElementById('icon').outerHTML + '|' + document.getElementById('formula').outerHTML;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "<svg id=\"icon\" viewBox=\"0 0 10 10\"><foreignObject id=\"foreign\"><div id=\"html\">Text</div></foreignObject></svg>|<math definitionURL=\"https://example.com\" id=\"formula\"><mi id=\"symbol\">x</mi></math>",
    );
    try subject.assertExists("#foreign");
    try subject.assertExists("#symbol");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 3 actions resolve expanded selectors on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='agree' type='checkbox'></main><div id='out'></div><script>document.getElementById('agree').addEventListener('change', () => { document.getElementById('out').textContent = String(document.getElementById('agree').checked); });</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.click("main > #agree");
    try subject.assertChecked("main > #agree", true);
    try subject.assertValue("#out", "true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 4 mock helpers operate on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<input id='upload' type='file'><div id='out'></div><script>document.getElementById('upload').addEventListener('change', () => { document.getElementById('out').textContent = document.getElementById('upload').value; });</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.setFiles("#upload", &.{"report.csv"});
    try subject.assertValue("#upload", "report.csv");
    try subject.assertValue("#out", "report.csv");
    try std.testing.expectEqualStrings(original, subject.html().?);

    const registry = subject.mocksMut();
    const selections = registry.fileInput().selections();
    try std.testing.expectEqual(@as(usize, 1), selections.len);
    try std.testing.expectEqualStrings("#upload", selections[0].selector);
    try std.testing.expectEqual(@as(usize, 1), selections[0].files.len);
    try std.testing.expectEqualStrings("report.csv", selections[0].files[0]);
}
