const std = @import("std");

const dom = @import("dom.zig");
const errors = @import("errors.zig");
const script = @import("script.zig");

pub const StorageSeed = struct {
    key: []const u8,
    value: []const u8,
};

pub const SessionConfig = struct {
    url: []const u8,
    html: ?[]const u8 = null,
    local_storage: []const StorageSeed = &.{},
};

pub const Session = struct {
    arena: std.heap.ArenaAllocator,
    config: SessionConfig,
    dom_store: dom.DomStore,
    script_runtime: script.ScriptRuntime,
    script_event_listeners: std.ArrayListUnmanaged(script.ScriptListenerRecord) = .{},
    focused_node: ?dom.NodeId = null,

    pub fn init(allocator: std.mem.Allocator, config: SessionConfig) errors.Result(Session) {
        var arena = std.heap.ArenaAllocator.init(allocator);
        errdefer arena.deinit();

        var script_runtime = script.ScriptRuntime.init();
        errdefer script_runtime.deinit();

        const arena_alloc = arena.allocator();
        const url_copy = try arena_alloc.dupe(u8, config.url);
        const html_copy = if (config.html) |html_source|
            try arena_alloc.dupe(u8, html_source)
        else
            null;

        const storage_copy = try arena_alloc.alloc(StorageSeed, config.local_storage.len);
        for (config.local_storage, 0..) |seed, index| {
            storage_copy[index] = .{
                .key = try arena_alloc.dupe(u8, seed.key),
                .value = try arena_alloc.dupe(u8, seed.value),
            };
        }

        var dom_store = try dom.DomStore.init(allocator);
        errdefer dom_store.deinit();
        var script_event_listeners: std.ArrayListUnmanaged(script.ScriptListenerRecord) = .{};
        errdefer script_event_listeners.deinit(arena.allocator());
        if (html_copy) |html_source| {
            try dom_store.bootstrapHtml(html_source);
            var bootstrap_host = BootstrapHost{
                .dom_store = &dom_store,
                .listeners = &script_event_listeners,
                .allocator = arena.allocator(),
            };
            try script_runtime.bootstrapInlineScripts(allocator, bootstrap_host);
        }

        return .{
            .arena = arena,
            .config = .{
                .url = url_copy,
                .html = html_copy,
                .local_storage = storage_copy,
            },
            .dom_store = dom_store,
            .script_runtime = script_runtime,
            .script_event_listeners = script_event_listeners,
            .focused_node = null,
        };
    }

    pub fn deinit(self: *Session) void {
        self.script_event_listeners.deinit(self.arena.allocator());
        self.script_runtime.deinit();
        self.dom_store.deinit();
        self.arena.deinit();
    }

    pub fn url(self: Session) []const u8 {
        return self.config.url;
    }

    pub fn html(self: Session) ?[]const u8 {
        return self.config.html;
    }

    pub fn localStorage(self: Session) []const StorageSeed {
        return self.config.local_storage;
    }

    pub fn domStore(self: *const Session) *const dom.DomStore {
        return &self.dom_store;
    }

    pub fn domStoreMut(self: *Session) *dom.DomStore {
        return &self.dom_store;
    }

    pub fn registerEventListener(
        self: *Session,
        target: script.ListenerTarget,
        event_type: []const u8,
        capture: bool,
        handler: script.ScriptFunction,
    ) errors.Result(void) {
        try appendScriptListener(
            self.arena.allocator(),
            &self.script_event_listeners,
            target,
            event_type,
            capture,
            handler,
        );
    }

    pub fn dispatchNode(self: *Session, node_id: dom.NodeId, event_type: []const u8) errors.Result(void) {
        const trimmed = std.mem.trim(u8, event_type, " \t\r\n");
        if (trimmed.len == 0) return error.EventError;
        _ = try self.dispatchDomEvent(node_id, trimmed, true, true);
    }

    pub fn clickNode(self: *Session, node_id: dom.NodeId) errors.Result(void) {
        try self.ensureElementNode(node_id);
        const outcome = try self.dispatchDomEvent(node_id, "click", true, true);
        if (!outcome.default_prevented) {
            try self.runClickDefaultActions(node_id);
        }
    }

    pub fn typeTextNode(self: *Session, node_id: dom.NodeId, text: []const u8) errors.Result(void) {
        try self.ensureElementNode(node_id);
        try self.dom_store.setFormControlValue(node_id, text);
        _ = try self.dispatchDomEvent(node_id, "input", true, false);
    }

    pub fn setCheckedNode(self: *Session, node_id: dom.NodeId, checked: bool) errors.Result(void) {
        try self.ensureElementNode(node_id);
        try self.dom_store.setFormControlChecked(node_id, checked);
        _ = try self.dispatchDomEvent(node_id, "input", true, false);
        _ = try self.dispatchDomEvent(node_id, "change", true, false);
    }

    pub fn setSelectValueNode(self: *Session, node_id: dom.NodeId, value: []const u8) errors.Result(void) {
        try self.ensureElementNode(node_id);
        try self.dom_store.setSelectValue(node_id, value);
        _ = try self.dispatchDomEvent(node_id, "input", true, false);
        _ = try self.dispatchDomEvent(node_id, "change", true, false);
    }

    pub fn focusNode(self: *Session, node_id: dom.NodeId) errors.Result(void) {
        try self.ensureElementNode(node_id);
        if (self.focused_node == node_id) {
            return;
        }

        if (self.focused_node) |previous| {
            _ = try self.dispatchDomEvent(previous, "blur", false, false);
        }

        self.focused_node = node_id;
        _ = try self.dispatchDomEvent(node_id, "focus", false, false);
    }

    pub fn blurNode(self: *Session, node_id: dom.NodeId) errors.Result(void) {
        try self.ensureElementNode(node_id);
        if (self.focused_node != node_id) {
            return;
        }

        self.focused_node = null;
        _ = try self.dispatchDomEvent(node_id, "blur", false, false);
    }

    pub fn submitNode(self: *Session, node_id: dom.NodeId) errors.Result(void) {
        try self.ensureElementNode(node_id);
        const node = self.dom_store.nodeAt(node_id) orelse return error.DomError;
        if (isFormNode(node)) {
            _ = try self.dispatchDomEvent(node_id, "submit", true, true);
            return;
        }

        const form_id = self.findAssociatedForm(node_id) orelse return error.DomError;
        _ = try self.dispatchDomEvent(form_id, "submit", true, true);
    }

    fn dispatchDomEvent(
        self: *Session,
        node_id: dom.NodeId,
        event_type: []const u8,
        bubbles: bool,
        cancelable: bool,
    ) errors.Result(DispatchOutcome) {
        try self.ensureElementNode(node_id);

        var event = script.ScriptEvent{
            .event_type = event_type,
            .target = .{ .element = node_id },
            .current_target = null,
            .bubbles = bubbles,
            .cancelable = cancelable,
        };
        const allocator = std.heap.page_allocator;
        var ancestors: std.ArrayList(script.ListenerTarget) = .empty;
        defer ancestors.deinit(allocator);
        try self.eventAncestorTargets(allocator, node_id, &ancestors);

        for (ancestors.items[0..]) |target| {
            // reverse iteration over the ancestor chain for capture listeners
            _ = target;
        }

        var index: usize = ancestors.items.len;
        while (index > 0) {
            index -= 1;
            try self.runEventListeners(allocator, ancestors.items[index], event_type, true, .capturing, &event);
            if (event.immediate_propagation_stopped or event.propagation_stopped) {
                event.current_target = null;
                event.phase = .none;
                return .{ .default_prevented = event.default_prevented };
            }
        }

        try self.runEventListeners(allocator, .{ .element = node_id }, event_type, true, .at_target, &event);
        if (event.immediate_propagation_stopped) {
            event.current_target = null;
            event.phase = .none;
            return .{ .default_prevented = event.default_prevented };
        }

        try self.runEventListeners(allocator, .{ .element = node_id }, event_type, false, .at_target, &event);
        if (event.immediate_propagation_stopped or event.propagation_stopped) {
            event.current_target = null;
            event.phase = .none;
            return .{ .default_prevented = event.default_prevented };
        }

        if (bubbles) {
            for (ancestors.items) |target| {
                try self.runEventListeners(allocator, target, event_type, false, .bubbling, &event);
                if (event.immediate_propagation_stopped or event.propagation_stopped) {
                    break;
                }
            }
        }

        event.current_target = null;
        event.phase = .none;
        return .{ .default_prevented = event.default_prevented };
    }

    fn runEventListeners(
        self: *Session,
        allocator: std.mem.Allocator,
        target: script.ListenerTarget,
        event_type: []const u8,
        capture: bool,
        phase: script.EventPhase,
        event: *script.ScriptEvent,
    ) errors.Result(void) {
        var listeners: std.ArrayList(script.ScriptListenerRecord) = .empty;
        defer listeners.deinit(allocator);

        for (self.script_event_listeners.items) |listener| {
            if (!std.meta.eql(listener.target, target)) continue;
            if (!std.mem.eql(u8, listener.event_type, event_type)) continue;
            if (listener.capture != capture) continue;
            try listeners.append(allocator, listener);
        }

        for (listeners.items, 0..) |listener, index| {
            if (event.immediate_propagation_stopped) break;

            event.current_target = target;
            event.phase = phase;

            var bindings = try self.listenerBindings(allocator, listener.handler, event);
            defer bindings.deinit(allocator);

            const source_name = try std.fmt.allocPrint(allocator, "event:{s}:{d}", .{ event_type, index });
            defer allocator.free(source_name);

            try self.script_runtime.evalScriptSourceWithBindings(
                allocator,
                self,
                listener.handler.body_source,
                source_name,
                bindings.items,
            );
        }
    }

    fn listenerBindings(
        self: *Session,
        allocator: std.mem.Allocator,
        handler: script.ScriptFunction,
        event: *script.ScriptEvent,
    ) errors.Result(std.ArrayList(script.Binding)) {
        _ = self;
        var bindings: std.ArrayList(script.Binding) = .empty;
        errdefer bindings.deinit(allocator);
        try bindings.append(allocator, .{
            .name = "event",
            .value = .{ .event = event },
        });

        for (handler.params, 0..) |param, index| {
            try bindings.append(allocator, .{
                .name = param,
                .value = if (index == 0)
                    .{ .event = event }
                else
                    .{ .undefined_value = {} },
            });
        }

        return bindings;
    }

    fn eventAncestorTargets(
        self: *Session,
        allocator: std.mem.Allocator,
        node_id: dom.NodeId,
        output: *std.ArrayList(script.ListenerTarget),
    ) errors.Result(void) {
        var current = self.dom_store.nodeAt(node_id).?.parent;
        while (current) |parent_id| {
            const parent = self.dom_store.nodeAt(parent_id) orelse break;
            switch (parent.kind) {
                .document => {
                    try output.append(allocator, .document);
                    break;
                },
                else => {
                    try output.append(allocator, .{ .element = parent_id });
                    current = parent.parent;
                },
            }
        }

        try output.append(allocator, .window);
    }

    fn ensureElementNode(self: *Session, node_id: dom.NodeId) errors.Result(void) {
        const node = self.dom_store.nodeAt(node_id) orelse return error.DomError;
        switch (node.kind) {
            .element => return,
            else => return error.DomError,
        }
    }

    fn findAssociatedForm(self: *Session, node_id: dom.NodeId) ?dom.NodeId {
        var current = self.dom_store.nodeAt(node_id).?.parent;
        while (current) |parent_id| {
            const parent = self.dom_store.nodeAt(parent_id) orelse break;
            if (isFormNode(parent)) return parent_id;
            current = parent.parent;
        }

        return null;
    }

    fn runClickDefaultActions(self: *Session, node_id: dom.NodeId) errors.Result(void) {
        const node = self.dom_store.nodeAt(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return error.DomError,
        };

        const input_type = elementAttributeValue(element, "type");
        if (std.mem.eql(u8, element.tag_name, "input") and isCheckableInputType(input_type)) {
            const checked = self.dom_store.checkedForNode(node_id) orelse false;
            try self.dom_store.setFormControlChecked(node_id, !checked);
            _ = try self.dispatchDomEvent(node_id, "input", true, false);
            _ = try self.dispatchDomEvent(node_id, "change", true, false);
        }

        if (isSubmitControl(element.tag_name, input_type)) {
            if (self.findAssociatedForm(node_id)) |form_id| {
                _ = try self.dispatchDomEvent(form_id, "submit", true, true);
            }
        }
    }

    fn domStore(self: *const Session) *const dom.DomStore {
        return &self.dom_store;
    }
};

