const std = @import("std");

const dom = @import("dom.zig");
const errors = @import("errors.zig");
const mocks = @import("mocks.zig");
const script = @import("script.zig");

pub const StorageSeed = struct {
    key: []const u8,
    value: []const u8,
};

pub const SessionConfig = struct {
    url: []const u8,
    html: ?[]const u8 = null,
    local_storage: []const StorageSeed = &.{},
    session_storage: []const StorageSeed = &.{},
    open_failure: ?[]const u8 = null,
    close_failure: ?[]const u8 = null,
    print_failure: ?[]const u8 = null,
    scroll_failure: ?[]const u8 = null,
};

const HistoryEntry = struct {
    url: []const u8,
    state: ?[]const u8 = null,
};

const HistoryModel = struct {
    allocator: std.mem.Allocator,
    entries: std.ArrayListUnmanaged(HistoryEntry) = .{},
    index: usize = 0,

    fn init(allocator: std.mem.Allocator, initial_url: []const u8) errors.Result(HistoryModel) {
        var model = HistoryModel{
            .allocator = allocator,
        };
        try model.entries.append(allocator, .{
            .url = try allocator.dupe(u8, initial_url),
            .state = null,
        });
        model.index = 0;
        return model;
    }

    fn deinit(self: *HistoryModel) void {
        self.entries.deinit(self.allocator);
    }

    fn length(self: *const HistoryModel) usize {
        return self.entries.items.len;
    }

    fn currentState(self: *const HistoryModel) ?[]const u8 {
        return self.entries.items[self.index].state;
    }

    fn current(self: *const HistoryModel) []const u8 {
        return self.entries.items[self.index].url;
    }

    fn copyState(self: *HistoryModel, state: ?[]const u8) errors.Result(?[]const u8) {
        if (state) |value| {
            return try self.allocator.dupe(u8, value);
        }
        return null;
    }

    fn push(self: *HistoryModel, state: ?[]const u8, url: []const u8) errors.Result([]const u8) {
        if (self.index + 1 < self.entries.items.len) {
            self.entries.items.len = self.index + 1;
        }

        const state_copy = try self.copyState(state);
        const copied = try self.allocator.dupe(u8, url);
        try self.entries.append(self.allocator, .{
            .url = copied,
            .state = state_copy,
        });
        self.index = self.entries.items.len - 1;
        return copied;
    }

    fn replace(self: *HistoryModel, state: ?[]const u8, url: []const u8) errors.Result([]const u8) {
        const state_copy = try self.copyState(state);
        const copied = try self.allocator.dupe(u8, url);
        if (self.entries.items.len == 0) {
            try self.entries.append(self.allocator, .{
                .url = copied,
                .state = state_copy,
            });
            self.index = 0;
            return copied;
        }

        self.entries.items[self.index] = .{
            .url = copied,
            .state = state_copy,
        };
        return copied;
    }

    fn go(self: *HistoryModel, delta: isize) ?[]const u8 {
        if (delta == 0) {
            return self.current();
        }

        if (delta < 0) {
            const steps: usize = @intCast(@abs(delta));
            if (steps > self.index) return null;
            self.index -= steps;
            return self.current();
        }

        const steps: usize = @intCast(delta);
        if (self.index + steps >= self.entries.items.len) return null;
        self.index += steps;
        return self.current();
    }
};

