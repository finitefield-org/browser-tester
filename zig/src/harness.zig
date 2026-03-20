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