const BootstrapHost = struct {
    dom_store: *dom.DomStore,
    listeners: *std.ArrayListUnmanaged(script.ScriptListenerRecord),
    allocator: std.mem.Allocator,

    pub fn domStore(self: *const BootstrapHost) *const dom.DomStore {
        return self.dom_store;
    }

    pub fn domStoreMut(self: *BootstrapHost) *dom.DomStore {
        return self.dom_store;
    }

    pub fn registerEventListener(
        self: *BootstrapHost,
        target: script.ListenerTarget,
        event_type: []const u8,
        capture: bool,
        handler: script.ScriptFunction,
    ) errors.Result(void) {
        try appendScriptListener(
            self.allocator,
            self.listeners,
            target,
            event_type,
            capture,
            handler,
        );
    }
};

const DispatchOutcome = struct {
    default_prevented: bool,
};

fn appendScriptListener(
    allocator: std.mem.Allocator,
    listeners: *std.ArrayListUnmanaged(script.ScriptListenerRecord),
    target: script.ListenerTarget,
    event_type: []const u8,
    capture: bool,
    handler: script.ScriptFunction,
) errors.Result(void) {
    const handler_copy = try duplicateScriptFunction(allocator, handler);
    const event_type_copy = try allocator.dupe(u8, event_type);
    try listeners.append(allocator, .{
        .target = target,
        .event_type = event_type_copy,
        .capture = capture,
        .handler = handler_copy,
    });
}

