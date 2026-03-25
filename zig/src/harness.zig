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
    session_storage: std.ArrayListUnmanaged(session.StorageSeed) = .{},
    random_seed_value: ?u64 = null,
    open_failure_value: ?[]const u8 = null,
    close_failure_value: ?[]const u8 = null,
    print_failure_value: ?[]const u8 = null,
    scroll_failure_value: ?[]const u8 = null,

    pub fn init(allocator: std.mem.Allocator) HarnessBuilder {
        return .{ .allocator = allocator };
    }

    pub fn deinit(self: *HarnessBuilder) void {
        self.local_storage.deinit(self.allocator);
        self.session_storage.deinit(self.allocator);
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

    pub fn addSessionStorage(
        self: *HarnessBuilder,
        key: []const u8,
        value: []const u8,
    ) errors.Result(void) {
        try self.session_storage.append(self.allocator, .{
            .key = key,
            .value = value,
        });
    }

    pub fn randomSeed(self: *HarnessBuilder, value: u64) *HarnessBuilder {
        self.random_seed_value = value;
        return self;
    }

    pub fn openFailure(self: *HarnessBuilder, value: []const u8) *HarnessBuilder {
        self.open_failure_value = value;
        return self;
    }

    pub fn closeFailure(self: *HarnessBuilder, value: []const u8) *HarnessBuilder {
        self.close_failure_value = value;
        return self;
    }

    pub fn printFailure(self: *HarnessBuilder, value: []const u8) *HarnessBuilder {
        self.print_failure_value = value;
        return self;
    }

    pub fn scrollFailure(self: *HarnessBuilder, value: []const u8) *HarnessBuilder {
        self.scroll_failure_value = value;
        return self;
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
                .session_storage = self.session_storage.items,
                .random_seed = self.random_seed_value,
                .open_failure = self.open_failure_value,
                .close_failure = self.close_failure_value,
                .print_failure = self.print_failure_value,
                .scroll_failure = self.scroll_failure_value,
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

    pub fn fromHtmlWithSessionStorage(
        allocator: std.mem.Allocator,
        html_source: []const u8,
        seeds: []const session.StorageSeed,
    ) errors.Result(Harness) {
        var subject_builder = HarnessBuilder.init(allocator);
        defer subject_builder.deinit();
        _ = subject_builder.html(html_source);
        for (seeds) |seed| {
            try subject_builder.addSessionStorage(seed.key, seed.value);
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

    pub fn fromHtmlWithUrlAndSessionStorage(
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
            try subject_builder.addSessionStorage(seed.key, seed.value);
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

    pub fn open(self: *Harness, url_source: []const u8) errors.Result(void) {
        return self.session.open(url_source, null, null);
    }

    pub fn close(self: *Harness) errors.Result(void) {
        return self.session.close();
    }

    pub fn print(self: *Harness) errors.Result(void) {
        return self.session.print();
    }

    pub fn scrollTo(self: *Harness, x: i64, y: i64) errors.Result(void) {
        return self.session.scrollTo(x, y);
    }

    pub fn scrollBy(self: *Harness, x: i64, y: i64) errors.Result(void) {
        return self.session.scrollBy(x, y);
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
        try self.session.submitNode(node_id, null);
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

test "regression: phase 10 universal selector resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section><button id='first'>First</button></section><button id='second'>Second</button></main><div id='out'></div><script>document.getElementById('out').textContent = String(document.querySelectorAll('*').length) + ':' + String(document.querySelectorAll('main *').length) + ':' + String(document.querySelectorAll('main > *').length);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "6:3:2");
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

test "regression: phase 11 form.elements includes associated controls outside the subtree on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='before' form='signup' name='before' value='Before'><form id='signup'><input id='inside' name='inside' value='Inside'><textarea id='bio' name='bio'>Bio</textarea></form><input id='after' form='signup' name='after' value='After'></main><div id='out'></div><script>const form = document.getElementById('signup'); const elements = form.elements; const before = form.length; const first = elements.item(0); const second = elements.item(1); const third = elements.item(2); const fourth = elements.item(3); const beforeNamed = elements.namedItem('before'); const afterNamed = elements.namedItem('after'); form.innerHTML += '<input id=\"extra\" name=\"extra\" value=\"Grace\">'; document.getElementById('out').textContent = String(before) + ':' + String(form.length) + ':' + first.id + ':' + second.id + ':' + third.id + ':' + fourth.id + ':' + beforeNamed.value + ':' + afterNamed.value + ':' + elements.namedItem('extra').value;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "4:5:before:inside:bio:after:Before:After:Grace");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 11 form owner and fieldset type reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='owner'></form><div id='host'><input id='input' form='owner'><button id='button' form='owner'></button><fieldset id='fieldset' form='owner'></fieldset><select id='select' form='owner'><optgroup id='group'><option id='option'>A</option></optgroup></select><textarea id='area' form='owner'></textarea><output id='output' form='owner'></output><meter id='meter' form='owner'></meter><progress id='progress' form='owner'></progress></div><div id='out'></div><script>const input = document.getElementById('input'); const button = document.getElementById('button'); const fieldset = document.getElementById('fieldset'); const select = document.getElementById('select'); const group = document.getElementById('group'); const option = document.getElementById('option'); const area = document.getElementById('area'); const output = document.getElementById('output'); const meter = document.getElementById('meter'); const progress = document.getElementById('progress'); const detached = document.createElement('input'); document.getElementById('out').textContent = input.form.id + ':' + button.form.id + ':' + fieldset.form.id + ':' + fieldset.type + ':' + select.form.id + ':' + group.form.id + ':' + option.form.id + ':' + area.form.id + ':' + output.form.id + ':' + meter.form.id + ':' + progress.form.id + ':' + String(detached.form);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "owner:owner:owner:fieldset:owner:owner:owner:owner:owner:owner:owner:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 11a HTMLSelectElement.form resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='owner'></form><select id='select' form='owner'><option id='option' value='a'>A</option></select><div id='out'></div><script>const select = document.getElementById('select'); const before = select.form.id + ':' + select.getAttribute('form') + ':' + String(select.options.length); select.setAttribute('form', 'owner'); document.getElementById('out').textContent = before + '|' + select.form.id + ':' + select.getAttribute('form') + ':' + String(select.options.length);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "owner:owner:1|owner:owner:1");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 11a1 HTMLSelectElement.name resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const select = document.createElement('select'); const first = document.createElement('option'); first.value = 'a'; select.appendChild(first); root.appendChild(select); const before = select.name + ':' + select.getAttribute('name') + ':' + String(document.getElementsByName('mode').length); select.name = 'mode'; const during = select.name + ':' + select.getAttribute('name') + ':' + String(document.getElementsByName('mode').length); select.name = ''; document.getElementById('out').textContent = before + '|' + during + '|' + select.name + ':' + select.getAttribute('name') + ':' + String(document.getElementsByName('mode').length);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":null:0|mode:mode:1|::0");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 11a2 HTMLSelectElement.labels resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><label id='first-label' for='mode'>Mode</label><select id='mode'><option value='a'>A</option></select><div id='wrapper'></div><div id='out'></div><script>const select = document.getElementById('mode'); const labels = select.labels; const before = String(labels.length) + ':' + labels.item(0).getAttribute('id'); document.getElementById('wrapper').innerHTML = '<label id=\"second-label\" for=\"mode\">Second</label>'; document.getElementById('out').textContent = before + '|' + String(labels.length) + ':' + labels.item(0).getAttribute('id') + ':' + labels.item(1).getAttribute('id') + '|' + select.labels.item(0).getAttribute('id');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:first-label|2:first-label:second-label|first-label");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 11a3 HTMLInputElement.labels resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><label id='first-label' for='name'>Name</label><input id='name' value='Ada'><div id='wrapper'></div><div id='out'></div><script>const input = document.getElementById('name'); const labels = input.labels; const before = String(labels.length) + ':' + labels.item(0).getAttribute('id'); document.getElementById('wrapper').innerHTML = '<label id=\"second-label\" for=\"name\">Second</label>'; document.getElementById('out').textContent = before + '|' + String(labels.length) + ':' + labels.item(0).getAttribute('id') + ':' + labels.item(1).getAttribute('id') + '|' + input.labels.item(0).getAttribute('id');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:first-label|2:first-label:second-label|first-label");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 11a4 HTMLTextAreaElement.labels resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><label id='first-label' for='bio'>Bio</label><textarea id='bio'>Hello</textarea><div id='wrapper'></div><div id='out'></div><script>const area = document.getElementById('bio'); const labels = area.labels; const before = String(labels.length) + ':' + labels.item(0).getAttribute('id'); document.getElementById('wrapper').innerHTML = '<label id=\"second-label\" for=\"bio\">Second</label>'; document.getElementById('out').textContent = before + '|' + String(labels.length) + ':' + labels.item(0).getAttribute('id') + ':' + labels.item(1).getAttribute('id') + '|' + area.labels.item(0).getAttribute('id');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:first-label|2:first-label:second-label|first-label");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 11a5 HTMLButtonElement.labels resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><label id='first-label' for='action'>Action</label><button id='action'>Run</button><div id='wrapper'></div><div id='out'></div><script>const button = document.getElementById('action'); const labels = button.labels; const before = String(labels.length) + ':' + labels.item(0).getAttribute('id'); document.getElementById('wrapper').innerHTML = '<label id=\"second-label\" for=\"action\">Second</label>'; document.getElementById('out').textContent = before + '|' + String(labels.length) + ':' + labels.item(0).getAttribute('id') + ':' + labels.item(1).getAttribute('id') + '|' + button.labels.item(0).getAttribute('id');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:first-label|2:first-label:second-label|first-label");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 11b HTMLFieldSetElement name reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='owner'><fieldset id='group' name='group' disabled><input name='first' value='Ada'></fieldset></form><div id='out'></div><script>const fieldset = document.getElementById('group'); const before = fieldset.name + ':' + String(fieldset.disabled) + ':' + String(fieldset.elements.length); fieldset.name = 'next'; fieldset.disabled = false; document.getElementById('out').textContent = before + '|' + fieldset.name + ':' + fieldset.getAttribute('name') + ':' + String(fieldset.disabled) + ':' + String(fieldset.getAttribute('disabled')) + ':' + String(fieldset.elements.length);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "group:true:1|next:next:false:null:1");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 11 legend form reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='owner'></form><fieldset id='group' form='owner'><legend id='legend'>Title</legend></fieldset><legend id='detached'>Loose</legend><div id='out'></div><script>const legend = document.getElementById('legend'); const detached = document.getElementById('detached'); const fieldset = document.getElementById('group'); document.getElementById('out').textContent = legend.form.id + ':' + String(detached.form) + ':' + fieldset.form.id;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "owner:null:owner");
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

test "regression: phase 11a6 HTMLSelectElement.options resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const select = document.createElement('select'); select.innerHTML = '<option id=\"first\" value=\"a\">A</option>'; document.getElementById('root').appendChild(select); const options = select.options; const before = String(options.length) + ':' + options.item(0).getAttribute('id'); select.innerHTML += '<option id=\"second\" value=\"b\">B</option>'; document.getElementById('out').textContent = before + '|' + String(options.length) + ':' + options.item(0).getAttribute('id') + ':' + options.item(1).getAttribute('id');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:first|2:first:second");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 41 form.length and select.length resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='signup'><input name='first' value='Ada'><textarea name='bio'>Bio</textarea></form><select id='mode'><option id='first-option' value='a'>A</option><option id='second-option' value='b'>B</option></select></main><div id='out'></div><script>const form = document.getElementById('signup'); const select = document.getElementById('mode'); const beforeForm = form.length; const beforeSelect = select.length; form.innerHTML += '<input name=\"extra\" value=\"Grace\">'; select.innerHTML += '<option id=\"third-option\" value=\"c\">C</option>'; document.getElementById('out').textContent = String(beforeForm) + ':' + String(beforeSelect) + ':' + String(form.length) + ':' + String(select.length) + ':' + String(form.elements.length) + ':' + String(select.options.length);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2:2:3:3:3:3");
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

test "regression: phase 42 option and optgroup reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><select id='select'><optgroup id='group' label='Group' disabled><option id='first' value='a' disabled>Alpha</option><option id='second' value='b' label='Bravo'>Beta</option></optgroup></select><div id='out'></div><script>const select = document.getElementById('select'); const group = document.getElementById('group'); const first = document.getElementById('first'); const second = document.getElementById('second'); const before = String(group.label) + ':' + String(group.disabled) + ':' + String(first.label) + ':' + String(first.defaultSelected) + ':' + String(first.text) + ':' + String(first.disabled) + ':' + String(first.index) + ':' + String(second.label) + ':' + String(second.defaultSelected) + ':' + String(second.text) + ':' + String(second.disabled) + ':' + String(second.index); group.label = 'Updated group'; group.disabled = false; first.label = 'Alpha label'; first.defaultSelected = true; first.text = 'Alpha!'; first.disabled = false; second.disabled = true; document.getElementById('out').textContent = before + '|' + group.label + ':' + String(group.disabled) + ':' + first.label + ':' + String(first.defaultSelected) + ':' + first.text + ':' + String(first.disabled) + ':' + String(first.index) + ':' + second.label + ':' + String(second.defaultSelected) + ':' + second.text + ':' + String(second.disabled) + ':' + String(second.index) + ':' + select.options.item(0).label;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "Group:true:Alpha:false:Alpha:true:0:Bravo:false:Beta:false:1|Updated group:false:Alpha label:true:Alpha!:false:0:Bravo:false:Beta:true:1:Alpha label");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 42a option label defaultSelected text and index resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><select id='select'><optgroup id='group' label='Group'><option id='first' value='a' selected>Alpha</option><option id='second' value='b' label='Bravo'>Beta</option></optgroup></select><div id='out'></div><script>const select = document.getElementById('select'); const group = document.getElementById('group'); const first = document.getElementById('first'); const second = document.getElementById('second'); const before = String(group.label) + ':' + String(group.disabled) + ':' + String(first.label) + ':' + String(first.defaultSelected) + ':' + String(first.text) + ':' + String(first.index) + ':' + String(second.label) + ':' + String(second.defaultSelected) + ':' + String(second.text) + ':' + String(second.index); group.label = 'Updated group'; first.label = 'Alpha label'; first.defaultSelected = false; first.text = 'Alpha!'; document.getElementById('out').textContent = before + '|' + group.label + ':' + String(group.disabled) + ':' + first.label + ':' + String(first.defaultSelected) + ':' + first.text + ':' + String(first.index) + ':' + second.label + ':' + String(second.defaultSelected) + ':' + second.text + ':' + String(second.index) + ':' + select.options.item(0).label;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "Group:false:Alpha:true:Alpha:0:Bravo:false:Beta:1|Updated group:false:Alpha label:false:Alpha!:0:Bravo:false:Beta:1:Alpha label");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 11 RadioNodeList value assignment resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='signup'><input type='radio' name='mode' id='mode-a' checked><input type='radio' name='mode' id='mode-b' value='b'><input type='radio' name='mode' id='mode-c' value='c'></form></main><div id='out'></div><script>const named = document.getElementById('signup').elements.namedItem('mode'); const initial = named.value; named.value = 'b'; const afterMatch = named.value; named.value = 'on'; const afterOn = named.value; const onA = String(document.getElementById('mode-a').checked); const onB = String(document.getElementById('mode-b').checked); named.value = 'missing'; document.getElementById('out').textContent = initial + ':' + afterMatch + ':' + afterOn + ':' + onA + ':' + onB + ':' + named.value + ':' + String(document.getElementById('mode-a').checked) + ':' + String(document.getElementById('mode-b').checked) + ':' + String(document.getElementById('mode-c').checked);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "on:b:on:true:false::false:false:false");
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

test "regression: phase 12a HTMLFieldSetElement elements and disabled resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='owner'><fieldset id='group' disabled><input id='first' name='first' value='Ada'><textarea id='second' name='bio'>Bio</textarea></fieldset></form><div id='out'></div><script>const fieldset = document.getElementById('group'); const elements = fieldset.elements; const before = String(fieldset.disabled) + ':' + String(elements.length) + ':' + elements.item(0).value + ':' + elements.item(1).value; fieldset.disabled = false; document.getElementById('out').textContent = before + '|' + String(fieldset.disabled) + ':' + String(fieldset.getAttribute('disabled')) + ':' + String(elements.length);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:2:Ada:Bio|false:null:2");
    try subject.assertExists("#group");
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

test "regression: phase 13a HTMLMapElement areas resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><map id='map' name='nav'><area id='first-area' name='first' href='/first'></map><div id='out'></div><script>const map = document.getElementById('map'); const areas = map.areas; const before = String(areas.length) + ':' + areas.item(0).getAttribute('id'); map.innerHTML += '<area id=\"second-area\" name=\"second\" href=\"/second\">'; document.getElementById('out').textContent = before + '|' + String(areas.length) + ':' + areas.item(0).getAttribute('id') + ':' + areas.item(1).getAttribute('id') + ':' + String(areas.namedItem('second-area'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:first-area|2:first-area:second-area:[object Element]");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 13 HTMLMapElement.name resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><map id='map' name='nav'><area id='first-area' name='first' href='/first'></map><div id='out'></div><script>const map = document.getElementById('map'); const before = map.name + ':' + String(map.areas.length) + ':' + map.areas.item(0).getAttribute('id'); map.name = 'menu'; const after = map.name + ':' + map.getAttribute('name') + ':' + String(map.areas.length) + ':' + map.areas.item(0).getAttribute('id'); document.getElementById('out').textContent = before + '|' + after;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "nav:1:first-area|menu:menu:1:first-area");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 14 element.labels resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><label id='explicit-label' for='control'>Explicit</label><input id='control' value='A'><label id='implicit-label'><input id='inner-control' value='B'>Implicit</label><label id='button-label' for='action'>Action</label><button id='action' value='Run'>Run</button><fieldset id='group'></fieldset><label id='group-label' for='group'>Group</label><div id='wrapper'></div><div id='out'></div><script>const control = document.getElementById('control'); const labels = control.labels; const inner = document.getElementById('inner-control').labels; const button = document.getElementById('action'); const buttonLabels = button.labels; const fieldset = document.getElementById('group'); const fieldsetLabels = fieldset.labels; const before = labels.length; const buttonBefore = buttonLabels.length; const fieldsetBefore = fieldsetLabels.length; document.getElementById('wrapper').innerHTML = '<label id=\"second-label\" for=\"control\">Second</label><label id=\"button-second\" for=\"action\">Second Button</label><label id=\"group-second\" for=\"group\">Second Group</label>'; document.getElementById('out').textContent = String(before) + ':' + String(labels.length) + ':' + labels.item(0).getAttribute('id') + ':' + labels.item(1).textContent + ':' + String(inner.length) + ':' + inner.item(0).getAttribute('id') + ':' + String(buttonBefore) + ':' + String(buttonLabels.length) + ':' + buttonLabels.item(0).getAttribute('id') + ':' + buttonLabels.item(1).getAttribute('id') + ':' + String(fieldsetBefore) + ':' + String(fieldsetLabels.length) + ':' + fieldsetLabels.item(0).getAttribute('id') + ':' + fieldsetLabels.item(1).getAttribute('id');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:2:explicit-label:Second:1:implicit-label:1:2:button-label:button-second:1:2:group-label:group-second");
    try subject.assertExists("#second-label");
    try subject.assertExists("#button-second");
    try subject.assertExists("#group-second");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 14 label.control resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><label id='explicit-label' for='control'>Explicit</label><input id='control' value='A'><label id='implicit-label'><input id='inner-control' value='B'>Implicit</label><div id='out'></div><script>document.getElementById('root').innerHTML = document.getElementById('root').innerHTML + '<input id=\"later-control\" value=\"Z\"><label id=\"later-label\" for=\"later-control\">Later</label>'; document.getElementById('out').textContent = document.getElementById('explicit-label').control.getAttribute('id') + ':' + document.getElementById('implicit-label').control.getAttribute('id') + ':' + document.getElementById('later-label').control.getAttribute('id');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "control:inner-control:later-control");
    try subject.assertExists("#later-label");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 14 label.htmlFor resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><label id='explicit-label' for='control'>Explicit</label><input id='control' value='A'><div id='out'></div><script>const label = document.getElementById('explicit-label'); const before = label.htmlFor + ':' + label.control.getAttribute('id'); label.htmlFor = 'inner-control'; document.getElementById('root').innerHTML += '<input id=\"inner-control\" value=\"B\">'; document.getElementById('out').textContent = before + '|' + label.htmlFor + ':' + label.control.getAttribute('id') + ':' + label.getAttribute('for');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "control:control|inner-control:inner-control:inner-control");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 14 label.form resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='owner'></form><input id='control' form='owner' value='A'><label id='explicit' for='control'>Explicit</label><label id='implicit'><input id='inner' form='owner' value='B'>Implicit</label><label id='empty'>Empty</label><div id='out'></div><script>document.getElementById('out').textContent = document.getElementById('explicit').form.id + ':' + document.getElementById('implicit').form.id + ':' + String(document.getElementById('empty').form);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "owner:owner:null");
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

test "regression: phase 15b HTMLImageElement modern reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><img id='photo' src='/photo.png' srcset='photo-1x.png 1x' sizes='100vw' loading='lazy' decoding='async' fetchpriority='low' crossorigin='anonymous' referrerpolicy='no-referrer' alt='Photo' width='320' height='240'><div id='out'></div><script>const photo = document.getElementById('photo'); const before = photo.src + ':' + photo.srcset + ':' + photo.sizes + ':' + photo.loading + ':' + photo.decoding + ':' + photo.fetchPriority + ':' + photo.crossOrigin + ':' + photo.referrerPolicy + ':' + photo.alt + ':' + photo.width + ':' + photo.height; photo.src = '/next.png'; photo.srcset = 'next-1x.png 1x'; photo.sizes = '50vw'; photo.loading = 'eager'; photo.decoding = 'sync'; photo.fetchPriority = 'high'; photo.crossOrigin = 'use-credentials'; photo.referrerPolicy = 'same-origin'; photo.alt = 'Next'; photo.width = 640; photo.height = 480; document.getElementById('out').textContent = before + '|' + photo.src + ':' + photo.srcset + ':' + photo.sizes + ':' + photo.loading + ':' + photo.decoding + ':' + photo.fetchPriority + ':' + photo.crossOrigin + ':' + photo.referrerPolicy + ':' + photo.alt + ':' + photo.width + ':' + photo.height + ':' + photo.getAttribute('src') + ':' + photo.getAttribute('srcset') + ':' + photo.getAttribute('sizes') + ':' + photo.getAttribute('loading') + ':' + photo.getAttribute('decoding') + ':' + photo.getAttribute('fetchpriority') + ':' + photo.getAttribute('crossorigin') + ':' + photo.getAttribute('referrerpolicy') + ':' + photo.getAttribute('alt') + ':' + photo.getAttribute('width') + ':' + photo.getAttribute('height');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "/photo.png:photo-1x.png 1x:100vw:lazy:async:low:anonymous:no-referrer:Photo:320:240|/next.png:next-1x.png 1x:50vw:eager:sync:high:use-credentials:same-origin:Next:640:480:/next.png:next-1x.png 1x:50vw:eager:sync:high:use-credentials:same-origin:Next:640:480");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15c HTMLImageElement useMap and isMap resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><img id='photo' src='/photo.png' alt='Photo' usemap='#map' ismap width='320' height='240'><div id='out'></div><script>const photo = document.getElementById('photo'); const before = photo.useMap + ':' + String(photo.isMap) + ':' + photo.width + ':' + photo.height + ':' + photo.currentSrc + ':' + String(photo.complete) + ':' + photo.naturalWidth + ':' + photo.naturalHeight; photo.useMap = '#next-map'; photo.isMap = false; photo.width = 640; photo.height = 480; document.getElementById('out').textContent = before + '|' + photo.useMap + ':' + String(photo.isMap) + ':' + photo.width + ':' + photo.height + ':' + photo.currentSrc + ':' + String(photo.complete) + ':' + photo.naturalWidth + ':' + photo.naturalHeight + ':' + photo.getAttribute('usemap') + ':' + photo.getAttribute('ismap');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "#map:true:320:240:/photo.png:true:0:0|#next-map:false:640:480:/photo.png:true:0:0:#next-map:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d2 HTMLSourceElement reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const source = document.createElement('source'); const before = source.src + ':' + source.srcset + ':' + source.sizes + ':' + source.media + ':' + source.type; source.src = '/next.avif'; source.srcset = 'next-1x.avif 1x'; source.sizes = '50vw'; source.media = 'print'; source.type = 'image/webp'; document.getElementById('out').textContent = before + '|' + source.src + ':' + source.srcset + ':' + source.sizes + ':' + source.media + ':' + source.type + ':' + source.getAttribute('src') + ':' + source.getAttribute('srcset') + ':' + source.getAttribute('sizes') + ':' + source.getAttribute('media') + ':' + source.getAttribute('type');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "::::|/next.avif:next-1x.avif 1x:50vw:print:image/webp:/next.avif:next-1x.avif 1x:50vw:print:image/webp",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d2a HTMLSourceElement responsive reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const source = document.createElement('source'); const before = source.srcset + ':' + source.sizes; source.srcset = 'next-1x.avif 1x'; source.sizes = '50vw'; document.getElementById('out').textContent = before + '|' + source.srcset + ':' + source.sizes + ':' + source.getAttribute('srcset') + ':' + source.getAttribute('sizes');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":|next-1x.avif 1x:50vw:next-1x.avif 1x:50vw");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d2ab HTMLSourceElement src resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const source = document.createElement('source'); const before = source.src + ':' + source.getAttribute('src'); source.src = '/next.avif'; document.getElementById('out').textContent = before + '|' + source.src + ':' + source.getAttribute('src');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":null|/next.avif:/next.avif");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d2b HTMLSourceElement media and type resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const source = document.createElement('source'); const before = source.media + ':' + source.type; source.media = 'print'; source.type = 'image/webp'; document.getElementById('out').textContent = before + '|' + source.media + ':' + source.type + ':' + source.getAttribute('media') + ':' + source.getAttribute('type');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":|print:image/webp:print:image/webp");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d2c HTMLSourceElement media resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const source = document.createElement('source'); const before = source.media + ':' + source.getAttribute('media'); source.media = 'print'; document.getElementById('out').textContent = before + '|' + source.media + ':' + source.getAttribute('media');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":null|print:print");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d2d HTMLSourceElement type resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const source = document.createElement('source'); const before = source.type + ':' + source.getAttribute('type'); source.type = 'image/webp'; document.getElementById('out').textContent = before + '|' + source.type + ':' + source.getAttribute('type');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":null|image/webp:image/webp");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d3 HTMLIFrameElement reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const frame = document.createElement('iframe'); frame.src = '/embed.html'; frame.srcdoc = '<p>hello</p>'; frame.loading = 'lazy'; frame.referrerPolicy = 'no-referrer'; frame.allow = 'fullscreen'; frame.sandbox = 'allow-forms allow-scripts allow-scripts'; frame.sandbox.add('allow-popups'); frame.sandbox.remove('allow-forms'); frame.sandbox.toggle('allow-modals'); frame.sandbox.replace('allow-scripts', 'allow-same-origin'); frame.allowFullscreen = true; frame.credentialless = true; frame.fetchPriority = 'high'; frame.width = '640'; frame.height = '360'; frame.name = 'preview'; document.getElementById('out').textContent = frame.src + ':' + frame.srcdoc + ':' + frame.loading + ':' + frame.referrerPolicy + ':' + frame.allow + ':' + frame.sandbox.value + ':' + String(frame.allowFullscreen) + ':' + String(frame.credentialless) + ':' + frame.fetchPriority + ':' + frame.width + ':' + frame.height + ':' + frame.name + ':' + String(frame.contentDocument) + ':' + String(frame.contentWindow) + ':' + frame.getAttribute('src') + ':' + frame.getAttribute('srcdoc') + ':' + frame.getAttribute('loading') + ':' + frame.getAttribute('referrerpolicy') + ':' + frame.getAttribute('allow') + ':' + frame.getAttribute('sandbox') + ':' + String(frame.hasAttribute('allowfullscreen')) + ':' + String(frame.hasAttribute('credentialless')) + ':' + frame.getAttribute('fetchpriority') + ':' + frame.getAttribute('width') + ':' + frame.getAttribute('height') + ':' + frame.getAttribute('name');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "/embed.html:<p>hello</p>:lazy:no-referrer:fullscreen:allow-same-origin allow-popups allow-modals:true:true:high:640:360:preview:null:null:/embed.html:<p>hello</p>:lazy:no-referrer:fullscreen:allow-same-origin allow-popups allow-modals:true:true:high:640:360:preview",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d3b HTMLIFrameElement sandbox reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const frame = document.createElement('iframe'); frame.sandbox = 'allow-forms allow-scripts allow-scripts'; const before = frame.sandbox.value + ':' + String(frame.sandbox.length) + ':' + String(frame.sandbox.contains('allow-forms')) + ':' + String(frame.sandbox.contains('allow-scripts')) + ':' + String(frame.hasAttribute('sandbox')); frame.sandbox.add('allow-popups'); frame.sandbox.remove('allow-forms'); frame.sandbox.toggle('allow-modals'); frame.sandbox.replace('allow-scripts', 'allow-same-origin'); document.getElementById('out').textContent = before + '|' + frame.sandbox.value + ':' + frame.getAttribute('sandbox') + ':' + String(frame.sandbox.length) + ':' + String(frame.sandbox.contains('allow-popups')) + ':' + String(frame.hasAttribute('sandbox'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "allow-forms allow-scripts:2:true:true:true|allow-same-origin allow-popups allow-modals:allow-same-origin allow-popups allow-modals:3:true:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d3b1 HTMLIFrameElement sandbox value resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const frame = document.createElement('iframe'); const before = frame.sandbox.value + ':' + String(frame.sandbox.length) + ':' + String(frame.hasAttribute('sandbox')); frame.sandbox = 'allow-forms allow-scripts'; document.getElementById('out').textContent = before + '|' + frame.sandbox.value + ':' + frame.getAttribute('sandbox') + ':' + String(frame.sandbox.length) + ':' + String(frame.hasAttribute('sandbox'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":0:false|allow-forms allow-scripts:allow-forms allow-scripts:2:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d3b2 HTMLIFrameElement sandbox methods resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const frame = document.createElement('iframe'); frame.sandbox = 'allow-forms allow-scripts allow-scripts'; const list = frame.sandbox; const before = list.value + ':' + String(list.length) + ':' + String(list.contains('allow-forms')) + ':' + String(list.contains('allow-scripts')) + ':' + String(frame.hasAttribute('sandbox')); list.add('allow-popups'); list.remove('allow-forms'); list.toggle('allow-modals'); list.replace('allow-scripts', 'allow-same-origin'); document.getElementById('out').textContent = before + '|' + list.value + ':' + frame.getAttribute('sandbox') + ':' + String(list.length) + ':' + String(list.contains('allow-popups')) + ':' + String(frame.hasAttribute('sandbox'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "allow-forms allow-scripts:2:true:true:true|allow-same-origin allow-popups allow-modals:allow-same-origin allow-popups allow-modals:3:true:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d3c HTMLIFrameElement fetchPriority resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const frame = document.createElement('iframe'); frame.fetchPriority = 'high'; document.getElementById('out').textContent = frame.fetchPriority + ':' + frame.getAttribute('fetchpriority') + ':' + String(frame.hasAttribute('fetchpriority'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "high:high:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d3d HTMLIFrameElement allowFullscreen resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const frame = document.createElement('iframe'); const before = String(frame.allowFullscreen) + ':' + String(frame.hasAttribute('allowfullscreen')); frame.allowFullscreen = true; document.getElementById('out').textContent = before + '|' + String(frame.allowFullscreen) + ':' + String(frame.hasAttribute('allowfullscreen'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:false|true:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d3e HTMLIFrameElement credentialless resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const frame = document.createElement('iframe'); const before = String(frame.credentialless) + ':' + String(frame.hasAttribute('credentialless')); frame.credentialless = true; document.getElementById('out').textContent = before + '|' + String(frame.credentialless) + ':' + String(frame.hasAttribute('credentialless'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:false|true:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d3f HTMLIFrameElement width and height resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const frame = document.createElement('iframe'); frame.width = '640'; frame.height = '360'; document.getElementById('out').textContent = frame.width + ':' + frame.height + ':' + frame.getAttribute('width') + ':' + frame.getAttribute('height');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "640:360:640:360");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d3g HTMLIFrameElement name resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const frame = document.createElement('iframe'); const before = frame.name + ':' + frame.getAttribute('name'); frame.name = 'preview'; document.getElementById('out').textContent = before + '|' + frame.name + ':' + frame.getAttribute('name');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":null|preview:preview");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d3h HTMLIFrameElement srcdoc resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const frame = document.createElement('iframe'); const before = frame.srcdoc + ':' + frame.getAttribute('srcdoc'); frame.srcdoc = '<p>hello</p>'; document.getElementById('out').textContent = before + '|' + frame.srcdoc + ':' + frame.getAttribute('srcdoc');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":null|<p>hello</p>:<p>hello</p>");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d3i HTMLIFrameElement loading resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const frame = document.createElement('iframe'); const before = frame.loading + ':' + frame.getAttribute('loading'); frame.loading = 'lazy'; document.getElementById('out').textContent = before + '|' + frame.loading + ':' + frame.getAttribute('loading');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":null|lazy:lazy");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d3j HTMLIFrameElement referrerPolicy resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const frame = document.createElement('iframe'); const before = frame.referrerPolicy + ':' + frame.getAttribute('referrerpolicy'); frame.referrerPolicy = 'no-referrer'; document.getElementById('out').textContent = before + '|' + frame.referrerPolicy + ':' + frame.getAttribute('referrerpolicy');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":null|no-referrer:no-referrer");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d3k HTMLIFrameElement allow resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const frame = document.createElement('iframe'); const before = frame.allow + ':' + frame.getAttribute('allow'); frame.allow = 'fullscreen'; document.getElementById('out').textContent = before + '|' + frame.allow + ':' + frame.getAttribute('allow');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":null|fullscreen:fullscreen");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d3l HTMLIFrameElement src resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const frame = document.createElement('iframe'); const before = frame.src + ':' + frame.getAttribute('src'); frame.src = '/embed.html'; document.getElementById('out').textContent = before + '|' + frame.src + ':' + frame.getAttribute('src');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":null|/embed.html:/embed.html");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d3m HTMLIFrameElement contentDocument and contentWindow resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const frame = document.createElement('iframe'); const before = String(frame.contentDocument) + ':' + String(frame.contentWindow); frame.src = '/embed.html'; frame.srcdoc = '<p>hello</p>'; document.getElementById('out').textContent = before + '|' + String(frame.contentDocument) + ':' + String(frame.contentWindow);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "null:null|null:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d4 HTMLMetaElement core reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const meta = document.createElement('meta'); const before = meta.name; meta.name = 'description'; document.getElementById('out').textContent = before + '|' + meta.name + ':' + meta.getAttribute('name');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "|description:description");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d4b HTMLMetaElement content reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const meta = document.createElement('meta'); const before = meta.content; meta.content = 'summary'; document.getElementById('out').textContent = before + '|' + meta.content + ':' + meta.getAttribute('content');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "|summary:summary");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d4c HTMLMetaElement httpEquiv reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const meta = document.createElement('meta'); const before = meta.httpEquiv; meta.httpEquiv = 'refresh'; document.getElementById('out').textContent = before + '|' + meta.httpEquiv + ':' + meta.getAttribute('http-equiv');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "|refresh:refresh");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d5 HTMLMetaElement charset and media reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const meta = document.createElement('meta'); const before = meta.charset + ':' + meta.media; meta.charset = 'utf-8'; meta.media = 'screen'; document.getElementById('out').textContent = before + '|' + meta.charset + ':' + meta.media + ':' + meta.getAttribute('charset') + ':' + meta.getAttribute('media');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":|utf-8:screen:utf-8:screen");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d5b HTMLMetaElement metadata reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const meta = document.createElement('meta'); const before = meta.name + ':' + meta.content + ':' + meta.httpEquiv; meta.name = 'description'; meta.content = 'summary'; meta.httpEquiv = 'refresh'; document.getElementById('out').textContent = before + '|' + meta.name + ':' + meta.content + ':' + meta.httpEquiv + ':' + meta.getAttribute('name') + ':' + meta.getAttribute('content') + ':' + meta.getAttribute('http-equiv');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "::|description:summary:refresh:description:summary:refresh");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d6 HTMLCanvasElement reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const canvas = document.createElement('canvas'); const before = String(canvas.width) + ':' + String(canvas.height) + ':' + String(canvas.getContext('2d')); canvas.width = 0; canvas.height = 1; const during = String(canvas.width) + ':' + String(canvas.height) + ':' + canvas.getAttribute('width') + ':' + canvas.getAttribute('height') + ':' + String(canvas.getContext('bitmaprenderer')); canvas.width = 640; canvas.height = 480; document.getElementById('out').textContent = before + '|' + during + '|' + String(canvas.width) + ':' + String(canvas.height) + ':' + canvas.getAttribute('width') + ':' + canvas.getAttribute('height');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "300:150:null|0:1:0:1:null|640:480:640:480");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d6b HTMLCanvasElement toDataURL resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const canvas = document.createElement('canvas'); const before = canvas.toDataURL(); canvas.width = 0; canvas.height = 0; const during = canvas.toDataURL('image/jpeg', 0.5); document.getElementById('out').textContent = before + '|' + during + '|' + canvas.toDataURL('image/png');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "data:image/png;base64,|data:,|data:,");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d6c HTMLCanvasElement toBlob resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const canvas = document.createElement('canvas'); canvas.width = 0; canvas.height = 0; const result = canvas.toBlob((blob) => { document.getElementById('out').setAttribute('data-seen', String(blob)); }, 'image/jpeg', 0.5); const seen = document.getElementById('out').getAttribute('data-seen'); document.getElementById('out').textContent = String(result) + ':' + seen + ':' + canvas.toDataURL('image/png');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "undefined:null:data:,");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d7 HTMLSlotElement reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><slot id='slot' name='main'><span id='fallback'>Fallback</span>text</slot><div id='out'></div><script>const slot = document.getElementById('slot'); const before = slot.name + ':' + String(slot.assignedNodes().length) + ':' + String(slot.assignedElements().length) + ':' + slot.assignedNodes().item(0).nodeName + ':' + slot.assignedNodes().item(1).nodeValue + ':' + slot.assignedElements().item(0).id; slot.name = 'next'; document.getElementById('out').textContent = before + '|' + slot.name + ':' + slot.getAttribute('name') + ':' + String(slot.assignedNodes().length) + ':' + String(slot.assignedElements().length);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "main:2:1:span:text:fallback|next:next:2:1");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d HTMLObjectElement reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='owner'></form><object id='media' form='owner' data='/movie.svg' type='image/svg+xml' name='viewer' width='640' height='480' usemap='#map'></object><div id='out'></div><script>const media = document.getElementById('media'); const before = media.data + ':' + media.type + ':' + media.name + ':' + media.width + ':' + media.height + ':' + media.useMap + ':' + media.form.id + ':' + String(media.contentDocument) + ':' + String(media.contentWindow) + ':' + String(media.getSVGDocument()) + ':' + String(media.willValidate) + ':' + String(media.validity.valid) + ':' + media.validationMessage + ':' + String(media.checkValidity()) + ':' + String(media.reportValidity()); media.data = '/updated.svg'; media.type = 'application/pdf'; media.name = 'updated'; media.width = '800'; media.height = '600'; media.useMap = '#updated-map'; media.setCustomValidity('broken'); document.getElementById('out').textContent = before + '|' + media.data + ':' + media.type + ':' + media.name + ':' + media.width + ':' + media.height + ':' + media.useMap + ':' + media.getAttribute('data') + ':' + media.getAttribute('type') + ':' + media.getAttribute('name') + ':' + media.getAttribute('width') + ':' + media.getAttribute('height') + ':' + media.getAttribute('usemap') + ':' + String(media.validity.valid) + ':' + media.validationMessage + ':' + String(media.checkValidity()) + ':' + String(media.reportValidity());</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "/movie.svg:image/svg+xml:viewer:640:480:#map:owner:null:null:null:true:true::true:true|/updated.svg:application/pdf:updated:800:600:#updated-map:/updated.svg:application/pdf:updated:800:600:#updated-map:false:broken:false:false",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d7a HTMLSlotElement assignedElements resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><slot id='slot' name='main'><span id='fallback'>Fallback</span>text</slot><div id='out'></div><script>const slot = document.getElementById('slot'); const before = String(slot.assignedElements().length) + ':' + slot.assignedElements().item(0).id; slot.name = 'next'; document.getElementById('out').textContent = before + '|' + String(slot.assignedElements().length) + ':' + slot.assignedElements().item(0).id + ':' + slot.getAttribute('name');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:fallback|1:fallback:next");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d7b HTMLSlotElement assignedNodes resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><slot id='slot' name='main'><span id='fallback'>Fallback</span>text</slot><div id='out'></div><script>const slot = document.getElementById('slot'); const before = String(slot.assignedNodes().length) + ':' + slot.assignedNodes().item(0).nodeName + ':' + slot.assignedNodes().item(1).nodeValue; slot.name = 'next'; document.getElementById('out').textContent = before + '|' + String(slot.assignedNodes().length) + ':' + slot.assignedNodes().item(0).nodeName + ':' + slot.assignedNodes().item(1).nodeValue + ':' + slot.getAttribute('name');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2:span:text|2:span:text:next");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 15d1 HTMLObjectElement null linkage and validity resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><object id='media' data='/movie.svg' type='image/svg+xml'></object><div id='out'></div><script>const media = document.getElementById('media'); const before = String(media.contentDocument) + ':' + String(media.contentWindow) + ':' + String(media.getSVGDocument()) + ':' + String(media.willValidate) + ':' + String(media.validity.valid) + ':' + media.validationMessage + ':' + String(media.checkValidity()) + ':' + String(media.reportValidity()); media.setCustomValidity('broken'); document.getElementById('out').textContent = before + '|' + String(media.contentDocument) + ':' + String(media.contentWindow) + ':' + String(media.getSVGDocument()) + ':' + String(media.willValidate) + ':' + String(media.validity.valid) + ':' + media.validationMessage + ':' + String(media.checkValidity()) + ':' + String(media.reportValidity());</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "null:null:null:true:true::true:true|null:null:null:true:false:broken:false:false");
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

test "regression: phase 17b document.styleSheets cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='first-style'>.primary { color: red; } .secondary { color: blue; }</style><link id='first-link' rel='stylesheet' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const sheets = document.styleSheets; const first = sheets.item(0); const second = sheets.item(1); const rules = first.cssRules; const before = String(rules.length) + ':' + String(second.cssRules.length) + ':' + String(rules.item(0)) + ':' + String(rules.item(0).selectorText) + ':' + String(rules.item(0).cssText); document.getElementById('root').innerHTML = document.getElementById('root').innerHTML; out.textContent = before + ':' + String(rules.length) + ':' + String(rules.item(1).selectorText);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2:0:[object CSSStyleRule]:.primary:.primary { color: red; }:2:.secondary");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17b2 CSSStyleRule.style resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; font-weight: bold; }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const style = rule.style; out.textContent = String(rule) + ':' + String(style) + ':' + style.cssText + ':' + String(style.length) + ':' + style.item(0) + ':' + style.item(1) + ':' + style.getPropertyValue('color') + ':' + style.getPropertyValue('font-weight') + ':' + style.getPropertyPriority('color');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSStyleRule]:color: red; font-weight: bold;:color: red; font-weight: bold;:2:color:font-weight:red:bold:",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17b2c CSSStyleRule.style mutation resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; font-weight: bold; }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const style = rule.style; const before = String(rule) + ':' + style.cssText + ':' + String(style.length) + ':' + style.item(0) + ':' + style.item(1) + ':' + style.getPropertyValue('color') + ':' + style.getPropertyValue('font-weight'); style.cssText = 'color: blue; font-style: italic;'; out.textContent = before + '|' + String(rule) + ':' + String(style) + ':' + style.cssText + ':' + String(style.length) + ':' + style.item(0) + ':' + style.item(1) + ':' + style.getPropertyValue('color') + ':' + style.getPropertyValue('font-style') + ':' + document.styleSheets.item(0).cssRules.item(0).cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSStyleRule]:color: red; font-weight: bold;:2:color:font-weight:red:bold|[object CSSStyleRule]:color: blue; font-style: italic;:color: blue; font-style: italic;:2:color:font-style:blue:italic:.primary { color: blue; font-style: italic; }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17b2b CSSStyleRule.selectorText resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='sheet'>.primary { color: red; }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const before = String(rule.selectorText) + ':' + String(rule.cssText); rule.selectorText = '.secondary'; out.textContent = before + '|' + String(rule.selectorText) + ':' + String(rule.cssText) + ':' + document.getElementById('sheet').textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        ".primary:.primary { color: red; }|.secondary:.secondary { color: red; }:.secondary { color: red; }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17c StyleSheetList and RadioNodeList forEach resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; } .secondary { color: blue; }</style><link rel='stylesheet' href='a.css'><form id='signup'><input type='radio' name='mode' id='mode-a' value='a'><input type='radio' name='mode' id='mode-b' value='b'></form></div><div id='out'></div><script>const sheets = document.styleSheets; sheets.forEach((sheet, index, list) => { document.getElementById('out').textContent += String(index) + ':' + String(sheet) + ':' + String(list) + '|'; }); const rules = sheets.item(0).cssRules; rules.forEach((rule, index, list) => { document.getElementById('out').textContent += String(index) + ':' + rule.selectorText + ':' + String(list) + '|'; }); const named = document.getElementById('signup').elements.namedItem('mode'); named.forEach((control, index, list) => { document.getElementById('out').textContent += String(index) + ':' + control.getAttribute('id') + ':' + String(list) + '|'; }); document.getElementById('root').textContent = 'gone';</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "0:[object CSSStyleSheet]:[object StyleSheetList]|1:[object CSSStyleSheet]:[object StyleSheetList]|0:.primary:[object CSSRuleList]|1:.secondary:[object CSSRuleList]|0:mode-a:[object RadioNodeList]|1:mode-b:[object RadioNodeList]|",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17d CSSStyleSheet insertRule and deleteRule resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const sheet = document.styleSheets.item(0); const inserted = sheet.insertRule('.secondary { color: blue; }'); const beforeDelete = String(inserted) + ':' + String(sheet.cssRules.length) + ':' + sheet.cssRules.item(0).selectorText + ':' + sheet.cssRules.item(1).selectorText; sheet.deleteRule(0); out.textContent = beforeDelete + ':' + String(sheet.cssRules.length) + ':' + sheet.cssRules.item(0).selectorText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "0:2:.secondary:.primary:1:.primary",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17e CSSStyleSheet.replaceSync resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='sheet'>.primary { color: red; }</style></div><div id='out'></div><script>const sheet = document.styleSheets.item(0); const before = sheet.cssRules.item(0).cssText; sheet.replaceSync('.secondary { color: blue; }'); document.getElementById('out').textContent = before + '|' + String(sheet.cssRules.length) + ':' + sheet.cssRules.item(0).selectorText + ':' + sheet.cssRules.item(0).cssText + ':' + document.getElementById('sheet').textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        ".primary { color: red; }|1:.secondary:.secondary { color: blue; }:.secondary { color: blue; }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17s2 CSSStyleSheet rules/addRule/removeRule resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const sheet = document.styleSheets.item(0); const rules = sheet.rules; const inserted = sheet.addRule('.secondary', 'color: blue;'); const beforeRemove = String(rules.length) + ':' + String(inserted) + ':' + rules.item(1).selectorText + ':' + rules.item(1).cssText; sheet.removeRule(0); out.textContent = beforeRemove + ':' + String(rules.length) + ':' + rules.item(0).selectorText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "2:1:.secondary:.secondary { color: blue; }:1:.secondary",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17e CSSMediaRule cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@media screen { .primary { color: red; } .secondary { color: blue; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const media = document.styleSheets.item(0).cssRules.item(0); const nested = media.cssRules; const list = media.media; const matches = String(media.matches); out.textContent = matches + ':' + String(media) + ':' + media.conditionText + ':' + String(list) + ':' + String(list.length) + ':' + list.item(0) + ':' + String(nested.length) + ':' + nested.item(0).selectorText + ':' + nested.item(1).selectorText + ':' + nested.item(0).cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "false:[object CSSMediaRule]:screen:screen:1:screen:2:.primary:.secondary:.primary { color: red; }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17e1 CSSMediaRule.conditionText resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='sheet'>@media screen and (min-width: 1px) { .primary { color: red; } .secondary { color: blue; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const before = String(rule) + ':' + rule.conditionText + ':' + String(rule.media) + ':' + rule.media.mediaText + ':' + String(rule.cssRules.length) + ':' + rule.cssRules.item(0).selectorText + ':' + rule.cssRules.item(1).selectorText; rule.conditionText = 'tv, speech'; const current = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(current) + ':' + current.conditionText + ':' + String(current.media) + ':' + current.media.mediaText + ':' + String(current.cssRules.length) + ':' + current.cssRules.item(0).selectorText + ':' + current.cssRules.item(1).selectorText + ':' + current.cssText + ':' + document.getElementById('sheet').textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSMediaRule]:screen and (min-width: 1px):screen and (min-width: 1px):screen and (min-width: 1px):2:.primary:.secondary|[object CSSMediaRule]:tv, speech:tv, speech:tv, speech:2:.primary:.secondary:@media tv, speech { .primary { color: red; }\n.secondary { color: blue; } }:@media tv, speech { .primary { color: red; }\n.secondary { color: blue; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17e1c CSSMediaRule.matches resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@media screen { .primary { color: red; } }</style></div><div id='out'></div><script>const media = document.styleSheets.item(0).cssRules.item(0); document.getElementById('out').textContent = String(media.matches);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17e1a CSSMediaRule insertRule and deleteRule resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='sheet'>@media screen { .primary { color: red; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const before = String(rule) + ':' + String(rule.cssRules.length) + ':' + rule.cssRules.item(0).selectorText; const inserted = rule.insertRule('.secondary { color: blue; }', 1); const afterInsert = document.styleSheets.item(0).cssRules.item(0); const snapshot = String(afterInsert) + ':' + String(afterInsert.cssRules.length) + ':' + afterInsert.cssRules.item(0).selectorText + ':' + afterInsert.cssRules.item(1).selectorText + ':' + afterInsert.cssText; afterInsert.deleteRule(0); const refreshed = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(inserted) + ':' + snapshot + '|' + String(refreshed.cssRules.length) + ':' + refreshed.cssRules.item(0).selectorText + ':' + refreshed.cssText + ':' + document.getElementById('sheet').textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSMediaRule]:1:.primary|1:[object CSSMediaRule]:2:.primary:.secondary:@media screen { .primary { color: red; }\n.secondary { color: blue; } }|1:.secondary:@media screen { .secondary { color: blue; } }:@media screen { .secondary { color: blue; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17e1b CSSSupportsRule insertRule and deleteRule resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='sheet'>@supports (display: grid) { .primary { color: red; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const before = String(rule) + ':' + rule.conditionText + ':' + String(rule.cssRules.length) + ':' + rule.cssRules.item(0).selectorText; const inserted = rule.insertRule('.secondary { color: blue; }', 1); const afterInsert = document.styleSheets.item(0).cssRules.item(0); const snapshot = String(afterInsert) + ':' + afterInsert.conditionText + ':' + String(afterInsert.cssRules.length) + ':' + afterInsert.cssRules.item(0).selectorText + ':' + afterInsert.cssRules.item(1).selectorText + ':' + afterInsert.cssText; afterInsert.deleteRule(0); const refreshed = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(inserted) + ':' + snapshot + '|' + String(refreshed) + ':' + refreshed.conditionText + ':' + String(refreshed.cssRules.length) + ':' + refreshed.cssRules.item(0).selectorText + ':' + refreshed.cssText + ':' + document.getElementById('sheet').textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSSupportsRule]:(display: grid):1:.primary|1:[object CSSSupportsRule]:(display: grid):2:.primary:.secondary:@supports (display: grid) { .primary { color: red; }\n.secondary { color: blue; } }|[object CSSSupportsRule]:(display: grid):1:.secondary:@supports (display: grid) { .secondary { color: blue; } }:@supports (display: grid) { .secondary { color: blue; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17e1c CSSContainerRule insertRule and deleteRule resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='sheet'>@container card (min-width: 1px) { .primary { color: red; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const before = String(rule) + ':' + rule.containerName + ':' + rule.containerQuery + ':' + String(rule.cssRules.length) + ':' + rule.cssRules.item(0).selectorText; const inserted = rule.insertRule('.secondary { color: blue; }', 1); const afterInsert = document.styleSheets.item(0).cssRules.item(0); const snapshot = String(afterInsert) + ':' + afterInsert.containerName + ':' + afterInsert.containerQuery + ':' + String(afterInsert.cssRules.length) + ':' + afterInsert.cssRules.item(0).selectorText + ':' + afterInsert.cssRules.item(1).selectorText + ':' + afterInsert.cssText; afterInsert.deleteRule(0); const refreshed = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(inserted) + ':' + snapshot + '|' + String(refreshed) + ':' + refreshed.containerName + ':' + refreshed.containerQuery + ':' + String(refreshed.cssRules.length) + ':' + refreshed.cssRules.item(0).selectorText + ':' + refreshed.cssText + ':' + document.getElementById('sheet').textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSContainerRule]:card:(min-width: 1px):1:.primary|1:[object CSSContainerRule]:card:(min-width: 1px):2:.primary:.secondary:@container card (min-width: 1px) { .primary { color: red; }\n.secondary { color: blue; } }|[object CSSContainerRule]:card:(min-width: 1px):1:.secondary:@container card (min-width: 1px) { .secondary { color: blue; } }:@container card (min-width: 1px) { .secondary { color: blue; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17e2 CSSDocumentRule cssText resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='sheet'>@document print { .primary { color: red; } .secondary { color: blue; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const before = String(rule) + ':' + rule.conditionText + ':' + String(rule.cssRules.length) + ':' + rule.cssRules.item(0).selectorText + ':' + rule.cssRules.item(1).selectorText; const current = document.styleSheets.item(0).cssRules.item(0); current.cssText = '@document speech { .primary { color: green; } .secondary { color: cyan; } }'; const refreshed = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(current) + ':' + current.conditionText + ':' + String(current.cssRules.length) + ':' + current.cssRules.item(0).selectorText + ':' + current.cssRules.item(1).selectorText + '|' + String(refreshed) + ':' + refreshed.conditionText + ':' + String(refreshed.cssRules.length) + ':' + refreshed.cssRules.item(0).selectorText + ':' + refreshed.cssRules.item(1).selectorText + ':' + refreshed.cssText + ':' + document.getElementById('sheet').textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSDocumentRule]:print:2:.primary:.secondary|[object CSSDocumentRule]:print:2:.primary:.secondary|[object CSSDocumentRule]:speech:2:.primary:.secondary:@document speech { .primary { color: green; } .secondary { color: cyan; } }:@document speech { .primary { color: green; } .secondary { color: cyan; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17e2 CSSDocumentRule conditionText resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='sheet'>@document print { .primary { color: red; } .secondary { color: blue; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const before = String(rule) + ':' + rule.conditionText + ':' + String(rule.cssRules.length) + ':' + rule.cssRules.item(0).selectorText + ':' + rule.cssRules.item(1).selectorText; rule.conditionText = 'speech'; const updated = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(updated) + ':' + updated.conditionText + ':' + String(updated.cssRules.length) + ':' + updated.cssRules.item(0).selectorText + ':' + updated.cssRules.item(1).selectorText + ':' + updated.cssText + ':' + document.getElementById('sheet').textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSDocumentRule]:print:2:.primary:.secondary|[object CSSDocumentRule]:speech:2:.primary:.secondary:@document speech { .primary { color: red; }\n.secondary { color: blue; } }:@document speech { .primary { color: red; }\n.secondary { color: blue; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17q CSSMediaRule media resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='out'></div><style>@media screen and (min-width: 1px) { .primary { color: red; } }</style><script>const rule = document.styleSheets.item(0).cssRules.item(0); const list = rule.media; const before = String(rule.matches) + ':' + String(rule) + ':' + rule.conditionText + ':' + String(list) + ':' + String(list.length) + ':' + list.item(0); list.appendMedium('print'); list.deleteMedium('screen and (min-width: 1px)'); list.mediaText = 'tv, speech'; const refreshed = document.styleSheets.item(0).cssRules.item(0); document.getElementById('out').textContent = before + ':' + String(refreshed.matches) + ':' + refreshed.media.mediaText + ':' + String(refreshed.media.length) + ':' + refreshed.media.item(0) + ':' + refreshed.media.item(1) + ':' + refreshed.conditionText + ':' + refreshed.media.mediaText + ':' + String(refreshed.media.length) + ':' + refreshed.media.item(0) + ':' + refreshed.media.item(1);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "false:[object CSSMediaRule]:screen and (min-width: 1px):screen and (min-width: 1px):1:screen and (min-width: 1px):false:tv, speech:2:tv:speech:tv, speech:tv, speech:2:tv:speech",
    );
    const html = subject.html().?;
    try std.testing.expect(std.mem.indexOf(u8, html, "Z") == null);
    try std.testing.expect(std.mem.indexOf(u8, html, "tv, speech") != null);
}

test "regression: phase 17r stylesheet owner element sheet resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='first-style' type='text/css' media='screen'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' type='text/css' href='a.css' media='print'></div><div id='out'></div><script>const out = document.getElementById('out'); const style = document.getElementById('first-style'); const link = document.getElementById('first-link'); const styleSheet = style.sheet; const linkSheet = link.sheet; style.media = 'tv'; link.media = 'speech'; style.type = 'text/plain'; link.type = 'application/css'; link.rel = 'preload'; link.href = 'b.css'; out.textContent = String(styleSheet) + ':' + String(linkSheet) + ':' + styleSheet.media.mediaText + ':' + linkSheet.media.mediaText + ':' + styleSheet.cssRules.item(0).selectorText + ':' + style.media + ':' + style.type + ':' + link.media + ':' + link.type + ':' + link.rel + ':' + link.href + ':' + String(link.sheet) + ':' + String(document.styleSheets.length);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSStyleSheet]:[object CSSStyleSheet]:tv:speech:.primary:tv:text/plain:speech:application/css:preload:b.css:null:1",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r1 stylesheet owner element disabled resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='first-style' disabled>.primary { color: red; }</style><link id='first-link' rel='stylesheet' disabled href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const style = document.getElementById('first-style'); const link = document.getElementById('first-link'); const styleSheet = style.sheet; const linkSheet = link.sheet; const before = String(style.disabled) + ':' + String(link.disabled) + ':' + String(styleSheet.disabled) + ':' + String(linkSheet.disabled); style.disabled = false; link.disabled = false; out.textContent = before + ':' + String(style.disabled) + ':' + String(link.disabled) + ':' + String(styleSheet.disabled) + ':' + String(linkSheet.disabled) + ':' + String(style.hasAttribute('disabled')) + ':' + String(link.hasAttribute('disabled'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "true:true:true:true:false:false:false:false:false:false",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r1a HTMLStyleElement disabled resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='first-style' disabled>.primary { color: red; }</style><div id='out'></div><script>const out = document.getElementById('out'); const style = document.getElementById('first-style'); const before = String(style.disabled) + ':' + String(style.getAttribute('disabled')); style.disabled = false; out.textContent = before + '|' + String(style.disabled) + ':' + String(style.getAttribute('disabled'));</script></div>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:|false:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r2 stylesheet owner element relList resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const link = document.getElementById('first-link'); const relList = link.relList; const sheets = document.styleSheets; const before = String(relList.length) + ':' + String(relList.contains('stylesheet')) + ':' + String(relList.supports('stylesheet')) + ':' + String(relList.supports('preload')) + ':' + String(relList.supports('bogus')) + ':' + String(sheets.length) + ':' + String(link.sheet) + ':' + link.href; relList.add('preload'); relList.replace('preload', 'modulepreload'); relList.remove('stylesheet'); link.href = 'b.css'; out.textContent = before + ':' + String(relList.length) + ':' + String(relList.contains('stylesheet')) + ':' + String(relList.contains('preload')) + ':' + String(relList.contains('modulepreload')) + ':' + String(relList.supports('stylesheet')) + ':' + String(relList.supports('preload')) + ':' + String(relList.supports('bogus')) + ':' + String(sheets.length) + ':' + String(link.sheet) + ':' + link.href;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "1:true:true:true:false:2:[object CSSStyleSheet]:a.css:1:false:false:true:true:true:false:1:null:b.css",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r2c stylesheet owner element relList replace resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><link id='first-link' rel='stylesheet preload stylesheet' href='a.css'></div><div id='out'></div><script>const link = document.getElementById('first-link'); const before = link.rel + ':' + link.relList.value + ':' + String(link.relList.replace('preload', 'modulepreload')) + ':' + String(link.relList.replace('missing', 'other')); document.getElementById('out').textContent = before + '|' + link.rel + ':' + link.relList.value + ':' + link.getAttribute('rel') + ':' + String(link.relList.contains('modulepreload')) + ':' + String(link.relList.contains('preload')) + ':' + String(link.sheet);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "stylesheet preload stylesheet:stylesheet preload:true:false|stylesheet modulepreload:stylesheet modulepreload:stylesheet modulepreload:true:false:[object CSSStyleSheet]",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r2b stylesheet owner element relList add remove and toggle resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; }</style><link id='first-link' rel='stylesheet preload stylesheet' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const link = document.getElementById('first-link'); const relList = link.relList; const before = link.rel + ':' + relList.value + ':' + String(relList.length) + ':' + String(relList.contains('stylesheet')); const toggled = relList.toggle('modulepreload'); relList.remove('preload'); relList.add('preconnect'); out.textContent = before + ':' + String(toggled) + ':' + link.rel + ':' + relList.value + ':' + link.getAttribute('rel') + ':' + String(relList.length) + ':' + String(relList.contains('stylesheet')) + ':' + String(relList.contains('modulepreload')) + ':' + String(relList.contains('preconnect'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "stylesheet preload stylesheet:stylesheet preload:2:true:true:stylesheet modulepreload preconnect:stylesheet modulepreload preconnect:stylesheet modulepreload preconnect:3:true:true:true",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r3 stylesheet owner element hreflang resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' hreflang='en' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const link = document.getElementById('first-link'); const sheets = document.styleSheets; const before = String(link.hreflang) + ':' + String(sheets.length) + ':' + String(link.sheet); link.hreflang = 'fr'; out.textContent = before + ':' + link.hreflang + ':' + String(sheets.length) + ':' + String(link.sheet) + ':' + link.href;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "en:2:[object CSSStyleSheet]:fr:2:[object CSSStyleSheet]:a.css",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r4 stylesheet owner element crossOrigin resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' crossorigin='anonymous' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const link = document.getElementById('first-link'); const sheets = document.styleSheets; const before = String(link.crossOrigin) + ':' + String(sheets.length) + ':' + String(link.sheet); link.crossOrigin = 'use-credentials'; out.textContent = before + ':' + link.crossOrigin + ':' + String(sheets.length) + ':' + String(link.sheet) + ':' + link.href;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "anonymous:2:[object CSSStyleSheet]:use-credentials:2:[object CSSStyleSheet]:a.css",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r5 stylesheet owner element referrerPolicy resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' referrerpolicy='no-referrer' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const link = document.getElementById('first-link'); const sheets = document.styleSheets; const before = String(link.referrerPolicy) + ':' + String(sheets.length) + ':' + String(link.sheet); link.referrerPolicy = 'same-origin'; out.textContent = before + ':' + link.referrerPolicy + ':' + String(sheets.length) + ':' + String(link.sheet) + ':' + link.href;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "no-referrer:2:[object CSSStyleSheet]:same-origin:2:[object CSSStyleSheet]:a.css",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 50 anchor and area hyperlink metadata resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><a id='anchor' hreflang='en' referrerpolicy='no-referrer' type='text/html' href='https://example.test/next'>Anchor</a><map name='map'><area id='area' hreflang='de' referrerpolicy='origin' type='text/html' href='https://example.test/files/diagram.png'></map></div><div id='out'></div><script>const out = document.getElementById('out'); const anchor = document.getElementById('anchor'); const area = document.querySelector('#area'); const before = String(anchor.hreflang) + ':' + String(anchor.referrerPolicy) + ':' + String(anchor.type) + ':' + String(area.hreflang) + ':' + String(area.referrerPolicy) + ':' + String(area.type); anchor.hreflang = 'fr'; anchor.referrerPolicy = 'same-origin'; anchor.type = 'application/xhtml+xml'; area.hreflang = 'it'; area.referrerPolicy = 'strict-origin'; area.type = 'text/plain'; out.textContent = before + '|' + anchor.hreflang + ':' + anchor.referrerPolicy + ':' + anchor.type + ':' + area.hreflang + ':' + area.referrerPolicy + ':' + area.type + ':' + anchor.getAttribute('hreflang') + ':' + anchor.getAttribute('referrerpolicy') + ':' + anchor.getAttribute('type') + ':' + area.getAttribute('hreflang') + ':' + area.getAttribute('referrerpolicy') + ':' + area.getAttribute('type');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "en:no-referrer:text/html:de:origin:text/html|fr:same-origin:application/xhtml+xml:it:strict-origin:text/plain:fr:same-origin:application/xhtml+xml:it:strict-origin:text/plain",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r6 stylesheet owner element integrity resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' integrity='sha384-abc' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const link = document.getElementById('first-link'); const sheets = document.styleSheets; const before = String(link.integrity) + ':' + String(sheets.length) + ':' + String(link.sheet); link.integrity = 'sha384-def'; out.textContent = before + ':' + link.integrity + ':' + String(sheets.length) + ':' + String(link.sheet) + ':' + link.href;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "sha384-abc:2:[object CSSStyleSheet]:sha384-def:2:[object CSSStyleSheet]:a.css",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r7 stylesheet owner element as resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' as='style' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const link = document.getElementById('first-link'); const sheets = document.styleSheets; const before = String(link.as) + ':' + String(sheets.length) + ':' + String(link.sheet); link.as = 'script'; out.textContent = before + ':' + link.as + ':' + String(sheets.length) + ':' + String(link.sheet) + ':' + link.href;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "style:2:[object CSSStyleSheet]:script:2:[object CSSStyleSheet]:a.css",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r7a HTMLLinkElement as resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' as='style' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const link = document.getElementById('first-link'); const before = String(link.as) + ':' + String(link.sheet); link.as = 'script'; out.textContent = before + ':' + link.as + ':' + String(link.sheet) + ':' + link.href;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "style:[object CSSStyleSheet]:script:[object CSSStyleSheet]:a.css",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r7b HTMLLinkElement href resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><link id='first-link' rel='stylesheet' href='a.css'></div><div id='out'></div><script>const link = document.getElementById('first-link'); const before = link.href + ':' + link.getAttribute('href'); link.href = 'b.css'; document.getElementById('out').textContent = before + '|' + link.href + ':' + link.getAttribute('href');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "a.css:a.css|b.css:b.css");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r7c HTMLLinkElement disabled resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><link id='first-link' rel='stylesheet' disabled href='a.css'></div><div id='out'></div><script>const link = document.getElementById('first-link'); const before = String(link.disabled) + ':' + String(link.sheet.disabled) + ':' + String(link.hasAttribute('disabled')); link.disabled = false; document.getElementById('out').textContent = before + '|' + String(link.disabled) + ':' + String(link.sheet.disabled) + ':' + String(link.hasAttribute('disabled'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:true:true|false:false:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r8 stylesheet owner element charset resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' charset='utf-8' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const link = document.getElementById('first-link'); const sheets = document.styleSheets; const before = String(link.charset) + ':' + String(sheets.length) + ':' + String(link.sheet); link.charset = 'windows-1252'; out.textContent = before + ':' + link.charset + ':' + String(sheets.length) + ':' + String(link.sheet) + ':' + link.href;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "utf-8:2:[object CSSStyleSheet]:windows-1252:2:[object CSSStyleSheet]:a.css",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r8b HTMLLinkElement charset resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' charset='utf-8' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const link = document.getElementById('first-link'); const before = String(link.charset); link.charset = 'windows-1252'; out.textContent = before + ':' + link.charset + ':' + link.getAttribute('charset') + ':' + String(link.sheet) + ':' + link.href;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "utf-8:windows-1252:windows-1252:[object CSSStyleSheet]:a.css",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r9a HTMLLinkElement imageSrcset resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const link = document.getElementById('first-link'); link.imageSrcset = 'a-1x.css 1x, a-2x.css 2x'; const before = String(link.imageSrcset) + ':' + String(link.sheet); link.imageSrcset = 'b-1x.css 1x'; out.textContent = before + '|' + link.imageSrcset + ':' + link.getAttribute('imagesrcset') + ':' + String(link.sheet) + ':' + link.href;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "a-1x.css 1x, a-2x.css 2x:[object CSSStyleSheet]|b-1x.css 1x:b-1x.css 1x:[object CSSStyleSheet]:a.css");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r9b HTMLLinkElement imageSizes resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const link = document.getElementById('first-link'); const before = String(link.imageSizes) + ':' + String(link.sheet); link.imageSizes = '100vw'; out.textContent = before + '|' + link.imageSizes + ':' + link.getAttribute('imagesizes') + ':' + String(link.sheet) + ':' + link.href;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":[object CSSStyleSheet]|100vw:100vw:[object CSSStyleSheet]:a.css");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r8a HTMLLinkElement type resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' type='text/css' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const link = document.getElementById('first-link'); const before = link.type + ':' + String(link.sheet); link.type = 'application/css'; out.textContent = before + '|' + link.type + ':' + String(link.sheet) + ':' + link.getAttribute('type');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "text/css:[object CSSStyleSheet]|application/css:[object CSSStyleSheet]:application/css");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r9 stylesheet owner element responsive image metadata resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' imagesrcset='a-1x.css 1x, a-2x.css 2x' imagesizes='100vw' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const link = document.getElementById('first-link'); const sheets = document.styleSheets; const before = String(link.imageSrcset) + ':' + String(link.imageSizes) + ':' + String(sheets.length) + ':' + String(link.sheet); link.imageSrcset = 'b-1x.css 1x'; link.imageSizes = '50vw'; out.textContent = before + ':' + link.imageSrcset + ':' + link.imageSizes + ':' + String(sheets.length) + ':' + String(link.sheet) + ':' + link.href;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "a-1x.css 1x, a-2x.css 2x:100vw:2:[object CSSStyleSheet]:b-1x.css 1x:50vw:2:[object CSSStyleSheet]:a.css",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r10 stylesheet owner element fetchPriority resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; }</style><link id='first-link' rel='stylesheet' fetchpriority='low' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const link = document.getElementById('first-link'); const sheets = document.styleSheets; const before = String(link.fetchPriority) + ':' + String(sheets.length) + ':' + String(link.sheet); link.fetchPriority = 'high'; out.textContent = before + ':' + link.fetchPriority + ':' + String(sheets.length) + ':' + String(link.sheet) + ':' + link.href;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "low:2:[object CSSStyleSheet]:high:2:[object CSSStyleSheet]:a.css",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r10a HTMLLinkElement fetchPriority resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><link id='first-link' rel='stylesheet' fetchpriority='low' href='a.css'></div><div id='out'></div><script>const link = document.getElementById('first-link'); const before = link.fetchPriority + ':' + link.getAttribute('fetchpriority'); link.fetchPriority = 'high'; document.getElementById('out').textContent = before + '|' + link.fetchPriority + ':' + link.getAttribute('fetchpriority');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "low:low|high:high");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r11 HTMLLinkElement media resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><link id='first-link' rel='stylesheet' media='print' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const link = document.getElementById('first-link'); const before = link.media + ':' + String(link.sheet) + ':' + link.sheet.media.mediaText; link.media = 'speech'; out.textContent = before + '|' + link.media + ':' + String(link.sheet) + ':' + link.sheet.media.mediaText + ':' + link.getAttribute('media');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "print:[object CSSStyleSheet]:print|speech:[object CSSStyleSheet]:speech:speech");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17f CSSSupportsRule cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@supports (display: grid) { .primary { color: red; } .secondary { color: blue; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const supports = document.styleSheets.item(0).cssRules.item(0); const nested = supports.cssRules; const before = String(supports) + ':' + supports.conditionText + ':' + String(nested.length) + ':' + nested.item(0).selectorText + ':' + nested.item(1).selectorText + ':' + nested.item(0).cssText; supports.conditionText = 'not (display: grid)'; const updated = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(updated) + ':' + updated.conditionText + ':' + updated.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSSupportsRule]:(display: grid):2:.primary:.secondary:.primary { color: red; }|[object CSSSupportsRule]:not (display: grid):@supports not (display: grid) { .primary { color: red; }\n.secondary { color: blue; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17g CSSKeyframesRule cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@keyframes pulse { from { opacity: 0; } to { opacity: 1; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const keyframes = document.styleSheets.item(0).cssRules.item(0); const nested = keyframes.cssRules; const before = String(keyframes) + ':' + keyframes.name + ':' + String(nested.length) + ':' + nested.item(0).keyText + ':' + nested.item(1).keyText + ':' + nested.item(1).cssText; nested.item(0).keyText = '25%'; const updated = document.styleSheets.item(0).cssRules.item(0).cssRules.item(0); out.textContent = before + '|' + String(updated) + ':' + updated.keyText + ':' + updated.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSKeyframesRule]:pulse:2:from:to:to { opacity: 1; }|[object CSSKeyframeRule]:25%:25% { opacity: 0; }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17g1 CSSKeyframesRule appendRule deleteRule and findRule resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@keyframes pulse { from { opacity: 0; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const keyframes = document.styleSheets.item(0).cssRules.item(0); const before = String(keyframes) + ':' + keyframes.name + ':' + String(keyframes.cssRules.length) + ':' + keyframes.cssRules.item(0).keyText; keyframes.appendRule('50% { opacity: 0.5; }'); const appended = document.styleSheets.item(0).cssRules.item(0); const found = appended.findRule('50%'); appended.deleteRule('from'); const deleted = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(appended) + ':' + String(appended.cssRules.length) + ':' + appended.cssRules.item(1).keyText + ':' + String(found) + ':' + found.keyText + ':' + found.cssText + '|' + String(deleted) + ':' + String(deleted.cssRules.length) + ':' + deleted.cssRules.item(0).keyText + ':' + deleted.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSKeyframesRule]:pulse:1:from|[object CSSKeyframesRule]:2:50%:[object CSSKeyframeRule]:50%:50% { opacity: 0.5; }|[object CSSKeyframesRule]:1:50%:@keyframes pulse { 50% { opacity: 0.5; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17g2 CSSKeyframesRule cssText mutations resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@keyframes pulse { from { opacity: 0; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const keyframes = document.styleSheets.item(0).cssRules.item(0); const before = String(keyframes) + ':' + keyframes.name + ':' + String(keyframes.cssRules.length) + ':' + keyframes.cssRules.item(0).keyText + ':' + keyframes.cssText; keyframes.cssText = '@keyframes pulse { 25% { opacity: 0.25; } }'; const current = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(current) + ':' + current.name + ':' + String(current.cssRules.length) + ':' + current.cssRules.item(0).keyText + ':' + current.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSKeyframesRule]:pulse:1:from:@keyframes pulse { from { opacity: 0; } }|[object CSSKeyframesRule]:pulse:1:25%:@keyframes pulse { 25% { opacity: 0.25; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17h CSSFontFaceRule cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@font-face { font-family: x; src: url(x.woff); }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const style = rule.style; const before = String(rule) + ':' + rule.cssText + ':' + String(style) + ':' + style.cssText + ':' + style.getPropertyValue('font-family') + ':' + style.getPropertyValue('src'); style.setProperty('font-family', 'y'); style.setProperty('src', 'url(y.woff)'); const updated = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(updated) + ':' + updated.cssText + ':' + updated.style.cssText + ':' + updated.style.getPropertyValue('font-family') + ':' + updated.style.getPropertyValue('src');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSFontFaceRule]:@font-face { font-family: x; src: url(x.woff); }:font-family: x; src: url(x.woff);:font-family: x; src: url(x.woff);:x:url(x.woff)|[object CSSFontFaceRule]:@font-face { font-family: y; src: url(y.woff); }:font-family: y; src: url(y.woff);:y:url(y.woff)",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17h1 CSSFontFaceRule cssText resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@font-face { font-family: x; src: url(x.woff); }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const before = String(rule) + ':' + rule.cssText + ':' + rule.style.cssText + ':' + rule.style.getPropertyValue('font-family') + ':' + rule.style.getPropertyValue('src'); rule.cssText = '@font-face { font-family: y; src: url(y.woff); }'; const updated = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(updated) + ':' + updated.cssText + ':' + updated.style.cssText + ':' + updated.style.getPropertyValue('font-family') + ':' + updated.style.getPropertyValue('src');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSFontFaceRule]:@font-face { font-family: x; src: url(x.woff); }:font-family: x; src: url(x.woff);:x:url(x.woff)|[object CSSFontFaceRule]:@font-face { font-family: y; src: url(y.woff); }:font-family: y; src: url(y.woff);:y:url(y.woff)",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17i CSSFontFeatureValuesRule cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@font-feature-values test { .x { color: red; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); out.textContent = String(rule) + ':' + rule.fontFamily + ':' + rule.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSFontFeatureValuesRule]:test:@font-feature-values test { .x { color: red; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17i1 CSSFontFeatureValuesRule cssText mutations resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@font-feature-values test { .x { color: red; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const sheet = document.styleSheets.item(0); const rule = sheet.cssRules.item(0); const before = String(rule) + ':' + rule.fontFamily + ':' + rule.cssText; rule.cssText = '@font-feature-values updated { .y { color: blue; } }'; const current = sheet.cssRules.item(0); out.textContent = before + '|' + String(current) + ':' + current.fontFamily + ':' + current.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSFontFeatureValuesRule]:test:@font-feature-values test { .x { color: red; } }|[object CSSFontFeatureValuesRule]:updated:@font-feature-values updated { .y { color: blue; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17i2 CSSFontPaletteValuesRule cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@font-palette-values --palette { font-family: Bungee Spice; base-palette: light; override-colors: 0 red; }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); out.textContent = String(rule) + ':' + rule.name + ':' + rule.fontFamily + ':' + rule.basePalette + ':' + rule.overrideColors + ':' + rule.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSFontPaletteValuesRule]:--palette:Bungee Spice:light:0 red:@font-palette-values --palette { font-family: Bungee Spice; base-palette: light; override-colors: 0 red; }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17i2j CSSFontPaletteValuesRule cssText mutations resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@font-palette-values --palette { font-family: Bungee Spice; base-palette: light; override-colors: 0 red; }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const sheet = document.styleSheets.item(0); const rule = sheet.cssRules.item(0); const before = String(rule) + ':' + rule.name + ':' + rule.fontFamily + ':' + rule.basePalette + ':' + rule.overrideColors + ':' + rule.cssText; rule.cssText = '@font-palette-values --theme { font-family: Bungee Spice; base-palette: dark; override-colors: 0 blue; }'; const current = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(current) + ':' + current.name + ':' + current.fontFamily + ':' + current.basePalette + ':' + current.overrideColors + ':' + current.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSFontPaletteValuesRule]:--palette:Bungee Spice:light:0 red:@font-palette-values --palette { font-family: Bungee Spice; base-palette: light; override-colors: 0 red; }|[object CSSFontPaletteValuesRule]:--theme:Bungee Spice:dark:0 blue:@font-palette-values --theme { font-family: Bungee Spice; base-palette: dark; override-colors: 0 blue; }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17i3 CSSColorProfileRule cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@color-profile --swopc { src: url(http://example.org/swop-coated.icc); rendering-intent: perceptual; components: cyan, magenta, yellow, black; }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); out.textContent = String(rule) + ':' + rule.name + ':' + rule.src + ':' + rule.renderingIntent + ':' + rule.components + ':' + rule.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSColorProfileRule]:--swopc:url(http://example.org/swop-coated.icc):perceptual:cyan, magenta, yellow, black:@color-profile --swopc { src: url(http://example.org/swop-coated.icc); rendering-intent: perceptual; components: cyan, magenta, yellow, black; }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17i3j CSSColorProfileRule cssText mutations resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@color-profile --swopc { src: url(http://example.org/swop-coated.icc); rendering-intent: perceptual; components: cyan, magenta, yellow, black; }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const sheet = document.styleSheets.item(0); const rule = sheet.cssRules.item(0); const before = String(rule) + ':' + rule.name + ':' + rule.src + ':' + rule.renderingIntent + ':' + rule.components + ':' + rule.cssText; rule.cssText = '@color-profile --swopc { src: url(http://example.org/swop-updated.icc); rendering-intent: perceptual; components: cyan, magenta, yellow, black; }'; const current = sheet.cssRules.item(0); out.textContent = before + '|' + String(current) + ':' + current.name + ':' + current.src + ':' + current.renderingIntent + ':' + current.components + ':' + current.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSColorProfileRule]:--swopc:url(http://example.org/swop-coated.icc):perceptual:cyan, magenta, yellow, black:@color-profile --swopc { src: url(http://example.org/swop-coated.icc); rendering-intent: perceptual; components: cyan, magenta, yellow, black; }|[object CSSColorProfileRule]:--swopc:url(http://example.org/swop-updated.icc):perceptual:cyan, magenta, yellow, black:@color-profile --swopc { src: url(http://example.org/swop-updated.icc); rendering-intent: perceptual; components: cyan, magenta, yellow, black; }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17i4 CSSRule.type resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; } @color-profile --swopc { src: url(http://example.org/swop-coated.icc); rendering-intent: perceptual; components: cyan, magenta, yellow, black; }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rules = document.styleSheets.item(0).cssRules; const styleRule = rules.item(0); const profileRule = rules.item(1); out.textContent = String(styleRule.type) + ':' + String(profileRule.type) + ':' + String(styleRule) + ':' + String(profileRule);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "1:0:[object CSSStyleRule]:[object CSSColorProfileRule]",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17i5 CSSRule.parentStyleSheet resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; } @media screen { .secondary { color: blue; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rules = document.styleSheets.item(0).cssRules; const styleRule = rules.item(0); const mediaRule = rules.item(1); const nestedRule = mediaRule.cssRules.item(0); out.textContent = String(styleRule.parentStyleSheet) + ':' + String(mediaRule.parentStyleSheet) + ':' + String(nestedRule.parentStyleSheet);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSStyleSheet]:[object CSSStyleSheet]:[object CSSStyleSheet]",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17i6 CSSRule.parentRule resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>.primary { color: red; } @media screen { .secondary { color: blue; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rules = document.styleSheets.item(0).cssRules; const styleRule = rules.item(0); const mediaRule = rules.item(1); const nestedRule = mediaRule.cssRules.item(0); out.textContent = String(styleRule.parentRule) + ':' + String(mediaRule.parentRule) + ':' + String(nestedRule.parentRule);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "null:null:[object CSSMediaRule]",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17j CSSContainerRule cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@container card (min-width: 1px) { .primary { color: red; } .secondary { color: blue; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const nested = rule.cssRules; const before = String(rule) + ':' + rule.containerName + ':' + rule.containerQuery + ':' + rule.conditionText + ':' + String(nested.length) + ':' + nested.item(0).selectorText + ':' + nested.item(1).selectorText + ':' + nested.item(0).cssText + ':' + rule.cssText; rule.conditionText = 'card (min-width: 2px)'; const updated = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(updated) + ':' + updated.containerName + ':' + updated.containerQuery + ':' + updated.conditionText + ':' + updated.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSContainerRule]:card:(min-width: 1px):card (min-width: 1px):2:.primary:.secondary:.primary { color: red; }:@container card (min-width: 1px) { .primary { color: red; } .secondary { color: blue; } }|[object CSSContainerRule]:card:(min-width: 2px):card (min-width: 2px):@container card (min-width: 2px) { .primary { color: red; }\n.secondary { color: blue; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17j1 CSSSupportsRule and CSSContainerRule cssText resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@supports (display: grid) { .supports { color: red; } .secondary { color: blue; } } @container card (min-width: 1px) { .container { color: purple; } .tertiary { color: green; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rules = document.styleSheets.item(0).cssRules; const supports = rules.item(0); const container = rules.item(1); const beforeSupports = String(supports) + ':' + supports.conditionText + ':' + String(supports.cssRules.length) + ':' + supports.cssRules.item(0).selectorText + ':' + supports.cssRules.item(1).selectorText; const beforeContainer = String(container) + ':' + container.containerName + ':' + container.containerQuery + ':' + container.conditionText + ':' + String(container.cssRules.length) + ':' + container.cssRules.item(0).selectorText + ':' + container.cssRules.item(1).selectorText; supports.cssText = '@supports (display: flex) { .supports { color: green; } .secondary { color: cyan; } }'; container.cssText = '@container card (min-width: 2px) { .container { color: orange; } .tertiary { color: yellow; } }'; const updatedSupports = document.styleSheets.item(0).cssRules.item(0); const updatedContainer = document.styleSheets.item(0).cssRules.item(1); out.textContent = beforeSupports + '|' + beforeContainer + '|' + String(updatedSupports) + ':' + updatedSupports.conditionText + ':' + String(updatedSupports.cssRules.length) + ':' + updatedSupports.cssRules.item(0).selectorText + ':' + updatedSupports.cssRules.item(1).selectorText + ':' + updatedSupports.cssText + '|' + String(updatedContainer) + ':' + updatedContainer.containerName + ':' + updatedContainer.containerQuery + ':' + updatedContainer.conditionText + ':' + String(updatedContainer.cssRules.length) + ':' + updatedContainer.cssRules.item(0).selectorText + ':' + updatedContainer.cssRules.item(1).selectorText + ':' + updatedContainer.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSSupportsRule]:(display: grid):2:.supports:.secondary|[object CSSContainerRule]:card:(min-width: 1px):card (min-width: 1px):2:.container:.tertiary|[object CSSSupportsRule]:(display: flex):2:.supports:.secondary:@supports (display: flex) { .supports { color: green; } .secondary { color: cyan; } }|[object CSSContainerRule]:card:(min-width: 2px):card (min-width: 2px):2:.container:.tertiary:@container card (min-width: 2px) { .container { color: orange; } .tertiary { color: yellow; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17j2 HTMLTableRowElement rowIndex, sectionRowIndex, and HTMLTableCellElement cellIndex resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<table id='table'><thead><tr><th>H</th></tr></thead><tbody><tr><td>B</td></tr></tbody><tfoot><tr><td>F</td></tr></tfoot></table><div id='out'></div><script>const table = document.getElementById('table'); const out = document.getElementById('out'); const headRow = table.rows.item(0); const bodyRow = table.rows.item(1); const footRow = table.rows.item(2); const bodyCell = bodyRow.cells.item(0); const detachedRow = document.createElement('tr'); const detachedCell = document.createElement('td'); detachedRow.append(detachedCell); out.textContent = String(headRow.rowIndex) + ':' + String(headRow.sectionRowIndex) + ':' + String(bodyRow.rowIndex) + ':' + String(bodyRow.sectionRowIndex) + ':' + String(footRow.rowIndex) + ':' + String(footRow.sectionRowIndex) + ':' + String(bodyCell.cellIndex) + ':' + String(detachedRow.rowIndex) + ':' + String(detachedRow.sectionRowIndex) + ':' + String(detachedCell.cellIndex);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "0:0:1:0:2:0:0:-1:-1:0",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17j3 HTMLTableCellElement colSpan and rowSpan resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<table id='table'><tr><td>A</td></tr></table><div id='out'></div><script>const table = document.getElementById('table'); const cell = table.rows.item(0).cells.item(0); const detached = document.createElement('td'); const before = String(cell.colSpan) + ':' + String(cell.rowSpan) + ':' + String(detached.colSpan) + ':' + String(detached.rowSpan); cell.colSpan = 3; cell.rowSpan = 2; detached.colSpan = 4; detached.rowSpan = 5; document.getElementById('out').textContent = before + '|' + String(cell.colSpan) + ':' + String(cell.rowSpan) + ':' + cell.getAttribute('colspan') + ':' + cell.getAttribute('rowspan') + ':' + String(detached.colSpan) + ':' + String(detached.rowSpan) + ':' + detached.getAttribute('colspan') + ':' + detached.getAttribute('rowspan');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "1:1:1:1|3:2:3:2:4:5:4:5",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17j4 HTMLTableHeaderCellElement headers scope and abbr resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<table id='table'><tr><th id='head' headers='left right' scope='col' abbr='Heading'>A</th></tr></table><div id='out'></div><script>const table = document.getElementById('table'); const cell = table.rows.item(0).cells.item(0); const detached = document.createElement('th'); const before = cell.headers + ':' + cell.scope + ':' + cell.abbr + ':' + detached.headers + ':' + detached.scope + ':' + detached.abbr; cell.headers = 'left center'; cell.scope = 'row'; cell.abbr = 'Row Heading'; detached.headers = 'top bottom'; detached.scope = 'colgroup'; detached.abbr = 'Detached'; document.getElementById('out').textContent = before + '|' + cell.headers + ':' + cell.scope + ':' + cell.abbr + ':' + cell.getAttribute('headers') + ':' + cell.getAttribute('scope') + ':' + cell.getAttribute('abbr') + ':' + detached.headers + ':' + detached.scope + ':' + detached.abbr + ':' + detached.getAttribute('headers') + ':' + detached.getAttribute('scope') + ':' + detached.getAttribute('abbr');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "left right:col:Heading:::|left center:row:Row Heading:left center:row:Row Heading:top bottom:colgroup:Detached:top bottom:colgroup:Detached",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17j5 HTMLTableColElement span and width resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<table id='table'><colgroup id='group' span='2' width='240px'><col id='col' span='3' width='120px'></colgroup><tbody><tr><td>A</td></tr></tbody></table><div id='out'></div><script>const col = document.getElementById('col'); const group = document.getElementById('group'); const detached = document.createElement('colgroup'); const before = 'col[span=' + col.span + ';width=' + col.width + ';align=' + col.align + ';ch=' + col.ch + ';chOff=' + col.chOff + ';vAlign=' + col.vAlign + ';bgColor=' + col.bgColor + ']|group[span=' + group.span + ';width=' + group.width + ';align=' + group.align + ';ch=' + group.ch + ';chOff=' + group.chOff + ';vAlign=' + group.vAlign + ';bgColor=' + group.bgColor + ']|detached[span=' + detached.span + ';width=' + detached.width + ';align=' + detached.align + ';ch=' + detached.ch + ';chOff=' + detached.chOff + ';vAlign=' + detached.vAlign + ';bgColor=' + detached.bgColor + ']'; col.span = 4; col.width = '100px'; col.align = 'left'; col.ch = '.'; col.chOff = '1'; col.vAlign = 'top'; col.bgColor = 'pink'; group.span = 5; group.width = '200px'; group.align = 'center'; group.ch = ':'; group.chOff = '2'; group.vAlign = 'middle'; group.bgColor = 'cyan'; detached.span = 6; detached.width = '300px'; detached.align = 'right'; detached.ch = '|'; detached.chOff = '3'; detached.vAlign = 'bottom'; detached.bgColor = 'orange'; document.getElementById('out').textContent = before + '|' + 'col[span=' + col.span + ';width=' + col.width + ';align=' + col.align + ';ch=' + col.ch + ';chOff=' + col.chOff + ';vAlign=' + col.vAlign + ';bgColor=' + col.bgColor + ';attrSpan=' + col.getAttribute('span') + ';attrWidth=' + col.getAttribute('width') + ';attrAlign=' + col.getAttribute('align') + ';attrChar=' + col.getAttribute('char') + ';attrCharOff=' + col.getAttribute('charoff') + ';attrVAlign=' + col.getAttribute('valign') + ';attrBgColor=' + col.getAttribute('bgcolor') + ']|group[span=' + group.span + ';width=' + group.width + ';align=' + group.align + ';ch=' + group.ch + ';chOff=' + group.chOff + ';vAlign=' + group.vAlign + ';bgColor=' + group.bgColor + ';attrSpan=' + group.getAttribute('span') + ';attrWidth=' + group.getAttribute('width') + ';attrAlign=' + group.getAttribute('align') + ';attrChar=' + group.getAttribute('char') + ';attrCharOff=' + group.getAttribute('charoff') + ';attrVAlign=' + group.getAttribute('valign') + ';attrBgColor=' + group.getAttribute('bgcolor') + ']|detached[span=' + detached.span + ';width=' + detached.width + ';align=' + detached.align + ';ch=' + detached.ch + ';chOff=' + detached.chOff + ';vAlign=' + detached.vAlign + ';bgColor=' + detached.bgColor + ';attrSpan=' + detached.getAttribute('span') + ';attrWidth=' + detached.getAttribute('width') + ';attrAlign=' + detached.getAttribute('align') + ';attrChar=' + detached.getAttribute('char') + ';attrCharOff=' + detached.getAttribute('charoff') + ';attrVAlign=' + detached.getAttribute('valign') + ';attrBgColor=' + detached.getAttribute('bgcolor') + ']';</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "col[span=3;width=120px;align=;ch=;chOff=;vAlign=;bgColor=]|group[span=2;width=240px;align=;ch=;chOff=;vAlign=;bgColor=]|detached[span=1;width=;align=;ch=;chOff=;vAlign=;bgColor=]|col[span=4;width=100px;align=left;ch=.;chOff=1;vAlign=top;bgColor=pink;attrSpan=4;attrWidth=100px;attrAlign=left;attrChar=.;attrCharOff=1;attrVAlign=top;attrBgColor=pink]|group[span=5;width=200px;align=center;ch=:;chOff=2;vAlign=middle;bgColor=cyan;attrSpan=5;attrWidth=200px;attrAlign=center;attrChar=:;attrCharOff=2;attrVAlign=middle;attrBgColor=cyan]|detached[span=6;width=300px;align=right;ch=|;chOff=3;vAlign=bottom;bgColor=orange;attrSpan=6;attrWidth=300px;attrAlign=right;attrChar=|;attrCharOff=3;attrVAlign=bottom;attrBgColor=orange]",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17j5b HTMLTableColElement legacy reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<table id='table'><colgroup id='group' span='2' width='240px'><col id='col' span='3' width='120px'></colgroup><tbody><tr><td>A</td></tr></tbody></table><div id='out'></div><script>const col = document.getElementById('col'); const group = document.getElementById('group'); const detached = document.createElement('colgroup'); const before = 'col[span=' + col.span + ';width=' + col.width + ';align=' + col.align + ';ch=' + col.ch + ';chOff=' + col.chOff + ';vAlign=' + col.vAlign + ';bgColor=' + col.bgColor + ']|group[span=' + group.span + ';width=' + group.width + ';align=' + group.align + ';ch=' + group.ch + ';chOff=' + group.chOff + ';vAlign=' + group.vAlign + ';bgColor=' + group.bgColor + ']|detached[span=' + detached.span + ';width=' + detached.width + ';align=' + detached.align + ';ch=' + detached.ch + ';chOff=' + detached.chOff + ';vAlign=' + detached.vAlign + ';bgColor=' + detached.bgColor + ']'; col.span = 4; col.width = '100px'; col.align = 'left'; col.ch = '.'; col.chOff = '1'; col.vAlign = 'top'; col.bgColor = 'pink'; group.span = 5; group.width = '200px'; group.align = 'center'; group.ch = ':'; group.chOff = '2'; group.vAlign = 'middle'; group.bgColor = 'cyan'; detached.span = 6; detached.width = '300px'; detached.align = 'right'; detached.ch = '|'; detached.chOff = '3'; detached.vAlign = 'bottom'; detached.bgColor = 'orange'; document.getElementById('out').textContent = before + '|' + 'col[span=' + col.span + ';width=' + col.width + ';align=' + col.align + ';ch=' + col.ch + ';chOff=' + col.chOff + ';vAlign=' + col.vAlign + ';bgColor=' + col.bgColor + ';attrSpan=' + col.getAttribute('span') + ';attrWidth=' + col.getAttribute('width') + ';attrAlign=' + col.getAttribute('align') + ';attrChar=' + col.getAttribute('char') + ';attrCharOff=' + col.getAttribute('charoff') + ';attrVAlign=' + col.getAttribute('valign') + ';attrBgColor=' + col.getAttribute('bgcolor') + ']|group[span=' + group.span + ';width=' + group.width + ';align=' + group.align + ';ch=' + group.ch + ';chOff=' + group.chOff + ';vAlign=' + group.vAlign + ';bgColor=' + group.bgColor + ';attrSpan=' + group.getAttribute('span') + ';attrWidth=' + group.getAttribute('width') + ';attrAlign=' + group.getAttribute('align') + ';attrChar=' + group.getAttribute('char') + ';attrCharOff=' + group.getAttribute('charoff') + ';attrVAlign=' + group.getAttribute('valign') + ';attrBgColor=' + group.getAttribute('bgcolor') + ']|detached[span=' + detached.span + ';width=' + detached.width + ';align=' + detached.align + ';ch=' + detached.ch + ';chOff=' + detached.chOff + ';vAlign=' + detached.vAlign + ';bgColor=' + detached.bgColor + ';attrSpan=' + detached.getAttribute('span') + ';attrWidth=' + detached.getAttribute('width') + ';attrAlign=' + detached.getAttribute('align') + ';attrChar=' + detached.getAttribute('char') + ';attrCharOff=' + detached.getAttribute('charoff') + ';attrVAlign=' + detached.getAttribute('valign') + ';attrBgColor=' + detached.getAttribute('bgcolor') + ']';</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "col[span=3;width=120px;align=;ch=;chOff=;vAlign=;bgColor=]|group[span=2;width=240px;align=;ch=;chOff=;vAlign=;bgColor=]|detached[span=1;width=;align=;ch=;chOff=;vAlign=;bgColor=]|col[span=4;width=100px;align=left;ch=.;chOff=1;vAlign=top;bgColor=pink;attrSpan=4;attrWidth=100px;attrAlign=left;attrChar=.;attrCharOff=1;attrVAlign=top;attrBgColor=pink]|group[span=5;width=200px;align=center;ch=:;chOff=2;vAlign=middle;bgColor=cyan;attrSpan=5;attrWidth=200px;attrAlign=center;attrChar=:;attrCharOff=2;attrVAlign=middle;attrBgColor=cyan]|detached[span=6;width=300px;align=right;ch=|;chOff=3;vAlign=bottom;bgColor=orange;attrSpan=6;attrWidth=300px;attrAlign=right;attrChar=|;attrCharOff=3;attrVAlign=bottom;attrBgColor=orange]",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17j5a HTMLTableElement legacy reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<table id='table' align='center' border='1' frame='box' rules='all' summary='Summary' width='100px' bgcolor='pink' cellpadding='2' cellspacing='3'></table><div id='out'></div><script>const table = document.getElementById('table'); const detached = document.createElement('table'); const before = 'table[align=' + table.align + ';border=' + table.border + ';frame=' + table.frame + ';rules=' + table.rules + ';summary=' + table.summary + ';width=' + table.width + ';bgColor=' + table.bgColor + ';cellPadding=' + table.cellPadding + ';cellSpacing=' + table.cellSpacing + ']|detached[align=' + detached.align + ';border=' + detached.border + ';frame=' + detached.frame + ';rules=' + detached.rules + ';summary=' + detached.summary + ';width=' + detached.width + ';bgColor=' + detached.bgColor + ';cellPadding=' + detached.cellPadding + ';cellSpacing=' + detached.cellSpacing + ']'; table.align = 'left'; table.border = '2'; table.frame = 'void'; table.rules = 'rows'; table.summary = 'Updated'; table.width = '200px'; table.bgColor = null; table.cellPadding = null; table.cellSpacing = null; detached.align = 'right'; detached.border = '3'; detached.frame = 'above'; detached.rules = 'cols'; detached.summary = 'Detached'; detached.width = '300px'; detached.bgColor = null; detached.cellPadding = null; detached.cellSpacing = null; document.getElementById('out').textContent = before + '|table[align=' + table.align + ';border=' + table.border + ';frame=' + table.frame + ';rules=' + table.rules + ';summary=' + table.summary + ';width=' + table.width + ';bgColor=' + table.bgColor + ';cellPadding=' + table.cellPadding + ';cellSpacing=' + table.cellSpacing + ';attrBgColor=' + String(table.getAttribute('bgcolor')) + ';attrCellPadding=' + String(table.getAttribute('cellpadding')) + ';attrCellSpacing=' + String(table.getAttribute('cellspacing')) + ']|detached[align=' + detached.align + ';border=' + detached.border + ';frame=' + detached.frame + ';rules=' + detached.rules + ';summary=' + detached.summary + ';width=' + detached.width + ';bgColor=' + detached.bgColor + ';cellPadding=' + detached.cellPadding + ';cellSpacing=' + detached.cellSpacing + ';attrBgColor=' + String(detached.getAttribute('bgcolor')) + ';attrCellPadding=' + String(detached.getAttribute('cellpadding')) + ';attrCellSpacing=' + String(detached.getAttribute('cellspacing')) + ']';</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "table[align=center;border=1;frame=box;rules=all;summary=Summary;width=100px;bgColor=pink;cellPadding=2;cellSpacing=3]|detached[align=;border=;frame=;rules=;summary=;width=;bgColor=;cellPadding=;cellSpacing=]|table[align=left;border=2;frame=void;rules=rows;summary=Updated;width=200px;bgColor=;cellPadding=;cellSpacing=;attrBgColor=;attrCellPadding=;attrCellSpacing=]|detached[align=right;border=3;frame=above;rules=cols;summary=Detached;width=300px;bgColor=;cellPadding=;cellSpacing=;attrBgColor=;attrCellPadding=;attrCellSpacing=]",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17j6a HTMLTableSectionElement HTMLTableRowElement and HTMLTableCellElement legacy reflection resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<table id='table' align='center' border='1' frame='box' rules='all' summary='Summary' width='100px' bgcolor='pink' cellpadding='2' cellspacing='3'></table><div id='out'></div><script>const table = document.getElementById('table'); const detached = document.createElement('table'); const before = 'table[align=' + table.align + ';border=' + table.border + ';frame=' + table.frame + ';rules=' + table.rules + ';summary=' + table.summary + ';width=' + table.width + ';bgColor=' + table.bgColor + ';cellPadding=' + table.cellPadding + ';cellSpacing=' + table.cellSpacing + ']|detached[align=' + detached.align + ';border=' + detached.border + ';frame=' + detached.frame + ';rules=' + detached.rules + ';summary=' + detached.summary + ';width=' + detached.width + ';bgColor=' + detached.bgColor + ';cellPadding=' + detached.cellPadding + ';cellSpacing=' + detached.cellSpacing + ']'; table.align = 'left'; table.border = '2'; table.frame = 'void'; table.rules = 'rows'; table.summary = 'Updated'; table.width = '200px'; table.bgColor = null; table.cellPadding = null; table.cellSpacing = null; detached.align = 'right'; detached.border = '3'; detached.frame = 'above'; detached.rules = 'cols'; detached.summary = 'Detached'; detached.width = '300px'; detached.bgColor = null; detached.cellPadding = null; detached.cellSpacing = null; document.getElementById('out').textContent = before + '|table[align=' + table.align + ';border=' + table.border + ';frame=' + table.frame + ';rules=' + table.rules + ';summary=' + table.summary + ';width=' + table.width + ';bgColor=' + table.bgColor + ';cellPadding=' + table.cellPadding + ';cellSpacing=' + table.cellSpacing + ';attrBgColor=' + String(table.getAttribute('bgcolor')) + ';attrCellPadding=' + String(table.getAttribute('cellpadding')) + ';attrCellSpacing=' + String(table.getAttribute('cellspacing')) + ']|detached[align=' + detached.align + ';border=' + detached.border + ';frame=' + detached.frame + ';rules=' + detached.rules + ';summary=' + detached.summary + ';width=' + detached.width + ';bgColor=' + detached.bgColor + ';cellPadding=' + detached.cellPadding + ';cellSpacing=' + detached.cellSpacing + ';attrBgColor=' + String(detached.getAttribute('bgcolor')) + ';attrCellPadding=' + String(detached.getAttribute('cellpadding')) + ';attrCellSpacing=' + String(detached.getAttribute('cellspacing')) + ']';</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "table[align=center;border=1;frame=box;rules=all;summary=Summary;width=100px;bgColor=pink;cellPadding=2;cellSpacing=3]|detached[align=;border=;frame=;rules=;summary=;width=;bgColor=;cellPadding=;cellSpacing=]|table[align=left;border=2;frame=void;rules=rows;summary=Updated;width=200px;bgColor=;cellPadding=;cellSpacing=;attrBgColor=;attrCellPadding=;attrCellSpacing=]|detached[align=right;border=3;frame=above;rules=cols;summary=Detached;width=300px;bgColor=;cellPadding=;cellSpacing=;attrBgColor=;attrCellPadding=;attrCellSpacing=]",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17j6 HTMLTableElement legacy reflection resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<table id='table' align='center' border='1' frame='box' rules='all' summary='Summary' width='100px' bgcolor='pink' cellpadding='2' cellspacing='3'><thead id='head' align='center' char='.' charoff='1' valign='top'><tr id='row' align='right' char=':' charoff='2' valign='middle' bgcolor='cyan'><td id='cell' align='left' axis='axis' height='10' width='20px' char='|' charoff='3' nowrap valign='bottom' bgcolor='pink'>A</td></tr></thead></table><div id='out'></div><script>const head = document.getElementById('head'); const row = document.getElementById('row'); const cell = document.getElementById('cell'); const detachedHead = document.createElement('thead'); const detachedRow = document.createElement('tr'); const detachedCell = document.createElement('td'); const before = 'head[align=' + head.align + ';ch=' + head.ch + ';chOff=' + head.chOff + ';vAlign=' + head.vAlign + ']|row[align=' + row.align + ';ch=' + row.ch + ';chOff=' + row.chOff + ';vAlign=' + row.vAlign + ';bgColor=' + row.bgColor + ']|cell[align=' + cell.align + ';axis=' + cell.axis + ';height=' + cell.height + ';width=' + cell.width + ';ch=' + cell.ch + ';chOff=' + cell.chOff + ';noWrap=' + String(cell.noWrap) + ';vAlign=' + cell.vAlign + ';bgColor=' + cell.bgColor + ']|detachedHead[align=' + detachedHead.align + ';ch=' + detachedHead.ch + ';chOff=' + detachedHead.chOff + ';vAlign=' + detachedHead.vAlign + ']|detachedRow[align=' + detachedRow.align + ';ch=' + detachedRow.ch + ';chOff=' + detachedRow.chOff + ';vAlign=' + detachedRow.vAlign + ';bgColor=' + detachedRow.bgColor + ']|detachedCell[align=' + detachedCell.align + ';axis=' + detachedCell.axis + ';height=' + detachedCell.height + ';width=' + detachedCell.width + ';ch=' + detachedCell.ch + ';chOff=' + detachedCell.chOff + ';noWrap=' + String(detachedCell.noWrap) + ';vAlign=' + detachedCell.vAlign + ';bgColor=' + detachedCell.bgColor + ']'; head.align = 'left'; head.ch = '*'; head.chOff = '4'; head.vAlign = 'bottom'; row.align = 'center'; row.ch = ';'; row.chOff = '5'; row.vAlign = 'top'; row.bgColor = null; cell.align = 'right'; cell.axis = 'y'; cell.height = '11'; cell.width = '30px'; cell.ch = '='; cell.chOff = '6'; cell.noWrap = false; cell.vAlign = 'middle'; cell.bgColor = null; detachedHead.align = 'justify'; detachedHead.ch = '!'; detachedHead.chOff = '7'; detachedHead.vAlign = 'baseline'; detachedRow.align = 'start'; detachedRow.ch = ','; detachedRow.chOff = '8'; detachedRow.vAlign = 'sub'; detachedRow.bgColor = null; detachedCell.align = 'end'; detachedCell.axis = 'z'; detachedCell.height = '12'; detachedCell.width = '40px'; detachedCell.ch = '~'; detachedCell.chOff = '9'; detachedCell.noWrap = true; detachedCell.vAlign = 'super'; detachedCell.bgColor = null; document.getElementById('out').textContent = before + '|head[align=' + head.align + ';ch=' + head.ch + ';chOff=' + head.chOff + ';vAlign=' + head.vAlign + ']|row[align=' + row.align + ';ch=' + row.ch + ';chOff=' + row.chOff + ';vAlign=' + row.vAlign + ';bgColor=' + row.bgColor + ';attrBgColor=' + row.getAttribute('bgcolor') + ']|cell[align=' + cell.align + ';axis=' + cell.axis + ';height=' + cell.height + ';width=' + cell.width + ';ch=' + cell.ch + ';chOff=' + cell.chOff + ';noWrap=' + String(cell.noWrap) + ';vAlign=' + cell.vAlign + ';bgColor=' + cell.bgColor + ';attrBgColor=' + cell.getAttribute('bgcolor') + ';attrNoWrap=' + String(cell.hasAttribute('nowrap')) + ']|detachedHead[align=' + detachedHead.align + ';ch=' + detachedHead.ch + ';chOff=' + detachedHead.chOff + ';vAlign=' + detachedHead.vAlign + ']|detachedRow[align=' + detachedRow.align + ';ch=' + detachedRow.ch + ';chOff=' + detachedRow.chOff + ';vAlign=' + detachedRow.vAlign + ';bgColor=' + detachedRow.bgColor + ';attrBgColor=' + detachedRow.getAttribute('bgcolor') + ']|detachedCell[align=' + detachedCell.align + ';axis=' + detachedCell.axis + ';height=' + detachedCell.height + ';width=' + detachedCell.width + ';ch=' + detachedCell.ch + ';chOff=' + detachedCell.chOff + ';noWrap=' + String(detachedCell.noWrap) + ';vAlign=' + detachedCell.vAlign + ';bgColor=' + detachedCell.bgColor + ';attrBgColor=' + detachedCell.getAttribute('bgcolor') + ';attrNoWrap=' + String(detachedCell.hasAttribute('nowrap')) + ']';</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "head[align=center;ch=.;chOff=1;vAlign=top]|row[align=right;ch=:;chOff=2;vAlign=middle;bgColor=cyan]|cell[align=left;axis=axis;height=10;width=20px;ch=|;chOff=3;noWrap=true;vAlign=bottom;bgColor=pink]|detachedHead[align=;ch=;chOff=;vAlign=]|detachedRow[align=;ch=;chOff=;vAlign=;bgColor=]|detachedCell[align=;axis=;height=;width=;ch=;chOff=;noWrap=false;vAlign=;bgColor=]|head[align=left;ch=*;chOff=4;vAlign=bottom]|row[align=center;ch=;;chOff=5;vAlign=top;bgColor=;attrBgColor=]|cell[align=right;axis=y;height=11;width=30px;ch==;chOff=6;noWrap=false;vAlign=middle;bgColor=;attrBgColor=;attrNoWrap=false]|detachedHead[align=justify;ch=!;chOff=7;vAlign=baseline]|detachedRow[align=start;ch=,;chOff=8;vAlign=sub;bgColor=;attrBgColor=]|detachedCell[align=end;axis=z;height=12;width=40px;ch=~;chOff=9;noWrap=true;vAlign=super;bgColor=;attrBgColor=;attrNoWrap=true]",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17k CSSSupportsConditionRule cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@supports-condition --thicker-underlines { text-decoration-thickness: 0.2em; text-underline-offset: 0.3em; }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const before = String(rule) + ':' + rule.name + ':' + String(rule.parentStyleSheet) + ':' + String(rule.parentRule) + ':' + String(rule.cssRules.length) + ':' + rule.cssText; rule.cssText = '@supports-condition --thicker-underlines { text-decoration-thickness: 0.4em; text-underline-offset: 0.6em; }'; const updated = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(updated) + ':' + updated.name + ':' + String(updated.parentStyleSheet) + ':' + String(updated.parentRule) + ':' + String(updated.cssRules.length) + ':' + updated.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSSupportsConditionRule]:--thicker-underlines:[object CSSStyleSheet]:null:0:@supports-condition --thicker-underlines { text-decoration-thickness: 0.2em; text-underline-offset: 0.3em; }|[object CSSSupportsConditionRule]:--thicker-underlines:[object CSSStyleSheet]:null:0:@supports-condition --thicker-underlines { text-decoration-thickness: 0.4em; text-underline-offset: 0.6em; }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17k CSSStartingStyleRule cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@starting-style { .primary { color: red; } .secondary { color: blue; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const nested = rule.cssRules; out.textContent = String(rule) + ':' + String(nested.length) + ':' + nested.item(0).selectorText + ':' + nested.item(1).selectorText + ':' + nested.item(0).cssText + ':' + rule.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSStartingStyleRule]:2:.primary:.secondary:.primary { color: red; }:@starting-style { .primary { color: red; } .secondary { color: blue; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17k1 CSSStartingStyleRule cssText mutations resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='sheet'>@starting-style { .primary { color: red; } .secondary { color: blue; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const sheet = document.styleSheets.item(0); const rule = sheet.cssRules.item(0); const before = String(rule) + ':' + String(rule.cssRules.length) + ':' + rule.cssRules.item(0).selectorText + ':' + rule.cssRules.item(1).selectorText + ':' + rule.cssText; rule.cssText = '@starting-style { .tertiary { color: green; } }'; const current = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(current) + ':' + String(current.cssRules.length) + ':' + current.cssRules.item(0).selectorText + ':' + current.cssText + ':' + document.getElementById('sheet').textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSStartingStyleRule]:2:.primary:.secondary:@starting-style { .primary { color: red; } .secondary { color: blue; } }|[object CSSStartingStyleRule]:1:.tertiary:@starting-style { .tertiary { color: green; } }:@starting-style { .tertiary { color: green; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17l CSSImportRule cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@import url(x.css) screen and (min-width: 1px);</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); out.textContent = String(rule) + ':' + rule.href + ':' + rule.mediaText + ':' + rule.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSImportRule]:x.css:screen and (min-width: 1px):@import url(x.css) screen and (min-width: 1px);",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17l1 CSSImportRule styleSheet resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@import url(x.css) screen and (min-width: 1px);</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const sheet = document.styleSheets.item(0); out.textContent = String(rule.styleSheet) + ':' + String(sheet.ownerRule);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "null:null",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17l1a CSSImportRule styleSheet and CSSStyleSheet.ownerRule resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@import url(x.css) screen and (min-width: 1px);</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const sheet = document.styleSheets.item(0); out.textContent = String(rule.styleSheet) + ':' + String(sheet.ownerRule) + ':' + String(rule.cssText);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "null:null:@import url(x.css) screen and (min-width: 1px);",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17l2 CSSImportRule supports/layer metadata resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@import url(x.css) layer(foo) supports(display: grid) screen and (min-width: 1px);</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); out.textContent = rule.layerName + ':' + rule.supportsText + ':' + rule.mediaText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "foo:display: grid:screen and (min-width: 1px)",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17l1 CSSCharsetRule cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@charset \"UTF-8\"; .primary { color: red; }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rules = document.styleSheets.item(0).cssRules; const charsetRule = rules.item(0); const styleRule = rules.item(1); out.textContent = String(charsetRule) + ':' + charsetRule.encoding + ':' + charsetRule.cssText + ':' + String(charsetRule.type) + ':' + String(styleRule.type);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSCharsetRule]:UTF-8:@charset \"UTF-8\";:2:1",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17l2 CSSStyleSheet ownerNode resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='first-style'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const first = document.styleSheets.item(0); const second = document.styleSheets.item(1); out.textContent = String(first.ownerNode) + ':' + first.ownerNode.getAttribute('id') + ':' + String(second.ownerNode) + ':' + second.ownerNode.getAttribute('id');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object Element]:first-style:[object Element]:first-link",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17l2a HTMLStyleElement.sheet and HTMLLinkElement.sheet resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><style id='style' media='screen'>.primary { color: red; }</style><link id='link' rel='stylesheet' media='print' href='a.css'></main><div id='out'></div><script>const style = document.getElementById('style'); const link = document.getElementById('link'); const styleSheet = style.sheet; const linkSheet = link.sheet; const before = String(styleSheet) + ':' + String(linkSheet) + ':' + styleSheet.media.mediaText + ':' + linkSheet.media.mediaText; style.media = 'tv'; link.media = 'speech'; document.getElementById('out').textContent = before + '|' + String(style.sheet) + ':' + String(link.sheet) + ':' + styleSheet.media.mediaText + ':' + linkSheet.media.mediaText + ':' + style.sheet.media.mediaText + ':' + link.sheet.media.mediaText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSStyleSheet]:[object CSSStyleSheet]:screen:print|[object CSSStyleSheet]:[object CSSStyleSheet]:tv:speech:tv:speech",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17l2b HTMLStyleElement media resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><style id='style' media='screen'>.primary { color: red; }</style><div id='out'></div><script>const style = document.getElementById('style'); const before = style.media + ':' + String(style.sheet) + ':' + style.sheet.media.mediaText; style.media = 'tv'; document.getElementById('out').textContent = before + '|' + style.media + ':' + String(style.sheet) + ':' + style.sheet.media.mediaText + ':' + style.getAttribute('media');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "screen:[object CSSStyleSheet]:screen|tv:[object CSSStyleSheet]:tv:tv");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17l2c HTMLStyleElement type resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><style id='style' type='text/css'>.primary { color: red; }</style><div id='out'></div><script>const style = document.getElementById('style'); const before = style.type + ':' + String(style.sheet); style.type = 'text/plain'; document.getElementById('out').textContent = before + '|' + style.type + ':' + style.getAttribute('type') + ':' + String(style.sheet);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "text/css:[object CSSStyleSheet]|text/plain:text/plain:[object CSSStyleSheet]");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17l3 CSSStyleSheet href/title resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='first-style'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' title='theme' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const first = document.styleSheets.item(0); const second = document.styleSheets.item(1); out.textContent = String(first.href) + ':' + String(first.title) + ':' + String(second.href) + ':' + String(second.title);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "null:null:a.css:theme",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17l4 CSSStyleSheet disabled resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='first-style' disabled>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css' disabled title='theme'></div><div id='out'></div><script>const out = document.getElementById('out'); const first = document.styleSheets.item(0); const second = document.styleSheets.item(1); first.disabled = false; out.textContent = String(first.disabled) + ':' + String(second.disabled) + ':' + String(document.getElementById('first-style').hasAttribute('disabled'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "false:true:false",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17s3 CSSStyleSheet href title disabled and media resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><style id='style' title='theme-a' disabled media='screen'>.primary { color: red; }</style><link id='link' rel='stylesheet' disabled title='theme-b' media='print' href='a.css'><div id='out'></div><script>const style = document.getElementById('style'); const link = document.getElementById('link'); const styleSheet = style.sheet; const linkSheet = link.sheet; const before = String(styleSheet.href) + ':' + String(styleSheet.title) + ':' + String(styleSheet.disabled) + ':' + styleSheet.media.mediaText + ':' + String(linkSheet.href) + ':' + String(linkSheet.title) + ':' + String(linkSheet.disabled) + ':' + linkSheet.media.mediaText; styleSheet.disabled = true; linkSheet.disabled = true; styleSheet.media.mediaText = 'tv'; linkSheet.media.mediaText = 'speech'; document.getElementById('out').textContent = before + '|' + String(styleSheet.href) + ':' + String(styleSheet.title) + ':' + String(styleSheet.disabled) + ':' + styleSheet.media.mediaText + ':' + String(style.getAttribute('disabled')) + ':' + String(link.getAttribute('disabled')) + ':' + String(linkSheet.href) + ':' + String(linkSheet.title) + ':' + String(linkSheet.disabled) + ':' + linkSheet.media.mediaText;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "null:theme-a:true:screen:a.css:theme-b:true:print|null:theme-a:true:tv:::a.css:theme-b:true:speech",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17m CSSNamespaceRule cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@namespace svg url(http://www.w3.org/2000/svg);</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); out.textContent = String(rule) + ':' + rule.prefix + ':' + rule.namespaceURI + ':' + rule.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSNamespaceRule]:svg:http://www.w3.org/2000/svg:@namespace svg url(http://www.w3.org/2000/svg);",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17m1 CSSNamespaceRule metadata resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@namespace svg url(http://www.w3.org/2000/svg);</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); out.textContent = String(rule) + ':' + rule.prefix + ':' + rule.namespaceURI + ':' + rule.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSNamespaceRule]:svg:http://www.w3.org/2000/svg:@namespace svg url(http://www.w3.org/2000/svg);",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17n CSSPageRule cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@page :first { margin: 1cm; }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const style = rule.style; const before = String(rule) + ':' + rule.selectorText + ':' + String(style) + ':' + style.cssText + ':' + style.getPropertyValue('margin'); style.cssText = 'margin: 2cm;'; const updated = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(updated) + ':' + updated.selectorText + ':' + String(updated.style) + ':' + updated.style.cssText + ':' + updated.style.getPropertyValue('margin');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSPageRule]::first:margin: 1cm;:margin: 1cm;:1cm|[object CSSPageRule]::first:margin: 2cm;:margin: 2cm;:2cm",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17n1 CSSPageRule.selectorText resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='sheet'>@page :first { margin: 1cm; }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const before = String(rule.selectorText) + ':' + String(rule.cssText); rule.selectorText = ':left'; const current = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(current.selectorText) + ':' + String(current.cssText) + ':' + document.getElementById('sheet').textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        ":first:@page :first { margin: 1cm; }|:left:@page :left { margin: 1cm; }:@page :left { margin: 1cm; }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17n2 CSSPageRule.cssText resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='sheet'>@page :first { margin: 1cm; }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const before = String(rule.selectorText) + ':' + String(rule.cssText); rule.cssText = '@page :left { margin: 2cm; }'; const current = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(current.selectorText) + ':' + String(current.cssText) + ':' + document.getElementById('sheet').textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        ":first:@page :first { margin: 1cm; }|:left:@page :left { margin: 2cm; }:@page :left { margin: 2cm; }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17n3 CSSMediaRule.cssText resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='sheet'>@media screen { .primary { color: red; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const before = String(rule.cssText); rule.cssText = '@media print { .secondary { color: blue; } }'; const current = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(current.conditionText) + ':' + String(current.cssText) + ':' + document.getElementById('sheet').textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "@media screen { .primary { color: red; } }|print:@media print { .secondary { color: blue; } }:@media print { .secondary { color: blue; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17b2b1 CSSStyleRule.cssText resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='sheet'>.primary { color: red; }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const before = String(rule.cssText); rule.cssText = '.secondary { color: blue; }'; const current = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(current.selectorText) + ':' + String(current.cssText) + ':' + document.getElementById('sheet').textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        ".primary { color: red; }|.secondary:.secondary { color: blue; }:.secondary { color: blue; }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17o CSSLayerBlockRule cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@layer base { .primary { color: red; } .secondary { color: blue; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const nested = rule.cssRules; out.textContent = String(rule) + ':' + rule.nameText + ':' + String(nested.length) + ':' + nested.item(0).selectorText + ':' + nested.item(1).selectorText + ':' + nested.item(0).cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSLayerBlockRule]:base:2:.primary:.secondary:.primary { color: red; }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17o1 CSSLayerStatementRule resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='out'></div><style>@layer base, theme;</style><script>const rule = document.styleSheets.item(0).cssRules.item(0); document.getElementById('out').textContent = String(rule) + ':' + rule.nameText + ':' + rule.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[object CSSLayerStatementRule]:base, theme:@layer base, theme;");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17o2 CSSLayerBlockRule nameText mutations resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='sheet'>@layer base { .primary { color: red; } .secondary { color: blue; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const nested = rule.cssRules; const before = String(rule) + ':' + rule.nameText + ':' + String(nested.length) + ':' + nested.item(0).selectorText + ':' + nested.item(1).selectorText + ':' + rule.cssText; rule.nameText = 'theme'; const current = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(current) + ':' + current.nameText + ':' + String(current.cssRules.length) + ':' + current.cssRules.item(0).selectorText + ':' + current.cssText + ':' + document.getElementById('sheet').textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSLayerBlockRule]:base:2:.primary:.secondary:@layer base { .primary { color: red; } .secondary { color: blue; } }|[object CSSLayerBlockRule]:theme:2:.primary:@layer theme { .primary { color: red; }\n.secondary { color: blue; } }:@layer theme { .primary { color: red; }\n.secondary { color: blue; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17o3 CSSLayerStatementRule nameText mutations resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@layer base, theme;</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const before = String(rule) + ':' + rule.nameText + ':' + rule.cssText; rule.nameText = 'updated'; const current = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(current) + ':' + current.nameText + ':' + current.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSLayerStatementRule]:base, theme:@layer base, theme;|[object CSSLayerStatementRule]:updated:@layer updated;",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17o4 CSSLayerStatementRule cssText mutations resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@layer base, theme;</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const before = String(rule) + ':' + rule.nameText + ':' + rule.cssText; rule.cssText = '@layer updated, theme;'; const current = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(current) + ':' + current.nameText + ':' + current.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSLayerStatementRule]:base, theme:@layer base, theme;|[object CSSLayerStatementRule]:updated, theme:@layer updated, theme;",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17o5 CSSLayerStatementRule nameList resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@layer base, theme, ui;</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const before = String(rule) + ':' + String(rule.nameList) + ':' + String(rule.nameList.length) + ':' + rule.nameList.item(0) + ':' + rule.nameList.item(1) + ':' + rule.nameList.item(2); rule.nameText = 'updated, theme'; const current = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(current) + ':' + String(current.nameList) + ':' + String(current.nameList.length) + ':' + current.nameList.item(0) + ':' + current.nameList.item(1) + ':' + current.nameList.contains('updated') + ':' + current.nameList.contains('theme');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSLayerStatementRule]:[object DOMStringList]:3:base:theme:ui|[object CSSLayerStatementRule]:[object DOMStringList]:2:updated:theme:true:true",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17p CSSCounterStyleRule cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@counter-style thumbs { system: cyclic; symbols: a b; negative: '-' '+'; prefix: pre; suffix: post; range: 1 3; pad: 2 0; fallback: decimal; speak-as: bullets; additive-symbols: 1 '*' 2 '**'; }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); out.textContent = String(rule) + ':' + rule.name + ':' + rule.system + ':' + rule.symbols + ':' + rule.negative + ':' + rule.prefix + ':' + rule.suffix + ':' + rule.range + ':' + rule.pad + ':' + rule.fallback + ':' + rule.speakAs + ':' + rule.additiveSymbols + ':' + rule.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSCounterStyleRule]:thumbs:cyclic:a b:'-' '+':pre:post:1 3:2 0:decimal:bullets:1 '*' 2 '**':@counter-style thumbs { system: cyclic; symbols: a b; negative: '-' '+'; prefix: pre; suffix: post; range: 1 3; pad: 2 0; fallback: decimal; speak-as: bullets; additive-symbols: 1 '*' 2 '**'; }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17p2 CSSPropertyRule cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@property --accent { syntax: \"<color>\"; inherits: false; initial-value: red; }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); out.textContent = String(rule) + ':' + rule.name + ':' + rule.syntax + ':' + String(rule.inherits) + ':' + rule.initialValue + ':' + rule.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSPropertyRule]:--accent:\"<color>\":false:red:@property --accent { syntax: \"<color>\"; inherits: false; initial-value: red; }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17p2j CSSCounterStyleRule and CSSPropertyRule cssText mutations resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@counter-style thumbs { system: cyclic; symbols: a b; negative: '-' '+'; prefix: pre; suffix: post; range: 1 3; pad: 2 0; fallback: decimal; speak-as: bullets; additive-symbols: 1 '*' 2 '**'; } @property --accent { syntax: \"<color>\"; inherits: false; initial-value: red; }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const sheet = document.styleSheets.item(0); const counter = sheet.cssRules.item(0); const property = sheet.cssRules.item(1); counter.cssText = \"@counter-style glyphs { system: fixed; symbols: a b; negative: '-' '+'; prefix: pre; suffix: post; range: 1 3; pad: 2 0; fallback: decimal; speak-as: bullets; additive-symbols: 1 '*' 2 '**'; }\"; property.cssText = '@property --gap { syntax: \"<length>\"; inherits: true; initial-value: 2px; }'; const updatedCounter = sheet.cssRules.item(0); const updatedProperty = sheet.cssRules.item(1); out.textContent = String(updatedCounter) + ':' + updatedCounter.name + ':' + updatedCounter.system + ':' + updatedCounter.cssText + '|' + String(updatedProperty) + ':' + updatedProperty.name + ':' + updatedProperty.syntax + ':' + String(updatedProperty.inherits) + ':' + updatedProperty.initialValue + ':' + updatedProperty.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSCounterStyleRule]:glyphs:fixed:@counter-style glyphs { system: fixed; symbols: a b; negative: '-' '+'; prefix: pre; suffix: post; range: 1 3; pad: 2 0; fallback: decimal; speak-as: bullets; additive-symbols: 1 '*' 2 '**'; }|[object CSSPropertyRule]:--gap:\"<length>\":true:2px:@property --gap { syntax: \"<length>\"; inherits: true; initial-value: 2px; }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17q CSSScopeRule cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@scope (.root) to (.leaf) { .primary { color: red; } .secondary { color: blue; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const nested = rule.cssRules; out.textContent = String(rule) + ':' + rule.start + ':' + rule.end + ':' + String(nested.length) + ':' + nested.item(0).selectorText + ':' + nested.item(1).selectorText + ':' + nested.item(0).cssText + ':' + rule.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSScopeRule]:.root:.leaf:2:.primary:.secondary:.primary { color: red; }:@scope (.root) to (.leaf) { .primary { color: red; } .secondary { color: blue; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17q1 CSSScopeRule cssText mutations resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='sheet'>@scope (.root) to (.leaf) { .primary { color: red; } .secondary { color: blue; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const sheet = document.styleSheets.item(0); const rule = sheet.cssRules.item(0); const before = String(rule) + ':' + rule.start + ':' + rule.end + ':' + String(rule.cssRules.length) + ':' + rule.cssRules.item(0).selectorText + ':' + rule.cssRules.item(1).selectorText + ':' + rule.cssText; rule.cssText = '@scope (.root) to (.leaf) { .tertiary { color: green; } }'; const current = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(current) + ':' + current.start + ':' + current.end + ':' + String(current.cssRules.length) + ':' + current.cssRules.item(0).selectorText + ':' + current.cssText + ':' + document.getElementById('sheet').textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSScopeRule]:.root:.leaf:2:.primary:.secondary:@scope (.root) to (.leaf) { .primary { color: red; } .secondary { color: blue; } }|[object CSSScopeRule]:.root:.leaf:1:.tertiary:@scope (.root) to (.leaf) { .tertiary { color: green; } }:@scope (.root) to (.leaf) { .tertiary { color: green; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r CSSPositionTryRule cssRules resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@position-try card { .primary { color: red; } .secondary { color: blue; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); out.textContent = String(rule) + ':' + rule.name + ':' + rule.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSPositionTryRule]:card:@position-try card { .primary { color: red; } .secondary { color: blue; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17r1 CSSPositionTryRule cssText mutations resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='sheet'>@position-try card { .primary { color: red; } }</style></div><div id='out'></div><script>const out = document.getElementById('out'); const sheet = document.styleSheets.item(0); const rule = sheet.cssRules.item(0); const before = String(rule) + ':' + rule.name + ':' + rule.cssText; rule.cssText = '@position-try docked { .secondary { color: blue; } }'; const current = document.styleSheets.item(0).cssRules.item(0); out.textContent = before + '|' + String(current) + ':' + current.name + ':' + current.cssText + ':' + document.getElementById('sheet').textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSPositionTryRule]:card:@position-try card { .primary { color: red; } }|[object CSSPositionTryRule]:docked:@position-try docked { .secondary { color: blue; } }:@position-try docked { .secondary { color: blue; } }",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17s CSSStyleSheet media resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='first-style' media='print'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' media='screen and (min-width: 1px), print' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const sheets = document.styleSheets; const first = sheets.item(0).media; const second = sheets.item(1).media; out.textContent = String(first) + ':' + first.mediaText + ':' + String(first.length) + ':' + first.item(0) + ':' + String(second) + ':' + second.mediaText + ':' + String(second.length) + ':' + second.item(0) + ':' + second.item(1);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "print:print:1:print:screen and (min-width: 1px), print:screen and (min-width: 1px), print:2:screen and (min-width: 1px):print",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17s1 CSSStyleSheet media mutation resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='first-style' media='print'>.primary { color: red; }</style></div><div id='out'></div><script>const style = document.getElementById('first-style'); const media = document.styleSheets.item(0).media; media.appendMedium('screen'); media.deleteMedium('print'); document.getElementById('out').textContent = String(media) + ':' + media.mediaText + ':' + String(media.length) + ':' + media.item(0) + ':' + style.getAttribute('media');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "screen:screen:1:screen:screen");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17s2 CSSStyleSheet mediaText mutation resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style id='first-style' media='print'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' media='screen and (min-width: 1px), print' href='a.css'></div><div id='out'></div><script>const out = document.getElementById('out'); const styleMedia = document.styleSheets.item(0).media; const linkMedia = document.styleSheets.item(1).media; const before = String(styleMedia) + ':' + String(linkMedia) + ':' + document.getElementById('first-style').getAttribute('media') + ':' + document.getElementById('first-link').getAttribute('media'); styleMedia.mediaText = 'tv, speech'; linkMedia.mediaText = 'print'; out.textContent = before + ':' + String(styleMedia) + ':' + String(linkMedia) + ':' + document.getElementById('first-style').getAttribute('media') + ':' + document.getElementById('first-link').getAttribute('media');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "print:screen and (min-width: 1px), print:print:screen and (min-width: 1px), print:tv, speech:print:tv, speech:print",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 17s1 CSSImportRule media resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='root'><style>@import url(x.css) screen and (min-width: 1px), print;</style></div><div id='out'></div><script>const out = document.getElementById('out'); const rule = document.styleSheets.item(0).cssRules.item(0); const media = rule.media; out.textContent = String(rule) + ':' + rule.href + ':' + rule.mediaText + ':' + String(media) + ':' + media.mediaText + ':' + String(media.length) + ':' + media.item(0) + ':' + media.item(1) + ':' + rule.cssText;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object CSSImportRule]:x.css:screen and (min-width: 1px), print:screen and (min-width: 1px), print:screen and (min-width: 1px), print:2:screen and (min-width: 1px):print:@import url(x.css) screen and (min-width: 1px), print;",
    );
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

test "regression: phase 18b table.insertRow and tr.insertCell resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<table id='table'><tbody id='body'><tr id='first-row'><td id='first-cell'>A</td></tr></tbody></table><div id='out'></div><script>const table = document.getElementById('table'); const beforeRow = table.insertRow(0); beforeRow.setAttribute('id', 'before'); const beforeCell = beforeRow.insertCell(); beforeCell.setAttribute('id', 'before-cell'); beforeCell.textContent = 'B'; const afterRow = table.insertRow(); afterRow.id = 'after'; const afterFirst = afterRow.insertCell(); afterFirst.id = 'after-first'; afterFirst.textContent = 'C'; const afterSecond = afterRow.insertCell(); afterSecond.id = 'after-second'; afterSecond.textContent = 'D'; afterRow.deleteCell(0); table.deleteRow(1); document.getElementById('out').textContent = String(table.rows.length) + ':' + String(table.rows.item(0).getAttribute('id')) + ':' + String(table.rows.item(1).getAttribute('id')) + ':' + String(table.rows.namedItem('first-row')) + ':' + String(beforeRow.cells.length) + ':' + String(afterRow.cells.length) + ':' + String(afterRow.cells.item(0).getAttribute('id'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2:before:after:null:1:1:after-second");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 18c table section reflection resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<table id='table'><tbody id='body'><tr><td>A</td></tr></tbody></table><div id='out'></div><script>const table = document.getElementById('table'); const before = String(table.caption) + ':' + String(table.tHead) + ':' + String(table.tFoot); const caption = table.createCaption(); caption.id = 'caption'; caption.textContent = 'C'; const head = table.createTHead(); head.id = 'head'; const foot = table.createTFoot(); foot.id = 'foot'; const middle = String(table.caption.getAttribute('id')) + ':' + String(table.tHead.getAttribute('id')) + ':' + String(table.tFoot.getAttribute('id')); table.deleteCaption(); table.deleteTHead(); table.deleteTFoot(); document.getElementById('out').textContent = before + '|' + middle + '|' + String(table.caption) + ':' + String(table.tHead) + ':' + String(table.tFoot);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "null:null:null|caption:head:foot|null:null:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 18c2 HTMLTableElement createCaption and deleteCaption resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<table id='table'><tbody id='body'><tr><td>A</td></tr></tbody></table><div id='out'></div><script>const table = document.getElementById('table'); const before = String(table.caption) + ':' + String(table.children.length); const caption = table.createCaption(); caption.id = 'caption'; caption.textContent = 'C'; const middle = String(table.caption.getAttribute('id')) + ':' + String(table.children.length) + ':' + String(table.children.item(0).getAttribute('id')); table.deleteCaption(); document.getElementById('out').textContent = before + '|' + middle + '|' + String(table.caption) + ':' + String(table.children.length);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "null:1|caption:2:caption|null:1");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 18c3 HTMLTableElement createTHead and createTFoot resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<table id='table'><tbody id='body'></tbody></table><div id='out'></div><script>const table = document.getElementById('table'); const before = String(table.tHead) + ':' + String(table.tFoot) + ':' + String(table.children.length); const head = table.createTHead(); head.id = 'head'; const foot = table.createTFoot(); foot.id = 'foot'; const middle = String(table.tHead.getAttribute('id')) + ':' + String(table.tFoot.getAttribute('id')) + ':' + String(table.children.length) + ':' + String(table.children.item(0).getAttribute('id')) + ':' + String(table.children.item(1).getAttribute('id')) + ':' + String(table.children.item(2).getAttribute('id')); table.deleteTHead(); table.deleteTFoot(); document.getElementById('out').textContent = before + '|' + middle + '|' + String(table.tHead) + ':' + String(table.tFoot) + ':' + String(table.children.length);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "null:null:1|head:foot:3:head:body:foot|null:null:1");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 18c4 HTMLTableElement createTBody resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<table id='table'><caption id='caption'></caption><colgroup id='group'></colgroup><tbody id='body-1'></tbody></table><div id='out'></div><script>const table = document.getElementById('table'); const before = String(table.children.length) + ':' + String(table.tBodies.length) + ':' + String(table.children.item(0).getAttribute('id')) + ':' + String(table.children.item(1).getAttribute('id')) + ':' + String(table.children.item(2).getAttribute('id')); const body2 = table.createTBody(); body2.id = 'body-2'; document.getElementById('out').textContent = before + '|' + String(table.children.length) + ':' + String(table.tBodies.length) + ':' + String(table.children.item(0).getAttribute('id')) + ':' + String(table.children.item(1).getAttribute('id')) + ':' + String(table.children.item(2).getAttribute('id')) + ':' + String(table.children.item(3).getAttribute('id'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "3:1:caption:group:body-1|4:2:caption:group:body-1:body-2");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 18c1 table body creation resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<table id='table'><caption id='caption'></caption><colgroup id='group'></colgroup><tbody id='body-1'></tbody></table><div id='out'></div><script>const table = document.getElementById('table'); const before = String(table.children.length) + ':' + String(table.children.item(0).getAttribute('id')) + ':' + String(table.children.item(1).getAttribute('id')) + ':' + String(table.children.item(2).getAttribute('id')) + ':' + String(table.tBodies.length); const head = table.createTHead(); head.id = 'head'; const body2 = table.createTBody(); body2.id = 'body-2'; const foot = table.createTFoot(); foot.id = 'foot'; document.getElementById('out').textContent = before + '|' + String(table.children.length) + ':' + String(table.children.item(0).getAttribute('id')) + ':' + String(table.children.item(1).getAttribute('id')) + ':' + String(table.children.item(2).getAttribute('id')) + ':' + String(table.children.item(3).getAttribute('id')) + ':' + String(table.children.item(4).getAttribute('id')) + ':' + String(table.children.item(5).getAttribute('id')) + ':' + String(table.tBodies.length) + ':' + String(table.tHead.getAttribute('id')) + ':' + String(table.tFoot.getAttribute('id'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "3:caption:group:body-1:1|6:caption:group:head:body-1:body-2:foot:2:head:foot");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 18d table section row mutation resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<table id='table'><thead id='head'></thead><tbody id='body'></tbody><tfoot id='foot'></tfoot></table><div id='out'></div><script>const head = document.getElementById('head'); const body = document.getElementById('body'); const foot = document.getElementById('foot'); const headRow = head.insertRow(); headRow.id = 'head-row'; headRow.insertCell().textContent = 'H'; const bodyRow = body.insertRow(); bodyRow.id = 'body-row'; bodyRow.insertCell().textContent = 'B'; const footRow = foot.insertRow(); footRow.id = 'foot-row'; footRow.insertCell().textContent = 'F'; const before = String(head.rows.length) + ':' + String(body.rows.length) + ':' + String(foot.rows.length) + ':' + String(document.getElementById('table').rows.length); head.deleteRow(0); body.deleteRow(); foot.deleteRow(0); document.getElementById('out').textContent = before + '|' + String(head.rows.length) + ':' + String(body.rows.length) + ':' + String(foot.rows.length) + ':' + String(document.getElementById('table').rows.length);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:1:1:3|0:0:0:0");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 18d1 HTMLTableSectionElement rows and mutation resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<table id='table'><thead id='head'></thead><tbody id='body'></tbody><tfoot id='foot'></tfoot></table><div id='out'></div><script>const table = document.getElementById('table'); const head = document.getElementById('head'); const body = document.getElementById('body'); const foot = document.getElementById('foot'); const before = String(head.rows.length) + ':' + String(body.rows.length) + ':' + String(foot.rows.length) + ':' + String(table.rows.length); const headRow = head.insertRow(); headRow.id = 'head-row'; const bodyRow = body.insertRow(); bodyRow.id = 'body-row'; const footRow = foot.insertRow(); footRow.id = 'foot-row'; const middle = String(head.rows.length) + ':' + String(body.rows.length) + ':' + String(foot.rows.length) + ':' + String(table.rows.length) + ':' + String(head.rows.item(0).getAttribute('id')) + ':' + String(body.rows.item(0).getAttribute('id')) + ':' + String(foot.rows.item(0).getAttribute('id')); head.deleteRow(0); body.deleteRow(0); foot.deleteRow(0); document.getElementById('out').textContent = before + '|' + middle + '|' + String(head.rows.length) + ':' + String(body.rows.length) + ':' + String(foot.rows.length) + ':' + String(table.rows.length);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "0:0:0:0|1:1:1:3:head-row:body-row:foot-row|0:0:0:0");
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

test "regression: phase 21 pseudo-class selectors resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' placeholder='Name'><button id='submit' disabled>Save</button><div id='empty'></div></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertExists(":root");
    try subject.assertExists("input:placeholder-shown");
    try subject.assertExists("button:disabled");
    try subject.assertExists("div:empty");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 22 scope pseudo-class selectors resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='section'><div id='child'>Child</div></section></main><div id='out'></div><script>const docScope = document.querySelector(':scope'); const root = document.getElementById('root'); const section = root.querySelector(':scope > section'); const missing = root.querySelector(':scope'); const matches = root.matches(':scope'); const closest = document.getElementById('child').closest(':scope'); document.getElementById('out').textContent = docScope.getAttribute('id') + ':' + section.getAttribute('id') + ':' + String(missing) + ':' + String(matches) + ':' + closest.getAttribute('id');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "root:section:null:true:child");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 22 blank pseudo-class selectors resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='blank-input' value='   '><textarea id='blank-textarea'>   </textarea><div id='blank-editable' contenteditable='true'>   </div><input id='filled' value='Ada'></main><div id='out'></div><script>const blankInput = document.getElementById('blank-input'); const blankTextarea = document.getElementById('blank-textarea'); const blankEditable = document.getElementById('blank-editable'); const filled = document.getElementById('filled'); document.getElementById('out').textContent = String(blankInput.matches(':blank')) + ':' + String(blankTextarea.matches(':blank')) + ':' + String(blankEditable.matches(':blank')) + ':' + String(filled.matches(':blank')) + ':' + String(document.querySelectorAll('#blank-input:blank').length) + ':' + String(document.querySelectorAll('#blank-textarea:blank').length) + ':' + String(document.querySelectorAll('#blank-editable:blank').length) + ':' + String(document.querySelectorAll('#filled:blank').length);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:true:true:false:1:1:1:0");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 23 :has pseudo-class selectors resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='first' class='child'>First</section><section id='child' class='child'><div id='grandchild' class='grandchild'>Grand</div></section></main><div id='out'></div><script>const docMatch = document.querySelector('main:has(#missing, #child)'); const directMatch = document.querySelector('main:has(> .child)'); const root = document.getElementById('root'); const section = document.getElementById('child'); const nested = document.querySelector('main:has(section .grandchild)'); const closest = section.closest('main:has(> .child)'); document.getElementById('out').textContent = docMatch.getAttribute('id') + ':' + directMatch.getAttribute('id') + ':' + String(root.matches('main:has(> .child)')) + ':' + String(section.matches(':has(.grandchild)')) + ':' + closest.getAttribute('id') + ':' + nested.getAttribute('id');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "root:root:true:true:root:root");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 24 focus-visible pseudo-class selectors resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='panel'><input id='field'></section><div id='out'></div><script>const field = document.getElementById('field'); field.focus(); document.getElementById('out').textContent = String(field.matches(':focus')) + ':' + String(field.matches(':focus-visible')) + ':' + String(document.querySelectorAll(':focus-visible').length) + ':' + String(document.querySelector('#panel:focus-visible')) + ':' + String(document.querySelector('#root:focus-visible'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:true:1:null:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 24 :lang and :dir pseudo-class selectors resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root' lang='en-US' dir='rtl'><section id='section'><span id='leaf' xml:lang='fr'>Bonjour</span></section></main><div id='out'></div><script>const langMatch = document.querySelector('main:lang(en)'); const inheritedLang = document.querySelector('section:lang(en-us)'); const dirMatch = document.querySelector('section:dir(rtl)'); const closest = document.getElementById('leaf').closest('main:dir(rtl)'); const matches = document.getElementById('section').matches(':dir(rtl)'); const leafLang = document.getElementById('leaf').matches(':lang(fr)'); document.getElementById('out').textContent = langMatch.getAttribute('id') + ':' + inheritedLang.getAttribute('id') + ':' + dirMatch.getAttribute('id') + ':' + closest.getAttribute('id') + ':' + String(matches) + ':' + String(leafLang);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "root:section:section:root:true:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 25 state pseudo-class selectors resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><progress id='loading'></progress><form id='signup'><input type='radio' name='mode' id='mode-a'><input type='radio' name='mode' id='mode-b'></form><form id='chosen'><input type='radio' name='picked' id='picked-a' checked><input type='radio' name='picked' id='picked-b'></form><form id='form'><input id='submit' type='submit'><input id='agree' type='checkbox' checked><input id='mode-c' type='radio' name='mode2'><input id='mode-d' type='radio' name='mode2' checked><select id='select'><option id='first' value='a'>A</option><option id='selected' value='b' selected>B</option></select></form><input id='name' value='Ada'><input id='readonly' value='Bee' readonly><textarea id='bio'>Hello</textarea><div id='editable' contenteditable='true'>Edit</div><select id='mode'><option id='option' value='a'>A</option></select><button id='button'>Button</button><input id='filled' type='text' required value='Ada'><input id='empty' type='text' required><input id='check' type='checkbox' required><input id='check-ok' type='checkbox' required checked><input id='low' type='number' min='2' max='6' value='1'><input id='high' type='number' min='2' max='6' value='7'><input id='in-range' type='number' min='2' max='6' value='4'><textarea id='bio-2' required></textarea><select id='mode-2' required><option value='a' selected>A</option><option value='b'>B</option></select></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertExists("#loading:indeterminate");
    try subject.assertExists("#picked-a:default");
    try subject.assertExists("#submit:default");
    try subject.assertExists("#agree:default");
    try subject.assertExists("#mode-d:default");
    try subject.assertExists("#selected:default");
    try subject.assertExists("#name:read-write");
    try subject.assertExists("#bio:read-write");
    try subject.assertExists("#editable:read-write");
    try subject.assertExists("#readonly:read-only");
    try subject.assertExists("#mode:read-only");
    try subject.assertExists("#option:read-only");
    try subject.assertExists("#button:read-only");
    try subject.assertExists("#filled:valid");
    try subject.assertExists("#check-ok:valid");
    try subject.assertExists("#in-range:valid");
    try subject.assertExists("#mode-2:valid");
    try subject.assertExists("#empty:invalid");
    try subject.assertExists("#check:invalid");
    try subject.assertExists("#low:out-of-range");
    try subject.assertExists("#high:out-of-range");
    try subject.assertExists("#in-range:in-range");
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

test "regression: phase 10a Element.children resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><span id='first'>First</span></main><div id='out'></div><script>const root = document.getElementById('root'); const children = root.children; const before = children.length; const first = children.item(0); const namedBefore = children.namedItem('second'); const second = document.createElement('span'); second.id = 'second'; second.textContent = 'Second'; root.appendChild(second); document.getElementById('out').textContent = String(before) + ':' + String(children.length) + ':' + first.getAttribute('id') + ':' + children.item(1).getAttribute('id') + ':' + String(namedBefore) + ':' + children.namedItem('second').getAttribute('id');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:2:first:second:null:second");
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

test "regression: phase 8 hasAttributes resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='button'></button><div id='out'></div><script>const button = document.createElement('button'); const before = button.hasAttributes(); button.setAttribute('data-flag', ''); document.getElementById('out').textContent = String(before) + ':' + String(button.hasAttributes());</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 getAttributeNames resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='button' data-kind='App'></button><div id='out'></div><script>const button = document.createElement('button'); button.setAttribute('id', 'button'); button.setAttribute('data-kind', 'App'); const names = button.getAttributeNames(); document.getElementById('out').textContent = String(names.length) + ':' + names.item(0) + ':' + names.item(1) + ':' + String(names.contains('id')) + ':' + String(names.contains('missing'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2:id:data-kind:true:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 namespace-aware attribute reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='button'></button><div id='out'></div><script>const button = document.getElementById('button'); button.setAttributeNS('urn:test', 'data-kind', 'App'); const before = button.getAttributeNS('urn:test', 'data-kind'); const present = button.hasAttributeNS('urn:test', 'data-kind'); button.removeAttributeNS('urn:test', 'data-kind'); document.getElementById('out').textContent = before + ':' + String(present) + ':' + String(button.hasAttributeNS('urn:test', 'data-kind'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "App:true:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.id resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='button'></button><div id='out'></div><script>const button = document.getElementById('button'); const before = button.id; button.id = 'updated'; document.getElementById('out').textContent = before + ':' + button.id + ':' + button.getAttribute('id');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "button:updated:updated");
    try subject.assertExists("#updated");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.hidden resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='box'></section><div id='out'></div><script>const box = document.getElementById('box'); const before = box.hidden; box.hidden = true; const during = box.hidden; box.hidden = false; document.getElementById('out').textContent = String(before) + ':' + String(during) + ':' + String(box.hidden) + ':' + String(box.hasAttribute('hidden'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:true:false:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.inert resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='box'></section><div id='out'></div><script>const box = document.getElementById('box'); const before = box.inert; box.inert = true; const during = box.inert; box.inert = false; document.getElementById('out').textContent = String(before) + ':' + String(during) + ':' + String(box.inert) + ':' + String(box.hasAttribute('inert'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:true:false:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.translate resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const outer = document.createElement('div'); const inner = document.createElement('span'); outer.appendChild(inner); const before = inner.translate; outer.translate = false; const inherited = inner.translate; inner.translate = true; const overridden = inner.translate; document.getElementById('out').textContent = String(before) + ':' + String(inherited) + ':' + String(overridden) + ':' + outer.getAttribute('translate') + ':' + inner.getAttribute('translate');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:false:true:no:yes");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.spellcheck resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const outer = document.createElement('div'); const inner = document.createElement('span'); outer.appendChild(inner); const before = inner.spellcheck; outer.spellcheck = false; const inherited = inner.spellcheck; inner.spellcheck = true; const overridden = inner.spellcheck; document.getElementById('out').textContent = String(before) + ':' + String(inherited) + ':' + String(overridden) + ':' + outer.getAttribute('spellcheck') + ':' + inner.getAttribute('spellcheck');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:false:true:false:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.draggable resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='box'></section><div id='out'></div><script>const box = document.getElementById('box'); const before = box.draggable; box.draggable = true; const during = box.draggable; box.draggable = false; document.getElementById('out').textContent = String(before) + ':' + String(during) + ':' + String(box.draggable) + ':' + String(box.hasAttribute('draggable'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:true:false:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.nonce resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const script = document.createElement('script'); const before = script.nonce; script.nonce = 'abc123'; const during = script.nonce; document.getElementById('out').textContent = before + ':' + during + ':' + script.nonce + ':' + script.getAttribute('nonce');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":abc123:abc123:abc123");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 HTMLScriptElement metadata resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><script id='script' src='https://example.test/app.js' type='module' async defer nomodule crossorigin='anonymous' integrity='sha384-abc' referrerpolicy='no-referrer' fetchpriority='high'></script><div id='out'></div><script>const script = document.querySelector('#script'); const before = '[' + script.src + ']:[' + String(script.async) + ']:[' + String(script.defer) + ']:[' + String(script.noModule) + ']:[' + script.type + ']:[' + script.crossOrigin + ']:[' + script.integrity + ']:[' + script.referrerPolicy + ']:[' + script.fetchPriority + ']:[' + script.charset + ']:[' + script.text + ']'; script.src = 'https://example.test/other.js'; script.async = false; script.defer = false; script.noModule = false; script.type = 'text/javascript'; script.crossOrigin = 'use-credentials'; script.integrity = 'sha384-def'; script.referrerPolicy = 'same-origin'; script.fetchPriority = 'low'; script.charset = 'utf-8'; script.text = 'inline text'; document.getElementById('out').textContent = before + '|' + '[' + script.src + ']:[' + String(script.async) + ']:[' + String(script.defer) + ']:[' + String(script.noModule) + ']:[' + script.type + ']:[' + script.crossOrigin + ']:[' + script.integrity + ']:[' + script.referrerPolicy + ']:[' + script.fetchPriority + ']:[' + script.charset + ']:[' + script.text + ']:[' + String(script.hasAttribute('src')) + ']:[' + String(script.hasAttribute('async')) + ']:[' + String(script.hasAttribute('defer')) + ']:[' + String(script.hasAttribute('nomodule')) + ']:[' + String(script.hasAttribute('type')) + ']:[' + String(script.hasAttribute('crossorigin')) + ']:[' + String(script.hasAttribute('integrity')) + ']:[' + String(script.hasAttribute('referrerpolicy')) + ']:[' + String(script.hasAttribute('fetchpriority')) + ']:[' + String(script.hasAttribute('charset')) + ']';</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[https://example.test/app.js]:[true]:[true]:[true]:[module]:[anonymous]:[sha384-abc]:[no-referrer]:[high]:[]:[]|[https://example.test/other.js]:[false]:[false]:[false]:[text/javascript]:[use-credentials]:[sha384-def]:[same-origin]:[low]:[utf-8]:[inline text]:[true]:[false]:[false]:[false]:[true]:[true]:[true]:[true]:[true]:[true]",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8a HTMLScriptElement charset resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><script id='script' charset='utf-8'></script><div id='out'></div><script>const script = document.querySelector('#script'); const before = script.charset + ':' + script.getAttribute('charset'); script.charset = 'windows-1252'; document.getElementById('out').textContent = before + '|' + script.charset + ':' + script.getAttribute('charset');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "utf-8:utf-8|windows-1252:windows-1252");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8b HTMLScriptElement src resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const script = document.createElement('script'); const before = script.src + ':' + script.getAttribute('src'); script.src = 'https://example.test/app.js'; document.getElementById('out').textContent = before + '|' + script.src + ':' + script.getAttribute('src');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":null|https://example.test/app.js:https://example.test/app.js");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8c HTMLScriptElement text resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const script = document.createElement('script'); const before = '[' + script.text + ']:[' + script.getAttribute('type') + ']'; script.text = 'inline text'; document.getElementById('out').textContent = before + '|' + '[' + script.text + ']:[' + script.getAttribute('type') + ']';</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[]:[null]|[inline text]:[null]");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8d HTMLScriptElement async resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const script = document.createElement('script'); const before = String(script.async) + ':' + String(script.hasAttribute('async')); script.async = true; document.getElementById('out').textContent = before + '|' + String(script.async) + ':' + String(script.hasAttribute('async'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:false|true:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8e HTMLScriptElement defer resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const script = document.createElement('script'); const before = String(script.defer) + ':' + String(script.hasAttribute('defer')); script.defer = true; document.getElementById('out').textContent = before + '|' + String(script.defer) + ':' + String(script.hasAttribute('defer'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:false|true:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8f HTMLScriptElement noModule resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const script = document.createElement('script'); const before = String(script.noModule) + ':' + String(script.hasAttribute('nomodule')); script.noModule = true; document.getElementById('out').textContent = before + '|' + String(script.noModule) + ':' + String(script.hasAttribute('nomodule'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:false|true:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8g HTMLScriptElement crossOrigin resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const script = document.createElement('script'); const before = String(script.crossOrigin) + ':' + String(script.hasAttribute('crossorigin')); script.crossOrigin = 'anonymous'; document.getElementById('out').textContent = before + '|' + script.crossOrigin + ':' + script.getAttribute('crossorigin');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":false|anonymous:anonymous");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8h HTMLScriptElement integrity resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const script = document.createElement('script'); const before = String(script.integrity) + ':' + String(script.hasAttribute('integrity')); script.integrity = 'sha384-abc'; document.getElementById('out').textContent = before + '|' + script.integrity + ':' + script.getAttribute('integrity');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":false|sha384-abc:sha384-abc");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8i HTMLScriptElement referrerPolicy resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const script = document.createElement('script'); const before = String(script.referrerPolicy) + ':' + String(script.hasAttribute('referrerpolicy')); script.referrerPolicy = 'same-origin'; document.getElementById('out').textContent = before + '|' + script.referrerPolicy + ':' + script.getAttribute('referrerpolicy');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":false|same-origin:same-origin");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8j HTMLScriptElement fetchPriority resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const script = document.createElement('script'); const before = String(script.fetchPriority) + ':' + String(script.hasAttribute('fetchpriority')); script.fetchPriority = 'high'; document.getElementById('out').textContent = before + '|' + script.fetchPriority + ':' + script.getAttribute('fetchpriority');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":false|high:high");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8k HTMLScriptElement type resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><script id='script' type='module'></script><div id='out'></div><script>const script = document.querySelector('#script'); const before = script.type + ':' + script.getAttribute('type'); script.type = 'text/javascript'; document.getElementById('out').textContent = before + '|' + script.type + ':' + script.getAttribute('type');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "module:module|text/javascript:text/javascript");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.autocapitalize resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const box = document.createElement('textarea'); const before = box.autocapitalize; box.autocapitalize = 'words'; const during = box.autocapitalize; document.getElementById('out').textContent = before + ':' + during + ':' + box.autocapitalize + ':' + box.getAttribute('autocapitalize');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":words:words:words");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 HTMLInputElement and HTMLTextAreaElement autocapitalize resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const input = document.createElement('input'); const area = document.createElement('textarea'); const before = input.autocapitalize + ':' + area.autocapitalize; input.autocapitalize = 'characters'; area.autocapitalize = 'words'; document.getElementById('out').textContent = before + '|' + input.autocapitalize + ':' + area.autocapitalize + ':' + input.getAttribute('autocapitalize') + ':' + area.getAttribute('autocapitalize');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":|characters:words:characters:words");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.autofocus resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const input = document.createElement('input'); const before = input.autofocus; input.autofocus = true; const during = input.autofocus; input.autofocus = false; document.getElementById('out').textContent = String(before) + ':' + String(during) + ':' + String(input.autofocus) + ':' + String(input.hasAttribute('autofocus'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:true:false:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 HTMLInputElement HTMLTextAreaElement HTMLButtonElement and HTMLSelectElement autofocus resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const input = document.createElement('input'); const area = document.createElement('textarea'); const button = document.createElement('button'); const select = document.createElement('select'); const before = String(input.autofocus) + ':' + String(area.autofocus) + ':' + String(button.autofocus) + ':' + String(select.autofocus); input.autofocus = true; area.autofocus = true; button.autofocus = true; select.autofocus = true; const during = String(input.autofocus) + ':' + String(area.autofocus) + ':' + String(button.autofocus) + ':' + String(select.autofocus) + ':' + String(input.hasAttribute('autofocus')) + ':' + String(area.hasAttribute('autofocus')) + ':' + String(button.hasAttribute('autofocus')) + ':' + String(select.hasAttribute('autofocus')); input.autofocus = false; area.autofocus = false; button.autofocus = false; select.autofocus = false; document.getElementById('out').textContent = before + '|' + during + '|' + String(input.autofocus) + ':' + String(area.autofocus) + ':' + String(button.autofocus) + ':' + String(select.autofocus) + ':' + String(input.hasAttribute('autofocus')) + ':' + String(area.hasAttribute('autofocus')) + ':' + String(button.hasAttribute('autofocus')) + ':' + String(select.hasAttribute('autofocus'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "false:false:false:false|true:true:true:true:true:true:true:true|false:false:false:false:false:false:false:false",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.placeholder resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const input = document.createElement('input'); const area = document.createElement('textarea'); const beforeInput = input.placeholder; const beforeArea = area.placeholder; root.appendChild(input); root.appendChild(area); input.placeholder = 'Name'; area.placeholder = 'Bio'; const duringInput = input.placeholder; const duringArea = area.placeholder; const placeholderShown = document.querySelectorAll(':placeholder-shown').length; document.getElementById('out').textContent = beforeInput + ':' + beforeArea + ':' + duringInput + ':' + duringArea + ':' + String(placeholderShown) + ':' + input.getAttribute('placeholder') + ':' + area.getAttribute('placeholder');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "::Name:Bio:2:Name:Bio");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 3c HTMLInputElement and HTMLTextAreaElement placeholder reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='input' placeholder='Name'><textarea id='area' placeholder='Bio'></textarea><div id='out'></div><script>const input = document.getElementById('input'); const area = document.getElementById('area'); const before = input.placeholder + ':' + area.placeholder + ':' + input.getAttribute('placeholder') + ':' + area.getAttribute('placeholder'); input.placeholder = 'Full name'; area.placeholder = 'Short bio'; document.getElementById('out').textContent = before + '|' + input.placeholder + ':' + area.placeholder + ':' + input.getAttribute('placeholder') + ':' + area.getAttribute('placeholder');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "Name:Bio:Name:Bio|Full name:Short bio:Full name:Short bio");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.minLength and maxLength resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const input = document.createElement('input'); const area = document.createElement('textarea'); input.id = 'input'; area.id = 'area'; input.value = 'abc'; area.textContent = 'abcd'; input.minLength = 4; area.maxLength = 3; root.appendChild(input); root.appendChild(area); document.getElementById('out').textContent = String(input.minLength) + ':' + String(input.maxLength) + ':' + String(area.minLength) + ':' + String(area.maxLength) + ':' + String(document.querySelectorAll(':invalid').length) + ':' + document.querySelector('#input:invalid').getAttribute('id') + ':' + document.querySelector('#area:invalid').getAttribute('id') + ':' + input.getAttribute('minlength') + ':' + area.getAttribute('maxlength');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "4:-1:-1:3:2:input:area:4:3");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 3d HTMLInputElement and HTMLTextAreaElement minLength and maxLength resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada'><textarea id='bio'>Hello</textarea><div id='out'></div><script>const name = document.getElementById('name'); const bio = document.getElementById('bio'); const before = String(name.minLength) + ':' + String(name.maxLength) + ':' + String(bio.minLength) + ':' + String(bio.maxLength); name.minLength = 2; name.maxLength = 5; bio.minLength = 3; bio.maxLength = 7; document.getElementById('out').textContent = before + '|' + String(name.minLength) + ':' + String(name.maxLength) + ':' + String(bio.minLength) + ':' + String(bio.maxLength) + ':' + String(name.getAttribute('minlength')) + ':' + String(name.getAttribute('maxlength')) + ':' + String(bio.getAttribute('minlength')) + ':' + String(bio.getAttribute('maxlength'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "-1:-1:-1:-1|2:5:3:7:2:5:3:7");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 3d1 HTMLInputElement HTMLTextAreaElement and HTMLSelectElement required resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='input' required><textarea id='area' required></textarea><select id='select' required><option value='a'>A</option></select><div id='out'></div><script>const input = document.getElementById('input'); const area = document.getElementById('area'); const select = document.getElementById('select'); const before = String(input.required) + ':' + String(area.required) + ':' + String(select.required) + ':' + String(input.hasAttribute('required')) + ':' + String(area.hasAttribute('required')) + ':' + String(select.hasAttribute('required')); input.required = false; area.required = false; select.required = false; document.getElementById('out').textContent = before + '|' + String(input.required) + ':' + String(area.required) + ':' + String(select.required) + ':' + String(input.getAttribute('required')) + ':' + String(area.getAttribute('required')) + ':' + String(select.getAttribute('required'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:true:true:true:true:true|false:false:false:null:null:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 3 checkValidity and reportValidity resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const form = document.createElement('form'); const input = document.createElement('input'); input.id = 'short'; input.minLength = 4; input.value = 'abc'; form.appendChild(input); document.getElementById('root').appendChild(form); document.getElementById('out').textContent = String(input.checkValidity()) + ':' + String(input.reportValidity()) + ':' + String(form.checkValidity()) + ':' + String(form.reportValidity()); input.value = 'abcd'; document.getElementById('out').textContent += '|' + String(input.checkValidity()) + ':' + String(input.reportValidity()) + ':' + String(form.checkValidity()) + ':' + String(form.reportValidity());</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:false:false:false|true:true:true:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 3 reportValidity dispatches invalid on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='form'><input id='short' minlength='4' value='abc'></form><div id='out'></div><script>const form = document.getElementById('form'); const input = document.getElementById('short'); form.addEventListener('invalid', () => { document.getElementById('out').textContent += 'form|'; }, true); input.addEventListener('invalid', () => { document.getElementById('out').textContent += 'input|'; }); const inputResult = String(input.reportValidity()); const inputEvents = document.getElementById('out').textContent; document.getElementById('out').textContent = ''; const formResult = String(form.reportValidity()); const formEvents = document.getElementById('out').textContent; document.getElementById('out').textContent = inputResult + ':' + inputEvents + '|' + formResult + ':' + formEvents;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:form|input||false:form|input|");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 3 setCustomValidity and validationMessage resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const input = document.createElement('input'); input.value = 'abc'; document.getElementById('root').appendChild(input); document.getElementById('out').textContent = String(input.checkValidity()) + ':' + input.validationMessage; input.setCustomValidity('bad'); document.getElementById('out').textContent += '|' + String(input.checkValidity()) + ':' + input.validationMessage + ':' + String(input.reportValidity()); input.setCustomValidity(''); document.getElementById('out').textContent += '|' + String(input.checkValidity()) + ':' + input.validationMessage;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:|false:bad:false|true:");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 3 Element.willValidate resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const field = document.createElement('input'); field.required = true; const hidden = document.createElement('input'); hidden.type = 'hidden'; hidden.required = true; const area = document.createElement('textarea'); area.required = true; area.readOnly = true; const select = document.createElement('select'); select.required = true; const option = document.createElement('option'); option.value = 'a'; option.selected = true; select.appendChild(option); root.appendChild(field); root.appendChild(hidden); root.appendChild(area); root.appendChild(select); document.getElementById('out').textContent = String(field.willValidate) + ':' + String(hidden.willValidate) + ':' + String(area.willValidate) + ':' + String(select.willValidate) + ':' + String(field.checkValidity()) + ':' + String(hidden.checkValidity()) + ':' + String(area.checkValidity()) + ':' + String(select.checkValidity());</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:false:false:true:false:true:true:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 3 Element.validity resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const input = document.createElement('input'); input.id = 'field'; input.type = 'number'; input.minLength = 4; input.min = '2'; input.max = '6'; input.step = '2'; input.value = '3'; root.appendChild(input); document.getElementById('out').textContent = String(input.validity.valid) + ':' + String(input.validity.tooShort) + ':' + String(input.checkValidity()) + ':' + String(input.validity) + ':' + input.min + ':' + input.max + ':' + input.step + ':' + String(input.validity.stepMismatch); input.value = '4'; input.setCustomValidity('bad'); document.getElementById('out').textContent += '|' + String(input.validity.valid) + ':' + String(input.validity.tooShort) + ':' + String(input.validity.customError) + ':' + String(input.checkValidity()) + ':' + input.validationMessage + ':' + String(input.validity) + ':' + String(input.validity.stepMismatch);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:false:false:[object ValidityState]:2:6:2:true|false:false:true:false:bad:[object ValidityState]:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 3 Element.validity typeMismatch resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const email = document.createElement('input'); email.id = 'email'; email.type = 'email'; email.value = 'not-an-email'; const url = document.createElement('input'); url.id = 'url'; url.type = 'url'; url.value = 'https://example.com/path'; root.appendChild(email); root.appendChild(url); document.getElementById('out').textContent = String(email.validity.typeMismatch) + ':' + String(email.checkValidity()) + ':' + String(url.validity.typeMismatch) + ':' + String(url.checkValidity()); email.value = 'ada@example.com, grace@example.com'; url.value = 'not a url'; document.getElementById('out').textContent += '|' + String(email.validity.typeMismatch) + ':' + String(email.checkValidity()) + ':' + String(url.validity.typeMismatch) + ':' + String(url.checkValidity());</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';
    try subject.assertValue("#out", "true:false:false:true|true:false:true:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 3 Element.validity patternMismatch resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const input = document.createElement('input'); input.pattern = '[0-9]{3}'; input.value = '12'; root.appendChild(input); document.getElementById('out').textContent = String(input.validity.patternMismatch) + ':' + String(input.checkValidity()); input.value = '123'; document.getElementById('out').textContent += '|' + String(input.validity.patternMismatch) + ':' + String(input.checkValidity());</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:false|false:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 3a HTMLInputElement pattern reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const input = document.createElement('input'); input.pattern = '[0-9]{3}'; input.value = '12'; root.appendChild(input); document.getElementById('out').textContent = input.pattern + ':' + String(input.validity.patternMismatch) + ':' + String(input.checkValidity()); input.value = '123'; input.pattern = '[0-9]{4}'; document.getElementById('out').textContent += '|' + input.pattern + ':' + String(input.validity.patternMismatch) + ':' + String(input.checkValidity()) + ':' + input.getAttribute('pattern');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[0-9]{3}:true:false|[0-9]{4}:true:false:[0-9]{4}");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 3b HTMLInputElement min max and step reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const input = document.createElement('input'); input.type = 'number'; input.value = '3'; root.appendChild(input); const before = input.min + ':' + input.max + ':' + input.step + ':' + String(input.hasAttribute('min')) + ':' + String(input.hasAttribute('max')) + ':' + String(input.hasAttribute('step')); input.min = '2'; input.max = '6'; input.step = '2'; const after = input.min + ':' + input.max + ':' + input.step + ':' + input.getAttribute('min') + ':' + input.getAttribute('max') + ':' + input.getAttribute('step'); input.min = ''; input.max = ''; input.step = ''; document.getElementById('out').textContent = before + '|' + after + '|' + input.min + ':' + input.max + ':' + input.step + ':' + String(input.hasAttribute('min')) + ':' + String(input.hasAttribute('max')) + ':' + String(input.hasAttribute('step'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":::false:false:false|2:6:2:2:6:2|:::true:true:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.name resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const form = document.createElement('form'); const input = document.createElement('input'); const area = document.createElement('textarea'); const beforeForm = form.name; const beforeInput = input.name; const beforeArea = area.name; form.name = 'signup'; input.name = 'first'; input.id = 'first-input'; area.name = 'bio'; root.appendChild(form); form.appendChild(input); form.appendChild(area); const duringForm = form.name; const duringInput = input.name; const duringArea = area.name; const formNamed = document.forms.namedItem('signup').name; const elementNamed = form.elements.namedItem('first').getAttribute('name'); const namedElements = document.getElementsByName('bio').length; document.getElementById('out').textContent = beforeForm + ':' + beforeInput + ':' + beforeArea + ':' + duringForm + ':' + duringInput + ':' + duringArea + ':' + formNamed + ':' + elementNamed + ':' + String(namedElements) + ':' + form.getAttribute('name') + ':' + input.getAttribute('name') + ':' + area.getAttribute('name');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":::signup:first:bio:signup:first:1:signup:first:bio");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 option.selected and select.value resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const select = document.createElement('select'); const first = document.createElement('option'); const second = document.createElement('option'); first.id = 'first'; first.value = 'a'; first.textContent = 'A'; second.id = 'second'; second.value = 'b'; second.textContent = 'B'; second.selected = true; select.appendChild(first); select.appendChild(second); root.appendChild(select); const before = select.value + ':' + first.value + ':' + second.value + ':' + String(first.selected) + ':' + String(second.selected) + ':' + String(select.selectedOptions.length); first.value = 'z'; select.value = 'z'; const after = select.value + ':' + first.value + ':' + second.value + ':' + String(first.selected) + ':' + String(second.selected) + ':' + String(select.selectedOptions.length); document.getElementById('out').textContent = before + '|' + after;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "b:a:b:false:true:1|z:z:b:true:false:1");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 defaultValue and defaultChecked resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada'><input id='agree' type='checkbox' checked><textarea id='bio'>Hello</textarea><div id='out'></div><script>document.getElementById('out').textContent = document.getElementById('name').defaultValue + ':' + String(document.getElementById('agree').defaultChecked) + ':' + document.getElementById('bio').defaultValue;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "Ada:true:Hello");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8a HTMLInputElement and HTMLTextAreaElement defaultValue resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada'><textarea id='bio'>Hello</textarea><div id='out'></div><script>const name = document.getElementById('name'); const bio = document.getElementById('bio'); const before = name.defaultValue + ':' + bio.defaultValue; name.defaultValue = 'Bea'; bio.defaultValue = 'World'; document.getElementById('out').textContent = before + '|' + name.defaultValue + ':' + name.value + ':' + name.getAttribute('value') + ':' + bio.defaultValue + ':' + bio.value + ':' + bio.textContent;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "Ada:Hello|Bea:Bea:Bea:World:World:World");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 3e HTMLInputElement checked reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='check' type='checkbox'><div id='out'></div><script>const check = document.getElementById('check'); const before = String(check.checked) + ':' + String(check.defaultChecked) + ':' + String(check.hasAttribute('checked')); check.checked = true; const during = String(check.checked) + ':' + String(check.defaultChecked) + ':' + String(check.hasAttribute('checked')); check.checked = false; document.getElementById('out').textContent = before + '|' + during + '|' + String(check.checked) + ':' + String(check.defaultChecked) + ':' + String(check.hasAttribute('checked'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:false:false|true:true:true|false:false:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 3e1 HTMLInputElement defaultChecked reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='agree' type='checkbox' checked><div id='out'></div><script>const agree = document.getElementById('agree'); const before = String(agree.defaultChecked) + ':' + String(agree.checked) + ':' + String(agree.hasAttribute('checked')); agree.defaultChecked = false; const during = String(agree.defaultChecked) + ':' + String(agree.checked) + ':' + String(agree.hasAttribute('checked')); agree.defaultChecked = true; document.getElementById('out').textContent = before + '|' + during + '|' + String(agree.defaultChecked) + ':' + String(agree.checked) + ':' + String(agree.hasAttribute('checked'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:true:true|false:false:false|true:true:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 3f HTMLButtonElement and HTMLInputElement formNoValidate reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='button' type='submit'></button><input id='input' type='submit'><div id='out'></div><script>const button = document.getElementById('button'); const input = document.getElementById('input'); const before = String(button.formNoValidate) + ':' + String(input.formNoValidate) + ':' + String(button.hasAttribute('formnovalidate')) + ':' + String(input.hasAttribute('formnovalidate')); button.formNoValidate = true; input.formNoValidate = true; const during = String(button.formNoValidate) + ':' + String(input.formNoValidate) + ':' + String(button.hasAttribute('formnovalidate')) + ':' + String(input.hasAttribute('formnovalidate')); button.formNoValidate = false; input.formNoValidate = false; document.getElementById('out').textContent = before + '|' + during + '|' + String(button.formNoValidate) + ':' + String(input.formNoValidate) + ':' + String(button.hasAttribute('formnovalidate')) + ':' + String(input.hasAttribute('formnovalidate'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:false:false:false|true:true:true:true|false:false:false:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 HTMLOutputElement reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='owner'></form><label id='label' for='result'>Answer</label><output id='result' form='owner' for='alpha alpha beta'>Hello</output><div id='out'></div><script>const result = document.getElementById('result'); const before = result.type + ':' + result.defaultValue + ':' + result.value + ':' + result.form.id + ':' + String(result.labels.length) + ':' + result.htmlFor.value + ':' + String(result.htmlFor.length) + ':' + String(result.htmlFor.contains('alpha')) + ':' + String(result.willValidate) + ':' + result.validationMessage + ':' + String(result.checkValidity()) + ':' + String(result.reportValidity()) + ':' + String(result.validity.customError); result.htmlFor.value = 'gamma gamma delta'; result.defaultValue = 'Reset'; result.value = 'Computed'; result.setCustomValidity('bad'); document.getElementById('out').textContent = before + '|' + result.defaultValue + ':' + result.value + ':' + result.htmlFor.value + ':' + String(result.validity.customError) + ':' + result.validationMessage + ':' + String(result.checkValidity()) + ':' + String(result.reportValidity()) + ':' + String(result.labels.length);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "output:Hello:Hello:owner:1:alpha beta:2:true:true::true:true:false|Reset:Computed:gamma delta:true:bad:false:false:1");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8b label, legend, and fieldset form owner reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='owner'></form><fieldset id='group' form='owner'><legend id='legend'>Title</legend></fieldset><label id='explicit' for='control'>Explicit</label><input id='control' form='owner'><label id='implicit'><input id='inner' form='owner'>Implicit</label><label id='detached'>Detached</label><div id='out'></div><script>const fieldset = document.getElementById('group'); const legend = document.getElementById('legend'); const explicit = document.getElementById('explicit'); const implicit = document.getElementById('implicit'); const detached = document.getElementById('detached'); document.getElementById('out').textContent = fieldset.form.id + ':' + legend.form.id + ':' + explicit.form.id + ':' + implicit.form.id + ':' + String(detached.form);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "owner:owner:owner:owner:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 form.noValidate and formNoValidate resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const form = document.createElement('form'); const button = document.createElement('button'); const input = document.createElement('input'); button.type = 'submit'; input.type = 'submit'; form.appendChild(button); form.appendChild(input); document.getElementById('root').appendChild(form); const before = String(form.noValidate) + ':' + String(button.formNoValidate) + ':' + String(input.formNoValidate); form.noValidate = true; button.formNoValidate = true; input.formNoValidate = true; const during = String(form.noValidate) + ':' + String(button.formNoValidate) + ':' + String(input.formNoValidate) + ':' + String(form.getAttribute('novalidate')) + ':' + String(button.getAttribute('formnovalidate')) + ':' + String(input.getAttribute('formnovalidate')); form.noValidate = false; button.formNoValidate = false; input.formNoValidate = false; document.getElementById('out').textContent = before + '|' + during + '|' + String(form.noValidate) + ':' + String(button.formNoValidate) + ':' + String(input.formNoValidate) + ':' + String(form.hasAttribute('novalidate')) + ':' + String(button.hasAttribute('formnovalidate')) + ':' + String(input.hasAttribute('formnovalidate'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:false:false|true:true:true:::|false:false:false:false:false:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 form submission reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='form'><button id='button' type='submit'></button><input id='input' type='submit'></form><div id='before'></div><div id='after'></div><script>const form = document.getElementById('form'); const button = document.getElementById('button'); const input = document.getElementById('input'); document.getElementById('before').textContent = 'form=' + form.action + ':' + form.method + ':' + form.enctype + ':' + form.encoding + ':' + form.target + ':' + form.acceptCharset + '|button=' + button.formAction + ':' + button.formMethod + ':' + button.formEnctype + ':' + button.formTarget + '|input=' + input.formAction + ':' + input.formMethod + ':' + input.formEnctype + ':' + input.formTarget; form.action = '/submit'; form.method = 'POST'; form.enctype = 'multipart/form-data'; form.target = '_blank'; form.acceptCharset = 'utf-8'; button.formAction = '/button-submit'; button.formMethod = 'Dialog'; button.formEnctype = 'text/plain'; button.formTarget = '_self'; input.formAction = '/input-submit'; input.formMethod = 'POST'; input.formEnctype = 'multipart/form-data'; input.formTarget = '_parent'; document.getElementById('after').textContent = 'form=' + form.action + ':' + form.method + ':' + form.enctype + ':' + form.encoding + ':' + form.target + ':' + form.acceptCharset + ':' + form.getAttribute('action') + ':' + form.getAttribute('method') + ':' + form.getAttribute('enctype') + ':' + form.getAttribute('target') + ':' + form.getAttribute('accept-charset') + '|button=' + button.formAction + ':' + button.formMethod + ':' + button.formEnctype + ':' + button.formTarget + ':' + button.getAttribute('formaction') + ':' + button.getAttribute('formmethod') + ':' + button.getAttribute('formenctype') + ':' + button.getAttribute('formtarget') + '|input=' + input.formAction + ':' + input.formMethod + ':' + input.formEnctype + ':' + input.formTarget + ':' + input.getAttribute('formaction') + ':' + input.getAttribute('formmethod') + ':' + input.getAttribute('formenctype') + ':' + input.getAttribute('formtarget');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#before", "form=https://app.local/:get:application/x-www-form-urlencoded:application/x-www-form-urlencoded::|button=https://app.local/:get:application/x-www-form-urlencoded:|input=https://app.local/:get:application/x-www-form-urlencoded:");
    try subject.assertValue("#after", "form=https://app.local/submit:post:multipart/form-data:multipart/form-data:_blank:utf-8:/submit:post:multipart/form-data:_blank:utf-8|button=https://app.local/button-submit:dialog:text/plain:_self:/button-submit:dialog:text/plain:_self|input=https://app.local/input-submit:post:multipart/form-data:_parent:/input-submit:post:multipart/form-data:_parent");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 form action method encoding and target resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='form'></form><div id='out'></div><script>const form = document.getElementById('form'); const before = form.action + ':' + form.method + ':' + form.enctype + ':' + form.encoding + ':' + form.target; form.action = '/submit'; form.method = 'POST'; form.encoding = 'multipart/form-data'; form.target = '_blank'; document.getElementById('out').textContent = before + '|' + form.action + ':' + form.method + ':' + form.enctype + ':' + form.encoding + ':' + form.target + ':' + form.getAttribute('action') + ':' + form.getAttribute('method') + ':' + form.getAttribute('enctype') + ':' + form.getAttribute('target');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "https://app.local/:get:application/x-www-form-urlencoded:application/x-www-form-urlencoded:|https://app.local/submit:post:multipart/form-data:multipart/form-data:_blank:/submit:post:multipart/form-data:_blank");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8a5 HTMLFormElement acceptCharset resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='form' accept-charset='utf-8'></form><div id='out'></div><script>const form = document.getElementById('form'); const before = form.acceptCharset; form.acceptCharset = 'iso-8859-1'; document.getElementById('out').textContent = before + ':' + form.acceptCharset + ':' + form.getAttribute('accept-charset');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "utf-8:iso-8859-1:iso-8859-1");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 form checkValidity and reportValidity resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='form'><input id='short' minlength='4' value='abc'></form><div id='out'></div><script>const form = document.getElementById('form'); const short = document.getElementById('short'); const before = String(form.checkValidity()) + ':' + String(form.reportValidity()) + ':' + String(short.checkValidity()) + ':' + String(short.reportValidity()); short.value = 'abcd'; document.getElementById('out').textContent = before + '|' + String(form.checkValidity()) + ':' + String(form.reportValidity()) + ':' + String(short.checkValidity()) + ':' + String(short.reportValidity());</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:false:false:false|true:true:true:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8c HTMLButtonElement and HTMLInputElement submitter overrides resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='form'></form><button id='button' type='submit' form='form'></button><input id='input' type='submit' form='form'><div id='out'></div><script>const button = document.getElementById('button'); const input = document.getElementById('input'); const before = button.formAction + ':' + input.formAction + ':' + button.formMethod + ':' + input.formMethod + ':' + button.formEnctype + ':' + input.formEnctype + ':' + button.formTarget + ':' + input.formTarget; button.formAction = '/button'; input.formAction = '/input'; button.formMethod = 'Dialog'; input.formMethod = 'POST'; button.formEnctype = 'text/plain'; input.formEnctype = 'multipart/form-data'; button.formTarget = '_self'; input.formTarget = '_parent'; document.getElementById('out').textContent = before + '|' + button.formAction + ':' + input.formAction + ':' + button.getAttribute('formaction') + ':' + input.getAttribute('formaction') + ':' + button.formMethod + ':' + input.formMethod + ':' + button.formEnctype + ':' + input.formEnctype + ':' + button.formTarget + ':' + input.formTarget;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "https://app.local/:https://app.local/:get:get:application/x-www-form-urlencoded:application/x-www-form-urlencoded::|https://app.local/button:https://app.local/input:/button:/input:dialog:post:text/plain:multipart/form-data:_self:_parent");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 form submit actions resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='form'><input id='name'><button id='button' type='submit'></button></form><div id='out'></div><script>const form = document.getElementById('form'); const name = document.getElementById('name'); const button = document.getElementById('button'); form.addEventListener('submit', (event) => { event.preventDefault(); document.getElementById('out').textContent += document.getElementById('name').value + '|'; }); name.value = 'Ada'; form.submit(); form.requestSubmit(button);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "Ada|Ada|");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 form.reset resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='form'><input id='name' value='Ada'></form><div id='out'></div><script>const form = document.getElementById('form'); form.addEventListener('reset', (event) => { event.preventDefault(); document.getElementById('out').textContent = 'reset:' + String(event.bubbles) + ':' + String(event.cancelable); }); form.reset();</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "reset:true:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 dialog controls resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><dialog id='dlg'></dialog><div id='out'></div><script>const dialog = document.getElementById('dlg'); dialog.closedBy = 'none'; dialog.addEventListener('cancel', () => { document.getElementById('out').textContent = 'cancel:' + String(document.getElementById('dlg').open) + ':' + document.getElementById('dlg').returnValue; }); dialog.addEventListener('close', () => { document.getElementById('out').textContent += '|close:' + String(document.getElementById('dlg').open) + ':' + document.getElementById('dlg').returnValue; }); dialog.show(); dialog.requestClose('done'); document.getElementById('out').textContent += '|after:' + String(dialog.open) + ':' + dialog.returnValue + ':' + dialog.closedBy;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "cancel:true:|close:false:done|after:false:done:none");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8a dialog closedBy reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><dialog id='dlg'></dialog><div id='out'></div><script>const dialog = document.getElementById('dlg'); dialog.closedBy = 'none'; dialog.showModal(); dialog.requestClose('done'); document.getElementById('out').textContent = dialog.closedBy + ':' + dialog.returnValue + ':' + String(dialog.open);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "none:done:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8c dialog close resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><dialog id='dlg'></dialog><div id='out'></div><script>const dialog = document.getElementById('dlg'); dialog.closedBy = 'none'; dialog.addEventListener('close', () => { document.getElementById('out').textContent = document.getElementById('dlg').open + ':' + document.getElementById('dlg').returnValue + ':' + document.getElementById('dlg').closedBy; }); dialog.showModal(); dialog.close('done');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:done:none");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46c3 command buttons resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><dialog id='dlg'></dialog><button id='open' type='button'>Open</button><button id='close' type='button'>Close</button><div id='out'></div><script>const dialog = document.getElementById('dlg'); const open = document.getElementById('open'); const close = document.getElementById('close'); open.command = 'show-modal'; open.commandForElement = dialog; close.command = 'request-close'; close.commandForElement = dialog; dialog.addEventListener('command', (event) => { document.getElementById('out').textContent += event.command + ':' + event.source.id + ':' + String(document.getElementById('dlg').open) + ';'; });</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.click("#open");
    try subject.assertValue("#out", "show-modal:open:false;");
    try subject.click("#close");
    try subject.assertValue("#out", "show-modal:open:false;request-close:close:true;");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 details controls resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><details id='details' name='accordion'><summary id='summary'>Title</summary><div>Body</div></details><div id='out'></div><script>const details = document.getElementById('details'); const out = document.getElementById('out'); out.textContent = String(details.open) + ':' + details.name; details.addEventListener('toggle', () => { const current = document.getElementById('details'); const output = document.getElementById('out'); output.textContent += '|' + String(current.open) + ':' + current.name; });</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.click("#summary");
    try subject.assertValue("#out", "false:accordion|true:accordion");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 reset button click resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='form'><input id='name' value='Ada'><button id='reset' type='reset'>Reset</button></form><div id='out'></div><script>const form = document.getElementById('form'); form.addEventListener('reset', () => { document.getElementById('out').textContent = document.getElementById('name').value; });</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.typeText("#name", "Alice");
    try subject.click("#reset");
    try subject.assertValue("#out", "Alice");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.multiple resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const mail = document.createElement('input'); mail.type = 'email'; mail.multiple = true; const select = document.createElement('select'); select.multiple = true; const first = document.createElement('option'); first.value = 'a'; first.selected = true; const second = document.createElement('option'); second.value = 'b'; select.appendChild(first); select.appendChild(second); document.getElementById('root').appendChild(mail); document.getElementById('root').appendChild(select); const before = String(mail.multiple) + ':' + String(select.multiple) + ':' + String(mail.hasAttribute('multiple')) + ':' + String(select.hasAttribute('multiple')) + ':' + String(select.selectedOptions.length); mail.multiple = false; select.multiple = false; document.getElementById('out').textContent = before + ':' + String(mail.multiple) + ':' + String(select.multiple) + ':' + String(mail.hasAttribute('multiple')) + ':' + String(select.hasAttribute('multiple')) + ':' + String(select.selectedOptions.length);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:true:true:true:1:false:false:false:false:1");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 select.selectedIndex resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const select = document.createElement('select'); const first = document.createElement('option'); const second = document.createElement('option'); first.id = 'first'; first.value = 'a'; first.textContent = 'A'; second.id = 'second'; second.value = 'b'; second.textContent = 'B'; second.selected = true; select.appendChild(first); select.appendChild(second); root.appendChild(select); const before = String(select.selectedIndex) + ':' + select.value + ':' + String(first.selected) + ':' + String(second.selected) + ':' + String(select.selectedOptions.length); select.selectedIndex = 0; const after = String(select.selectedIndex) + ':' + select.value + ':' + String(first.selected) + ':' + String(second.selected) + ':' + String(select.selectedOptions.length); document.getElementById('out').textContent = before + '|' + after;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:b:false:true:1|0:a:true:false:1");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8b select.size resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const select = document.createElement('select'); const first = document.createElement('option'); const second = document.createElement('option'); first.value = 'a'; second.value = 'b'; select.appendChild(first); select.appendChild(second); root.appendChild(select); const before = String(select.size); select.size = 6; document.getElementById('out').textContent = before + ':' + String(select.size) + ':' + select.getAttribute('size');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "0:6:6");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8c select.type resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const select = document.createElement('select'); const first = document.createElement('option'); const second = document.createElement('option'); first.value = 'a'; second.value = 'b'; select.appendChild(first); select.appendChild(second); root.appendChild(select); const before = select.type; select.multiple = true; document.getElementById('out').textContent = before + ':' + select.type + ':' + String(select.multiple);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "select-one:select-multiple:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8c1 select.type toggle resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const select = document.createElement('select'); const first = document.createElement('option'); const second = document.createElement('option'); first.value = 'a'; second.value = 'b'; select.appendChild(first); select.appendChild(second); document.getElementById('root').appendChild(select); const before = select.type; select.multiple = true; const during = select.type; select.multiple = false; document.getElementById('out').textContent = before + ':' + during + ':' + select.type + ':' + String(select.multiple);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "select-one:select-multiple:select-one:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8c2 select.multiple exact reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const select = document.createElement('select'); const first = document.createElement('option'); const second = document.createElement('option'); first.value = 'a'; second.value = 'b'; select.appendChild(first); select.appendChild(second); document.getElementById('root').appendChild(select); const before = String(select.multiple) + ':' + select.type + ':' + String(select.selectedOptions.length) + ':' + String(select.hasAttribute('multiple')); select.multiple = true; const during = String(select.multiple) + ':' + select.type + ':' + String(select.selectedOptions.length) + ':' + String(select.hasAttribute('multiple')); select.multiple = false; document.getElementById('out').textContent = before + '|' + during + '|' + String(select.multiple) + ':' + select.type + ':' + String(select.selectedOptions.length) + ':' + String(select.hasAttribute('multiple'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:select-one:0:false|true:select-multiple:0:true|false:select-one:0:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.disabled and required resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const root = document.getElementById('root'); const button = document.createElement('button'); button.id = 'button'; const input = document.createElement('input'); input.id = 'field'; const area = document.createElement('textarea'); area.id = 'area'; const select = document.createElement('select'); select.id = 'select'; const option = document.createElement('option'); option.value = 'a'; option.selected = true; select.appendChild(option); root.appendChild(button); root.appendChild(input); root.appendChild(area); root.appendChild(select); const beforeDisabled = button.disabled; const beforeRequired = input.required; const beforeAreaDisabled = area.disabled; const beforeSelectDisabled = select.disabled; button.disabled = true; input.required = true; area.disabled = true; select.disabled = true; const duringDisabled = button.disabled; const duringRequired = input.required; const duringAreaDisabled = area.disabled; const duringSelectDisabled = select.disabled; const disabledMatches = document.querySelectorAll(':disabled').length; const requiredMatches = document.querySelectorAll(':required').length; button.disabled = false; input.required = false; area.disabled = false; select.disabled = false; document.getElementById('out').textContent = String(beforeDisabled) + ':' + String(beforeRequired) + ':' + String(beforeAreaDisabled) + ':' + String(beforeSelectDisabled) + ':' + String(duringDisabled) + ':' + String(duringRequired) + ':' + String(duringAreaDisabled) + ':' + String(duringSelectDisabled) + ':' + String(disabledMatches) + ':' + String(requiredMatches) + ':' + String(button.disabled) + ':' + String(input.required) + ':' + String(area.disabled) + ':' + String(select.disabled) + ':' + String(document.querySelectorAll(':disabled').length) + ':' + String(document.querySelectorAll(':required').length);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:false:false:false:true:true:true:true:3:1:false:false:false:false:0:0");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 3g HTMLInputElement HTMLButtonElement HTMLSelectElement and HTMLTextAreaElement disabled reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='input'><button id='button'></button><select id='select'><option value='a'>A</option></select><textarea id='area'></textarea><div id='out'></div><script>const input = document.getElementById('input'); const button = document.getElementById('button'); const select = document.getElementById('select'); const area = document.getElementById('area'); const before = String(input.disabled) + ':' + String(button.disabled) + ':' + String(select.disabled) + ':' + String(area.disabled) + ':' + String(input.hasAttribute('disabled')) + ':' + String(button.hasAttribute('disabled')) + ':' + String(select.hasAttribute('disabled')) + ':' + String(area.hasAttribute('disabled')); input.disabled = true; button.disabled = true; select.disabled = true; area.disabled = true; const during = String(input.disabled) + ':' + String(button.disabled) + ':' + String(select.disabled) + ':' + String(area.disabled) + ':' + String(input.hasAttribute('disabled')) + ':' + String(button.hasAttribute('disabled')) + ':' + String(select.hasAttribute('disabled')) + ':' + String(area.hasAttribute('disabled')); input.disabled = false; button.disabled = false; select.disabled = false; area.disabled = false; document.getElementById('out').textContent = before + '|' + during + '|' + String(input.disabled) + ':' + String(button.disabled) + ':' + String(select.disabled) + ':' + String(area.disabled) + ':' + String(input.hasAttribute('disabled')) + ':' + String(button.hasAttribute('disabled')) + ':' + String(select.hasAttribute('disabled')) + ':' + String(area.hasAttribute('disabled'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:false:false:false:false:false:false:false|true:true:true:true:true:true:true:true|false:false:false:false:false:false:false:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 3h HTMLInputElement and HTMLTextAreaElement selectionStart selectionEnd and selectionDirection resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada'><textarea id='bio'>Hello</textarea><input id='check' type='checkbox'><div id='out'></div><script>const name = document.getElementById('name'); const bio = document.getElementById('bio'); const check = document.getElementById('check'); const before = String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + ':' + String(bio.selectionStart) + ':' + String(bio.selectionEnd) + ':' + bio.selectionDirection + ':' + String(check.selectionStart) + ':' + String(check.selectionEnd) + ':' + String(check.selectionDirection); name.setSelectionRange(1, 3, 'backward'); bio.select(); document.getElementById('out').textContent = before + '|' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + '|' + String(bio.selectionStart) + ':' + String(bio.selectionEnd) + ':' + bio.selectionDirection + '|' + String(check.selectionStart) + ':' + String(check.selectionEnd) + ':' + String(check.selectionDirection);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "3:3:none:5:5:none:null:null:null|1:3:backward|0:5:none|null:null:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 3h1 HTMLInputElement and HTMLTextAreaElement select resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada'><textarea id='bio'>Hello</textarea><div id='out'></div><script>const name = document.getElementById('name'); const bio = document.getElementById('bio'); const before = String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + ':' + String(bio.selectionStart) + ':' + String(bio.selectionEnd) + ':' + bio.selectionDirection; name.select(); bio.select(); document.getElementById('out').textContent = before + '|' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + '|' + String(bio.selectionStart) + ':' + String(bio.selectionEnd) + ':' + bio.selectionDirection;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "3:3:none:5:5:none|0:3:none|0:5:none");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.autocomplete resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const input = document.createElement('input'); const before = input.autocomplete; input.autocomplete = 'email'; const during = input.autocomplete; document.getElementById('out').textContent = before + ':' + during + ':' + input.autocomplete + ':' + input.getAttribute('autocomplete');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":email:email:email");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8a HTMLInputElement.autocomplete resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const input = document.createElement('input'); const before = input.autocomplete; input.autocomplete = 'email'; document.getElementById('out').textContent = before + ':' + input.autocomplete + ':' + input.getAttribute('autocomplete');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":email:email");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8a2 HTMLInputElement.form resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='owner'></form><div id='out'></div><script>const input = document.createElement('input'); const before = String(input.form); input.setAttribute('form', 'owner'); const during = input.form.id; input.removeAttribute('form'); document.getElementById('out').textContent = before + '|' + during + '|' + String(input.form);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "null|owner|null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8a3 HTMLButtonElement.form resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='owner'></form><div id='out'></div><script>const button = document.createElement('button'); const before = String(button.form); button.setAttribute('form', 'owner'); const during = button.form.id; button.removeAttribute('form'); document.getElementById('out').textContent = before + '|' + during + '|' + String(button.form);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "null|owner|null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8a4 HTMLTextAreaElement.form resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='owner'></form><div id='out'></div><script>const area = document.createElement('textarea'); const before = String(area.form); area.setAttribute('form', 'owner'); const during = area.form.id; area.removeAttribute('form'); document.getElementById('out').textContent = before + '|' + during + '|' + String(area.form);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "null|owner|null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8a1 HTMLTextAreaElement.autocomplete resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const area = document.createElement('textarea'); const before = area.autocomplete; area.autocomplete = 'off'; document.getElementById('out').textContent = before + ':' + area.autocomplete + ':' + area.getAttribute('autocomplete');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":off:off");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8f HTMLInputElement and HTMLTextAreaElement value reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada'><textarea id='bio'>Hello</textarea><div id='out'></div><script>const name = document.getElementById('name'); const bio = document.getElementById('bio'); const before = name.value + ':' + bio.value + ':' + name.defaultValue + ':' + bio.defaultValue; name.value = 'Bea'; bio.value = 'World'; document.getElementById('out').textContent = before + '|' + name.value + ':' + name.defaultValue + ':' + name.getAttribute('value') + ':' + bio.value + ':' + bio.defaultValue + ':' + bio.textContent;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "Ada:Hello:Ada:Hello|Bea:Bea:Bea:World:World:World");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 HTMLFormElement.autocomplete resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='form'></form><div id='out'></div><script>const form = document.getElementById('form'); const before = form.autocomplete; form.autocomplete = 'off'; const during = form.autocomplete; form.autocomplete = 'on'; document.getElementById('out').textContent = before + ':' + during + ':' + form.autocomplete + ':' + form.getAttribute('autocomplete');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "on:off:on:on");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 HTMLSelectElement.autocomplete resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const select = document.createElement('select'); const before = select.autocomplete; select.autocomplete = 'section-checkout shipping'; document.getElementById('root').appendChild(select); document.getElementById('out').textContent = before + ':' + select.autocomplete + ':' + select.getAttribute('autocomplete');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":section-checkout shipping:section-checkout shipping");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.inputMode resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const box = document.createElement('input'); const before = box.inputMode; box.inputMode = 'numeric'; const during = box.inputMode; document.getElementById('out').textContent = before + ':' + during + ':' + box.inputMode + ':' + box.getAttribute('inputmode');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":numeric:numeric:numeric");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 HTMLInputElement and HTMLTextAreaElement inputMode resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const input = document.createElement('input'); const area = document.createElement('textarea'); const before = input.inputMode + ':' + area.inputMode; input.inputMode = 'numeric'; area.inputMode = 'search'; document.getElementById('out').textContent = before + '|' + input.inputMode + ':' + area.inputMode + ':' + input.getAttribute('inputmode') + ':' + area.getAttribute('inputmode');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":|numeric:search:numeric:search");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.readOnly resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const input = document.createElement('input'); const area = document.createElement('textarea'); const before = String(input.readOnly) + ':' + String(area.readOnly); input.readOnly = true; area.readOnly = true; document.getElementById('out').textContent = before + ':' + String(input.readOnly) + ':' + String(area.readOnly) + ':' + String(input.hasAttribute('readonly')) + ':' + String(area.hasAttribute('readonly'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:false:true:true:true:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8a7 HTMLInputElement and HTMLTextAreaElement readOnly resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='input'><textarea id='area'></textarea><div id='out'></div><script>const input = document.getElementById('input'); const area = document.getElementById('area'); const before = String(input.readOnly) + ':' + String(area.readOnly) + ':' + String(input.hasAttribute('readonly')) + ':' + String(area.hasAttribute('readonly')); input.readOnly = true; area.readOnly = true; document.getElementById('out').textContent = before + '|' + String(input.readOnly) + ':' + String(area.readOnly) + ':' + String(input.getAttribute('readonly')) + ':' + String(area.getAttribute('readonly'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:false:false:false|true:true::");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.accessKey resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const button = document.createElement('button'); const before = button.accessKey; button.accessKey = 'k'; const during = button.accessKey; document.getElementById('out').textContent = before + ':' + during + ':' + button.accessKey + ':' + button.getAttribute('accesskey');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":k:k:k");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element aria and role reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const box = document.createElement('div'); const beforeRole = box.role; const beforeHidden = box.ariaHidden; box.role = 'button'; box.ariaLabel = 'Menu'; box.ariaDescription = 'Opens menu'; box.ariaRoleDescription = 'toggle button'; box.ariaHidden = 'true'; document.getElementById('out').textContent = beforeRole + ':' + beforeHidden + ':' + box.role + ':' + box.ariaLabel + ':' + box.getAttribute('aria-label') + ':' + box.ariaDescription + ':' + box.getAttribute('aria-description') + ':' + box.ariaRoleDescription + ':' + box.getAttribute('aria-roledescription') + ':' + box.ariaHidden + ':' + box.getAttribute('aria-hidden');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "::button:Menu:Menu:Opens menu:Opens menu:toggle button:toggle button:true:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.contentEditable resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const box = document.createElement('section'); const child = document.createElement('span'); box.appendChild(child); const before = box.contentEditable; const childBefore = child.isContentEditable; box.contentEditable = 'true'; const during = box.contentEditable; const childDuring = child.isContentEditable; box.contentEditable = 'false'; document.getElementById('out').textContent = before + ':' + String(childBefore) + ':' + during + ':' + String(childDuring) + ':' + String(box.isContentEditable) + ':' + box.getAttribute('contenteditable');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "inherit:false:true:true:false:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.tabIndex resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const button = document.createElement('button'); const panel = document.createElement('div'); const buttonBefore = button.tabIndex; const panelBefore = panel.tabIndex; panel.tabIndex = 3; const panelDuring = panel.tabIndex; panel.tabIndex = -1; document.getElementById('out').textContent = String(buttonBefore) + ':' + String(panelBefore) + ':' + String(panelDuring) + ':' + String(panel.tabIndex) + ':' + panel.getAttribute('tabindex');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "0:-1:3:-1:-1");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 Element.title lang and dir resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='localized'></section><div id='out'></div><script>const localized = document.getElementById('localized'); localized.title = 'Greeting'; localized.lang = 'en-US'; localized.dir = 'rtl'; document.getElementById('out').textContent = localized.title + ':' + localized.lang + ':' + localized.dir + ':' + document.querySelector(':lang(en)').id + ':' + document.querySelector(':dir(rtl)').id;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "Greeting:en-US:rtl:localized:localized");
    try subject.assertExists(":lang(en)");
    try subject.assertExists(":dir(rtl)");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 class and dataset views resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='button' class='base' data-kind='App'>First</button><div id='out'></div><script>document.getElementById('button').className = 'primary secondary'; document.getElementById('button').classList.add('tertiary'); document.getElementById('button').classList.remove('secondary'); const replaced = document.getElementById('button').classList.replace('primary', 'accent'); const missing = document.getElementById('button').classList.replace('missing', 'other'); document.getElementById('button').dataset.userId = '42'; document.getElementById('out').textContent = String(document.getElementById('button').classList.length) + ':' + String(document.getElementById('button').classList.contains('accent')) + ':' + String(replaced) + ':' + String(missing) + ':' + String(document.getElementById('button').classList.toggle('active')) + ':' + document.getElementById('button').className + ':' + document.getElementById('button').dataset.kind + ':' + document.getElementById('button').dataset.userId + ':' + String(document.getElementById('button').classList) + ':' + String(document.getElementById('button').dataset);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2:true:true:false:true:accent tertiary active:App:42:[object DOMTokenList]:[object DOMStringMap]");
    try subject.assertExists(".active");
    try subject.assertExists("[data-user-id]");
    try subject.assertExists("[data-kind=App]");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 inline style declaration surface resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='box' style='color: red; background-color: white;'></div><div id='out'></div><script>const box = document.getElementById('box'); const style = box.style; const before = String(style); style.backgroundColor = 'blue'; style.setProperty('border-top-width', '2px'); style.removeProperty('color'); document.getElementById('out').textContent = before + ':' + box.getAttribute('style') + ':' + String(style.length) + ':' + style.item(0) + ':' + style.getPropertyValue('background-color');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "color: red; background-color: white;:background-color: blue; border-top-width: 2px;:2:background-color:blue");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 9 getComputedStyle resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='box' style='color: red; background-color: white;'></div><div id='out'></div><script>const box = document.getElementById('box'); const style = window.getComputedStyle(box); const before = String(style); box.style.backgroundColor = 'blue'; const after = String(style); document.getElementById('out').textContent = before + ':' + after + ':' + style.getPropertyValue('background-color') + ':' + String(window.getComputedStyle(box, null));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "color: red; background-color: white;:color: red; background-color: blue;:blue:color: red; background-color: blue;");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 9 getBoundingClientRect resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='box' style='left: 10px; top: 20px; width: 30px; height: 40px;'></div><div id='out'></div><script>const box = document.getElementById('box'); const rect = box.getBoundingClientRect(); box.style.left = '15px'; box.style.width = '60px'; document.getElementById('out').textContent = String(rect) + ':' + String(rect.left) + ':' + String(rect.width) + ':' + String(rect.right) + ':' + String(box.getBoundingClientRect().left) + ':' + String(box.getBoundingClientRect().width) + ':' + String(box.getBoundingClientRect().right);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[object DOMRect]:10:30:40:15:60:75");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 9 getClientRects resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='box' style='left: 10px; top: 20px; width: 30px; height: 40px;'></div><div id='out'></div><script>const box = document.getElementById('box'); const rects = box.getClientRects(); box.style.left = '15px'; box.style.width = '60px'; document.getElementById('out').textContent = String(rects) + ':' + String(rects.length) + ':' + String(rects.item(0)) + ':' + String(rects.item(0).left) + ':' + String(rects.item(0).width) + ':' + String(box.getClientRects().item(0).left) + ':' + String(box.getClientRects().item(0).width);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[object DOMRectList]:1:[object DOMRect]:10:30:15:60");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 inline style important priority resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='box' style='/* lead */ color: red !important; background-color: white; /* tail */'></div><div id='out'></div><script>const box = document.getElementById('box'); const style = box.style; const before = String(style); style.setProperty('border-top-width', '2px', 'important'); document.getElementById('out').textContent = before + ':' + String(style) + ':' + box.getAttribute('style') + ':' + style.getPropertyValue('color');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "color: red !important; background-color: white;:color: red !important; background-color: white; border-top-width: 2px !important;:color: red !important; background-color: white; border-top-width: 2px !important;:red");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 inline style declaration parsing survives semicolons inside values on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='box'></div><div id='out'></div><script>const box = document.getElementById('box'); const style = box.style; style.cssText = \"content: 'A;B'; background-image: url(data:image/svg+xml;utf8,foo);\"; style.setProperty('border-image-source', 'url(data:image/svg+xml;utf8,bar)'); document.getElementById('out').textContent = style.cssText + '|' + style.getPropertyValue('content') + '|' + style.getPropertyValue('background-image') + '|' + style.getPropertyValue('border-image-source');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "content: 'A;B'; background-image: url(data:image/svg+xml;utf8,foo); border-image-source: url(data:image/svg+xml;utf8,bar);|'A;B'|url(data:image/svg+xml;utf8,foo)|url(data:image/svg+xml;utf8,bar)");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 inline style property priority resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='box' style='color: red !important; background-color: white;'></div><div id='out'></div><script>const style = document.getElementById('box').style; document.getElementById('out').textContent = style.getPropertyPriority('color') + ':' + style.getPropertyPriority('background-color') + ':' + style.getPropertyPriority('missing');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "important::");
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

test "regression: phase 8 tree mutation replaceWith resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='source'><button id='old'>Old</button><span id='tail'>Tail</span></section><button id='replacement'>Replacement</button><div id='out'></div><script>document.getElementById('old').replaceWith(document.getElementById('replacement')); document.getElementById('out').textContent = document.getElementById('source').textContent + ':' + String(document.querySelectorAll('#source > button').length) + ':' + document.querySelector('#source > #replacement').textContent + ':' + String(document.querySelector('#old'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "ReplacementTail:1:Replacement:null");
    try subject.assertExists("#source > #replacement");
    try subject.assertExists("#source > #tail");
    try std.testing.expectError(error.AssertionFailed, subject.assertExists("#old"));
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 tree mutation cloneNode resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='source' data-kind='orig'><button id='button'>One</button><span id='tail'>Tail</span></section><template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const clone = document.getElementById('source').cloneNode(true); const fragment = document.getElementById('tpl').content.cloneNode(); document.getElementById('out').textContent = clone.getAttribute('id') + ':' + clone.getAttribute('data-kind') + ':' + String(clone.parentNode) + ':' + clone.textContent + ':' + String(clone.querySelectorAll('button').length) + ':' + String(document.querySelectorAll('#source').length) + '|' + String(fragment) + ':' + fragment.innerHTML + ':' + String(fragment.childNodes.length) + ':' + String(fragment.querySelector('#inner')) + ':' + document.getElementById('tpl').content.textContent;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "source:orig:null:OneTail:1:1|[object DocumentFragment]::0:null:Inner");
    try subject.assertExists("#source");
    try subject.assertExists("#source > #button");
    try subject.assertExists("#source > #tail");
    try subject.assertExists("#tpl");
    try subject.assertExists("#tpl > #inner");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 detached construction resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='target'><span id='text-source'>Old</span><span id='comment-source'>Keep</span></section><div id='out'></div><script>const article = document.createElement('ARTICLE'); article.setAttribute('id', 'created'); article.textContent = 'Body'; const text = document.createTextNode('Replaced'); const comment = document.createComment('note'); const fragment = document.createDocumentFragment(); fragment.innerHTML = '<span id=\"fragment-child\">Fragment</span>'; const before = String(article.parentNode) + ':' + String(text.parentNode) + ':' + String(comment.parentNode) + ':' + String(fragment) + ':' + fragment.innerHTML + ':' + String(fragment.childNodes.length) + ':' + String(fragment.hasChildNodes()) + ':' + String(fragment.isConnected) + ':' + String(fragment.firstChild) + ':' + String(fragment.lastChild) + ':' + String(fragment.nextSibling) + ':' + String(fragment.previousSibling) + ':' + String(fragment.ownerDocument) + ':' + String(fragment.parentNode) + ':' + String(fragment.parentElement) + ':' + String(fragment.querySelector('#fragment-child')) + ':' + String(text) + ':' + String(comment) + ':' + text.textContent + ':' + comment.textContent; document.getElementById('target').appendChild(article); document.getElementById('text-source').childNodes.item(0).replaceWith(text); document.getElementById('comment-source').childNodes.item(0).replaceWith(comment); document.getElementById('out').textContent = before + '|' + document.getElementById('target').innerHTML + '|' + document.getElementById('text-source').innerHTML + '|' + document.getElementById('comment-source').innerHTML + '|' + String(document.querySelector('#created'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "null:null:null:[object DocumentFragment]:<span id=\"fragment-child\">Fragment</span>:1:true:false:[object Element]:[object Element]:null:null:[object Document]:null:null:[object Element]:[object Node]:[object Node]:Replaced:note|<span id=\"text-source\">Replaced</span><span id=\"comment-source\"><!--note--></span><article id=\"created\">Body</article>|Replaced|<!--note-->|[object Element]",
    );
    try subject.assertExists("#target > #text-source");
    try subject.assertExists("#target > #comment-source");
    try subject.assertExists("#target > #created");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 element.innerText resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='panel'><span>One</span><span>Two</span></section><div id='out'></div><script>const panel = document.getElementById('panel'); const before = panel.innerText; panel.innerText = 'Reset'; document.getElementById('out').textContent = before + ':' + panel.innerText + ':' + panel.textContent;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "OneTwo:Reset:Reset");
    try subject.assertExists("#panel");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 element.outerText resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='panel'><span>One</span><span>Two</span></section></main><div id='out'></div><script>const panel = document.getElementById('panel'); const before = panel.outerText; panel.outerText = 'Reset'; document.getElementById('out').textContent = before + ':' + document.getElementById('root').textContent + ':' + String(document.getElementById('panel'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "OneTwo:Reset:null");
    try subject.assertExists("#root");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 createElementNS resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg'); const gradient = document.createElementNS('http://www.w3.org/2000/svg', 'linearGradient'); const html = document.createElementNS('http://www.w3.org/1999/xhtml', 'DIV'); const fragment = document.createDocumentFragment(); svg.appendChild(gradient); document.getElementById('root').appendChild(svg); document.getElementById('root').appendChild(html); document.getElementById('out').textContent = svg.namespaceURI + ':' + gradient.namespaceURI + ':' + html.namespaceURI + ':' + String(fragment.namespaceURI) + ':' + svg.nodeName + ':' + gradient.nodeName + ':' + html.nodeName + ':' + svg.outerHTML + '|' + html.outerHTML;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "http://www.w3.org/2000/svg:http://www.w3.org/2000/svg:http://www.w3.org/1999/xhtml:null:svg:linearGradient:div:<svg><linearGradient></linearGradient></svg>|<div></div>");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 createAttributeNS resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='out'></div><script>const namespaced = document.createAttributeNS('urn:test', 'svg:stroke'); namespaced.nodeValue = 'azure'; const plain = document.createAttributeNS(null, 'data-role'); plain.value = 'dialog'; document.getElementById('out').textContent = String(namespaced) + ':' + namespaced.name + ':' + String(namespaced.namespaceURI) + ':' + namespaced.localName + ':' + String(namespaced.prefix) + ':' + namespaced.nodeName + ':' + String(namespaced.nodeType) + ':' + namespaced.value + ':' + namespaced.data + ':' + namespaced.textContent + ':' + String(namespaced.ownerDocument) + ':' + String(namespaced.parentNode) + ':' + String(namespaced.parentElement) + ':' + String(namespaced.ownerElement) + ':' + String(plain.namespaceURI) + ':' + plain.name + ':' + plain.localName + ':' + String(plain.prefix) + ':' + plain.value;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object Attr]:svg:stroke:urn:test:stroke:svg:svg:stroke:2:azure:azure:azure:[object Document]:null:null:null:null:data-role:data-role:null:dialog",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 element.attributes resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='host' data-role='menu' aria-label='Label'></div><div id='out'></div><script>const attrs = document.getElementById('host').attributes; const keys = attrs.keys(); const values = attrs.values(); const entries = attrs.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); document.getElementById('out').textContent = String(attrs) + ':' + String(attrs.length) + ':' + String(firstKey.value) + ':' + String(firstValue.value) + ':' + firstValue.value.name + ':' + firstValue.value.value + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.name + ':' + firstEntry.value.value.value + ':' + String(attrs.getNamedItem('data-role')) + ':' + String(attrs.getNamedItemNS(null, 'aria-label')) + ':' + String(attrs.item(99));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[object NamedNodeMap]:3:0:[object Attr]:id:host:0:id:host:[object Attr]:[object Attr]:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 element.attributes namespace lookups resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='host' data-role='menu'></div><div id='out'></div><script>const host = document.getElementById('host'); host.setAttributeNS('urn:test', 'svg:stroke', 'azure'); const attrs = host.attributes; document.getElementById('out').textContent = String(attrs.length) + ':' + String(attrs.getNamedItemNS('urn:test', 'stroke')) + ':' + attrs.getNamedItemNS('urn:test', 'stroke').namespaceURI + ':' + attrs.getNamedItemNS('urn:test', 'stroke').prefix + ':' + String(attrs.getNamedItemNS(null, 'stroke')) + ':' + String(attrs.item(2)) + ':' + attrs.item(2).name + ':' + attrs.item(2).prefix;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "3:[object Attr]:urn:test:svg:null:[object Attr]:svg:stroke:svg");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 element.attributes mutators resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='host' data-role='menu' aria-label='Label'></div><div id='out'></div><script>const host = document.getElementById('host'); const attrs = host.attributes; const replacement = document.createAttribute('data-role'); replacement.value = 'dialog'; const previous = attrs.setNamedItem(replacement); const namespaced = document.createAttributeNS('urn:test', 'svg:stroke'); namespaced.value = 'azure'; const nsPrevious = attrs.setNamedItemNS(namespaced); const before = String(previous) + ':' + previous.name + ':' + previous.value + ':' + String(previous.ownerElement) + ':' + String(replacement.ownerElement) + ':' + String(nsPrevious) + ':' + String(namespaced.ownerElement) + ':' + String(attrs.length); document.getElementById('out').textContent = before + ':'; attrs.forEach((attribute, index, list) => { document.getElementById('out').textContent += String(index) + ':' + attribute.name + ':' + attribute.value + ':' + String(list.length) + ';'; }, null); const removed = attrs.removeNamedItem('data-role'); const removedNS = attrs.removeNamedItemNS('urn:test', 'stroke'); document.getElementById('out').textContent += '|' + String(removed) + ':' + removed.value + ':' + String(removed.ownerElement) + ':' + String(removedNS) + ':' + removedNS.name + ':' + removedNS.prefix + ':' + String(removedNS.ownerElement) + ':' + String(attrs.length) + ':' + String(replacement.ownerElement) + ':' + String(namespaced.ownerElement) + ':' + String(host.getAttributeNode('data-role')) + ':' + String(host.getAttributeNodeNS('urn:test', 'stroke'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object Attr]:data-role:menu:null:[object Element]:null:[object Element]:4:0:id:host:4;1:data-role:dialog:4;2:aria-label:Label:4;3:svg:stroke:azure:4;|[object Attr]:dialog:null:[object Attr]:svg:stroke:svg:null:2:[object Element]:[object Element]:null:null",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 createAttribute resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='out'></div><script>const attr = document.createAttribute('data-role'); attr.value = 'dialog'; document.getElementById('out').textContent = String(attr) + ':' + attr.name + ':' + String(attr.namespaceURI) + ':' + attr.localName + ':' + String(attr.prefix) + ':' + attr.nodeName + ':' + String(attr.nodeType) + ':' + attr.value + ':' + attr.data + ':' + attr.textContent + ':' + String(attr.ownerDocument) + ':' + String(attr.parentNode) + ':' + String(attr.parentElement) + ':' + String(attr.ownerElement);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[object Attr]:data-role:null:data-role:null:data-role:2:dialog:dialog:dialog:[object Document]:null:null:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 element attribute node APIs resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='host' data-role='menu'></div><div id='out'></div><script>const host = document.getElementById('host'); const created = document.createAttribute('data-state'); created.value = 'open'; const previous = host.setAttributeNode(created); const snapshot = host.getAttributeNode('data-state'); const removed = host.removeAttributeNode(created); const attached = host.getAttributeNode('data-role'); document.getElementById('out').textContent = String(previous) + ':' + String(snapshot) + ':' + snapshot.name + ':' + snapshot.value + ':' + String(snapshot.ownerElement) + ':' + String(created.ownerElement) + ':' + String(removed) + ':' + String(host.getAttributeNode('data-state')) + ':' + String(attached) + ':' + attached.value + ':' + String(attached.ownerElement);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "null:[object Attr]:data-state:open:[object Element]:null:[object Attr]:null:[object Attr]:menu:[object Element]");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 element attribute node NS APIs resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<div id='host'></div><div id='out'></div><script>const host = document.getElementById('host'); const created = document.createAttributeNS('urn:test', 'svg:stroke'); created.value = 'azure'; const previous = host.setAttributeNodeNS(created); const snapshot = host.getAttributeNodeNS('urn:test', 'stroke'); document.getElementById('out').textContent = String(previous) + ':' + String(snapshot) + ':' + snapshot.name + ':' + String(snapshot.namespaceURI) + ':' + snapshot.localName + ':' + String(snapshot.prefix) + ':' + snapshot.value + ':' + String(snapshot.ownerElement) + ':' + String(created.ownerElement);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "null:[object Attr]:svg:stroke:urn:test:stroke:svg:azure:[object Element]:[object Element]");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 9 Node.nodeValue, data, and splitText resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='host'>Hello</div><div id='out'></div><script>const element = document.getElementById('out'); const fragment = document.createDocumentFragment(); const text = document.createTextNode('Hello'); const comment = document.createComment('note'); element.nodeValue = 'ignored'; fragment.nodeValue = 'ignored'; document.nodeValue = 'ignored'; text.data = 'World'; comment.data = 'updated'; const split = document.getElementById('host').childNodes.item(0).splitText(2); document.getElementById('out').textContent = String(document.nodeValue) + ':' + String(element.nodeValue) + ':' + String(fragment.nodeValue) + ':' + text.data + ':' + comment.data + ':' + text.nodeValue + ':' + comment.nodeValue + ':' + text.textContent + ':' + comment.textContent + ':' + split.data + ':' + split.parentNode.textContent + ':' + String(split.parentNode.childNodes.length);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "null:null:null:World:updated:World:updated:World:updated:llo:Hello:2");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 9 Text.wholeText resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='host'>Hello</div><div id='out'></div><script>const text = document.getElementById('host').childNodes.item(0); const split = text.splitText(3); document.getElementById('out').textContent = text.wholeText + ':' + split.wholeText + ':' + text.data + ':' + split.data + ':' + String(text.length) + ':' + String(split.length);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "Hello:Hello:Hel:lo:3:2");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 9 CharacterData methods resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='host'>Hello</div><div id='note'><!--note--></div><div id='out'></div><script>const text = document.getElementById('host').childNodes.item(0); const comment = document.getElementById('note').childNodes.item(0); const substring = text.substringData(1, 3); text.appendData('!'); text.insertData(1, 'X'); text.deleteData(2, 2); text.replaceData(1, 2, 'Q'); comment.appendData('!'); comment.insertData(0, '['); comment.deleteData(1, 1); comment.replaceData(0, 1, '('); document.getElementById('out').textContent = substring + ':' + text.data + ':' + String(text.length) + ':' + comment.data + ':' + String(comment.length) + ':' + text.substringData(0, 2) + ':' + comment.substringData(0, 2);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "ell:HQo!:4:(ote!:5:HQ:(o");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 tree mutation removeChild resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='target'><button id='first'>First</button><button id='second'>Second</button></section><div id='out'></div><script>const target = document.getElementById('target'); const removed = target.removeChild(document.getElementById('second')); document.getElementById('out').textContent = String(removed) + ':' + removed.textContent + ':' + String(target.childNodes.length) + ':' + String(document.querySelector('#second'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[object Element]:Second:1:null");
    try subject.assertExists("#target > #first");
    try std.testing.expectError(error.AssertionFailed, subject.assertExists("#second"));
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 normalize and importNode resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='target'>a</div><template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const text = document.getElementById('target').childNodes.item(0); text.replaceWith(document.createTextNode('a'), document.createTextNode(''), document.createTextNode('b')); document.getElementById('target').normalize(); const fragment = document.importNode(document.getElementById('tpl').content, true); document.getElementById('out').textContent = String(document.getElementById('target').childNodes.length) + ':' + document.getElementById('target').textContent + '|' + String(fragment) + ':' + fragment.innerHTML + ':' + String(fragment.childNodes.length) + ':' + String(fragment.querySelector('#inner'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:ab|[object DocumentFragment]:<span id=\"inner\">Inner</span>:1:[object Element]");
    try subject.assertExists("#target");
    try subject.assertExists("#tpl > #inner");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 9 Node.contains resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='outer'><span id='inside'>Inside</span></section><template id='tpl'><span id='frag'>Frag</span></template><div id='out'></div><script>const outer = document.getElementById('outer'); const inside = document.getElementById('inside'); const fragment = document.getElementById('tpl').content; const fragmentChild = fragment.querySelector('#frag'); document.getElementById('out').textContent = String(document.contains(document)) + ':' + String(document.contains(outer)) + ':' + String(document.contains(inside)) + ':' + String(outer.contains(inside)) + ':' + String(inside.contains(outer)) + ':' + String(fragment.contains(fragmentChild)) + ':' + String(fragment.contains(document)) + ':' + String(document.contains(null)) + ':' + String(document.contains(fragmentChild));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:true:true:true:false:true:false:false:false");
    try subject.assertExists("#outer > #inside");
    try subject.assertExists("#tpl > #frag");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 9 Node.compareDocumentPosition resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='outer'><span id='inside'>Inside</span></section><template id='tpl'><span id='frag'>Frag</span></template><div id='out'></div><script>const outer = document.getElementById('outer'); const inside = document.getElementById('inside'); const fragment = document.getElementById('tpl').content; const fragmentChild = fragment.querySelector('#frag'); document.getElementById('out').textContent = String(document.compareDocumentPosition(outer)) + ':' + String(outer.compareDocumentPosition(document)) + ':' + String(outer.compareDocumentPosition(inside)) + ':' + String(inside.compareDocumentPosition(outer)) + ':' + String(outer.compareDocumentPosition(fragmentChild)) + ':' + String(fragmentChild.compareDocumentPosition(outer)) + ':' + String(fragment.compareDocumentPosition(fragmentChild)) + ':' + String(fragmentChild.compareDocumentPosition(fragment)) + ':' + String(document.compareDocumentPosition(document));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "20:10:20:10:37:35:20:10:0");
    try subject.assertExists("#outer > #inside");
    try subject.assertExists("#tpl > #frag");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 9 Node.isSameNode and Node.isEqualNode resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const left = document.createElement('div'); left.appendChild(document.createTextNode('Hello')); const right = document.createElement('div'); right.appendChild(document.createTextNode('Hello')); const fragLeft = document.createDocumentFragment(); fragLeft.appendChild(document.createTextNode('Hello')); const fragRight = document.createDocumentFragment(); fragRight.appendChild(document.createTextNode('Hello')); document.getElementById('out').textContent = String(document.isSameNode(document)) + ':' + String(document.isEqualNode(document)) + ':' + String(left.isSameNode(right)) + ':' + String(left.isEqualNode(right)) + ':' + String(fragLeft.isSameNode(fragRight)) + ':' + String(fragLeft.isEqualNode(fragRight));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:true:false:true:false:true");
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

test "regression: phase 8 HTML serialization mixed-quote attributes resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='target'></div><div id='out'></div><script>document.getElementById('target').setAttribute('data-label', 'a\\'b\"c&d<e>'); document.getElementById('out').textContent = document.getElementById('target').outerHTML;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "<div data-label=\"a'b&quot;c&amp;d&lt;e&gt;\" id=\"target\"></div>",
    );
    try subject.assertExists("#target");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 HTML serialization character references resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='attr' data-label='a&amp;b&copy;&reg;&nbsp;&#160;&#xA0;&AMP;&LT;&GT;&QUOT;&NBSP;&COPY;&REG'></div><div id='text'>a&amp;b&copy;&reg;&nbsp;&#160;&#xA0;&AMP;&LT;&GT;&QUOT;&NBSP;&COPY;&REG</div><div id='out'></div><script>const attr = document.getElementById('attr').getAttribute('data-label'); const text = document.getElementById('text').textContent; const html = document.getElementById('attr').outerHTML; document.getElementById('out').textContent = attr + '|' + text + '|' + html;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "a&b\xc2\xa9\xc2\xae\xc2\xa0\xc2\xa0\xc2\xa0&<>\"\xc2\xa0\xc2\xa9\xc2\xae|a&b\xc2\xa9\xc2\xae\xc2\xa0\xc2\xa0\xc2\xa0&<>\"\xc2\xa0\xc2\xa9\xc2\xae|<div data-label='a&amp;b\xc2\xa9\xc2\xae\xc2\xa0\xc2\xa0\xc2\xa0&amp;&lt;&gt;\"\xc2\xa0\xc2\xa9\xc2\xae' id=\"attr\"></div>",
    );
    try subject.assertExists("#attr");
    try subject.assertExists("#text");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 document.open write writeln and close resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const opened = document.open(); document.write('<main id=\"root\"><div id=\"out\"></div><span id=\"name\">Ada</span>'); document.writeln('</main>'); document.close(); document.getElementById('out').textContent = String(opened) + ':' + document.getElementById('name').textContent + ':' + String(document.getElementById('root').nextSibling.nodeType);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[object Document]:Ada:3");
    try subject.assertExists("#name");
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

test "regression: phase 8 insertAdjacentElement and insertAdjacentText resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section></main><div id='out'></div><script>const target = document.getElementById('target'); const before = document.createElement('aside'); before.id = 'before'; const inserted = target.insertAdjacentElement('beforebegin', before); const text = target.insertAdjacentText('afterbegin', 'First'); target.insertAdjacentText('beforeend', 'Last'); const after = document.createElement('aside'); after.id = 'after'; target.insertAdjacentElement('afterend', after); document.getElementById('out').textContent = String(inserted) + ':' + String(text) + ':' + document.getElementById('root').innerHTML + ':' + document.getElementById('target').innerHTML;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object Element]:undefined:<aside id=\"before\"></aside><section id=\"target\">First<button class=\"primary\" id=\"old\">Old</button>Last</section><aside id=\"after\"></aside>:First<button class=\"primary\" id=\"old\">Old</button>Last",
    );
    try subject.assertExists("#before");
    try subject.assertExists("#after");
    try subject.assertExists("#target > #old");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 template.content surfaces resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<template id='tpl'><span id='first'>First</span><em id='middle'>Middle</em>text<b id='last'>Last</b></template><div id='out'></div><script>const content = document.getElementById('tpl').content; document.getElementById('out').textContent = String(content) + '|' + content.innerHTML + '|' + content.firstElementChild.id + ':' + content.lastElementChild.id + ':' + String(content.childElementCount); content.innerHTML = '<span id=\"second\">Second</span>text<b id=\"third\">Third</b>'; document.getElementById('out').textContent += '|' + String(content) + '|' + content.innerHTML + '|' + content.firstElementChild.id + ':' + content.lastElementChild.id + ':' + String(content.childElementCount);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object DocumentFragment]|<span id=\"first\">First</span><em id=\"middle\">Middle</em>text<b id=\"last\">Last</b>|first:last:3|[object DocumentFragment]|<span id=\"second\">Second</span>text<b id=\"third\">Third</b>|second:third:2",
    );
    try subject.assertExists("#second");
    try subject.assertExists("#third");
    try std.testing.expectError(error.AssertionFailed, subject.assertExists("#first"));
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8a template.content.children resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<template id='tpl'><span id='first'>First</span></template><div id='out'></div><script>const content = document.getElementById('tpl').content; const children = content.children; const before = children.length; const first = children.item(0); const namedBefore = children.namedItem('second'); const second = document.createElement('span'); second.id = 'second'; second.textContent = 'Second'; content.appendChild(second); document.getElementById('out').textContent = String(before) + ':' + String(children.length) + ':' + first.id + ':' + children.item(1).id + ':' + String(namedBefore) + ':' + children.namedItem('second').id;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:2:first:second:null:second");
    try subject.assertExists("#first");
    try subject.assertExists("#second");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8a1 template.content.childNodes resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<template id='tpl'><span id='first'>First</span></template><div id='out'></div><script>const content = document.getElementById('tpl').content; const nodes = content.childNodes; const before = nodes.length; const first = nodes.item(0); const text = document.createTextNode('Second'); content.appendChild(text); document.getElementById('out').textContent = String(before) + ':' + String(nodes.length) + ':' + first.nodeName + ':' + nodes.item(1).nodeName + ':' + nodes.item(1).data;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:2:span:#text:Second");
    try subject.assertExists("#first");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8a2 template.content fragment traversal resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<template id='tpl'><span id='first'>First</span>text<b id='last'>Last</b></template><div id='out'></div><script>const content = document.getElementById('tpl').content; const first = content.firstChild; const last = content.lastChild; const firstElement = content.firstElementChild; const lastElement = content.lastElementChild; const text = first.nextSibling; document.getElementById('out').textContent = String(content.isConnected) + ':' + String(content.hasChildNodes()) + ':' + first.nodeName + ':' + last.nodeName + ':' + text.nodeName + ':' + text.data + ':' + String(first.previousSibling) + ':' + String(firstElement) + ':' + String(lastElement) + ':' + firstElement.nextElementSibling.nodeName + ':' + lastElement.previousElementSibling.nodeName;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:true:span:b:#text:text:null:[object Element]:[object Element]:b:span");
    try subject.assertExists("#first");
    try subject.assertExists("#last");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8a3 HTMLTemplateElement declarative shadow root flags resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><template id='tpl' shadowrootmode='open' shadowrootdelegatesfocus shadowrootclonable shadowrootserializable shadowrootcustomelementregistry='registry'><span>Hidden</span></template><div id='out'></div><script>const tpl = document.getElementById('tpl'); const before = tpl.shadowRootMode + ':' + String(tpl.shadowRootDelegatesFocus) + ':' + String(tpl.shadowRootClonable) + ':' + String(tpl.shadowRootSerializable) + ':' + tpl.shadowRootCustomElementRegistry; tpl.shadowRootMode = 'closed'; tpl.shadowRootDelegatesFocus = false; tpl.shadowRootClonable = false; tpl.shadowRootSerializable = true; tpl.shadowRootCustomElementRegistry = 'registry-2'; document.getElementById('out').textContent = before + '|' + tpl.shadowRootMode + ':' + String(tpl.shadowRootDelegatesFocus) + ':' + String(tpl.shadowRootClonable) + ':' + String(tpl.shadowRootSerializable) + ':' + tpl.shadowRootCustomElementRegistry + ':' + String(tpl.hasAttribute('shadowrootmode')) + ':' + String(tpl.hasAttribute('shadowrootdelegatesfocus')) + ':' + String(tpl.hasAttribute('shadowrootclonable')) + ':' + String(tpl.hasAttribute('shadowrootserializable'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "open:true:true:true:registry|closed:false:false:true:registry-2:true:false:false:true",
    );
    try subject.assertExists("#tpl");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 template.content query methods resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<template id='tpl'><span id='inner'>Inner</span><span id='second' class='hit'>Second</span></template><div id='out'></div><script>const content = document.getElementById('tpl').content; document.getElementById('out').textContent = String(content) + ':' + String(content.getElementById('inner')) + ':' + String(content.querySelector('.hit')) + ':' + String(content.querySelectorAll('span').length) + ':' + content.querySelectorAll('span').item(1).textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "[object DocumentFragment]:[object Element]:[object Element]:2:Second",
    );
    try subject.assertExists("#inner");
    try subject.assertExists("#second");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 8 HTMLTemplateElement declarative shadow root flags resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><template id='tpl' shadowrootmode='open' shadowrootdelegatesfocus shadowrootclonable shadowrootserializable shadowrootcustomelementregistry='registry'><span>Hidden</span></template><div id='out'></div><script>const tpl = document.getElementById('tpl'); const before = tpl.shadowRootMode + ':' + String(tpl.shadowRootDelegatesFocus) + ':' + String(tpl.shadowRootClonable) + ':' + String(tpl.shadowRootSerializable) + ':' + tpl.shadowRootCustomElementRegistry; tpl.shadowRootMode = 'closed'; tpl.shadowRootDelegatesFocus = false; tpl.shadowRootClonable = false; tpl.shadowRootSerializable = true; tpl.shadowRootCustomElementRegistry = 'registry-2'; document.getElementById('out').textContent = before + '|' + tpl.shadowRootMode + ':' + String(tpl.shadowRootDelegatesFocus) + ':' + String(tpl.shadowRootClonable) + ':' + String(tpl.shadowRootSerializable) + ':' + tpl.shadowRootCustomElementRegistry + ':' + String(tpl.hasAttribute('shadowrootmode')) + ':' + String(tpl.hasAttribute('shadowrootdelegatesfocus')) + ':' + String(tpl.hasAttribute('shadowrootclonable')) + ':' + String(tpl.hasAttribute('shadowrootserializable'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "open:true:true:true:registry|closed:false:false:true:registry-2:true:false:false:true",
    );
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
    const original = "<input id='upload' type='file'><div id='out'></div><script>const upload = document.getElementById('upload'); document.getElementById('out').textContent = String(upload.files.length) + ':' + String(upload.files.item(0)); upload.addEventListener('change', () => { const current = document.getElementById('upload'); document.getElementById('out').textContent = String(current.files.length) + ':' + current.files.item(0) + ':' + current.files.item(1) + ':' + String(current.files); });</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.setFiles("#upload", &.{"report.csv"});
    try subject.assertValue("#upload", "report.csv");
    try subject.assertValue("#out", "1:report.csv:null:[object FileList]");
    try std.testing.expectEqualStrings(original, subject.html().?);

    const registry = subject.mocksMut();
    const selections = registry.fileInput().selections();
    try std.testing.expectEqual(@as(usize, 1), selections.len);
    try std.testing.expectEqualStrings("#upload", selections[0].selector);
    try std.testing.expectEqual(@as(usize, 1), selections[0].files.len);
    try std.testing.expectEqualStrings("report.csv", selections[0].files[0]);
}

test "regression: phase 4a HTMLInputElement.files resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<input id='upload' type='file'><div id='out'></div><script>const upload = document.getElementById('upload'); document.getElementById('out').textContent = String(upload.files.length) + ':' + String(upload.files.item(0)); upload.addEventListener('change', () => { const current = document.getElementById('upload'); document.getElementById('out').textContent = String(current.files.length) + ':' + String(current.files.item(0)) + ':' + String(current.files); });</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.setFiles("#upload", &.{"report.csv"});
    try subject.assertValue("#out", "1:report.csv:[object FileList]");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 4a1 HTMLInputElement.files iterator helpers resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='upload' type='file'><div id='out'></div><script>const upload = document.getElementById('upload'); const files = upload.files; const keys = files.keys(); const values = files.values(); const entries = files.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); document.getElementById('out').textContent = String(files.length) + ':' + String(firstKey.done) + ':' + String(firstValue.done) + ':' + String(firstEntry.done) + ':' + String(files.item(0)); files.forEach((value, index, list) => { document.getElementById('out').textContent += String(index) + ':' + value + ':' + String(list.length) + ';'; }, null);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "0:true:true:true:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47s FileList iterator parity resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<input id='upload' type='file'><div id='out'></div><script>const upload = document.getElementById('upload'); upload.addEventListener('change', () => { const current = document.getElementById('upload'); const keys = current.files.keys(); const values = current.files.values(); const entries = current.files.entries(); const firstKey = keys.next(); const secondKey = keys.next(); const thirdKey = keys.next(); const firstValue = values.next(); const secondValue = values.next(); const thirdValue = values.next(); const firstEntry = entries.next(); const secondEntry = entries.next(); const thirdEntry = entries.next(); const before = String(current.files.length) + ':' + String(firstKey.value) + ':' + String(secondKey.value) + ':' + String(thirdKey.done) + '|' + firstValue.value + ':' + secondValue.value + ':' + String(thirdValue.done) + '|' + String(firstEntry.value.index) + ':' + firstEntry.value.value + ':' + String(secondEntry.value.index) + ':' + secondEntry.value.value + ':' + String(thirdEntry.done) + '|' + String(current.files.item(0)) + ':' + String(current.files.item(1)) + ':' + String(current.files.item(2)); document.getElementById('out').textContent = ''; current.files.forEach((value, index, list) => { document.getElementById('out').textContent += String(index) + ':' + value + ':' + String(list.length) + ';'; }, null); document.getElementById('out').textContent = before + '|' + document.getElementById('out').textContent; });</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.setFiles("#upload", &.{ "alpha.csv", "beta.csv" });
    try subject.assertValue("#out", "2:0:1:true|alpha.csv:beta.csv:true|0:alpha.csv:1:beta.csv:true|alpha.csv:beta.csv:null|0:alpha.csv:2;1:beta.csv:2;");
    try std.testing.expectEqualStrings(original, subject.html().?);

    const registry = subject.mocksMut();
    const selections = registry.fileInput().selections();
    try std.testing.expectEqual(@as(usize, 1), selections.len);
    try std.testing.expectEqualStrings("#upload", selections[0].selector);
    try std.testing.expectEqual(@as(usize, 2), selections[0].files.len);
    try std.testing.expectEqualStrings("alpha.csv", selections[0].files[0]);
    try std.testing.expectEqualStrings("beta.csv", selections[0].files[1]);
}

test "regression: phase 25 target and nth pseudo-class selectors operate on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><a id='fallback' name='named'>Named</a><section id='list'><span id='a' class='match'>A</span><div id='b'>B</div><span id='c'>C</span><div id='d' class='match'>D</div></section><div id='out'></div><script>document.getElementById('out').textContent = document.querySelector(':target').getAttribute('id') + ':' + document.querySelector('#list > span:nth-child(1)').getAttribute('id') + ':' + document.querySelector('#list > div:nth-of-type(2)').getAttribute('id');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://app.local/#named", html_bytes);
    defer subject.deinit();

    try subject.assertValue("#out", "fallback:a:d");
    try subject.navigate("https://app.local/#d");
    try subject.assertExists("#d:target");

    html_bytes[1] = 'Z';
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 26 :not, :is, and :where pseudo-class selectors operate on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='buttons'><button id='first' class='primary'>First</button><button id='second' class='secondary'>Second</button></section><div id='out'></div><script>document.getElementById('out').textContent = document.querySelector('#buttons > button:not(.missing, .secondary)').getAttribute('id') + ':' + String(document.querySelectorAll('#buttons > button:is(.primary, .secondary)').length) + ':' + String(document.querySelectorAll('#buttons > button:where(.primary, .secondary)').length) + ':' + document.querySelector('#buttons > button:where(.missing, .secondary)').getAttribute('id');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "first:2:2:second");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 27 document.documentElement, head, body, and title aliases resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<html id='html'><head id='head'><title>Initial</title></head><body id='body'><main id='out'></main><script>document.title = 'Updated'; const html = document.documentElement; const head = document.head; const body = document.body; document.getElementById('out').textContent = html.getAttribute('id') + ':' + head.getAttribute('id') + ':' + body.getAttribute('id') + ':' + document.title;</script></body></html>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "html:head:body:Updated");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 27 document.location and window.location aliases resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const before = document.location; document.location = 'https://example.test:8443/next'; const after = window.location; document.getElementById('out').textContent = before + ':' + after;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://example.test:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "https://example.test:8443/start?x#old:https://example.test:8443/next");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 28 Location href and navigation methods resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const location = window.location; const before = location.href; location.assign('https://example.test:8443/assign'); const afterAssign = location.href; location.href = 'https://example.test:8443/href'; const afterHref = location.href; location.replace('https://example.test:8443/replace'); const afterReplace = location.href; location.reload(); const afterReload = location.href; document.getElementById('out').textContent = before + ':' + afterAssign + ':' + afterHref + ':' + afterReplace + ':' + afterReload;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://example.test:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

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
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 28b Location.hash getter and setter resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const location = document.location; const before = location.hash; location.hash = '#copy'; const after = window.location.hash; document.getElementById('out').textContent = before + ':' + after + ':' + location.href;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://example.test:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "#old:#copy:https://example.test:8443/start?x#copy");
    try std.testing.expectEqualStrings("https://example.test:8443/start?x#copy", subject.mocksMut().location().currentUrl().?);
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().location().navigations().len);
    try std.testing.expectEqualStrings("https://example.test:8443/start?x#copy", subject.mocksMut().location().navigations()[0]);
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 28c hashchange listeners and onhashchange handlers resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>window.addEventListener('hashchange', () => { document.getElementById('out').textContent += 'listener:' + window.location.hash; }); window.onhashchange = () => { document.getElementById('out').textContent += '|property:' + window.location.hash; }; window.location.hash = 'copy';</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://example.test:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "listener:#copy|property:#copy");
    try std.testing.expectEqualStrings("https://example.test:8443/start?x#copy", subject.mocksMut().location().currentUrl().?);
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().location().navigations().len);
    try std.testing.expectEqualStrings("https://example.test:8443/start?x#copy", subject.mocksMut().location().navigations()[0]);
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 28c2 popstate listeners and onpopstate handlers resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>window.addEventListener('popstate', () => { document.getElementById('out').textContent += 'listener:' + window.history.state; }); window.onpopstate = () => { document.getElementById('out').textContent += '|property:' + window.history.state; }; window.history.pushState('seed', '', 'https://example.test:8443/seed'); window.history.pushState('next', '', 'https://example.test:8443/next'); window.history.back();</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://example.test:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "listener:seed|property:seed");
    try std.testing.expectEqualStrings("https://example.test:8443/seed", subject.mocksMut().location().currentUrl().?);
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().location().navigations().len);
    try std.testing.expectEqualStrings("https://example.test:8443/seed", subject.mocksMut().location().navigations()[0]);
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 28c3 focusin and focusout listeners resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><section id='panel'><input id='first'><input id='second'></section><script>const panel = document.getElementById('panel'); panel.addEventListener('focusin', (event) => { document.getElementById('out').textContent += 'focusin|'; }); panel.addEventListener('focusout', (event) => { document.getElementById('out').textContent += 'focusout|'; }); document.getElementById('first').addEventListener('focus', (event) => { document.getElementById('out').textContent += 'focus:first|'; }); document.getElementById('first').addEventListener('blur', (event) => { document.getElementById('out').textContent += 'blur:first|'; }); document.getElementById('second').addEventListener('focus', (event) => { document.getElementById('out').textContent += 'focus:second|'; }); document.getElementById('second').addEventListener('blur', (event) => { document.getElementById('out').textContent += 'blur:second'; });</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.focus("#first");
    try subject.focus("#second");
    try subject.assertValue(
        "#out",
        "focusin|focus:first|focusout|blur:first|focusin|focus:second|",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 28c4 window focus and blur listeners resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><section id='panel'><input id='first'><input id='second'></section><script>window.addEventListener('focus', () => { document.getElementById('out').textContent += 'focus|'; }); window.onfocus = () => { document.getElementById('out').textContent += 'property-focus|'; }; window.addEventListener('blur', () => { document.getElementById('out').textContent += 'blur|'; }); window.onblur = () => { document.getElementById('out').textContent += 'property-blur'; };</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.focus("#first");
    try subject.blur("#first");
    try subject.assertValue("#out", "focus|property-focus|blur|property-blur");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 28c5 window load listeners resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>window.addEventListener('load', () => { document.getElementById('out').textContent += 'load|'; }); window.onload = () => { document.getElementById('out').textContent += 'property-load|'; }; window.addEventListener('pageshow', () => { document.getElementById('out').textContent += 'pageshow|'; }); window.onpageshow = () => { document.getElementById('out').textContent += 'property-pageshow'; };</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "load|property-load|pageshow|property-pageshow");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 28c5b pagehide and pageshow listeners resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><button id='nav'>Go</button><script>window.addEventListener('pagehide', () => { document.getElementById('out').textContent += 'hide|'; }); window.onpagehide = () => { document.getElementById('out').textContent += 'property-hide|'; }; window.addEventListener('pageshow', () => { document.getElementById('out').textContent += 'show|'; }); window.onpageshow = () => { document.getElementById('out').textContent += 'property-show|'; }; document.getElementById('nav').addEventListener('click', () => { document.getElementById('out').textContent = ''; document.location = 'https://example.test:8443/next'; });</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.click("#nav");
    try subject.assertValue("#out", "hide|property-hide|show|property-show|");
    try std.testing.expectEqualStrings("https://example.test:8443/next", subject.mocksMut().location().currentUrl().?);
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 28c5c beforeunload, pagehide, unload, and pageshow listeners resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><button id='nav'>Go</button><script>window.addEventListener('beforeunload', () => { document.getElementById('out').textContent += 'before|'; }); window.onbeforeunload = () => { document.getElementById('out').textContent += 'property-before|'; }; window.addEventListener('pagehide', () => { document.getElementById('out').textContent += 'hide|'; }); window.onpagehide = () => { document.getElementById('out').textContent += 'property-hide|'; }; window.addEventListener('unload', () => { document.getElementById('out').textContent += 'unload|'; }); window.onunload = () => { document.getElementById('out').textContent += 'property-unload|'; }; window.addEventListener('pageshow', () => { document.getElementById('out').textContent += 'show|'; }); window.onpageshow = () => { document.getElementById('out').textContent += 'property-show|'; }; document.getElementById('nav').addEventListener('click', () => { document.getElementById('out').textContent = ''; document.location = 'https://example.test:8443/next'; });</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.click("#nav");
    try subject.assertValue("#out", "before|property-before|hide|property-hide|unload|property-unload|show|property-show|");
    try std.testing.expectEqualStrings("https://example.test:8443/next", subject.mocksMut().location().currentUrl().?);
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 28c6 storage listeners and onstorage handlers resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>window.addEventListener('storage', () => { document.getElementById('out').textContent += 'listener:' + window.localStorage.getItem('theme') + '|'; }); window.onstorage = () => { document.getElementById('out').textContent += 'property:' + window.localStorage.getItem('theme'); }; window.localStorage.setItem('theme', 'light');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var builder = Harness.builder(allocator);
    defer builder.deinit();
    _ = builder.html(html_bytes);
    try builder.addLocalStorage("theme", "dark");

    var subject = try builder.build();
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "listener:light|property:light");
    try std.testing.expectEqualStrings(original, subject.html().?);
    try std.testing.expectEqualStrings("light", subject.mocksMut().storage().local().get("theme").?);
}

test "regression: phase 28d Location component getters and setters resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const before = window.location.protocol + '|' + window.location.host + '|' + window.location.hostname + '|' + window.location.port; document.location.protocol = 'http:'; document.location.host = 'example.test:8080'; document.location.hostname = 'example.test'; document.location.port = '8080'; const after = window.location.protocol + '|' + window.location.host + '|' + window.location.hostname + '|' + window.location.port; document.getElementById('out').textContent = before + ':' + after + ':' + document.location;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://app.local:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "https:|app.local:8443|app.local|8443:http:|example.test:8080|example.test|8080:http://example.test:8080/start?x#old",
    );
    try std.testing.expectEqualStrings("http://example.test:8080/start?x#old", subject.mocksMut().location().currentUrl().?);
    try std.testing.expectEqual(@as(usize, 4), subject.mocksMut().location().navigations().len);
    try std.testing.expectEqualStrings("http://app.local:8443/start?x#old", subject.mocksMut().location().navigations()[0]);
    try std.testing.expectEqualStrings("http://example.test:8080/start?x#old", subject.mocksMut().location().navigations()[1]);
    try std.testing.expectEqualStrings("http://example.test:8080/start?x#old", subject.mocksMut().location().navigations()[2]);
    try std.testing.expectEqualStrings("http://example.test:8080/start?x#old", subject.mocksMut().location().navigations()[3]);
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 28e Location username password pathname and search resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const before = window.location.username + '|' + window.location.password + '|' + window.location.pathname + '|' + window.location.search; document.location.username = 'bob'; document.location.password = 'hunter2'; document.location.pathname = 'next'; document.location.search = '?copy'; const after = window.location.username + '|' + window.location.password + '|' + window.location.pathname + '|' + window.location.search; document.getElementById('out').textContent = before + ':' + after + ':' + document.location;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://alice:secret@app.local:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "alice|secret|/start|?x:bob|hunter2|/next|?copy:https://bob:hunter2@app.local:8443/next?copy#old",
    );
    try std.testing.expectEqualStrings("https://bob:hunter2@app.local:8443/next?copy#old", subject.mocksMut().location().currentUrl().?);
    try std.testing.expectEqual(@as(usize, 4), subject.mocksMut().location().navigations().len);
    try std.testing.expectEqualStrings("https://bob:secret@app.local:8443/start?x#old", subject.mocksMut().location().navigations()[0]);
    try std.testing.expectEqualStrings("https://bob:hunter2@app.local:8443/start?x#old", subject.mocksMut().location().navigations()[1]);
    try std.testing.expectEqualStrings("https://bob:hunter2@app.local:8443/next?x#old", subject.mocksMut().location().navigations()[2]);
    try std.testing.expectEqualStrings("https://bob:hunter2@app.local:8443/next?copy#old", subject.mocksMut().location().navigations()[3]);
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 29 window.history navigation methods resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const history = window.history; const beforeLength = history.length; history.replaceState(null, '', 'https://example.test:8443/replaced'); history.pushState(null, '', 'https://example.test:8443/pushed'); history.back(); history.forward(); history.go(-1); document.getElementById('out').textContent = String(beforeLength) + ':' + String(history.length) + ':' + String(history.state) + ':' + window.location.href;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://example.test:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:2:null:https://example.test:8443/replaced");
    try std.testing.expectEqual(@as(usize, 3), subject.mocksMut().location().navigations().len);
    try std.testing.expectEqualStrings("https://example.test:8443/replaced", subject.mocksMut().location().navigations()[0]);
    try std.testing.expectEqualStrings("https://example.test:8443/pushed", subject.mocksMut().location().navigations()[1]);
    try std.testing.expectEqualStrings("https://example.test:8443/replaced", subject.mocksMut().location().navigations()[2]);
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 30 history.state payloads resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const history = window.history; const before = String(history.state); history.replaceState('seed', '', 'https://example.test:8443/replaced'); const afterReplace = String(history.state); history.pushState('pushed', '', 'https://example.test:8443/pushed'); const afterPush = String(history.state); history.back(); const afterBack = String(history.state); document.getElementById('out').textContent = before + ':' + afterReplace + ':' + afterPush + ':' + afterBack + ':' + window.location.href;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://example.test:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "null:seed:pushed:seed:https://example.test:8443/replaced");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 31 window scroll aliases resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const before = String(window.scrollX) + ':' + String(window.scrollY) + ':' + String(window.pageXOffset) + ':' + String(window.pageYOffset); window.scrollTo(10, 20); window.scrollBy(-3, 5); const afterScroll = String(window.scrollX) + ':' + String(window.scrollY) + ':' + String(window.pageXOffset) + ':' + String(window.pageYOffset); window.location = 'https://example.test:8443/next'; const afterNavigation = String(window.scrollX) + ':' + String(window.scrollY) + ':' + String(window.pageXOffset) + ':' + String(window.pageYOffset); document.getElementById('out').textContent = before + '|' + afterScroll + '|' + afterNavigation;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://example.test:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "0:0:0:0|7:25:7:25|0:0:0:0");
    try std.testing.expectEqualStrings(
        "https://example.test:8443/next",
        subject.mocksMut().location().currentUrl().?,
    );
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().location().navigations().len);
    try std.testing.expectEqualStrings("https://example.test:8443/next", subject.mocksMut().location().navigations()[0]);
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 32 window.navigator aliases resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<embed id='first-embed'><embed name='second-embed'><main id='out'></main><script>const navigator = window.navigator; document.getElementById('out').textContent = String(navigator) + ':' + navigator.userAgent + ':' + navigator.appCodeName + ':' + navigator.appName + ':' + navigator.appVersion + ':' + navigator.product + ':' + navigator.productSub + ':' + navigator.vendor + ':[' + navigator.vendorSub + ']:' + String(navigator.pdfViewerEnabled) + ':' + navigator.doNotTrack + ':' + String(navigator.javaEnabled()) + ':' + String(navigator.plugins) + ':' + String(navigator.plugins.length) + ':' + navigator.platform + ':' + navigator.language + ':' + String(navigator.cookieEnabled) + ':' + String(navigator.onLine) + ':' + String(navigator.webdriver) + ':' + String(navigator.hardwareConcurrency) + ':' + String(navigator.maxTouchPoints);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://example.test:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[object Navigator]:browser_tester:browser_tester:browser_tester:browser_tester:browser_tester:browser_tester:browser_tester:[]:false:unspecified:false:[object PluginArray]:2:unknown:en-US:true:true:false:8:0");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 32b window.navigator.plugins.refresh resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<embed id='first-embed'><embed name='second-embed'><main id='out'></main><script>const plugins = window.navigator.plugins; document.getElementById('out').textContent = String(plugins.refresh()) + ':' + String(plugins.length) + ':' + String(plugins);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://example.test:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "undefined:2:[object PluginArray]");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 32c window.navigator.plugins collection helpers resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<embed id='first-embed' name='first-embed'><embed name='second-embed'><main id='out'></main><script>const plugins = window.navigator.plugins; const keys = plugins.keys(); const values = plugins.values(); const entries = plugins.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); const secondValue = values.next(); const secondEntry = entries.next(); const before = String(plugins.length) + ':' + String(firstKey.value) + ':' + firstValue.value.id + ':' + firstValue.value.getAttribute('name') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.id + ':' + firstEntry.value.value.getAttribute('name') + ':' + secondValue.value.getAttribute('name') + ':' + String(secondEntry.value.index) + ':' + secondEntry.value.value.getAttribute('name'); plugins.forEach(() => { document.getElementById('out').textContent = 'called'; }, null); document.getElementById('out').textContent = before + ':' + document.getElementById('out').textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://example.test:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "2:0:first-embed:first-embed:0:first-embed:first-embed:second-embed:1:second-embed:called",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 32c window.navigator.refresh resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const navigator = window.navigator; document.getElementById('out').textContent = String(navigator.refresh()) + ':' + String(navigator.plugins.refresh()) + ':' + String(navigator.plugins.length);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://example.test:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "undefined:undefined:0");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 41 window.navigator languages resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const languages = window.navigator.languages; const keys = languages.keys(); const values = languages.values(); const entries = languages.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); document.getElementById('out').textContent = window.navigator.userLanguage + ':' + window.navigator.browserLanguage + ':' + window.navigator.systemLanguage + ':' + window.navigator.oscpu + ':' + String(languages.length) + ':' + languages.item(0) + ':' + languages.toString() + ':' + String(languages.contains('en-US')) + ':' + String(languages.contains('fr-FR')) + ':' + String(firstKey.value) + ':' + firstValue.value + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value; languages.forEach(() => { document.getElementById('out').textContent = 'called'; }, null); document.getElementById('out').textContent = document.getElementById('out').textContent + ':' + 'called';</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://example.test:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "called:called");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 40 window.navigator.mimeTypes resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const mimeTypes = window.navigator.mimeTypes; const keys = mimeTypes.keys(); const values = mimeTypes.values(); const entries = mimeTypes.entries(); document.getElementById('out').textContent = 'before'; mimeTypes.forEach(() => { document.getElementById('out').textContent = 'called'; }, null); document.getElementById('out').textContent = String(mimeTypes) + ':' + mimeTypes.toString() + ':' + String(mimeTypes.length) + ':' + String(mimeTypes.item(0)) + ':' + String(mimeTypes.namedItem('text/plain')) + ':' + String(keys.next().done) + ':' + String(values.next().done) + ':' + String(entries.next().done) + ':' + document.getElementById('out').textContent;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://example.test:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[object MimeTypeArray]:[object MimeTypeArray]:0:null:null:true:true:true:before");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 38 window identity aliases resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const view = document.defaultView; document.getElementById('out').textContent = String(view) + ':' + String(window.window) + ':' + String(window.self) + ':' + String(window.top) + ':' + String(window.parent) + ':' + String(window.opener) + ':' + String(window.closed);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://example.test:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[object Window]:[object Window]:[object Window]:[object Window]:[object Window]:null:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 39 document.scrollingElement, window.frames, window.length, and history.scrollRestoration resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const history = window.history; const scrolling = document.scrollingElement; history.scrollRestoration = 'manual'; document.getElementById('out').textContent = String(scrolling) + ':' + String(window.frames) + ':' + String(window.length) + ':' + history.scrollRestoration;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://example.test:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[object Element]:[object Window]:0:manual");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 39b window.frameElement resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<html id='html'><head><title>Example</title></head><body id='body'><main id='out'></main><script>const metadata = document.compatMode + ':' + document.characterSet + ':' + document.charset + ':' + document.contentType; const active = document.activeElement.getAttribute('id'); const documentChildren = document.children; const windowChildren = window.children; document.getElementById('out').textContent = metadata + ':' + active + ':' + String(documentChildren.length) + ':' + String(windowChildren.length) + ':' + String(window.frameElement) + ':' + documentChildren.item(0).getAttribute('id') + ':' + windowChildren.item(0).getAttribute('id');</script></body></html>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "CSS1Compat:UTF-8:UTF-8:text/html:body:1:1:null:html:html");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 27 document.baseURI and element.baseURI aliases resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><span id='child'></span></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = document.baseURI + ':' + root.baseURI + ':' + child.baseURI;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://example.test:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "https://example.test:8443/start?x#old:https://example.test:8443/start?x#old:https://example.test:8443/start?x#old");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 27 document.domain and origin aliases resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><span id='child'></span></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = document.domain + ':' + document.origin + ':' + window.origin + ':' + root.origin + ':' + child.origin;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://example.test:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "example.test:https://example.test:8443:https://example.test:8443:https://example.test:8443:https://example.test:8443");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 27 location ancestorOrigins resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const location = window.location; const origins = location.ancestorOrigins; document.getElementById('out').textContent = String(origins) + ':' + String(origins.length) + ':' + String(origins.item(0));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://example.test:8443/start?x#old", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[object DOMStringList]:0:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 27 node and element reflection helpers resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<!--pre--><main id='root'><span id='child'></span><!--tail--></main><div id='out'></div><script>const doc = document; const root = document.getElementById('root'); const child = document.getElementById('child'); const comment = document.childNodes.item(0); document.getElementById('out').textContent = String(doc.ownerDocument) + ':' + String(doc.parentNode) + ':' + String(doc.parentElement) + ':' + String(doc.firstElementChild) + ':' + String(doc.lastElementChild) + ':' + String(doc.childElementCount) + ':' + String(root.ownerDocument) + ':' + String(root.parentNode) + ':' + String(root.parentElement) + ':' + String(root.firstElementChild) + ':' + String(root.lastElementChild) + ':' + String(root.childElementCount) + ':' + String(child.ownerDocument) + ':' + String(child.parentNode) + ':' + String(child.parentElement) + ':' + String(comment.ownerDocument) + ':' + String(comment.parentNode) + ':' + String(comment.parentElement);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "null:null:null:[object Element]:[object Element]:3:[object Document]:[object Document]:null:[object Element]:[object Element]:1:[object Document]:[object Element]:[object Element]:[object Document]:[object Document]:null",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 28 defined pseudo-class selectors resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><x-widget id='widget'></x-widget><svg id='svg'><text id='svg-text'>Hi</text></svg></main><div id='out'></div><script>const defined = document.querySelectorAll(':defined'); const widget = document.getElementById('widget'); const svg = document.getElementById('svg'); document.getElementById('out').textContent = defined.item(0).getAttribute('id') + ':' + defined.item(1).getAttribute('id') + ':' + defined.item(2).getAttribute('id') + ':' + String(widget.matches(':defined')) + ':' + String(svg.matches(':defined'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "root:svg:svg-text:false:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 29 currentScript and readyState resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><button id='button'></button><script id='first'>document.getElementById('out').textContent = document.currentScript.getAttribute('id') + ':' + document.readyState; document.getElementById('button').addEventListener('click', () => { document.getElementById('out').textContent += ':' + String(document.currentScript) + ':' + document.readyState; });</script><script id='second'>document.getElementById('out').textContent += ':' + document.currentScript.getAttribute('id') + ':' + document.readyState;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "first:loading:second:loading");
    try subject.click("#button");
    try subject.assertValue("#out", "first:loading:second:loading:null:complete");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 32 document.activeElement resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<html id='html'><body id='body'><input id='field'><div id='out'></div><script>document.getElementById('field').addEventListener('focus', () => { document.getElementById('out').textContent = document.activeElement.getAttribute('id'); });</script></body></html>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.focus("#field");
    try subject.assertValue("#out", "field");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 33 document.referrer and dir resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<html id='html' dir='ltr'><body id='body'><main id='out'></main><script>const referrer = '[' + document.referrer + ']'; const before = document.dir; document.dir = 'rtl'; document.getElementById('out').textContent = referrer + ':' + before + ':' + document.dir + ':' + document.documentElement.getAttribute('dir');</script></body></html>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[]:ltr:rtl:rtl");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 34 window.name resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const before = window.name; window.name = 'updated'; document.getElementById('out').textContent = before + ':' + document.defaultView.name;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":updated");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 35 document.cookie resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>document.cookie = 'theme=dark'; document.cookie = 'theme=light'; document.getElementById('out').textContent = document.cookie;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "theme=light");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 36 viewport and visibility aliases resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>document.getElementById('out').textContent = String(window.devicePixelRatio) + ':' + String(window.innerWidth) + ':' + String(window.innerHeight) + ':' + String(window.outerWidth) + ':' + String(window.outerHeight) + ':' + document.visibilityState + ':' + String(document.hidden) + ':' + String(document.hasFocus());</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:1024:768:1280:800:visible:false:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 37 screen aliases resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>document.getElementById('out').textContent = String(window.screenX) + ':' + String(window.screenY) + ':' + String(window.screenLeft) + ':' + String(window.screenTop);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "0:0:0:0");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 37 screen orientation aliases resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const orientation = window.screen.orientation; document.getElementById('out').textContent = orientation.type + ':' + String(orientation.angle) + ':' + String(orientation);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "landscape-primary:0:[object ScreenOrientation]");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 40 Math constants and Math.random resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>document.getElementById('out').textContent = String(Math) + ':' + String(window.Math) + ':' + String(Math.PI) + ':' + String(window.Math.PI) + ':' + String(Math.random()) + ':' + String(window.Math.random());</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[object Math]:[object Math]:3.141592653589793:3.141592653589793:0.114:0.363");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 40 Math.random respects HarnessBuilder.randomSeed on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>document.getElementById('out').textContent = String(Math.random()) + ':' + String(window.Math.random());</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var builder = Harness.builder(allocator);
    defer builder.deinit();
    _ = builder.randomSeed(0);
    _ = builder.html(html_bytes);

    var subject = try builder.build();
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "0.041:0.034");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 40 crypto.randomUUID respects HarnessBuilder.randomSeed on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>document.getElementById('out').textContent = String(window.crypto) + ':' + window.crypto.randomUUID();</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var builder = Harness.builder(allocator);
    defer builder.deinit();
    _ = builder.randomSeed(0);
    _ = builder.html(html_bytes);

    var subject = try builder.build();
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[object Crypto]:29da53d4-9dee-4728-9182-3bfc0596ef50");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 38 performance.now resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const performance = window.performance; document.getElementById('out').textContent = String(performance) + ':' + String(window.performance) + ':' + String(performance.timeOrigin) + ':' + String(performance.now()); window.setTimeout(() => { document.getElementById('out').textContent = document.getElementById('out').textContent + ':' + String(performance.now()) + ':' + String(window.performance.now()); }, 5);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[object Performance]:[object Performance]:0:0");
    try subject.advanceTime(5);
    try subject.assertValue("#out", "[object Performance]:[object Performance]:0:0:5:5");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 39 anchor click default actions navigate and capture downloads on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><a id='nav' href='https://example.test/next'>Go</a><a id='download' download='report.csv' href='https://example.test/files/report.csv'>Download</a><div id='out'></div><script>document.getElementById('nav').addEventListener('click', () => { document.getElementById('out').textContent = 'clicked'; });</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://app.local/start", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.click("#nav");
    try subject.assertValue("#out", "clicked");
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
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 50 anchor and area download reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><a id='anchor' href='https://example.test/files/report.csv'>Anchor</a><map name='map'><area id='area' download='area.bin' href='https://example.test/files/diagram.png'></map><div id='out'></div><script>const anchor = document.getElementById('anchor'); const area = document.querySelector('#area'); const before = String(anchor.download) + ':' + String(area.download); anchor.download = 'anchor.txt'; area.download = 'area-updated.bin'; document.getElementById('out').textContent = before + '|' + anchor.download + ':' + area.download;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":area.bin|anchor.txt:area-updated.bin");
    try subject.click("#anchor");
    try subject.click("#area");
    try std.testing.expectEqual(@as(usize, 2), subject.mocksMut().downloads().artifacts().len);
    try std.testing.expectEqualStrings(
        "anchor.txt",
        subject.mocksMut().downloads().artifacts()[0].file_name,
    );
    try std.testing.expectEqualStrings(
        "https://example.test/files/report.csv",
        subject.mocksMut().downloads().artifacts()[0].bytes,
    );
    try std.testing.expectEqualStrings(
        "area-updated.bin",
        subject.mocksMut().downloads().artifacts()[1].file_name,
    );
    try std.testing.expectEqualStrings(
        "https://example.test/files/diagram.png",
        subject.mocksMut().downloads().artifacts()[1].bytes,
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 50 anchor target reflection and area target click observation resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><a id='anchor' href='https://example.test/next' target='_blank'>Anchor</a><map name='map'><area id='area' target='popup' href='https://example.test/files/diagram.png'></map><div id='out'></div><script>const anchor = document.getElementById('anchor'); const area = document.querySelector('#area'); const before = String(anchor.target) + ':' + String(area.target); anchor.target = 'reports'; area.target = 'diagram'; document.getElementById('out').textContent = before + '|' + anchor.target + ':' + area.target;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtmlWithUrl(allocator, "https://app.local/start", html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "_blank:popup|reports:diagram");
    try subject.click("#anchor");
    try subject.click("#area");
    try std.testing.expectEqual(@as(usize, 2), subject.mocksMut().open().calls().len);
    try std.testing.expectEqualStrings(
        "https://example.test/next",
        subject.mocksMut().open().calls()[0].url.?,
    );
    try std.testing.expectEqualStrings(
        "reports",
        subject.mocksMut().open().calls()[0].target.?,
    );
    try std.testing.expectEqualStrings(
        "https://example.test/files/diagram.png",
        subject.mocksMut().open().calls()[1].url.?,
    );
    try std.testing.expectEqualStrings(
        "diagram",
        subject.mocksMut().open().calls()[1].target.?,
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 50 anchor and area href reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><a id='anchor' href='https://example.test/next'>Anchor</a><map name='map'><area id='area' href='https://example.test/files/diagram.png'></map><div id='out'></div><script>const anchor = document.getElementById('anchor'); const area = document.querySelector('#area'); const before = String(anchor.href) + ':' + String(area.href); anchor.href = 'https://example.test/other'; area.href = 'https://example.test/files/diagram-2.png'; document.getElementById('out').textContent = before + '|' + anchor.href + ':' + area.href;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

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
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 50 anchor and area rel reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><a id='anchor' rel='noopener noreferrer noopener' href='https://example.test/next'>Anchor</a><map name='map'><area id='area' rel='preload preload' href='https://example.test/files/diagram.png'></map><div id='out'></div><script>const anchor = document.getElementById('anchor'); const area = document.querySelector('#area'); const before = String(anchor.rel) + ':' + String(anchor.relList.length) + ':' + String(anchor.relList.contains('noopener')) + ':' + String(anchor.relList.supports('noopener')) + ':' + String(anchor.relList.supports('bogus')) + ':' + String(area.rel) + ':' + String(area.relList.length) + ':' + String(area.relList.contains('preload')) + ':' + String(area.relList.supports('preload')); anchor.relList.add('preload'); anchor.relList.remove('noopener'); anchor.relList.toggle('modulepreload'); anchor.relList.replace('noreferrer', 'preconnect'); area.relList.add('preconnect'); area.relList.remove('preload'); area.relList.toggle('modulepreload'); area.relList.replace('preconnect', 'noreferrer'); document.getElementById('out').textContent = before + '|' + anchor.rel + ':' + anchor.relList.value + ':' + area.rel + ':' + area.relList.value;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "noopener noreferrer noopener:2:true:true:false:preload preload:1:true:true|preconnect preload modulepreload:preconnect preload modulepreload:noreferrer modulepreload:noreferrer modulepreload",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 50b form rel reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='form' rel='noopener noreferrer opener noopener'></form><div id='out'></div><script>const form = document.getElementById('form'); const before = form.rel + ':' + String(form.relList.length) + ':' + String(form.relList.contains('noopener')) + ':' + String(form.relList.supports('noopener')) + ':' + String(form.relList.supports('opener')) + ':' + String(form.relList.supports('stylesheet')); form.rel = 'opener'; const during = form.rel + ':' + form.relList.value + ':' + form.getAttribute('rel') + ':' + String(form.relList.supports('noopener')) + ':' + String(form.relList.supports('stylesheet')); form.relList.value = 'noreferrer noopener'; document.getElementById('out').textContent = before + '|' + during + '|' + form.rel + ':' + form.relList.value + ':' + form.getAttribute('rel') + ':' + String(form.relList.supports('noopener')) + ':' + String(form.relList.supports('stylesheet'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "noopener noreferrer opener noopener:3:true:true:true:false|opener:opener:opener:true:false|noreferrer noopener:noreferrer noopener:noreferrer noopener:true:false",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 50c form relList add remove toggle and replace resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='form' rel='noopener noreferrer opener noopener'></form><div id='out'></div><script>const form = document.getElementById('form'); const before = form.rel + ':' + form.relList.value + ':' + String(form.relList.length) + ':' + String(form.relList.contains('noopener')) + ':' + String(form.relList.supports('noopener')) + ':' + String(form.relList.supports('stylesheet')); form.relList.add('preload'); form.relList.replace('preload', 'modulepreload'); form.relList.remove('noopener'); form.relList.toggle('preconnect'); document.getElementById('out').textContent = before + '|' + form.rel + ':' + form.relList.value + ':' + form.getAttribute('rel') + ':' + String(form.relList.length) + ':' + String(form.relList.contains('modulepreload')) + ':' + String(form.relList.contains('preconnect')) + ':' + String(form.relList.supports('noopener')) + ':' + String(form.relList.supports('stylesheet'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "noopener noreferrer opener noopener:noopener noreferrer opener:3:true:true:false|noreferrer opener modulepreload preconnect:noreferrer opener modulepreload preconnect:noreferrer opener modulepreload preconnect:4:true:true:true:false",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 50c1 HTMLFormElement rel reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='form' rel='noopener noreferrer opener noopener'></form><div id='out'></div><script>const form = document.getElementById('form'); const before = form.rel + ':' + String(form.relList.length) + ':' + String(form.relList.contains('noopener')) + ':' + String(form.relList.supports('noopener')) + ':' + String(form.relList.supports('opener')) + ':' + String(form.relList.supports('stylesheet')); form.rel = 'opener'; const during = form.rel + ':' + form.relList.value + ':' + form.getAttribute('rel') + ':' + String(form.relList.supports('noopener')) + ':' + String(form.relList.supports('stylesheet')); form.relList.value = 'noreferrer noopener'; const after = form.rel + ':' + form.relList.value + ':' + form.getAttribute('rel') + ':' + String(form.relList.supports('noopener')) + ':' + String(form.relList.supports('stylesheet')); document.getElementById('out').textContent = before + '|' + during + '|' + after;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "noopener noreferrer opener noopener:3:true:true:true:false|opener:opener:opener:true:false|noreferrer noopener:noreferrer noopener:noreferrer noopener:true:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 50d form name reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='form' name='signup'></form><div id='out'></div><script>const form = document.getElementById('form'); const before = form.name + ':' + document.forms.namedItem('signup').id; form.name = 'profile'; document.getElementById('out').textContent = before + '|' + form.name + ':' + form.getAttribute('name') + ':' + document.forms.namedItem('profile').id + ':' + String(document.forms.namedItem('signup'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "signup:form|profile:profile:form:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 50 anchor and area ping reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><a id='anchor' ping='https://example.test/ping-a https://example.test/ping-b' href='https://example.test/next'>Anchor</a><map name='map'><area id='area' ping='https://example.test/ping-c' href='https://example.test/files/diagram.png'></map><div id='out'></div><script>const anchor = document.getElementById('anchor'); const area = document.querySelector('#area'); const before = String(anchor.ping) + ':' + String(area.ping); anchor.ping = 'https://example.test/ping-d'; area.ping = 'https://example.test/ping-e https://example.test/ping-f'; document.getElementById('out').textContent = before + '|' + anchor.ping + ':' + area.ping + ':' + anchor.getAttribute('ping') + ':' + area.getAttribute('ping');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "https://example.test/ping-a https://example.test/ping-b:https://example.test/ping-c|https://example.test/ping-d:https://example.test/ping-e https://example.test/ping-f:https://example.test/ping-d:https://example.test/ping-e https://example.test/ping-f",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 50a anchor and area hreflang reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><a id='anchor' hreflang='en-GB' href='https://example.test/next'>Anchor</a><map name='map'><area id='area' hreflang='fr' href='https://example.test/files/diagram.png'></map><div id='out'></div><script>const anchor = document.getElementById('anchor'); const area = document.querySelector('#area'); const before = anchor.hreflang + ':' + area.hreflang; anchor.hreflang = 'de'; area.hreflang = 'ja'; document.getElementById('out').textContent = before + '|' + anchor.hreflang + ':' + area.hreflang + ':' + anchor.getAttribute('hreflang') + ':' + area.getAttribute('hreflang');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "en-GB:fr|de:ja:de:ja");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 50b anchor and area referrerPolicy reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><a id='anchor' referrerpolicy='no-referrer' href='https://example.test/next'>Anchor</a><map name='map'><area id='area' referrerpolicy='origin' href='https://example.test/files/diagram.png'></map><div id='out'></div><script>const anchor = document.getElementById('anchor'); const area = document.querySelector('#area'); const before = anchor.referrerPolicy + ':' + area.referrerPolicy; anchor.referrerPolicy = 'same-origin'; area.referrerPolicy = 'no-referrer'; document.getElementById('out').textContent = before + '|' + anchor.referrerPolicy + ':' + area.referrerPolicy + ':' + anchor.getAttribute('referrerpolicy') + ':' + area.getAttribute('referrerpolicy');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "no-referrer:origin|same-origin:no-referrer:same-origin:no-referrer");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 50 anchor and area URL decomposition resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><a id='anchor' href='https://user:pass@example.test:8443/next/path?x=1#frag'>Anchor</a><map name='map'><area id='area' href='https://user:pass@example.test:8443/files/diagram.png?y=2#section'></map><link id='link' href='https://user:pass@example.test:8443/assets/style.css?z=3#sheet'><div id='out'></div><script>const anchor = document.getElementById('anchor'); const area = document.querySelector('#area'); const link = document.getElementById('link'); document.getElementById('out').textContent = String(anchor.origin) + '|' + anchor.protocol + '|' + anchor.host + '|' + anchor.hostname + '|' + anchor.port + '|' + anchor.username + '|' + anchor.password + '|' + anchor.pathname + '|' + anchor.search + '|' + anchor.hash + '||' + String(area.origin) + '|' + area.protocol + '|' + area.host + '|' + area.hostname + '|' + area.port + '|' + area.username + '|' + area.password + '|' + area.pathname + '|' + area.search + '|' + area.hash + '||' + String(link.origin) + '|' + link.protocol + '|' + link.host + '|' + link.hostname + '|' + link.port + '|' + link.username + '|' + link.password + '|' + link.pathname + '|' + link.search + '|' + link.hash;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "https://example.test:8443|https:|example.test:8443|example.test|8443|user|pass|/next/path|?x=1|#frag||https://example.test:8443|https:|example.test:8443|example.test|8443|user|pass|/files/diagram.png|?y=2|#section||https://example.test:8443|https:|example.test:8443|example.test|8443|user|pass|/assets/style.css|?z=3|#sheet",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 50 anchor text reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><a id='anchor' href='https://example.test/next'>Anchor</a><div id='out'></div><script>const anchor = document.getElementById('anchor'); const before = anchor.text + ':' + anchor.textContent; anchor.text = 'Updated'; document.getElementById('out').textContent = before + '|' + anchor.text + ':' + anchor.textContent + ':' + anchor.innerHTML + ':' + anchor.outerHTML;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "Anchor:Anchor|Updated:Updated:Updated:<a href=\"https://example.test/next\" id=\"anchor\">Updated</a>",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 50 area alt coords and shape reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><map name='map'><area id='area' alt='Site map entry' coords='0,0,10,10' shape='rect' href='https://example.test/files/diagram.png'></map><div id='out'></div><script>const area = document.querySelector('#area'); const before = String(area.alt) + ':' + String(area.coords) + ':' + String(area.shape); area.alt = 'Updated'; area.coords = '1,2,3,4'; area.shape = 'circle'; document.getElementById('out').textContent = before + '|' + area.alt + ':' + area.coords + ':' + area.shape + ':' + area.getAttribute('alt') + ':' + area.getAttribute('coords') + ':' + area.getAttribute('shape');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "Site map entry:0,0,10,10:rect|Updated:1,2,3,4:circle:Updated:1,2,3,4:circle",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 50 area noHref reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><map name='map'><area id='area' nohref href='https://example.test/files/diagram.png'></map><div id='out'></div><script>const area = document.querySelector('#area'); const before = String(area.noHref) + ':' + String(area.getAttribute('nohref')); area.noHref = false; const during = String(area.noHref) + ':' + String(area.getAttribute('nohref')); area.noHref = true; document.getElementById('out').textContent = before + '|' + during + '|' + String(area.noHref) + ':' + String(area.getAttribute('nohref'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:|false:null|true:");
    try subject.click("#area");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 51a blocking reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><style id='style' blocking='render'></style><link id='link' rel='stylesheet' blocking='render' href='a.css'><div id='out'></div><script>const style = document.getElementById('style'); const link = document.getElementById('link'); const script = document.createElement('script'); const before = style.blocking.value + ':' + link.blocking.value + ':' + script.blocking.value + ':' + String(style.blocking.length) + ':' + String(link.blocking.contains('render')) + ':' + String(script.blocking.supports('render')) + ':' + String(script.blocking.supports('layout')); script.blocking.add('render'); style.blocking.remove('render'); link.blocking.remove('render'); script.blocking.remove('render'); document.getElementById('out').textContent = before + '|' + style.blocking.value + ':' + link.blocking.value + ':' + script.blocking.value + ':' + String(style.getAttribute('blocking')) + ':' + String(link.getAttribute('blocking')) + ':' + String(script.getAttribute('blocking')) + ':' + String(style.blocking.length) + ':' + String(link.blocking.length) + ':' + String(script.blocking.length);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "render:render::1:true:true:false|:::null:null:null:0:0:0",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 51a1 HTMLLinkElement blocking resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><link id='link' rel='stylesheet' blocking='render' href='a.css'><div id='out'></div><script>const link = document.getElementById('link'); const before = link.blocking.value + ':' + String(link.blocking.length) + ':' + String(link.blocking.contains('render')); link.blocking.add('render'); link.blocking.remove('render'); document.getElementById('out').textContent = before + '|' + link.blocking.value + ':' + String(link.blocking.length) + ':' + String(link.blocking.contains('render')) + ':' + link.getAttribute('blocking');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "render:1:true|:0:false:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 51b HTMLButtonElement type reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='submit'></button><button id='reset' type='reset'></button><button id='plain' type='button'></button><div id='out'></div><script>const submit = document.getElementById('submit'); const reset = document.getElementById('reset'); const plain = document.getElementById('plain'); const before = submit.type + ':' + reset.type + ':' + plain.type; submit.type = 'reset'; reset.type = 'submit'; plain.type = 'submit'; document.getElementById('out').textContent = before + '|' + submit.type + ':' + submit.getAttribute('type') + ':' + reset.type + ':' + reset.getAttribute('type') + ':' + plain.type + ':' + plain.getAttribute('type');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "submit:reset:button|reset:reset:submit:submit:submit:submit");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 51c HTMLButtonElement value reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='submit'>Save</button><button id='plain' type='button'>Plain</button><div id='out'></div><script>const submit = document.getElementById('submit'); const plain = document.getElementById('plain'); const before = String(submit.value) + ':' + submit.textContent + ':' + String(plain.value) + ':' + plain.textContent; submit.value = 'go'; plain.value = 'noop'; document.getElementById('out').textContent = before + '|' + submit.value + ':' + submit.getAttribute('value') + ':' + submit.textContent + ':' + plain.value + ':' + plain.getAttribute('value') + ':' + plain.textContent;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":Save::Plain|go:go:Save:noop:noop:Plain");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 51d HTMLButtonElement name reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='action' name='save'>Save</button><div id='out'></div><script>const button = document.getElementById('action'); const before = button.name + ':' + button.getAttribute('name'); button.name = 'submit'; document.getElementById('out').textContent = before + '|' + button.name + ':' + button.getAttribute('name');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "save:save|submit:submit");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 51e HTMLInputElement name reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='input' name='first' value='Ada'><div id='out'></div><script>const input = document.getElementById('input'); const before = input.name + ':' + input.getAttribute('name') + ':' + String(document.getElementsByName('first').length); input.name = 'second'; document.getElementById('out').textContent = before + '|' + input.name + ':' + input.getAttribute('name') + ':' + String(document.getElementsByName('first').length) + ':' + String(document.getElementsByName('second').length);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "first:first:1|second:second:0:1");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 51 HTMLDataElement value and HTMLTimeElement dateTime resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><data id='datum' value='UPC:022014640201'>North Coast Organic Apple Cider</data><time id='stamp'>2011-11-18</time><div id='out'></div><script>const datum = document.getElementById('datum'); const stamp = document.getElementById('stamp'); const before = datum.value + ':' + stamp.dateTime + ':' + String(stamp.getAttribute('datetime')); datum.value = 'UPC:022014640202'; stamp.dateTime = '2012-05-30T10:00'; document.getElementById('out').textContent = before + '|' + datum.value + ':' + stamp.dateTime + ':' + datum.getAttribute('value') + ':' + stamp.getAttribute('datetime') + ':' + stamp.textContent;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "UPC:022014640201:2011-11-18:null|UPC:022014640202:2012-05-30T10:00:UPC:022014640202:2012-05-30T10:00:2011-11-18",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 51a HTMLTimeElement dateTime resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><time id='stamp'>2011-11-18</time><div id='out'></div><script>const stamp = document.getElementById('stamp'); const before = stamp.dateTime + ':' + String(stamp.hasAttribute('datetime')); stamp.dateTime = '2012-05-30T10:00'; document.getElementById('out').textContent = before + '|' + stamp.dateTime + ':' + stamp.getAttribute('datetime') + ':' + stamp.textContent;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2011-11-18:false|2012-05-30T10:00:2012-05-30T10:00:2011-11-18");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 51b HTMLDataElement value resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><data id='datum' value='UPC:022014640201'>North Coast Organic Apple Cider</data><div id='out'></div><script>const datum = document.getElementById('datum'); const before = datum.value; datum.value = 'UPC:022014640202'; document.getElementById('out').textContent = before + '|' + datum.value + ':' + datum.getAttribute('value') + ':' + datum.textContent;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "UPC:022014640201|UPC:022014640202:UPC:022014640202:North Coast Organic Apple Cider");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 51d HTMLQuoteElement cite and HTMLModElement reflection resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><blockquote id='quote' cite='https://example.test/original'>Quote</blockquote><q id='short' cite='https://example.test/nested'>Quote</q><ins id='inserted' cite='https://example.test/inserted' datetime='2024-01-01T12:00'>Inserted</ins><del id='removed' cite='https://example.test/removed' datetime='2024-01-02T15:30'>Old</del><div id='out'></div><script>const quote = document.getElementById('quote'); const short = document.getElementById('short'); const inserted = document.getElementById('inserted'); const removed = document.getElementById('removed'); const before = quote.cite + ':' + short.cite + ':' + inserted.cite + ':' + inserted.dateTime + ':' + removed.cite + ':' + removed.dateTime; quote.cite = 'https://example.test/updated'; short.cite = 'https://example.test/revised'; inserted.cite = 'https://example.test/next'; inserted.dateTime = '2024-02-03T04:05'; removed.cite = 'https://example.test/old'; removed.dateTime = '2024-02-06T07:08'; document.getElementById('out').textContent = before + '|' + quote.cite + ':' + short.cite + ':' + inserted.cite + ':' + inserted.dateTime + ':' + removed.cite + ':' + removed.dateTime + ':' + quote.getAttribute('cite') + ':' + short.getAttribute('cite') + ':' + inserted.getAttribute('cite') + ':' + inserted.getAttribute('datetime') + ':' + removed.getAttribute('cite') + ':' + removed.getAttribute('datetime');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "https://example.test/original:https://example.test/nested:https://example.test/inserted:2024-01-01T12:00:https://example.test/removed:2024-01-02T15:30|https://example.test/updated:https://example.test/revised:https://example.test/next:2024-02-03T04:05:https://example.test/old:2024-02-06T07:08:https://example.test/updated:https://example.test/revised:https://example.test/next:2024-02-03T04:05:https://example.test/old:2024-02-06T07:08",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 40 typeText updates textarea selection on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><textarea id='bio'>Hello</textarea><div id='out'></div><script>document.getElementById('bio').addEventListener('input', () => { const bio = document.getElementById('bio'); document.getElementById('out').textContent = String(bio.selectionStart) + ':' + String(bio.selectionEnd) + ':' + bio.selectionDirection; });</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.typeText("#bio", "World");
    try subject.assertValue("#bio", "World");
    try subject.assertValue("#out", "5:5:none");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 41 setRangeText updates selection on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.setSelectionRange(4, 12); name.setRangeText('Byron', 4, 12, 'select'); document.getElementById('out').textContent = name.value + ':' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "Ada Byron:4:9:none");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 41a HTMLInputElement and HTMLTextAreaElement setRangeText resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada Lovelace'><textarea id='bio'>Ada Lovelace</textarea><div id='out'></div><script>const name = document.getElementById('name'); const bio = document.getElementById('bio'); name.setSelectionRange(4, 12); bio.setSelectionRange(4, 12); name.setRangeText('Byron', 4, 12, 'select'); bio.setRangeText('Byron', 4, 12, 'select'); document.getElementById('out').textContent = name.value + ':' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + '|' + bio.value + ':' + String(bio.selectionStart) + ':' + String(bio.selectionEnd) + ':' + bio.selectionDirection;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "Ada Byron:4:9:none|Ada Byron:4:9:none");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 42 stepUp and stepDown resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='count' type='number' min='2' max='6' step='2' value='2'><div id='out'></div><script>const count = document.getElementById('count'); const before = count.value + ':' + String(count.validity.stepMismatch); count.stepUp(); const afterUp = count.value + ':' + String(count.validity.stepMismatch); count.stepDown(2); const afterDown = count.value + ':' + String(count.validity.stepMismatch); document.getElementById('out').textContent = before + '|' + afterUp + '|' + afterDown;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2:false|4:false|2:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 43 slot resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const box = document.createElement('div'); const before = box.slot; box.slot = 'primary'; document.getElementById('out').textContent = before + ':' + box.slot + ':' + box.getAttribute('slot');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":primary:primary");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 44 part resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const box = document.createElement('div'); const part = box.part; part.add('primary'); part.add('secondary'); part.remove('secondary'); const replaced = part.replace('primary', 'accent'); const missing = part.replace('missing', 'other'); document.getElementById('out').textContent = String(part.length) + ':' + box.getAttribute('part') + ':' + String(part.contains('accent')) + ':' + String(replaced) + ':' + String(missing) + ':' + String(part);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:accent:true:true:false:[object DOMTokenList]");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 45 input.valueAsNumber resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='num' type='number' value='42.5'><input id='date' type='date' value='2017-06-01'><input id='dt' type='datetime-local' value='2017-06-01T08:30'><input id='time' type='time' value='15:30:05.006'><input id='range' type='range' min='2' max='10' step='2' value='9'><input id='text' type='text' value='5'><div id='out'></div><script>const num = document.getElementById('num'); const date = document.getElementById('date'); const dt = document.getElementById('dt'); const time = document.getElementById('time'); const range = document.getElementById('range'); const text = document.getElementById('text'); const notANumber = text.valueAsNumber; const first = num.valueAsNumber + ':' + date.valueAsNumber + ':' + dt.valueAsNumber + ':' + time.valueAsNumber + ':' + range.valueAsNumber + ':' + String(notANumber); num.valueAsNumber = 10; date.valueAsNumber = 1496275200000; dt.valueAsNumber = 1496305805006; time.valueAsNumber = 32405006; range.valueAsNumber = 9; const second = num.value + ':' + num.valueAsNumber + '|' + date.value + ':' + date.valueAsNumber + '|' + dt.value + ':' + dt.valueAsNumber + '|' + time.value + ':' + time.valueAsNumber + '|' + range.value + ':' + range.valueAsNumber; num.valueAsNumber = notANumber; date.valueAsNumber = notANumber; dt.valueAsNumber = notANumber; time.valueAsNumber = notANumber; range.valueAsNumber = notANumber; const third = '[' + num.value + ']:' + String(num.valueAsNumber) + '|[' + date.value + ']:' + String(date.valueAsNumber) + '|[' + dt.value + ']:' + String(dt.valueAsNumber) + '|[' + time.value + ']:' + String(time.valueAsNumber) + '|' + range.value + ':' + range.valueAsNumber; document.getElementById('out').textContent = first + '|' + second + '|' + third;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "42.5:1496275200000:1496305800000:55805006:10:NaN|10:10|2017-06-01:1496275200000|2017-06-01T08:30:05.006:1496305805006|09:00:05.006:32405006|10:10|[]:NaN|[]:NaN|[]:NaN|[]:NaN|6:6",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46 input.valueAsDate resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='date' type='date' value='2017-06-01'><input id='dt' type='datetime-local' value='2017-06-01T08:30:05.006'><input id='time' type='time' value='09:00:05.006'><input id='month' type='month' value='2017-06'><input id='text' type='text' value='ignored'><div id='out'></div><script>const date = document.getElementById('date'); const dt = document.getElementById('dt'); const time = document.getElementById('time'); const month = document.getElementById('month'); const text = document.getElementById('text'); const dateObj = new Date(1496275200000); const dtObj = new Date(1496305805006); const timeObj = new Date(32405006); const first = date.valueAsDate.toISOString() + ':' + String(date.valueAsDate.valueOf()) + '|' + dt.valueAsDate.toISOString() + ':' + String(dt.valueAsDate.valueOf()) + '|' + time.valueAsDate.toISOString() + ':' + String(time.valueAsDate.valueOf()) + '|' + month.valueAsDate.toISOString() + ':' + String(month.valueAsDate.valueOf()) + '|' + String(text.valueAsDate); date.valueAsDate = dateObj; dt.valueAsDate = dtObj; time.valueAsDate = timeObj; month.valueAsDate = dateObj; const second = date.value + ':' + date.valueAsDate.toISOString() + '|' + dt.value + ':' + dt.valueAsDate.toISOString() + '|' + time.value + ':' + time.valueAsDate.toISOString() + '|' + month.value + ':' + month.valueAsDate.toISOString(); date.valueAsDate = null; dt.valueAsDate = null; time.valueAsDate = null; month.valueAsDate = null; const third = '[' + date.value + ']:' + String(date.valueAsDate) + '|[' + dt.value + ']:' + String(dt.valueAsDate) + '|[' + time.value + ']:' + String(time.valueAsDate) + '|[' + month.value + ']:' + String(month.valueAsDate); document.getElementById('out').textContent = first + '|' + second + '|' + third;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "2017-06-01T00:00:00.000Z:1496275200000|2017-06-01T08:30:05.006Z:1496305805006|1970-01-01T09:00:05.006Z:32405006|2017-06-01T00:00:00.000Z:1496275200000|null|2017-06-01:2017-06-01T00:00:00.000Z|2017-06-01T08:30:05.006:2017-06-01T08:30:05.006Z|09:00:05.006:1970-01-01T09:00:05.006Z|2017-06:2017-06-01T00:00:00.000Z|[]:null|[]:null|[]:null|[]:null",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46b input.list resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='search' list='suggestions'><input id='orphan' list='missing'><datalist id='suggestions'><option id='first' value='alpha'>Alpha</option><option value='beta'>Beta</option></datalist><div id='out'></div><script>const search = document.getElementById('search'); const orphan = document.getElementById('orphan'); const list = search.list; document.getElementById('out').textContent = list.id + ':' + String(list.options.length) + ':' + list.options.item(0).value + ':' + String(orphan.list);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "suggestions:2:alpha:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46c showPicker resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='date' type='date' value='2017-06-01'><select id='mode'><option value='a'>A</option><option value='b' selected>B</option></select><div id='out'></div><script>const date = document.getElementById('date'); const mode = document.getElementById('mode'); document.getElementById('out').textContent = String(date.showPicker()) + ':' + String(mode.showPicker()) + ':' + date.value + ':' + String(mode.selectedIndex);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "undefined:undefined:2017-06-01:1");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46c2 popover controls resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><article id='popover' popover='manual'></article><button id='trigger' popovertarget='popover' popovertargetaction='show'></button><div id='out'></div><script>const popover = document.getElementById('popover'); const trigger = document.getElementById('trigger'); popover.showPopover(); trigger.click(); trigger.setAttribute('popovertargetaction', 'hide'); trigger.click(); document.getElementById('out').textContent = popover.popover + ':' + String(popover.matches(':popover-open')) + ':' + String(document.querySelectorAll(':popover-open').length) + ':' + trigger.getAttribute('popovertarget') + ':' + trigger.getAttribute('popovertargetaction');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "manual:false:0:popover:hide");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46c3 command show-modal resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><dialog id='dlg'></dialog><button id='open' type='button'>Open</button><div id='out'></div><script>const dialog = document.getElementById('dlg'); const open = document.getElementById('open'); open.command = 'show-modal'; open.commandForElement = dialog; dialog.addEventListener('command', (event) => { document.getElementById('out').textContent = event.command; });</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.click("#open");
    try subject.assertValue("#out", "show-modal");
    try subject.assertValue("#dlg", "");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46c4 command request-close cancellation resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><dialog id='dlg'></dialog><button id='close' type='button'>Close</button><div id='out'></div><script>const dialog = document.getElementById('dlg'); const close = document.getElementById('close'); close.command = 'request-close'; close.commandForElement = dialog; dialog.addEventListener('command', (event) => { document.getElementById('out').textContent += event.command + ':' + event.source.id + ':' + String(document.getElementById('dlg').open) + ';'; event.preventDefault(); }); dialog.showModal();</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.click("#close");
    try subject.assertValue("#out", "request-close:close:true;");
    try subject.assertValue("#dlg", "");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46c5 popover target reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><article id='popover' popover='manual'></article><button id='trigger' type='button'></button><div id='out'></div><script>const popover = document.getElementById('popover'); const trigger = document.getElementById('trigger'); const before = String(trigger.popoverTargetElement) + ':' + trigger.popoverTargetAction; trigger.popoverTargetElement = popover; trigger.popoverTargetAction = 'show'; trigger.click(); const afterShow = trigger.popoverTargetElement.id + ':' + trigger.popoverTargetAction + ':' + trigger.getAttribute('popovertarget') + ':' + trigger.getAttribute('popovertargetaction') + ':' + String(popover.matches(':popover-open')) + ':' + String(document.querySelectorAll(':popover-open').length); trigger.popoverTargetElement = null; const afterClear = String(trigger.popoverTargetElement) + ':' + String(trigger.getAttribute('popovertarget')); document.getElementById('out').textContent = before + '|' + afterShow + '|' + afterClear;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "null:toggle|popover:show:popover:show:true:1|null:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46d input.indeterminate resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><progress id='loading'></progress><input id='check' type='checkbox'><div id='out'></div><script>const check = document.getElementById('check'); const before = String(check.indeterminate) + ':' + String(document.querySelectorAll(':indeterminate').length); check.indeterminate = true; const after = String(check.indeterminate) + ':' + String(document.querySelectorAll(':indeterminate').length); check.indeterminate = false; const cleared = String(check.indeterminate) + ':' + String(document.querySelectorAll(':indeterminate').length); document.getElementById('out').textContent = before + '|' + after + '|' + cleared;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:1|true:2|false:1");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46de input.capture resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='file' type='file' capture='user'><div id='out'></div><script>const file = document.getElementById('file'); const before = file.capture + ':' + file.getAttribute('capture'); file.capture = 'environment'; const during = file.capture + ':' + file.getAttribute('capture'); file.capture = ''; document.getElementById('out').textContent = before + '|' + during + '|' + file.capture + ':' + String(file.getAttribute('capture'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "user:user|environment:environment|:");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46e input.accept and input.size resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='upload' type='file' accept='.png,image/*'><input id='plain' type='text'><input id='fixed' type='text' size='12'><div id='out'></div><script>const upload = document.getElementById('upload'); const plain = document.getElementById('plain'); const fixed = document.getElementById('fixed'); const before = upload.accept + ':' + String(plain.size) + ':' + String(fixed.size); upload.accept = 'audio/*'; plain.size = 18; fixed.size = 6; document.getElementById('out').textContent = before + '|' + upload.accept + ':' + String(plain.size) + ':' + String(fixed.size) + ':' + upload.getAttribute('accept') + ':' + plain.getAttribute('size') + ':' + fixed.getAttribute('size');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ".png,image/*:20:12|audio/*:18:6:audio/*:18:6");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46f1 HTMLInputElement image reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='photo' type='image' src='/photo.png' alt='Photo' usemap='#map' width='320' height='240'><div id='out'></div><script>const photo = document.getElementById('photo'); const before = photo.src + ':' + photo.alt + ':' + photo.useMap + ':' + String(photo.width) + ':' + String(photo.height); photo.src = '/next.png'; photo.alt = 'Next'; photo.useMap = '#next-map'; photo.width = 640; photo.height = 480; document.getElementById('out').textContent = before + '|' + photo.src + ':' + photo.alt + ':' + photo.useMap + ':' + String(photo.width) + ':' + String(photo.height) + ':' + photo.getAttribute('src') + ':' + photo.getAttribute('alt') + ':' + photo.getAttribute('usemap') + ':' + photo.getAttribute('width') + ':' + photo.getAttribute('height');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "/photo.png:Photo:#map:320:240|/next.png:Next:#next-map:640:480:/next.png:Next:#next-map:640:480",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46f2 HTMLInputElement color reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='color' type='color' colorspace='display-p3' alpha><div id='out'></div><script>const color = document.getElementById('color'); const before = String(color.alpha) + ':' + color.colorSpace + ':' + color.getAttribute('colorspace') + ':' + String(color.hasAttribute('alpha')); color.alpha = false; color.colorSpace = 'bogus'; document.getElementById('out').textContent = before + '|' + String(color.alpha) + ':' + color.colorSpace + ':' + color.getAttribute('colorspace') + ':' + String(color.hasAttribute('alpha'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:display-p3:display-p3:true|false:limited-srgb:limited-srgb:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46f2b HTMLInputElement multiple reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='mail' type='email'><div id='out'></div><script>const mail = document.getElementById('mail'); const before = String(mail.multiple) + ':' + String(mail.hasAttribute('multiple')); mail.multiple = true; const during = String(mail.multiple) + ':' + String(mail.getAttribute('multiple')); mail.multiple = false; document.getElementById('out').textContent = before + '|' + during + '|' + String(mail.multiple) + ':' + String(mail.hasAttribute('multiple'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:false|true:|false:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46f3 HTMLInputElement capture reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='file' type='file' capture='environment'><div id='out'></div><script>const file = document.getElementById('file'); const before = file.capture + ':' + file.getAttribute('capture'); file.capture = 'user'; const during = file.capture + ':' + file.getAttribute('capture'); file.capture = 'bogus'; const after = file.capture + ':' + file.getAttribute('capture'); document.getElementById('out').textContent = before + '|' + during + '|' + after;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "environment:environment|user:user|:bogus");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46f3b HTMLInputElement capture normalization resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='file' type='file' capture='environment'><div id='out'></div><script>const file = document.getElementById('file'); const before = file.capture + ':' + file.getAttribute('capture'); file.capture = 'user'; const during = file.capture + ':' + file.getAttribute('capture'); file.capture = 'bogus'; const after = file.capture + ':' + file.getAttribute('capture'); document.getElementById('out').textContent = before + '|' + during + '|' + after;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "environment:environment|user:user|:bogus");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46f4 HTMLInputElement type reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='field'><div id='out'></div><script>const field = document.getElementById('field'); const before = field.type + ':' + field.getAttribute('type'); field.type = 'NUMBER'; const during = field.type + ':' + field.getAttribute('type'); field.type = 'search'; document.getElementById('out').textContent = before + '|' + during + '|' + field.type + ':' + field.getAttribute('type');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "text:null|number:number|search:search");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46f enterKeyHint resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='box' enterkeyhint='search'></div><textarea id='area'></textarea><div id='out'></div><script>const box = document.getElementById('box'); const area = document.getElementById('area'); const before = box.enterKeyHint + ':' + area.enterKeyHint; box.enterKeyHint = 'send'; area.enterKeyHint = 'next'; document.getElementById('out').textContent = before + '|' + box.enterKeyHint + ':' + area.enterKeyHint + ':' + box.getAttribute('enterkeyhint') + ':' + area.getAttribute('enterkeyhint');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "search:|send:next:send:next");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46g dirName resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='input' dirname='input.dir'><textarea id='area' dirname='area.dir'></textarea><div id='out'></div><script>const input = document.getElementById('input'); const area = document.getElementById('area'); const before = input.dirName + ':' + area.dirName; input.dirName = 'input.next'; area.dirName = 'area.next'; document.getElementById('out').textContent = before + '|' + input.dirName + ':' + area.dirName + ':' + input.getAttribute('dirname') + ':' + area.getAttribute('dirname');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "input.dir:area.dir|input.next:area.next:input.next:area.next");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 46h textarea rows cols and wrap resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><textarea id='area' rows='0' cols='0' wrap='bogus'>Hello</textarea><div id='out'></div><script>const area = document.getElementById('area'); const before = String(area.rows) + ':' + String(area.cols) + ':' + area.wrap + ':' + String(area.getAttribute('rows')) + ':' + String(area.getAttribute('cols')) + ':' + String(area.getAttribute('wrap')); area.rows = 4; area.cols = 10; area.wrap = 'hard'; const during = String(area.rows) + ':' + String(area.cols) + ':' + area.wrap + ':' + String(area.getAttribute('rows')) + ':' + String(area.getAttribute('cols')) + ':' + String(area.getAttribute('wrap')); area.rows = 0; area.cols = 0; area.wrap = 'soft'; document.getElementById('out').textContent = before + '|' + during + '|' + String(area.rows) + ':' + String(area.cols) + ':' + area.wrap + ':' + String(area.getAttribute('rows')) + ':' + String(area.getAttribute('cols')) + ':' + String(area.getAttribute('wrap'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2:20:soft:0:0:bogus|4:10:hard:4:10:hard|2:20:soft:0:0:soft");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47 selectionchange resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><textarea id='bio'>Hello</textarea><div id='out'></div><script>document.onselectionchange = () => { document.getElementById('out').textContent += '1'; };</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.typeText("#bio", "World");
    try subject.assertValue("#out", "1");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47a getSelection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(1, 3, 'backward'); const selection = document.getSelection(); document.getElementById('out').textContent = String(selection) + ':' + String(selection.rangeCount) + ':' + selection.type + ':' + String(selection.containsNode(name));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "da:1:Range:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47b collapseToEnd resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); document.getSelection().collapseToEnd(); document.getElementById('out').textContent = String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "12:12:none");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47c readystatechange resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>document.onreadystatechange = () => { document.getElementById('out').textContent += ':' + document.readyState; }; document.getElementById('out').textContent = document.readyState;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "loading:complete");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47d removeAllRanges resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(1, 3, 'backward'); const selection = document.getSelection(); selection.removeAllRanges(); const current = document.getSelection(); document.getElementById('out').textContent = String(current.rangeCount) + ':' + String(current.isCollapsed) + ':' + current.type + ':' + String(current.anchorNode);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "0:true:None:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47e collapse resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const selection = document.getSelection(); selection.collapse(name, 2); document.getElementById('out').textContent = String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + ':' + String(document.getSelection()) + ':' + String(document.getSelection().rangeCount) + ':' + document.getSelection().type;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2:2:none::1:Caret");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47f extend resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const selection = document.getSelection(); selection.extend(name, 2); const current = document.getSelection(); document.getElementById('out').textContent = String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + ':' + String(current) + ':' + String(current.rangeCount) + ':' + current.type + ':' + String(current.anchorOffset) + ':' + String(current.focusOffset);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2:12:backward:a Lovelace:1:Range:12:2");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47g setBaseAndExtent resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); const selection = document.getSelection(); selection.setBaseAndExtent(name, 12, name, 4); const current = document.getSelection(); document.getElementById('out').textContent = String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + ':' + String(current) + ':' + String(current.rangeCount) + ':' + current.type + ':' + String(current.anchorOffset) + ':' + String(current.focusOffset);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "4:12:backward:Lovelace:1:Range:12:4");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47h deleteFromDocument resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const selection = document.getSelection(); selection.deleteFromDocument(); const current = document.getSelection(); document.getElementById('out').textContent = name.value + '|' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + '|' + String(current.rangeCount) + ':' + current.type + ':' + String(current.anchorOffset) + ':' + String(current.focusOffset);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "Ada |4:4:none|1:Caret:4:4");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47i setPosition resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); const selection = document.getSelection(); selection.setPosition(name, 2); const current = document.getSelection(); document.getElementById('out').textContent = String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + ':' + String(current) + ':' + String(current.rangeCount) + ':' + current.type + ':' + String(current.anchorOffset) + ':' + String(current.focusOffset);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2:2:none::1:Caret:2:2");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47j selectAllChildren resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); const selection = document.getSelection(); selection.selectAllChildren(name); const current = document.getSelection(); document.getElementById('out').textContent = String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + ':' + String(current) + ':' + String(current.rangeCount) + ':' + current.type + ':' + String(current.anchorOffset) + ':' + String(current.focusOffset);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "0:3:none:Ada:1:Range:0:3");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47k getRangeAt resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(1, 3, 'backward'); const range = document.getSelection().getRangeAt(0); document.getElementById('out').textContent = String(range) + '|' + range.startContainer.id + ':' + range.endContainer.id + '|' + String(range.startOffset) + ':' + String(range.endOffset) + '|' + String(range.collapsed) + '|' + range.commonAncestorContainer.id;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "da|name:name|1:3|false|name");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: document.createRange resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const range = document.createRange(); document.getElementById('out').textContent = String(range) + ':' + String(range.collapsed) + ':' + String(range.startContainer) + ':' + String(range.endContainer) + ':' + String(range.commonAncestorContainer) + ':' + String(range.startOffset) + ':' + String(range.endOffset);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", ":true:null:null:null:0:0");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: Range.setStart, Range.setEnd, and Range.collapse resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='box'>Hello</div><div id='out'></div><script>const box = document.getElementById('box'); const start = document.createRange().selectNodeContents(box).setEnd(box, 2).setStart(box, 4); const end = document.createRange().selectNodeContents(box).setStart(box, 4).setEnd(box, 2); const collapsed = document.createRange().selectNodeContents(box).collapse(); document.getElementById('out').textContent = String(start) + '|' + String(start.startOffset) + ':' + String(start.endOffset) + ':' + String(start.collapsed) + '|' + String(end) + '|' + String(end.startOffset) + ':' + String(end.endOffset) + ':' + String(end.collapsed) + '|' + String(collapsed) + '|' + String(collapsed.startOffset) + ':' + String(collapsed.endOffset) + ':' + String(collapsed.collapsed);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "|4:4:true||2:2:true||5:5:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: Range.cloneRange resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const range = document.getSelection().getRangeAt(0); const clone = range.cloneRange(); document.getElementById('out').textContent = String(range) + '|' + String(clone) + '|' + clone.startContainer.id + ':' + clone.endContainer.id + '|' + String(clone.startOffset) + ':' + String(clone.endOffset) + '|' + String(clone.collapsed) + '|' + clone.commonAncestorContainer.id;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "Lovelace|Lovelace|name:name|4:12|false|name");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: Range.deleteContents resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const range = document.getSelection().getRangeAt(0); range.deleteContents(); document.getElementById('out').textContent = name.value + '|' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + '|' + String(document.getSelection().rangeCount) + ':' + String(document.getSelection().type) + ':' + String(document.getSelection().anchorOffset) + ':' + String(document.getSelection().focusOffset);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "Ada |4:4:none|1:Caret:4:4");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: Range.isPointInRange resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const range = document.getSelection().getRangeAt(0); document.getElementById('out').textContent = String(range.isPointInRange(name, 4)) + ':' + String(range.isPointInRange(name, 9)) + ':' + String(range.isPointInRange(name, 12)) + ':' + String(range.isPointInRange(name, 3));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:true:true:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: Range.comparePoint resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const range = document.getSelection().getRangeAt(0); document.getElementById('out').textContent = String(range.comparePoint(name, 4)) + ':' + String(range.comparePoint(name, 9)) + ':' + String(range.comparePoint(name, 12)) + ':' + String(range.comparePoint(name, 3));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "0:0:0:-1");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: Range.cloneContents resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const range = document.getSelection().getRangeAt(0); const fragment = range.cloneContents(); document.getElementById('out').textContent = fragment.textContent + '|' + String(fragment) + '|' + name.value + '|' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + '|' + String(document.getSelection().rangeCount) + ':' + String(document.getSelection().type);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "Lovelace|[object DocumentFragment]|Ada Lovelace|4:12:backward|1:Range");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47l progress and meter reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='owner'></form><label id='progress-label' for='progress'>Progress</label><label id='meter-label' for='meter'>Meter</label><progress id='progress' form='owner' max='4' value='3'>75%</progress><meter id='meter' form='owner' min='2' max='10' value='12' low='3' high='8' optimum='9'>12</meter><div id='out'></div><script>const progress = document.getElementById('progress'); const meter = document.getElementById('meter'); const before = String(progress.value) + ':' + String(progress.max) + ':' + String(progress.position) + ':' + String(progress.labels.length) + ':' + progress.form.id + '|' + String(meter.value) + ':' + String(meter.min) + ':' + String(meter.max) + ':' + String(meter.low) + ':' + String(meter.high) + ':' + String(meter.optimum) + ':' + String(meter.labels.length) + ':' + meter.form.id; progress.max = 2; progress.value = 5; meter.min = 1; meter.max = 5; meter.value = 4; meter.low = 2; meter.high = 3; meter.optimum = 2.5; const after = progress.getAttribute('max') + ':' + progress.getAttribute('value') + ':' + String(progress.value) + ':' + String(progress.max) + ':' + String(progress.position) + '|' + meter.getAttribute('min') + ':' + String(meter.min) + ':' + meter.getAttribute('max') + ':' + String(meter.max) + ':' + meter.getAttribute('value') + ':' + String(meter.value) + ':' + meter.getAttribute('low') + ':' + String(meter.low) + ':' + meter.getAttribute('high') + ':' + String(meter.high) + ':' + meter.getAttribute('optimum') + ':' + String(meter.optimum); document.getElementById('out').textContent = before + '|' + after;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "3:4:0.75:1:owner|10:2:10:3:8:9:1:owner|2:5:2:2:1|1:1:5:5:4:4:2:2:3:3:2.5:2.5");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: Range.createContextualFragment resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(1, 3, 'backward'); const range = document.getSelection().getRangeAt(0); const fragment = range.createContextualFragment('<strong>Byron</strong>'); document.getElementById('out').textContent = fragment.innerHTML + '|' + fragment.textContent + '|' + name.value + '|' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + '|' + String(document.getSelection().rangeCount) + ':' + String(document.getSelection().type);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "<strong>Byron</strong>|Byron|Ada|1:3:backward|1:Range");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: Range.selectNode and selectNodeContents resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='box'><span>Hello</span><em>!</em></div><div id='out'></div><script>const box = document.getElementById('box'); const contents = document.createRange().selectNodeContents(box); const node = document.createRange().selectNode(box); document.getElementById('out').textContent = String(contents) + '|' + String(contents.startOffset) + ':' + String(contents.endOffset) + ':' + String(contents.collapsed) + '|' + String(node) + '|' + String(node.startOffset) + ':' + String(node.endOffset) + ':' + String(node.collapsed);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "Hello!|0:6:false|Hello!|0:6:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: Range.insertNode resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='box'><span>Hello</span></div><div id='out'></div><script>const box = document.getElementById('box'); const range = document.createRange().selectNodeContents(box); const node = document.createElement('em'); node.textContent = 'Byron'; range.insertNode(node); document.getElementById('out').textContent = box.innerHTML + '|' + String(range) + '|' + String(range.startOffset) + ':' + String(range.endOffset) + ':' + String(range.collapsed);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "<em>Byron</em><span>Hello</span>|Hello|0:5:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: Range.surroundContents resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='box'><span>Hello</span><em>!</em></div><div id='out'></div><script>const box = document.getElementById('box'); const range = document.createRange().selectNodeContents(box); const wrapper = document.createElement('strong'); range.surroundContents(wrapper); document.getElementById('out').textContent = box.innerHTML + '|' + wrapper.innerHTML;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "<strong><span>Hello</span><em>!</em></strong>|<span>Hello</span><em>!</em>");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: Range.compareBoundaryPoints resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const first = document.getSelection().getRangeAt(0); name.setSelectionRange(1, 3, 'backward'); const second = document.getSelection().getRangeAt(0); document.getElementById('out').textContent = String(first.compareBoundaryPoints(0, second)) + ':' + String(first.compareBoundaryPoints(1, second)) + ':' + String(first.compareBoundaryPoints(2, second)) + ':' + String(first.compareBoundaryPoints(3, second));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:1:1:1");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: Range.extractContents resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada Lovelace'><div id='out'></div><script>const name = document.getElementById('name'); name.focus(); name.setSelectionRange(4, 12, 'backward'); const range = document.getSelection().getRangeAt(0); const fragment = range.extractContents(); document.getElementById('out').textContent = fragment.textContent + '|' + String(fragment) + '|' + name.value + '|' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection + '|' + String(document.getSelection().rangeCount) + ':' + String(document.getSelection().type) + ':' + String(document.getSelection().anchorOffset) + ':' + String(document.getSelection().focusOffset);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "Lovelace|[object DocumentFragment]|Ada |4:4:none|1:Caret:4:4");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47l addRange and removeRange resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='name' value='Ada'><div id='count'></div><div id='out'></div><script>const name = document.getElementById('name'); const count = document.getElementById('count'); document.onselectionchange = () => { document.getElementById('count').textContent += '1'; }; name.focus(); name.setSelectionRange(1, 3, 'backward'); const selection = document.getSelection(); const range = selection.getRangeAt(0); selection.removeRange(range); const removedSelection = document.getSelection(); const removed = String(count.textContent) + ':' + String(removedSelection.rangeCount) + ':' + removedSelection.type + ':' + String(removedSelection.anchorNode) + ':' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection; selection.addRange(range); const restoredSelection = document.getSelection(); const restored = String(count.textContent) + ':' + String(restoredSelection.rangeCount) + ':' + restoredSelection.type + ':' + String(restoredSelection) + ':' + String(restoredSelection.anchorOffset) + ':' + String(restoredSelection.focusOffset) + ':' + String(name.selectionStart) + ':' + String(name.selectionEnd) + ':' + name.selectionDirection; selection.removeAllRanges(); selection.removeRange(range); const clearedSelection = document.getSelection(); const cleared = String(count.textContent) + ':' + String(clearedSelection.rangeCount) + ':' + clearedSelection.type + ':' + String(clearedSelection.anchorNode); document.getElementById('out').textContent = removed + '|' + restored + '|' + cleared;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "11:0:None:null:1:3:backward|111:1:Range:da:1:3:1:3:none|1111:0:None:null");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47m HTMLMediaElement playback state resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><video id='clip' muted></video><div id='out'></div><script>const clip = document.getElementById('clip'); const before = String(clip.defaultMuted) + ':' + String(clip.muted) + ':' + String(clip.currentTime) + ':' + String(clip.duration) + ':' + String(clip.paused) + ':' + String(clip.seeking) + ':' + String(clip.ended) + ':' + String(clip.readyState) + ':' + String(clip.networkState) + ':' + String(clip.volume) + ':' + String(clip.defaultPlaybackRate) + ':' + String(clip.playbackRate) + ':' + String(clip.preservesPitch); clip.currentTime = 42.5; clip.muted = false; clip.volume = 0.25; clip.defaultPlaybackRate = 1.5; clip.playbackRate = 0.5; clip.preservesPitch = false; clip.disablePictureInPicture = true; clip.disableRemotePlayback = true; clip.controlsList.add('nodownload', 'nofullscreen', 'noremoteplayback'); document.getElementById('out').textContent = before + '|' + String(clip.defaultMuted) + ':' + String(clip.muted) + ':' + String(clip.currentTime) + ':' + String(clip.duration) + ':' + String(clip.paused) + ':' + String(clip.seeking) + ':' + String(clip.ended) + ':' + String(clip.readyState) + ':' + String(clip.networkState) + ':' + String(clip.volume) + ':' + String(clip.defaultPlaybackRate) + ':' + String(clip.playbackRate) + ':' + String(clip.preservesPitch) + ':' + String(clip.disablePictureInPicture) + ':' + String(clip.disableRemotePlayback) + ':' + String(clip.controlsList.value) + ':' + String(clip.controlsList.length) + ':' + String(clip.controlsList.item(0)) + ':' + String(clip.controlsList.supports('noremoteplayback')) + ':' + String(clip.controlsList.contains('nodownload')) + ':' + String(clip.hasAttribute('disablepictureinpicture')) + ':' + String(clip.hasAttribute('disableremoteplayback')) + ':' + String(clip.hasAttribute('controlslist')) + ':' + clip.getAttribute('muted');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:true:0:NaN:true:false:false:0:0:1:1:1:true|true:false:42.5:NaN:true:false:false:0:0:0.25:1.5:0.5:false:true:true:nodownload nofullscreen noremoteplayback:3:nodownload:true:true:true:true:true:");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47p HTMLAudioElement playback state resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><audio id='clip' src='track.ogg' muted controls preload='metadata' crossorigin='anonymous'></audio><div id='out'></div><script>const clip = document.getElementById('clip'); const before = String(clip.currentSrc) + ':' + String(clip.defaultMuted) + ':' + String(clip.muted) + ':' + String(clip.currentTime) + ':' + String(clip.duration) + ':' + String(clip.paused) + ':' + String(clip.seeking) + ':' + String(clip.ended) + ':' + String(clip.readyState) + ':' + String(clip.networkState) + ':' + String(clip.volume) + ':' + String(clip.defaultPlaybackRate) + ':' + String(clip.playbackRate) + ':' + String(clip.preservesPitch) + ':' + String(clip.disableRemotePlayback) + ':' + String(clip.controlsList.value); clip.currentTime = 11.25; clip.muted = false; clip.volume = 0.25; clip.defaultPlaybackRate = 1.5; clip.playbackRate = 0.5; clip.preservesPitch = false; clip.disableRemotePlayback = true; clip.controlsList.add('nodownload', 'noremoteplayback'); document.getElementById('out').textContent = before + '|' + String(clip.currentSrc) + ':' + String(clip.defaultMuted) + ':' + String(clip.muted) + ':' + String(clip.currentTime) + ':' + String(clip.duration) + ':' + String(clip.paused) + ':' + String(clip.seeking) + ':' + String(clip.ended) + ':' + String(clip.readyState) + ':' + String(clip.networkState) + ':' + String(clip.volume) + ':' + String(clip.defaultPlaybackRate) + ':' + String(clip.playbackRate) + ':' + String(clip.preservesPitch) + ':' + String(clip.disableRemotePlayback) + ':' + String(clip.controlsList.value) + ':' + String(clip.controlsList.length) + ':' + String(clip.controlsList.item(0)) + ':' + String(clip.controlsList.supports('noremoteplayback')) + ':' + String(clip.controlsList.contains('nodownload')) + ':' + clip.getAttribute('crossorigin');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "track.ogg:true:true:0:NaN:true:false:false:0:0:1:1:1:true:false:|track.ogg:true:false:11.25:NaN:true:false:false:0:0:0.25:1.5:0.5:false:true:nodownload noremoteplayback:2:nodownload:true:true:anonymous");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47p1 HTMLProgressElement and HTMLMeterElement reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='owner'></form><label id='progress-label' for='progress'>Progress</label><label id='meter-label' for='meter'>Meter</label><progress id='progress' form='owner' max='4' value='3'>75%</progress><meter id='meter' form='owner' min='2' max='10' value='12' low='3' high='8' optimum='9'>12</meter><div id='out'></div><script>const progress = document.getElementById('progress'); const meter = document.getElementById('meter'); const before = String(progress.value) + ':' + String(progress.max) + ':' + String(progress.position) + ':' + String(progress.labels.length) + ':' + progress.form.id + '|' + String(meter.value) + ':' + String(meter.min) + ':' + String(meter.max) + ':' + String(meter.low) + ':' + String(meter.high) + ':' + String(meter.optimum) + ':' + String(meter.labels.length) + ':' + meter.form.id; progress.max = 2; progress.value = 5; meter.min = 1; meter.max = 5; meter.value = 4; meter.low = 2; meter.high = 3; meter.optimum = 2.5; const after = progress.getAttribute('max') + ':' + progress.getAttribute('value') + ':' + String(progress.value) + ':' + String(progress.max) + ':' + String(progress.position) + '|' + meter.getAttribute('min') + ':' + String(meter.min) + ':' + meter.getAttribute('max') + ':' + String(meter.max) + ':' + meter.getAttribute('value') + ':' + String(meter.value) + ':' + meter.getAttribute('low') + ':' + String(meter.low) + ':' + meter.getAttribute('high') + ':' + String(meter.high) + ':' + meter.getAttribute('optimum') + ':' + String(meter.optimum); document.getElementById('out').textContent = before + '|' + after;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "3:4:0.75:1:owner|10:2:10:3:8:9:1:owner|2:5:2:2:1|1:1:5:5:4:4:2:2:3:3:2.5:2.5");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47u HTMLMediaElement methods resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><video id='clip' src='movie.mp4'></video><div id='out'></div><script>const clip = document.getElementById('clip'); const before = clip.canPlayType('video/mp4') + ':' + clip.canPlayType('application/json') + ':' + String(clip.load()) + ':' + String(clip.pause()); clip.load(); clip.pause(); document.getElementById('out').textContent = before + '|' + clip.canPlayType('VIDEO/MP4') + ':' + clip.canPlayType('text/plain') + ':' + String(clip.load()) + ':' + String(clip.pause());</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "maybe::undefined:undefined|maybe::undefined:undefined");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47q FormData and formdata resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><form id='form'><input id='name' name='name' value='Ada'><select id='choice' name='choice'><option value='alpha' selected>Alpha</option></select><button id='submit' type='submit' name='submit' value='go'></button></form><div id='out'></div><script>const form = document.getElementById('form'); const button = document.getElementById('submit'); const data = new FormData(form); const before = String(data) + ':' + data.get('name') + ':' + data.get('choice') + ':' + String(data.getAll('choice').item(0)) + ':' + String(data.has('submit')) + ':' + String(data.getAll('choice').length); data.append('extra', '1'); data.set('name', 'Bea'); data.delete('choice'); const after = String(data) + ':' + data.get('name') + ':' + String(data.has('choice')) + ':' + String(data.get('extra')) + ':' + String(data.getAll('extra').length); form.addEventListener('submit', (event) => { document.getElementById('out').textContent += 'submit:' + event.submitter.id + '|'; }); form.addEventListener('formdata', (event) => { const formData = event.formData; document.getElementById('out').textContent += String(event) + ':' + formData.get('name') + ':' + formData.get('choice') + ':' + formData.get('submit') + ':' + String(formData.has('listener')) + '|'; formData.append('listener', 'yes'); document.getElementById('out').textContent += formData.get('listener'); }); form.requestSubmit(button); document.getElementById('out').textContent += '|' + before + '|' + after;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "submit:submit|[object FormDataEvent]:Ada:alpha:go:false|yes|[object FormData]:Ada:alpha:alpha:false:1|[object FormData]:Bea:false:1:1");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47r FormData iterator parity resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>const data = new FormData(); data.append('alpha', '1'); data.append('beta', '2'); data.append('alpha', '3'); const keys = data.keys(); const values = data.values(); const entries = data.entries(); const firstKey = keys.next(); const secondKey = keys.next(); const thirdKey = keys.next(); const fourthKey = keys.next(); const firstValue = values.next(); const secondValue = values.next(); const thirdValue = values.next(); const fourthValue = values.next(); const firstEntry = entries.next(); const secondEntry = entries.next(); const thirdEntry = entries.next(); const fourthEntry = entries.next(); document.getElementById('out').textContent = String(firstKey.value) + ':' + String(secondKey.value) + ':' + String(thirdKey.value) + ':' + String(fourthKey.done) + '|' + firstValue.value + ':' + secondValue.value + ':' + thirdValue.value + ':' + String(fourthValue.done) + '|' + firstEntry.value.name + ':' + firstEntry.value.value + ':' + secondEntry.value.name + ':' + secondEntry.value.value + ':' + thirdEntry.value.name + ':' + thirdEntry.value.value + ':' + String(fourthEntry.done) + '|'; data.forEach((value, name, formData) => { document.getElementById('out').textContent += name + ':' + value + ':' + formData.get(name) + ';'; }, null);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "alpha:beta:alpha:true|1:2:3:true|alpha:1:beta:2:alpha:3:true|alpha:1:1;beta:2:2;alpha:3:1;");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47o HTMLMediaElement controlsList resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><video id='clip'></video><div id='out'></div><script>const clip = document.getElementById('clip'); clip.controlsList.add('nodownload', 'nofullscreen'); document.getElementById('out').textContent = String(clip.controlsList.value) + ':' + String(clip.controlsList.length) + ':' + String(clip.controlsList.item(0)) + ':' + String(clip.controlsList.supports('nofullscreen')) + ':' + String(clip.controlsList.contains('nofullscreen')) + ':' + String(clip.hasAttribute('controlslist'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "nodownload nofullscreen:2:nodownload:true:true:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47n HTMLMediaElement boolean flags resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><video id='clip'></video><div id='out'></div><script>const clip = document.getElementById('clip'); clip.disablePictureInPicture = true; clip.disableRemotePlayback = true; document.getElementById('out').textContent = String(clip.disablePictureInPicture) + ':' + String(clip.disableRemotePlayback) + ':' + String(clip.hasAttribute('disablepictureinpicture')) + ':' + String(clip.hasAttribute('disableremoteplayback'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "true:true:true:true");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47n0 HTMLVideoElement poster and playsInline resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><video id='clip' poster='poster.png' playsinline></video><div id='out'></div><script>const clip = document.getElementById('clip'); const before = clip.poster + ':' + String(clip.playsInline) + ':' + clip.getAttribute('poster') + ':' + String(clip.hasAttribute('playsinline')); clip.poster = 'next.png'; clip.playsInline = false; document.getElementById('out').textContent = before + '|' + clip.poster + ':' + String(clip.playsInline) + ':' + clip.getAttribute('poster') + ':' + String(clip.hasAttribute('playsinline'));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "poster.png:true:poster.png:true|next.png:false:next.png:false");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47n1 HTMLMediaElement TimeRanges resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><video id='clip' src='movie.mp4'></video><div id='out'></div><script>const clip = document.getElementById('clip'); document.getElementById('out').textContent = String(clip.buffered) + ':' + String(clip.buffered.length) + ':' + String(clip.seekable) + ':' + String(clip.seekable.length) + ':' + String(clip.played) + ':' + String(clip.played.length) + ':' + String(clip.buffered.toString());</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[object TimeRanges]:0:[object TimeRanges]:0:[object TimeRanges]:0:[object TimeRanges]");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 47n2 HTMLMediaElement textTracks resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><video id='clip'><track id='cue' kind='subtitles' label='English' srclang='en'></video><div id='out'></div><script>const clip = document.getElementById('clip'); const tracks = clip.textTracks; const track = tracks.item(0); const before = String(tracks) + ':' + String(tracks.length) + ':' + track.kind + ':' + track.mode + ':' + String(track.readyState); track.mode = 'showing'; document.getElementById('out').textContent = before + '|' + String(clip.textTracks.length) + ':' + clip.textTracks.item(0).kind + ':' + clip.textTracks.item(0).mode + ':' + String(clip.textTracks.item(0).readyState);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "[object TextTrackList]:1:subtitles:disabled:0|1:subtitles:showing:0");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 48 scroll handlers resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='doc'></main><main id='win'></main><script>document.onscroll = () => { document.getElementById('doc').textContent = String(window.scrollX) + ':' + String(window.scrollY); }; window.onscroll = () => { document.getElementById('win').textContent = String(window.scrollX) + ':' + String(window.scrollY); }; window.scrollTo(10, 20); window.scrollBy(-3, 5);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#doc", "7:25");
    try subject.assertValue("#win", "7:25");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 48 scrollIntoView resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><section id='target'></section><div id='out'></div><script>window.scrollTo(10, 20); document.getElementById('target').scrollIntoView(); document.getElementById('out').textContent = String(window.scrollX) + ':' + String(window.scrollY);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "0:0");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 48a HTMLTrackElement reflection resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><track id='cue' kind='subtitles' src='captions.vtt' srclang='en' label='English' default><div id='out'></div><script>const cue = document.getElementById('cue'); const before = cue.kind + ':' + cue.src + ':' + cue.srclang + ':' + cue.label + ':' + String(cue.default) + ':' + String(cue.readyState) + ':' + String(cue.track) + ':' + cue.track.kind + ':' + cue.track.label + ':' + cue.track.language + ':' + cue.track.mode + ':' + String(cue.track.readyState); cue.kind = 'captions'; cue.src = 'subtitles.vtt'; cue.srclang = 'fr'; cue.label = 'Français'; cue.default = false; cue.track.mode = 'showing'; document.getElementById('out').textContent = before + '|' + cue.kind + ':' + cue.src + ':' + cue.srclang + ':' + cue.label + ':' + String(cue.default) + ':' + String(cue.readyState) + ':' + String(cue.track) + ':' + cue.track.kind + ':' + cue.track.label + ':' + cue.track.language + ':' + cue.track.mode + ':' + String(cue.track.readyState);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "subtitles:captions.vtt:en:English:true:0:[object TextTrack]:subtitles:English:en:disabled:0|captions:subtitles.vtt:fr:Français:false:0:[object TextTrack]:captions:Français:fr:showing:0");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 48b HTMLDetailsElement open and name resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><details id='details'><summary id='summary'>Title</summary><div>Body</div></details><div id='out'></div><script>const details = document.getElementById('details'); const out = document.getElementById('out'); details.name = 'accordion'; out.textContent = String(details.open) + ':' + details.name; details.addEventListener('toggle', () => { const current = document.getElementById('details'); const output = document.getElementById('out'); output.textContent += '|' + String(current.open) + ':' + current.name; }); details.open = true;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "false:accordion|true:accordion");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 49 stepUp and stepDown resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><input id='date' type='date' value='2017-06-01'><input id='month' type='month' step='2' value='2017-06'><input id='time' type='time' step='1800' value='09:00'><input id='week' type='week' value='2017-W01'><div id='out'></div><script>const date = document.getElementById('date'); const month = document.getElementById('month'); const time = document.getElementById('time'); const week = document.getElementById('week'); date.stepUp(2); month.stepUp(); time.stepDown(); week.stepUp(); document.getElementById('out').textContent = date.value + '|' + month.value + '|' + time.value + '|' + week.value;</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "2017-06-03|2017-08|08:30|2017-W02");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: token-list value and item resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='button' class='base primary base'>First</button><div id='parted' part='primary secondary primary'></div><link id='link' rel='stylesheet preload stylesheet' href='a.css'><div id='out'></div><script>const button = document.getElementById('button'); const parted = document.getElementById('parted'); const link = document.getElementById('link'); const before = button.classList.value + ':' + parted.part.value + ':' + link.relList.value + ':' + String(button.classList.item(0)) + ':' + String(button.classList.item(9)) + ':' + String(parted.part.item(0)) + ':' + String(link.relList.item(0)) + ':' + String(link.relList.item(9)); button.classList.value = 'alpha  beta alpha'; parted.part.value = 'accent accent tertiary'; link.relList.value = 'stylesheet preload stylesheet'; document.getElementById('out').textContent = before + '|' + button.className + ':' + button.classList.value + ':' + String(button.classList.item(0)) + ':' + String(button.classList.item(1)) + ':' + parted.getAttribute('part') + ':' + parted.part.value + ':' + String(parted.part.item(0)) + ':' + link.rel + ':' + link.relList.value + ':' + String(link.sheet) + ':' + String(link.relList.contains('stylesheet')) + ':' + String(link.relList.supports('preload')) + ':' + String(link.relList.item(0)) + ':' + String(link.relList.item(1));</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "base primary:primary secondary:stylesheet preload:base:null:primary:stylesheet:null|alpha beta:alpha beta:alpha:beta:accent tertiary:accent tertiary:accent:stylesheet preload:stylesheet preload:[object CSSStyleSheet]:true:true:stylesheet:preload");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: token-list iterators and forEach resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='button' class='base primary base'>First</button><div id='parted' part='primary secondary primary'></div><link id='link' rel='stylesheet preload stylesheet' href='a.css'><div id='out'></div><script>const button = document.getElementById('button'); const parted = document.getElementById('parted'); const link = document.getElementById('link'); const classKeys = button.classList.keys(); const classValues = button.classList.values(); const classEntries = button.classList.entries(); const classKey0 = classKeys.next(); const classKey1 = classKeys.next(); const classKey2 = classKeys.next(); const classValue0 = classValues.next(); const classValue1 = classValues.next(); const classValue2 = classValues.next(); const classEntry0 = classEntries.next(); const classEntry1 = classEntries.next(); const classEntry2 = classEntries.next(); document.getElementById('out').textContent = String(classKey0.value) + ':' + String(classKey1.value) + ':' + String(classKey2.done) + '|' + classValue0.value + ':' + classValue1.value + ':' + String(classValue2.done) + '|' + String(classEntry0.value.index) + ':' + classEntry0.value.value + ':' + String(classEntry1.value.index) + ':' + classEntry1.value.value + ':' + String(classEntry2.done) + '|'; button.classList.forEach((token, index, list) => { document.getElementById('out').textContent += String(index) + ':' + token + ':' + String(list.length) + ';'; }, null); document.getElementById('out').textContent += '|'; parted.part.forEach((token, index, list) => { document.getElementById('out').textContent += String(index) + ':' + token + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent += '|'; link.relList.forEach((token, index, list) => { document.getElementById('out').textContent += String(index) + ':' + token + ':' + String(list.length) + ';'; }, null); button.classList.value = 'alpha beta'; parted.part.value = 'accent tertiary'; link.relList.value = 'stylesheet preload';</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue(
        "#out",
        "0:1:true|base:primary:true|0:base:1:primary:true|0:base:2;1:primary:2;|0:primary:2;1:secondary:2;|0:stylesheet:2;1:preload:2;",
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: storage mock resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const local = window.localStorage; const session = window.sessionStorage; const before = String(local.length) + ':' + String(session.length) + ':' + local.getItem('token') + ':' + session.getItem('session-token'); local.setItem('theme', 'dark'); local.removeItem('token'); session.setItem('scratch', 'xyz'); session.clear(); document.getElementById('out').textContent = before + '|' + String(local.length) + ':' + local.key(0) + ':' + String(session.length);</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var builder = Harness.builder(allocator);
    defer builder.deinit();
    _ = builder.html(html_bytes);
    try builder.addLocalStorage("token", "abc");
    try builder.addSessionStorage("session-token", "xyz");

    var subject = try builder.build();
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:1:abc:xyz|1:theme:0");
    try std.testing.expectEqualStrings(original, subject.html().?);
    try std.testing.expectEqualStrings("dark", subject.mocksMut().storage().local().get("theme").?);
    try std.testing.expectEqual(@as(?[]const u8, null), subject.mocksMut().storage().local().get("token"));
    try std.testing.expectEqual(@as(?[]const u8, null), subject.mocksMut().storage().session().get("session-token"));
}

test "regression: phase 35 fromHtmlWithUrlAndSessionStorage resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='out'></main><script>const session = window.sessionStorage; const before = String(session.length) + ':' + session.getItem('session-token'); session.setItem('scratch', 'xyz'); session.removeItem('session-token'); document.getElementById('out').textContent = before + '|' + String(session.length) + ':' + session.getItem('scratch');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    const seeds = [_]session.StorageSeed{
        .{
            .key = "session-token",
            .value = "seed",
        },
    };

    var subject = try Harness.fromHtmlWithUrlAndSessionStorage(
        allocator,
        "https://example.test/session",
        html_bytes,
        &seeds,
    );
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:seed|1:xyz");
    try std.testing.expectEqualStrings(original, subject.html().?);
    try std.testing.expectEqualStrings(
        "https://example.test/session",
        subject.url(),
    );
    try std.testing.expectEqual(@as(?[]const u8, null), subject.mocksMut().storage().session().get("session-token"));
    try std.testing.expectEqualStrings("xyz", subject.mocksMut().storage().session().get("scratch").?);
}

test "regression: matchMedia mock resolves on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='toggle'>Toggle</button><div id='out'></div><script>document.getElementById('toggle').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); document.getElementById('out').textContent = String(mql) + ':' + mql.media + ':' + String(mql.matches); });</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.mocksMut().matchMedia().seedMatch("(max-width: 600px)", true);
    try subject.click("#toggle");
    try subject.assertValue("#out", "[object MediaQueryList]:(max-width: 600px):true");
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().matchMedia().calls().len);
    try std.testing.expectEqualStrings(
        "(max-width: 600px)",
        subject.mocksMut().matchMedia().calls()[0].query,
    );
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: matchMedia listeners on the copied html snapshot track reseeded state" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='toggle'>Toggle</button><div id='out'></div><script>document.getElementById('toggle').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); mql.addListener(() => { document.getElementById('out').textContent = 'changed'; }); document.getElementById('out').textContent = String(mql.matches); });</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

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
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: matchMedia change event listeners on the copied html snapshot track reseeded state" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='toggle'>Toggle</button><div id='out'></div><script>document.getElementById('toggle').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); mql.addEventListener('change', () => { document.getElementById('out').textContent = 'changed'; }); document.getElementById('out').textContent = String(mql.matches); });</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

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
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: matchMedia onchange callbacks on the copied html snapshot track reseeded state" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='toggle'>Toggle</button><div id='out'></div><script>document.getElementById('toggle').addEventListener('click', () => { const mql = window.matchMedia('(max-width: 600px)'); mql.onchange = () => { document.getElementById('out').textContent = 'changed'; }; document.getElementById('out').textContent = String(mql.matches); });</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

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
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: open, close, and print mocks resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>window.open('https://example.test/popup', '_blank', 'noopener'); window.close(); window.print(); document.getElementById('out').textContent = 'done';</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "done");
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().open().calls().len);
    try std.testing.expectEqualStrings(
        "https://example.test/popup",
        subject.mocksMut().open().calls()[0].url.?,
    );
    try std.testing.expectEqualStrings(
        "_blank",
        subject.mocksMut().open().calls()[0].target.?,
    );
    try std.testing.expectEqualStrings(
        "noopener",
        subject.mocksMut().open().calls()[0].features.?,
    );
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().close().calls().len);
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().print().calls().len);
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: print lifecycle handlers resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>window.onbeforeprint = () => { document.getElementById('out').textContent += 'before|'; }; window.onafterprint = () => { document.getElementById('out').textContent += 'after|'; }; window.print(); document.getElementById('out').textContent += 'done';</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "before|after|done");
    try std.testing.expectEqual(@as(usize, 1), subject.mocksMut().print().calls().len);
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 30 collection entries resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><span class='item'>One</span><span class='item'>Two</span><style id='style'>.primary { color: red; }</style><link id='link' rel='stylesheet' href='a.css'><form id='signup'><input type='radio' name='mode' id='mode-a' value='a'><input type='radio' name='mode' id='mode-b' value='b'></form></main><div id='out'></div><script>const nodeEntries = document.querySelectorAll('.item').entries(); const childEntries = document.getElementById('root').children.entries(); const styleEntries = document.styleSheets.entries(); const radioEntries = document.getElementById('signup').elements.namedItem('mode').entries(); document.getElementById('root').textContent = 'gone'; const firstNode = nodeEntries.next(); const secondNode = nodeEntries.next(); const firstChild = childEntries.next(); const secondChild = childEntries.next(); const firstStyle = styleEntries.next(); const firstRadio = radioEntries.next(); const secondRadio = radioEntries.next(); document.getElementById('out').textContent = String(firstNode.value.index) + ':' + firstNode.value.value.textContent + ':' + String(secondNode.value.index) + ':' + secondNode.value.value.textContent + ':' + String(firstChild.value.index) + ':' + firstChild.value.value.textContent + ':' + String(secondChild.value.index) + ':' + secondChild.value.value.textContent + ':' + String(firstStyle.value.index) + ':' + String(firstStyle.value.value) + ':' + String(firstRadio.value.index) + ':' + firstRadio.value.value.value + ':' + String(secondRadio.value.index) + ':' + secondRadio.value.value.value;</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "0:One:1:Two:0:One:1:Two:0:[object CSSStyleSheet]:0:a:1:b");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 31 select.options add and remove resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><select id='mode'><option id='first' value='a'>A</option></select><option id='extra' value='b'>B</option></main><div id='out'></div><script>const select = document.getElementById('mode'); const extra = document.getElementById('extra'); const before = select.options.length; select.options.add(extra); const afterAdd = select.options.length; const entries = select.options.entries(); const firstEntry = entries.next(); select.options.remove(0); document.getElementById('out').textContent = String(before) + ':' + String(afterAdd) + ':' + String(select.options.length) + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + String(select.options.item(0).getAttribute('id')) + ':' + String(select.options.namedItem('first'));</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:2:1:0:first:extra:null");
    try subject.assertExists("#mode > #extra");
    try std.testing.expectError(error.AssertionFailed, subject.assertExists("#first"));
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 31 select.add and select.remove resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><select id='mode'><option id='first' value='a'>A</option></select><option id='extra' value='b'>B</option></main><div id='out'></div><script>const select = document.getElementById('mode'); const extra = document.getElementById('extra'); const before = String(select.length) + ':' + String(select.options.length); select.add(extra); const third = document.createElement('option'); third.id = 'third'; third.value = 'c'; third.textContent = 'C'; select.add(third, 0); const afterAdd = String(select.length) + ':' + String(select.options.length) + ':' + select.options.item(0).getAttribute('id') + ':' + select.options.item(1).getAttribute('id') + ':' + select.options.item(2).getAttribute('id'); select.remove(1); document.getElementById('out').textContent = before + '|' + afterAdd + '|' + String(select.length) + ':' + String(select.options.length) + ':' + select.options.item(0).getAttribute('id') + ':' + select.options.item(1).getAttribute('id');</script>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "1:1|3:3:third:first:extra|2:2:third:extra");
    try subject.assertExists("#mode > #extra");
    try std.testing.expectError(error.AssertionFailed, subject.assertExists("#mode > #first"));
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 32 nth-child of selector lists resolve on the copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><a id='fallback' name='named'>Named</a><section id='list'><span id='a' class='match'>A</span><div id='b'>B</div><span id='c'>C</span><div id='d' class='match'>D</div></section><div id='out'></div><script>document.getElementById('out').textContent = document.querySelector('#list > .match:nth-child(1 of .match)').getAttribute('id') + ':' + document.querySelector('#list > .match:nth-last-child(1 of .match)').getAttribute('id');</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "a:d");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 33 queueMicrotask drains after copied html bootstrap and actions" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><button id='button'>Go</button><div id='out'></div><script>document.getElementById('out').textContent = 'boot'; window.queueMicrotask(() => { document.getElementById('out').textContent = 'booted'; }); document.getElementById('button').addEventListener('click', () => { document.getElementById('out').textContent = 'sync'; window.queueMicrotask(() => { document.getElementById('out').textContent = 'async'; }); });</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "booted");
    try subject.click("#button");
    try subject.assertValue("#out", "async");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 34 timer queue drains on advanceTime for copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>document.getElementById('out').textContent = 'boot'; window.setTimeout(() => { document.getElementById('out').textContent = 'timer'; queueMicrotask(() => { document.getElementById('out').textContent = 'micro'; }); }, 5);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "boot");
    try subject.advanceTime(4);
    try subject.assertValue("#out", "boot");
    try subject.advanceTime(1);
    try subject.assertValue("#out", "micro");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 35 interval timer queue repeats on advanceTime for copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>document.getElementById('out').textContent = 'boot'; const repeating = setInterval(() => { document.getElementById('out').textContent = document.getElementById('out').textContent + 'x'; }, 5); const cancelledByGlobal = setInterval(() => { document.getElementById('out').textContent = 'global'; }, 7); clearInterval(cancelledByGlobal); const cancelledByWindow = window.setInterval(() => { document.getElementById('out').textContent = 'window'; }, 9); window.clearInterval(cancelledByWindow);</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "boot");
    try subject.advanceTime(5);
    try subject.assertValue("#out", "bootx");
    try subject.advanceTime(5);
    try subject.assertValue("#out", "bootxx");
    try std.testing.expectEqualStrings(original, subject.html().?);
}

test "regression: phase 36 requestAnimationFrame queue resolves on copied html snapshot" {
    const allocator = std.testing.allocator;
    const original = "<main id='root'><div id='out'></div><script>document.getElementById('out').textContent = 'boot'; const cancelled = requestAnimationFrame(() => { document.getElementById('out').textContent = 'cancelled'; }); cancelAnimationFrame(cancelled); window.requestAnimationFrame(() => { document.getElementById('out').textContent = document.getElementById('out').textContent + ':raf'; });</script></main>";
    var html_bytes = try allocator.dupe(u8, original);
    defer allocator.free(html_bytes);

    var subject = try Harness.fromHtml(allocator, html_bytes);
    defer subject.deinit();

    html_bytes[1] = 'Z';

    try subject.assertValue("#out", "boot");
    try subject.advanceTime(15);
    try subject.assertValue("#out", "boot");
    try subject.advanceTime(1);
    try subject.assertValue("#out", "boot:raf");
    try std.testing.expectEqualStrings(original, subject.html().?);
}