pub const Session = struct {
    arena_owner: std.mem.Allocator,
    arena: *std.heap.ArenaAllocator,
    config: SessionConfig,
    dom_store: dom.DomStore,
    script_runtime: script.ScriptRuntime,
    script_event_listeners: std.ArrayListUnmanaged(script.ScriptListenerRecord) = .{},
    mock_registry: mocks.MockRegistry,
    history: HistoryModel,
    clock_ms: i64 = 0,
    scroll_x: i64 = 0,
    scroll_y: i64 = 0,
    window_name: []const u8 = "",

    pub fn init(allocator: std.mem.Allocator, config: SessionConfig) errors.Result(Session) {
        const arena = try allocator.create(std.heap.ArenaAllocator);
        arena.* = std.heap.ArenaAllocator.init(allocator);
        errdefer allocator.destroy(arena);
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
        const session_storage_copy = try arena_alloc.alloc(StorageSeed, config.session_storage.len);
        for (config.session_storage, 0..) |seed, index| {
            session_storage_copy[index] = .{
                .key = try arena_alloc.dupe(u8, seed.key),
                .value = try arena_alloc.dupe(u8, seed.value),
            };
        }

        var dom_store = try dom.DomStore.init(allocator);
        errdefer dom_store.deinit();
        var script_event_listeners: std.ArrayListUnmanaged(script.ScriptListenerRecord) = .{};
        errdefer script_event_listeners.deinit(arena.allocator());
        var mock_registry = mocks.MockRegistry.init(arena_alloc);
        errdefer mock_registry.deinit();
        for (config.local_storage, 0..) |_, index| {
            try mock_registry.storage().seedLocal(
                storage_copy[index].key,
                storage_copy[index].value,
            );
        }
        for (config.session_storage, 0..) |_, index| {
            try mock_registry.storage().seedSession(
                session_storage_copy[index].key,
                session_storage_copy[index].value,
            );
        }
        if (config.open_failure) |message| {
            try mock_registry.open().fail(message);
        }
        if (config.close_failure) |message| {
            try mock_registry.close().fail(message);
        }
        if (config.print_failure) |message| {
            try mock_registry.print().fail(message);
        }
        if (config.scroll_failure) |message| {
            try mock_registry.scroll().fail(message);
        }
        try mock_registry.location().setCurrent(url_copy);
        var history = try HistoryModel.init(arena_alloc, url_copy);
        errdefer history.deinit();
        var window_name: []const u8 = "";
        var scroll_x: i64 = 0;
        var scroll_y: i64 = 0;
        if (html_copy) |html_source| {
            try dom_store.bootstrapHtml(html_source);
        }
        try dom_store.setTargetFragment(fragmentIdentifierFromUrl(url_copy));
        if (html_copy) |html_source| {
            _ = html_source;
            var bootstrap_host = BootstrapHost{
                .dom_store = &dom_store,
                .listeners = &script_event_listeners,
                .location = mock_registry.location(),
                .match_media = mock_registry.matchMedia(),
                .open_mocks = mock_registry.open(),
                .close_mocks = mock_registry.close(),
                .print_mocks = mock_registry.print(),
                .scroll_mocks = mock_registry.scroll(),
                .history = &history,
                .allocator = arena.allocator(),
                .window_name = &window_name,
                .scroll_x = &scroll_x,
                .scroll_y = &scroll_y,
                .storage = mock_registry.storage(),
            };
            try script_runtime.bootstrapInlineScripts(allocator, &bootstrap_host);
        }

        return .{
            .arena_owner = allocator,
            .arena = arena,
            .config = .{
                .url = url_copy,
                .html = html_copy,
                .local_storage = storage_copy,
                .session_storage = session_storage_copy,
            },
            .dom_store = dom_store,
            .script_runtime = script_runtime,
            .script_event_listeners = script_event_listeners,
            .mock_registry = mock_registry,
            .history = history,
            .clock_ms = 0,
            .scroll_x = scroll_x,
            .scroll_y = scroll_y,
            .window_name = window_name,
        };
    }

    pub fn deinit(self: *Session) void {
        self.mock_registry.deinit();
        self.history.deinit();
        self.script_event_listeners.deinit(self.arena.allocator());
        self.script_runtime.deinit();
        self.dom_store.deinit();
        self.arena.deinit();
        self.arena_owner.destroy(self.arena);
    }

    pub fn url(self: *const Session) []const u8 {
        return self.config.url;
    }

    pub fn html(self: *const Session) ?[]const u8 {
        return self.config.html;
    }

    pub fn localStorage(self: *const Session) []const StorageSeed {
        return self.config.local_storage;
    }

    pub fn sessionStorage(self: *const Session) []const StorageSeed {
        return self.config.session_storage;
    }

    pub fn nowMs(self: *const Session) i64 {
        return self.clock_ms;
    }

    pub fn advanceTime(self: *Session, delta_ms: i64) errors.Result(void) {
        if (delta_ms < 0) return error.TimerError;
        self.clock_ms = std.math.add(i64, self.clock_ms, delta_ms) catch return error.TimerError;
        return;
    }

    pub fn flush(self: *Session) errors.Result(void) {
        _ = self;
        return;
    }

    pub fn mocksMut(self: *Session) *mocks.MockRegistry {
        return &self.mock_registry;
    }

    pub fn domStore(self: *const Session) *const dom.DomStore {
        return &self.dom_store;
    }

    pub fn domStoreMut(self: *Session) *dom.DomStore {
        return &self.dom_store;
    }

    pub fn currentLocationUrl(self: *const Session) []const u8 {
        return @constCast(&self.mock_registry).location().currentUrl() orelse self.config.url;
    }

    pub fn assignLocation(self: *Session, url_source: []const u8) errors.Result(void) {
        const trimmed = std.mem.trim(u8, url_source, " \t\r\n");
        if (trimmed.len == 0) return error.MockError;

        const target = try self.history.push(null, trimmed);
        try syncLocationState(self.mock_registry.location(), &self.dom_store, target, true);
        self.resetScrollPosition();
        return;
    }

    pub fn replaceLocation(self: *Session, url_source: []const u8) errors.Result(void) {
        const trimmed = std.mem.trim(u8, url_source, " \t\r\n");
        if (trimmed.len == 0) return error.MockError;

        const target = try self.history.replace(null, trimmed);
        try syncLocationState(self.mock_registry.location(), &self.dom_store, target, true);
        self.resetScrollPosition();
        return;
    }

    pub fn reloadLocation(self: *Session) errors.Result(void) {
        const target = self.history.current();
        try syncLocationState(self.mock_registry.location(), &self.dom_store, target, true);
        self.resetScrollPosition();
        return;
    }

    pub fn historyPushState(self: *Session, state: ?[]const u8, url_source: []const u8) errors.Result(void) {
        const trimmed = std.mem.trim(u8, url_source, " \t\r\n");
        if (trimmed.len == 0) return error.MockError;

        const target = try self.history.push(state, trimmed);
        try syncLocationState(self.mock_registry.location(), &self.dom_store, target, false);
        return;
    }

    pub fn historyReplaceState(self: *Session, state: ?[]const u8, url_source: []const u8) errors.Result(void) {
        const trimmed = std.mem.trim(u8, url_source, " \t\r\n");
        if (trimmed.len == 0) return error.MockError;

        const target = try self.history.replace(state, trimmed);
        try syncLocationState(self.mock_registry.location(), &self.dom_store, target, false);
        return;
    }

    pub fn historyLength(self: *const Session) usize {
        return self.history.length();
    }

    pub fn historyState(self: *const Session) ?[]const u8 {
        return self.history.currentState();
    }

    pub fn historyBack(self: *Session) errors.Result(void) {
        return self.historyGo(-1);
    }

    pub fn historyForward(self: *Session) errors.Result(void) {
        return self.historyGo(1);
    }

    pub fn historyGo(self: *Session, delta: isize) errors.Result(void) {
        if (delta == 0) {
            try self.reloadLocation();
            return;
        }

        if (self.history.go(delta)) |target| {
            try syncLocationState(self.mock_registry.location(), &self.dom_store, target, true);
        }
        return;
    }

    pub fn documentElement(self: *const Session) ?dom.NodeId {
        return self.dom_store.documentElement();
    }

    pub fn documentHead(self: *const Session) ?dom.NodeId {
        return self.dom_store.documentHead();
    }

    pub fn documentBody(self: *const Session) ?dom.NodeId {
        return self.dom_store.documentBody();
    }

    pub fn documentTitle(self: *const Session) []const u8 {
        return self.dom_store.documentTitle();
    }

    pub fn documentCompatMode(self: *const Session) []const u8 {
        _ = self;
        return "CSS1Compat";
    }

    pub fn documentCharacterSet(self: *const Session) []const u8 {
        _ = self;
        return "UTF-8";
    }

    pub fn documentContentType(self: *const Session) []const u8 {
        _ = self;
        return "text/html";
    }

    pub fn documentReferrer(self: *const Session) []const u8 {
        _ = self;
        return "";
    }

    pub fn windowName(self: *const Session) []const u8 {
        return self.window_name;
    }

    pub fn setWindowName(self: *Session, value: []const u8) errors.Result(void) {
        self.window_name = try self.arena.allocator().dupe(u8, value);
        return;
    }

    pub fn windowScrollX(self: *const Session) i64 {
        return self.scroll_x;
    }

    pub fn windowScrollY(self: *const Session) i64 {
        return self.scroll_y;
    }

    pub fn windowPageXOffset(self: *const Session) i64 {
        return self.scroll_x;
    }

    pub fn windowPageYOffset(self: *const Session) i64 {
        return self.scroll_y;
    }

    fn resetScrollPosition(self: *Session) void {
        self.scroll_x = 0;
        self.scroll_y = 0;
    }

    fn storageMap(self: *const Session, target: script.StorageTarget) *std.StringArrayHashMapUnmanaged([]const u8) {
        const storage = @constCast(&self.mock_registry).storage();
        return switch (target) {
            .local => storage.local(),
            .session => storage.session(),
        };
    }

    fn storageMapMut(self: *Session, target: script.StorageTarget) *std.StringArrayHashMapUnmanaged([]const u8) {
        return self.storageMap(target);
    }

    pub fn storageLength(self: *const Session, target: script.StorageTarget) usize {
        return self.storageMap(target).count();
    }

    pub fn storageGetItem(self: *const Session, target: script.StorageTarget, key: []const u8) ?[]const u8 {
        return self.storageMap(target).get(key);
    }

    pub fn storageSetItem(
        self: *Session,
        target: script.StorageTarget,
        key: []const u8,
        value: []const u8,
    ) errors.Result(void) {
        const storage = self.storageMapMut(target);
        if (storage.getPtr(key)) |value_ptr| {
            value_ptr.* = try self.arena.allocator().dupe(u8, value);
            return;
        }

        const key_copy = try self.arena.allocator().dupe(u8, key);
        const value_copy = try self.arena.allocator().dupe(u8, value);
        try storage.put(self.arena.allocator(), key_copy, value_copy);
        return;
    }

    pub fn storageRemoveItem(self: *Session, target: script.StorageTarget, key: []const u8) errors.Result(void) {
        _ = self.storageMapMut(target).orderedRemove(key);
        return;
    }

    pub fn storageClear(self: *Session, target: script.StorageTarget) errors.Result(void) {
        self.storageMapMut(target).clearRetainingCapacity();
        return;
    }

    pub fn storageKey(self: *const Session, target: script.StorageTarget, index: usize) ?[]const u8 {
        const keys = self.storageMap(target).keys();
        if (index >= keys.len) return null;
        return keys[index];
    }

    pub fn documentDir(self: *const Session) []const u8 {
        const document_element = self.dom_store.documentElement() orelse return "";
        return (self.dom_store.getAttribute(document_element, "dir") catch return "") orelse "";
    }

    pub fn setDocumentDir(self: *Session, value: []const u8) errors.Result(void) {
        const document_element = self.dom_store.documentElement() orelse return;
        try self.dom_store.setAttribute(document_element, "dir", value);
        return;
    }

    pub fn documentActiveElement(self: *const Session) ?dom.NodeId {
        return self.dom_store.activeElement();
    }

    pub fn documentReadyState(self: *const Session) []const u8 {
        _ = self;
        return "complete";
    }

    pub fn currentScript(self: *const Session) ?dom.NodeId {
        _ = self;
        return null;
    }

    pub fn setDocumentTitle(self: *Session, value: []const u8) errors.Result(void) {
        try self.dom_store.setDocumentTitle(value);
        return;
    }

    pub fn alert(self: *Session, message: []const u8) errors.Result(void) {
        try self.mock_registry.dialogs().recordAlert(message);
        return;
    }

    pub fn confirm(self: *Session, message: []const u8) errors.Result(bool) {
        const dialogs = self.mock_registry.dialogs();
        try dialogs.recordConfirm(message);
        return dialogs.takeConfirm() orelse error.MockError;
    }

    pub fn prompt(self: *Session, message: []const u8) errors.Result(?[]const u8) {
        const dialogs = self.mock_registry.dialogs();
        try dialogs.recordPrompt(message);
        const response = dialogs.takePrompt() orelse return error.MockError;
        return response;
    }

    pub fn readClipboard(self: *Session) errors.Result([]const u8) {
        return self.mock_registry.clipboard().seededText() orelse error.MockError;
    }

    pub fn writeClipboard(self: *Session, text: []const u8) errors.Result(void) {
        try self.mock_registry.clipboard().recordWrite(text);
        return;
    }

    pub fn open(
        self: *Session,
        url_source: ?[]const u8,
        target: ?[]const u8,
        features: ?[]const u8,
    ) errors.Result(void) {
        try self.mock_registry.open().recordCall(url_source, target, features);
        return;
    }

    pub fn close(self: *Session) errors.Result(void) {
        try self.mock_registry.close().recordCall();
        return;
    }

    pub fn print(self: *Session) errors.Result(void) {
        try self.mock_registry.print().recordCall();
        return;
    }

    pub fn scrollTo(self: *Session, x: i64, y: i64) errors.Result(void) {
        try self.mock_registry.scroll().recordCall(.To, x, y);
        self.scroll_x = x;
        self.scroll_y = y;
        return;
    }

    pub fn scrollBy(self: *Session, x: i64, y: i64) errors.Result(void) {
        try self.mock_registry.scroll().recordCall(.By, x, y);
        self.scroll_x = std.math.add(i64, self.scroll_x, x) catch return error.ScriptRuntime;
        self.scroll_y = std.math.add(i64, self.scroll_y, y) catch return error.ScriptRuntime;
        return;
    }

    pub fn captureDownload(
        self: *Session,
        file_name: []const u8,
        bytes: []const u8,
    ) errors.Result(void) {
        if (std.mem.trim(u8, file_name, " \t\r\n").len == 0) {
            return error.MockError;
        }

        try self.mock_registry.downloads().capture(file_name, bytes);
        return;
    }

    pub fn fetch(self: *Session, url_source: []const u8) errors.Result(mocks.FetchResponse) {
        const trimmed = std.mem.trim(u8, url_source, " \t\r\n");
        if (trimmed.len == 0) return error.MockError;

        const fetch_mocks = self.mock_registry.fetch();
        try fetch_mocks.recordCall(trimmed);

        if (fetch_mocks.findError(trimmed)) |rule| {
            _ = rule;
            return error.MockError;
        }

        if (fetch_mocks.findResponse(trimmed)) |response| {
            return .{
                .url = trimmed,
                .status = response.status,
                .body = response.body,
            };
        }

        return error.MockError;
    }

    pub fn navigate(self: *Session, url_source: []const u8) errors.Result(void) {
        return self.assignLocation(url_source);
    }

    pub fn matchMedia(self: *Session, query_source: []const u8) errors.Result(bool) {
        return matchMediaQuery(self.mock_registry.matchMedia(), query_source);
    }

    pub fn setFilesNode(
        self: *Session,
        node_id: dom.NodeId,
        selector: []const u8,
        files: []const []const u8,
    ) errors.Result(void) {
        try self.dom_store.setFileInputFiles(node_id, files);
        try self.mock_registry.fileInput().setFiles(selector, files);
        _ = try self.dispatchDomEvent(node_id, "input", true, false);
        _ = try self.dispatchDomEvent(node_id, "change", true, false);
        return;
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
        return;
    }

    pub fn dispatchNode(self: *Session, node_id: dom.NodeId, event_type: []const u8) errors.Result(void) {
        const trimmed = std.mem.trim(u8, event_type, " \t\r\n");
        if (trimmed.len == 0) return error.EventError;
        _ = try self.dispatchDomEvent(node_id, trimmed, true, true);
        return;
    }

    pub fn clickNode(self: *Session, node_id: dom.NodeId) errors.Result(void) {
        try self.ensureElementNode(node_id);
        const outcome = try self.dispatchDomEvent(node_id, "click", true, true);
        if (!outcome.default_prevented) {
            try self.runClickDefaultActions(node_id);
        }
        return;
    }

    pub fn typeTextNode(self: *Session, node_id: dom.NodeId, text: []const u8) errors.Result(void) {
        try self.ensureElementNode(node_id);
        try self.dom_store.setFormControlValue(node_id, text);
        _ = try self.dispatchDomEvent(node_id, "input", true, false);
        return;
    }

    pub fn setCheckedNode(self: *Session, node_id: dom.NodeId, checked: bool) errors.Result(void) {
        try self.ensureElementNode(node_id);
        try self.dom_store.setFormControlChecked(node_id, checked);
        _ = try self.dispatchDomEvent(node_id, "input", true, false);
        _ = try self.dispatchDomEvent(node_id, "change", true, false);
        return;
    }

    pub fn setSelectValueNode(self: *Session, node_id: dom.NodeId, value: []const u8) errors.Result(void) {
        try self.ensureElementNode(node_id);
        try self.dom_store.setSelectValue(node_id, value);
        _ = try self.dispatchDomEvent(node_id, "input", true, false);
        _ = try self.dispatchDomEvent(node_id, "change", true, false);
        return;
    }

    pub fn focusNode(self: *Session, node_id: dom.NodeId) errors.Result(void) {
        try self.ensureElementNode(node_id);
        if (self.dom_store.focusedNode()) |focused| {
            if (std.meta.eql(focused, node_id)) {
                return;
            }
        }

        if (self.dom_store.focusedNode()) |previous| {
            self.dom_store.setFocusedNode(null);
            _ = try self.dispatchDomEvent(previous, "blur", false, false);
        }

        self.dom_store.setFocusedNode(node_id);
        _ = try self.dispatchDomEvent(node_id, "focus", false, false);
        return;
    }

    pub fn blurNode(self: *Session, node_id: dom.NodeId) errors.Result(void) {
        try self.ensureElementNode(node_id);
        if (self.dom_store.focusedNode()) |focused| {
            if (!std.meta.eql(focused, node_id)) {
                return;
            }
        } else {
            return;
        }

        self.dom_store.setFocusedNode(null);
        _ = try self.dispatchDomEvent(node_id, "blur", false, false);
        return;
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
        return;
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

        return;
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
        return;
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

        return;
    }
};

