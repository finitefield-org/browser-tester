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
    random_seed: ?u64 = null,
    open_failure: ?[]const u8 = null,
    close_failure: ?[]const u8 = null,
    print_failure: ?[]const u8 = null,
    scroll_failure: ?[]const u8 = null,
};

const HistoryEntry = struct {
    url: []const u8,
    state: ?[]const u8 = null,
};

const scheduler_step_limit: usize = 10_000;

const ScheduledTimer = struct {
    id: u64,
    at_ms: i64,
    interval_ms: ?i64 = null,
    kind: TimerKind = .timeout,
    handler: script.ScriptFunction,
};

const TimerKind = enum {
    timeout,
    interval,
    animation_frame,
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
    match_media_listeners: std.ArrayListUnmanaged(script.MatchMediaListenerRecord) = .{},
    match_media_onchange_listeners: std.ArrayListUnmanaged(script.MatchMediaListenerRecord) = .{},
    queued_microtasks: std.ArrayListUnmanaged(script.ScriptFunction) = .{},
    timers: std.ArrayListUnmanaged(ScheduledTimer) = .{},
    next_timer_id: u64 = 1,
    active_timer_id: ?u64 = null,
    active_timer_cancelled: bool = false,
    mock_registry: mocks.MockRegistry,
    history: HistoryModel,
    history_scroll_restoration: []const u8 = "auto",
    math_random_state: u64 = 1,
    crypto_random_state: u64 = 1,
    document_write_buffer: std.ArrayListUnmanaged(u8) = .{},
    clock_ms: i64 = 0,
    scroll_x: i64 = 0,
    scroll_y: i64 = 0,
    window_name: []const u8 = "",
    window_load_handler: ?script.ScriptFunction = null,
    window_beforeprint_handler: ?script.ScriptFunction = null,
    window_afterprint_handler: ?script.ScriptFunction = null,
    window_beforeunload_handler: ?script.ScriptFunction = null,
    window_unload_handler: ?script.ScriptFunction = null,
    window_focus_handler: ?script.ScriptFunction = null,
    window_blur_handler: ?script.ScriptFunction = null,
    window_pageshow_handler: ?script.ScriptFunction = null,
    window_pagehide_handler: ?script.ScriptFunction = null,
    document_scroll_handler: ?script.ScriptFunction = null,
    document_selectionchange_handler: ?script.ScriptFunction = null,
    document_readystatechange_handler: ?script.ScriptFunction = null,
    window_hashchange_handler: ?script.ScriptFunction = null,
    window_popstate_handler: ?script.ScriptFunction = null,
    window_storage_handler: ?script.ScriptFunction = null,
    window_scroll_handler: ?script.ScriptFunction = null,
    document_ready_state: []const u8 = "complete",
    document_ready_state_change_pending: bool = true,
    cookie_jar: std.StringArrayHashMapUnmanaged([]const u8) = .empty,

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
        var match_media_listeners: std.ArrayListUnmanaged(script.MatchMediaListenerRecord) = .{};
        errdefer match_media_listeners.deinit(arena.allocator());
        var match_media_onchange_listeners: std.ArrayListUnmanaged(script.MatchMediaListenerRecord) = .{};
        errdefer match_media_onchange_listeners.deinit(arena.allocator());
        var queued_microtasks: std.ArrayListUnmanaged(script.ScriptFunction) = .{};
        errdefer queued_microtasks.deinit(arena.allocator());
        var timers: std.ArrayListUnmanaged(ScheduledTimer) = .{};
        errdefer timers.deinit(arena.allocator());
        var cookie_jar: std.StringArrayHashMapUnmanaged([]const u8) = .empty;
        errdefer cookie_jar.deinit(arena_alloc);
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
        var history_scroll_restoration: []const u8 = "auto";
        var math_random_state: u64 = config.random_seed orelse 1;
        var crypto_random_state: u64 = config.random_seed orelse 1;
        var window_name: []const u8 = "";
        var window_hashchange_handler: ?script.ScriptFunction = null;
        var scroll_x: i64 = 0;
        var scroll_y: i64 = 0;
        var document_write_buffer: std.ArrayListUnmanaged(u8) = .{};
        var next_timer_id: u64 = 1;
        var window_load_handler: ?script.ScriptFunction = null;
        var window_beforeprint_handler: ?script.ScriptFunction = null;
        var window_afterprint_handler: ?script.ScriptFunction = null;
        var window_beforeunload_handler: ?script.ScriptFunction = null;
        var window_unload_handler: ?script.ScriptFunction = null;
        var window_focus_handler: ?script.ScriptFunction = null;
        var window_blur_handler: ?script.ScriptFunction = null;
        var window_pageshow_handler: ?script.ScriptFunction = null;
        var window_pagehide_handler: ?script.ScriptFunction = null;
        var document_scroll_handler: ?script.ScriptFunction = null;
        var document_selectionchange_handler: ?script.ScriptFunction = null;
        var document_readystatechange_handler: ?script.ScriptFunction = null;
        var window_popstate_handler: ?script.ScriptFunction = null;
        var window_storage_handler: ?script.ScriptFunction = null;
        var window_scroll_handler: ?script.ScriptFunction = null;
        var document_ready_state: []const u8 = "loading";
        var document_ready_state_change_pending: bool = true;
        var bootstrap_host = BootstrapHost{
            .dom_store = &dom_store,
            .listeners = &script_event_listeners,
            .match_media_listeners = &match_media_listeners,
            .match_media_onchange_listeners = &match_media_onchange_listeners,
            .location = mock_registry.location(),
            .match_media = mock_registry.matchMedia(),
            .open_mocks = mock_registry.open(),
            .close_mocks = mock_registry.close(),
            .print_mocks = mock_registry.print(),
            .scroll_mocks = mock_registry.scroll(),
            .download_mocks = mock_registry.downloads(),
            .history = &history,
            .allocator = arena.allocator(),
            .window_name = &window_name,
            .window_load_handler = &window_load_handler,
            .window_beforeprint_handler = &window_beforeprint_handler,
            .window_afterprint_handler = &window_afterprint_handler,
            .window_beforeunload_handler = &window_beforeunload_handler,
            .window_unload_handler = &window_unload_handler,
            .window_focus_handler = &window_focus_handler,
            .window_blur_handler = &window_blur_handler,
            .window_pageshow_handler = &window_pageshow_handler,
            .window_pagehide_handler = &window_pagehide_handler,
            .document_scroll_handler = &document_scroll_handler,
            .document_selectionchange_handler = &document_selectionchange_handler,
            .document_readystatechange_handler = &document_readystatechange_handler,
            .window_hashchange_handler = &window_hashchange_handler,
            .window_popstate_handler = &window_popstate_handler,
            .window_storage_handler = &window_storage_handler,
            .window_scroll_handler = &window_scroll_handler,
            .document_ready_state = &document_ready_state,
            .document_ready_state_change_pending = &document_ready_state_change_pending,
            .history_scroll_restoration = &history_scroll_restoration,
            .math_random_state = &math_random_state,
            .crypto_random_state = &crypto_random_state,
            .document_write_buffer = &document_write_buffer,
            .cookie_jar = &cookie_jar,
            .scroll_x = &scroll_x,
            .scroll_y = &scroll_y,
            .queued_microtasks = &queued_microtasks,
            .timers = &timers,
            .next_timer_id = &next_timer_id,
            .storage = mock_registry.storage(),
        };
        if (html_copy) |html_source| {
            try dom_store.bootstrapHtml(html_source);
        }
        try dom_store.setTargetFragment(fragmentIdentifierFromUrl(url_copy));
        if (html_copy) |html_source| {
            _ = html_source;
            try script_runtime.bootstrapInlineScripts(allocator, &bootstrap_host);
            var bootstrap_steps: usize = 0;
            try drainQueuedMicrotasks(allocator, &script_runtime, &bootstrap_host, &queued_microtasks, &bootstrap_steps);
        }
        try script_runtime.dispatchDocumentEvent(
            std.heap.page_allocator,
            &bootstrap_host,
            "DOMContentLoaded",
            script_event_listeners.items,
            null,
            "onDOMContentLoaded",
        );
        bootstrap_host.setDocumentReadyState("complete");
        if (bootstrap_host.documentReadyStateChangePending()) {
            try script_runtime.dispatchDocumentEvent(
                std.heap.page_allocator,
                &bootstrap_host,
                "readystatechange",
                script_event_listeners.items,
                bootstrap_host.documentReadyStateChange(),
                "onreadystatechange",
            );
            bootstrap_host.setDocumentReadyStateChangePending(false);
        }
        try script_runtime.dispatchWindowEvent(
            std.heap.page_allocator,
            &bootstrap_host,
            "load",
            script_event_listeners.items,
            bootstrap_host.windowLoad(),
            "onload",
        );
        try script_runtime.dispatchWindowEvent(
            std.heap.page_allocator,
            &bootstrap_host,
            "pageshow",
            script_event_listeners.items,
            bootstrap_host.windowPageShow(),
            "onpageshow",
        );

        return .{
            .arena_owner = allocator,
            .arena = arena,
            .config = .{
                .url = url_copy,
                .html = html_copy,
                .local_storage = storage_copy,
                .session_storage = session_storage_copy,
                .random_seed = config.random_seed,
            },
            .dom_store = dom_store,
            .script_runtime = script_runtime,
            .script_event_listeners = script_event_listeners,
            .match_media_listeners = match_media_listeners,
            .match_media_onchange_listeners = match_media_onchange_listeners,
            .queued_microtasks = queued_microtasks,
            .timers = timers,
            .next_timer_id = next_timer_id,
            .mock_registry = mock_registry,
            .history = history,
            .history_scroll_restoration = history_scroll_restoration,
            .math_random_state = math_random_state,
            .crypto_random_state = crypto_random_state,
            .document_write_buffer = document_write_buffer,
            .clock_ms = 0,
            .scroll_x = scroll_x,
            .scroll_y = scroll_y,
            .window_name = window_name,
            .window_load_handler = window_load_handler,
            .window_beforeprint_handler = window_beforeprint_handler,
            .window_afterprint_handler = window_afterprint_handler,
            .window_beforeunload_handler = window_beforeunload_handler,
            .window_unload_handler = window_unload_handler,
            .window_focus_handler = window_focus_handler,
            .window_blur_handler = window_blur_handler,
            .window_pageshow_handler = window_pageshow_handler,
            .window_pagehide_handler = window_pagehide_handler,
            .document_scroll_handler = document_scroll_handler,
            .document_selectionchange_handler = document_selectionchange_handler,
            .document_readystatechange_handler = document_readystatechange_handler,
            .window_hashchange_handler = window_hashchange_handler,
            .window_popstate_handler = window_popstate_handler,
            .window_storage_handler = window_storage_handler,
            .window_scroll_handler = window_scroll_handler,
            .document_ready_state = "complete",
            .document_ready_state_change_pending = false,
            .cookie_jar = cookie_jar,
        };
    }

    pub fn deinit(self: *Session) void {
        self.document_write_buffer.deinit(self.arena.allocator());
        self.mock_registry.deinit();
        self.history.deinit();
        self.cookie_jar.deinit(self.arena.allocator());
        self.script_event_listeners.deinit(self.arena.allocator());
        self.match_media_listeners.deinit(self.arena.allocator());
        self.match_media_onchange_listeners.deinit(self.arena.allocator());
        self.queued_microtasks.deinit(self.arena.allocator());
        self.timers.deinit(self.arena.allocator());
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
        try self.runSchedulerCycle();
        return;
    }

    pub fn flush(self: *Session) errors.Result(void) {
        try self.runSchedulerCycle();
        return;
    }

    fn runSchedulerCycle(self: *Session) errors.Result(void) {
        const allocator = std.heap.page_allocator;
        var step_count: usize = 0;
        try drainQueuedMicrotasks(
            allocator,
            &self.script_runtime,
            self,
            &self.queued_microtasks,
            &step_count,
        );
        try runDueTimers(self, allocator, &step_count);
        try self.runMatchMediaListeners(allocator);
        try self.runMatchMediaOnChangeListeners(allocator);
        try drainQueuedMicrotasks(
            allocator,
            &self.script_runtime,
            self,
            &self.queued_microtasks,
            &step_count,
        );
    }

    fn runMatchMediaListeners(self: *Session, allocator: std.mem.Allocator) errors.Result(void) {
        var listeners: std.ArrayList(script.MatchMediaListenerRecord) = .empty;
        defer listeners.deinit(allocator);

        try listeners.appendSlice(allocator, self.match_media_listeners.items);
        for (listeners.items, 0..) |listener, index| {
            const current = self.matchMediaCurrent(listener.query);
            if (current == listener.last_matches) continue;

            var updated = false;
            for (self.match_media_listeners.items, 0..) |*stored_listener, stored_index| {
                if (stored_index != index) continue;
                if (!std.mem.eql(u8, stored_listener.query, listener.query)) continue;
                if (!scriptFunctionEquals(stored_listener.handler, listener.handler)) continue;
                stored_listener.last_matches = current;
                updated = true;
                break;
            }
            if (!updated) continue;

            const source_name = try std.fmt.allocPrint(allocator, "matchmedia:{s}:{d}", .{ listener.query, index });
            defer allocator.free(source_name);
            try self.script_runtime.evalScriptSourceWithBindings(
                allocator,
                self,
                listener.handler.body_source,
                source_name,
                &.{},
            );
        }
    }

    fn runMatchMediaOnChangeListeners(self: *Session, allocator: std.mem.Allocator) errors.Result(void) {
        var listeners: std.ArrayList(script.MatchMediaListenerRecord) = .empty;
        defer listeners.deinit(allocator);

        try listeners.appendSlice(allocator, self.match_media_onchange_listeners.items);
        for (listeners.items, 0..) |listener, index| {
            const current = self.matchMediaCurrent(listener.query);
            if (current == listener.last_matches) continue;

            var updated = false;
            for (self.match_media_onchange_listeners.items, 0..) |*stored_listener, stored_index| {
                if (stored_index != index) continue;
                if (!std.mem.eql(u8, stored_listener.query, listener.query)) continue;
                stored_listener.last_matches = current;
                updated = true;
                break;
            }
            if (!updated) continue;

            const source_name = try std.fmt.allocPrint(allocator, "matchmedia:onchange:{s}:{d}", .{ listener.query, index });
            defer allocator.free(source_name);
            try self.script_runtime.evalScriptSourceWithBindings(
                allocator,
                self,
                listener.handler.body_source,
                source_name,
                &.{},
            );
        }
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

    pub fn scriptEventListeners(self: *const Session) []const script.ScriptListenerRecord {
        return self.script_event_listeners.items;
    }

    pub fn currentLocationUrl(self: *const Session) []const u8 {
        return @constCast(&self.mock_registry).location().currentUrl() orelse self.config.url;
    }

    pub fn assignLocation(self: *Session, url_source: []const u8) errors.Result(void) {
        const trimmed = std.mem.trim(u8, url_source, " \t\r\n");
        if (trimmed.len == 0) return error.MockError;

        const before_url = self.currentLocationUrl();
        const target = try self.history.push(null, trimmed);
        const page_lifecycle = shouldDispatchPageLifecycle(before_url, target);
        if (page_lifecycle) {
            try self.script_runtime.dispatchWindowEvent(
                std.heap.page_allocator,
                self,
                "beforeunload",
                self.scriptEventListeners(),
                self.windowBeforeUnload(),
                "onbeforeunload",
            );
            try self.script_runtime.dispatchWindowEvent(
                std.heap.page_allocator,
                self,
                "pagehide",
                self.scriptEventListeners(),
                self.windowPageHide(),
                "onpagehide",
            );
            try self.script_runtime.dispatchWindowEvent(
                std.heap.page_allocator,
                self,
                "unload",
                self.scriptEventListeners(),
                self.windowUnload(),
                "onunload",
            );
        }
        try syncLocationState(self.mock_registry.location(), &self.dom_store, target, true);
        self.resetScrollPosition();
        try self.script_runtime.dispatchHashChangeIfNeeded(std.heap.page_allocator, self, before_url, target);
        if (page_lifecycle) {
            try self.script_runtime.dispatchWindowEvent(
                std.heap.page_allocator,
                self,
                "pageshow",
                self.scriptEventListeners(),
                self.windowPageShow(),
                "onpageshow",
            );
        }
        return;
    }

    pub fn replaceLocation(self: *Session, url_source: []const u8) errors.Result(void) {
        const trimmed = std.mem.trim(u8, url_source, " \t\r\n");
        if (trimmed.len == 0) return error.MockError;

        const before_url = self.currentLocationUrl();
        const target = try self.history.replace(null, trimmed);
        const page_lifecycle = shouldDispatchPageLifecycle(before_url, target);
        if (page_lifecycle) {
            try self.script_runtime.dispatchWindowEvent(
                std.heap.page_allocator,
                self,
                "beforeunload",
                self.scriptEventListeners(),
                self.windowBeforeUnload(),
                "onbeforeunload",
            );
            try self.script_runtime.dispatchWindowEvent(
                std.heap.page_allocator,
                self,
                "pagehide",
                self.scriptEventListeners(),
                self.windowPageHide(),
                "onpagehide",
            );
            try self.script_runtime.dispatchWindowEvent(
                std.heap.page_allocator,
                self,
                "unload",
                self.scriptEventListeners(),
                self.windowUnload(),
                "onunload",
            );
        }
        try syncLocationState(self.mock_registry.location(), &self.dom_store, target, true);
        self.resetScrollPosition();
        try self.script_runtime.dispatchHashChangeIfNeeded(std.heap.page_allocator, self, before_url, target);
        if (page_lifecycle) {
            try self.script_runtime.dispatchWindowEvent(
                std.heap.page_allocator,
                self,
                "pageshow",
                self.scriptEventListeners(),
                self.windowPageShow(),
                "onpageshow",
            );
        }
        return;
    }

    pub fn reloadLocation(self: *Session) errors.Result(void) {
        const target = self.history.current();
        try self.script_runtime.dispatchWindowEvent(
            std.heap.page_allocator,
            self,
            "beforeunload",
            self.scriptEventListeners(),
            self.windowBeforeUnload(),
            "onbeforeunload",
        );
        try self.script_runtime.dispatchWindowEvent(
            std.heap.page_allocator,
            self,
            "pagehide",
            self.scriptEventListeners(),
            self.windowPageHide(),
            "onpagehide",
        );
        try self.script_runtime.dispatchWindowEvent(
            std.heap.page_allocator,
            self,
            "unload",
            self.scriptEventListeners(),
            self.windowUnload(),
            "onunload",
        );
        try syncLocationState(self.mock_registry.location(), &self.dom_store, target, true);
        self.resetScrollPosition();
        try self.script_runtime.dispatchWindowEvent(
            std.heap.page_allocator,
            self,
            "pageshow",
            self.scriptEventListeners(),
            self.windowPageShow(),
            "onpageshow",
        );
        return;
    }

    pub fn locationHash(self: *const Session, allocator: std.mem.Allocator) errors.Result([]const u8) {
        return try locationHashFromUrl(allocator, self.currentLocationUrl());
    }

    pub fn setLocationHash(self: *Session, value: []const u8) errors.Result(void) {
        const target = try locationUrlWithHash(self.arena.allocator(), self.currentLocationUrl(), value);
        try self.assignLocation(target);
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

    pub fn historyScrollRestoration(self: *const Session) []const u8 {
        return self.history_scroll_restoration;
    }

    pub fn setHistoryScrollRestoration(self: *Session, value: []const u8) errors.Result(void) {
        if (!std.mem.eql(u8, value, "auto") and !std.mem.eql(u8, value, "manual")) {
            return error.ScriptRuntime;
        }

        self.history_scroll_restoration = value;
        return;
    }

    pub fn historyBack(self: *Session) errors.Result(void) {
        return self.historyGo(-1);
    }

    pub fn historyForward(self: *Session) errors.Result(void) {
        return self.historyGo(1);
    }

    pub fn historyGo(self: *Session, delta: isize) errors.Result(void) {
        const before_url = self.currentLocationUrl();
        if (delta == 0) {
            try self.reloadLocation();
            return;
        }

        if (self.history.go(delta)) |target| {
            try syncLocationState(self.mock_registry.location(), &self.dom_store, target, true);
            try self.script_runtime.dispatchWindowEvent(
                std.heap.page_allocator,
                self,
                "popstate",
                self.scriptEventListeners(),
                self.windowPopState(),
                "onpopstate",
            );
            try self.script_runtime.dispatchHashChangeIfNeeded(std.heap.page_allocator, self, before_url, target);
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

    pub fn documentScrollingElement(self: *const Session) ?dom.NodeId {
        return self.dom_store.documentElement() orelse self.dom_store.documentBody();
    }

    pub fn documentTitle(self: *const Session) []const u8 {
        return self.dom_store.documentTitle();
    }

    pub fn documentOpen(self: *Session) errors.Result(void) {
        try self.dom_store.clearDocument();
        self.document_write_buffer.clearRetainingCapacity();
        self.document_ready_state = "loading";
        self.document_ready_state_change_pending = true;
        return;
    }

    pub fn documentWrite(self: *Session, markup: []const u8) errors.Result(void) {
        try self.document_write_buffer.appendSlice(self.arena.allocator(), markup);
        return;
    }

    pub fn documentClose(self: *Session) errors.Result(void) {
        if (self.document_write_buffer.items.len > 0) {
            try self.dom_store.appendHtmlToDocument(self.document_write_buffer.items);
            self.document_write_buffer.clearRetainingCapacity();
        }
        try self.script_runtime.dispatchDocumentEvent(
            std.heap.page_allocator,
            self,
            "DOMContentLoaded",
            self.scriptEventListeners(),
            null,
            "onDOMContentLoaded",
        );
        self.document_ready_state = "complete";
        if (self.document_ready_state_change_pending) {
            try self.script_runtime.dispatchDocumentEvent(
                std.heap.page_allocator,
                self,
                "readystatechange",
                self.scriptEventListeners(),
                self.documentReadyStateChange(),
                "onreadystatechange",
            );
            self.document_ready_state_change_pending = false;
        }
        return;
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

    pub fn documentDomain(self: *const Session, allocator: std.mem.Allocator) errors.Result([]const u8) {
        return try domainFromUrl(allocator, self.currentLocationUrl());
    }

    pub fn documentCookie(self: *const Session, allocator: std.mem.Allocator) errors.Result([]const u8) {
        return try cookieJarText(allocator, &self.cookie_jar);
    }

    pub fn setDocumentCookie(self: *Session, value: []const u8) errors.Result(void) {
        try cookieJarSet(self.arena.allocator(), &self.cookie_jar, value);
        return;
    }

    pub fn documentVisibilityState(self: *const Session) []const u8 {
        _ = self;
        return "visible";
    }

    pub fn documentHidden(self: *const Session) bool {
        _ = self;
        return false;
    }

    pub fn documentHasFocus(self: *const Session) bool {
        _ = self;
        return true;
    }

    pub fn windowName(self: *const Session) []const u8 {
        return self.window_name;
    }

    pub fn windowLoad(self: *const Session) ?script.ScriptFunction {
        return self.window_load_handler;
    }

    pub fn windowBeforePrint(self: *const Session) ?script.ScriptFunction {
        return self.window_beforeprint_handler;
    }

    pub fn setWindowBeforePrint(self: *Session, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_beforeprint_handler = try duplicateScriptFunction(self.arena.allocator(), function);
            return;
        }
        self.window_beforeprint_handler = null;
        return;
    }

    pub fn windowAfterPrint(self: *const Session) ?script.ScriptFunction {
        return self.window_afterprint_handler;
    }

    pub fn setWindowAfterPrint(self: *Session, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_afterprint_handler = try duplicateScriptFunction(self.arena.allocator(), function);
            return;
        }
        self.window_afterprint_handler = null;
        return;
    }

    pub fn setWindowLoad(self: *Session, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_load_handler = try duplicateScriptFunction(self.arena.allocator(), function);
            return;
        }
        self.window_load_handler = null;
        return;
    }

    pub fn windowBeforeUnload(self: *const Session) ?script.ScriptFunction {
        return self.window_beforeunload_handler;
    }

    pub fn setWindowBeforeUnload(self: *Session, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_beforeunload_handler = try duplicateScriptFunction(self.arena.allocator(), function);
            return;
        }
        self.window_beforeunload_handler = null;
        return;
    }

    pub fn windowUnload(self: *const Session) ?script.ScriptFunction {
        return self.window_unload_handler;
    }

    pub fn setWindowUnload(self: *Session, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_unload_handler = try duplicateScriptFunction(self.arena.allocator(), function);
            return;
        }
        self.window_unload_handler = null;
        return;
    }

    pub fn windowFocus(self: *const Session) ?script.ScriptFunction {
        return self.window_focus_handler;
    }

    pub fn windowHashChange(self: *const Session) ?script.ScriptFunction {
        return self.window_hashchange_handler;
    }

    pub fn setWindowName(self: *Session, value: []const u8) errors.Result(void) {
        self.window_name = try self.arena.allocator().dupe(u8, value);
        return;
    }

    pub fn setWindowFocus(self: *Session, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_focus_handler = try duplicateScriptFunction(self.arena.allocator(), function);
            return;
        }
        self.window_focus_handler = null;
        return;
    }

    pub fn setWindowHashChange(self: *Session, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_hashchange_handler = try duplicateScriptFunction(self.arena.allocator(), function);
            return;
        }
        self.window_hashchange_handler = null;
        return;
    }

    pub fn windowBlur(self: *const Session) ?script.ScriptFunction {
        return self.window_blur_handler;
    }

    pub fn windowPageShow(self: *const Session) ?script.ScriptFunction {
        return self.window_pageshow_handler;
    }

    pub fn windowPageHide(self: *const Session) ?script.ScriptFunction {
        return self.window_pagehide_handler;
    }

    pub fn documentScroll(self: *const Session) ?script.ScriptFunction {
        return self.document_scroll_handler;
    }

    pub fn documentSelectionChange(self: *const Session) ?script.ScriptFunction {
        return self.document_selectionchange_handler;
    }

    pub fn documentReadyStateChange(self: *const Session) ?script.ScriptFunction {
        return self.document_readystatechange_handler;
    }

    pub fn setWindowBlur(self: *Session, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_blur_handler = try duplicateScriptFunction(self.arena.allocator(), function);
            return;
        }
        self.window_blur_handler = null;
        return;
    }

    pub fn setWindowPageShow(self: *Session, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_pageshow_handler = try duplicateScriptFunction(self.arena.allocator(), function);
            return;
        }
        self.window_pageshow_handler = null;
        return;
    }

    pub fn setWindowPageHide(self: *Session, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_pagehide_handler = try duplicateScriptFunction(self.arena.allocator(), function);
            return;
        }
        self.window_pagehide_handler = null;
        return;
    }

    pub fn setDocumentScroll(self: *Session, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.document_scroll_handler = try duplicateScriptFunction(self.arena.allocator(), function);
            return;
        }
        self.document_scroll_handler = null;
        return;
    }

    pub fn setDocumentSelectionChange(self: *Session, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.document_selectionchange_handler = try duplicateScriptFunction(self.arena.allocator(), function);
            return;
        }
        self.document_selectionchange_handler = null;
        return;
    }

    pub fn setDocumentReadyStateChange(self: *Session, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.document_readystatechange_handler = try duplicateScriptFunction(self.arena.allocator(), function);
            return;
        }
        self.document_readystatechange_handler = null;
        return;
    }

    pub fn windowPopState(self: *const Session) ?script.ScriptFunction {
        return self.window_popstate_handler;
    }

    pub fn setWindowPopState(self: *Session, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_popstate_handler = try duplicateScriptFunction(self.arena.allocator(), function);
            return;
        }
        self.window_popstate_handler = null;
        return;
    }

    pub fn windowStorage(self: *const Session) ?script.ScriptFunction {
        return self.window_storage_handler;
    }

    pub fn setWindowStorage(self: *Session, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_storage_handler = try duplicateScriptFunction(self.arena.allocator(), function);
            return;
        }
        self.window_storage_handler = null;
        return;
    }

    pub fn windowScroll(self: *const Session) ?script.ScriptFunction {
        return self.window_scroll_handler;
    }

    pub fn setWindowScroll(self: *Session, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_scroll_handler = try duplicateScriptFunction(self.arena.allocator(), function);
            return;
        }
        self.window_scroll_handler = null;
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

    pub fn windowNavigatorUserAgent(self: *const Session) []const u8 {
        _ = self;
        return "browser_tester";
    }

    pub fn windowNavigatorAppCodeName(self: *const Session) []const u8 {
        _ = self;
        return "browser_tester";
    }

    pub fn windowNavigatorAppName(self: *const Session) []const u8 {
        _ = self;
        return "browser_tester";
    }

    pub fn windowNavigatorAppVersion(self: *const Session) []const u8 {
        _ = self;
        return "browser_tester";
    }

    pub fn windowNavigatorProduct(self: *const Session) []const u8 {
        _ = self;
        return "browser_tester";
    }

    pub fn windowNavigatorProductSub(self: *const Session) []const u8 {
        _ = self;
        return "browser_tester";
    }

    pub fn windowNavigatorVendor(self: *const Session) []const u8 {
        _ = self;
        return "browser_tester";
    }

    pub fn windowNavigatorVendorSub(self: *const Session) []const u8 {
        _ = self;
        return "";
    }

    pub fn windowNavigatorPdfViewerEnabled(self: *const Session) bool {
        _ = self;
        return false;
    }

    pub fn windowNavigatorDoNotTrack(self: *const Session) []const u8 {
        _ = self;
        return "unspecified";
    }

    pub fn windowNavigatorPlatform(self: *const Session) []const u8 {
        _ = self;
        return "unknown";
    }

    pub fn windowNavigatorLanguage(self: *const Session) []const u8 {
        _ = self;
        return "en-US";
    }

    pub fn windowNavigatorUserLanguage(self: *const Session) []const u8 {
        _ = self;
        return "en-US";
    }

    pub fn windowNavigatorBrowserLanguage(self: *const Session) []const u8 {
        _ = self;
        return "en-US";
    }

    pub fn windowNavigatorSystemLanguage(self: *const Session) []const u8 {
        _ = self;
        return "en-US";
    }

    pub fn windowNavigatorOscpu(self: *const Session) []const u8 {
        _ = self;
        return "unknown";
    }

    pub fn windowNavigatorLanguages(self: *const Session) []const []const u8 {
        _ = self;
        return &.{"en-US"};
    }

    pub fn windowNavigatorJavaEnabled(self: *const Session) bool {
        _ = self;
        return false;
    }

    pub fn windowNavigatorCookieEnabled(self: *const Session) bool {
        _ = self;
        return true;
    }

    pub fn windowNavigatorOnLine(self: *const Session) bool {
        _ = self;
        return true;
    }

    pub fn windowNavigatorWebdriver(self: *const Session) bool {
        _ = self;
        return false;
    }

    pub fn windowNavigatorHardwareConcurrency(self: *const Session) i64 {
        _ = self;
        return 8;
    }

    pub fn windowNavigatorMaxTouchPoints(self: *const Session) i64 {
        _ = self;
        return 0;
    }

    pub fn windowDevicePixelRatio(self: *const Session) f64 {
        _ = self;
        return 1.0;
    }

    pub fn windowInnerWidth(self: *const Session) i64 {
        _ = self;
        return 1024;
    }

    pub fn windowInnerHeight(self: *const Session) i64 {
        _ = self;
        return 768;
    }

    pub fn windowOuterWidth(self: *const Session) i64 {
        _ = self;
        return 1280;
    }

    pub fn windowOuterHeight(self: *const Session) i64 {
        _ = self;
        return 800;
    }

    pub fn windowScreenWidth(self: *const Session) i64 {
        _ = self;
        return 1280;
    }

    pub fn windowScreenHeight(self: *const Session) i64 {
        _ = self;
        return 800;
    }

    pub fn windowScreenAvailWidth(self: *const Session) i64 {
        _ = self;
        return 1280;
    }

    pub fn windowScreenAvailHeight(self: *const Session) i64 {
        _ = self;
        return 800;
    }

    pub fn windowScreenAvailLeft(self: *const Session) i64 {
        _ = self;
        return 0;
    }

    pub fn windowScreenAvailTop(self: *const Session) i64 {
        _ = self;
        return 0;
    }

    pub fn windowScreenColorDepth(self: *const Session) i64 {
        _ = self;
        return 24;
    }

    pub fn windowScreenPixelDepth(self: *const Session) i64 {
        _ = self;
        return 24;
    }

    pub fn windowScreenOrientationType(self: *const Session) []const u8 {
        _ = self;
        return "landscape-primary";
    }

    pub fn windowScreenOrientationAngle(self: *const Session) i64 {
        _ = self;
        return 0;
    }

    pub fn windowScreenX(self: *const Session) i64 {
        _ = self;
        return 0;
    }

    pub fn windowScreenY(self: *const Session) i64 {
        _ = self;
        return 0;
    }

    pub fn windowScreenLeft(self: *const Session) i64 {
        _ = self;
        return 0;
    }

    pub fn windowScreenTop(self: *const Session) i64 {
        _ = self;
        return 0;
    }

    pub fn mathRandom(self: *Session) f64 {
        self.math_random_state = self.math_random_state *% 73 +% 41;
        return @as(f64, @floatFromInt(self.math_random_state % 1000)) / 1000.0;
    }

    pub fn cryptoRandomUUID(self: *Session, allocator: std.mem.Allocator) errors.Result([]const u8) {
        return try cryptoRandomUuidFromState(allocator, &self.crypto_random_state);
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
            if (std.mem.eql(u8, value_ptr.*, value)) {
                return;
            }
            value_ptr.* = try self.arena.allocator().dupe(u8, value);
            try self.dispatchStorageEvent();
            return;
        }

        const key_copy = try self.arena.allocator().dupe(u8, key);
        const value_copy = try self.arena.allocator().dupe(u8, value);
        try storage.put(self.arena.allocator(), key_copy, value_copy);
        try self.dispatchStorageEvent();
        return;
    }

    pub fn storageRemoveItem(self: *Session, target: script.StorageTarget, key: []const u8) errors.Result(void) {
        if (self.storageMapMut(target).contains(key)) {
            _ = self.storageMapMut(target).orderedRemove(key);
            try self.dispatchStorageEvent();
        }
        return;
    }

    pub fn storageClear(self: *Session, target: script.StorageTarget) errors.Result(void) {
        if (self.storageMapMut(target).count() > 0) {
            self.storageMapMut(target).clearRetainingCapacity();
            try self.dispatchStorageEvent();
        }
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
        return self.document_ready_state;
    }

    pub fn currentScript(self: *const Session) ?dom.NodeId {
        _ = self;
        return null;
    }

    fn dispatchStorageEvent(self: *Session) errors.Result(void) {
        try self.script_runtime.dispatchWindowEvent(
            std.heap.page_allocator,
            self,
            "storage",
            self.scriptEventListeners(),
            self.windowStorage(),
            "onstorage",
        );
        return;
    }

    pub fn dispatchSelectionChange(self: *Session) errors.Result(void) {
        try self.script_runtime.dispatchDocumentEvent(
            std.heap.page_allocator,
            self,
            "selectionchange",
            self.scriptEventListeners(),
            self.documentSelectionChange(),
            "onselectionchange",
        );
        return;
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
        var runtime = script.ScriptRuntime{};
        try runtime.dispatchWindowEvent(
            std.heap.page_allocator,
            self,
            "beforeprint",
            self.scriptEventListeners(),
            self.windowBeforePrint(),
            "onbeforeprint",
        );
        try runtime.dispatchWindowEvent(
            std.heap.page_allocator,
            self,
            "afterprint",
            self.scriptEventListeners(),
            self.windowAfterPrint(),
            "onafterprint",
        );
        return;
    }

    pub fn scrollTo(self: *Session, x: i64, y: i64) errors.Result(void) {
        try self.mock_registry.scroll().recordCall(.To, x, y);
        const old_x = self.scroll_x;
        const old_y = self.scroll_y;
        self.scroll_x = x;
        self.scroll_y = y;
        if (old_x != x or old_y != y) {
            var runtime = script.ScriptRuntime{};
            try runtime.dispatchDocumentEvent(
                std.heap.page_allocator,
                self,
                "scroll",
                self.scriptEventListeners(),
                self.documentScroll(),
                "onscroll",
            );
            try runtime.dispatchWindowEvent(
                std.heap.page_allocator,
                self,
                "scroll",
                self.scriptEventListeners(),
                self.windowScroll(),
                "onscroll",
            );
        }
        return;
    }

    pub fn scrollBy(self: *Session, x: i64, y: i64) errors.Result(void) {
        try self.mock_registry.scroll().recordCall(.By, x, y);
        const next_x = std.math.add(i64, self.scroll_x, x) catch return error.ScriptRuntime;
        const next_y = std.math.add(i64, self.scroll_y, y) catch return error.ScriptRuntime;
        const old_x = self.scroll_x;
        const old_y = self.scroll_y;
        self.scroll_x = next_x;
        self.scroll_y = next_y;
        if (old_x != next_x or old_y != next_y) {
            var runtime = script.ScriptRuntime{};
            try runtime.dispatchDocumentEvent(
                std.heap.page_allocator,
                self,
                "scroll",
                self.scriptEventListeners(),
                self.documentScroll(),
                "onscroll",
            );
            try runtime.dispatchWindowEvent(
                std.heap.page_allocator,
                self,
                "scroll",
                self.scriptEventListeners(),
                self.windowScroll(),
                "onscroll",
            );
        }
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

    pub fn matchMediaCurrent(self: *const Session, query_source: []const u8) bool {
        return @constCast(&self.mock_registry).matchMedia().currentMatch(query_source) orelse false;
    }

    pub fn matchMediaOnChange(self: *const Session, query_source: []const u8) ?script.ScriptFunction {
        return findMatchMediaOnChange(&self.match_media_onchange_listeners, query_source);
    }

    pub fn setMatchMediaOnChange(
        self: *Session,
        query_source: []const u8,
        handler: ?script.ScriptFunction,
    ) errors.Result(void) {
        if (handler) |function| {
            try upsertMatchMediaOnChange(
                self.arena.allocator(),
                &self.match_media_onchange_listeners,
                query_source,
                function,
                self.matchMediaCurrent(query_source),
            );
            return;
        }
        removeMatchMediaOnChange(&self.match_media_onchange_listeners, query_source);
        return;
    }

    pub fn registerMatchMediaListener(
        self: *Session,
        query_source: []const u8,
        handler: script.ScriptFunction,
    ) errors.Result(void) {
        try appendMatchMediaListener(
            self.arena.allocator(),
            &self.match_media_listeners,
            query_source,
            handler,
            self.matchMediaCurrent(query_source),
        );
        return;
    }

    pub fn unregisterMatchMediaListener(
        self: *Session,
        query_source: []const u8,
        handler: script.ScriptFunction,
    ) errors.Result(void) {
        removeMatchMediaListener(&self.match_media_listeners, query_source, handler);
        return;
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
        try self.flush();
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

    pub fn queueMicrotask(self: *Session, handler: script.ScriptFunction) errors.Result(void) {
        try appendQueuedMicrotask(self.arena.allocator(), &self.queued_microtasks, handler);
        return;
    }

    pub fn scheduleTimer(
        self: *Session,
        handler: script.ScriptFunction,
        delay_ms: i64,
    ) errors.Result(u64) {
        const at_ms = std.math.add(i64, self.clock_ms, delay_ms) catch return error.TimerError;
        return try appendScheduledTimer(
            self.arena.allocator(),
            &self.next_timer_id,
            &self.timers,
            at_ms,
            null,
            .timeout,
            handler,
        );
    }

    pub fn scheduleIntervalTimer(
        self: *Session,
        handler: script.ScriptFunction,
        delay_ms: i64,
    ) errors.Result(u64) {
        const at_ms = std.math.add(i64, self.clock_ms, delay_ms) catch return error.TimerError;
        return try appendScheduledTimer(
            self.arena.allocator(),
            &self.next_timer_id,
            &self.timers,
            at_ms,
            delay_ms,
            .interval,
            handler,
        );
    }

    pub fn scheduleAnimationFrame(
        self: *Session,
        handler: script.ScriptFunction,
    ) errors.Result(u64) {
        const at_ms = try nextAnimationFrameTime(self.clock_ms);
        return try appendScheduledTimer(
            self.arena.allocator(),
            &self.next_timer_id,
            &self.timers,
            at_ms,
            null,
            .animation_frame,
            handler,
        );
    }

    pub fn clearTimer(self: *Session, timer_id: u64) void {
        if (self.active_timer_id) |active_timer_id| {
            if (active_timer_id == timer_id) {
                self.active_timer_cancelled = true;
                return;
            }
        }
        clearScheduledTimer(&self.timers, timer_id);
    }

    pub fn dispatchNode(self: *Session, node_id: dom.NodeId, event_type: []const u8) errors.Result(void) {
        const trimmed = std.mem.trim(u8, event_type, " \t\r\n");
        if (trimmed.len == 0) return error.EventError;
        _ = try self.dispatchDomEvent(node_id, trimmed, true, true);
        try self.flush();
        return;
    }

    pub fn clickNode(self: *Session, node_id: dom.NodeId) errors.Result(void) {
        try self.ensureElementNode(node_id);
        const outcome = try self.dispatchDomEvent(node_id, "click", true, true);
        if (!outcome.default_prevented) {
            try runClickDefaultActionsForHost(self.arena.allocator(), self, node_id);
        }
        try self.flush();
        return;
    }

    pub fn typeTextNode(self: *Session, node_id: dom.NodeId, text: []const u8) errors.Result(void) {
        try self.ensureElementNode(node_id);
        try self.dom_store.setFormControlValue(node_id, text);
        if (self.dom_store.isSelectionControlNode(node_id)) {
            try self.dispatchSelectionChange();
        }
        _ = try self.dispatchDomEvent(node_id, "input", true, false);
        try self.flush();
        return;
    }

    pub fn setCheckedNode(self: *Session, node_id: dom.NodeId, checked: bool) errors.Result(void) {
        try self.ensureElementNode(node_id);
        try self.dom_store.setFormControlChecked(node_id, checked);
        _ = try self.dispatchDomEvent(node_id, "input", true, false);
        _ = try self.dispatchDomEvent(node_id, "change", true, false);
        try self.flush();
        return;
    }

    pub fn setSelectValueNode(self: *Session, node_id: dom.NodeId, value: []const u8) errors.Result(void) {
        try self.ensureElementNode(node_id);
        try self.dom_store.setSelectValue(node_id, value);
        _ = try self.dispatchDomEvent(node_id, "input", true, false);
        _ = try self.dispatchDomEvent(node_id, "change", true, false);
        try self.flush();
        return;
    }

    pub fn focusNode(self: *Session, node_id: dom.NodeId) errors.Result(void) {
        try self.ensureElementNode(node_id);
        const had_focus = self.dom_store.focusedNode() != null;
        if (self.dom_store.focusedNode()) |focused| {
            if (std.meta.eql(focused, node_id)) {
                return;
            }
        }

        if (self.dom_store.focusedNode()) |previous| {
            self.dom_store.setFocusedNode(null);
            _ = try self.dispatchDomEvent(previous, "focusout", true, false);
            _ = try self.dispatchDomEvent(previous, "blur", false, false);
        }

        self.dom_store.setFocusedNode(node_id);
        _ = try self.dispatchDomEvent(node_id, "focusin", true, false);
        _ = try self.dispatchDomEvent(node_id, "focus", false, false);
        if (!had_focus) {
            try self.script_runtime.dispatchWindowEvent(
                std.heap.page_allocator,
                self,
                "focus",
                self.scriptEventListeners(),
                self.windowFocus(),
                "onfocus",
            );
        }
        try self.flush();
        return;
    }

    pub fn blurNode(self: *Session, node_id: dom.NodeId) errors.Result(void) {
        try self.ensureElementNode(node_id);
        const had_focus = self.dom_store.focusedNode() != null;
        if (self.dom_store.focusedNode()) |focused| {
            if (!std.meta.eql(focused, node_id)) {
                return;
            }
        } else {
            return;
        }

        self.dom_store.setFocusedNode(null);
        _ = try self.dispatchDomEvent(node_id, "focusout", true, false);
        _ = try self.dispatchDomEvent(node_id, "blur", false, false);
        if (had_focus) {
            try self.script_runtime.dispatchWindowEvent(
                std.heap.page_allocator,
                self,
                "blur",
                self.scriptEventListeners(),
                self.windowBlur(),
                "onblur",
            );
        }
        try self.flush();
        return;
    }

    pub fn submitNode(self: *Session, node_id: dom.NodeId, submitter: ?dom.NodeId) errors.Result(void) {
        try self.ensureElementNode(node_id);
        const node = self.dom_store.nodeAt(node_id) orelse return error.DomError;
        const submitter_node = submitter orelse if (isFormNode(node)) null else node_id;
        if (isFormNode(node)) {
            var runtime = script.ScriptRuntime{};
            _ = try runtime.dispatchSubmitEvent(self.arena.allocator(), self, node_id, submitter_node);
            try self.flush();
            return;
        }

        const form_id = self.findAssociatedForm(node_id) orelse return error.DomError;
        var runtime = script.ScriptRuntime{};
        _ = try runtime.dispatchSubmitEvent(self.arena.allocator(), self, form_id, submitter_node);
        try self.flush();
        return;
    }

    pub fn resetNode(self: *Session, node_id: dom.NodeId) errors.Result(void) {
        try self.ensureElementNode(node_id);
        const node = self.dom_store.nodeAt(node_id) orelse return error.DomError;
        if (!isFormNode(node)) return error.DomError;

        _ = try self.dispatchDomEvent(node_id, "reset", true, true);
        try self.flush();
        return;
    }

    pub fn showDialog(self: *Session, node_id: dom.NodeId) errors.Result(void) {
        const node = self.dom_store.nodeAt(node_id) orelse return error.DomError;
        if (!isDialogNode(node)) return error.DomError;
        if (try self.dom_store.hasAttribute(node_id, "open")) return error.ScriptRuntime;
        try self.dom_store.setAttribute(node_id, "open", "");
        return;
    }

    pub fn setDetailsOpen(self: *Session, node_id: dom.NodeId, should_open: bool) errors.Result(void) {
        const node = self.dom_store.nodeAt(node_id) orelse return error.DomError;
        if (!isDetailsNode(node)) return error.DomError;
        const is_open = try self.dom_store.hasAttribute(node_id, "open");
        if (should_open == is_open) return;
        if (should_open) {
            try self.dom_store.setAttribute(node_id, "open", "");
        } else {
            try self.dom_store.removeAttribute(node_id, "open");
        }
        _ = try self.dispatchDomEvent(node_id, "toggle", false, false);
        try self.flush();
        return;
    }

    pub fn closeDialog(self: *Session, node_id: dom.NodeId, return_value: ?[]const u8) errors.Result(void) {
        const node = self.dom_store.nodeAt(node_id) orelse return error.DomError;
        if (!isDialogNode(node)) return error.DomError;
        if (!try self.dom_store.hasAttribute(node_id, "open")) return;
        try self.dom_store.setDialogReturnValue(node_id, return_value orelse "");
        try self.dom_store.removeAttribute(node_id, "open");
        _ = try self.dispatchDomEvent(node_id, "close", false, false);
        try self.flush();
        return;
    }

    pub fn requestCloseDialog(self: *Session, node_id: dom.NodeId, return_value: ?[]const u8) errors.Result(void) {
        const node = self.dom_store.nodeAt(node_id) orelse return error.DomError;
        if (!isDialogNode(node)) return error.DomError;
        if (!try self.dom_store.hasAttribute(node_id, "open")) return;

        var runtime = script.ScriptRuntime{};
        const outcome = try runtime.dispatchDomEvent(self.arena.allocator(), self, node_id, "cancel", false, true);
        if (outcome.default_prevented) {
            try self.flush();
            return;
        }

        try self.closeDialog(node_id, return_value);
        return;
    }

    pub fn showPopover(self: *Session, node_id: dom.NodeId) errors.Result(void) {
        try self.ensureElementNode(node_id);
        const state = self.dom_store.popoverStateForNode(node_id) orelse return error.DomError;
        if (state == .no_popover) return error.ScriptRuntime;
        const visible = self.dom_store.popoverVisibleForNode(node_id) orelse return error.DomError;
        if (visible) return;

        try self.dom_store.setPopoverVisible(node_id, true);
        _ = try self.dispatchDomEvent(node_id, "toggle", false, false);
        try self.flush();
        return;
    }

    pub fn hidePopover(self: *Session, node_id: dom.NodeId) errors.Result(void) {
        try self.ensureElementNode(node_id);
        const state = self.dom_store.popoverStateForNode(node_id) orelse return error.DomError;
        if (state == .no_popover) return error.ScriptRuntime;
        const visible = self.dom_store.popoverVisibleForNode(node_id) orelse return error.DomError;
        if (!visible) return;

        try self.dom_store.setPopoverVisible(node_id, false);
        _ = try self.dispatchDomEvent(node_id, "toggle", false, false);
        try self.flush();
        return;
    }

    pub fn togglePopover(self: *Session, node_id: dom.NodeId, force: ?bool) errors.Result(bool) {
        try self.ensureElementNode(node_id);
        const state = self.dom_store.popoverStateForNode(node_id) orelse return error.DomError;
        if (state == .no_popover) return error.ScriptRuntime;
        const visible = self.dom_store.popoverVisibleForNode(node_id) orelse return error.DomError;
        const desired = force orelse !visible;
        if (desired == visible) {
            return desired;
        }

        try self.dom_store.setPopoverVisible(node_id, desired);
        _ = try self.dispatchDomEvent(node_id, "toggle", false, false);
        try self.flush();
        return desired;
    }

    pub fn reportValidityNode(self: *Session, node_id: dom.NodeId) errors.Result(bool) {
        try self.ensureElementNode(node_id);
        const node = self.dom_store.nodeAt(node_id) orelse return error.DomError;
        const tag_name = switch (node.kind) {
            .element => |element| element.tag_name,
            else => return error.DomError,
        };

        if (!std.mem.eql(u8, tag_name, "form") and
            !std.mem.eql(u8, tag_name, "input") and
            !std.mem.eql(u8, tag_name, "textarea") and
            !std.mem.eql(u8, tag_name, "select") and
            !std.mem.eql(u8, tag_name, "output") and
            !std.mem.eql(u8, tag_name, "object"))
        {
            return error.DomError;
        }

        var invalid_nodes: std.ArrayList(dom.NodeId) = .empty;
        defer invalid_nodes.deinit(std.heap.page_allocator);
        try self.dom_store.collectInvalidSubtreeNodes(node_id, &invalid_nodes, std.heap.page_allocator);
        if (invalid_nodes.items.len == 0) {
            return true;
        }

        for (invalid_nodes.items) |invalid_node_id| {
            _ = try self.dispatchDomEvent(invalid_node_id, "invalid", false, true);
        }
        try self.flush();
        return false;
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
        try runClickDefaultActionsForHost(self.arena.allocator(), self, node_id);
    }
};

const BootstrapHost = struct {
    dom_store: *dom.DomStore,
    listeners: *std.ArrayListUnmanaged(script.ScriptListenerRecord),
    match_media_listeners: *std.ArrayListUnmanaged(script.MatchMediaListenerRecord),
    match_media_onchange_listeners: *std.ArrayListUnmanaged(script.MatchMediaListenerRecord),
    queued_microtasks: *std.ArrayListUnmanaged(script.ScriptFunction),
    timers: *std.ArrayListUnmanaged(ScheduledTimer),
    next_timer_id: *u64,
    location: *mocks.LocationMocks,
    match_media: *mocks.MatchMediaMocks,
    open_mocks: *mocks.OpenMocks,
    close_mocks: *mocks.CloseMocks,
    print_mocks: *mocks.PrintMocks,
    scroll_mocks: *mocks.ScrollMocks,
    download_mocks: *mocks.DownloadMocks,
    history: *HistoryModel,
    allocator: std.mem.Allocator,
    window_name: *[]const u8,
    window_load_handler: *?script.ScriptFunction,
    window_beforeprint_handler: *?script.ScriptFunction,
    window_afterprint_handler: *?script.ScriptFunction,
    window_beforeunload_handler: *?script.ScriptFunction,
    window_unload_handler: *?script.ScriptFunction,
    window_focus_handler: *?script.ScriptFunction,
    window_blur_handler: *?script.ScriptFunction,
    window_pageshow_handler: *?script.ScriptFunction,
    window_pagehide_handler: *?script.ScriptFunction,
    document_scroll_handler: *?script.ScriptFunction,
    document_selectionchange_handler: *?script.ScriptFunction,
    document_readystatechange_handler: *?script.ScriptFunction,
    cookie_jar: *std.StringArrayHashMapUnmanaged([]const u8),
    scroll_x: *i64,
    scroll_y: *i64,
    history_scroll_restoration: *[]const u8,
    document_ready_state: *[]const u8,
    document_ready_state_change_pending: *bool,
    math_random_state: *u64,
    crypto_random_state: *u64,
    document_write_buffer: *std.ArrayListUnmanaged(u8),
    window_hashchange_handler: *?script.ScriptFunction,
    window_popstate_handler: *?script.ScriptFunction,
    window_storage_handler: *?script.ScriptFunction,
    window_scroll_handler: *?script.ScriptFunction,
    storage: *mocks.StorageSeeds,
    current_script: ?dom.NodeId = null,

    pub fn domStore(self: *const BootstrapHost) *const dom.DomStore {
        return self.dom_store;
    }

    pub fn domStoreMut(self: *BootstrapHost) *dom.DomStore {
        return self.dom_store;
    }

    pub fn scriptEventListeners(self: *const BootstrapHost) []const script.ScriptListenerRecord {
        return self.listeners.items;
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

    pub fn documentScrollingElement(self: *const BootstrapHost) ?dom.NodeId {
        return self.dom_store.documentElement() orelse self.dom_store.documentBody();
    }

    pub fn documentTitle(self: *const BootstrapHost) []const u8 {
        return self.dom_store.documentTitle();
    }

    pub fn documentOpen(self: *BootstrapHost) errors.Result(void) {
        try self.dom_store.clearDocument();
        self.document_write_buffer.clearRetainingCapacity();
        self.document_ready_state.* = "loading";
        self.document_ready_state_change_pending.* = true;
        return;
    }

    pub fn documentWrite(self: *BootstrapHost, markup: []const u8) errors.Result(void) {
        try self.document_write_buffer.appendSlice(self.allocator, markup);
        return;
    }

    pub fn documentClose(self: *BootstrapHost) errors.Result(void) {
        if (self.document_write_buffer.items.len > 0) {
            try self.dom_store.appendHtmlToDocument(self.document_write_buffer.items);
            self.document_write_buffer.clearRetainingCapacity();
        }
        var runtime = script.ScriptRuntime{};
        try runtime.dispatchDocumentEvent(
            self.allocator,
            self,
            "DOMContentLoaded",
            self.listeners.items,
            null,
            "onDOMContentLoaded",
        );
        self.document_ready_state.* = "complete";
        if (self.document_ready_state_change_pending.*) {
            try self.dispatchDocumentReadyStateChange();
            self.document_ready_state_change_pending.* = false;
        }
        return;
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

    pub fn documentDomain(self: *const BootstrapHost, allocator: std.mem.Allocator) errors.Result([]const u8) {
        return try domainFromUrl(allocator, self.currentLocationUrl());
    }

    pub fn documentCookie(self: *const BootstrapHost, allocator: std.mem.Allocator) errors.Result([]const u8) {
        return try cookieJarText(allocator, self.cookie_jar);
    }

    pub fn documentVisibilityState(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "visible";
    }

    pub fn documentHidden(self: *const BootstrapHost) bool {
        _ = self;
        return false;
    }

    pub fn documentHasFocus(self: *const BootstrapHost) bool {
        _ = self;
        return true;
    }

    pub fn windowName(self: *const BootstrapHost) []const u8 {
        return self.window_name.*;
    }

    pub fn windowLoad(self: *const BootstrapHost) ?script.ScriptFunction {
        return self.window_load_handler.*;
    }

    pub fn setWindowLoad(self: *BootstrapHost, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_load_handler.* = try duplicateScriptFunction(self.allocator, function);
            return;
        }
        self.window_load_handler.* = null;
        return;
    }

    pub fn windowBeforePrint(self: *const BootstrapHost) ?script.ScriptFunction {
        return self.window_beforeprint_handler.*;
    }

    pub fn setWindowBeforePrint(self: *BootstrapHost, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_beforeprint_handler.* = try duplicateScriptFunction(self.allocator, function);
            return;
        }
        self.window_beforeprint_handler.* = null;
        return;
    }

    pub fn windowAfterPrint(self: *const BootstrapHost) ?script.ScriptFunction {
        return self.window_afterprint_handler.*;
    }

    pub fn setWindowAfterPrint(self: *BootstrapHost, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_afterprint_handler.* = try duplicateScriptFunction(self.allocator, function);
            return;
        }
        self.window_afterprint_handler.* = null;
        return;
    }

    pub fn windowBeforeUnload(self: *const BootstrapHost) ?script.ScriptFunction {
        return self.window_beforeunload_handler.*;
    }

    pub fn setWindowBeforeUnload(self: *BootstrapHost, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_beforeunload_handler.* = try duplicateScriptFunction(self.allocator, function);
            return;
        }
        self.window_beforeunload_handler.* = null;
        return;
    }

    pub fn windowUnload(self: *const BootstrapHost) ?script.ScriptFunction {
        return self.window_unload_handler.*;
    }

    pub fn setWindowUnload(self: *BootstrapHost, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_unload_handler.* = try duplicateScriptFunction(self.allocator, function);
            return;
        }
        self.window_unload_handler.* = null;
        return;
    }

    pub fn windowFocus(self: *const BootstrapHost) ?script.ScriptFunction {
        return self.window_focus_handler.*;
    }

    pub fn setWindowName(self: *BootstrapHost, value: []const u8) errors.Result(void) {
        self.window_name.* = try self.allocator.dupe(u8, value);
        return;
    }

    pub fn setWindowFocus(self: *BootstrapHost, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_focus_handler.* = try duplicateScriptFunction(self.allocator, function);
            return;
        }
        self.window_focus_handler.* = null;
        return;
    }

    pub fn windowHashChange(self: *const BootstrapHost) ?script.ScriptFunction {
        return self.window_hashchange_handler.*;
    }

    pub fn setWindowHashChange(self: *BootstrapHost, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_hashchange_handler.* = try duplicateScriptFunction(self.allocator, function);
            return;
        }
        self.window_hashchange_handler.* = null;
        return;
    }

    pub fn windowBlur(self: *const BootstrapHost) ?script.ScriptFunction {
        return self.window_blur_handler.*;
    }

    pub fn windowPageShow(self: *const BootstrapHost) ?script.ScriptFunction {
        return self.window_pageshow_handler.*;
    }

    pub fn windowPageHide(self: *const BootstrapHost) ?script.ScriptFunction {
        return self.window_pagehide_handler.*;
    }

    pub fn documentScroll(self: *const BootstrapHost) ?script.ScriptFunction {
        return self.document_scroll_handler.*;
    }

    pub fn documentSelectionChange(self: *const BootstrapHost) ?script.ScriptFunction {
        return self.document_selectionchange_handler.*;
    }

    pub fn documentReadyStateChange(self: *const BootstrapHost) ?script.ScriptFunction {
        return self.document_readystatechange_handler.*;
    }

    pub fn setWindowBlur(self: *BootstrapHost, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_blur_handler.* = try duplicateScriptFunction(self.allocator, function);
            return;
        }
        self.window_blur_handler.* = null;
        return;
    }

    pub fn setWindowPageShow(self: *BootstrapHost, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_pageshow_handler.* = try duplicateScriptFunction(self.allocator, function);
            return;
        }
        self.window_pageshow_handler.* = null;
        return;
    }

    pub fn setWindowPageHide(self: *BootstrapHost, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_pagehide_handler.* = try duplicateScriptFunction(self.allocator, function);
            return;
        }
        self.window_pagehide_handler.* = null;
        return;
    }

    pub fn setDocumentScroll(self: *BootstrapHost, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.document_scroll_handler.* = try duplicateScriptFunction(self.allocator, function);
            return;
        }
        self.document_scroll_handler.* = null;
        return;
    }

    pub fn setDocumentSelectionChange(self: *BootstrapHost, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.document_selectionchange_handler.* = try duplicateScriptFunction(self.allocator, function);
            return;
        }
        self.document_selectionchange_handler.* = null;
        return;
    }

    pub fn setDocumentReadyStateChange(self: *BootstrapHost, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.document_readystatechange_handler.* = try duplicateScriptFunction(self.allocator, function);
            return;
        }
        self.document_readystatechange_handler.* = null;
        return;
    }

    pub fn documentReadyStateChangePending(self: *const BootstrapHost) bool {
        return self.document_ready_state_change_pending.*;
    }

    pub fn setDocumentReadyStateChangePending(self: *BootstrapHost, pending: bool) void {
        self.document_ready_state_change_pending.* = pending;
    }

    pub fn windowPopState(self: *const BootstrapHost) ?script.ScriptFunction {
        return self.window_popstate_handler.*;
    }

    pub fn setWindowPopState(self: *BootstrapHost, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_popstate_handler.* = try duplicateScriptFunction(self.allocator, function);
            return;
        }
        self.window_popstate_handler.* = null;
        return;
    }

    pub fn windowStorage(self: *const BootstrapHost) ?script.ScriptFunction {
        return self.window_storage_handler.*;
    }

    pub fn setWindowStorage(self: *BootstrapHost, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_storage_handler.* = try duplicateScriptFunction(self.allocator, function);
            return;
        }
        self.window_storage_handler.* = null;
        return;
    }

    pub fn windowScroll(self: *const BootstrapHost) ?script.ScriptFunction {
        return self.window_scroll_handler.*;
    }

    pub fn setWindowScroll(self: *BootstrapHost, handler: ?script.ScriptFunction) errors.Result(void) {
        if (handler) |function| {
            self.window_scroll_handler.* = try duplicateScriptFunction(self.allocator, function);
            return;
        }
        self.window_scroll_handler.* = null;
        return;
    }

    pub fn setDocumentCookie(self: *BootstrapHost, value: []const u8) errors.Result(void) {
        try cookieJarSet(self.allocator, self.cookie_jar, value);
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

    pub fn windowNavigatorUserAgent(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "browser_tester";
    }

    pub fn windowNavigatorAppCodeName(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "browser_tester";
    }

    pub fn windowNavigatorAppName(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "browser_tester";
    }

    pub fn windowNavigatorAppVersion(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "browser_tester";
    }

    pub fn windowNavigatorProduct(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "browser_tester";
    }

    pub fn windowNavigatorProductSub(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "browser_tester";
    }

    pub fn windowNavigatorVendor(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "browser_tester";
    }

    pub fn windowNavigatorVendorSub(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "";
    }

    pub fn windowNavigatorPdfViewerEnabled(self: *const BootstrapHost) bool {
        _ = self;
        return false;
    }

    pub fn windowNavigatorDoNotTrack(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "unspecified";
    }

    pub fn windowNavigatorPlatform(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "unknown";
    }

    pub fn windowNavigatorLanguage(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "en-US";
    }

    pub fn windowNavigatorUserLanguage(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "en-US";
    }

    pub fn windowNavigatorBrowserLanguage(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "en-US";
    }

    pub fn windowNavigatorSystemLanguage(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "en-US";
    }

    pub fn windowNavigatorOscpu(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "unknown";
    }

    pub fn windowNavigatorLanguages(self: *const BootstrapHost) []const []const u8 {
        _ = self;
        return &.{"en-US"};
    }

    pub fn windowNavigatorJavaEnabled(self: *const BootstrapHost) bool {
        _ = self;
        return false;
    }

    pub fn windowNavigatorCookieEnabled(self: *const BootstrapHost) bool {
        _ = self;
        return true;
    }

    pub fn windowNavigatorOnLine(self: *const BootstrapHost) bool {
        _ = self;
        return true;
    }

    pub fn windowNavigatorWebdriver(self: *const BootstrapHost) bool {
        _ = self;
        return false;
    }

    pub fn windowNavigatorHardwareConcurrency(self: *const BootstrapHost) i64 {
        _ = self;
        return 8;
    }

    pub fn windowNavigatorMaxTouchPoints(self: *const BootstrapHost) i64 {
        _ = self;
        return 0;
    }

    pub fn windowDevicePixelRatio(self: *const BootstrapHost) f64 {
        _ = self;
        return 1.0;
    }

    pub fn windowInnerWidth(self: *const BootstrapHost) i64 {
        _ = self;
        return 1024;
    }

    pub fn windowInnerHeight(self: *const BootstrapHost) i64 {
        _ = self;
        return 768;
    }

    pub fn windowOuterWidth(self: *const BootstrapHost) i64 {
        _ = self;
        return 1280;
    }

    pub fn windowOuterHeight(self: *const BootstrapHost) i64 {
        _ = self;
        return 800;
    }

    pub fn windowScreenWidth(self: *const BootstrapHost) i64 {
        _ = self;
        return 1280;
    }

    pub fn windowScreenHeight(self: *const BootstrapHost) i64 {
        _ = self;
        return 800;
    }

    pub fn windowScreenAvailWidth(self: *const BootstrapHost) i64 {
        _ = self;
        return 1280;
    }

    pub fn windowScreenAvailHeight(self: *const BootstrapHost) i64 {
        _ = self;
        return 800;
    }

    pub fn windowScreenAvailLeft(self: *const BootstrapHost) i64 {
        _ = self;
        return 0;
    }

    pub fn windowScreenAvailTop(self: *const BootstrapHost) i64 {
        _ = self;
        return 0;
    }

    pub fn windowScreenColorDepth(self: *const BootstrapHost) i64 {
        _ = self;
        return 24;
    }

    pub fn windowScreenPixelDepth(self: *const BootstrapHost) i64 {
        _ = self;
        return 24;
    }

    pub fn windowScreenOrientationType(self: *const BootstrapHost) []const u8 {
        _ = self;
        return "landscape-primary";
    }

    pub fn windowScreenOrientationAngle(self: *const BootstrapHost) i64 {
        _ = self;
        return 0;
    }

    pub fn windowScreenX(self: *const BootstrapHost) i64 {
        _ = self;
        return 0;
    }

    pub fn windowScreenY(self: *const BootstrapHost) i64 {
        _ = self;
        return 0;
    }

    pub fn windowScreenLeft(self: *const BootstrapHost) i64 {
        _ = self;
        return 0;
    }

    pub fn windowScreenTop(self: *const BootstrapHost) i64 {
        _ = self;
        return 0;
    }

    pub fn mathRandom(self: *BootstrapHost) f64 {
        self.math_random_state.* = self.math_random_state.* *% 73 +% 41;
        return @as(f64, @floatFromInt(self.math_random_state.* % 1000)) / 1000.0;
    }

    pub fn cryptoRandomUUID(self: *BootstrapHost, allocator: std.mem.Allocator) errors.Result([]const u8) {
        return try cryptoRandomUuidFromState(allocator, self.crypto_random_state);
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
            if (std.mem.eql(u8, value_ptr.*, value)) {
                return;
            }
            value_ptr.* = try self.allocator.dupe(u8, value);
            try self.dispatchStorageEvent();
            return;
        }

        const key_copy = try self.allocator.dupe(u8, key);
        const value_copy = try self.allocator.dupe(u8, value);
        try storage.put(self.allocator, key_copy, value_copy);
        try self.dispatchStorageEvent();
        return;
    }

    pub fn storageRemoveItem(self: *BootstrapHost, target: script.StorageTarget, key: []const u8) errors.Result(void) {
        if (self.storageMapMut(target).contains(key)) {
            _ = self.storageMapMut(target).orderedRemove(key);
            try self.dispatchStorageEvent();
        }
        return;
    }

    pub fn storageClear(self: *BootstrapHost, target: script.StorageTarget) errors.Result(void) {
        if (self.storageMapMut(target).count() > 0) {
            self.storageMapMut(target).clearRetainingCapacity();
            try self.dispatchStorageEvent();
        }
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

    fn dispatchStorageEvent(self: *BootstrapHost) errors.Result(void) {
        var runtime = script.ScriptRuntime{};
        try runtime.dispatchWindowEvent(
            self.allocator,
            self,
            "storage",
            self.listeners.items,
            self.windowStorage(),
            "onstorage",
        );
        return;
    }

    pub fn dispatchSelectionChange(self: *BootstrapHost) errors.Result(void) {
        var runtime = script.ScriptRuntime{};
        try runtime.dispatchDocumentEvent(
            self.allocator,
            self,
            "selectionchange",
            self.listeners.items,
            self.documentSelectionChange(),
            "onselectionchange",
        );
        return;
    }

    pub fn dispatchDocumentReadyStateChange(self: *BootstrapHost) errors.Result(void) {
        var runtime = script.ScriptRuntime{};
        try runtime.dispatchDocumentEvent(
            self.allocator,
            self,
            "readystatechange",
            self.listeners.items,
            self.documentReadyStateChange(),
            "onreadystatechange",
        );
        return;
    }

    pub fn documentActiveElement(self: *const BootstrapHost) ?dom.NodeId {
        return self.dom_store.activeElement();
    }

    pub fn documentReadyState(self: *const BootstrapHost) []const u8 {
        return self.document_ready_state.*;
    }

    pub fn setDocumentReadyState(self: *BootstrapHost, value: []const u8) void {
        self.document_ready_state.* = value;
    }

    pub fn nowMs(self: *const BootstrapHost) i64 {
        _ = self;
        return 0;
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

        const before_url = self.currentLocationUrl();
        const target = try self.history.push(null, trimmed);
        const page_lifecycle = shouldDispatchPageLifecycle(before_url, target);
        if (page_lifecycle) {
            var runtime = script.ScriptRuntime{};
            try runtime.dispatchWindowEvent(
                self.allocator,
                self,
                "beforeunload",
                self.listeners.items,
                self.windowBeforeUnload(),
                "onbeforeunload",
            );
            try runtime.dispatchWindowEvent(
                self.allocator,
                self,
                "pagehide",
                self.listeners.items,
                self.windowPageHide(),
                "onpagehide",
            );
            try runtime.dispatchWindowEvent(
                self.allocator,
                self,
                "unload",
                self.listeners.items,
                self.windowUnload(),
                "onunload",
            );
        }
        try syncLocationState(self.location, self.dom_store, target, true);
        self.resetScrollPosition();
        if (page_lifecycle) {
            var runtime = script.ScriptRuntime{};
            try runtime.dispatchWindowEvent(
                self.allocator,
                self,
                "pageshow",
                self.listeners.items,
                self.windowPageShow(),
                "onpageshow",
            );
        }
        return;
    }

    pub fn replaceLocation(self: *BootstrapHost, url_source: []const u8) errors.Result(void) {
        const trimmed = std.mem.trim(u8, url_source, " \t\r\n");
        if (trimmed.len == 0) return error.MockError;

        const before_url = self.currentLocationUrl();
        const target = try self.history.replace(null, trimmed);
        const page_lifecycle = shouldDispatchPageLifecycle(before_url, target);
        if (page_lifecycle) {
            var runtime = script.ScriptRuntime{};
            try runtime.dispatchWindowEvent(
                self.allocator,
                self,
                "beforeunload",
                self.listeners.items,
                self.windowBeforeUnload(),
                "onbeforeunload",
            );
            try runtime.dispatchWindowEvent(
                self.allocator,
                self,
                "pagehide",
                self.listeners.items,
                self.windowPageHide(),
                "onpagehide",
            );
            try runtime.dispatchWindowEvent(
                self.allocator,
                self,
                "unload",
                self.listeners.items,
                self.windowUnload(),
                "onunload",
            );
        }
        try syncLocationState(self.location, self.dom_store, target, true);
        self.resetScrollPosition();
        if (page_lifecycle) {
            var runtime = script.ScriptRuntime{};
            try runtime.dispatchWindowEvent(
                self.allocator,
                self,
                "pageshow",
                self.listeners.items,
                self.windowPageShow(),
                "onpageshow",
            );
        }
        return;
    }

    pub fn reloadLocation(self: *BootstrapHost) errors.Result(void) {
        const target = self.history.current();
        var runtime = script.ScriptRuntime{};
        try runtime.dispatchWindowEvent(
            self.allocator,
            self,
            "beforeunload",
            self.listeners.items,
            self.windowBeforeUnload(),
            "onbeforeunload",
        );
        try runtime.dispatchWindowEvent(
            self.allocator,
            self,
            "pagehide",
            self.listeners.items,
            self.windowPageHide(),
            "onpagehide",
        );
        try runtime.dispatchWindowEvent(
            self.allocator,
            self,
            "unload",
            self.listeners.items,
            self.windowUnload(),
            "onunload",
        );
        try syncLocationState(self.location, self.dom_store, target, true);
        self.resetScrollPosition();
        try runtime.dispatchWindowEvent(
            self.allocator,
            self,
            "pageshow",
            self.listeners.items,
            self.windowPageShow(),
            "onpageshow",
        );
        return;
    }

    pub fn locationHash(self: *const BootstrapHost, allocator: std.mem.Allocator) errors.Result([]const u8) {
        return try locationHashFromUrl(allocator, self.currentLocationUrl());
    }

    pub fn setLocationHash(self: *BootstrapHost, value: []const u8) errors.Result(void) {
        const target = try locationUrlWithHash(self.allocator, self.currentLocationUrl(), value);
        try self.assignLocation(target);
        return;
    }

    pub fn historyLength(self: *const BootstrapHost) usize {
        return self.history.length();
    }

    pub fn historyState(self: *const BootstrapHost) ?[]const u8 {
        return self.history.currentState();
    }

    pub fn historyScrollRestoration(self: *const BootstrapHost) []const u8 {
        return self.history_scroll_restoration.*;
    }

    pub fn setHistoryScrollRestoration(self: *BootstrapHost, value: []const u8) errors.Result(void) {
        if (!std.mem.eql(u8, value, "auto") and !std.mem.eql(u8, value, "manual")) {
            return error.ScriptRuntime;
        }

        self.history_scroll_restoration.* = value;
        return;
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
            var runtime = script.ScriptRuntime{};
            try runtime.dispatchWindowEvent(
                self.allocator,
                self,
                "popstate",
                self.listeners.items,
                self.windowPopState(),
                "onpopstate",
            );
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

    pub fn submitNode(self: *BootstrapHost, node_id: dom.NodeId, submitter: ?dom.NodeId) errors.Result(void) {
        const node = self.dom_store.nodeAt(node_id) orelse return error.DomError;
        const form_id = if (isFormNode(node))
            node_id
        else
            self.findAssociatedForm(node_id) orelse return error.DomError;

        var runtime = script.ScriptRuntime{};
        _ = try runtime.dispatchSubmitEvent(self.allocator, self, form_id, submitter);
        return;
    }

    pub fn resetNode(self: *BootstrapHost, node_id: dom.NodeId) errors.Result(void) {
        const node = self.dom_store.nodeAt(node_id) orelse return error.DomError;
        if (!isFormNode(node)) return error.DomError;

        var runtime = script.ScriptRuntime{};
        _ = try runtime.dispatchDomEvent(self.allocator, self, node_id, "reset", true, true);
        return;
    }

    pub fn showDialog(self: *BootstrapHost, node_id: dom.NodeId) errors.Result(void) {
        const node = self.dom_store.nodeAt(node_id) orelse return error.DomError;
        if (!isDialogNode(node)) return error.DomError;
        if (try self.dom_store.hasAttribute(node_id, "open")) return error.ScriptRuntime;
        try self.dom_store.setAttribute(node_id, "open", "");
        return;
    }

    pub fn setDetailsOpen(self: *BootstrapHost, node_id: dom.NodeId, should_open: bool) errors.Result(void) {
        const node = self.dom_store.nodeAt(node_id) orelse return error.DomError;
        if (!isDetailsNode(node)) return error.DomError;
        const is_open = try self.dom_store.hasAttribute(node_id, "open");
        if (should_open == is_open) return;
        if (should_open) {
            try self.dom_store.setAttribute(node_id, "open", "");
        } else {
            try self.dom_store.removeAttribute(node_id, "open");
        }
        var runtime = script.ScriptRuntime{};
        _ = try runtime.dispatchDomEvent(self.allocator, self, node_id, "toggle", false, false);
        return;
    }

    pub fn closeDialog(self: *BootstrapHost, node_id: dom.NodeId, return_value: ?[]const u8) errors.Result(void) {
        const node = self.dom_store.nodeAt(node_id) orelse return error.DomError;
        if (!isDialogNode(node)) return error.DomError;
        if (!try self.dom_store.hasAttribute(node_id, "open")) return;
        try self.dom_store.setDialogReturnValue(node_id, return_value orelse "");
        try self.dom_store.removeAttribute(node_id, "open");
        var runtime = script.ScriptRuntime{};
        _ = try runtime.dispatchDomEvent(self.allocator, self, node_id, "close", false, false);
        return;
    }

    pub fn requestCloseDialog(self: *BootstrapHost, node_id: dom.NodeId, return_value: ?[]const u8) errors.Result(void) {
        const node = self.dom_store.nodeAt(node_id) orelse return error.DomError;
        if (!isDialogNode(node)) return error.DomError;
        if (!try self.dom_store.hasAttribute(node_id, "open")) return;

        var runtime = script.ScriptRuntime{};
        const outcome = try runtime.dispatchDomEvent(self.allocator, self, node_id, "cancel", false, true);
        if (outcome.default_prevented) return;

        try self.closeDialog(node_id, return_value);
        return;
    }

    pub fn showPopover(self: *BootstrapHost, node_id: dom.NodeId) errors.Result(void) {
        const state = self.dom_store.popoverStateForNode(node_id) orelse return error.DomError;
        if (state == .no_popover) return error.ScriptRuntime;
        const visible = self.dom_store.popoverVisibleForNode(node_id) orelse return error.DomError;
        if (visible) return;

        try self.dom_store.setPopoverVisible(node_id, true);
        var runtime = script.ScriptRuntime{};
        _ = try runtime.dispatchDomEvent(self.allocator, self, node_id, "toggle", false, false);
        return;
    }

    pub fn hidePopover(self: *BootstrapHost, node_id: dom.NodeId) errors.Result(void) {
        const state = self.dom_store.popoverStateForNode(node_id) orelse return error.DomError;
        if (state == .no_popover) return error.ScriptRuntime;
        const visible = self.dom_store.popoverVisibleForNode(node_id) orelse return error.DomError;
        if (!visible) return;

        try self.dom_store.setPopoverVisible(node_id, false);
        var runtime = script.ScriptRuntime{};
        _ = try runtime.dispatchDomEvent(self.allocator, self, node_id, "toggle", false, false);
        return;
    }

    pub fn togglePopover(self: *BootstrapHost, node_id: dom.NodeId, force: ?bool) errors.Result(bool) {
        const state = self.dom_store.popoverStateForNode(node_id) orelse return error.DomError;
        if (state == .no_popover) return error.ScriptRuntime;
        const visible = self.dom_store.popoverVisibleForNode(node_id) orelse return error.DomError;
        const desired = force orelse !visible;
        if (desired == visible) {
            return desired;
        }

        try self.dom_store.setPopoverVisible(node_id, desired);
        var runtime = script.ScriptRuntime{};
        _ = try runtime.dispatchDomEvent(self.allocator, self, node_id, "toggle", false, false);
        return desired;
    }

    pub fn reportValidityNode(self: *BootstrapHost, node_id: dom.NodeId) errors.Result(bool) {
        const node = self.dom_store.nodeAt(node_id) orelse return error.DomError;
        const tag_name = switch (node.kind) {
            .element => |element| element.tag_name,
            else => return error.DomError,
        };

        if (!std.mem.eql(u8, tag_name, "form") and
            !std.mem.eql(u8, tag_name, "input") and
            !std.mem.eql(u8, tag_name, "textarea") and
            !std.mem.eql(u8, tag_name, "select") and
            !std.mem.eql(u8, tag_name, "output") and
            !std.mem.eql(u8, tag_name, "object"))
        {
            return error.DomError;
        }

        var invalid_nodes: std.ArrayList(dom.NodeId) = .empty;
        defer invalid_nodes.deinit(self.allocator);
        try self.dom_store.collectInvalidSubtreeNodes(node_id, &invalid_nodes, self.allocator);
        if (invalid_nodes.items.len == 0) {
            return true;
        }

        var runtime = script.ScriptRuntime{};
        for (invalid_nodes.items) |invalid_node_id| {
            _ = try runtime.dispatchDomEvent(self.allocator, self, invalid_node_id, "invalid", false, true);
        }
        return false;
    }

    pub fn matchMedia(self: *BootstrapHost, query_source: []const u8) errors.Result(bool) {
        return matchMediaQuery(self.match_media, query_source);
    }

    pub fn matchMediaCurrent(self: *const BootstrapHost, query_source: []const u8) bool {
        return self.match_media.currentMatch(query_source) orelse false;
    }

    pub fn matchMediaOnChange(self: *const BootstrapHost, query_source: []const u8) ?script.ScriptFunction {
        return findMatchMediaOnChange(self.match_media_onchange_listeners, query_source);
    }

    pub fn setMatchMediaOnChange(
        self: *BootstrapHost,
        query_source: []const u8,
        handler: ?script.ScriptFunction,
    ) errors.Result(void) {
        if (handler) |function| {
            try upsertMatchMediaOnChange(
                self.allocator,
                self.match_media_onchange_listeners,
                query_source,
                function,
                self.matchMediaCurrent(query_source),
            );
            return;
        }
        removeMatchMediaOnChange(self.match_media_onchange_listeners, query_source);
        return;
    }

    pub fn registerMatchMediaListener(
        self: *BootstrapHost,
        query_source: []const u8,
        handler: script.ScriptFunction,
    ) errors.Result(void) {
        try appendMatchMediaListener(
            self.allocator,
            self.match_media_listeners,
            query_source,
            handler,
            self.matchMediaCurrent(query_source),
        );
        return;
    }

    pub fn unregisterMatchMediaListener(
        self: *BootstrapHost,
        query_source: []const u8,
        handler: script.ScriptFunction,
    ) errors.Result(void) {
        removeMatchMediaListener(self.match_media_listeners, query_source, handler);
        return;
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

    pub fn captureDownload(
        self: *BootstrapHost,
        file_name: []const u8,
        bytes: []const u8,
    ) errors.Result(void) {
        try self.download_mocks.capture(file_name, bytes);
        return;
    }

    pub fn close(self: *BootstrapHost) errors.Result(void) {
        try self.close_mocks.recordCall();
        return;
    }

    pub fn print(self: *BootstrapHost) errors.Result(void) {
        try self.print_mocks.recordCall();
        var runtime = script.ScriptRuntime{};
        try runtime.dispatchWindowEvent(
            self.allocator,
            self,
            "beforeprint",
            self.listeners.items,
            self.windowBeforePrint(),
            "onbeforeprint",
        );
        try runtime.dispatchWindowEvent(
            self.allocator,
            self,
            "afterprint",
            self.listeners.items,
            self.windowAfterPrint(),
            "onafterprint",
        );
        return;
    }

    pub fn scrollTo(self: *BootstrapHost, x: i64, y: i64) errors.Result(void) {
        try self.scroll_mocks.recordCall(.To, x, y);
        const old_x = self.scroll_x.*;
        const old_y = self.scroll_y.*;
        self.scroll_x.* = x;
        self.scroll_y.* = y;
        if (old_x != x or old_y != y) {
            var runtime = script.ScriptRuntime{};
            try runtime.dispatchDocumentEvent(
                self.allocator,
                self,
                "scroll",
                self.listeners.items,
                self.documentScroll(),
                "onscroll",
            );
            try runtime.dispatchWindowEvent(
                self.allocator,
                self,
                "scroll",
                self.listeners.items,
                self.windowScroll(),
                "onscroll",
            );
        }
        return;
    }

    pub fn scrollBy(self: *BootstrapHost, x: i64, y: i64) errors.Result(void) {
        try self.scroll_mocks.recordCall(.By, x, y);
        const next_x = std.math.add(i64, self.scroll_x.*, x) catch return error.ScriptRuntime;
        const next_y = std.math.add(i64, self.scroll_y.*, y) catch return error.ScriptRuntime;
        const old_x = self.scroll_x.*;
        const old_y = self.scroll_y.*;
        self.scroll_x.* = next_x;
        self.scroll_y.* = next_y;
        if (old_x != next_x or old_y != next_y) {
            var runtime = script.ScriptRuntime{};
            try runtime.dispatchDocumentEvent(
                self.allocator,
                self,
                "scroll",
                self.listeners.items,
                self.documentScroll(),
                "onscroll",
            );
            try runtime.dispatchWindowEvent(
                self.allocator,
                self,
                "scroll",
                self.listeners.items,
                self.windowScroll(),
                "onscroll",
            );
        }
        return;
    }

    pub fn clickNode(self: *BootstrapHost, node_id: dom.NodeId) errors.Result(void) {
        var runtime = script.ScriptRuntime{};
        const outcome = try runtime.dispatchDomEvent(self.allocator, self, node_id, "click", true, true);
        if (!outcome.default_prevented) {
            try runClickDefaultActionsForHost(self.allocator, self, node_id);
        }
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

    pub fn queueMicrotask(self: *BootstrapHost, handler: script.ScriptFunction) errors.Result(void) {
        try appendQueuedMicrotask(self.allocator, self.queued_microtasks, handler);
        return;
    }

    pub fn scheduleTimer(
        self: *BootstrapHost,
        handler: script.ScriptFunction,
        delay_ms: i64,
    ) errors.Result(u64) {
        return try appendScheduledTimer(
            self.allocator,
            self.next_timer_id,
            self.timers,
            delay_ms,
            null,
            .timeout,
            handler,
        );
    }

    pub fn scheduleIntervalTimer(
        self: *BootstrapHost,
        handler: script.ScriptFunction,
        delay_ms: i64,
    ) errors.Result(u64) {
        return try appendScheduledTimer(
            self.allocator,
            self.next_timer_id,
            self.timers,
            delay_ms,
            delay_ms,
            .interval,
            handler,
        );
    }

    pub fn scheduleAnimationFrame(
        self: *BootstrapHost,
        handler: script.ScriptFunction,
    ) errors.Result(u64) {
        return try appendScheduledTimer(
            self.allocator,
            self.next_timer_id,
            self.timers,
            try nextAnimationFrameTime(0),
            null,
            .animation_frame,
            handler,
        );
    }

    pub fn clearTimer(self: *BootstrapHost, timer_id: u64) void {
        clearScheduledTimer(self.timers, timer_id);
    }

    fn findAssociatedForm(self: *const BootstrapHost, node_id: dom.NodeId) ?dom.NodeId {
        var current = self.dom_store.nodeAt(node_id).?.parent;
        while (current) |parent_id| {
            const parent = self.dom_store.nodeAt(parent_id) orelse break;
            if (isFormNode(parent)) return parent_id;
            current = parent.parent;
        }

        return null;
    }
};

fn runClickDefaultActionsForHost(
    allocator: std.mem.Allocator,
    host: anytype,
    node_id: dom.NodeId,
) errors.Result(void) {
    const node = host.domStore().nodeAt(node_id) orelse return error.DomError;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return error.DomError,
    };

    const input_type = elementAttributeValue(element, "type");
    const form_owner = host.findAssociatedForm(node_id);
    if (std.mem.eql(u8, element.tag_name, "input") and isCheckableInputType(input_type)) {
        const checked = host.domStore().checkedForNode(node_id) orelse false;
        try host.domStoreMut().setFormControlChecked(node_id, !checked);
        var runtime = script.ScriptRuntime{};
        _ = try runtime.dispatchDomEvent(allocator, host, node_id, "input", true, false);
        _ = try runtime.dispatchDomEvent(allocator, host, node_id, "change", true, false);
    }

    const is_button = std.mem.eql(u8, element.tag_name, "button");
    const should_submit = if (is_button)
        try script.buttonIsSubmitButton(host, node_id)
    else
        isSubmitControl(element.tag_name, input_type);

    if (should_submit) {
        if (form_owner) |form_id| {
            var runtime = script.ScriptRuntime{};
            _ = try runtime.dispatchSubmitEvent(allocator, host, form_id, node_id);
            return;
        }
    }

    if (isResetControl(element.tag_name, input_type)) {
        if (form_owner) |form_id| {
            try host.resetNode(form_id);
            return;
        }
    }

    if (std.mem.eql(u8, element.tag_name, "summary")) {
        if (node.parent) |parent_id| {
            const parent = host.domStore().nodeAt(parent_id) orelse return error.DomError;
            if (isDetailsNode(parent)) {
                if (dom.firstElementChild(host.domStore(), parent_id)) |first_summary_id| {
                    if (first_summary_id.index == node_id.index and first_summary_id.generation == node_id.generation) {
                        const is_open = try host.domStore().hasAttribute(parent_id, "open");
                        try host.setDetailsOpen(parent_id, !is_open);
                        return;
                    }
                }
            }
        }
    }

    if (form_owner != null and std.mem.eql(u8, element.tag_name, "input")) {
        const type_text = input_type orelse "text";
        if (!std.mem.eql(u8, type_text, "button")) {
            return;
        }
    }

    if (is_button) {
        const command_target = try script.buttonCommandForElement(host, node_id);
        if (command_target) |target_id| {
            const command = try script.buttonCommandValue(host, node_id);
            if (!try script.buttonCommandValidForTarget(host, target_id, command)) {
                return;
            }

            var runtime = script.ScriptRuntime{};
            const outcome = try runtime.dispatchCommandEvent(allocator, host, target_id, node_id, command);
            if (outcome.default_prevented) return;
            if (!dom.isConnected(host.domStore(), target_id)) return;

            if (command.len == 0) return;
            if (command.len >= 2 and command[0] == '-' and command[1] == '-') return;

            if (std.mem.eql(u8, command, "toggle-popover")) {
                if (host.domStore().popoverStateForNode(target_id)) |state| {
                    if (state == .no_popover) return;
                    if (host.domStore().popoverVisibleForNode(target_id) orelse return error.DomError) {
                        try host.hidePopover(target_id);
                    } else {
                        try host.showPopover(target_id);
                    }
                }
                return;
            }

            if (std.mem.eql(u8, command, "show-popover")) {
                if (host.domStore().popoverStateForNode(target_id)) |_| {
                    try host.showPopover(target_id);
                }
                return;
            }

            if (std.mem.eql(u8, command, "hide-popover")) {
                if (host.domStore().popoverStateForNode(target_id)) |_| {
                    try host.hidePopover(target_id);
                }
                return;
            }

            if (std.mem.eql(u8, command, "close") or
                std.mem.eql(u8, command, "request-close") or
                std.mem.eql(u8, command, "show-modal"))
            {
                const target_node = host.domStore().nodeAt(target_id) orelse return error.DomError;
                const target_element = switch (target_node.kind) {
                    .element => |target_element| target_element,
                    else => return error.DomError,
                };
                if (!std.mem.eql(u8, target_element.tag_name, "dialog")) return;

                const return_value = elementAttributeValue(element, "value");
                if (std.mem.eql(u8, command, "close")) {
                    if (!try host.domStore().hasAttribute(target_id, "open")) return;
                    try host.closeDialog(target_id, return_value);
                    return;
                }

                if (std.mem.eql(u8, command, "request-close")) {
                    try host.requestCloseDialog(target_id, return_value);
                    return;
                }

                if (try host.domStore().hasAttribute(target_id, "open")) return;
                try host.showDialog(target_id);
                return;
            }

            return;
        }
    }

    if (try script.popoverTargetElement(host, node_id)) |popover_id| {
        const action = try script.popoverTargetActionValue(host, node_id);
        if (std.mem.eql(u8, action, "show")) {
            try host.showPopover(popover_id);
            return;
        }
        if (std.mem.eql(u8, action, "hide")) {
            try host.hidePopover(popover_id);
            return;
        }
        _ = try host.togglePopover(popover_id, null);
        return;
    }

    if (std.mem.eql(u8, element.tag_name, "a") or std.mem.eql(u8, element.tag_name, "area")) {
        if (std.mem.eql(u8, element.tag_name, "area") and elementAttributeValue(element, "nohref") != null) {
            return;
        }
        const href = elementAttributeValue(element, "href") orelse return;
        const trimmed_href = std.mem.trim(u8, href, " \t\r\n");
        if (trimmed_href.len == 0) return;

        if (elementAttributeValue(element, "download")) |download_attr| {
            const trimmed_download = std.mem.trim(u8, download_attr, " \t\r\n");
            const file_name = if (trimmed_download.len > 0)
                trimmed_download
            else
                try downloadFileNameFromHref(allocator, trimmed_href);
            try host.captureDownload(file_name, trimmed_href);
            return;
        }

        if (elementAttributeValue(element, "target")) |target_attr| {
            const trimmed_target = std.mem.trim(u8, target_attr, " \t\r\n");
            if (trimmed_target.len > 0 and !std.mem.eql(u8, trimmed_target, "_self")) {
                try host.open(trimmed_href, trimmed_target, null);
                return;
            }
        }

        try host.assignLocation(trimmed_href);
    }

    return;
}

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

fn appendMatchMediaListener(
    allocator: std.mem.Allocator,
    listeners: *std.ArrayListUnmanaged(script.MatchMediaListenerRecord),
    query_source: []const u8,
    handler: script.ScriptFunction,
    current_matches: bool,
) errors.Result(void) {
    try listeners.append(allocator, .{
        .query = try allocator.dupe(u8, query_source),
        .handler = try duplicateScriptFunction(allocator, handler),
        .last_matches = current_matches,
    });
    return;
}

fn findMatchMediaOnChange(
    listeners: *const std.ArrayListUnmanaged(script.MatchMediaListenerRecord),
    query_source: []const u8,
) ?script.ScriptFunction {
    for (listeners.items) |record| {
        if (std.mem.eql(u8, record.query, query_source)) {
            return record.handler;
        }
    }
    return null;
}

fn upsertMatchMediaOnChange(
    allocator: std.mem.Allocator,
    listeners: *std.ArrayListUnmanaged(script.MatchMediaListenerRecord),
    query_source: []const u8,
    handler: script.ScriptFunction,
    current_matches: bool,
) errors.Result(void) {
    for (listeners.items) |*record| {
        if (!std.mem.eql(u8, record.query, query_source)) continue;
        record.handler = try duplicateScriptFunction(allocator, handler);
        record.last_matches = current_matches;
        return;
    }

    try listeners.append(allocator, .{
        .query = try allocator.dupe(u8, query_source),
        .handler = try duplicateScriptFunction(allocator, handler),
        .last_matches = current_matches,
    });
    return;
}

fn removeMatchMediaOnChange(
    listeners: *std.ArrayListUnmanaged(script.MatchMediaListenerRecord),
    query_source: []const u8,
) void {
    var index: usize = 0;
    while (index < listeners.items.len) {
        const record = listeners.items[index];
        if (!std.mem.eql(u8, record.query, query_source)) {
            index += 1;
            continue;
        }
        _ = listeners.orderedRemove(index);
        break;
    }
}

fn removeMatchMediaListener(
    listeners: *std.ArrayListUnmanaged(script.MatchMediaListenerRecord),
    query_source: []const u8,
    handler: script.ScriptFunction,
) void {
    var index: usize = 0;
    while (index < listeners.items.len) {
        const record = listeners.items[index];
        if (!std.mem.eql(u8, record.query, query_source)) {
            index += 1;
            continue;
        }
        if (!scriptFunctionEquals(record.handler, handler)) {
            index += 1;
            continue;
        }
        _ = listeners.orderedRemove(index);
    }
}

fn appendQueuedMicrotask(
    allocator: std.mem.Allocator,
    queued_microtasks: *std.ArrayListUnmanaged(script.ScriptFunction),
    handler: script.ScriptFunction,
) errors.Result(void) {
    const handler_copy = try duplicateScriptFunction(allocator, handler);
    try queued_microtasks.append(allocator, handler_copy);
    return;
}

fn appendScheduledTimer(
    allocator: std.mem.Allocator,
    next_timer_id: *u64,
    timers: *std.ArrayListUnmanaged(ScheduledTimer),
    at_ms: i64,
    interval_ms: ?i64,
    kind: TimerKind,
    handler: script.ScriptFunction,
) errors.Result(u64) {
    const handler_copy = try duplicateScriptFunction(allocator, handler);
    const timer_id = next_timer_id.*;
    next_timer_id.* = std.math.add(u64, next_timer_id.*, 1) catch return error.TimerError;
    try timers.append(allocator, .{
        .id = timer_id,
        .at_ms = at_ms,
        .interval_ms = interval_ms,
        .kind = kind,
        .handler = handler_copy,
    });
    bubbleScheduledTimerUp(timers.items);
    return timer_id;
}

fn reinsertScheduledTimer(
    allocator: std.mem.Allocator,
    timers: *std.ArrayListUnmanaged(ScheduledTimer),
    timer: ScheduledTimer,
) errors.Result(void) {
    try timers.append(allocator, timer);
    bubbleScheduledTimerUp(timers.items);
    return;
}

fn clearScheduledTimer(
    timers: *std.ArrayListUnmanaged(ScheduledTimer),
    timer_id: u64,
) void {
    var index: usize = 0;
    while (index < timers.items.len) : (index += 1) {
        if (timers.items[index].id == timer_id) {
            _ = timers.orderedRemove(index);
            return;
        }
    }
}

fn bubbleScheduledTimerUp(timers: []ScheduledTimer) void {
    if (timers.len == 0) return;
    var index = timers.len - 1;
    while (index > 0 and scheduledTimerPrecedes(timers[index], timers[index - 1])) : (index -= 1) {
        std.mem.swap(ScheduledTimer, &timers[index], &timers[index - 1]);
    }
}

fn scheduledTimerPrecedes(lhs: ScheduledTimer, rhs: ScheduledTimer) bool {
    return lhs.at_ms < rhs.at_ms or (lhs.at_ms == rhs.at_ms and lhs.id < rhs.id);
}

fn nextAnimationFrameTime(current_ms: i64) errors.Result(i64) {
    const frame_ms: i64 = 16;
    const remainder = @mod(current_ms, frame_ms);
    const delta = if (remainder == 0) frame_ms else frame_ms - remainder;
    const next_ms = std.math.add(i64, current_ms, delta) catch return error.TimerError;
    return next_ms;
}

fn animationFrameBindings(
    allocator: std.mem.Allocator,
    function: script.ScriptFunction,
    timestamp_ms: i64,
) errors.Result(std.ArrayList(script.Binding)) {
    var bindings_out: std.ArrayList(script.Binding) = .empty;
    errdefer bindings_out.deinit(allocator);

    if (function.params.len > 0) {
        try bindings_out.append(allocator, .{
            .name = function.params[0],
            .value = .{ .number = @floatFromInt(timestamp_ms) },
        });
    }

    return bindings_out;
}

fn drainQueuedMicrotasks(
    allocator: std.mem.Allocator,
    runtime: *script.ScriptRuntime,
    host: anytype,
    queued_microtasks: *std.ArrayListUnmanaged(script.ScriptFunction),
    step_count: *usize,
) errors.Result(void) {
    while (queued_microtasks.items.len > 0) {
        step_count.* += 1;
        if (step_count.* > scheduler_step_limit) return error.TimerError;
        const handler = queued_microtasks.orderedRemove(0);
        var bindings = try script.functionBindings(allocator, handler, &.{});
        defer bindings.deinit(allocator);

        const source_name = try std.fmt.allocPrint(allocator, "microtask:{d}", .{queued_microtasks.items.len});
        defer allocator.free(source_name);

        try runtime.evalScriptSourceWithBindings(
            allocator,
            host,
            handler.body_source,
            source_name,
            bindings.items,
        );
    }
    return;
}

fn runDueTimers(
    self: *Session,
    allocator: std.mem.Allocator,
    step_count: *usize,
) errors.Result(void) {
    while (self.timers.items.len > 0) {
        var timer = self.timers.items[0];
        if (timer.at_ms > self.clock_ms) break;

        _ = self.timers.orderedRemove(0);
        step_count.* += 1;
        if (step_count.* > scheduler_step_limit) return error.TimerError;

        self.active_timer_id = timer.id;
        self.active_timer_cancelled = false;
        defer {
            self.active_timer_id = null;
            self.active_timer_cancelled = false;
        }

        var bindings = switch (timer.kind) {
            .animation_frame => try animationFrameBindings(allocator, timer.handler, timer.at_ms),
            else => try script.functionBindings(allocator, timer.handler, &.{}),
        };
        defer bindings.deinit(allocator);

        const source_name = try std.fmt.allocPrint(allocator, "timer:{d}", .{timer.id});
        defer allocator.free(source_name);

        try self.script_runtime.evalScriptSourceWithBindings(
            allocator,
            self,
            timer.handler.body_source,
            source_name,
            bindings.items,
        );

        try drainQueuedMicrotasks(
            allocator,
            &self.script_runtime,
            self,
            &self.queued_microtasks,
            step_count,
        );

        if (timer.interval_ms) |interval_ms| {
            if (!self.active_timer_cancelled) {
                timer.at_ms = std.math.add(i64, timer.at_ms, interval_ms) catch return error.TimerError;
                try reinsertScheduledTimer(allocator, &self.timers, timer);
            }
        }
    }
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

fn scriptFunctionEquals(lhs: script.ScriptFunction, rhs: script.ScriptFunction) bool {
    if (!std.mem.eql(u8, lhs.body_source, rhs.body_source)) return false;
    if (lhs.params.len != rhs.params.len) return false;
    for (lhs.params, rhs.params) |lhs_param, rhs_param| {
        if (!std.mem.eql(u8, lhs_param, rhs_param)) return false;
    }
    return true;
}

fn elementAttributeValue(element: dom.ElementData, name: []const u8) ?[]const u8 {
    for (element.attributes.items) |attribute| {
        if (std.mem.eql(u8, attribute.name, name)) return attribute.value;
    }
    return null;
}

fn downloadFileNameFromHref(allocator: std.mem.Allocator, href: []const u8) errors.Result([]const u8) {
    const without_fragment = href[0..(std.mem.indexOfAny(u8, href, "?#") orelse href.len)];
    const file_name = if (std.mem.lastIndexOfScalar(u8, without_fragment, '/')) |slash_index| blk: {
        break :blk without_fragment[slash_index + 1 ..];
    } else without_fragment;
    if (file_name.len == 0) {
        return allocator.dupe(u8, "download");
    }
    return allocator.dupe(u8, file_name);
}

fn isFormNode(node: *const dom.NodeRecord) bool {
    return switch (node.kind) {
        .element => |element| std.mem.eql(u8, element.tag_name, "form"),
        else => false,
    };
}

fn isDialogNode(node: *const dom.NodeRecord) bool {
    return switch (node.kind) {
        .element => |element| std.mem.eql(u8, element.tag_name, "dialog"),
        else => false,
    };
}

fn isDetailsNode(node: *const dom.NodeRecord) bool {
    return switch (node.kind) {
        .element => |element| std.mem.eql(u8, element.tag_name, "details"),
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

fn isResetControl(tag_name: []const u8, input_type: ?[]const u8) bool {
    if (std.mem.eql(u8, tag_name, "button")) {
        return std.mem.eql(u8, input_type orelse "submit", "reset");
    }

    if (std.mem.eql(u8, tag_name, "input")) {
        return std.mem.eql(u8, input_type orelse "text", "reset");
    }

    return false;
}

fn fragmentIdentifierFromUrl(url: []const u8) ?[]const u8 {
    const fragment_index = std.mem.indexOfScalar(u8, url, '#') orelse return null;
    if (fragment_index + 1 >= url.len) return null;
    return url[fragment_index + 1 ..];
}

fn urlWithoutFragmentText(url: []const u8) []const u8 {
    const fragment_index = std.mem.indexOfScalar(u8, url, '#') orelse return url;
    return url[0..fragment_index];
}

fn shouldDispatchPageLifecycle(before_url: []const u8, after_url: []const u8) bool {
    return !std.mem.eql(u8, urlWithoutFragmentText(before_url), urlWithoutFragmentText(after_url));
}

fn locationHashFromUrl(allocator: std.mem.Allocator, url: []const u8) errors.Result([]const u8) {
    const fragment = fragmentIdentifierFromUrl(url) orelse return allocator.dupe(u8, "");
    if (fragment.len == 0) return allocator.dupe(u8, "");
    return try std.fmt.allocPrint(allocator, "#{s}", .{fragment});
}

fn locationUrlWithHash(
    allocator: std.mem.Allocator,
    current_url: []const u8,
    hash_source: []const u8,
) errors.Result([]const u8) {
    const fragment = if (hash_source.len > 0 and hash_source[0] == '#')
        hash_source[1..]
    else
        hash_source;

    const base_end = std.mem.indexOfScalar(u8, current_url, '#') orelse current_url.len;
    const base = current_url[0..base_end];
    if (fragment.len == 0) {
        return allocator.dupe(u8, base);
    }

    return try std.fmt.allocPrint(allocator, "{s}#{s}", .{ base, fragment });
}

fn cookieJarText(
    allocator: std.mem.Allocator,
    cookie_jar: *const std.StringArrayHashMapUnmanaged([]const u8),
) errors.Result([]const u8) {
    const keys_slice = cookie_jar.keys();
    var keys = try allocator.alloc([]const u8, keys_slice.len);
    defer allocator.free(keys);
    @memcpy(keys, keys_slice);

    if (keys.len > 1) {
        var index: usize = 0;
        while (index + 1 < keys.len) : (index += 1) {
            var smallest = index;
            var scan = index + 1;
            while (scan < keys.len) : (scan += 1) {
                if (std.mem.order(u8, keys[scan], keys[smallest]) == .lt) {
                    smallest = scan;
                }
            }
            if (smallest != index) {
                const temp = keys[index];
                keys[index] = keys[smallest];
                keys[smallest] = temp;
            }
        }
    }

    var joined: std.ArrayList(u8) = .empty;
    errdefer joined.deinit(allocator);
    for (keys, 0..) |key, key_index| {
        if (key_index > 0) {
            try joined.appendSlice(allocator, "; ");
        }
        try joined.appendSlice(allocator, key);
        try joined.append(allocator, '=');
        try joined.appendSlice(allocator, cookie_jar.get(key).?);
    }

    return try joined.toOwnedSlice(allocator);
}

fn cookieJarSet(
    allocator: std.mem.Allocator,
    cookie_jar: *std.StringArrayHashMapUnmanaged([]const u8),
    value: []const u8,
) errors.Result(void) {
    const trimmed = std.mem.trim(u8, value, " \t\r\n");
    if (trimmed.len == 0) return error.ScriptRuntime;

    const pair_end = std.mem.indexOfScalar(u8, trimmed, ';') orelse trimmed.len;
    const pair = trimmed[0..pair_end];
    const equals_index = std.mem.indexOfScalar(u8, pair, '=') orelse return error.ScriptRuntime;

    const name = std.mem.trim(u8, pair[0..equals_index], " \t\r\n");
    if (name.len == 0) return error.ScriptRuntime;
    const cookie_value = std.mem.trimLeft(u8, pair[equals_index + 1 ..], " \t\r\n");

    if (cookie_jar.getPtr(name)) |value_ptr| {
        value_ptr.* = try allocator.dupe(u8, cookie_value);
        return;
    }

    const key_copy = try allocator.dupe(u8, name);
    errdefer allocator.free(key_copy);
    const value_copy = try allocator.dupe(u8, cookie_value);
    errdefer allocator.free(value_copy);
    try cookie_jar.put(allocator, key_copy, value_copy);
    return;
}

fn cryptoRandomUuidFromState(allocator: std.mem.Allocator, state: *u64) errors.Result([]const u8) {
    var bytes: [16]u8 = undefined;
    for (bytes[0..]) |*byte| {
        state.* = state.* *% 73 +% 41;
        byte.* = @truncate(state.*);
    }

    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    const hex = "0123456789abcdef";
    var out: [36]u8 = undefined;
    var out_index: usize = 0;
    for (bytes[0..], 0..) |byte, index| {
        if (index == 4 or index == 6 or index == 8 or index == 10) {
            out[out_index] = '-';
            out_index += 1;
        }
        out[out_index] = hex[byte >> 4];
        out_index += 1;
        out[out_index] = hex[byte & 0x0f];
        out_index += 1;
    }

    return try allocator.dupe(u8, out[0..]);
}

fn domainFromUrl(allocator: std.mem.Allocator, url: []const u8) errors.Result([]const u8) {
    const colon_index = std.mem.indexOfScalar(u8, url, ':') orelse return allocator.dupe(u8, "null");
    if (colon_index + 2 >= url.len) return allocator.dupe(u8, "null");

    const after_colon = url[colon_index + 1 ..];
    if (!std.mem.startsWith(u8, after_colon, "//")) {
        return allocator.dupe(u8, "null");
    }

    const remainder = url[colon_index + 3 ..];
    const authority_end = std.mem.indexOfAny(u8, remainder, "/?#") orelse remainder.len;
    var authority = remainder[0..authority_end];
    if (authority.len == 0) {
        return allocator.dupe(u8, "null");
    }

    if (std.mem.lastIndexOfScalar(u8, authority, '@')) |at_index| {
        authority = authority[at_index + 1 ..];
    }

    var host = authority;
    if (host.len == 0) {
        return allocator.dupe(u8, "null");
    }

    if (host[0] == '[') {
        const end_bracket = std.mem.indexOfScalar(u8, host, ']') orelse return allocator.dupe(u8, "null");
        host = host[1..end_bracket];
    } else if (std.mem.indexOfScalar(u8, host, ':')) |port_index| {
        host = host[0..port_index];
    }

    if (host.len == 0) {
        return allocator.dupe(u8, "null");
    }

    const out = try allocator.dupe(u8, host);
    for (out) |*byte| {
        byte.* = std.ascii.toLower(byte.*);
    }
    return out;
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
