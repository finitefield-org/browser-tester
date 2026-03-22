const std = @import("std");

const bt_errors = @import("errors.zig");

pub const FetchResponseRule = struct {
    url: []const u8,
    status: u16,
    body: []const u8,
};

pub const FetchErrorRule = struct {
    url: []const u8,
    message: []const u8,
};

pub const FetchCall = struct {
    url: []const u8,
};

pub const FetchResponse = struct {
    url: []const u8,
    status: u16,
    body: []const u8,
};

pub const FetchMocks = struct {
    allocator: std.mem.Allocator,
    response_rules: std.ArrayListUnmanaged(FetchResponseRule) = .{},
    error_rules: std.ArrayListUnmanaged(FetchErrorRule) = .{},
    call_log: std.ArrayListUnmanaged(FetchCall) = .{},

    pub fn init(allocator: std.mem.Allocator) FetchMocks {
        return .{
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *FetchMocks) void {
        self.response_rules.deinit(self.allocator);
        self.error_rules.deinit(self.allocator);
        self.call_log.deinit(self.allocator);
    }

    pub fn respondText(
        self: *FetchMocks,
        url: []const u8,
        status: u16,
        body: []const u8,
    ) bt_errors.Result(void) {
        try self.response_rules.append(self.allocator, .{
            .url = try self.allocator.dupe(u8, url),
            .status = status,
            .body = try self.allocator.dupe(u8, body),
        });
    }

    pub fn fail(self: *FetchMocks, url: []const u8, message: []const u8) bt_errors.Result(void) {
        try self.error_rules.append(self.allocator, .{
            .url = try self.allocator.dupe(u8, url),
            .message = try self.allocator.dupe(u8, message),
        });
    }

    pub fn recordCall(self: *FetchMocks, url: []const u8) bt_errors.Result(void) {
        try self.call_log.append(self.allocator, .{
            .url = try self.allocator.dupe(u8, url),
        });
    }

    pub fn responses(self: *const FetchMocks) []const FetchResponseRule {
        return self.response_rules.items;
    }

    pub fn errors(self: *const FetchMocks) []const FetchErrorRule {
        return self.error_rules.items;
    }

    pub fn calls(self: *const FetchMocks) []const FetchCall {
        return self.call_log.items;
    }

    pub fn findResponse(self: *const FetchMocks, url: []const u8) ?FetchResponseRule {
        var index: usize = self.response_rules.items.len;
        while (index > 0) {
            index -= 1;
            const response = self.response_rules.items[index];
            if (std.mem.eql(u8, response.url, url)) return response;
        }
        return null;
    }

    pub fn findError(self: *const FetchMocks, url: []const u8) ?FetchErrorRule {
        var index: usize = self.error_rules.items.len;
        while (index > 0) {
            index -= 1;
            const rule = self.error_rules.items[index];
            if (std.mem.eql(u8, rule.url, url)) return rule;
        }
        return null;
    }

    pub fn reset(self: *FetchMocks) void {
        self.response_rules = .{};
        self.error_rules = .{};
        self.call_log = .{};
    }
};

pub const DialogMocks = struct {
    allocator: std.mem.Allocator,
    confirm_queue: std.ArrayListUnmanaged(bool) = .{},
    prompt_queue: std.ArrayListUnmanaged(?[]const u8) = .{},
    alert_messages: std.ArrayListUnmanaged([]const u8) = .{},
    confirm_messages: std.ArrayListUnmanaged([]const u8) = .{},
    prompt_messages: std.ArrayListUnmanaged([]const u8) = .{},

    pub fn init(allocator: std.mem.Allocator) DialogMocks {
        return .{
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *DialogMocks) void {
        self.confirm_queue.deinit(self.allocator);
        self.prompt_queue.deinit(self.allocator);
        self.alert_messages.deinit(self.allocator);
        self.confirm_messages.deinit(self.allocator);
        self.prompt_messages.deinit(self.allocator);
    }

    pub fn pushConfirm(self: *DialogMocks, value: bool) bt_errors.Result(void) {
        try self.confirm_queue.append(self.allocator, value);
    }

    pub fn pushPrompt(self: *DialogMocks, value: ?[]const u8) bt_errors.Result(void) {
        try self.prompt_queue.append(
            self.allocator,
            if (value) |text|
                try self.allocator.dupe(u8, text)
            else
                null,
        );
    }

    pub fn recordAlert(self: *DialogMocks, message: []const u8) bt_errors.Result(void) {
        try self.alert_messages.append(self.allocator, try self.allocator.dupe(u8, message));
    }

    pub fn recordConfirm(self: *DialogMocks, message: []const u8) bt_errors.Result(void) {
        try self.confirm_messages.append(self.allocator, try self.allocator.dupe(u8, message));
    }

    pub fn recordPrompt(self: *DialogMocks, message: []const u8) bt_errors.Result(void) {
        try self.prompt_messages.append(self.allocator, try self.allocator.dupe(u8, message));
    }

    pub fn takeConfirm(self: *DialogMocks) ?bool {
        if (self.confirm_queue.items.len == 0) return null;
        return self.confirm_queue.orderedRemove(0);
    }

    pub fn takePrompt(self: *DialogMocks) ??[]const u8 {
        if (self.prompt_queue.items.len == 0) return null;
        return self.prompt_queue.orderedRemove(0);
    }

    pub fn confirmQueue(self: *const DialogMocks) []const bool {
        return self.confirm_queue.items;
    }

    pub fn promptQueue(self: *const DialogMocks) []const ?[]const u8 {
        return self.prompt_queue.items;
    }

    pub fn alertMessages(self: *const DialogMocks) []const []const u8 {
        return self.alert_messages.items;
    }

    pub fn confirmMessages(self: *const DialogMocks) []const []const u8 {
        return self.confirm_messages.items;
    }

    pub fn promptMessages(self: *const DialogMocks) []const []const u8 {
        return self.prompt_messages.items;
    }

    pub fn reset(self: *DialogMocks) void {
        self.confirm_queue = .{};
        self.prompt_queue = .{};
        self.alert_messages = .{};
        self.confirm_messages = .{};
        self.prompt_messages = .{};
    }
};

pub const ClipboardMocks = struct {
    allocator: std.mem.Allocator,
    seeded_text: ?[]const u8 = null,
    write_log: std.ArrayListUnmanaged([]const u8) = .{},

    pub fn init(allocator: std.mem.Allocator) ClipboardMocks {
        return .{
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *ClipboardMocks) void {
        self.write_log.deinit(self.allocator);
    }

    pub fn seedText(self: *ClipboardMocks, value: []const u8) bt_errors.Result(void) {
        self.seeded_text = try self.allocator.dupe(u8, value);
    }

    pub fn seededText(self: *const ClipboardMocks) ?[]const u8 {
        return self.seeded_text;
    }

    pub fn recordWrite(self: *ClipboardMocks, value: []const u8) bt_errors.Result(void) {
        const copied = try self.allocator.dupe(u8, value);
        self.seeded_text = copied;
        try self.write_log.append(self.allocator, copied);
    }

    pub fn writes(self: *const ClipboardMocks) []const []const u8 {
        return self.write_log.items;
    }

    pub fn reset(self: *ClipboardMocks) void {
        self.seeded_text = null;
        self.write_log = .{};
    }
};

pub const OpenCall = struct {
    url: ?[]const u8,
    target: ?[]const u8,
    features: ?[]const u8,
};

pub const OpenMocks = struct {
    allocator: std.mem.Allocator,
    failure_message: ?[]const u8 = null,
    call_log: std.ArrayListUnmanaged(OpenCall) = .{},

    pub fn init(allocator: std.mem.Allocator) OpenMocks {
        return .{
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *OpenMocks) void {
        self.call_log.deinit(self.allocator);
    }

    pub fn fail(self: *OpenMocks, message: []const u8) bt_errors.Result(void) {
        self.failure_message = try self.allocator.dupe(u8, message);
    }

    pub fn recordCall(
        self: *OpenMocks,
        url: ?[]const u8,
        target: ?[]const u8,
        features: ?[]const u8,
    ) bt_errors.Result(void) {
        try self.call_log.append(self.allocator, .{
            .url = if (url) |value| try self.allocator.dupe(u8, value) else null,
            .target = if (target) |value| try self.allocator.dupe(u8, value) else null,
            .features = if (features) |value| try self.allocator.dupe(u8, value) else null,
        });

        if (self.failure_message != null) return error.MockError;
        return;
    }

    pub fn calls(self: *const OpenMocks) []const OpenCall {
        return self.call_log.items;
    }

    pub fn reset(self: *OpenMocks) void {
        self.failure_message = null;
        self.call_log = .{};
    }
};

pub const CloseCall = struct {
    _reserved: u8 = 0,
};

pub const CloseMocks = struct {
    allocator: std.mem.Allocator,
    failure_message: ?[]const u8 = null,
    call_log: std.ArrayListUnmanaged(CloseCall) = .{},

    pub fn init(allocator: std.mem.Allocator) CloseMocks {
        return .{
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *CloseMocks) void {
        self.call_log.deinit(self.allocator);
    }

    pub fn fail(self: *CloseMocks, message: []const u8) bt_errors.Result(void) {
        self.failure_message = try self.allocator.dupe(u8, message);
    }

    pub fn recordCall(self: *CloseMocks) bt_errors.Result(void) {
        try self.call_log.append(self.allocator, .{});
        if (self.failure_message != null) return error.MockError;
        return;
    }

    pub fn calls(self: *const CloseMocks) []const CloseCall {
        return self.call_log.items;
    }

    pub fn reset(self: *CloseMocks) void {
        self.failure_message = null;
        self.call_log = .{};
    }
};

pub const ScrollMethod = enum {
    To,
    By,
};

pub const ScrollCall = struct {
    method: ScrollMethod,
    x: i64,
    y: i64,
};

pub const ScrollMocks = struct {
    allocator: std.mem.Allocator,
    failure_message: ?[]const u8 = null,
    call_log: std.ArrayListUnmanaged(ScrollCall) = .{},

    pub fn init(allocator: std.mem.Allocator) ScrollMocks {
        return .{
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *ScrollMocks) void {
        self.call_log.deinit(self.allocator);
    }

    pub fn fail(self: *ScrollMocks, message: []const u8) bt_errors.Result(void) {
        self.failure_message = try self.allocator.dupe(u8, message);
    }

    pub fn recordCall(self: *ScrollMocks, method: ScrollMethod, x: i64, y: i64) bt_errors.Result(void) {
        try self.call_log.append(self.allocator, .{
            .method = method,
            .x = x,
            .y = y,
        });
        if (self.failure_message != null) return error.MockError;
        return;
    }

    pub fn calls(self: *const ScrollMocks) []const ScrollCall {
        return self.call_log.items;
    }

    pub fn reset(self: *ScrollMocks) void {
        self.failure_message = null;
        self.call_log = .{};
    }
};

pub const PrintCall = struct {
    _reserved: u8 = 0,
};

pub const PrintMocks = struct {
    allocator: std.mem.Allocator,
    failure_message: ?[]const u8 = null,
    call_log: std.ArrayListUnmanaged(PrintCall) = .{},

    pub fn init(allocator: std.mem.Allocator) PrintMocks {
        return .{
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *PrintMocks) void {
        self.call_log.deinit(self.allocator);
    }

    pub fn fail(self: *PrintMocks, message: []const u8) bt_errors.Result(void) {
        self.failure_message = try self.allocator.dupe(u8, message);
    }

    pub fn recordCall(self: *PrintMocks) bt_errors.Result(void) {
        try self.call_log.append(self.allocator, .{});
        if (self.failure_message != null) return error.MockError;
        return;
    }

    pub fn calls(self: *const PrintMocks) []const PrintCall {
        return self.call_log.items;
    }

    pub fn reset(self: *PrintMocks) void {
        self.failure_message = null;
        self.call_log = .{};
    }
};

pub const LocationMocks = struct {
    allocator: std.mem.Allocator,
    current_url: ?[]const u8 = null,
    navigation_log: std.ArrayListUnmanaged([]const u8) = .{},

    pub fn init(allocator: std.mem.Allocator) LocationMocks {
        return .{
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *LocationMocks) void {
        self.navigation_log.deinit(self.allocator);
    }

    pub fn setCurrent(self: *LocationMocks, url: []const u8) bt_errors.Result(void) {
        self.current_url = try self.allocator.dupe(u8, url);
    }

    pub fn recordNavigation(self: *LocationMocks, url: []const u8) bt_errors.Result(void) {
        try self.navigation_log.append(self.allocator, try self.allocator.dupe(u8, url));
    }

    pub fn currentUrl(self: *const LocationMocks) ?[]const u8 {
        return self.current_url;
    }

    pub fn navigations(self: *const LocationMocks) []const []const u8 {
        return self.navigation_log.items;
    }

    pub fn reset(self: *LocationMocks) void {
        self.current_url = null;
        self.navigation_log = .{};
    }
};

pub const MatchMediaRule = struct {
    query: []const u8,
    matches: bool = false,
    is_failure: bool = false,
};

pub const MatchMediaCall = struct {
    query: []const u8,
};

pub const MatchMediaMocks = struct {
    allocator: std.mem.Allocator,
    rule_log: std.ArrayListUnmanaged(MatchMediaRule) = .{},
    call_log: std.ArrayListUnmanaged(MatchMediaCall) = .{},

    pub fn init(allocator: std.mem.Allocator) MatchMediaMocks {
        return .{
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *MatchMediaMocks) void {
        self.rule_log.deinit(self.allocator);
        self.call_log.deinit(self.allocator);
    }

    pub fn seedMatch(
        self: *MatchMediaMocks,
        query: []const u8,
        matches: bool,
    ) bt_errors.Result(void) {
        try self.rule_log.append(self.allocator, .{
            .query = try self.allocator.dupe(u8, query),
            .matches = matches,
        });
    }

    pub fn fail(self: *MatchMediaMocks, query: []const u8) bt_errors.Result(void) {
        try self.rule_log.append(self.allocator, .{
            .query = try self.allocator.dupe(u8, query),
            .is_failure = true,
        });
    }

    pub fn recordCall(self: *MatchMediaMocks, query: []const u8) bt_errors.Result(void) {
        try self.call_log.append(self.allocator, .{
            .query = try self.allocator.dupe(u8, query),
        });
    }

    pub fn calls(self: *const MatchMediaMocks) []const MatchMediaCall {
        return self.call_log.items;
    }

    pub fn findRule(self: *const MatchMediaMocks, query: []const u8) ?MatchMediaRule {
        var index: usize = self.rule_log.items.len;
        while (index > 0) {
            index -= 1;
            const rule = self.rule_log.items[index];
            if (std.mem.eql(u8, rule.query, query)) return rule;
        }
        return null;
    }

    pub fn currentMatch(self: *const MatchMediaMocks, query: []const u8) ?bool {
        if (self.findRule(query)) |rule| {
            if (rule.is_failure) return null;
            return rule.matches;
        }
        return null;
    }

    pub fn reset(self: *MatchMediaMocks) void {
        self.rule_log = .{};
        self.call_log = .{};
    }
};

pub const DownloadCapture = struct {
    file_name: []const u8,
    bytes: []const u8,
};

pub const DownloadMocks = struct {
    allocator: std.mem.Allocator,
    artifact_log: std.ArrayListUnmanaged(DownloadCapture) = .{},

    pub fn init(allocator: std.mem.Allocator) DownloadMocks {
        return .{
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *DownloadMocks) void {
        self.artifact_log.deinit(self.allocator);
    }

    pub fn capture(self: *DownloadMocks, file_name: []const u8, bytes: []const u8) bt_errors.Result(void) {
        try self.artifact_log.append(self.allocator, .{
            .file_name = try self.allocator.dupe(u8, file_name),
            .bytes = try self.allocator.dupe(u8, bytes),
        });
    }

    pub fn artifacts(self: *const DownloadMocks) []const DownloadCapture {
        return self.artifact_log.items;
    }

    pub fn reset(self: *DownloadMocks) void {
        self.artifact_log = .{};
    }
};

pub const FileInputSelection = struct {
    selector: []const u8,
    files: []const []const u8,
};

pub const FileInputMocks = struct {
    allocator: std.mem.Allocator,
    selection_log: std.ArrayListUnmanaged(FileInputSelection) = .{},

    pub fn init(allocator: std.mem.Allocator) FileInputMocks {
        return .{
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *FileInputMocks) void {
        self.selection_log.deinit(self.allocator);
    }

    pub fn setFiles(
        self: *FileInputMocks,
        selector: []const u8,
        files: []const []const u8,
    ) bt_errors.Result(void) {
        const selector_copy = try self.allocator.dupe(u8, selector);
        const files_copy = try self.allocator.alloc([]const u8, files.len);
        for (files, 0..) |file, index| {
            files_copy[index] = try self.allocator.dupe(u8, file);
        }
        try self.selection_log.append(self.allocator, .{
            .selector = selector_copy,
            .files = files_copy,
        });
    }

    pub fn selections(self: *const FileInputMocks) []const FileInputSelection {
        return self.selection_log.items;
    }

    pub fn reset(self: *FileInputMocks) void {
        self.selection_log = .{};
    }
};

pub const StorageSeeds = struct {
    allocator: std.mem.Allocator,
    local_map: std.StringArrayHashMapUnmanaged([]const u8) = .empty,
    session_map: std.StringArrayHashMapUnmanaged([]const u8) = .empty,

    pub fn init(allocator: std.mem.Allocator) StorageSeeds {
        return .{
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *StorageSeeds) void {
        self.local_map.deinit(self.allocator);
        self.session_map.deinit(self.allocator);
    }

    pub fn seedLocal(self: *StorageSeeds, key: []const u8, value: []const u8) bt_errors.Result(void) {
        try self.local_map.put(
            self.allocator,
            try self.allocator.dupe(u8, key),
            try self.allocator.dupe(u8, value),
        );
    }

    pub fn seedSession(self: *StorageSeeds, key: []const u8, value: []const u8) bt_errors.Result(void) {
        try self.session_map.put(
            self.allocator,
            try self.allocator.dupe(u8, key),
            try self.allocator.dupe(u8, value),
        );
    }

    pub fn local(self: *StorageSeeds) *std.StringArrayHashMapUnmanaged([]const u8) {
        return &self.local_map;
    }

    pub fn session(self: *StorageSeeds) *std.StringArrayHashMapUnmanaged([]const u8) {
        return &self.session_map;
    }

    pub fn reset(self: *StorageSeeds) void {
        self.local_map.clearRetainingCapacity();
        self.session_map.clearRetainingCapacity();
    }
};

pub const MockRegistry = struct {
    fetch_mocks: FetchMocks,
    dialogs_mocks: DialogMocks,
    clipboard_mocks: ClipboardMocks,
    open_mocks: OpenMocks,
    close_mocks: CloseMocks,
    print_mocks: PrintMocks,
    scroll_mocks: ScrollMocks,
    location_mocks: LocationMocks,
    match_media_mocks: MatchMediaMocks,
    downloads_mocks: DownloadMocks,
    file_input_mocks: FileInputMocks,
    storage_seeds: StorageSeeds,

    pub fn init(allocator: std.mem.Allocator) MockRegistry {
        return .{
            .fetch_mocks = FetchMocks.init(allocator),
            .dialogs_mocks = DialogMocks.init(allocator),
            .clipboard_mocks = ClipboardMocks.init(allocator),
            .open_mocks = OpenMocks.init(allocator),
            .close_mocks = CloseMocks.init(allocator),
            .print_mocks = PrintMocks.init(allocator),
            .scroll_mocks = ScrollMocks.init(allocator),
            .location_mocks = LocationMocks.init(allocator),
            .match_media_mocks = MatchMediaMocks.init(allocator),
            .downloads_mocks = DownloadMocks.init(allocator),
            .file_input_mocks = FileInputMocks.init(allocator),
            .storage_seeds = StorageSeeds.init(allocator),
        };
    }

    pub fn deinit(self: *MockRegistry) void {
        self.fetch_mocks.deinit();
        self.dialogs_mocks.deinit();
        self.clipboard_mocks.deinit();
        self.open_mocks.deinit();
        self.close_mocks.deinit();
        self.print_mocks.deinit();
        self.scroll_mocks.deinit();
        self.location_mocks.deinit();
        self.match_media_mocks.deinit();
        self.downloads_mocks.deinit();
        self.file_input_mocks.deinit();
        self.storage_seeds.deinit();
    }

    pub fn fetch(self: *MockRegistry) *FetchMocks {
        return &self.fetch_mocks;
    }

    pub fn dialogs(self: *MockRegistry) *DialogMocks {
        return &self.dialogs_mocks;
    }

    pub fn clipboard(self: *MockRegistry) *ClipboardMocks {
        return &self.clipboard_mocks;
    }

    pub fn open(self: *MockRegistry) *OpenMocks {
        return &self.open_mocks;
    }

    pub fn close(self: *MockRegistry) *CloseMocks {
        return &self.close_mocks;
    }

    pub fn print(self: *MockRegistry) *PrintMocks {
        return &self.print_mocks;
    }

    pub fn scroll(self: *MockRegistry) *ScrollMocks {
        return &self.scroll_mocks;
    }

    pub fn location(self: *MockRegistry) *LocationMocks {
        return &self.location_mocks;
    }

    pub fn matchMedia(self: *MockRegistry) *MatchMediaMocks {
        return &self.match_media_mocks;
    }

    pub fn downloads(self: *MockRegistry) *DownloadMocks {
        return &self.downloads_mocks;
    }

    pub fn fileInput(self: *MockRegistry) *FileInputMocks {
        return &self.file_input_mocks;
    }

    pub fn storage(self: *MockRegistry) *StorageSeeds {
        return &self.storage_seeds;
    }

    pub fn resetAll(self: *MockRegistry) void {
        self.fetch_mocks.reset();
        self.dialogs_mocks.reset();
        self.clipboard_mocks.reset();
        self.open_mocks.reset();
        self.close_mocks.reset();
        self.print_mocks.reset();
        self.scroll_mocks.reset();
        self.location_mocks.reset();
        self.match_media_mocks.reset();
        self.downloads_mocks.reset();
        self.file_input_mocks.reset();
        self.storage_seeds.reset();
    }
};