const BootstrapHost = struct {
    dom_store: *dom.DomStore,
    listeners: *std.ArrayListUnmanaged(script.ScriptListenerRecord),
    location: *mocks.LocationMocks,
    match_media: *mocks.MatchMediaMocks,
    open_mocks: *mocks.OpenMocks,
    close_mocks: *mocks.CloseMocks,
    print_mocks: *mocks.PrintMocks,
    scroll_mocks: *mocks.ScrollMocks,
    history: *HistoryModel,
    allocator: std.mem.Allocator,
    window_name: *[]const u8,
    scroll_x: *i64,
    scroll_y: *i64,
    storage: *mocks.StorageSeeds,
    current_script: ?dom.NodeId = null,

    pub fn domStore(self: *const BootstrapHost) *const dom.DomStore {
        return self.dom_store;
    }

    pub fn domStoreMut(self: *BootstrapHost) *dom.DomStore {
        return self.dom_store;
    }

    pub fn currentLocationUrl(self: *const BootstrapHost) []const u8 {
        return self.location.currentUrl() orelse "";
    }

    pub fn documentElement(self: *const BootstrapHost) ?dom.NodeId {
        return self.dom_store.documentElement();
    }

    pub fn documentHead(self: *const BootstrapHost) ?dom.NodeId {
        return self.dom_store.documentHead();
    }

    pub fn documentBody(self: *const BootstrapHost) ?dom.NodeId {
        return self.dom_store.documentBody();
    }

    pub fn documentTitle(self: *const BootstrapHost) []const u8 {
        return self.dom_store.documentTitle();
    }

    pub fn documentCompatMode(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "CSS1Compat";
    }

    pub fn documentCharacterSet(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "UTF-8";
    }

    pub fn documentContentType(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "text/html";
    }

    pub fn documentReferrer(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "";
    }

    pub fn windowName(self: *const BootstrapHost) []const u8 {
        return self.window_name.*;
    }

    pub fn setWindowName(self: *BootstrapHost, value: []const u8) errors.Result(void) {
        self.window_name.* = try self.allocator.dupe(u8, value);
        return;
    }

    pub fn windowScrollX(self: *const BootstrapHost) i64 {
        return self.scroll_x.*;
    }

    pub fn windowScrollY(self: *const BootstrapHost) i64 {
        return self.scroll_y.*;
    }

    pub fn windowPageXOffset(self: *const BootstrapHost) i64 {
        return self.scroll_x.*;
    }

    pub fn windowPageYOffset(self: *const BootstrapHost) i64 {
        return self.scroll_y.*;
    }

    fn resetScrollPosition(self: *BootstrapHost) void {
        self.scroll_x.* = 0;
        self.scroll_y.* = 0;
    }

    fn storageMap(self: *const BootstrapHost, target: script.StorageTarget) *std.StringArrayHashMapUnmanaged([]const u8) {
        return switch (target) {
            .local => self.storage.local(),
            .session => self.storage.session(),
        };
    }

    fn storageMapMut(self: *BootstrapHost, target: script.StorageTarget) *std.StringArrayHashMapUnmanaged([]const u8) {
        return self.storageMap(target);
    }

    pub fn storageLength(self: *const BootstrapHost, target: script.StorageTarget) usize {
        return self.storageMap(target).count();
    }

    pub fn storageGetItem(self: *const BootstrapHost, target: script.StorageTarget, key: []const u8) ?[]const u8 {
        return self.storageMap(target).get(key);
    }

    pub fn storageSetItem(
        self: *BootstrapHost,
        target: script.StorageTarget,
        key: []const u8,
        value: []const u8,
    ) errors.Result(void) {
        const storage = self.storageMapMut(target);
        if (storage.getPtr(key)) |value_ptr| {
            value_ptr.* = try self.allocator.dupe(u8, value);
            return;
        }

        const key_copy = try self.allocator.dupe(u8, key);
        const value_copy = try self.allocator.dupe(u8, value);
        try storage.put(self.allocator, key_copy, value_copy);
        return;
    }

    pub fn storageRemoveItem(self: *BootstrapHost, target: script.StorageTarget, key: []const u8) errors.Result(void) {
        _ = self.storageMapMut(target).orderedRemove(key);
        return;
    }

    pub fn storageClear(self: *BootstrapHost, target: script.StorageTarget) errors.Result(void) {
        self.storageMapMut(target).clearRetainingCapacity();
        return;
    }

    pub fn storageKey(self: *const BootstrapHost, target: script.StorageTarget, index: usize) ?[]const u8 {
        const keys = self.storageMap(target).keys();
        if (index >= keys.len) return null;
        return keys[index];
    }

    pub fn documentDir(self: *const BootstrapHost) []const u8 {
        const document_element = self.dom_store.documentElement() orelse return "";
        return (self.dom_store.getAttribute(document_element, "dir") catch return "") orelse "";
    }

    pub fn setDocumentDir(self: *BootstrapHost, value: []const u8) errors.Result(void) {
        const document_element = self.dom_store.documentElement() orelse return;
        try self.dom_store.setAttribute(document_element, "dir", value);
        return;
    }

    pub fn documentActiveElement(self: *const BootstrapHost) ?dom.NodeId {
        return self.dom_store.activeElement();
    }

    pub fn documentReadyState(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "loading";
    }

    pub fn currentScript(self: *const BootstrapHost) ?dom.NodeId {
        return self.current_script;
    }

    pub fn setCurrentScript(self: *BootstrapHost, current_script: ?dom.NodeId) void {
        self.current_script = current_script;
    }

    pub fn setDocumentTitle(self: *BootstrapHost, value: []const u8) errors.Result(void) {
        try self.dom_store.setDocumentTitle(value);
        return;
    }

    pub fn assignLocation(self: *BootstrapHost, url_source: []const u8) errors.Result(void) {
        const trimmed = std.mem.trim(u8, url_source, " \t\r\n");
        if (trimmed.len == 0) return error.MockError;

        const target = try self.history.push(null, trimmed);
        try syncLocationState(self.location, self.dom_store, target, true);
        self.resetScrollPosition();
        return;
    }

    pub fn replaceLocation(self: *BootstrapHost, url_source: []const u8) errors.Result(void) {
        const trimmed = std.mem.trim(u8, url_source, " \t\r\n");
        if (trimmed.len == 0) return error.MockError;

        const target = try self.history.replace(null, trimmed);
        try syncLocationState(self.location, self.dom_store, target, true);
        self.resetScrollPosition();
        return;
    }

    pub fn reloadLocation(self: *BootstrapHost) errors.Result(void) {
        const target = self.history.current();
        try syncLocationState(self.location, self.dom_store, target, true);
        self.resetScrollPosition();
        return;
    }

    pub fn historyLength(self: *const BootstrapHost) usize {
        return self.history.length();
    }

    pub fn historyState(self: *const BootstrapHost) ?[]const u8 {
        return self.history.currentState();
    }

    pub fn historyBack(self: *BootstrapHost) errors.Result(void) {
        return self.historyGo(-1);
    }

    pub fn historyForward(self: *BootstrapHost) errors.Result(void) {
        return self.historyGo(1);
    }

    pub fn historyGo(self: *BootstrapHost, delta: isize) errors.Result(void) {
        if (delta == 0) {
            try self.reloadLocation();
            return;
        }

        if (self.history.go(delta)) |target| {
            try syncLocationState(self.location, self.dom_store, target, true);
        }
        return;
    }

    pub fn historyPushState(self: *BootstrapHost, state: ?[]const u8, url_source: []const u8) errors.Result(void) {
        const trimmed = std.mem.trim(u8, url_source, " \t\r\n");
        if (trimmed.len == 0) return error.MockError;

        const target = try self.history.push(state, trimmed);
        try syncLocationState(self.location, self.dom_store, target, false);
        return;
    }

    pub fn historyReplaceState(self: *BootstrapHost, state: ?[]const u8, url_source: []const u8) errors.Result(void) {
        const trimmed = std.mem.trim(u8, url_source, " \t\r\n");
        if (trimmed.len == 0) return error.MockError;

        const target = try self.history.replace(state, trimmed);
        try syncLocationState(self.location, self.dom_store, target, false);
        return;
    }

    pub fn navigate(self: *BootstrapHost, url_source: []const u8) errors.Result(void) {
        return self.assignLocation(url_source);
    }

    pub fn matchMedia(self: *BootstrapHost, query_source: []const u8) errors.Result(bool) {
        return matchMediaQuery(self.match_media, query_source);
    }

    pub fn open(
        self: *BootstrapHost,
        url_source: ?[]const u8,
        target: ?[]const u8,
        features: ?[]const u8,
    ) errors.Result(void) {
        try self.open_mocks.recordCall(url_source, target, features);
        return;
    }

    pub fn close(self: *BootstrapHost) errors.Result(void) {
        try self.close_mocks.recordCall();
        return;
    }

    pub fn print(self: *BootstrapHost) errors.Result(void) {
        try self.print_mocks.recordCall();
        return;
    }

    pub fn scrollTo(self: *BootstrapHost, x: i64, y: i64) errors.Result(void) {
        try self.scroll_mocks.recordCall(.To, x, y);
        self.scroll_x.* = x;
        self.scroll_y.* = y;
        return;
    }

    pub fn scrollBy(self: *BootstrapHost, x: i64, y: i64) errors.Result(void) {
        try self.scroll_mocks.recordCall(.By, x, y);
        self.scroll_x.* = std.math.add(i64, self.scroll_x.*, x) catch return error.ScriptRuntime;
        self.scroll_y.* = std.math.add(i64, self.scroll_y.*, y) catch return error.ScriptRuntime;
        return;
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
        return;
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
    return;
}

fn matchMediaQuery(
    media_mocks: *mocks.MatchMediaMocks,
    query_source: []const u8,
) errors.Result(bool) {
    try media_mocks.recordCall(query_source);

    if (media_mocks.findRule(query_source)) |rule| {
        if (rule.is_failure) {
            return error.MockError;
        }

        return rule.matches;
    }

    return error.MockError;
}

fn syncLocationState(
    location: *mocks.LocationMocks,
    dom_store: *dom.DomStore,
    url: []const u8,
    record_navigation: bool,
) errors.Result(void) {
    try location.setCurrent(url);
    if (record_navigation) {
        try location.recordNavigation(url);
    }
    try dom_store.setTargetFragment(fragmentIdentifierFromUrl(url));
    return;
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

fn fragmentIdentifierFromUrl(url: []const u8) ?[]const u8 {
    const fragment_index = std.mem.indexOfScalar(u8, url, '#') orelse return null;
    if (fragment_index + 1 >= url.len) return null;
    return url[fragment_index + 1 ..];
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