fn duplicateScriptFunction(
    allocator: std.mem.Allocator,
    handler: script.ScriptFunction,
) errors.Result(script.ScriptFunction) {
    const params = try allocator.alloc([]const u8, handler.params.len);
    for (handler.params, 0..) |param, index| {
        params[index] = try allocator.dupe(u8, param);
    }

    return .{
        .params = params,
        .body_source = try allocator.dupe(u8, handler.body_source),
    };
}

fn elementAttributeValue(element: dom.ElementData, name: []const u8) ?[]const u8 {
    for (element.attributes.items) |attribute| {
        if (std.mem.eql(u8, attribute.name, name)) return attribute.value;
    }
    return null;
}

fn isFormNode(node: *const dom.NodeRecord) bool {
    return switch (node.kind) {
        .element => |element| std.mem.eql(u8, element.tag_name, "form"),
        else => false,
    };
}

fn isCheckableInputType(input_type: ?[]const u8) bool {
    const value = input_type orelse "text";
    return std.mem.eql(u8, value, "checkbox") or std.mem.eql(u8, value, "radio");
}

fn isSubmitControl(tag_name: []const u8, input_type: ?[]const u8) bool {
    if (std.mem.eql(u8, tag_name, "button")) {
        return !std.mem.eql(u8, input_type orelse "submit", "button") and !std.mem.eql(u8, input_type orelse "submit", "reset");
    }

    if (std.mem.eql(u8, tag_name, "input")) {
        const value = input_type orelse "text";
        return std.mem.eql(u8, value, "submit") or std.mem.eql(u8, value, "image");
    }

    return false;
}

test "session boots html into the dom store" {
    const allocator = std.testing.allocator;
    var subject = try Session.init(allocator, .{
        .url = "https://app.local/",
        .html = "<main id='app'><span>Hello</span></main>",
        .local_storage = &.{},
    });
    defer subject.deinit();

    try std.testing.expectEqual(@as(usize, 4), subject.domStore().nodeCount());

    const dump = try subject.domStore().dumpDom(allocator);
    defer allocator.free(dump);

    try std.testing.expectEqualStrings(
        "#document\n  <main id=\"app\">\n    <span>\n      \"Hello\"\n    </span>\n  </main>\n",
        dump,
    );
}

test "session boots inline scripts into the dom store" {
    const allocator = std.testing.allocator;
    var subject = try Session.init(allocator, .{
        .url = "https://app.local/",
        .html = "<main id='out'>Before</main><script>document.getElementById('out').textContent = 'Hello';</script>",
        .local_storage = &.{},
    });
    defer subject.deinit();

    const dump = try subject.domStore().dumpDom(allocator);
    defer allocator.free(dump);

    try std.testing.expectEqualStrings(
        "#document\n  <main id=\"out\">\n    \"Hello\"\n  </main>\n  <script>\n    \"document.getElementById('out').textContent = 'Hello';\"\n  </script>\n",
        dump,
    );
}
