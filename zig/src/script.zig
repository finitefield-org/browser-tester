const std = @import("std");

const dom = @import("dom.zig");
const errors = @import("errors.zig");

pub const ListenerTarget = union(enum) {
    document,
    window,
    element: dom.NodeId,
};

pub const StorageTarget = enum {
    local,
    session,
};

pub const EventPhase = enum(u8) {
    none = 0,
    capturing = 1,
    at_target = 2,
    bubbling = 3,
};

pub const ScriptEvent = struct {
    event_type: []const u8,
    target: ListenerTarget,
    current_target: ?ListenerTarget = null,
    bubbles: bool,
    cancelable: bool,
    default_prevented: bool = false,
    propagation_stopped: bool = false,
    immediate_propagation_stopped: bool = false,
    phase: EventPhase = .none,

    pub fn eventType(self: *const ScriptEvent) []const u8 {
        return self.event_type;
    }

    pub fn preventDefault(self: *ScriptEvent) void {
        if (self.cancelable) {
            self.default_prevented = true;
        }
    }

    pub fn stopPropagation(self: *ScriptEvent) void {
        self.propagation_stopped = true;
    }

    pub fn stopImmediatePropagation(self: *ScriptEvent) void {
        self.propagation_stopped = true;
        self.immediate_propagation_stopped = true;
    }
};

pub const ScriptFunction = struct {
    params: []const []const u8 = &.{},
    body_source: []const u8 = "",
};

pub const ScriptListenerRecord = struct {
    target: ListenerTarget,
    event_type: []const u8,
    capture: bool,
    handler: ScriptFunction,
};

pub const MatchMediaListenerRecord = struct {
    query: []const u8,
    handler: ScriptFunction,
    last_matches: bool,
};

pub const Binding = struct {
    name: []const u8,
    value: Value,
};

const NodeListItemKind = enum {
    element,
    node,
};

const NodeListLiveKind = enum {
    children,
    labels,
    named_elements,
};

const HtmlCollectionKind = enum {
    children,
    forms,
    form_elements,
    document_images,
    document_links,
    document_embeds,
    document_plugins,
    navigator_plugins,
    document_applets,
    document_all,
    select_options,
    selected_options,
    fieldset_elements,
    datalist_options,
    map_areas,
    table_t_bodies,
    table_rows,
    row_cells,
    tag_name,
    tag_name_ns,
    class_name,
};

const NodeList = struct {
    items: []dom.NodeId,
    item_kind: NodeListItemKind = .element,
    live_root: ?dom.NodeId = null,
    live_kind: ?NodeListLiveKind = null,
    live_name: ?[]const u8 = null,
};

const HtmlCollection = struct {
    root: dom.NodeId,
    kind: HtmlCollectionKind = .children,
    query: ?[]const u8 = null,
    namespace_uri: ?[]const u8 = null,
    local_name: ?[]const u8 = null,
};

const RadioNodeList = struct {
    root: dom.NodeId,
    name: []const u8,
};

pub const ScriptRuntime = struct {
    pub fn init() ScriptRuntime {
        return .{};
    }

    pub fn deinit(self: *ScriptRuntime) void {
        _ = self;
    }

    pub fn bootstrapInlineScripts(
        self: *ScriptRuntime,
        allocator: std.mem.Allocator,
        host: anytype,
    ) errors.Result(void) {
        var script_ids: std.ArrayList(dom.NodeId) = .empty;
        defer script_ids.deinit(allocator);

        const store = host.domStore();
        for (store.records()) |node| {
            const element = switch (node.kind) {
                .element => |element| element,
                else => continue,
            };

            if (std.mem.eql(u8, element.tag_name, "script")) {
                try script_ids.append(allocator, node.id);
            }
        }

        for (script_ids.items) |script_id| {
            const before_url = host.currentLocationUrl();
            {
                host.setCurrentScript(script_id);
                defer host.setCurrentScript(null);
                const source = try store.textContent(allocator, script_id);
                defer allocator.free(source);
                try self.evalScriptSourceWithBindings(allocator, host, source, "inline-script", &.{});
            }
            const after_url = host.currentLocationUrl();
            try self.dispatchHashChangeIfNeeded(allocator, host, before_url, after_url);
        }

        return;
    }

    pub fn dispatchHashChangeIfNeeded(
        self: *ScriptRuntime,
        allocator: std.mem.Allocator,
        host: anytype,
        before_url: []const u8,
        after_url: []const u8,
    ) errors.Result(void) {
        if (std.mem.eql(u8, urlFragmentText(before_url), urlFragmentText(after_url))) {
            return;
        }

        try self.dispatchWindowEvent(
            allocator,
            host,
            "hashchange",
            host.scriptEventListeners(),
            host.windowHashChange(),
            "onhashchange",
        );
        return;
    }

    pub fn dispatchWindowEvent(
        self: *ScriptRuntime,
        allocator: std.mem.Allocator,
        host: anytype,
        event_type: []const u8,
        listeners: []const ScriptListenerRecord,
        on_handler: ?ScriptFunction,
        handler_name: []const u8,
    ) errors.Result(void) {
        var event = ScriptEvent{
            .event_type = event_type,
            .target = .window,
            .current_target = null,
            .bubbles = false,
            .cancelable = false,
        };

        var matched_listeners: std.ArrayList(ScriptListenerRecord) = .empty;
        defer matched_listeners.deinit(allocator);

        for (listeners) |listener| {
            switch (listener.target) {
                .window => {},
                else => continue,
            }
            if (!std.mem.eql(u8, listener.event_type, event_type)) continue;
            try matched_listeners.append(allocator, listener);
        }

        for ([_]bool{ true, false }) |capture| {
            for (matched_listeners.items, 0..) |listener, index| {
                if (listener.capture != capture) continue;
                if (event.immediate_propagation_stopped) break;

                event.current_target = .window;
                event.phase = .at_target;

                var bindings = try eventListenerBindings(allocator, listener.handler, &event);
                defer bindings.deinit(allocator);

                const source_name = try std.fmt.allocPrint(allocator, "event:{s}:{d}", .{ event_type, index });
                defer allocator.free(source_name);

                try self.evalScriptSourceWithBindings(
                    allocator,
                    host,
                    listener.handler.body_source,
                    source_name,
                    bindings.items,
                );
            }
        }

        if (!event.immediate_propagation_stopped and !event.propagation_stopped) {
            if (on_handler) |handler| {
                event.current_target = .window;
                event.phase = .at_target;

                var bindings = try eventListenerBindings(allocator, handler, &event);
                defer bindings.deinit(allocator);

                const source_name = try std.fmt.allocPrint(allocator, "event:{s}:{s}", .{ event_type, handler_name });
                defer allocator.free(source_name);

                try self.evalScriptSourceWithBindings(
                    allocator,
                    host,
                    handler.body_source,
                    source_name,
                    bindings.items,
                );
            }
        }

        event.current_target = null;
        event.phase = .none;
        return;
    }

    pub fn evalScriptSourceWithBindings(
        self: *ScriptRuntime,
        allocator: std.mem.Allocator,
        host: anytype,
        source: []const u8,
        source_name: []const u8,
        bindings: []const Binding,
    ) errors.Result(void) {
        _ = self;
        return try executeScriptSource(allocator, host, source, source_name, bindings);
    }
};

const Program = struct {
    statements: []Statement,
};

const Statement = union(enum) {
    expression: *Expr,
    assignment: Assignment,
    const_declaration: ConstDeclaration,
};

const Assignment = struct {
    target: PropertyTarget,
    value: *Expr,
};

const ConstDeclaration = struct {
    name: []const u8,
    value: *Expr,
};

const PropertyTarget = struct {
    object: *Expr,
    property: []const u8,
};

const Expr = union(enum) {
    identifier: []const u8,
    string: []const u8,
    number: []const u8,
    boolean: bool,
    null_value,
    undefined_value,
    member: MemberExpr,
    call: CallExpr,
    binary_add: BinaryAddExpr,
    arrow_function: ScriptFunction,
};

const MemberExpr = struct {
    object: *Expr,
    property: []const u8,
};

const CallExpr = struct {
    callee: *Expr,
    args: []*Expr,
};

const BinaryAddExpr = struct {
    left: *Expr,
    right: *Expr,
};

const CollectionIteratorState = struct {
    items: []Value,
    index: usize = 0,
};

const IteratorResult = struct {
    value: Value,
    done: bool,
};

const CollectionEntry = struct {
    index: usize,
    value: Value,
};

const MediaQueryList = struct {
    host: ?*anyopaque = null,
    media: []const u8,
    matches: bool,
    current_matches_fn: ?*const fn (*anyopaque, []const u8) bool = null,
    get_onchange_fn: ?*const fn (*anyopaque, []const u8) ?ScriptFunction = null,
    set_onchange_fn: ?*const fn (*anyopaque, []const u8, ?ScriptFunction) errors.Result(void) = null,
    add_listener_fn: ?*const fn (*anyopaque, []const u8, ScriptFunction) errors.Result(void) = null,
    remove_listener_fn: ?*const fn (*anyopaque, []const u8, ScriptFunction) errors.Result(void) = null,

    fn currentMatches(self: *const MediaQueryList) bool {
        if (self.current_matches_fn) |current_matches_fn| {
            const host = self.host orelse return self.matches;
            return current_matches_fn(host, self.media);
        }
        return self.matches;
    }
};

const MediaListState = struct {
    host: ?*anyopaque = null,
    node_id: ?dom.NodeId = null,
    media_text: []const u8,
    current_media_text_fn: ?*const fn (*anyopaque, dom.NodeId) errors.Result([]const u8) = null,
    set_media_text_fn: ?*const fn (*anyopaque, dom.NodeId, []const u8) errors.Result(void) = null,

    fn currentText(self: *const MediaListState) errors.Result([]const u8) {
        if (self.current_media_text_fn) |current_media_text_fn| {
            const host = self.host orelse return error.ScriptRuntime;
            const node_id = self.node_id orelse return error.ScriptRuntime;
            return try current_media_text_fn(host, node_id);
        }
        return self.media_text;
    }

    fn setText(self: *MediaListState, value: []const u8) errors.Result(void) {
        if (self.set_media_text_fn) |set_media_text_fn| {
            const host = self.host orelse return error.ScriptRuntime;
            const node_id = self.node_id orelse return error.ScriptRuntime;
            try set_media_text_fn(host, node_id, value);
            if (self.current_media_text_fn) |current_media_text_fn| {
                self.media_text = try current_media_text_fn(host, node_id);
            } else {
                self.media_text = value;
            }
            return;
        }
        return error.ScriptRuntime;
    }
};

const StringListState = struct {
    items: []const []const u8,

    fn length(self: *const StringListState) usize {
        return self.items.len;
    }

    fn item(self: *const StringListState, index: usize) ?[]const u8 {
        if (index >= self.items.len) return null;
        return self.items[index];
    }

    fn contains(self: *const StringListState, value: []const u8) bool {
        for (self.items) |candidate| {
            if (std.mem.eql(u8, candidate, value)) {
                return true;
            }
        }
        return false;
    }
};

fn mediaListLength(media_text: []const u8) usize {
    var count: usize = 0;
    var iter = std.mem.splitScalar(u8, media_text, ',');
    while (iter.next()) |part| {
        const trimmed = std.mem.trim(u8, part, " \t\r\n\x0c");
        if (trimmed.len == 0) continue;
        count += 1;
    }
    return count;
}

fn mediaListItems(
    allocator: std.mem.Allocator,
    media_text: []const u8,
) errors.Result(std.ArrayList([]const u8)) {
    var items: std.ArrayList([]const u8) = .empty;
    errdefer items.deinit(allocator);

    var iter = std.mem.splitScalar(u8, media_text, ',');
    while (iter.next()) |part| {
        const trimmed = std.mem.trim(u8, part, " \t\r\n\x0c");
        if (trimmed.len == 0) continue;
        try items.append(allocator, trimmed);
    }
    return items;
}

fn mediaListSerialize(
    allocator: std.mem.Allocator,
    items: []const []const u8,
) errors.Result([]const u8) {
    var out: std.ArrayList(u8) = .empty;
    errdefer out.deinit(allocator);

    for (items, 0..) |item, index| {
        if (index > 0) {
            try out.appendSlice(allocator, ", ");
        }
        try out.appendSlice(allocator, item);
    }

    const result = try allocator.dupe(u8, out.items);
    out.deinit(allocator);
    return result;
}

fn mediaListNormalizeItem(value: []const u8) errors.Result([]const u8) {
    const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
    if (trimmed.len == 0) return error.ScriptRuntime;
    if (std.mem.indexOfScalar(u8, trimmed, ',') != null) return error.ScriptRuntime;
    return trimmed;
}

fn mediaListContains(items: []const []const u8, value: []const u8) bool {
    for (items) |candidate| {
        if (std.mem.eql(u8, candidate, value)) return true;
    }
    return false;
}

fn mediaListIndexOf(items: []const []const u8, value: []const u8) ?usize {
    for (items, 0..) |candidate, index| {
        if (std.mem.eql(u8, candidate, value)) return index;
    }
    return null;
}

fn mediaListItem(
    allocator: std.mem.Allocator,
    media_text: []const u8,
    index: usize,
) errors.Result(?[]const u8) {
    var count: usize = 0;
    var iter = std.mem.splitScalar(u8, media_text, ',');
    while (iter.next()) |part| {
        const trimmed = std.mem.trim(u8, part, " \t\r\n\x0c");
        if (trimmed.len == 0) continue;
        if (count == index) {
            return try allocator.dupe(u8, trimmed);
        }
        count += 1;
    }
    return null;
}

const MathState = struct {
    host: *anyopaque,
    random_fn: *const fn (*anyopaque) f64,

    fn random(self: *const MathState) f64 {
        return self.random_fn(self.host);
    }
};

const LocationState = struct {
    host: *anyopaque,
    current_url: []const u8,
    current_url_fn: *const fn (*anyopaque) []const u8,
    hash_fn: *const fn (*anyopaque, std.mem.Allocator) errors.Result([]const u8),
    assign_fn: *const fn (*anyopaque, []const u8) errors.Result(void),
    replace_fn: *const fn (*anyopaque, []const u8) errors.Result(void),
    reload_fn: *const fn (*anyopaque) errors.Result(void),
    set_hash_fn: *const fn (*anyopaque, []const u8) errors.Result(void),
    ancestor_origins: StringListState = .{ .items = &.{} },

    fn refresh(self: *LocationState) void {
        self.current_url = self.current_url_fn(self.host);
    }

    fn hash(self: *const LocationState, allocator: std.mem.Allocator) errors.Result([]const u8) {
        return try self.hash_fn(self.host, allocator);
    }

    fn assign(self: *LocationState, url: []const u8) errors.Result(void) {
        try self.assign_fn(self.host, url);
        self.refresh();
        return;
    }

    fn replace(self: *LocationState, url: []const u8) errors.Result(void) {
        try self.replace_fn(self.host, url);
        self.refresh();
        return;
    }

    fn reload(self: *LocationState) errors.Result(void) {
        try self.reload_fn(self.host);
        self.refresh();
        return;
    }

    fn setHash(self: *LocationState, value: []const u8) errors.Result(void) {
        try self.set_hash_fn(self.host, value);
        self.refresh();
        return;
    }

    fn protocol(self: *const LocationState, allocator: std.mem.Allocator) errors.Result([]const u8) {
        return try locationProtocolFromUrl(allocator, self.current_url);
    }

    fn hostValue(self: *const LocationState, allocator: std.mem.Allocator) errors.Result([]const u8) {
        return try locationHostFromUrl(allocator, self.current_url);
    }

    fn hostname(self: *const LocationState, allocator: std.mem.Allocator) errors.Result([]const u8) {
        return try locationHostnameFromUrl(allocator, self.current_url);
    }

    fn port(self: *const LocationState, allocator: std.mem.Allocator) errors.Result([]const u8) {
        _ = allocator;
        return try locationPortFromUrl(self.current_url);
    }

    fn username(self: *const LocationState, allocator: std.mem.Allocator) errors.Result([]const u8) {
        _ = allocator;
        return try locationUsernameFromUrl(self.current_url);
    }

    fn password(self: *const LocationState, allocator: std.mem.Allocator) errors.Result([]const u8) {
        _ = allocator;
        return try locationPasswordFromUrl(self.current_url);
    }

    fn pathname(self: *const LocationState, allocator: std.mem.Allocator) errors.Result([]const u8) {
        _ = allocator;
        return try locationPathnameFromUrl(self.current_url);
    }

    fn search(self: *const LocationState, allocator: std.mem.Allocator) errors.Result([]const u8) {
        return try locationSearchFromUrl(allocator, self.current_url);
    }

    fn setProtocol(self: *LocationState, allocator: std.mem.Allocator, value: []const u8) errors.Result(void) {
        const url = try locationUrlWithProtocol(allocator, self.current_url, value);
        defer allocator.free(url);
        try self.assign(url);
        return;
    }

    fn setHost(self: *LocationState, allocator: std.mem.Allocator, value: []const u8) errors.Result(void) {
        const url = try locationUrlWithHost(allocator, self.current_url, value);
        defer allocator.free(url);
        try self.assign(url);
        return;
    }

    fn setHostname(self: *LocationState, allocator: std.mem.Allocator, value: []const u8) errors.Result(void) {
        const url = try locationUrlWithHostname(allocator, self.current_url, value);
        defer allocator.free(url);
        try self.assign(url);
        return;
    }

    fn setPort(self: *LocationState, allocator: std.mem.Allocator, value: []const u8) errors.Result(void) {
        const url = try locationUrlWithPort(allocator, self.current_url, value);
        defer allocator.free(url);
        try self.assign(url);
        return;
    }

    fn setUsername(self: *LocationState, allocator: std.mem.Allocator, value: []const u8) errors.Result(void) {
        const url = try locationUrlWithUsername(allocator, self.current_url, value);
        defer allocator.free(url);
        try self.assign(url);
        return;
    }

    fn setPassword(self: *LocationState, allocator: std.mem.Allocator, value: []const u8) errors.Result(void) {
        const url = try locationUrlWithPassword(allocator, self.current_url, value);
        defer allocator.free(url);
        try self.assign(url);
        return;
    }

    fn setPathname(self: *LocationState, allocator: std.mem.Allocator, value: []const u8) errors.Result(void) {
        const url = try locationUrlWithPathname(allocator, self.current_url, value);
        defer allocator.free(url);
        try self.assign(url);
        return;
    }

    fn setSearch(self: *LocationState, allocator: std.mem.Allocator, value: []const u8) errors.Result(void) {
        const url = try locationUrlWithSearch(allocator, self.current_url, value);
        defer allocator.free(url);
        try self.assign(url);
        return;
    }
};

const CssRuleListState = union(enum) {
    sheet: dom.NodeId,
    items: []Value,
};

const CssStyleRuleState = struct {
    selector_text: []const u8,
    css_text: []const u8,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssMediaRuleState = struct {
    condition_text: []const u8,
    css_text: []const u8,
    css_rules: []Value,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssSupportsRuleState = struct {
    condition_text: []const u8,
    css_text: []const u8,
    css_rules: []Value,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssDocumentRuleState = struct {
    condition_text: []const u8,
    css_text: []const u8,
    css_rules: []Value,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssContainerRuleState = struct {
    condition_text: []const u8,
    css_text: []const u8,
    css_rules: []Value,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssStartingStyleRuleState = struct {
    css_text: []const u8,
    css_rules: []Value,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssKeyframesRuleState = struct {
    name: []const u8,
    css_text: []const u8,
    css_rules: []Value,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssKeyframeRuleState = struct {
    key_text: []const u8,
    css_text: []const u8,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssFontFaceRuleState = struct {
    css_text: []const u8,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssFontFeatureValuesRuleState = struct {
    font_family: []const u8,
    css_text: []const u8,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssFontPaletteValuesRuleState = struct {
    name: []const u8,
    css_text: []const u8,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssColorProfileRuleState = struct {
    name: []const u8,
    src: []const u8,
    rendering_intent: []const u8,
    components: []const u8,
    css_text: []const u8,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssPageRuleState = struct {
    selector_text: []const u8,
    css_text: []const u8,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssPositionTryRuleState = struct {
    name: []const u8,
    css_text: []const u8,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssScopeRuleState = struct {
    start_text: ?[]const u8,
    end_text: ?[]const u8,
    css_text: []const u8,
    css_rules: []Value,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssLayerRuleState = struct {
    name_text: []const u8,
    css_text: []const u8,
    css_rules: []Value,
    is_statement: bool = false,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssCounterStyleRuleState = struct {
    name: []const u8,
    css_text: []const u8,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssPropertyRuleState = struct {
    name: []const u8,
    syntax: []const u8,
    inherits: bool,
    initial_value: []const u8,
    css_text: []const u8,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssImportRuleState = struct {
    href: []const u8,
    media_text: []const u8,
    supports_text: []const u8,
    layer_name: []const u8,
    css_text: []const u8,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssCharsetRuleState = struct {
    encoding: []const u8,
    css_text: []const u8,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssNamespaceRuleState = struct {
    prefix: []const u8,
    namespace_uri: []const u8,
    css_text: []const u8,
    parent_style_sheet: ?dom.NodeId = null,
    parent_rule: ?*const CssRuleState = null,
};

const CssRuleState = union(enum) {
    style: CssStyleRuleState,
    media: CssMediaRuleState,
    supports: CssSupportsRuleState,
    document: CssDocumentRuleState,
    container: CssContainerRuleState,
    starting_style: CssStartingStyleRuleState,
    keyframes: CssKeyframesRuleState,
    keyframe: CssKeyframeRuleState,
    font_face: CssFontFaceRuleState,
    font_feature_values: CssFontFeatureValuesRuleState,
    font_palette_values: CssFontPaletteValuesRuleState,
    color_profile: CssColorProfileRuleState,
    page: CssPageRuleState,
    position_try: CssPositionTryRuleState,
    scope: CssScopeRuleState,
    layer: CssLayerRuleState,
    counter_style: CssCounterStyleRuleState,
    property: CssPropertyRuleState,
    charset: CssCharsetRuleState,
    import: CssImportRuleState,
    namespace: CssNamespaceRuleState,
};

const PerformanceState = struct {
    host: *anyopaque,
    now_fn: *const fn (*anyopaque) i64,
    time_origin: i64 = 0,

    fn now(self: *const PerformanceState) f64 {
        return @floatFromInt(self.now_fn(self.host));
    }

    fn timeOrigin(self: *const PerformanceState) f64 {
        return @floatFromInt(self.time_origin);
    }
};

const CryptoState = struct {
    host: *anyopaque,
    random_uuid_fn: *const fn (*anyopaque, std.mem.Allocator) errors.Result([]const u8),

    fn randomUUID(self: *const CryptoState, allocator: std.mem.Allocator) errors.Result([]const u8) {
        return try self.random_uuid_fn(self.host, allocator);
    }
};

const HistoryState = struct {
    host: *anyopaque,
    length_fn: *const fn (*anyopaque) usize,
    state_fn: *const fn (*anyopaque) ?[]const u8,
    scroll_restoration_fn: *const fn (*anyopaque) []const u8,
    set_scroll_restoration_fn: *const fn (*anyopaque, []const u8) errors.Result(void),
    back_fn: *const fn (*anyopaque) errors.Result(void),
    forward_fn: *const fn (*anyopaque) errors.Result(void),
    go_fn: *const fn (*anyopaque, isize) errors.Result(void),
    push_state_fn: *const fn (*anyopaque, ?[]const u8, []const u8) errors.Result(void),
    replace_state_fn: *const fn (*anyopaque, ?[]const u8, []const u8) errors.Result(void),

    fn length(self: HistoryState) usize {
        return self.length_fn(self.host);
    }

    fn state(self: HistoryState) ?[]const u8 {
        return self.state_fn(self.host);
    }

    fn scrollRestoration(self: HistoryState) []const u8 {
        return self.scroll_restoration_fn(self.host);
    }

    fn setScrollRestoration(self: HistoryState, value: []const u8) errors.Result(void) {
        return self.set_scroll_restoration_fn(self.host, value);
    }

    fn back(self: HistoryState) errors.Result(void) {
        return self.back_fn(self.host);
    }

    fn forward(self: HistoryState) errors.Result(void) {
        return self.forward_fn(self.host);
    }

    fn go(self: HistoryState, delta: isize) errors.Result(void) {
        return self.go_fn(self.host, delta);
    }

    fn pushState(self: HistoryState, history_state: ?[]const u8, url: []const u8) errors.Result(void) {
        return self.push_state_fn(self.host, history_state, url);
    }

    fn replaceState(self: HistoryState, history_state: ?[]const u8, url: []const u8) errors.Result(void) {
        return self.replace_state_fn(self.host, history_state, url);
    }
};

const StorageState = struct {
    host: *anyopaque,
    target: StorageTarget,
    length_fn: *const fn (*anyopaque, StorageTarget) usize,
    get_item_fn: *const fn (*anyopaque, StorageTarget, []const u8) ?[]const u8,
    set_item_fn: *const fn (*anyopaque, StorageTarget, []const u8, []const u8) errors.Result(void),
    remove_item_fn: *const fn (*anyopaque, StorageTarget, []const u8) errors.Result(void),
    clear_fn: *const fn (*anyopaque, StorageTarget) errors.Result(void),
    key_fn: *const fn (*anyopaque, StorageTarget, usize) ?[]const u8,

    fn length(self: *const StorageState) usize {
        return self.length_fn(self.host, self.target);
    }

    fn getItem(self: *const StorageState, storage_key: []const u8) ?[]const u8 {
        return self.get_item_fn(self.host, self.target, storage_key);
    }

    fn setItem(self: *StorageState, storage_key: []const u8, value: []const u8) errors.Result(void) {
        return self.set_item_fn(self.host, self.target, storage_key, value);
    }

    fn removeItem(self: *StorageState, storage_key: []const u8) errors.Result(void) {
        return self.remove_item_fn(self.host, self.target, storage_key);
    }

    fn clear(self: *StorageState) errors.Result(void) {
        return self.clear_fn(self.host, self.target);
    }

    fn key(self: *const StorageState, index: usize) ?[]const u8 {
        return self.key_fn(self.host, self.target, index);
    }
};

const StylePropertyEntry = struct {
    name: []const u8,
    value: []const u8,
    important: bool = false,
};

const StyleValueEntry = struct {
    value: []const u8,
    important: bool,
};

const StyleDeclarationState = struct {
    host: *anyopaque,
    element: dom.NodeId,
    get_attribute_fn: *const fn (*anyopaque, dom.NodeId, []const u8, std.mem.Allocator) errors.Result(?[]const u8),
    set_attribute_fn: *const fn (*anyopaque, dom.NodeId, []const u8, []const u8) errors.Result(void),
};

const CssRuleStyleDeclarationState = struct {
    css_text: []const u8,
};

fn urlFragmentText(url: []const u8) []const u8 {
    const fragment_index = std.mem.indexOfScalar(u8, url, '#') orelse return "";
    if (fragment_index + 1 >= url.len) return "";
    return url[fragment_index + 1 ..];
}

const Value = union(enum) {
    undefined_value,
    null_value,
    boolean: bool,
    number: f64,
    string: []const u8,
    element: dom.NodeId,
    node: dom.NodeId,
    template_content: dom.NodeId,
    class_list: dom.NodeId,
    dataset: dom.NodeId,
    node_list: NodeList,
    html_collection: HtmlCollection,
    radio_node_list: RadioNodeList,
    media_query_list: MediaQueryList,
    string_list: StringListState,
    media_list: MediaListState,
    math: *MathState,
    crypto: *CryptoState,
    navigator,
    mime_type_array,
    screen,
    screen_orientation,
    performance: *PerformanceState,
    style_declaration: *StyleDeclarationState,
    storage: *StorageState,
    location: *LocationState,
    history: HistoryState,
    collection_iterator: *CollectionIteratorState,
    iterator_result: *IteratorResult,
    collection_entry: *CollectionEntry,
    document_scripts,
    document_anchors,
    document_style_sheets,
    css_rule_list: CssRuleListState,
    css_rule: CssRuleState,
    style_sheet: dom.NodeId,
    event: *ScriptEvent,
    function: ScriptFunction,
    document,
    window,
};

fn executeScriptSource(
    allocator: std.mem.Allocator,
    host: anytype,
    source: []const u8,
    source_name: []const u8,
    bindings: []const Binding,
) errors.Result(void) {
    _ = source_name;
    var arena = std.heap.ArenaAllocator.init(allocator);
    defer arena.deinit();

    const arena_alloc = arena.allocator();
    var parser = Parser.init(arena_alloc, source);
    const program = try parser.parseProgram();
    try evalProgram(arena_alloc, host, program, bindings);
    return;
}

const Parser = struct {
    allocator: std.mem.Allocator,
    input: []const u8,
    pos: usize = 0,

    fn init(allocator: std.mem.Allocator, input: []const u8) Parser {
        return .{
            .allocator = allocator,
            .input = input,
            .pos = 0,
        };
    }

    fn parseProgram(self: *Parser) errors.Result(Program) {
        var statements: std.ArrayList(Statement) = .empty;
        errdefer statements.deinit(self.allocator);

        self.skipWhitespaceAndComments();
        while (!self.isEof()) {
            try statements.append(self.allocator, try self.parseStatement());
            self.skipWhitespaceAndComments();
            _ = self.consumeChar(';');
            self.skipWhitespaceAndComments();
        }

        return Program{
            .statements = try self.allocator.dupe(Statement, statements.items),
        };
    }

    fn parseStatement(self: *Parser) errors.Result(Statement) {
        self.skipWhitespaceAndComments();
        if (self.consumeKeyword("const")) {
            self.skipWhitespaceAndComments();
            const name = try self.parseIdentifier();
            self.skipWhitespaceAndComments();
            if (!self.consumeChar('=')) return error.ScriptParse;
            const value = try self.parseExpression();
            return .{
                .const_declaration = .{
                    .name = name,
                    .value = value,
                },
            };
        }

        const expr = try self.parseExpression();
        self.skipWhitespaceAndComments();

        if (self.consumeStr("+=")) {
            const rhs = try self.parseExpression();
            const target = try self.exprToPropertyTarget(expr);
            return .{
                .assignment = .{
                    .target = target,
                    .value = try self.makeBinaryAdd(expr, rhs),
                },
            };
        }

        if (self.consumeChar('=')) {
            const value = try self.parseExpression();
            return .{
                .assignment = .{
                    .target = try self.exprToPropertyTarget(expr),
                    .value = value,
                },
            };
        }

        return .{ .expression = expr };
    }

    fn parseExpression(self: *Parser) errors.Result(*Expr) {
        return try self.parseAdditive();
    }

    fn parseAdditive(self: *Parser) errors.Result(*Expr) {
        var expr = try self.parsePostfix();
        while (true) {
            self.skipWhitespaceAndComments();
            if (self.peekChar() == '+' and self.peekNextChar() != '=') {
                _ = self.consumeChar('+');
                const rhs = try self.parsePostfix();
                expr = try self.makeBinaryAdd(expr, rhs);
                continue;
            }
            break;
        }
        return expr;
    }

    fn parsePostfix(self: *Parser) errors.Result(*Expr) {
        var expr = try self.parsePrimary();
        while (true) {
            self.skipWhitespaceAndComments();
            if (self.consumeChar('.')) {
                const property = try self.parseIdentifier();
                expr = try self.makeExpr(.{
                    .member = .{
                        .object = expr,
                        .property = property,
                    },
                });
                continue;
            }

            if (self.peekChar() == '(') {
                const args = try self.parseCallArguments();
                expr = try self.makeExpr(.{
                    .call = .{
                        .callee = expr,
                        .args = args,
                    },
                });
                continue;
            }

            break;
        }

        return expr;
    }

    fn parsePrimary(self: *Parser) errors.Result(*Expr) {
        self.skipWhitespaceAndComments();
        if (self.isEof()) return error.ScriptParse;

        const current = self.peekChar().?;
        return switch (current) {
            '\'', '"' => self.makeExpr(.{ .string = try self.parseString() }),
            '-' => blk: {
                const next = self.peekNextChar() orelse break :blk error.ScriptParse;
                if (!std.ascii.isDigit(next)) break :blk error.ScriptParse;
                _ = self.consumeChar('-');
                const digits = try self.parseNumber();
                const text = try std.fmt.allocPrint(self.allocator, "-{s}", .{digits});
                break :blk try self.makeExpr(.{ .number = text });
            },
            '(' => blk: {
                if (try self.tryParseArrowFunction()) |function| {
                    break :blk try self.makeExpr(.{ .arrow_function = function });
                }

                _ = self.consumeChar('(');
                const expr = try self.parseExpression();
                self.skipWhitespaceAndComments();
                if (!self.consumeChar(')')) return error.ScriptParse;
                break :blk expr;
            },
            else => if (isIdentifierStartByte(current))
                self.parseIdentifierExpr()
            else if (std.ascii.isDigit(current))
                self.makeExpr(.{ .number = try self.parseNumber() })
            else
                error.ScriptParse,
        };
    }

    fn tryParseArrowFunction(self: *Parser) errors.Result(?ScriptFunction) {
        const start = self.pos;
        if (self.peekChar() != '(') return null;

        self.pos += 1;
        self.skipWhitespaceAndComments();

        var params: std.ArrayList([]const u8) = .empty;
        errdefer params.deinit(self.allocator);

        if (self.peekChar() != ')') {
            while (true) {
                const param = self.parseIdentifier() catch {
                    self.pos = start;
                    return null;
                };
                try params.append(self.allocator, param);
                self.skipWhitespaceAndComments();
                if (self.consumeChar(',')) {
                    self.skipWhitespaceAndComments();
                    continue;
                }
                break;
            }
        }

        self.skipWhitespaceAndComments();
        if (!self.consumeChar(')')) {
            self.pos = start;
            return null;
        }

        self.skipWhitespaceAndComments();
        if (!self.consumeStr("=>")) {
            self.pos = start;
            return null;
        }

        self.skipWhitespaceAndComments();
        if (self.peekChar() != '{') {
            self.pos = start;
            return null;
        }

        const body_source = try self.captureBracedBlock();
        return ScriptFunction{
            .params = try self.allocator.dupe([]const u8, params.items),
            .body_source = body_source,
        };
    }

    fn captureBracedBlock(self: *Parser) errors.Result([]const u8) {
        if (!self.consumeChar('{')) return error.ScriptParse;

        const body_start = self.pos;
        var depth: usize = 1;
        var quote: ?u8 = null;

        while (self.pos < self.input.len) {
            const ch = self.input[self.pos];
            if (quote) |current_quote| {
                if (ch == '\\') {
                    self.pos += 2;
                    continue;
                }
                self.pos += 1;
                if (ch == current_quote) {
                    quote = null;
                }
                continue;
            }

            if (ch == '\'' or ch == '"') {
                quote = ch;
                self.pos += 1;
                continue;
            }

            if (ch == '/' and self.peekNextChar() == '/') {
                self.pos += 2;
                while (self.pos < self.input.len and self.input[self.pos] != '\n') {
                    self.pos += 1;
                }
                continue;
            }

            if (ch == '/' and self.peekNextChar() == '*') {
                self.pos += 2;
                while (self.pos + 1 < self.input.len) {
                    if (self.input[self.pos] == '*' and self.input[self.pos + 1] == '/') {
                        self.pos += 2;
                        break;
                    }
                    self.pos += 1;
                }
                continue;
            }

            if (ch == '{') {
                depth += 1;
                self.pos += 1;
                continue;
            }

            if (ch == '}') {
                depth -= 1;
                if (depth == 0) {
                    const body = self.input[body_start..self.pos];
                    self.pos += 1;
                    return body;
                }
                self.pos += 1;
                continue;
            }

            self.pos += 1;
        }

        return error.ScriptParse;
    }

    fn parseIdentifierExpr(self: *Parser) errors.Result(*Expr) {
        const ident = try self.parseIdentifier();
        return self.makeExpr(.{
            .identifier = ident,
        });
    }

    fn parseCallArguments(self: *Parser) errors.Result([]*Expr) {
        if (!self.consumeChar('(')) return error.ScriptParse;
        self.skipWhitespaceAndComments();

        var args: std.ArrayList(*Expr) = .empty;
        errdefer args.deinit(self.allocator);

        if (self.consumeChar(')')) {
            return try self.allocator.dupe(*Expr, args.items);
        }

        while (true) {
            const expr = try self.parseExpression();
            try args.append(self.allocator, expr);
            self.skipWhitespaceAndComments();
            if (self.consumeChar(')')) {
                break;
            }
            if (!self.consumeChar(',')) return error.ScriptParse;
            self.skipWhitespaceAndComments();
        }

        return try self.allocator.dupe(*Expr, args.items);
    }

    fn parseString(self: *Parser) errors.Result([]const u8) {
        const quote = self.bumpByte() orelse return error.ScriptParse;
        var out: std.ArrayList(u8) = .empty;
        errdefer out.deinit(self.allocator);

        while (true) {
            const ch = self.bumpByte() orelse return error.ScriptParse;
            if (ch == quote) break;
            if (ch == '\\') {
                const escaped = self.bumpByte() orelse return error.ScriptParse;
                try out.append(self.allocator, switch (escaped) {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '\'' => '\'',
                    '"' => '"',
                    else => escaped,
                });
                continue;
            }
            try out.append(self.allocator, ch);
        }

        return try self.allocator.dupe(u8, out.items);
    }

    fn parseNumber(self: *Parser) errors.Result([]const u8) {
        const start = self.pos;
        var seen_dot = false;

        while (self.peekChar()) |ch| {
            if (std.ascii.isDigit(ch)) {
                self.pos += 1;
                continue;
            }
            if (ch == '.' and !seen_dot) {
                seen_dot = true;
                self.pos += 1;
                continue;
            }
            break;
        }

        if (self.pos == start) return error.ScriptParse;
        return self.input[start..self.pos];
    }

    fn parseIdentifier(self: *Parser) errors.Result([]const u8) {
        const start = self.pos;
        const first = self.peekChar() orelse return error.ScriptParse;
        if (!isIdentifierStartByte(first)) return error.ScriptParse;
        self.pos += 1;

        while (self.peekChar()) |ch| {
            if (!isIdentifierContinueByte(ch)) break;
            self.pos += 1;
        }

        return self.input[start..self.pos];
    }

    fn exprToPropertyTarget(self: *Parser, expr: *Expr) errors.Result(PropertyTarget) {
        _ = self;
        return switch (expr.*) {
            .member => |member| .{
                .object = member.object,
                .property = member.property,
            },
            else => error.ScriptParse,
        };
    }

    fn makeExpr(self: *Parser, value: Expr) errors.Result(*Expr) {
        const expr = try self.allocator.create(Expr);
        expr.* = value;
        return expr;
    }

    fn makeBinaryAdd(self: *Parser, left: *Expr, right: *Expr) errors.Result(*Expr) {
        return try self.makeExpr(.{
            .binary_add = .{
                .left = left,
                .right = right,
            },
        });
    }

    fn skipWhitespaceAndComments(self: *Parser) void {
        while (!self.isEof()) {
            const ch = self.peekChar().?;
            if (isWhitespaceByte(ch)) {
                self.pos += 1;
                continue;
            }

            if (ch == '/' and self.peekNextChar() == '/') {
                self.pos += 2;
                while (!self.isEof()) {
                    const next = self.bumpByte() orelse break;
                    if (next == '\n') break;
                }
                continue;
            }

            if (ch == '/' and self.peekNextChar() == '*') {
                self.pos += 2;
                while (!self.isEof()) {
                    const next = self.bumpByte() orelse break;
                    if (next == '*' and self.peekChar() == '/') {
                        self.pos += 1;
                        break;
                    }
                }
                continue;
            }

            break;
        }
    }

    fn consumeStr(self: *Parser, expected: []const u8) bool {
        if (!std.mem.startsWith(u8, self.input[self.pos..], expected)) {
            return false;
        }
        self.pos += expected.len;
        return true;
    }

    fn consumeKeyword(self: *Parser, expected: []const u8) bool {
        if (!self.consumeStr(expected)) return false;
        if (self.pos < self.input.len and isIdentifierContinueByte(self.input[self.pos])) {
            self.pos -= expected.len;
            return false;
        }
        return true;
    }

    fn consumeChar(self: *Parser, expected: u8) bool {
        if (self.peekChar() == expected) {
            self.pos += 1;
            return true;
        }
        return false;
    }

    fn peekChar(self: *Parser) ?u8 {
        return if (self.pos < self.input.len) self.input[self.pos] else null;
    }

    fn peekNextChar(self: *Parser) ?u8 {
        if (self.pos + 1 >= self.input.len) return null;
        return self.input[self.pos + 1];
    }

    fn bumpByte(self: *Parser) ?u8 {
        const ch = self.peekChar() orelse return null;
        self.pos += 1;
        return ch;
    }

    fn isEof(self: *Parser) bool {
        return self.pos >= self.input.len;
    }
};

fn evalProgram(
    allocator: std.mem.Allocator,
    host: anytype,
    program: Program,
    bindings: []const Binding,
) errors.Result(void) {
    var local_bindings: std.ArrayList(Binding) = .empty;
    defer local_bindings.deinit(allocator);
    try local_bindings.appendSlice(allocator, bindings);

    for (program.statements) |statement| {
        try evalStatement(allocator, host, &local_bindings, statement);
    }
    return;
}

fn evalStatement(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: *std.ArrayList(Binding),
    statement: Statement,
) errors.Result(void) {
    switch (statement) {
        .expression => |expr| {
            _ = try evalExpr(allocator, host, bindings.items, expr);
        },
        .assignment => |assignment| {
            const value = try evalExpr(allocator, host, bindings.items, assignment.value);
            try evalAssignment(allocator, host, bindings.items, assignment.target, value);
        },
        .const_declaration => |declaration| {
            const value = try evalExpr(allocator, host, bindings.items, declaration.value);
            try bindings.append(allocator, .{
                .name = declaration.name,
                .value = value,
            });
        },
    }
}

fn evalAssignment(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    target: PropertyTarget,
    value: Value,
) errors.Result(void) {
    const object = try evalExpr(allocator, host, bindings, target.object);
    switch (object) {
        .document => {
            if (std.mem.eql(u8, target.property, "nodeValue")) {
                return;
            }
            if (std.mem.eql(u8, target.property, "title")) {
                const text = try asString(allocator, value);
                try host.setDocumentTitle(text);
                return;
            }

            if (std.mem.eql(u8, target.property, "cookie")) {
                const text = try asString(allocator, value);
                try host.setDocumentCookie(text);
                return;
            }

            if (std.mem.eql(u8, target.property, "location")) {
                const text = try asString(allocator, value);
                try host.assignLocation(text);
                return;
            }

            if (std.mem.eql(u8, target.property, "dir")) {
                const text = try asString(allocator, value);
                try host.setDocumentDir(text);
                return;
            }

            return error.ScriptRuntime;
        },
        .screen => return error.ScriptRuntime,
        .screen_orientation => return error.ScriptRuntime,
        .math => return error.ScriptRuntime,
        .window => {
            if (std.mem.eql(u8, target.property, "name")) {
                const text = try asString(allocator, value);
                try host.setWindowName(text);
                return;
            }

            if (std.mem.eql(u8, target.property, "onhashchange")) {
                const next_handler = switch (value) {
                    .function => |function| function,
                    .null_value, .undefined_value => null,
                    else => return error.ScriptRuntime,
                };
                try host.setWindowHashChange(next_handler);
                return;
            }

            if (std.mem.eql(u8, target.property, "onload")) {
                const next_handler = switch (value) {
                    .function => |function| function,
                    .null_value, .undefined_value => null,
                    else => return error.ScriptRuntime,
                };
                try host.setWindowLoad(next_handler);
                return;
            }

            if (std.mem.eql(u8, target.property, "onfocus")) {
                const next_handler = switch (value) {
                    .function => |function| function,
                    .null_value, .undefined_value => null,
                    else => return error.ScriptRuntime,
                };
                try host.setWindowFocus(next_handler);
                return;
            }

            if (std.mem.eql(u8, target.property, "onblur")) {
                const next_handler = switch (value) {
                    .function => |function| function,
                    .null_value, .undefined_value => null,
                    else => return error.ScriptRuntime,
                };
                try host.setWindowBlur(next_handler);
                return;
            }

            if (std.mem.eql(u8, target.property, "onpopstate")) {
                const next_handler = switch (value) {
                    .function => |function| function,
                    .null_value, .undefined_value => null,
                    else => return error.ScriptRuntime,
                };
                try host.setWindowPopState(next_handler);
                return;
            }

            if (std.mem.eql(u8, target.property, "onstorage")) {
                const next_handler = switch (value) {
                    .function => |function| function,
                    .null_value, .undefined_value => null,
                    else => return error.ScriptRuntime,
                };
                try host.setWindowStorage(next_handler);
                return;
            }

            if (std.mem.eql(u8, target.property, "title")) {
                const text = try asString(allocator, value);
                try host.setDocumentTitle(text);
                return;
            }

            if (std.mem.eql(u8, target.property, "domain")) {
                return error.ScriptRuntime;
            }

            if (std.mem.eql(u8, target.property, "location")) {
                const text = try asString(allocator, value);
                try host.assignLocation(text);
                return;
            }

            return error.ScriptRuntime;
        },
        .history => |history| {
            if (std.mem.eql(u8, target.property, "scrollRestoration")) {
                const text = try asString(allocator, value);
                try history.setScrollRestoration(text);
                return;
            }

            return error.ScriptRuntime;
        },
        .navigator => return error.ScriptRuntime,
        .media_query_list => |mql| {
            if (std.mem.eql(u8, target.property, "onchange")) {
                if (mql.set_onchange_fn) |set_onchange_fn| {
                    const host_ptr = mql.host orelse return error.ScriptRuntime;
                    const next_handler = switch (value) {
                        .function => |function| function,
                        .null_value, .undefined_value => null,
                        else => return error.ScriptRuntime,
                    };
                    try set_onchange_fn(host_ptr, mql.media, next_handler);
                    return;
                }
            }

            return error.ScriptRuntime;
        },
        .mime_type_array => return error.ScriptRuntime,
        .location => |location| {
            if (std.mem.eql(u8, target.property, "hash")) {
                const text = try asString(allocator, value);
                try location.setHash(text);
                return;
            }

            if (std.mem.eql(u8, target.property, "href")) {
                const text = try asString(allocator, value);
                try location.assign(text);
                return;
            }

            if (std.mem.eql(u8, target.property, "protocol")) {
                const text = try asString(allocator, value);
                try location.setProtocol(allocator, text);
                return;
            }

            if (std.mem.eql(u8, target.property, "host")) {
                const text = try asString(allocator, value);
                try location.setHost(allocator, text);
                return;
            }

            if (std.mem.eql(u8, target.property, "hostname")) {
                const text = try asString(allocator, value);
                try location.setHostname(allocator, text);
                return;
            }

            if (std.mem.eql(u8, target.property, "port")) {
                const text = try asString(allocator, value);
                try location.setPort(allocator, text);
                return;
            }

            if (std.mem.eql(u8, target.property, "username")) {
                const text = try asString(allocator, value);
                try location.setUsername(allocator, text);
                return;
            }

            if (std.mem.eql(u8, target.property, "password")) {
                const text = try asString(allocator, value);
                try location.setPassword(allocator, text);
                return;
            }

            if (std.mem.eql(u8, target.property, "pathname")) {
                const text = try asString(allocator, value);
                try location.setPathname(allocator, text);
                return;
            }

            if (std.mem.eql(u8, target.property, "search")) {
                const text = try asString(allocator, value);
                try location.setSearch(allocator, text);
                return;
            }

            return error.ScriptRuntime;
        },
        .style_declaration => |style| {
            if (std.mem.eql(u8, target.property, "cssText")) {
                const text = try asString(allocator, value);
                try styleDeclarationSetCssText(allocator, style, text);
                return;
            }

            if (std.mem.eql(u8, target.property, "length") or
                std.mem.eql(u8, target.property, "item") or
                std.mem.eql(u8, target.property, "getPropertyValue") or
                std.mem.eql(u8, target.property, "setProperty") or
                std.mem.eql(u8, target.property, "removeProperty") or
                std.mem.eql(u8, target.property, "toString"))
            {
                return error.ScriptRuntime;
            }

            const property_name = try stylePropertyName(allocator, target.property);
            defer allocator.free(property_name);
            const text = try asString(allocator, value);
            try styleDeclarationSetProperty(allocator, style, property_name, text, null);
            return;
        },
        .style_sheet => |sheet_id| {
            if (std.mem.eql(u8, target.property, "disabled")) {
                if (isTruthy(value)) {
                    try host.domStoreMut().setAttribute(sheet_id, "disabled", "");
                } else {
                    try host.domStoreMut().removeAttribute(sheet_id, "disabled");
                }
                return;
            }

            return error.ScriptRuntime;
        },
        .radio_node_list => |list| {
            if (std.mem.eql(u8, target.property, "value")) {
                const text = try asString(allocator, value);
                try radioNodeListSetValue(allocator, host, list, text);
                return;
            }

            return error.ScriptRuntime;
        },
        .element => |element| {
            if (std.mem.eql(u8, target.property, "nodeValue")) {
                return;
            }
            if (std.mem.eql(u8, target.property, "id")) {
                const text = try asString(allocator, value);
                try host.domStoreMut().setAttribute(element, "id", text);
                return;
            }
            if (std.mem.eql(u8, target.property, "title")) {
                const text = try asString(allocator, value);
                try host.domStoreMut().setAttribute(element, "title", text);
                return;
            }
            if (std.mem.eql(u8, target.property, "lang")) {
                const text = try asString(allocator, value);
                try host.domStoreMut().setAttribute(element, "lang", text);
                return;
            }
            if (std.mem.eql(u8, target.property, "dir")) {
                const text = try asString(allocator, value);
                try host.domStoreMut().setAttribute(element, "dir", text);
                return;
            }
            if (std.mem.eql(u8, target.property, "hidden")) {
                if (isTruthy(value)) {
                    try host.domStoreMut().setAttribute(element, "hidden", "");
                } else {
                    try host.domStoreMut().removeAttribute(element, "hidden");
                }
                return;
            }
            if (std.mem.eql(u8, target.property, "inert")) {
                if (isTruthy(value)) {
                    try host.domStoreMut().setAttribute(element, "inert", "");
                } else {
                    try host.domStoreMut().removeAttribute(element, "inert");
                }
                return;
            }
            if (std.mem.eql(u8, target.property, "disabled")) {
                if (!try elementSupportsDisabledProperty(host, element)) {
                    return error.ScriptRuntime;
                }
                if (isTruthy(value)) {
                    try host.domStoreMut().setAttribute(element, "disabled", "");
                } else {
                    try host.domStoreMut().removeAttribute(element, "disabled");
                }
                return;
            }
            if (std.mem.eql(u8, target.property, "required")) {
                if (!try elementSupportsRequiredProperty(host, element)) {
                    return error.ScriptRuntime;
                }
                if (isTruthy(value)) {
                    try host.domStoreMut().setAttribute(element, "required", "");
                } else {
                    try host.domStoreMut().removeAttribute(element, "required");
                }
                return;
            }
            if (std.mem.eql(u8, target.property, "translate")) {
                const text = if (isTruthy(value)) "yes" else "no";
                try host.domStoreMut().setAttribute(element, "translate", text);
                return;
            }
            if (std.mem.eql(u8, target.property, "spellcheck")) {
                const text = if (isTruthy(value)) "true" else "false";
                try host.domStoreMut().setAttribute(element, "spellcheck", text);
                return;
            }
            if (std.mem.eql(u8, target.property, "draggable")) {
                if (isTruthy(value)) {
                    try host.domStoreMut().setAttribute(element, "draggable", "");
                } else {
                    try host.domStoreMut().removeAttribute(element, "draggable");
                }
                return;
            }
            if (std.mem.eql(u8, target.property, "nonce")) {
                const text = try asString(allocator, value);
                try host.domStoreMut().setAttribute(element, "nonce", text);
                return;
            }
            if (std.mem.eql(u8, target.property, "autocapitalize")) {
                const text = try asString(allocator, value);
                try host.domStoreMut().setAttribute(element, "autocapitalize", text);
                return;
            }
            if (std.mem.eql(u8, target.property, "autofocus")) {
                if (isTruthy(value)) {
                    try host.domStoreMut().setAttribute(element, "autofocus", "");
                } else {
                    try host.domStoreMut().removeAttribute(element, "autofocus");
                }
                return;
            }
            if (std.mem.eql(u8, target.property, "autocomplete")) {
                const text = try asString(allocator, value);
                try host.domStoreMut().setAttribute(element, "autocomplete", text);
                return;
            }
            if (std.mem.eql(u8, target.property, "name")) {
                const text = try asString(allocator, value);
                try host.domStoreMut().setAttribute(element, "name", text);
                return;
            }
            if (std.mem.eql(u8, target.property, "placeholder")) {
                if (!try elementSupportsPlaceholderProperty(host, element)) {
                    return error.ScriptRuntime;
                }
                const text = try asString(allocator, value);
                try host.domStoreMut().setAttribute(element, "placeholder", text);
                return;
            }
            if (std.mem.eql(u8, target.property, "inputMode")) {
                const text = try asString(allocator, value);
                try host.domStoreMut().setAttribute(element, "inputmode", text);
                return;
            }
            if (std.mem.eql(u8, target.property, "readOnly")) {
                if (isTruthy(value)) {
                    try host.domStoreMut().setAttribute(element, "readonly", "");
                } else {
                    try host.domStoreMut().removeAttribute(element, "readonly");
                }
                return;
            }
            if (std.mem.eql(u8, target.property, "accessKey")) {
                const text = try asString(allocator, value);
                try host.domStoreMut().setAttribute(element, "accesskey", text);
                return;
            }
            if (std.mem.eql(u8, target.property, "contentEditable")) {
                const text = try asString(allocator, value);
                try host.domStoreMut().setAttribute(element, "contenteditable", text);
                return;
            }
            if (std.mem.eql(u8, target.property, "isContentEditable")) {
                return error.ScriptRuntime;
            }
            if (std.mem.eql(u8, target.property, "tabIndex")) {
                const tab_index = try tabIndexFromValue(allocator, value);
                const text = try std.fmt.allocPrint(allocator, "{d}", .{tab_index});
                defer allocator.free(text);
                try host.domStoreMut().setAttribute(element, "tabindex", text);
                return;
            }
            if (std.mem.eql(u8, target.property, "innerHTML")) {
                const html = try asString(allocator, value);
                try host.domStoreMut().setInnerHtml(element, html);
                return;
            }

            if (std.mem.eql(u8, target.property, "outerHTML")) {
                const html = try asString(allocator, value);
                try host.domStoreMut().setOuterHtml(element, html);
                return;
            }

            if (std.mem.eql(u8, target.property, "className")) {
                const text = try asString(allocator, value);
                try host.domStoreMut().setAttribute(element, "class", text);
                return;
            }

            if (std.mem.eql(u8, target.property, "style")) {
                return error.ScriptRuntime;
            }

            if (std.mem.eql(u8, target.property, "textContent")) {
                const text = try asString(allocator, value);
                try host.domStoreMut().setTextContent(element, text);
                return;
            }

            if (std.mem.eql(u8, target.property, "value")) {
                const text = try asString(allocator, value);
                host.domStoreMut().setFormControlValue(element, text) catch return error.ScriptRuntime;
                return;
            }
            if (std.mem.eql(u8, target.property, "selectedIndex")) {
                const tag_name = host.domStore().tagNameForNode(element) orelse return error.ScriptRuntime;
                if (!std.mem.eql(u8, tag_name, "select")) {
                    return error.ScriptRuntime;
                }
                const selected_index = try historyDeltaFromValue(allocator, value);
                host.domStoreMut().setSelectSelectedIndex(element, selected_index) catch return error.ScriptRuntime;
                return;
            }

            if (std.mem.eql(u8, target.property, "selected")) {
                const tag_name = host.domStore().tagNameForNode(element) orelse return error.ScriptRuntime;
                if (!std.mem.eql(u8, tag_name, "option")) {
                    return error.ScriptRuntime;
                }
                try host.domStoreMut().setOptionSelected(element, isTruthy(value));
                return;
            }

            if (std.mem.eql(u8, target.property, "selectionStart")) {
                const current = host.domStore().selectionStateForNode(element) catch |err| switch (err) {
                    error.OutOfMemory => return error.OutOfMemory,
                    else => return error.ScriptRuntime,
                };
                const state = current orelse return error.ScriptRuntime;
                const start = try selectionIndexFromValue(allocator, value);
                host.domStoreMut().setSelectionRange(element, start, state.end, .none) catch |err| switch (err) {
                    error.OutOfMemory => return error.OutOfMemory,
                    else => return error.ScriptRuntime,
                };
                return;
            }

            if (std.mem.eql(u8, target.property, "selectionEnd")) {
                const current = host.domStore().selectionStateForNode(element) catch |err| switch (err) {
                    error.OutOfMemory => return error.OutOfMemory,
                    else => return error.ScriptRuntime,
                };
                const state = current orelse return error.ScriptRuntime;
                const end = try selectionIndexFromValue(allocator, value);
                host.domStoreMut().setSelectionRange(element, state.start, end, .none) catch |err| switch (err) {
                    error.OutOfMemory => return error.OutOfMemory,
                    else => return error.ScriptRuntime,
                };
                return;
            }

            if (std.mem.eql(u8, target.property, "selectionDirection")) {
                const direction_text = try asString(allocator, value);
                const direction = selectionDirectionFromString(direction_text) orelse return error.ScriptRuntime;
                const current = host.domStore().selectionStateForNode(element) catch |err| switch (err) {
                    error.OutOfMemory => return error.OutOfMemory,
                    else => return error.ScriptRuntime,
                };
                const state = current orelse return error.ScriptRuntime;
                host.domStoreMut().setSelectionRange(element, state.start, state.end, direction) catch |err| switch (err) {
                    error.OutOfMemory => return error.OutOfMemory,
                    else => return error.ScriptRuntime,
                };
                return;
            }

            if (std.mem.eql(u8, target.property, "checked")) {
                host.domStoreMut().setFormControlChecked(element, isTruthy(value)) catch return error.ScriptRuntime;
                return;
            }

            if (std.mem.eql(u8, target.property, "classList")) {
                return error.ScriptRuntime;
            }

            if (std.mem.eql(u8, target.property, "dataset")) {
                return error.ScriptRuntime;
            }

            return error.ScriptRuntime;
        },
        .template_content => |element| {
            if (std.mem.eql(u8, target.property, "nodeValue")) {
                return;
            }
            if (std.mem.eql(u8, target.property, "innerHTML")) {
                const html = try asString(allocator, value);
                try host.domStoreMut().setInnerHtml(element, html);
                return;
            }

            if (std.mem.eql(u8, target.property, "textContent")) {
                const text = try asString(allocator, value);
                try host.domStoreMut().setTextContent(element, text);
                return;
            }

            return error.ScriptRuntime;
        },
        .dataset => |element| {
            const attribute_name = try datasetAttributeName(allocator, target.property);
            const text = try asString(allocator, value);
            try host.domStoreMut().setAttribute(element, attribute_name, text);
            return;
        },
        .node => |node_id| {
            if (std.mem.eql(u8, target.property, "nodeValue") or
                std.mem.eql(u8, target.property, "textContent") or
                std.mem.eql(u8, target.property, "data"))
            {
                const text = try asString(allocator, value);
                host.domStoreMut().setNodeValue(node_id, text) catch |err| switch (err) {
                    error.OutOfMemory => return error.OutOfMemory,
                    else => return error.ScriptRuntime,
                };
                return;
            }

            return error.ScriptRuntime;
        },
        .class_list => return error.ScriptRuntime,
        else => return error.ScriptRuntime,
    }
}

fn evalExpr(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    expr: *Expr,
) errors.Result(Value) {
    return switch (expr.*) {
        .identifier => |name| evalIdentifier(allocator, host, bindings, name),
        .string => |value| .{ .string = value },
        .number => |value| .{ .number = try parseNumberLiteral(value) },
        .boolean => |value| .{ .boolean = value },
        .null_value => .{ .null_value = {} },
        .undefined_value => .{ .undefined_value = {} },
        .member => |member| evalMember(allocator, host, bindings, member),
        .call => |call| evalCall(allocator, host, bindings, call),
        .binary_add => |binary| evalBinaryAdd(allocator, host, bindings, binary),
        .arrow_function => |function| .{ .function = function },
    };
}

fn evalIdentifier(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    name: []const u8,
) errors.Result(Value) {
    var index = bindings.len;
    while (index > 0) {
        index -= 1;
        const binding = bindings[index];
        if (std.mem.eql(u8, binding.name, name)) {
            return binding.value;
        }
    }

    if (std.mem.eql(u8, name, "document")) return .{ .document = {} };
    if (std.mem.eql(u8, name, "window")) return .{ .window = {} };
    if (std.mem.eql(u8, name, "performance")) return try makePerformanceValue(allocator, host);
    if (std.mem.eql(u8, name, "Math")) return try makeMathValue(allocator, host);
    if (std.mem.eql(u8, name, "crypto")) return try makeCryptoValue(allocator, host);
    if (std.mem.eql(u8, name, "undefined")) return .{ .undefined_value = {} };
    if (std.mem.eql(u8, name, "null")) return .{ .null_value = {} };
    if (std.mem.eql(u8, name, "true")) return .{ .boolean = true };
    if (std.mem.eql(u8, name, "false")) return .{ .boolean = false };
    if (std.mem.eql(u8, name, "event")) return error.ScriptRuntime;
    return error.ScriptRuntime;
}

fn evalMember(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    member: MemberExpr,
) errors.Result(Value) {
    const object = try evalExpr(allocator, host, bindings, member.object);
    return switch (object) {
        .document => blk: {
            if (std.mem.eql(u8, member.property, "defaultView")) {
                break :blk Value{ .window = {} };
            }
            if (std.mem.eql(u8, member.property, "documentElement")) {
                if (host.documentElement()) |element| {
                    break :blk Value{ .element = element };
                }
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "head")) {
                if (host.documentHead()) |element| {
                    break :blk Value{ .element = element };
                }
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "body")) {
                if (host.documentBody()) |element| {
                    break :blk Value{ .element = element };
                }
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "scrollingElement")) {
                if (host.documentScrollingElement()) |element| {
                    break :blk Value{ .element = element };
                }
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "compatMode")) {
                break :blk Value{ .string = host.documentCompatMode() };
            }
            if (std.mem.eql(u8, member.property, "characterSet") or
                std.mem.eql(u8, member.property, "charset"))
            {
                break :blk Value{ .string = host.documentCharacterSet() };
            }
            if (std.mem.eql(u8, member.property, "contentType")) {
                break :blk Value{ .string = host.documentContentType() };
            }
            if (std.mem.eql(u8, member.property, "referrer")) {
                break :blk Value{ .string = host.documentReferrer() };
            }
            if (std.mem.eql(u8, member.property, "cookie")) {
                break :blk Value{ .string = try host.documentCookie(allocator) };
            }
            if (std.mem.eql(u8, member.property, "visibilityState")) {
                break :blk Value{ .string = host.documentVisibilityState() };
            }
            if (std.mem.eql(u8, member.property, "hidden")) {
                break :blk Value{ .boolean = host.documentHidden() };
            }
            if (std.mem.eql(u8, member.property, "dir")) {
                break :blk Value{ .string = host.documentDir() };
            }
            if (std.mem.eql(u8, member.property, "title")) {
                break :blk Value{ .string = host.documentTitle() };
            }
            if (std.mem.eql(u8, member.property, "readyState")) {
                break :blk Value{ .string = host.documentReadyState() };
            }
            if (std.mem.eql(u8, member.property, "currentScript")) {
                if (host.currentScript()) |script_id| {
                    break :blk Value{ .element = script_id };
                }
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "activeElement")) {
                if (host.documentActiveElement()) |element| {
                    break :blk Value{ .element = element };
                }
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "location")) {
                break :blk try makeLocationValue(allocator, host);
            }
            if (std.mem.eql(u8, member.property, "crypto")) {
                break :blk try makeCryptoValue(allocator, host);
            }
            if (std.mem.eql(u8, member.property, "URL") or
                std.mem.eql(u8, member.property, "documentURI") or
                std.mem.eql(u8, member.property, "baseURI"))
            {
                break :blk Value{ .string = host.currentLocationUrl() };
            }
            if (std.mem.eql(u8, member.property, "origin")) {
                break :blk Value{ .string = try originFromUrl(allocator, host.currentLocationUrl()) };
            }
            if (std.mem.eql(u8, member.property, "domain")) {
                break :blk Value{ .string = try host.documentDomain(allocator) };
            }
            if (std.mem.eql(u8, member.property, "images")) {
                break :blk Value{ .html_collection = .{ .root = host.domStore().documentId(), .kind = .document_images } };
            }
            if (std.mem.eql(u8, member.property, "links")) {
                break :blk Value{ .html_collection = .{ .root = host.domStore().documentId(), .kind = .document_links } };
            }
            if (std.mem.eql(u8, member.property, "embeds")) {
                break :blk Value{ .html_collection = .{ .root = host.domStore().documentId(), .kind = .document_embeds } };
            }
            if (std.mem.eql(u8, member.property, "plugins")) {
                break :blk Value{ .html_collection = .{ .root = host.domStore().documentId(), .kind = .navigator_plugins } };
            }
            if (std.mem.eql(u8, member.property, "applets")) {
                break :blk Value{ .html_collection = .{ .root = host.domStore().documentId(), .kind = .document_applets } };
            }
            if (std.mem.eql(u8, member.property, "scripts")) {
                break :blk Value{ .document_scripts = {} };
            }
            if (std.mem.eql(u8, member.property, "anchors")) {
                break :blk Value{ .document_anchors = {} };
            }
            if (std.mem.eql(u8, member.property, "styleSheets")) {
                break :blk Value{ .document_style_sheets = {} };
            }
            if (std.mem.eql(u8, member.property, "all")) {
                break :blk Value{ .html_collection = .{ .root = host.domStore().documentId(), .kind = .document_all } };
            }
            if (std.mem.eql(u8, member.property, "forms")) {
                break :blk Value{ .html_collection = .{ .root = host.domStore().documentId(), .kind = .forms } };
            }
            if (std.mem.eql(u8, member.property, "childNodes")) {
                break :blk Value{
                    .node_list = .{
                        .items = &.{},
                        .item_kind = .node,
                        .live_root = host.domStore().documentId(),
                        .live_kind = .children,
                    },
                };
            }
            if (std.mem.eql(u8, member.property, "children")) {
                break :blk Value{ .html_collection = .{ .root = host.domStore().documentId() } };
            }
            if (try nodeLikeMember(allocator, host, host.domStore().documentId(), member.property)) |value| {
                break :blk value;
            }
            break :blk error.ScriptRuntime;
        },
        .window => blk: {
            if (std.mem.eql(u8, member.property, "document")) {
                break :blk Value{ .document = {} };
            }
            if (std.mem.eql(u8, member.property, "window") or
                std.mem.eql(u8, member.property, "self") or
                std.mem.eql(u8, member.property, "top") or
                std.mem.eql(u8, member.property, "parent"))
            {
                break :blk Value{ .window = {} };
            }
            if (std.mem.eql(u8, member.property, "frames")) {
                break :blk Value{ .window = {} };
            }
            if (std.mem.eql(u8, member.property, "opener")) {
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "closed")) {
                break :blk Value{ .boolean = false };
            }
            if (std.mem.eql(u8, member.property, "length")) {
                break :blk Value{ .number = 0 };
            }
            if (std.mem.eql(u8, member.property, "name")) {
                break :blk Value{ .string = host.windowName() };
            }
            if (std.mem.eql(u8, member.property, "localStorage")) {
                break :blk try makeStorageValue(allocator, host, .local);
            }
            if (std.mem.eql(u8, member.property, "sessionStorage")) {
                break :blk try makeStorageValue(allocator, host, .session);
            }
            if (std.mem.eql(u8, member.property, "children")) {
                break :blk Value{ .html_collection = .{ .root = host.domStore().documentId() } };
            }
            if (std.mem.eql(u8, member.property, "navigator")) {
                break :blk Value{ .navigator = {} };
            }
            if (std.mem.eql(u8, member.property, "Math")) {
                break :blk try makeMathValue(allocator, host);
            }
            if (std.mem.eql(u8, member.property, "crypto")) {
                break :blk try makeCryptoValue(allocator, host);
            }
            if (std.mem.eql(u8, member.property, "screen")) {
                break :blk Value{ .screen = {} };
            }
            if (std.mem.eql(u8, member.property, "devicePixelRatio")) {
                break :blk Value{ .number = host.windowDevicePixelRatio() };
            }
            if (std.mem.eql(u8, member.property, "innerWidth")) {
                break :blk Value{ .number = @floatFromInt(host.windowInnerWidth()) };
            }
            if (std.mem.eql(u8, member.property, "innerHeight")) {
                break :blk Value{ .number = @floatFromInt(host.windowInnerHeight()) };
            }
            if (std.mem.eql(u8, member.property, "outerWidth")) {
                break :blk Value{ .number = @floatFromInt(host.windowOuterWidth()) };
            }
            if (std.mem.eql(u8, member.property, "outerHeight")) {
                break :blk Value{ .number = @floatFromInt(host.windowOuterHeight()) };
            }
            if (std.mem.eql(u8, member.property, "screenX") or
                std.mem.eql(u8, member.property, "screenLeft"))
            {
                break :blk Value{ .number = @floatFromInt(host.windowScreenX()) };
            }
            if (std.mem.eql(u8, member.property, "screenY") or
                std.mem.eql(u8, member.property, "screenTop"))
            {
                break :blk Value{ .number = @floatFromInt(host.windowScreenY()) };
            }
            if (std.mem.eql(u8, member.property, "scrollX")) {
                break :blk Value{ .number = @floatFromInt(host.windowScrollX()) };
            }
            if (std.mem.eql(u8, member.property, "scrollY")) {
                break :blk Value{ .number = @floatFromInt(host.windowScrollY()) };
            }
            if (std.mem.eql(u8, member.property, "pageXOffset")) {
                break :blk Value{ .number = @floatFromInt(host.windowPageXOffset()) };
            }
            if (std.mem.eql(u8, member.property, "pageYOffset")) {
                break :blk Value{ .number = @floatFromInt(host.windowPageYOffset()) };
            }
            if (std.mem.eql(u8, member.property, "title")) {
                break :blk Value{ .string = host.documentTitle() };
            }
            if (std.mem.eql(u8, member.property, "onhashchange")) {
                if (host.windowHashChange()) |handler| {
                    break :blk Value{ .function = handler };
                }
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "onload")) {
                if (host.windowLoad()) |handler| {
                    break :blk Value{ .function = handler };
                }
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "onfocus")) {
                if (host.windowFocus()) |handler| {
                    break :blk Value{ .function = handler };
                }
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "onblur")) {
                if (host.windowBlur()) |handler| {
                    break :blk Value{ .function = handler };
                }
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "onpopstate")) {
                if (host.windowPopState()) |handler| {
                    break :blk Value{ .function = handler };
                }
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "onstorage")) {
                if (host.windowStorage()) |handler| {
                    break :blk Value{ .function = handler };
                }
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "location")) {
                break :blk try makeLocationValue(allocator, host);
            }
            if (std.mem.eql(u8, member.property, "history")) {
                break :blk makeHistoryValue(host);
            }
            if (std.mem.eql(u8, member.property, "performance")) {
                break :blk try makePerformanceValue(allocator, host);
            }
            if (std.mem.eql(u8, member.property, "origin")) {
                break :blk Value{ .string = try originFromUrl(allocator, host.currentLocationUrl()) };
            }
            break :blk error.ScriptRuntime;
        },
        .location => |location| blk: {
            if (std.mem.eql(u8, member.property, "hash")) {
                break :blk Value{ .string = try location.hash(allocator) };
            }
            if (std.mem.eql(u8, member.property, "href")) {
                break :blk Value{ .string = location.current_url };
            }
            if (std.mem.eql(u8, member.property, "protocol")) {
                break :blk Value{ .string = try location.protocol(allocator) };
            }
            if (std.mem.eql(u8, member.property, "host")) {
                break :blk Value{ .string = try location.hostValue(allocator) };
            }
            if (std.mem.eql(u8, member.property, "hostname")) {
                break :blk Value{ .string = try location.hostname(allocator) };
            }
            if (std.mem.eql(u8, member.property, "port")) {
                break :blk Value{ .string = try location.port(allocator) };
            }
            if (std.mem.eql(u8, member.property, "username")) {
                break :blk Value{ .string = try location.username(allocator) };
            }
            if (std.mem.eql(u8, member.property, "password")) {
                break :blk Value{ .string = try location.password(allocator) };
            }
            if (std.mem.eql(u8, member.property, "pathname")) {
                break :blk Value{ .string = try location.pathname(allocator) };
            }
            if (std.mem.eql(u8, member.property, "search")) {
                break :blk Value{ .string = try location.search(allocator) };
            }
            if (std.mem.eql(u8, member.property, "ancestorOrigins")) {
                break :blk Value{ .string_list = location.ancestor_origins };
            }
            if (std.mem.eql(u8, member.property, "origin")) {
                break :blk Value{ .string = try originFromUrl(allocator, location.current_url) };
            }
            break :blk error.ScriptRuntime;
        },
        .navigator => blk: {
            if (std.mem.eql(u8, member.property, "userAgent")) {
                break :blk Value{ .string = host.windowNavigatorUserAgent() };
            }
            if (std.mem.eql(u8, member.property, "appCodeName")) {
                break :blk Value{ .string = host.windowNavigatorAppCodeName() };
            }
            if (std.mem.eql(u8, member.property, "appName")) {
                break :blk Value{ .string = host.windowNavigatorAppName() };
            }
            if (std.mem.eql(u8, member.property, "appVersion")) {
                break :blk Value{ .string = host.windowNavigatorAppVersion() };
            }
            if (std.mem.eql(u8, member.property, "product")) {
                break :blk Value{ .string = host.windowNavigatorProduct() };
            }
            if (std.mem.eql(u8, member.property, "productSub")) {
                break :blk Value{ .string = host.windowNavigatorProductSub() };
            }
            if (std.mem.eql(u8, member.property, "vendor")) {
                break :blk Value{ .string = host.windowNavigatorVendor() };
            }
            if (std.mem.eql(u8, member.property, "vendorSub")) {
                break :blk Value{ .string = host.windowNavigatorVendorSub() };
            }
            if (std.mem.eql(u8, member.property, "pdfViewerEnabled")) {
                break :blk Value{ .boolean = host.windowNavigatorPdfViewerEnabled() };
            }
            if (std.mem.eql(u8, member.property, "doNotTrack")) {
                break :blk Value{ .string = host.windowNavigatorDoNotTrack() };
            }
            if (std.mem.eql(u8, member.property, "userLanguage")) {
                break :blk Value{ .string = host.windowNavigatorUserLanguage() };
            }
            if (std.mem.eql(u8, member.property, "browserLanguage")) {
                break :blk Value{ .string = host.windowNavigatorBrowserLanguage() };
            }
            if (std.mem.eql(u8, member.property, "systemLanguage")) {
                break :blk Value{ .string = host.windowNavigatorSystemLanguage() };
            }
            if (std.mem.eql(u8, member.property, "oscpu")) {
                break :blk Value{ .string = host.windowNavigatorOscpu() };
            }
            if (std.mem.eql(u8, member.property, "plugins")) {
                break :blk Value{ .html_collection = .{ .root = host.domStore().documentId(), .kind = .navigator_plugins } };
            }
            if (std.mem.eql(u8, member.property, "mimeTypes")) {
                break :blk Value{ .mime_type_array = {} };
            }
            if (std.mem.eql(u8, member.property, "languages")) {
                break :blk Value{ .string_list = .{ .items = host.windowNavigatorLanguages() } };
            }
            if (std.mem.eql(u8, member.property, "platform")) {
                break :blk Value{ .string = host.windowNavigatorPlatform() };
            }
            if (std.mem.eql(u8, member.property, "language")) {
                break :blk Value{ .string = host.windowNavigatorLanguage() };
            }
            if (std.mem.eql(u8, member.property, "cookieEnabled")) {
                break :blk Value{ .boolean = host.windowNavigatorCookieEnabled() };
            }
            if (std.mem.eql(u8, member.property, "onLine")) {
                break :blk Value{ .boolean = host.windowNavigatorOnLine() };
            }
            if (std.mem.eql(u8, member.property, "webdriver")) {
                break :blk Value{ .boolean = host.windowNavigatorWebdriver() };
            }
            if (std.mem.eql(u8, member.property, "hardwareConcurrency")) {
                break :blk Value{ .number = @floatFromInt(host.windowNavigatorHardwareConcurrency()) };
            }
            if (std.mem.eql(u8, member.property, "maxTouchPoints")) {
                break :blk Value{ .number = @floatFromInt(host.windowNavigatorMaxTouchPoints()) };
            }
            break :blk error.ScriptRuntime;
        },
        .string_list => blk: {
            if (std.mem.eql(u8, member.property, "length")) {
                break :blk Value{ .number = @floatFromInt(object.string_list.length()) };
            }
            break :blk error.ScriptRuntime;
        },
        .mime_type_array => blk: {
            if (std.mem.eql(u8, member.property, "length")) {
                break :blk Value{ .number = 0 };
            }
            break :blk error.ScriptRuntime;
        },
        .performance => blk: {
            if (std.mem.eql(u8, member.property, "timeOrigin")) {
                break :blk Value{ .number = object.performance.timeOrigin() };
            }
            break :blk error.ScriptRuntime;
        },
        .screen => blk: {
            if (std.mem.eql(u8, member.property, "width")) {
                break :blk Value{ .number = @floatFromInt(host.windowScreenWidth()) };
            }
            if (std.mem.eql(u8, member.property, "height")) {
                break :blk Value{ .number = @floatFromInt(host.windowScreenHeight()) };
            }
            if (std.mem.eql(u8, member.property, "availWidth")) {
                break :blk Value{ .number = @floatFromInt(host.windowScreenAvailWidth()) };
            }
            if (std.mem.eql(u8, member.property, "availHeight")) {
                break :blk Value{ .number = @floatFromInt(host.windowScreenAvailHeight()) };
            }
            if (std.mem.eql(u8, member.property, "availLeft")) {
                break :blk Value{ .number = @floatFromInt(host.windowScreenAvailLeft()) };
            }
            if (std.mem.eql(u8, member.property, "availTop")) {
                break :blk Value{ .number = @floatFromInt(host.windowScreenAvailTop()) };
            }
            if (std.mem.eql(u8, member.property, "left")) {
                break :blk Value{ .number = @floatFromInt(host.windowScreenLeft()) };
            }
            if (std.mem.eql(u8, member.property, "top")) {
                break :blk Value{ .number = @floatFromInt(host.windowScreenTop()) };
            }
            if (std.mem.eql(u8, member.property, "colorDepth")) {
                break :blk Value{ .number = @floatFromInt(host.windowScreenColorDepth()) };
            }
            if (std.mem.eql(u8, member.property, "pixelDepth")) {
                break :blk Value{ .number = @floatFromInt(host.windowScreenPixelDepth()) };
            }
            if (std.mem.eql(u8, member.property, "orientation")) {
                break :blk Value{ .screen_orientation = {} };
            }
            break :blk error.ScriptRuntime;
        },
        .screen_orientation => blk: {
            if (std.mem.eql(u8, member.property, "type")) {
                break :blk Value{ .string = host.windowScreenOrientationType() };
            }
            if (std.mem.eql(u8, member.property, "angle")) {
                break :blk Value{ .number = @floatFromInt(host.windowScreenOrientationAngle()) };
            }
            break :blk error.ScriptRuntime;
        },
        .math => blk: {
            if (std.mem.eql(u8, member.property, "E")) {
                break :blk Value{ .number = 2.718281828459045 };
            }
            if (std.mem.eql(u8, member.property, "LN10")) {
                break :blk Value{ .number = 2.302585092994046 };
            }
            if (std.mem.eql(u8, member.property, "LN2")) {
                break :blk Value{ .number = 0.6931471805599453 };
            }
            if (std.mem.eql(u8, member.property, "LOG10E")) {
                break :blk Value{ .number = 0.4342944819032518 };
            }
            if (std.mem.eql(u8, member.property, "LOG2E")) {
                break :blk Value{ .number = 1.4426950408889634 };
            }
            if (std.mem.eql(u8, member.property, "PI")) {
                break :blk Value{ .number = 3.141592653589793 };
            }
            if (std.mem.eql(u8, member.property, "SQRT1_2")) {
                break :blk Value{ .number = 0.7071067811865476 };
            }
            if (std.mem.eql(u8, member.property, "SQRT2")) {
                break :blk Value{ .number = 1.4142135623730951 };
            }
            break :blk error.ScriptRuntime;
        },
        .crypto => return error.ScriptRuntime,
        .history => |history| blk: {
            if (std.mem.eql(u8, member.property, "length")) {
                break :blk Value{ .number = @floatFromInt(history.length()) };
            }
            if (std.mem.eql(u8, member.property, "state")) {
                if (history.state()) |state| {
                    break :blk Value{ .string = state };
                }
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "scrollRestoration")) {
                break :blk Value{ .string = history.scrollRestoration() };
            }
            break :blk error.ScriptRuntime;
        },
        .storage => |storage| blk: {
            if (std.mem.eql(u8, member.property, "length")) {
                break :blk Value{ .number = @floatFromInt(storage.length()) };
            }
            break :blk error.ScriptRuntime;
        },
        .element => |element| blk: {
            if (std.mem.eql(u8, member.property, "content")) {
                const tag_name = host.domStore().tagNameForNode(element) orelse break :blk error.ScriptRuntime;
                if (std.mem.eql(u8, tag_name, "template")) {
                    break :blk Value{ .template_content = element };
                }
                break :blk error.ScriptRuntime;
            }
            if (std.mem.eql(u8, member.property, "innerHTML")) {
                const html = try host.domStore().innerHtml(allocator, element);
                break :blk Value{ .string = html };
            }
            if (std.mem.eql(u8, member.property, "outerHTML")) {
                const html = try host.domStore().outerHtml(allocator, element);
                break :blk Value{ .string = html };
            }
            if (std.mem.eql(u8, member.property, "className")) {
                const class_name = (try host.domStore().getAttribute(element, "class")) orelse "";
                break :blk Value{ .string = class_name };
            }
            if (std.mem.eql(u8, member.property, "id")) {
                const id = (try host.domStore().getAttribute(element, "id")) orelse "";
                break :blk Value{ .string = id };
            }
            if (std.mem.eql(u8, member.property, "title")) {
                const title = (try host.domStore().getAttribute(element, "title")) orelse "";
                break :blk Value{ .string = title };
            }
            if (std.mem.eql(u8, member.property, "lang")) {
                const lang = (try host.domStore().getAttribute(element, "lang")) orelse "";
                break :blk Value{ .string = lang };
            }
            if (std.mem.eql(u8, member.property, "dir")) {
                const dir = (try host.domStore().getAttribute(element, "dir")) orelse "";
                break :blk Value{ .string = dir };
            }
            if (std.mem.eql(u8, member.property, "hidden")) {
                const hidden = try host.domStore().hasAttribute(element, "hidden");
                break :blk Value{ .boolean = hidden };
            }
            if (std.mem.eql(u8, member.property, "inert")) {
                const inert = try host.domStore().hasAttribute(element, "inert");
                break :blk Value{ .boolean = inert };
            }
            if (std.mem.eql(u8, member.property, "disabled")) {
                if (!try elementSupportsDisabledProperty(host, element)) break :blk error.ScriptRuntime;
                const disabled = try host.domStore().hasAttribute(element, "disabled");
                break :blk Value{ .boolean = disabled };
            }
            if (std.mem.eql(u8, member.property, "required")) {
                if (!try elementSupportsRequiredProperty(host, element)) break :blk error.ScriptRuntime;
                const required = try host.domStore().hasAttribute(element, "required");
                break :blk Value{ .boolean = required };
            }
            if (std.mem.eql(u8, member.property, "translate")) {
                const translate = try elementTranslateValue(host, element);
                break :blk Value{ .boolean = translate };
            }
            if (std.mem.eql(u8, member.property, "spellcheck")) {
                const spellcheck = try elementSpellcheckValue(host, element);
                break :blk Value{ .boolean = spellcheck };
            }
            if (std.mem.eql(u8, member.property, "draggable")) {
                const draggable = try host.domStore().hasAttribute(element, "draggable");
                break :blk Value{ .boolean = draggable };
            }
            if (std.mem.eql(u8, member.property, "nonce")) {
                const nonce = (try host.domStore().getAttribute(element, "nonce")) orelse "";
                break :blk Value{ .string = nonce };
            }
            if (std.mem.eql(u8, member.property, "autocapitalize")) {
                const autocapitalize = (try host.domStore().getAttribute(element, "autocapitalize")) orelse "";
                break :blk Value{ .string = autocapitalize };
            }
            if (std.mem.eql(u8, member.property, "autofocus")) {
                const autofocus = try host.domStore().hasAttribute(element, "autofocus");
                break :blk Value{ .boolean = autofocus };
            }
            if (std.mem.eql(u8, member.property, "autocomplete")) {
                const autocomplete = (try host.domStore().getAttribute(element, "autocomplete")) orelse "";
                break :blk Value{ .string = autocomplete };
            }
            if (std.mem.eql(u8, member.property, "name")) {
                const name = (try host.domStore().getAttribute(element, "name")) orelse "";
                break :blk Value{ .string = name };
            }
            if (std.mem.eql(u8, member.property, "placeholder")) {
                if (!try elementSupportsPlaceholderProperty(host, element)) break :blk error.ScriptRuntime;
                const placeholder = (try host.domStore().getAttribute(element, "placeholder")) orelse "";
                break :blk Value{ .string = placeholder };
            }
            if (std.mem.eql(u8, member.property, "inputMode")) {
                const input_mode = (try host.domStore().getAttribute(element, "inputmode")) orelse "";
                break :blk Value{ .string = input_mode };
            }
            if (std.mem.eql(u8, member.property, "readOnly")) {
                const readonly = try host.domStore().hasAttribute(element, "readonly");
                break :blk Value{ .boolean = readonly };
            }
            if (std.mem.eql(u8, member.property, "accessKey")) {
                const access_key = (try host.domStore().getAttribute(element, "accesskey")) orelse "";
                break :blk Value{ .string = access_key };
            }
            if (std.mem.eql(u8, member.property, "contentEditable")) {
                const content_editable = try elementContentEditableText(allocator, host, element);
                break :blk Value{ .string = content_editable };
            }
            if (std.mem.eql(u8, member.property, "isContentEditable")) {
                const content_editable = try elementIsContentEditable(host, element);
                break :blk Value{ .boolean = content_editable };
            }
            if (std.mem.eql(u8, member.property, "tabIndex")) {
                const tab_index = try elementTabIndexValue(allocator, host, element);
                break :blk Value{ .number = @floatFromInt(tab_index) };
            }
            if (std.mem.eql(u8, member.property, "style")) {
                break :blk try makeStyleDeclarationValue(allocator, host, element);
            }
            if (std.mem.eql(u8, member.property, "classList")) {
                break :blk Value{ .class_list = element };
            }
            if (std.mem.eql(u8, member.property, "dataset")) {
                break :blk Value{ .dataset = element };
            }
            if (std.mem.eql(u8, member.property, "textContent")) {
                const text = try host.domStore().textContent(allocator, element);
                break :blk Value{ .string = text };
            }
            if (std.mem.eql(u8, member.property, "value")) {
                const value = try host.domStore().valueForNode(allocator, element);
                break :blk Value{ .string = value };
            }
            if (std.mem.eql(u8, member.property, "selected")) {
                const tag_name = host.domStore().tagNameForNode(element) orelse break :blk error.ScriptRuntime;
                if (!std.mem.eql(u8, tag_name, "option")) break :blk error.ScriptRuntime;
                break :blk Value{ .boolean = host.domStore().optionSelectedForNode(element) orelse false };
            }
            if (std.mem.eql(u8, member.property, "selectedIndex")) {
                const tag_name = host.domStore().tagNameForNode(element) orelse break :blk error.ScriptRuntime;
                if (!std.mem.eql(u8, tag_name, "select")) break :blk error.ScriptRuntime;
                const selected_index = try host.domStore().selectedIndexForNode(element);
                break :blk Value{ .number = @floatFromInt(selected_index) };
            }
            if (std.mem.eql(u8, member.property, "selectionStart")) {
                const selection = host.domStore().selectionStateForNode(element) catch |err| switch (err) {
                    error.OutOfMemory => return error.OutOfMemory,
                    else => return error.ScriptRuntime,
                };
                if (selection) |state| {
                    break :blk Value{ .number = @floatFromInt(state.start) };
                }
                const tag_name = host.domStore().tagNameForNode(element) orelse break :blk error.ScriptRuntime;
                if (std.mem.eql(u8, tag_name, "input")) {
                    break :blk Value{ .null_value = {} };
                }
                break :blk error.ScriptRuntime;
            }
            if (std.mem.eql(u8, member.property, "selectionEnd")) {
                const selection = host.domStore().selectionStateForNode(element) catch |err| switch (err) {
                    error.OutOfMemory => return error.OutOfMemory,
                    else => return error.ScriptRuntime,
                };
                if (selection) |state| {
                    break :blk Value{ .number = @floatFromInt(state.end) };
                }
                const tag_name = host.domStore().tagNameForNode(element) orelse break :blk error.ScriptRuntime;
                if (std.mem.eql(u8, tag_name, "input")) {
                    break :blk Value{ .null_value = {} };
                }
                break :blk error.ScriptRuntime;
            }
            if (std.mem.eql(u8, member.property, "selectionDirection")) {
                const selection = host.domStore().selectionStateForNode(element) catch |err| switch (err) {
                    error.OutOfMemory => return error.OutOfMemory,
                    else => return error.ScriptRuntime,
                };
                if (selection) |state| {
                    break :blk Value{ .string = selectionDirectionName(state.direction) };
                }
                const tag_name = host.domStore().tagNameForNode(element) orelse break :blk error.ScriptRuntime;
                if (std.mem.eql(u8, tag_name, "input")) {
                    break :blk Value{ .null_value = {} };
                }
                break :blk error.ScriptRuntime;
            }
            if (std.mem.eql(u8, member.property, "options")) {
                const tag_name = host.domStore().tagNameForNode(element) orelse break :blk error.ScriptRuntime;
                if (std.mem.eql(u8, tag_name, "select")) {
                    break :blk Value{ .html_collection = .{ .root = element, .kind = .select_options } };
                }
                if (std.mem.eql(u8, tag_name, "datalist")) {
                    break :blk Value{ .html_collection = .{ .root = element, .kind = .datalist_options } };
                }
                break :blk error.ScriptRuntime;
            }
            if (std.mem.eql(u8, member.property, "selectedOptions")) {
                const tag_name = host.domStore().tagNameForNode(element) orelse break :blk error.ScriptRuntime;
                if (std.mem.eql(u8, tag_name, "select")) {
                    break :blk Value{ .html_collection = .{ .root = element, .kind = .selected_options } };
                }
                break :blk error.ScriptRuntime;
            }
            if (std.mem.eql(u8, member.property, "elements")) {
                const tag_name = host.domStore().tagNameForNode(element) orelse break :blk error.ScriptRuntime;
                if (std.mem.eql(u8, tag_name, "form")) {
                    break :blk Value{ .html_collection = .{ .root = element, .kind = .form_elements } };
                }
                if (std.mem.eql(u8, tag_name, "fieldset")) {
                    break :blk Value{ .html_collection = .{ .root = element, .kind = .fieldset_elements } };
                }
                break :blk error.ScriptRuntime;
            }
            if (std.mem.eql(u8, member.property, "areas")) {
                const tag_name = host.domStore().tagNameForNode(element) orelse break :blk error.ScriptRuntime;
                if (std.mem.eql(u8, tag_name, "map")) {
                    break :blk Value{ .html_collection = .{ .root = element, .kind = .map_areas } };
                }
                break :blk error.ScriptRuntime;
            }
            if (std.mem.eql(u8, member.property, "tBodies")) {
                const tag_name = host.domStore().tagNameForNode(element) orelse break :blk error.ScriptRuntime;
                if (std.mem.eql(u8, tag_name, "table")) {
                    break :blk Value{ .html_collection = .{ .root = element, .kind = .table_t_bodies } };
                }
                break :blk error.ScriptRuntime;
            }
            if (std.mem.eql(u8, member.property, "rows")) {
                const tag_name = host.domStore().tagNameForNode(element) orelse break :blk error.ScriptRuntime;
                if (std.mem.eql(u8, tag_name, "table") or
                    std.mem.eql(u8, tag_name, "thead") or
                    std.mem.eql(u8, tag_name, "tbody") or
                    std.mem.eql(u8, tag_name, "tfoot"))
                {
                    break :blk Value{ .html_collection = .{ .root = element, .kind = .table_rows } };
                }
                break :blk error.ScriptRuntime;
            }
            if (std.mem.eql(u8, member.property, "checked")) {
                break :blk Value{ .boolean = host.domStore().checkedForNode(element) orelse false };
            }
            if (std.mem.eql(u8, member.property, "baseURI")) {
                break :blk Value{ .string = host.currentLocationUrl() };
            }
            if (std.mem.eql(u8, member.property, "origin")) {
                break :blk Value{ .string = try originFromUrl(allocator, host.currentLocationUrl()) };
            }
            if (std.mem.eql(u8, member.property, "cells")) {
                const tag_name = host.domStore().tagNameForNode(element) orelse break :blk error.ScriptRuntime;
                if (std.mem.eql(u8, tag_name, "tr")) {
                    break :blk Value{ .html_collection = .{ .root = element, .kind = .row_cells } };
                }
                break :blk error.ScriptRuntime;
            }
            if (std.mem.eql(u8, member.property, "childNodes")) {
                break :blk Value{
                    .node_list = .{
                        .items = &.{},
                        .item_kind = .node,
                        .live_root = element,
                        .live_kind = .children,
                    },
                };
            }
            if (std.mem.eql(u8, member.property, "labels")) {
                const labelable = try isLabelableElement(host, element);
                if (!labelable) break :blk error.ScriptRuntime;
                break :blk Value{
                    .node_list = .{
                        .items = &.{},
                        .item_kind = .element,
                        .live_root = element,
                        .live_kind = .labels,
                    },
                };
            }
            if (std.mem.eql(u8, member.property, "children")) {
                break :blk Value{ .html_collection = .{ .root = element } };
            }
            if (std.mem.eql(u8, member.property, "length")) {
                const tag_name = host.domStore().tagNameForNode(element) orelse break :blk error.ScriptRuntime;
                if (std.mem.eql(u8, tag_name, "form")) {
                    const items = try formElementsItems(allocator, host, element);
                    defer allocator.free(items);
                    break :blk Value{ .number = @floatFromInt(items.len) };
                }
                if (std.mem.eql(u8, tag_name, "select")) {
                    const items = try selectOptionsItems(allocator, host, element);
                    defer allocator.free(items);
                    break :blk Value{ .number = @floatFromInt(items.len) };
                }
                break :blk error.ScriptRuntime;
            }
            if (try nodeLikeMember(allocator, host, element, member.property)) |value| {
                break :blk value;
            }
            break :blk error.ScriptRuntime;
        },
        .template_content => |element| blk: {
            if (std.mem.eql(u8, member.property, "innerHTML")) {
                const html = try host.domStore().innerHtml(allocator, element);
                break :blk Value{ .string = html };
            }
            if (std.mem.eql(u8, member.property, "textContent")) {
                const text = try host.domStore().textContent(allocator, element);
                break :blk Value{ .string = text };
            }
            if (std.mem.eql(u8, member.property, "nodeValue")) {
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "isConnected")) {
                break :blk Value{ .boolean = false };
            }
            if (std.mem.eql(u8, member.property, "ownerDocument")) {
                break :blk Value{ .document = {} };
            }
            if (std.mem.eql(u8, member.property, "parentNode") or
                std.mem.eql(u8, member.property, "parentElement") or
                std.mem.eql(u8, member.property, "nextSibling") or
                std.mem.eql(u8, member.property, "previousSibling") or
                std.mem.eql(u8, member.property, "nextElementSibling") or
                std.mem.eql(u8, member.property, "previousElementSibling"))
            {
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "nodeName")) {
                break :blk Value{ .string = "#document-fragment" };
            }
            if (std.mem.eql(u8, member.property, "nodeType")) {
                break :blk Value{ .number = 11 };
            }
            if (std.mem.eql(u8, member.property, "firstChild")) {
                const child_id = dom.firstChild(host.domStore(), element) orelse return Value{ .null_value = {} };
                break :blk try nodeValueForNodeId(allocator, host, child_id);
            }
            if (std.mem.eql(u8, member.property, "lastChild")) {
                const child_id = dom.lastChild(host.domStore(), element) orelse return Value{ .null_value = {} };
                break :blk try nodeValueForNodeId(allocator, host, child_id);
            }
            if (std.mem.eql(u8, member.property, "childNodes")) {
                break :blk Value{
                    .node_list = .{
                        .items = &.{},
                        .item_kind = .node,
                        .live_root = element,
                        .live_kind = .children,
                    },
                };
            }
            if (std.mem.eql(u8, member.property, "children")) {
                break :blk Value{ .html_collection = .{ .root = element } };
            }
            break :blk error.ScriptRuntime;
        },
        .class_list => blk: {
            if (std.mem.eql(u8, member.property, "length")) {
                var tokens = try classListTokens(allocator, host, object.class_list);
                defer tokens.deinit(allocator);
                break :blk Value{ .number = @floatFromInt(tokens.items.len) };
            }
            break :blk error.ScriptRuntime;
        },
        .style_declaration => blk: {
            if (std.mem.eql(u8, member.property, "cssText")) {
                break :blk Value{ .string = try styleDeclarationCssText(allocator, object.style_declaration) };
            }
            if (std.mem.eql(u8, member.property, "length")) {
                break :blk Value{ .number = @floatFromInt(try styleDeclarationLength(allocator, object.style_declaration)) };
            }
            if (std.mem.eql(u8, member.property, "item") or
                std.mem.eql(u8, member.property, "getPropertyValue") or
                std.mem.eql(u8, member.property, "getPropertyPriority") or
                std.mem.eql(u8, member.property, "setProperty") or
                std.mem.eql(u8, member.property, "removeProperty") or
                std.mem.eql(u8, member.property, "toString"))
            {
                break :blk error.ScriptRuntime;
            }

            const property_name = try stylePropertyName(allocator, member.property);
            defer allocator.free(property_name);
            break :blk Value{ .string = try styleDeclarationGetPropertyValue(allocator, object.style_declaration, property_name) };
        },
        .node => |node_id| blk: {
            if (try nodeLikeMember(allocator, host, node_id, member.property)) |value| {
                break :blk value;
            }
            break :blk error.ScriptRuntime;
        },
        .document_scripts => blk: {
            if (std.mem.eql(u8, member.property, "length")) {
                const scripts = try documentScriptsItems(allocator, host);
                defer allocator.free(scripts);
                break :blk Value{ .number = @floatFromInt(scripts.len) };
            }
            break :blk error.ScriptRuntime;
        },
        .document_anchors => blk: {
            if (std.mem.eql(u8, member.property, "length")) {
                const anchors = try documentAnchorsItems(allocator, host);
                defer allocator.free(anchors);
                break :blk Value{ .number = @floatFromInt(anchors.len) };
            }
            break :blk error.ScriptRuntime;
        },
        .document_style_sheets => blk: {
            if (std.mem.eql(u8, member.property, "length")) {
                const sheets = try documentStyleSheetsItems(allocator, host);
                defer allocator.free(sheets);
                break :blk Value{ .number = @floatFromInt(sheets.len) };
            }
            break :blk error.ScriptRuntime;
        },
        .css_rule_list => |list| blk: {
            if (std.mem.eql(u8, member.property, "length")) {
                const rules = try cssRuleListCurrentValues(allocator, host, list);
                defer allocator.free(rules);
                break :blk Value{ .number = @floatFromInt(rules.len) };
            }
            break :blk error.ScriptRuntime;
        },
        .style_sheet => |sheet_id| blk: {
            if (std.mem.eql(u8, member.property, "cssRules")) {
                break :blk Value{ .css_rule_list = .{ .sheet = sheet_id } };
            }
            if (std.mem.eql(u8, member.property, "ownerRule")) {
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "ownerNode")) {
                if (host.domStore().tagNameForNode(sheet_id) != null) {
                    break :blk Value{ .element = sheet_id };
                }
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "href")) {
                if (try host.domStore().getAttribute(sheet_id, "href")) |href| {
                    break :blk Value{ .string = href };
                }
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "title")) {
                if (try host.domStore().getAttribute(sheet_id, "title")) |title| {
                    break :blk Value{ .string = title };
                }
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "disabled")) {
                const disabled = try host.domStore().hasAttribute(sheet_id, "disabled");
                break :blk Value{ .boolean = disabled };
            }
            if (std.mem.eql(u8, member.property, "media")) {
                const media_text = (try host.domStore().getAttribute(sheet_id, "media")) orelse "";
                const Host = @TypeOf(host);
                break :blk Value{ .media_list = .{
                    .host = @ptrCast(host),
                    .node_id = sheet_id,
                    .media_text = media_text,
                    .current_media_text_fn = struct {
                        fn call(ptr: *anyopaque, node_id: dom.NodeId) errors.Result([]const u8) {
                            const typed: Host = @ptrCast(@alignCast(ptr));
                            const value = try typed.domStore().getAttribute(node_id, "media");
                            return value orelse "";
                        }
                    }.call,
                    .set_media_text_fn = struct {
                        fn call(ptr: *anyopaque, node_id: dom.NodeId, value: []const u8) errors.Result(void) {
                            const typed: Host = @ptrCast(@alignCast(ptr));
                            try typed.domStoreMut().setAttribute(node_id, "media", value);
                            return;
                        }
                    }.call,
                } };
            }
            break :blk error.ScriptRuntime;
        },
        .css_rule => |rule| blk: {
            if (std.mem.eql(u8, member.property, "type")) {
                break :blk Value{ .number = @floatFromInt(cssRuleType(rule)) };
            }
            if (std.mem.eql(u8, member.property, "parentStyleSheet")) {
                if (cssRuleParentStyleSheet(rule)) |sheet_id| {
                    break :blk Value{ .style_sheet = sheet_id };
                }
                break :blk Value{ .null_value = {} };
            }
            if (std.mem.eql(u8, member.property, "parentRule")) {
                if (cssRuleParentRule(rule)) |parent_rule| {
                    break :blk Value{ .css_rule = parent_rule.* };
                }
                break :blk Value{ .null_value = {} };
            }
            switch (rule) {
                .style => |style| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = style.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "selectorText")) {
                        break :blk Value{ .string = style.selector_text };
                    }
                    if (std.mem.eql(u8, member.property, "style")) {
                        break :blk try makeCssRuleStyleDeclarationValue(
                            allocator,
                            try cssRuleStyleDeclarationText(style.css_text),
                        );
                    }
                    break :blk error.ScriptRuntime;
                },
                .media => |media| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = media.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "conditionText")) {
                        break :blk Value{ .string = media.condition_text };
                    }
                    if (std.mem.eql(u8, member.property, "media")) {
                        break :blk Value{ .media_list = .{ .media_text = media.condition_text } };
                    }
                    if (std.mem.eql(u8, member.property, "cssRules")) {
                        break :blk Value{ .css_rule_list = .{ .items = media.css_rules } };
                    }
                    break :blk error.ScriptRuntime;
                },
                .supports => |supports| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = supports.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "conditionText")) {
                        break :blk Value{ .string = supports.condition_text };
                    }
                    if (std.mem.eql(u8, member.property, "cssRules")) {
                        break :blk Value{ .css_rule_list = .{ .items = supports.css_rules } };
                    }
                    break :blk error.ScriptRuntime;
                },
                .document => |document| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = document.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "conditionText")) {
                        break :blk Value{ .string = document.condition_text };
                    }
                    if (std.mem.eql(u8, member.property, "cssRules")) {
                        break :blk Value{ .css_rule_list = .{ .items = document.css_rules } };
                    }
                    break :blk error.ScriptRuntime;
                },
                .container => |container| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = container.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "conditionText")) {
                        break :blk Value{ .string = container.condition_text };
                    }
                    if (std.mem.eql(u8, member.property, "cssRules")) {
                        break :blk Value{ .css_rule_list = .{ .items = container.css_rules } };
                    }
                    break :blk error.ScriptRuntime;
                },
                .starting_style => |starting_style| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = starting_style.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "cssRules")) {
                        break :blk Value{ .css_rule_list = .{ .items = starting_style.css_rules } };
                    }
                    break :blk error.ScriptRuntime;
                },
                .keyframes => |keyframes| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = keyframes.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "name")) {
                        break :blk Value{ .string = keyframes.name };
                    }
                    if (std.mem.eql(u8, member.property, "cssRules")) {
                        break :blk Value{ .css_rule_list = .{ .items = keyframes.css_rules } };
                    }
                    break :blk error.ScriptRuntime;
                },
                .keyframe => |keyframe| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = keyframe.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "keyText")) {
                        break :blk Value{ .string = keyframe.key_text };
                    }
                    break :blk error.ScriptRuntime;
                },
                .font_face => |font_face| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = font_face.css_text };
                    }
                    break :blk error.ScriptRuntime;
                },
                .font_feature_values => |font_feature_values| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = font_feature_values.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "fontFamily")) {
                        break :blk Value{ .string = font_feature_values.font_family };
                    }
                    break :blk error.ScriptRuntime;
                },
                .font_palette_values => |font_palette_values| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = font_palette_values.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "name")) {
                        break :blk Value{ .string = font_palette_values.name };
                    }
                    break :blk error.ScriptRuntime;
                },
                .color_profile => |color_profile| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = color_profile.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "name")) {
                        break :blk Value{ .string = color_profile.name };
                    }
                    if (std.mem.eql(u8, member.property, "src")) {
                        break :blk Value{ .string = color_profile.src };
                    }
                    if (std.mem.eql(u8, member.property, "renderingIntent")) {
                        break :blk Value{ .string = color_profile.rendering_intent };
                    }
                    if (std.mem.eql(u8, member.property, "components")) {
                        break :blk Value{ .string = color_profile.components };
                    }
                    break :blk error.ScriptRuntime;
                },
                .scope => |scope| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = scope.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "start")) {
                        break :blk if (scope.start_text) |text| Value{ .string = text } else Value{ .null_value = {} };
                    }
                    if (std.mem.eql(u8, member.property, "end")) {
                        break :blk if (scope.end_text) |text| Value{ .string = text } else Value{ .null_value = {} };
                    }
                    if (std.mem.eql(u8, member.property, "cssRules")) {
                        break :blk Value{ .css_rule_list = .{ .items = scope.css_rules } };
                    }
                    break :blk error.ScriptRuntime;
                },
                .page => |page| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = page.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "selectorText")) {
                        break :blk Value{ .string = page.selector_text };
                    }
                    break :blk error.ScriptRuntime;
                },
                .position_try => |position_try| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = position_try.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "name")) {
                        break :blk Value{ .string = position_try.name };
                    }
                    break :blk error.ScriptRuntime;
                },
                .layer => |layer| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = layer.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "nameText")) {
                        break :blk Value{ .string = layer.name_text };
                    }
                    if (std.mem.eql(u8, member.property, "cssRules")) {
                        if (layer.is_statement) return error.ScriptRuntime;
                        break :blk Value{ .css_rule_list = .{ .items = layer.css_rules } };
                    }
                    break :blk error.ScriptRuntime;
                },
                .counter_style => |counter_style| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = counter_style.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "name")) {
                        break :blk Value{ .string = counter_style.name };
                    }
                    break :blk error.ScriptRuntime;
                },
                .property => |property_rule| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = property_rule.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "name")) {
                        break :blk Value{ .string = property_rule.name };
                    }
                    if (std.mem.eql(u8, member.property, "syntax")) {
                        break :blk Value{ .string = property_rule.syntax };
                    }
                    if (std.mem.eql(u8, member.property, "inherits")) {
                        break :blk Value{ .boolean = property_rule.inherits };
                    }
                    if (std.mem.eql(u8, member.property, "initialValue")) {
                        break :blk Value{ .string = property_rule.initial_value };
                    }
                    break :blk error.ScriptRuntime;
                },
                .charset => |charset_rule| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = charset_rule.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "encoding")) {
                        break :blk Value{ .string = charset_rule.encoding };
                    }
                    break :blk error.ScriptRuntime;
                },
                .import => |import_rule| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = import_rule.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "href")) {
                        break :blk Value{ .string = import_rule.href };
                    }
                    if (std.mem.eql(u8, member.property, "mediaText")) {
                        break :blk Value{ .string = import_rule.media_text };
                    }
                    if (std.mem.eql(u8, member.property, "supportsText")) {
                        break :blk Value{ .string = import_rule.supports_text };
                    }
                    if (std.mem.eql(u8, member.property, "layerName")) {
                        break :blk Value{ .string = import_rule.layer_name };
                    }
                    if (std.mem.eql(u8, member.property, "styleSheet")) {
                        break :blk Value{ .null_value = {} };
                    }
                    if (std.mem.eql(u8, member.property, "media")) {
                        break :blk Value{ .media_list = .{ .media_text = import_rule.media_text } };
                    }
                    break :blk error.ScriptRuntime;
                },
                .namespace => |namespace_rule| {
                    if (std.mem.eql(u8, member.property, "cssText")) {
                        break :blk Value{ .string = namespace_rule.css_text };
                    }
                    if (std.mem.eql(u8, member.property, "prefix")) {
                        break :blk Value{ .string = namespace_rule.prefix };
                    }
                    if (std.mem.eql(u8, member.property, "namespaceURI")) {
                        break :blk Value{ .string = namespace_rule.namespace_uri };
                    }
                    break :blk error.ScriptRuntime;
                },
            }
        },
        .html_collection => |collection| blk: {
            if (std.mem.eql(u8, member.property, "length")) {
                const items = try htmlCollectionCurrentIds(allocator, host, collection);
                defer allocator.free(items);
                break :blk Value{ .number = @floatFromInt(items.len) };
            }
            break :blk error.ScriptRuntime;
        },
        .radio_node_list => |list| blk: {
            if (std.mem.eql(u8, member.property, "length")) {
                const items = try radioNodeListCurrentIds(allocator, host, list);
                defer allocator.free(items);
                break :blk Value{ .number = @floatFromInt(items.len) };
            }
            if (std.mem.eql(u8, member.property, "value")) {
                const value = try radioNodeListValue(allocator, host, list);
                break :blk Value{ .string = value };
            }
            break :blk error.ScriptRuntime;
        },
        .media_query_list => |mql| blk: {
            if (std.mem.eql(u8, member.property, "matches")) {
                break :blk Value{ .boolean = mql.currentMatches() };
            }
            if (std.mem.eql(u8, member.property, "media")) {
                break :blk Value{ .string = mql.media };
            }
            if (std.mem.eql(u8, member.property, "onchange")) {
                if (mql.get_onchange_fn) |get_onchange_fn| {
                    const host_ptr = mql.host orelse return error.ScriptRuntime;
                    if (get_onchange_fn(host_ptr, mql.media)) |handler| {
                        break :blk Value{ .function = handler };
                    }
                }
                break :blk Value{ .null_value = {} };
            }
            break :blk error.ScriptRuntime;
        },
        .media_list => |media| blk: {
            if (std.mem.eql(u8, member.property, "mediaText")) {
                break :blk Value{ .string = try media.currentText() };
            }
            if (std.mem.eql(u8, member.property, "length")) {
                const media_text = try media.currentText();
                break :blk Value{ .number = @floatFromInt(mediaListLength(media_text)) };
            }
            break :blk error.ScriptRuntime;
        },
        .dataset => blk: {
            const attribute_name = try datasetAttributeName(allocator, member.property);
            const value = host.domStore().getAttribute(object.dataset, attribute_name) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            if (value) |text| {
                break :blk Value{ .string = text };
            }
            break :blk Value{ .undefined_value = {} };
        },
        .node_list => |list| blk: {
            if (std.mem.eql(u8, member.property, "length")) {
                const items = try nodeListCurrentIds(allocator, host, list);
                defer allocator.free(items);
                break :blk Value{ .number = @floatFromInt(items.len) };
            }
            break :blk error.ScriptRuntime;
        },
        .collection_iterator => error.ScriptRuntime,
        .iterator_result => |result| blk: {
            if (std.mem.eql(u8, member.property, "value")) {
                break :blk result.value;
            }
            if (std.mem.eql(u8, member.property, "done")) {
                break :blk Value{ .boolean = result.done };
            }
            break :blk error.ScriptRuntime;
        },
        .collection_entry => |entry| blk: {
            if (std.mem.eql(u8, member.property, "index")) {
                break :blk Value{ .number = @floatFromInt(entry.index) };
            }
            if (std.mem.eql(u8, member.property, "value")) {
                break :blk entry.value;
            }
            break :blk error.ScriptRuntime;
        },
        .event => |event| blk: {
            if (std.mem.eql(u8, member.property, "type")) {
                break :blk Value{ .string = event.eventType() };
            }
            if (std.mem.eql(u8, member.property, "target")) {
                break :blk valueForListenerTarget(event.target);
            }
            if (std.mem.eql(u8, member.property, "currentTarget")) {
                break :blk if (event.current_target) |target| valueForListenerTarget(target) else Value{ .undefined_value = {} };
            }
            if (std.mem.eql(u8, member.property, "defaultPrevented")) {
                break :blk Value{ .boolean = event.default_prevented };
            }
            if (std.mem.eql(u8, member.property, "cancelable")) {
                break :blk Value{ .boolean = event.cancelable };
            }
            if (std.mem.eql(u8, member.property, "bubbles")) {
                break :blk Value{ .boolean = event.bubbles };
            }
            if (std.mem.eql(u8, member.property, "eventPhase")) {
                break :blk Value{ .number = @floatFromInt(@intFromEnum(event.phase)) };
            }
            break :blk error.ScriptRuntime;
        },
        .null_value, .undefined_value => error.ScriptRuntime,
        .boolean, .number, .string, .function => error.ScriptRuntime,
    };
}

fn evalCall(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    call: CallExpr,
) errors.Result(Value) {
    switch (call.callee.*) {
        .identifier => |name| {
            if (std.mem.eql(u8, name, "String")) {
                const value = if (call.args.len > 0)
                    try evalExpr(allocator, host, bindings, call.args[0])
                else
                    Value{ .undefined_value = {} };
                return Value{ .string = try asString(allocator, value) };
            }

            if (std.mem.eql(u8, name, "setTimeout")) {
                if (call.args.len == 0 or call.args.len > 2) return error.ScriptRuntime;
                const callback_value = try evalExpr(allocator, host, bindings, call.args[0]);
                const callback = switch (callback_value) {
                    .function => |function| function,
                    else => return error.ScriptRuntime,
                };
                const delay_ms = if (call.args.len >= 2)
                    try timerDelayFromExpr(allocator, host, bindings, call.args[1])
                else
                    0;
                const timer_id = try host.scheduleTimer(callback, delay_ms);
                return Value{ .number = @floatFromInt(timer_id) };
            }

            if (std.mem.eql(u8, name, "setInterval")) {
                if (call.args.len == 0 or call.args.len > 2) return error.ScriptRuntime;
                const callback_value = try evalExpr(allocator, host, bindings, call.args[0]);
                const callback = switch (callback_value) {
                    .function => |function| function,
                    else => return error.ScriptRuntime,
                };
                const delay_ms = if (call.args.len >= 2)
                    try timerDelayFromExpr(allocator, host, bindings, call.args[1])
                else
                    0;
                const timer_id = try host.scheduleIntervalTimer(callback, delay_ms);
                return Value{ .number = @floatFromInt(timer_id) };
            }

            if (std.mem.eql(u8, name, "requestAnimationFrame")) {
                if (call.args.len != 1) return error.ScriptRuntime;
                const callback_value = try evalExpr(allocator, host, bindings, call.args[0]);
                const callback = switch (callback_value) {
                    .function => |function| function,
                    else => return error.ScriptRuntime,
                };
                const timer_id = try host.scheduleAnimationFrame(callback);
                return Value{ .number = @floatFromInt(timer_id) };
            }

            if (std.mem.eql(u8, name, "clearTimeout")) {
                if (call.args.len > 1) return error.ScriptRuntime;
                if (call.args.len == 1) {
                    if (try timerIdFromExpr(allocator, host, bindings, call.args[0])) |timer_id| {
                        host.clearTimer(timer_id);
                    }
                }
                return Value{ .undefined_value = {} };
            }

            if (std.mem.eql(u8, name, "clearInterval")) {
                if (call.args.len > 1) return error.ScriptRuntime;
                if (call.args.len == 1) {
                    if (try timerIdFromExpr(allocator, host, bindings, call.args[0])) |timer_id| {
                        host.clearTimer(timer_id);
                    }
                }
                return Value{ .undefined_value = {} };
            }

            if (std.mem.eql(u8, name, "cancelAnimationFrame")) {
                if (call.args.len > 1) return error.ScriptRuntime;
                if (call.args.len == 1) {
                    if (try timerIdFromExpr(allocator, host, bindings, call.args[0])) |timer_id| {
                        host.clearTimer(timer_id);
                    }
                }
                return Value{ .undefined_value = {} };
            }

            if (std.mem.eql(u8, name, "queueMicrotask")) {
                if (call.args.len != 1) return error.ScriptRuntime;
                const callback_value = try evalExpr(allocator, host, bindings, call.args[0]);
                const callback = switch (callback_value) {
                    .function => |function| function,
                    else => return error.ScriptRuntime,
                };

                try host.queueMicrotask(callback);
                return Value{ .undefined_value = {} };
            }

            return error.ScriptRuntime;
        },
        .member => |member| {
            const object = try evalExpr(allocator, host, bindings, member.object);
            return try evalMethodCall(allocator, host, bindings, object, member.property, call.args);
        },
        .arrow_function => return error.ScriptRuntime,
        else => return error.ScriptRuntime,
    }
}

fn evalMethodCall(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    object: Value,
    method: []const u8,
    args: []*Expr,
) errors.Result(Value) {
    return switch (object) {
        .document => if (std.mem.eql(u8, method, "compareDocumentPosition")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const position = try nodeCompareDocumentPositionValue(allocator, host, bindings, object, args[0]);
            break :blk Value{ .number = @floatFromInt(position) };
        } else if (std.mem.eql(u8, method, "isSameNode")) blk: {
            break :blk try nodeSameNodeValue(allocator, host, bindings, object, args);
        } else if (std.mem.eql(u8, method, "isEqualNode")) blk: {
            break :blk try nodeEqualNodeValue(allocator, host, bindings, object, args);
        } else if (std.mem.eql(u8, method, "contains")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const contains = try nodeContainsValue(allocator, host, bindings, false, host.domStore().documentId(), args[0]);
            break :blk Value{ .boolean = contains };
        } else if (std.mem.eql(u8, method, "getElementById")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const id_value = try evalExpr(allocator, host, bindings, args[0]);
            const id = try asString(allocator, id_value);
            if (host.domStore().findElementById(id)) |element_id| {
                break :blk Value{ .element = element_id };
            }
            break :blk Value{ .null_value = {} };
        } else if (std.mem.eql(u8, method, "createElement")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const tag_value = try evalExpr(allocator, host, bindings, args[0]);
            const tag_name = try asString(allocator, tag_value);
            const element = host.domStoreMut().createElementDetached(tag_name) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .element = element };
        } else if (std.mem.eql(u8, method, "removeChild")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            break :blk try nodeRemoveChildValue(allocator, host, bindings, host.domStore().documentId(), args[0]);
        } else if (std.mem.eql(u8, method, "createTextNode")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const text_value = try evalExpr(allocator, host, bindings, args[0]);
            const text = try asString(allocator, text_value);
            const node_id = host.domStoreMut().createTextNode(text) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .node = node_id };
        } else if (std.mem.eql(u8, method, "createComment")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const text_value = try evalExpr(allocator, host, bindings, args[0]);
            const text = try asString(allocator, text_value);
            const node_id = host.domStoreMut().createComment(text) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .node = node_id };
        } else if (std.mem.eql(u8, method, "createDocumentFragment")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            const fragment = host.domStoreMut().createElementDetached("template") catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .template_content = fragment };
        } else if (std.mem.eql(u8, method, "normalize")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try normalizeNodeValue(allocator, host, host.domStore().documentId());
        } else if (std.mem.eql(u8, method, "importNode")) blk: {
            if (args.len != 1 and args.len != 2) return error.ScriptRuntime;
            const source_value = try evalExpr(allocator, host, bindings, args[0]);
            const deep = if (args.len == 2) deep_blk: {
                const deep_value = try evalExpr(allocator, host, bindings, args[1]);
                break :deep_blk isTruthy(deep_value);
            } else false;
            switch (source_value) {
                .element => |element| break :blk try cloneNodeValue(allocator, host, element, deep, false),
                .node => |node_id| break :blk try cloneNodeValue(allocator, host, node_id, deep, false),
                .template_content => |element| break :blk try cloneNodeValue(allocator, host, element, deep, true),
                .document => return error.ScriptRuntime,
                else => return error.ScriptRuntime,
            }
        } else if (std.mem.eql(u8, method, "hasFocus")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .boolean = host.documentHasFocus() };
        } else if (std.mem.eql(u8, method, "hasChildNodes")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .boolean = dom.hasChildNodes(host.domStore(), host.domStore().documentId()) };
        } else if (std.mem.eql(u8, method, "getElementsByTagName")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const tag_value = try evalExpr(allocator, host, bindings, args[0]);
            const tag_name = try asString(allocator, tag_value);
            break :blk Value{ .html_collection = .{
                .root = host.domStore().documentId(),
                .kind = .tag_name,
                .query = tag_name,
            } };
        } else if (std.mem.eql(u8, method, "getElementsByTagNameNS")) blk: {
            if (args.len != 2) return error.ScriptRuntime;
            const namespace_value = try evalExpr(allocator, host, bindings, args[0]);
            const namespace_uri = try asString(allocator, namespace_value);
            const local_name_value = try evalExpr(allocator, host, bindings, args[1]);
            const local_name = try asString(allocator, local_name_value);
            break :blk Value{ .html_collection = .{
                .root = host.domStore().documentId(),
                .kind = .tag_name_ns,
                .namespace_uri = namespace_uri,
                .local_name = local_name,
            } };
        } else if (std.mem.eql(u8, method, "getElementsByClassName")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const class_value = try evalExpr(allocator, host, bindings, args[0]);
            const class_names = try asString(allocator, class_value);
            break :blk Value{ .html_collection = .{
                .root = host.domStore().documentId(),
                .kind = .class_name,
                .query = class_names,
            } };
        } else if (std.mem.eql(u8, method, "getElementsByName")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const name_value = try evalExpr(allocator, host, bindings, args[0]);
            const name = try asString(allocator, name_value);
            break :blk Value{
                .node_list = .{
                    .items = &.{},
                    .item_kind = .element,
                    .live_root = host.domStore().documentId(),
                    .live_kind = .named_elements,
                    .live_name = name,
                },
            };
        } else if (std.mem.eql(u8, method, "querySelector")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const selector_value = try evalExpr(allocator, host, bindings, args[0]);
            const selector = try asString(allocator, selector_value);
            const match = host.domStore().querySelector(allocator, selector) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            if (match) |element_id| {
                break :blk Value{ .element = element_id };
            }
            break :blk Value{ .null_value = {} };
        } else if (std.mem.eql(u8, method, "querySelectorAll")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const selector_value = try evalExpr(allocator, host, bindings, args[0]);
            const selector = try asString(allocator, selector_value);
            const matches = host.domStore().select(allocator, selector) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .node_list = .{ .items = matches } };
        } else if (std.mem.eql(u8, method, "addEventListener")) blk: {
            break :blk try registerListener(allocator, host, bindings, .document, args);
        } else error.ScriptRuntime,
        .window => if (std.mem.eql(u8, method, "document")) Value{ .document = {} } else if (std.mem.eql(u8, method, "addEventListener")) blk: {
            break :blk try registerListener(allocator, host, bindings, .window, args);
        } else if (std.mem.eql(u8, method, "queueMicrotask")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const callback_value = try evalExpr(allocator, host, bindings, args[0]);
            const callback = switch (callback_value) {
                .function => |function| function,
                else => return error.ScriptRuntime,
            };
            try host.queueMicrotask(callback);
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "setTimeout")) blk: {
            if (args.len == 0 or args.len > 2) return error.ScriptRuntime;
            const callback_value = try evalExpr(allocator, host, bindings, args[0]);
            const callback = switch (callback_value) {
                .function => |function| function,
                else => return error.ScriptRuntime,
            };
            const delay_ms = if (args.len >= 2)
                try timerDelayFromExpr(allocator, host, bindings, args[1])
            else
                0;
            const timer_id = try host.scheduleTimer(callback, delay_ms);
            break :blk Value{ .number = @floatFromInt(timer_id) };
        } else if (std.mem.eql(u8, method, "setInterval")) blk: {
            if (args.len == 0 or args.len > 2) return error.ScriptRuntime;
            const callback_value = try evalExpr(allocator, host, bindings, args[0]);
            const callback = switch (callback_value) {
                .function => |function| function,
                else => return error.ScriptRuntime,
            };
            const delay_ms = if (args.len >= 2)
                try timerDelayFromExpr(allocator, host, bindings, args[1])
            else
                0;
            const timer_id = try host.scheduleIntervalTimer(callback, delay_ms);
            break :blk Value{ .number = @floatFromInt(timer_id) };
        } else if (std.mem.eql(u8, method, "requestAnimationFrame")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const callback_value = try evalExpr(allocator, host, bindings, args[0]);
            const callback = switch (callback_value) {
                .function => |function| function,
                else => return error.ScriptRuntime,
            };
            const timer_id = try host.scheduleAnimationFrame(callback);
            break :blk Value{ .number = @floatFromInt(timer_id) };
        } else if (std.mem.eql(u8, method, "clearTimeout")) blk: {
            if (args.len > 1) return error.ScriptRuntime;
            if (args.len == 1) {
                if (try timerIdFromExpr(allocator, host, bindings, args[0])) |timer_id| {
                    host.clearTimer(timer_id);
                }
            }
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "clearInterval")) blk: {
            if (args.len > 1) return error.ScriptRuntime;
            if (args.len == 1) {
                if (try timerIdFromExpr(allocator, host, bindings, args[0])) |timer_id| {
                    host.clearTimer(timer_id);
                }
            }
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "cancelAnimationFrame")) blk: {
            if (args.len > 1) return error.ScriptRuntime;
            if (args.len == 1) {
                if (try timerIdFromExpr(allocator, host, bindings, args[0])) |timer_id| {
                    host.clearTimer(timer_id);
                }
            }
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "open")) blk: {
            if (args.len > 3) return error.ScriptRuntime;
            const url = if (args.len >= 1)
                try asString(allocator, try evalExpr(allocator, host, bindings, args[0]))
            else
                null;
            const target = if (args.len >= 2)
                try asString(allocator, try evalExpr(allocator, host, bindings, args[1]))
            else
                null;
            const features = if (args.len >= 3)
                try asString(allocator, try evalExpr(allocator, host, bindings, args[2]))
            else
                null;
            try host.open(url, target, features);
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "close")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            try host.close();
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "print")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            try host.print();
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "scrollTo")) blk: {
            if (args.len > 2) return error.ScriptRuntime;
            const x = if (args.len >= 1)
                try scrollCoordinate(try evalExpr(allocator, host, bindings, args[0]), "scrollTo")
            else
                0;
            const y = if (args.len >= 2)
                try scrollCoordinate(try evalExpr(allocator, host, bindings, args[1]), "scrollTo")
            else
                0;
            try host.scrollTo(x, y);
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "scrollBy")) blk: {
            if (args.len > 2) return error.ScriptRuntime;
            const x = if (args.len >= 1)
                try scrollCoordinate(try evalExpr(allocator, host, bindings, args[0]), "scrollBy")
            else
                0;
            const y = if (args.len >= 2)
                try scrollCoordinate(try evalExpr(allocator, host, bindings, args[1]), "scrollBy")
            else
                0;
            try host.scrollBy(x, y);
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "matchMedia")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const query_value = try evalExpr(allocator, host, bindings, args[0]);
            const query = try asString(allocator, query_value);
            const matches = try host.matchMedia(query);
            const Host = @TypeOf(host);
            break :blk Value{ .media_query_list = .{
                .host = @ptrCast(host),
                .media = query,
                .matches = matches,
                .current_matches_fn = struct {
                    fn call(ptr: *anyopaque, query_source: []const u8) bool {
                        const typed: Host = @ptrCast(@alignCast(ptr));
                        return typed.matchMediaCurrent(query_source);
                    }
                }.call,
                .get_onchange_fn = struct {
                    fn call(ptr: *anyopaque, query_source: []const u8) ?ScriptFunction {
                        const typed: Host = @ptrCast(@alignCast(ptr));
                        return typed.matchMediaOnChange(query_source);
                    }
                }.call,
                .set_onchange_fn = struct {
                    fn call(ptr: *anyopaque, query_source: []const u8, handler: ?ScriptFunction) errors.Result(void) {
                        const typed: Host = @ptrCast(@alignCast(ptr));
                        return typed.setMatchMediaOnChange(query_source, handler);
                    }
                }.call,
                .add_listener_fn = struct {
                    fn call(ptr: *anyopaque, query_source: []const u8, handler: ScriptFunction) errors.Result(void) {
                        const typed: Host = @ptrCast(@alignCast(ptr));
                        return typed.registerMatchMediaListener(query_source, handler);
                    }
                }.call,
                .remove_listener_fn = struct {
                    fn call(ptr: *anyopaque, query_source: []const u8, handler: ScriptFunction) errors.Result(void) {
                        const typed: Host = @ptrCast(@alignCast(ptr));
                        return typed.unregisterMatchMediaListener(query_source, handler);
                    }
                }.call,
            } };
        } else error.ScriptRuntime,
        .crypto => if (std.mem.eql(u8, method, "randomUUID")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .string = try object.crypto.randomUUID(allocator) };
        } else if (std.mem.eql(u8, method, "toString")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .string = "[object Crypto]" };
        } else error.ScriptRuntime,
        .navigator => if (std.mem.eql(u8, method, "javaEnabled")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .boolean = host.windowNavigatorJavaEnabled() };
        } else if (std.mem.eql(u8, method, "toString")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .string = "[object Navigator]" };
        } else error.ScriptRuntime,
        .mime_type_array => if (std.mem.eql(u8, method, "item")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            _ = try evalExpr(allocator, host, bindings, args[0]);
            break :blk Value{ .null_value = {} };
        } else if (std.mem.eql(u8, method, "namedItem")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            _ = try evalExpr(allocator, host, bindings, args[0]);
            break :blk Value{ .null_value = {} };
        } else if (std.mem.eql(u8, method, "toString")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .string = "[object MimeTypeArray]" };
        } else error.ScriptRuntime,
        .string_list => if (std.mem.eql(u8, method, "item")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const index_value = try evalExpr(allocator, host, bindings, args[0]);
            const index = try asNodeListIndex(index_value);
            if (index >= object.string_list.items.len) {
                break :blk Value{ .null_value = {} };
            }
            break :blk Value{ .string = object.string_list.items[index] };
        } else if (std.mem.eql(u8, method, "contains")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const value = try evalExpr(allocator, host, bindings, args[0]);
            const text = try asString(allocator, value);
            break :blk Value{ .boolean = object.string_list.contains(text) };
        } else if (std.mem.eql(u8, method, "toString")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .string = "[object DOMStringList]" };
        } else error.ScriptRuntime,
        .math => if (std.mem.eql(u8, method, "random")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .number = object.math.random() };
        } else if (std.mem.eql(u8, method, "toString")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .string = "[object Math]" };
        } else error.ScriptRuntime,
        .performance => if (std.mem.eql(u8, method, "now")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .number = object.performance.now() };
        } else if (std.mem.eql(u8, method, "toString")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .string = "[object Performance]" };
        } else error.ScriptRuntime,
        .screen => if (std.mem.eql(u8, method, "toString")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .string = "[object Screen]" };
        } else error.ScriptRuntime,
        .screen_orientation => if (std.mem.eql(u8, method, "toString")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .string = "[object ScreenOrientation]" };
        } else error.ScriptRuntime,
        .location => if (std.mem.eql(u8, method, "assign")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const url_value = try evalExpr(allocator, host, bindings, args[0]);
            const url = try asString(allocator, url_value);
            try object.location.assign(url);
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "replace")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const url_value = try evalExpr(allocator, host, bindings, args[0]);
            const url = try asString(allocator, url_value);
            try object.location.replace(url);
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "reload")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            try object.location.reload();
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "valueOf")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .string = object.location.current_url };
        } else if (std.mem.eql(u8, method, "toString")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .string = object.location.current_url };
        } else error.ScriptRuntime,
        .history => if (std.mem.eql(u8, method, "back")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            try object.history.back();
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "forward")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            try object.history.forward();
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "go")) blk: {
            if (args.len > 1) return error.ScriptRuntime;
            const delta = if (args.len == 0) @as(isize, 0) else try historyDeltaFromExpr(allocator, host, bindings, args[0]);
            try object.history.go(delta);
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "pushState")) blk: {
            if (args.len < 2 or args.len > 3) return error.ScriptRuntime;
            const state_value = try evalExpr(allocator, host, bindings, args[0]);
            const state = try historyStateFromValue(allocator, state_value);
            _ = try evalExpr(allocator, host, bindings, args[1]);
            const url = if (args.len == 3) url_blk: {
                const url_value = try evalExpr(allocator, host, bindings, args[2]);
                break :url_blk try asString(allocator, url_value);
            } else host.currentLocationUrl();
            try object.history.pushState(state, url);
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "replaceState")) blk: {
            if (args.len < 2 or args.len > 3) return error.ScriptRuntime;
            const state_value = try evalExpr(allocator, host, bindings, args[0]);
            const state = try historyStateFromValue(allocator, state_value);
            _ = try evalExpr(allocator, host, bindings, args[1]);
            const url = if (args.len == 3) url_blk: {
                const url_value = try evalExpr(allocator, host, bindings, args[2]);
                break :url_blk try asString(allocator, url_value);
            } else host.currentLocationUrl();
            try object.history.replaceState(state, url);
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "toString")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .string = "[object History]" };
        } else error.ScriptRuntime,
        .storage => if (std.mem.eql(u8, method, "getItem")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const key_value = try evalExpr(allocator, host, bindings, args[0]);
            const key = try asString(allocator, key_value);
            if (object.storage.getItem(key)) |text| {
                break :blk Value{ .string = text };
            }
            break :blk Value{ .null_value = {} };
        } else if (std.mem.eql(u8, method, "setItem")) blk: {
            if (args.len != 2) return error.ScriptRuntime;
            const key_value = try evalExpr(allocator, host, bindings, args[0]);
            const key = try asString(allocator, key_value);
            const value_value = try evalExpr(allocator, host, bindings, args[1]);
            const value = try asString(allocator, value_value);
            try object.storage.setItem(key, value);
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "removeItem")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const key_value = try evalExpr(allocator, host, bindings, args[0]);
            const key = try asString(allocator, key_value);
            try object.storage.removeItem(key);
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "clear")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            try object.storage.clear();
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "key")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const index_value = try evalExpr(allocator, host, bindings, args[0]);
            const index = try asNodeListIndex(index_value);
            if (object.storage.key(index)) |text| {
                break :blk Value{ .string = text };
            }
            break :blk Value{ .null_value = {} };
        } else if (std.mem.eql(u8, method, "toString")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .string = "[object Storage]" };
        } else error.ScriptRuntime,
        .style_declaration => if (std.mem.eql(u8, method, "getPropertyValue")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const name_value = try evalExpr(allocator, host, bindings, args[0]);
            const name = try asString(allocator, name_value);
            break :blk Value{ .string = try styleDeclarationGetPropertyValue(allocator, object.style_declaration, name) };
        } else if (std.mem.eql(u8, method, "getPropertyPriority")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const name_value = try evalExpr(allocator, host, bindings, args[0]);
            const name = try asString(allocator, name_value);
            break :blk Value{ .string = try styleDeclarationGetPropertyPriority(allocator, object.style_declaration, name) };
        } else if (std.mem.eql(u8, method, "setProperty")) blk: {
            if (args.len != 2 and args.len != 3) return error.ScriptRuntime;
            const name_value = try evalExpr(allocator, host, bindings, args[0]);
            const name = try asString(allocator, name_value);
            const value_value = try evalExpr(allocator, host, bindings, args[1]);
            const text = try asString(allocator, value_value);
            const priority = if (args.len == 3) priority_blk: {
                const priority_value = try evalExpr(allocator, host, bindings, args[2]);
                break :priority_blk switch (priority_value) {
                    .undefined_value, .null_value => null,
                    else => try asString(allocator, priority_value),
                };
            } else null;
            try styleDeclarationSetProperty(allocator, object.style_declaration, name, text, priority);
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "removeProperty")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const name_value = try evalExpr(allocator, host, bindings, args[0]);
            const name = try asString(allocator, name_value);
            break :blk Value{ .string = try styleDeclarationRemoveProperty(allocator, object.style_declaration, name) };
        } else if (std.mem.eql(u8, method, "item")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const index_value = try evalExpr(allocator, host, bindings, args[0]);
            const index = try asNodeListIndex(index_value);
            break :blk Value{ .string = try styleDeclarationItem(allocator, object.style_declaration, index) };
        } else if (std.mem.eql(u8, method, "toString")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .string = try styleDeclarationCssText(allocator, object.style_declaration) };
        } else error.ScriptRuntime,
        .element => |element| if (std.mem.eql(u8, method, "compareDocumentPosition")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const position = try nodeCompareDocumentPositionValue(allocator, host, bindings, object, args[0]);
            break :blk Value{ .number = @floatFromInt(position) };
        } else if (std.mem.eql(u8, method, "isSameNode")) blk: {
            break :blk try nodeSameNodeValue(allocator, host, bindings, object, args);
        } else if (std.mem.eql(u8, method, "isEqualNode")) blk: {
            break :blk try nodeEqualNodeValue(allocator, host, bindings, object, args);
        } else if (std.mem.eql(u8, method, "contains")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const contains = try nodeContainsValue(allocator, host, bindings, false, element, args[0]);
            break :blk Value{ .boolean = contains };
        } else if (std.mem.eql(u8, method, "textContent")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            const text = try host.domStore().textContent(allocator, element);
            break :blk Value{ .string = text };
        } else if (std.mem.eql(u8, method, "hasChildNodes")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .boolean = dom.hasChildNodes(host.domStore(), element) };
        } else if (std.mem.eql(u8, method, "appendChild")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            break :blk try nodeAppendChildValue(allocator, host, bindings, element, args[0]);
        } else if (std.mem.eql(u8, method, "insertBefore")) blk: {
            if (args.len != 2) return error.ScriptRuntime;
            break :blk try nodeInsertBeforeValue(allocator, host, bindings, element, args[0], args[1]);
        } else if (std.mem.eql(u8, method, "replaceChild")) blk: {
            if (args.len != 2) return error.ScriptRuntime;
            break :blk try nodeReplaceChildValue(allocator, host, bindings, element, args[0], args[1]);
        } else if (std.mem.eql(u8, method, "removeChild")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            break :blk try nodeRemoveChildValue(allocator, host, bindings, element, args[0]);
        } else if (std.mem.eql(u8, method, "getElementsByTagName")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const tag_value = try evalExpr(allocator, host, bindings, args[0]);
            const tag_name = try asString(allocator, tag_value);
            break :blk Value{ .html_collection = .{
                .root = element,
                .kind = .tag_name,
                .query = tag_name,
            } };
        } else if (std.mem.eql(u8, method, "getElementsByTagNameNS")) blk: {
            if (args.len != 2) return error.ScriptRuntime;
            const namespace_value = try evalExpr(allocator, host, bindings, args[0]);
            const namespace_uri = try asString(allocator, namespace_value);
            const local_name_value = try evalExpr(allocator, host, bindings, args[1]);
            const local_name = try asString(allocator, local_name_value);
            break :blk Value{ .html_collection = .{
                .root = element,
                .kind = .tag_name_ns,
                .namespace_uri = namespace_uri,
                .local_name = local_name,
            } };
        } else if (std.mem.eql(u8, method, "getElementsByClassName")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const class_value = try evalExpr(allocator, host, bindings, args[0]);
            const class_names = try asString(allocator, class_value);
            break :blk Value{ .html_collection = .{
                .root = element,
                .kind = .class_name,
                .query = class_names,
            } };
        } else if (std.mem.eql(u8, method, "getAttributeNS")) blk: {
            if (args.len != 2) return error.ScriptRuntime;
            _ = try evalExpr(allocator, host, bindings, args[0]);
            const local_name_value = try evalExpr(allocator, host, bindings, args[1]);
            const local_name = try asString(allocator, local_name_value);
            const value = host.domStore().getAttribute(element, local_name) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            if (value) |text| {
                break :blk Value{ .string = text };
            }
            break :blk Value{ .null_value = {} };
        } else if (std.mem.eql(u8, method, "getAttribute")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const name_value = try evalExpr(allocator, host, bindings, args[0]);
            const name = try asString(allocator, name_value);
            const value = host.domStore().getAttribute(element, name) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            if (value) |text| {
                break :blk Value{ .string = text };
            }
            break :blk Value{ .null_value = {} };
        } else if (std.mem.eql(u8, method, "setAttributeNS")) blk: {
            if (args.len != 3) return error.ScriptRuntime;
            _ = try evalExpr(allocator, host, bindings, args[0]);
            const local_name_value = try evalExpr(allocator, host, bindings, args[1]);
            const local_name = try asString(allocator, local_name_value);
            const value_value = try evalExpr(allocator, host, bindings, args[2]);
            const value = try asString(allocator, value_value);
            host.domStoreMut().setAttribute(element, local_name, value) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "setAttribute")) blk: {
            if (args.len != 2) return error.ScriptRuntime;
            const name_value = try evalExpr(allocator, host, bindings, args[0]);
            const name = try asString(allocator, name_value);
            const value_value = try evalExpr(allocator, host, bindings, args[1]);
            const value = try asString(allocator, value_value);
            host.domStoreMut().setAttribute(element, name, value) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "removeAttributeNS")) blk: {
            if (args.len != 2) return error.ScriptRuntime;
            _ = try evalExpr(allocator, host, bindings, args[0]);
            const local_name_value = try evalExpr(allocator, host, bindings, args[1]);
            const local_name = try asString(allocator, local_name_value);
            host.domStoreMut().removeAttribute(element, local_name) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "removeAttribute")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const name_value = try evalExpr(allocator, host, bindings, args[0]);
            const name = try asString(allocator, name_value);
            host.domStoreMut().removeAttribute(element, name) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "hasAttributeNS")) blk: {
            if (args.len != 2) return error.ScriptRuntime;
            _ = try evalExpr(allocator, host, bindings, args[0]);
            const local_name_value = try evalExpr(allocator, host, bindings, args[1]);
            const local_name = try asString(allocator, local_name_value);
            const present = host.domStore().hasAttribute(element, local_name) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .boolean = present };
        } else if (std.mem.eql(u8, method, "hasAttribute")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const name_value = try evalExpr(allocator, host, bindings, args[0]);
            const name = try asString(allocator, name_value);
            const present = host.domStore().hasAttribute(element, name) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .boolean = present };
        } else if (std.mem.eql(u8, method, "hasAttributes")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            const node = host.domStore().nodeAt(element) orelse break :blk error.ScriptRuntime;
            switch (node.kind) {
                .element => |element_node| break :blk Value{ .boolean = element_node.attributes.items.len > 0 },
                else => break :blk error.ScriptRuntime,
            }
        } else if (std.mem.eql(u8, method, "getAttributeNames")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            const node = host.domStore().nodeAt(element) orelse break :blk error.ScriptRuntime;
            switch (node.kind) {
                .element => |element_node| {
                    var names: std.ArrayList([]const u8) = .empty;
                    errdefer names.deinit(allocator);
                    for (element_node.attributes.items) |attribute| {
                        try names.append(allocator, attribute.name);
                    }
                    break :blk Value{ .string_list = .{ .items = try names.toOwnedSlice(allocator) } };
                },
                else => break :blk error.ScriptRuntime,
            }
        } else if (std.mem.eql(u8, method, "toggleAttribute")) blk: {
            if (args.len != 1 and args.len != 2) return error.ScriptRuntime;
            const name_value = try evalExpr(allocator, host, bindings, args[0]);
            const name = try asString(allocator, name_value);
            const force: ?bool = if (args.len == 2) force_blk: {
                const force_value = try evalExpr(allocator, host, bindings, args[1]);
                break :force_blk isTruthy(force_value);
            } else null;
            const present = host.domStoreMut().toggleAttribute(element, name, force) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .boolean = present };
        } else if (std.mem.eql(u8, method, "insertAdjacentHTML")) blk: {
            if (args.len != 2) return error.ScriptRuntime;
            const position_value = try evalExpr(allocator, host, bindings, args[0]);
            const position = try asString(allocator, position_value);
            const html_value = try evalExpr(allocator, host, bindings, args[1]);
            const html = try asString(allocator, html_value);
            host.domStoreMut().insertAdjacentHtml(element, position, html) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "appendChild")) blk: {
            break :blk try elementAppendChild(allocator, host, bindings, element, args);
        } else if (std.mem.eql(u8, method, "insertBefore")) blk: {
            break :blk try elementInsertBefore(allocator, host, bindings, element, args);
        } else if (std.mem.eql(u8, method, "replaceChild")) blk: {
            break :blk try elementReplaceChild(allocator, host, bindings, element, args);
        } else if (std.mem.eql(u8, method, "replaceChildren")) blk: {
            break :blk try elementReplaceChildren(allocator, host, bindings, element, args);
        } else if (std.mem.eql(u8, method, "append")) blk: {
            break :blk try elementAppend(allocator, host, bindings, element, args);
        } else if (std.mem.eql(u8, method, "prepend")) blk: {
            break :blk try elementPrepend(allocator, host, bindings, element, args);
        } else if (std.mem.eql(u8, method, "before")) blk: {
            break :blk try elementBefore(allocator, host, bindings, element, args);
        } else if (std.mem.eql(u8, method, "after")) blk: {
            break :blk try elementAfter(allocator, host, bindings, element, args);
        } else if (std.mem.eql(u8, method, "remove")) blk: {
            break :blk try elementRemove(allocator, host, bindings, element, args);
        } else if (std.mem.eql(u8, method, "cloneNode")) blk: {
            if (args.len > 1) return error.ScriptRuntime;
            const deep = if (args.len == 1) deep_blk: {
                const deep_value = try evalExpr(allocator, host, bindings, args[0]);
                break :deep_blk isTruthy(deep_value);
            } else false;
            break :blk try cloneNodeValue(allocator, host, element, deep, false);
        } else if (std.mem.eql(u8, method, "normalize")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try normalizeNodeValue(allocator, host, element);
        } else if (std.mem.eql(u8, method, "replaceWith")) blk: {
            break :blk try nodeReplaceWith(allocator, host, bindings, element, args);
        } else if (std.mem.eql(u8, method, "setSelectionRange")) blk: {
            if (args.len != 2 and args.len != 3) return error.ScriptRuntime;
            const start_value = try evalExpr(allocator, host, bindings, args[0]);
            const start = try selectionIndexFromValue(allocator, start_value);
            const end_value = try evalExpr(allocator, host, bindings, args[1]);
            const end = try selectionIndexFromValue(allocator, end_value);
            const direction = if (args.len == 3) direction_blk: {
                const direction_value = try evalExpr(allocator, host, bindings, args[2]);
                const direction_text = try asString(allocator, direction_value);
                break :direction_blk selectionDirectionFromString(direction_text) orelse return error.ScriptRuntime;
            } else .none;
            host.domStoreMut().setSelectionRange(element, start, end, direction) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "select")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            if (!host.domStore().isSelectionControlNode(element)) return error.ScriptRuntime;
            const value = try host.domStore().valueForNode(allocator, element);
            defer allocator.free(value);
            host.domStoreMut().setSelectionRange(element, 0, value.len, .none) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "querySelector")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const selector_value = try evalExpr(allocator, host, bindings, args[0]);
            const selector = try asString(allocator, selector_value);
            const match = host.domStore().querySelectorWithin(allocator, element, selector) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            if (match) |element_id| {
                break :blk Value{ .element = element_id };
            }
            break :blk Value{ .null_value = {} };
        } else if (std.mem.eql(u8, method, "querySelectorAll")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const selector_value = try evalExpr(allocator, host, bindings, args[0]);
            const selector = try asString(allocator, selector_value);
            const matches = host.domStore().selectWithin(allocator, element, selector) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .node_list = .{ .items = matches, .item_kind = .element } };
        } else if (std.mem.eql(u8, method, "matches")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const selector_value = try evalExpr(allocator, host, bindings, args[0]);
            const selector = try asString(allocator, selector_value);
            const matches = host.domStore().matchesSelector(allocator, element, selector) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .boolean = matches };
        } else if (std.mem.eql(u8, method, "closest")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const selector_value = try evalExpr(allocator, host, bindings, args[0]);
            const selector = try asString(allocator, selector_value);
            const match = host.domStore().closestSelector(allocator, element, selector) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            if (match) |element_id| {
                break :blk Value{ .element = element_id };
            }
            break :blk Value{ .null_value = {} };
        } else if (std.mem.eql(u8, method, "addEventListener")) blk: {
            break :blk try registerListener(allocator, host, bindings, .{ .element = element }, args);
        } else error.ScriptRuntime,
        .document_scripts => if (std.mem.eql(u8, method, "item")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const index_value = try evalExpr(allocator, host, bindings, args[0]);
            const index = try asNodeListIndex(index_value);
            const scripts = try documentScriptsItems(allocator, host);
            defer allocator.free(scripts);
            if (index >= scripts.len) {
                break :blk Value{ .null_value = {} };
            }
            break :blk Value{ .element = scripts[index] };
        } else if (std.mem.eql(u8, method, "keys")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try documentScriptsKeys(allocator, host);
        } else if (std.mem.eql(u8, method, "values")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try documentScriptsValues(allocator, host);
        } else if (std.mem.eql(u8, method, "entries")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try documentScriptsEntries(allocator, host);
        } else if (std.mem.eql(u8, method, "namedItem")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const name_value = try evalExpr(allocator, host, bindings, args[0]);
            const name = try asString(allocator, name_value);
            const match = try documentScriptsNamedItem(allocator, host, name);
            if (match) |element_id| {
                break :blk Value{ .element = element_id };
            }
            break :blk Value{ .null_value = {} };
        } else error.ScriptRuntime,
        .document_anchors => if (std.mem.eql(u8, method, "item")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const index_value = try evalExpr(allocator, host, bindings, args[0]);
            const index = try asNodeListIndex(index_value);
            const anchors = try documentAnchorsItems(allocator, host);
            defer allocator.free(anchors);
            if (index >= anchors.len) {
                break :blk Value{ .null_value = {} };
            }
            break :blk Value{ .element = anchors[index] };
        } else if (std.mem.eql(u8, method, "keys")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try documentAnchorsKeys(allocator, host);
        } else if (std.mem.eql(u8, method, "values")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try documentAnchorsValues(allocator, host);
        } else if (std.mem.eql(u8, method, "entries")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try documentAnchorsEntries(allocator, host);
        } else if (std.mem.eql(u8, method, "namedItem")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const name_value = try evalExpr(allocator, host, bindings, args[0]);
            const name = try asString(allocator, name_value);
            const match = try documentAnchorsNamedItem(allocator, host, name);
            if (match) |element_id| {
                break :blk Value{ .element = element_id };
            }
            break :blk Value{ .null_value = {} };
        } else error.ScriptRuntime,
        .document_style_sheets => if (std.mem.eql(u8, method, "item")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const index_value = try evalExpr(allocator, host, bindings, args[0]);
            const index = try asNodeListIndex(index_value);
            const sheets = try documentStyleSheetsItems(allocator, host);
            defer allocator.free(sheets);
            if (index >= sheets.len) {
                break :blk Value{ .null_value = {} };
            }
            break :blk Value{ .style_sheet = sheets[index] };
        } else if (std.mem.eql(u8, method, "keys")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try documentStyleSheetsKeys(allocator, host);
        } else if (std.mem.eql(u8, method, "values")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try documentStyleSheetsValues(allocator, host);
        } else if (std.mem.eql(u8, method, "entries")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try documentStyleSheetsEntries(allocator, host);
        } else if (std.mem.eql(u8, method, "forEach")) blk: {
            break :blk try documentStyleSheetsForEach(allocator, host, bindings, args);
        } else error.ScriptRuntime,
        .template_content => |element| if (std.mem.eql(u8, method, "compareDocumentPosition")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const position = try nodeCompareDocumentPositionValue(allocator, host, bindings, object, args[0]);
            break :blk Value{ .number = @floatFromInt(position) };
        } else if (std.mem.eql(u8, method, "isSameNode")) blk: {
            break :blk try nodeSameNodeValue(allocator, host, bindings, object, args);
        } else if (std.mem.eql(u8, method, "isEqualNode")) blk: {
            break :blk try nodeEqualNodeValue(allocator, host, bindings, object, args);
        } else if (std.mem.eql(u8, method, "contains")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const contains = try nodeContainsValue(allocator, host, bindings, true, element, args[0]);
            break :blk Value{ .boolean = contains };
        } else if (std.mem.eql(u8, method, "getElementById")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const id_value = try evalExpr(allocator, host, bindings, args[0]);
            const id = try asString(allocator, id_value);
            if (host.domStore().findElementByIdWithin(element, id)) |match| {
                break :blk Value{ .element = match };
            }
            break :blk Value{ .null_value = {} };
        } else if (std.mem.eql(u8, method, "appendChild")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            break :blk try nodeAppendChildValue(allocator, host, bindings, element, args[0]);
        } else if (std.mem.eql(u8, method, "insertBefore")) blk: {
            if (args.len != 2) return error.ScriptRuntime;
            break :blk try nodeInsertBeforeValue(allocator, host, bindings, element, args[0], args[1]);
        } else if (std.mem.eql(u8, method, "replaceChild")) blk: {
            if (args.len != 2) return error.ScriptRuntime;
            break :blk try nodeReplaceChildValue(allocator, host, bindings, element, args[0], args[1]);
        } else if (std.mem.eql(u8, method, "removeChild")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            break :blk try nodeRemoveChildValue(allocator, host, bindings, element, args[0]);
        } else if (std.mem.eql(u8, method, "hasChildNodes")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .boolean = dom.hasChildNodes(host.domStore(), element) };
        } else if (std.mem.eql(u8, method, "cloneNode")) blk: {
            if (args.len > 1) return error.ScriptRuntime;
            const deep = if (args.len == 1) deep_blk: {
                const deep_value = try evalExpr(allocator, host, bindings, args[0]);
                break :deep_blk isTruthy(deep_value);
            } else false;
            break :blk try cloneNodeValue(allocator, host, element, deep, true);
        } else if (std.mem.eql(u8, method, "normalize")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try normalizeNodeValue(allocator, host, element);
        } else if (std.mem.eql(u8, method, "querySelector")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const selector_value = try evalExpr(allocator, host, bindings, args[0]);
            const selector = try asString(allocator, selector_value);
            const match = host.domStore().querySelectorWithin(allocator, element, selector) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            if (match) |match_id| {
                break :blk Value{ .element = match_id };
            }
            break :blk Value{ .null_value = {} };
        } else if (std.mem.eql(u8, method, "querySelectorAll")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const selector_value = try evalExpr(allocator, host, bindings, args[0]);
            const selector = try asString(allocator, selector_value);
            const matches = host.domStore().selectWithin(allocator, element, selector) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .node_list = .{ .items = matches, .item_kind = .element } };
        } else error.ScriptRuntime,
        .class_list => |element| if (std.mem.eql(u8, method, "contains")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const token_value = try evalExpr(allocator, host, bindings, args[0]);
            const token = try validateClassListToken(allocator, token_value);
            var tokens = try classListTokens(allocator, host, element);
            defer tokens.deinit(allocator);
            break :blk Value{ .boolean = classListContains(tokens.items, token) };
        } else if (std.mem.eql(u8, method, "add")) blk: {
            if (args.len == 0) return error.ScriptRuntime;
            var tokens = try classListTokens(allocator, host, element);
            defer tokens.deinit(allocator);

            var changed = false;
            for (args) |arg| {
                const token_value = try evalExpr(allocator, host, bindings, arg);
                const token = try validateClassListToken(allocator, token_value);
                if (!classListContains(tokens.items, token)) {
                    try tokens.append(allocator, token);
                    changed = true;
                }
            }

            if (changed) {
                try writeClassListTokens(allocator, host, element, tokens.items);
            }
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "remove")) blk: {
            if (args.len == 0) return error.ScriptRuntime;
            var tokens = try classListTokens(allocator, host, element);
            defer tokens.deinit(allocator);

            const original_len = tokens.items.len;
            for (args) |arg| {
                const token_value = try evalExpr(allocator, host, bindings, arg);
                const token = try validateClassListToken(allocator, token_value);
                var index: usize = 0;
                while (index < tokens.items.len) : (index += 1) {
                    if (std.mem.eql(u8, tokens.items[index], token)) {
                        _ = tokens.orderedRemove(index);
                        break;
                    }
                }
            }

            if (tokens.items.len != original_len) {
                try writeClassListTokens(allocator, host, element, tokens.items);
            }
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "toggle")) blk: {
            if (args.len != 1 and args.len != 2) return error.ScriptRuntime;
            const token_value = try evalExpr(allocator, host, bindings, args[0]);
            const token = try validateClassListToken(allocator, token_value);
            const force: ?bool = if (args.len == 2) force_value_blk: {
                const force_value = try evalExpr(allocator, host, bindings, args[1]);
                break :force_value_blk isTruthy(force_value);
            } else null;

            var tokens = try classListTokens(allocator, host, element);
            defer tokens.deinit(allocator);
            const present = classListContains(tokens.items, token);
            const now_present = if (force) |forced| present_blk: {
                if (forced) {
                    if (!present) {
                        try tokens.append(allocator, token);
                        try writeClassListTokens(allocator, host, element, tokens.items);
                    }
                    break :present_blk true;
                }

                if (present) {
                    var index: usize = 0;
                    while (index < tokens.items.len) : (index += 1) {
                        if (std.mem.eql(u8, tokens.items[index], token)) {
                            _ = tokens.orderedRemove(index);
                            break;
                        }
                    }
                    try writeClassListTokens(allocator, host, element, tokens.items);
                }
                break :present_blk false;
            } else present_blk: {
                if (present) {
                    var index: usize = 0;
                    while (index < tokens.items.len) : (index += 1) {
                        if (std.mem.eql(u8, tokens.items[index], token)) {
                            _ = tokens.orderedRemove(index);
                            break;
                        }
                    }
                    try writeClassListTokens(allocator, host, element, tokens.items);
                    break :present_blk false;
                }

                try tokens.append(allocator, token);
                try writeClassListTokens(allocator, host, element, tokens.items);
                break :present_blk true;
            };

            break :blk Value{ .boolean = now_present };
        } else if (std.mem.eql(u8, method, "item")) blk: {
            break :blk error.ScriptRuntime;
        } else if (std.mem.eql(u8, method, "length")) blk: {
            break :blk error.ScriptRuntime;
        } else error.ScriptRuntime,
        .dataset => |_| error.ScriptRuntime,
        .style_sheet => |sheet_id| if (std.mem.eql(u8, method, "insertRule")) blk: {
            break :blk try styleSheetInsertRule(allocator, host, bindings, sheet_id, args);
        } else if (std.mem.eql(u8, method, "deleteRule")) blk: {
            break :blk try styleSheetDeleteRule(allocator, host, bindings, sheet_id, args);
        } else error.ScriptRuntime,
        .css_rule_list => |list| if (std.mem.eql(u8, method, "item")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const index_value = try evalExpr(allocator, host, bindings, args[0]);
            const index = try asNodeListIndex(index_value);
            const rules = try cssRuleListCurrentValues(allocator, host, list);
            defer allocator.free(rules);
            if (index >= rules.len) {
                break :blk Value{ .null_value = {} };
            }
            break :blk rules[index];
        } else if (std.mem.eql(u8, method, "keys")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try cssRuleListKeys(allocator, host, list);
        } else if (std.mem.eql(u8, method, "values")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try cssRuleListValues(allocator, host, list);
        } else if (std.mem.eql(u8, method, "entries")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try cssRuleListEntries(allocator, host, list);
        } else if (std.mem.eql(u8, method, "forEach")) blk: {
            break :blk try cssRuleListForEach(allocator, host, bindings, list, args);
        } else if (std.mem.eql(u8, method, "toString")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .string = "[object CSSRuleList]" };
        } else error.ScriptRuntime,
        .css_rule => |rule| if (std.mem.eql(u8, method, "toString")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .string = switch (rule) {
                .style => "[object CSSStyleRule]",
                .charset => "[object CSSCharsetRule]",
                .media => "[object CSSMediaRule]",
                .supports => "[object CSSSupportsRule]",
                .document => "[object CSSDocumentRule]",
                .container => "[object CSSContainerRule]",
                .starting_style => "[object CSSStartingStyleRule]",
                .keyframes => "[object CSSKeyframesRule]",
                .keyframe => "[object CSSKeyframeRule]",
                .font_face => "[object CSSFontFaceRule]",
                .font_feature_values => "[object CSSFontFeatureValuesRule]",
                .font_palette_values => "[object CSSFontPaletteValuesRule]",
                .color_profile => "[object CSSColorProfileRule]",
                .scope => "[object CSSScopeRule]",
                .page => "[object CSSPageRule]",
                .position_try => "[object CSSPositionTryRule]",
                .layer => |layer| if (layer.is_statement) "[object CSSLayerStatementRule]" else "[object CSSLayerBlockRule]",
                .counter_style => "[object CSSCounterStyleRule]",
                .property => "[object CSSPropertyRule]",
                .import => "[object CSSImportRule]",
                .namespace => "[object CSSNamespaceRule]",
            } };
        } else error.ScriptRuntime,
        .node_list => |list| if (std.mem.eql(u8, method, "item")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const index_value = try evalExpr(allocator, host, bindings, args[0]);
            const index = try asNodeListIndex(index_value);
            const items = try nodeListCurrentValues(allocator, host, list);
            defer allocator.free(items);
            if (index >= items.len) {
                break :blk Value{ .null_value = {} };
            }
            break :blk items[index];
        } else if (std.mem.eql(u8, method, "keys")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try nodeListKeys(allocator, host, list);
        } else if (std.mem.eql(u8, method, "values")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try nodeListValues(allocator, host, list);
        } else if (std.mem.eql(u8, method, "entries")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try nodeListEntries(allocator, host, list);
        } else if (std.mem.eql(u8, method, "forEach")) blk: {
            break :blk try nodeListForEach(allocator, host, bindings, list, args);
        } else error.ScriptRuntime,
        .html_collection => |collection| if (std.mem.eql(u8, method, "item")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const index_value = try evalExpr(allocator, host, bindings, args[0]);
            const index = try asNodeListIndex(index_value);
            const items = try htmlCollectionCurrentIds(allocator, host, collection);
            defer allocator.free(items);
            if (index >= items.len) {
                break :blk Value{ .null_value = {} };
            }
            break :blk Value{ .element = items[index] };
        } else if (std.mem.eql(u8, method, "namedItem")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const name_value = try evalExpr(allocator, host, bindings, args[0]);
            const name = try asString(allocator, name_value);
            break :blk try htmlCollectionNamedItem(allocator, host, collection, name);
        } else if (std.mem.eql(u8, method, "keys")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try htmlCollectionKeys(allocator, host, collection);
        } else if (std.mem.eql(u8, method, "values")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try htmlCollectionValues(allocator, host, collection);
        } else if (std.mem.eql(u8, method, "entries")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try htmlCollectionEntries(allocator, host, collection);
        } else if (std.mem.eql(u8, method, "add")) blk: {
            break :blk try htmlCollectionSelectOptionsAdd(allocator, host, bindings, collection, args);
        } else if (std.mem.eql(u8, method, "remove")) blk: {
            break :blk try htmlCollectionSelectOptionsRemove(allocator, host, bindings, collection, args);
        } else if (std.mem.eql(u8, method, "forEach")) blk: {
            break :blk try htmlCollectionForEach(allocator, host, bindings, collection, args);
        } else if (std.mem.eql(u8, method, "refresh") and collection.kind == .navigator_plugins) blk: {
            if (args.len > 1) return error.ScriptRuntime;
            if (args.len == 1) {
                _ = try evalExpr(allocator, host, bindings, args[0]);
            }
            break :blk Value{ .undefined_value = {} };
        } else error.ScriptRuntime,
        .radio_node_list => |list| if (std.mem.eql(u8, method, "item")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const index_value = try evalExpr(allocator, host, bindings, args[0]);
            const index = try asNodeListIndex(index_value);
            const items = try radioNodeListCurrentIds(allocator, host, list);
            defer allocator.free(items);
            if (index >= items.len) {
                break :blk Value{ .null_value = {} };
            }
            break :blk Value{ .element = items[index] };
        } else if (std.mem.eql(u8, method, "keys")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try radioNodeListKeys(allocator, host, list);
        } else if (std.mem.eql(u8, method, "values")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try radioNodeListValues(allocator, host, list);
        } else if (std.mem.eql(u8, method, "entries")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try radioNodeListEntries(allocator, host, list);
        } else if (std.mem.eql(u8, method, "forEach")) blk: {
            break :blk try radioNodeListForEach(allocator, host, bindings, list, args);
        } else error.ScriptRuntime,
        .media_query_list => |mql| if (std.mem.eql(u8, method, "toString")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .string = "[object MediaQueryList]" };
        } else if (std.mem.eql(u8, method, "addEventListener")) blk: {
            if (args.len < 2 or args.len > 3) return error.ScriptRuntime;
            const event_type_value = try evalExpr(allocator, host, bindings, args[0]);
            const event_type = try asString(allocator, event_type_value);
            if (!std.mem.eql(u8, event_type, "change")) {
                break :blk Value{ .undefined_value = {} };
            }
            const callback_value = try evalExpr(allocator, host, bindings, args[1]);
            const callback = switch (callback_value) {
                .function => |function| function,
                .null_value, .undefined_value => {
                    break :blk Value{ .undefined_value = {} };
                },
                else => return error.ScriptRuntime,
            };
            if (mql.add_listener_fn) |add_listener_fn| {
                const host_ptr = mql.host orelse return error.ScriptRuntime;
                try add_listener_fn(host_ptr, mql.media, callback);
            }
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "removeEventListener")) blk: {
            if (args.len < 2 or args.len > 3) return error.ScriptRuntime;
            const event_type_value = try evalExpr(allocator, host, bindings, args[0]);
            const event_type = try asString(allocator, event_type_value);
            if (!std.mem.eql(u8, event_type, "change")) {
                break :blk Value{ .undefined_value = {} };
            }
            const callback_value = try evalExpr(allocator, host, bindings, args[1]);
            const callback = switch (callback_value) {
                .function => |function| function,
                .null_value, .undefined_value => {
                    break :blk Value{ .undefined_value = {} };
                },
                else => return error.ScriptRuntime,
            };
            if (mql.remove_listener_fn) |remove_listener_fn| {
                const host_ptr = mql.host orelse return error.ScriptRuntime;
                try remove_listener_fn(host_ptr, mql.media, callback);
            }
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "addListener")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const callback_value = try evalExpr(allocator, host, bindings, args[0]);
            const callback = switch (callback_value) {
                .function => |function| function,
                .null_value, .undefined_value => {
                    break :blk Value{ .undefined_value = {} };
                },
                else => return error.ScriptRuntime,
            };
            if (mql.add_listener_fn) |add_listener_fn| {
                const host_ptr = mql.host orelse return error.ScriptRuntime;
                try add_listener_fn(host_ptr, mql.media, callback);
            }
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "removeListener")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const callback_value = try evalExpr(allocator, host, bindings, args[0]);
            const callback = switch (callback_value) {
                .function => |function| function,
                .null_value, .undefined_value => {
                    break :blk Value{ .undefined_value = {} };
                },
                else => return error.ScriptRuntime,
            };
            if (mql.remove_listener_fn) |remove_listener_fn| {
                const host_ptr = mql.host orelse return error.ScriptRuntime;
                try remove_listener_fn(host_ptr, mql.media, callback);
            }
            break :blk Value{ .undefined_value = {} };
        } else error.ScriptRuntime,
        .media_list => |media| if (std.mem.eql(u8, method, "item")) media_item: {
            if (args.len != 1) return error.ScriptRuntime;
            const index_value = try evalExpr(allocator, host, bindings, args[0]);
            const index = try asNodeListIndex(index_value);
            const media_text = try media.currentText();
            if (try mediaListItem(allocator, media_text, index)) |text| {
                break :media_item Value{ .string = text };
            }
            break :media_item Value{ .null_value = {} };
        } else if (std.mem.eql(u8, method, "appendMedium")) append_medium: {
            if (args.len != 1) return error.ScriptRuntime;
            var media_state = media;
            const medium_value = try evalExpr(allocator, host, bindings, args[0]);
            const medium = try mediaListNormalizeItem(try asString(allocator, medium_value));
            const current_text = try media_state.currentText();
            var items = try mediaListItems(allocator, current_text);
            defer items.deinit(allocator);
            if (!mediaListContains(items.items, medium)) {
                try items.append(allocator, medium);
            }
            try media_state.setText(try mediaListSerialize(allocator, items.items));
            break :append_medium Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "deleteMedium")) delete_medium: {
            if (args.len != 1) return error.ScriptRuntime;
            var media_state = media;
            const medium_value = try evalExpr(allocator, host, bindings, args[0]);
            const medium = try mediaListNormalizeItem(try asString(allocator, medium_value));
            const current_text = try media_state.currentText();
            var items = try mediaListItems(allocator, current_text);
            defer items.deinit(allocator);
            if (mediaListIndexOf(items.items, medium)) |index| {
                _ = items.orderedRemove(index);
                try media_state.setText(try mediaListSerialize(allocator, items.items));
            }
            break :delete_medium Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "toString")) media_to_string: {
            if (args.len != 0) return error.ScriptRuntime;
            break :media_to_string Value{ .string = try media.currentText() };
        } else error.ScriptRuntime,
        .collection_iterator => |iterator| if (std.mem.eql(u8, method, "next")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try collectionIteratorNext(allocator, iterator);
        } else error.ScriptRuntime,
        .node => |node_id| if (std.mem.eql(u8, method, "compareDocumentPosition")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const position = try nodeCompareDocumentPositionValue(allocator, host, bindings, object, args[0]);
            break :blk Value{ .number = @floatFromInt(position) };
        } else if (std.mem.eql(u8, method, "isSameNode")) blk: {
            break :blk try nodeSameNodeValue(allocator, host, bindings, object, args);
        } else if (std.mem.eql(u8, method, "isEqualNode")) blk: {
            break :blk try nodeEqualNodeValue(allocator, host, bindings, object, args);
        } else if (std.mem.eql(u8, method, "substringData")) blk: {
            if (args.len != 2) return error.ScriptRuntime;
            const text = host.domStore().characterDataText(node_id) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            const offset_value = try evalExpr(allocator, host, bindings, args[0]);
            const offset = try selectionIndexFromValue(allocator, offset_value);
            if (offset > text.len) return error.ScriptRuntime;
            const count_value = try evalExpr(allocator, host, bindings, args[1]);
            const count = try selectionIndexFromValue(allocator, count_value);
            const remaining = text.len - offset;
            const actual_count = if (count > remaining) remaining else count;
            break :blk Value{ .string = try allocator.dupe(u8, text[offset .. offset + actual_count]) };
        } else if (std.mem.eql(u8, method, "appendData")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const text = try asString(allocator, try evalExpr(allocator, host, bindings, args[0]));
            host.domStoreMut().appendCharacterData(node_id, text) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "insertData")) blk: {
            if (args.len != 2) return error.ScriptRuntime;
            const offset_value = try evalExpr(allocator, host, bindings, args[0]);
            const offset = try selectionIndexFromValue(allocator, offset_value);
            const text = try asString(allocator, try evalExpr(allocator, host, bindings, args[1]));
            host.domStoreMut().insertCharacterData(node_id, offset, text) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "deleteData")) blk: {
            if (args.len != 2) return error.ScriptRuntime;
            const offset_value = try evalExpr(allocator, host, bindings, args[0]);
            const offset = try selectionIndexFromValue(allocator, offset_value);
            const count_value = try evalExpr(allocator, host, bindings, args[1]);
            const count = try selectionIndexFromValue(allocator, count_value);
            host.domStoreMut().deleteCharacterData(node_id, offset, count) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "replaceData")) blk: {
            if (args.len != 3) return error.ScriptRuntime;
            const offset_value = try evalExpr(allocator, host, bindings, args[0]);
            const offset = try selectionIndexFromValue(allocator, offset_value);
            const count_value = try evalExpr(allocator, host, bindings, args[1]);
            const count = try selectionIndexFromValue(allocator, count_value);
            const text = try asString(allocator, try evalExpr(allocator, host, bindings, args[2]));
            host.domStoreMut().replaceCharacterData(node_id, offset, count, text) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "contains")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const contains = try nodeContainsValue(allocator, host, bindings, false, node_id, args[0]);
            break :blk Value{ .boolean = contains };
        } else if (std.mem.eql(u8, method, "remove")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            host.domStoreMut().removeNode(node_id) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "removeChild")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            break :blk try nodeRemoveChildValue(allocator, host, bindings, node_id, args[0]);
        } else if (std.mem.eql(u8, method, "splitText")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const offset_value = try evalExpr(allocator, host, bindings, args[0]);
            const offset = try selectionIndexFromValue(allocator, offset_value);
            const split = host.domStoreMut().splitTextNode(node_id, offset) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .node = split };
        } else if (std.mem.eql(u8, method, "hasChildNodes")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk Value{ .boolean = dom.hasChildNodes(host.domStore(), node_id) };
        } else if (std.mem.eql(u8, method, "cloneNode")) blk: {
            if (args.len > 1) return error.ScriptRuntime;
            const deep = if (args.len == 1) deep_blk: {
                const deep_value = try evalExpr(allocator, host, bindings, args[0]);
                break :deep_blk isTruthy(deep_value);
            } else false;
            break :blk try cloneNodeValue(allocator, host, node_id, deep, false);
        } else if (std.mem.eql(u8, method, "normalize")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try normalizeNodeValue(allocator, host, node_id);
        } else if (std.mem.eql(u8, method, "replaceWith")) blk: {
            break :blk try nodeReplaceWith(allocator, host, bindings, node_id, args);
        } else error.ScriptRuntime,
        .iterator_result, .collection_entry => error.ScriptRuntime,
        .event => |event| if (std.mem.eql(u8, method, "preventDefault")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            event.preventDefault();
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "stopPropagation")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            event.stopPropagation();
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "stopImmediatePropagation")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            event.stopImmediatePropagation();
            break :blk Value{ .undefined_value = {} };
        } else error.ScriptRuntime,
        .null_value, .undefined_value => error.ScriptRuntime,
        .boolean, .number, .string, .function => error.ScriptRuntime,
    };
}

fn nodeListForEach(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    list: NodeList,
    args: []*Expr,
) errors.Result(Value) {
    const callback_expr: *Expr = switch (args.len) {
        1, 2 => args[0],
        else => return error.ScriptRuntime,
    };
    const this_arg_expr: ?*Expr = if (args.len == 2) args[1] else null;

    const callback_value = try evalExpr(allocator, host, bindings, callback_expr);
    const callback = switch (callback_value) {
        .function => |function| function,
        else => return error.ScriptRuntime,
    };

    if (this_arg_expr) |expr| {
        _ = try evalExpr(allocator, host, bindings, expr);
    }

    const items = try nodeListCurrentValues(allocator, host, list);
    defer allocator.free(items);

    for (items, 0..) |item, index| {
        const positional = [_]Value{
            item,
            .{ .number = @floatFromInt(index) },
            .{ .node_list = list },
        };
        var function_bindings = try functionBindings(allocator, callback, positional[0..]);
        defer function_bindings.deinit(allocator);

        const source_name = try std.fmt.allocPrint(allocator, "nodelist:forEach:{d}", .{index});
        defer allocator.free(source_name);

        try executeScriptSource(
            allocator,
            host,
            callback.body_source,
            source_name,
            function_bindings.items,
        );
    }

    return Value{ .undefined_value = {} };
}

fn collectionItemForNodeId(node_id: dom.NodeId, kind: NodeListItemKind) Value {
    return switch (kind) {
        .element => .{ .element = node_id },
        .node => .{ .node = node_id },
    };
}

fn nodeValueForNodeId(
    allocator: std.mem.Allocator,
    host: anytype,
    node_id: dom.NodeId,
) errors.Result(Value) {
    _ = allocator;
    const node = host.domStore().nodeAt(node_id) orelse return error.ScriptRuntime;
    return switch (node.kind) {
        .document => Value{ .document = {} },
        .element => Value{ .element = node_id },
        .text, .comment => Value{ .node = node_id },
    };
}

fn directChildIds(
    allocator: std.mem.Allocator,
    host: anytype,
    root: dom.NodeId,
    element_only: bool,
) errors.Result([]dom.NodeId) {
    const children = host.domStore().childIds(root);
    if (!element_only) {
        return try allocator.dupe(dom.NodeId, children);
    }

    var filtered: std.ArrayList(dom.NodeId) = .empty;
    defer filtered.deinit(allocator);

    for (children) |child_id| {
        if (host.domStore().tagNameForNode(child_id) != null) {
            try filtered.append(allocator, child_id);
        }
    }

    return try allocator.dupe(dom.NodeId, filtered.items);
}

fn nodeListCurrentIds(
    allocator: std.mem.Allocator,
    host: anytype,
    list: NodeList,
) errors.Result([]dom.NodeId) {
    if (list.live_root) |root| {
        return switch (list.live_kind orelse .children) {
            .children => try directChildIds(allocator, host, root, list.item_kind == .element),
            .labels => try elementLabelsIds(allocator, host, root),
            .named_elements => try namedElementsIds(allocator, host, root, list.live_name),
        };
    }

    return try allocator.dupe(dom.NodeId, list.items);
}

fn nodeListCurrentValues(
    allocator: std.mem.Allocator,
    host: anytype,
    list: NodeList,
) errors.Result([]Value) {
    const node_ids = try nodeListCurrentIds(allocator, host, list);
    errdefer allocator.free(node_ids);

    var items = try allocator.alloc(Value, node_ids.len);
    errdefer allocator.free(items);

    for (node_ids, 0..) |node_id, index| {
        items[index] = collectionItemForNodeId(node_id, list.item_kind);
    }

    allocator.free(node_ids);
    return items;
}

fn htmlCollectionCurrentIds(
    allocator: std.mem.Allocator,
    host: anytype,
    collection: HtmlCollection,
) errors.Result([]dom.NodeId) {
    return switch (collection.kind) {
        .children => try directChildIds(allocator, host, collection.root, true),
        .forms => try documentFormsItems(allocator, host),
        .document_images => try documentImagesItems(allocator, host),
        .document_links => try documentLinksItems(allocator, host),
        .document_embeds => try documentEmbedsItems(allocator, host),
        .document_plugins => try documentPluginsItems(allocator, host),
        .navigator_plugins => try documentPluginsItems(allocator, host),
        .document_applets => try documentAppletsItems(allocator, host),
        .document_all => try documentAllItems(allocator, host),
        .form_elements => try formElementsItems(allocator, host, collection.root),
        .select_options => try selectOptionsItems(allocator, host, collection.root),
        .selected_options => try selectedOptionsItems(allocator, host, collection.root),
        .fieldset_elements => try fieldsetElementsItems(allocator, host, collection.root),
        .datalist_options => try datalistOptionsItems(allocator, host, collection.root),
        .map_areas => try mapAreasItems(allocator, host, collection.root),
        .table_t_bodies => try tableBodiesItems(allocator, host, collection.root),
        .table_rows => try tableRowsItems(allocator, host, collection.root),
        .row_cells => try rowCellsItems(allocator, host, collection.root),
        .tag_name => try tagNameItems(allocator, host, collection.root, collection.query),
        .tag_name_ns => try tagNameNsItems(
            allocator,
            host,
            collection.root,
            collection.namespace_uri,
            collection.local_name,
        ),
        .class_name => try classNameItems(allocator, host, collection.root, collection.query),
    };
}

fn htmlCollectionNamedItem(
    allocator: std.mem.Allocator,
    host: anytype,
    collection: HtmlCollection,
    name: []const u8,
) errors.Result(Value) {
    const items = try htmlCollectionCurrentIds(allocator, host, collection);
    defer allocator.free(items);

    for (items) |item| {
        const id = (try host.domStore().getAttribute(item, "id")) orelse "";
        if (std.mem.eql(u8, id, name)) return Value{ .element = item };
    }

    var first_name_match: ?dom.NodeId = null;
    var name_match_count: usize = 0;
    for (items) |item| {
        const attr_name = (try host.domStore().getAttribute(item, "name")) orelse "";
        if (std.mem.eql(u8, attr_name, name)) {
            if (first_name_match == null) {
                first_name_match = item;
            }
            name_match_count += 1;
        }
    }

    if (name_match_count == 0) {
        return Value{ .null_value = {} };
    }

    if (collection.kind == .form_elements and name_match_count > 1) {
        return Value{ .radio_node_list = .{ .root = collection.root, .name = name } };
    }

    return Value{ .element = first_name_match.? };
}

fn htmlCollectionForEach(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    collection: HtmlCollection,
    args: []*Expr,
) errors.Result(Value) {
    const callback_expr: *Expr = switch (args.len) {
        1, 2 => args[0],
        else => return error.ScriptRuntime,
    };
    const this_arg_expr: ?*Expr = if (args.len == 2) args[1] else null;

    const callback_value = try evalExpr(allocator, host, bindings, callback_expr);
    const callback = switch (callback_value) {
        .function => |function| function,
        else => return error.ScriptRuntime,
    };

    if (this_arg_expr) |expr| {
        _ = try evalExpr(allocator, host, bindings, expr);
    }

    const items = try htmlCollectionCurrentIds(allocator, host, collection);
    defer allocator.free(items);

    for (items, 0..) |item, index| {
        const positional = [_]Value{
            .{ .element = item },
            .{ .number = @floatFromInt(index) },
            .{ .html_collection = collection },
        };
        var function_bindings = try functionBindings(allocator, callback, positional[0..]);
        defer function_bindings.deinit(allocator);

        const source_name = try std.fmt.allocPrint(allocator, "htmlcollection:forEach:{d}", .{index});
        defer allocator.free(source_name);

        try executeScriptSource(
            allocator,
            host,
            callback.body_source,
            source_name,
            function_bindings.items,
        );
    }

    return Value{ .undefined_value = {} };
}

fn nodeLikeMember(
    allocator: std.mem.Allocator,
    host: anytype,
    node_id: dom.NodeId,
    property: []const u8,
) errors.Result(?Value) {
    const node = host.domStore().nodeAt(node_id) orelse return error.ScriptRuntime;

    if (std.mem.eql(u8, property, "baseURI")) {
        return Value{ .string = host.currentLocationUrl() };
    }

    if (std.mem.eql(u8, property, "origin")) {
        return Value{ .string = try originFromUrl(allocator, host.currentLocationUrl()) };
    }

    if (std.mem.eql(u8, property, "nodeName")) {
        const name = host.domStore().nodeNameForNode(node_id) orelse return error.ScriptRuntime;
        return Value{ .string = name };
    }

    if (std.mem.eql(u8, property, "nodeType")) {
        const node_type = host.domStore().nodeTypeForNode(node_id) orelse return error.ScriptRuntime;
        return Value{ .number = @floatFromInt(node_type) };
    }

    if (std.mem.eql(u8, property, "nodeValue")) {
        return switch (node.kind) {
            .document, .element => Value{ .null_value = {} },
            .text => |text| Value{ .string = text },
            .comment => |comment| Value{ .string = comment },
        };
    }

    if (std.mem.eql(u8, property, "data")) {
        return switch (node.kind) {
            .text => |text| Value{ .string = text },
            .comment => |comment| Value{ .string = comment },
            else => error.ScriptRuntime,
        };
    }

    if (std.mem.eql(u8, property, "length")) {
        return switch (node.kind) {
            .text => |text| Value{ .number = @floatFromInt(text.len) },
            .comment => |comment| Value{ .number = @floatFromInt(comment.len) },
            else => error.ScriptRuntime,
        };
    }

    if (std.mem.eql(u8, property, "wholeText")) {
        return switch (node.kind) {
            .text => Value{ .string = try host.domStore().wholeText(allocator, node_id) },
            else => error.ScriptRuntime,
        };
    }

    if (std.mem.eql(u8, property, "isConnected")) {
        return Value{ .boolean = dom.isConnected(host.domStore(), node_id) };
    }

    if (std.mem.eql(u8, property, "ownerDocument")) {
        if (node.kind == .document) {
            return Value{ .null_value = {} };
        }
        return Value{ .document = {} };
    }

    if (std.mem.eql(u8, property, "parentNode")) {
        const parent_id = node.parent orelse return Value{ .null_value = {} };
        const parent_node = host.domStore().nodeAt(parent_id) orelse return error.ScriptRuntime;
        return switch (parent_node.kind) {
            .document => Value{ .document = {} },
            .element => Value{ .element = parent_id },
            else => Value{ .node = parent_id },
        };
    }

    if (std.mem.eql(u8, property, "parentElement")) {
        const parent_id = node.parent orelse return Value{ .null_value = {} };
        const parent_node = host.domStore().nodeAt(parent_id) orelse return error.ScriptRuntime;
        if (parent_node.kind == .element) {
            return Value{ .element = parent_id };
        }
        return Value{ .null_value = {} };
    }

    if (std.mem.eql(u8, property, "firstChild")) {
        const child_id = dom.firstChild(host.domStore(), node_id) orelse return Value{ .null_value = {} };
        return try nodeValueForNodeId(allocator, host, child_id);
    }

    if (std.mem.eql(u8, property, "lastChild")) {
        const child_id = dom.lastChild(host.domStore(), node_id) orelse return Value{ .null_value = {} };
        return try nodeValueForNodeId(allocator, host, child_id);
    }

    if (std.mem.eql(u8, property, "nextSibling")) {
        const sibling_id = dom.nextSibling(host.domStore(), node_id) orelse return Value{ .null_value = {} };
        return try nodeValueForNodeId(allocator, host, sibling_id);
    }

    if (std.mem.eql(u8, property, "previousSibling")) {
        const sibling_id = dom.previousSibling(host.domStore(), node_id) orelse return Value{ .null_value = {} };
        return try nodeValueForNodeId(allocator, host, sibling_id);
    }

    if (std.mem.eql(u8, property, "nextElementSibling")) {
        const sibling_id = dom.nextElementSibling(host.domStore(), node_id) orelse return Value{ .null_value = {} };
        return Value{ .element = sibling_id };
    }

    if (std.mem.eql(u8, property, "previousElementSibling")) {
        const sibling_id = dom.previousElementSibling(host.domStore(), node_id) orelse return Value{ .null_value = {} };
        return Value{ .element = sibling_id };
    }

    if (std.mem.eql(u8, property, "firstElementChild")) {
        switch (node.kind) {
            .document, .element => {
                const children = host.domStore().childIds(node_id);
                for (children) |child_id| {
                    if (host.domStore().tagNameForNode(child_id) != null) {
                        return Value{ .element = child_id };
                    }
                }
                return Value{ .null_value = {} };
            },
            else => return error.ScriptRuntime,
        }
    }

    if (std.mem.eql(u8, property, "lastElementChild")) {
        switch (node.kind) {
            .document, .element => {
                const children = host.domStore().childIds(node_id);
                var index = children.len;
                while (index > 0) {
                    index -= 1;
                    const child_id = children[index];
                    if (host.domStore().tagNameForNode(child_id) != null) {
                        return Value{ .element = child_id };
                    }
                }
                return Value{ .null_value = {} };
            },
            else => return error.ScriptRuntime,
        }
    }

    if (std.mem.eql(u8, property, "childElementCount")) {
        switch (node.kind) {
            .document, .element => {
                var count: usize = 0;
                for (host.domStore().childIds(node_id)) |child_id| {
                    if (host.domStore().tagNameForNode(child_id) != null) {
                        count += 1;
                    }
                }
                return Value{ .number = @floatFromInt(count) };
            },
            else => return error.ScriptRuntime,
        }
    }

    if (std.mem.eql(u8, property, "textContent")) {
        return switch (node.kind) {
            .document => Value{ .null_value = {} },
            .element => Value{ .string = try host.domStore().textContent(allocator, node_id) },
            .text => |text| Value{ .string = text },
            .comment => |comment| Value{ .string = comment },
        };
    }

    if (std.mem.eql(u8, property, "childNodes")) {
        return Value{
            .node_list = .{
                .items = &.{},
                .item_kind = .node,
                .live_root = node_id,
                .live_kind = .children,
            },
        };
    }

    if (std.mem.eql(u8, property, "children")) {
        const node_record = host.domStore().nodeAt(node_id) orelse return error.ScriptRuntime;
        switch (node_record.kind) {
            .document, .element => {
                return Value{ .html_collection = .{ .root = node_id } };
            },
            else => return error.ScriptRuntime,
        }
    }

    return null;
}

fn cloneNodeValue(
    allocator: std.mem.Allocator,
    host: anytype,
    node_id: dom.NodeId,
    deep: bool,
    as_template_content: bool,
) errors.Result(Value) {
    _ = allocator;
    const node = host.domStore().nodeAt(node_id) orelse return error.ScriptRuntime;
    const source_kind = node.kind;
    if (source_kind == .document) {
        return error.ScriptRuntime;
    }

    const cloned_node_id = host.domStoreMut().cloneNode(node_id, deep) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };

    if (as_template_content) {
        return Value{ .template_content = cloned_node_id };
    }

    return switch (source_kind) {
        .element => Value{ .element = cloned_node_id },
        .text, .comment => Value{ .node = cloned_node_id },
        .document => error.ScriptRuntime,
    };
}

fn normalizeNodeValue(
    allocator: std.mem.Allocator,
    host: anytype,
    node_id: dom.NodeId,
) errors.Result(Value) {
    _ = allocator;
    host.domStoreMut().normalize(node_id) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return Value{ .undefined_value = {} };
}

fn collectionIteratorFromValues(
    allocator: std.mem.Allocator,
    items: []Value,
) errors.Result(Value) {
    errdefer allocator.free(items);

    const state = try allocator.create(CollectionIteratorState);
    state.* = .{
        .items = items,
        .index = 0,
    };
    return Value{ .collection_iterator = state };
}

fn collectionEntriesFromValues(
    allocator: std.mem.Allocator,
    items: []Value,
) errors.Result(Value) {
    var entries = try allocator.alloc(Value, items.len);
    errdefer allocator.free(entries);

    for (items, 0..) |item, index| {
        const entry = try allocator.create(CollectionEntry);
        entry.* = .{
            .index = index,
            .value = item,
        };
        entries[index] = Value{ .collection_entry = entry };
    }

    return try collectionIteratorFromValues(allocator, entries);
}

fn collectionIteratorFromNodeIds(
    allocator: std.mem.Allocator,
    node_ids: []const dom.NodeId,
    kind: NodeListItemKind,
    keys: bool,
) errors.Result(Value) {
    var items = try allocator.alloc(Value, node_ids.len);
    errdefer allocator.free(items);

    for (node_ids, 0..) |node_id, index| {
        items[index] = if (keys)
            Value{ .number = @floatFromInt(index) }
        else
            collectionItemForNodeId(node_id, kind);
    }

    return try collectionIteratorFromValues(allocator, items);
}

fn collectionEntriesFromNodeIds(
    allocator: std.mem.Allocator,
    node_ids: []const dom.NodeId,
    kind: NodeListItemKind,
) errors.Result(Value) {
    var items = try allocator.alloc(Value, node_ids.len);
    errdefer allocator.free(items);

    for (node_ids, 0..) |node_id, index| {
        items[index] = collectionItemForNodeId(node_id, kind);
    }

    return try collectionEntriesFromValues(allocator, items);
}

fn collectionEntriesFromStyleSheetIds(
    allocator: std.mem.Allocator,
    node_ids: []const dom.NodeId,
) errors.Result(Value) {
    var items = try allocator.alloc(Value, node_ids.len);
    errdefer allocator.free(items);

    for (node_ids, 0..) |node_id, index| {
        items[index] = Value{ .style_sheet = node_id };
    }

    return try collectionEntriesFromValues(allocator, items);
}

fn collectionIteratorNext(
    allocator: std.mem.Allocator,
    iterator: *CollectionIteratorState,
) errors.Result(Value) {
    const done = iterator.index >= iterator.items.len;
    const result = try allocator.create(IteratorResult);
    result.* = .{
        .value = if (done)
            Value{ .undefined_value = {} }
        else
            iterator.items[iterator.index],
        .done = done,
    };
    if (!done) {
        iterator.index += 1;
    }
    return Value{ .iterator_result = result };
}

fn nodeListKeys(
    allocator: std.mem.Allocator,
    host: anytype,
    list: NodeList,
) errors.Result(Value) {
    const items = try nodeListCurrentIds(allocator, host, list);
    defer allocator.free(items);
    return try collectionIteratorFromNodeIds(allocator, items, list.item_kind, true);
}

fn nodeListValues(
    allocator: std.mem.Allocator,
    host: anytype,
    list: NodeList,
) errors.Result(Value) {
    const items = try nodeListCurrentIds(allocator, host, list);
    defer allocator.free(items);
    return try collectionIteratorFromNodeIds(allocator, items, list.item_kind, false);
}

fn nodeListEntries(
    allocator: std.mem.Allocator,
    host: anytype,
    list: NodeList,
) errors.Result(Value) {
    const items = try nodeListCurrentIds(allocator, host, list);
    defer allocator.free(items);
    return try collectionEntriesFromNodeIds(allocator, items, list.item_kind);
}

fn htmlCollectionKeys(
    allocator: std.mem.Allocator,
    host: anytype,
    collection: HtmlCollection,
) errors.Result(Value) {
    const items = try htmlCollectionCurrentIds(allocator, host, collection);
    defer allocator.free(items);
    return try collectionIteratorFromNodeIds(allocator, items, .element, true);
}

fn htmlCollectionValues(
    allocator: std.mem.Allocator,
    host: anytype,
    collection: HtmlCollection,
) errors.Result(Value) {
    const items = try htmlCollectionCurrentIds(allocator, host, collection);
    defer allocator.free(items);
    return try collectionIteratorFromNodeIds(allocator, items, .element, false);
}

fn htmlCollectionEntries(
    allocator: std.mem.Allocator,
    host: anytype,
    collection: HtmlCollection,
) errors.Result(Value) {
    const items = try htmlCollectionCurrentIds(allocator, host, collection);
    defer allocator.free(items);
    return try collectionEntriesFromNodeIds(allocator, items, .element);
}

fn documentScriptsItems(
    allocator: std.mem.Allocator,
    host: anytype,
) errors.Result([]dom.NodeId) {
    const scripts = host.domStore().select(allocator, "script") catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return scripts;
}

fn documentScriptsKeys(
    allocator: std.mem.Allocator,
    host: anytype,
) errors.Result(Value) {
    const scripts = try documentScriptsItems(allocator, host);
    defer allocator.free(scripts);
    return try collectionIteratorFromNodeIds(allocator, scripts, .element, true);
}

fn documentScriptsValues(
    allocator: std.mem.Allocator,
    host: anytype,
) errors.Result(Value) {
    const scripts = try documentScriptsItems(allocator, host);
    defer allocator.free(scripts);
    return try collectionIteratorFromNodeIds(allocator, scripts, .element, false);
}

fn documentScriptsEntries(
    allocator: std.mem.Allocator,
    host: anytype,
) errors.Result(Value) {
    const scripts = try documentScriptsItems(allocator, host);
    defer allocator.free(scripts);
    return try collectionEntriesFromNodeIds(allocator, scripts, .element);
}

fn documentScriptsNamedItem(
    allocator: std.mem.Allocator,
    host: anytype,
    name: []const u8,
) errors.Result(?dom.NodeId) {
    const scripts = try documentScriptsItems(allocator, host);
    defer allocator.free(scripts);

    for (scripts) |script_id| {
        const id = (try host.domStore().getAttribute(script_id, "id")) orelse "";
        if (std.mem.eql(u8, id, name)) {
            return script_id;
        }

        const script_name = (try host.domStore().getAttribute(script_id, "name")) orelse "";
        if (std.mem.eql(u8, script_name, name)) {
            return script_id;
        }
    }

    return null;
}

fn documentAnchorsItems(
    allocator: std.mem.Allocator,
    host: anytype,
) errors.Result([]dom.NodeId) {
    const candidates = host.domStore().select(allocator, "a") catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    defer allocator.free(candidates);

    var write_index: usize = 0;
    for (candidates) |anchor_id| {
        const present = host.domStore().hasAttribute(anchor_id, "name") catch |err| switch (err) {
            error.OutOfMemory => return error.OutOfMemory,
            else => return error.ScriptRuntime,
        };
        if (present) {
            candidates[write_index] = anchor_id;
            write_index += 1;
        }
    }

    return try allocator.dupe(dom.NodeId, candidates[0..write_index]);
}

fn documentAnchorsKeys(
    allocator: std.mem.Allocator,
    host: anytype,
) errors.Result(Value) {
    const anchors = try documentAnchorsItems(allocator, host);
    defer allocator.free(anchors);
    return try collectionIteratorFromNodeIds(allocator, anchors, .element, true);
}

fn documentAnchorsValues(
    allocator: std.mem.Allocator,
    host: anytype,
) errors.Result(Value) {
    const anchors = try documentAnchorsItems(allocator, host);
    defer allocator.free(anchors);
    return try collectionIteratorFromNodeIds(allocator, anchors, .element, false);
}

fn documentAnchorsEntries(
    allocator: std.mem.Allocator,
    host: anytype,
) errors.Result(Value) {
    const anchors = try documentAnchorsItems(allocator, host);
    defer allocator.free(anchors);
    return try collectionEntriesFromNodeIds(allocator, anchors, .element);
}

fn documentAnchorsNamedItem(
    allocator: std.mem.Allocator,
    host: anytype,
    name: []const u8,
) errors.Result(?dom.NodeId) {
    const anchors = try documentAnchorsItems(allocator, host);
    defer allocator.free(anchors);

    for (anchors) |anchor_id| {
        const id = (try host.domStore().getAttribute(anchor_id, "id")) orelse "";
        if (std.mem.eql(u8, id, name)) {
            return anchor_id;
        }

        const anchor_name = (try host.domStore().getAttribute(anchor_id, "name")) orelse "";
        if (std.mem.eql(u8, anchor_name, name)) {
            return anchor_id;
        }
    }

    return null;
}

fn documentFormsItems(
    allocator: std.mem.Allocator,
    host: anytype,
) errors.Result([]dom.NodeId) {
    const forms = host.domStore().select(allocator, "form") catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return forms;
}

fn formElementsItems(
    allocator: std.mem.Allocator,
    host: anytype,
    form_id: dom.NodeId,
) errors.Result([]dom.NodeId) {
    return host.domStore().selectWithin(
        allocator,
        form_id,
        "input, textarea, select, button, output, fieldset, object",
    ) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
}

fn fieldsetElementsItems(
    allocator: std.mem.Allocator,
    host: anytype,
    fieldset_id: dom.NodeId,
) errors.Result([]dom.NodeId) {
    return host.domStore().selectWithin(
        allocator,
        fieldset_id,
        "input, select, textarea, button",
    ) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
}

fn selectOptionsItems(
    allocator: std.mem.Allocator,
    host: anytype,
    select_id: dom.NodeId,
) errors.Result([]dom.NodeId) {
    return host.domStore().selectWithin(allocator, select_id, "option") catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
}

fn datalistOptionsItems(
    allocator: std.mem.Allocator,
    host: anytype,
    datalist_id: dom.NodeId,
) errors.Result([]dom.NodeId) {
    return try selectOptionsItems(allocator, host, datalist_id);
}

fn htmlCollectionSelectOptionsAdd(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    collection: HtmlCollection,
    args: []*Expr,
) errors.Result(Value) {
    if (collection.kind != .select_options) return error.ScriptRuntime;
    if (args.len != 1) return error.ScriptRuntime;

    const option = try evalElementHandle(allocator, host, bindings, args[0]);
    const tag_name = host.domStore().tagNameForNode(option) orelse return error.ScriptRuntime;
    if (!std.mem.eql(u8, tag_name, "option")) return error.ScriptRuntime;

    _ = host.domStoreMut().appendChild(collection.root, option) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };

    return Value{ .undefined_value = {} };
}

fn htmlCollectionSelectOptionsRemove(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    collection: HtmlCollection,
    args: []*Expr,
) errors.Result(Value) {
    if (collection.kind != .select_options) return error.ScriptRuntime;
    if (args.len != 1) return error.ScriptRuntime;

    const index_value = try evalExpr(allocator, host, bindings, args[0]);
    const index = optionalNodeListIndex(index_value) orelse return Value{ .undefined_value = {} };

    const items = try selectOptionsItems(allocator, host, collection.root);
    defer allocator.free(items);
    if (index >= items.len) {
        return Value{ .undefined_value = {} };
    }

    host.domStoreMut().removeNode(items[index]) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };

    return Value{ .undefined_value = {} };
}

fn documentSelectItems(
    allocator: std.mem.Allocator,
    host: anytype,
    selector: []const u8,
) errors.Result([]dom.NodeId) {
    return host.domStore().select(allocator, selector) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
}

fn descendantElementIds(
    allocator: std.mem.Allocator,
    host: anytype,
    root: dom.NodeId,
) errors.Result([]dom.NodeId) {
    var filtered: std.ArrayList(dom.NodeId) = .empty;
    errdefer filtered.deinit(allocator);

    try appendAllElementIds(allocator, host, root, &filtered);
    const result = try allocator.dupe(dom.NodeId, filtered.items);
    filtered.deinit(allocator);
    return result;
}

fn namedElementsIds(
    allocator: std.mem.Allocator,
    host: anytype,
    root: dom.NodeId,
    name: ?[]const u8,
) errors.Result([]dom.NodeId) {
    const target_name = name orelse return allocator.alloc(dom.NodeId, 0);
    const candidates = try descendantElementIds(allocator, host, root);
    defer allocator.free(candidates);

    var filtered: std.ArrayList(dom.NodeId) = .empty;
    defer filtered.deinit(allocator);

    for (candidates) |node_id| {
        const candidate_name = (try host.domStore().getAttribute(node_id, "name")) orelse continue;
        if (std.mem.eql(u8, candidate_name, target_name)) {
            try filtered.append(allocator, node_id);
        }
    }

    return try allocator.dupe(dom.NodeId, filtered.items);
}

fn tagNameItems(
    allocator: std.mem.Allocator,
    host: anytype,
    root: dom.NodeId,
    tag_name: ?[]const u8,
) errors.Result([]dom.NodeId) {
    const query = tag_name orelse return allocator.alloc(dom.NodeId, 0);
    const candidates = try descendantElementIds(allocator, host, root);
    if (std.mem.eql(u8, query, "*")) {
        return candidates;
    }
    defer allocator.free(candidates);

    var filtered: std.ArrayList(dom.NodeId) = .empty;
    defer filtered.deinit(allocator);

    for (candidates) |node_id| {
        const candidate_tag = host.domStore().tagNameForNode(node_id) orelse continue;
        if (std.ascii.eqlIgnoreCase(candidate_tag, query)) {
            try filtered.append(allocator, node_id);
        }
    }

    return try allocator.dupe(dom.NodeId, filtered.items);
}

fn tagNameNsItems(
    allocator: std.mem.Allocator,
    host: anytype,
    root: dom.NodeId,
    namespace_uri: ?[]const u8,
    local_name: ?[]const u8,
) errors.Result([]dom.NodeId) {
    const namespace_query = namespace_uri orelse return allocator.alloc(dom.NodeId, 0);
    const local_query = local_name orelse return allocator.alloc(dom.NodeId, 0);
    const candidates = try descendantElementIds(allocator, host, root);
    if (std.mem.eql(u8, namespace_query, "*") and std.mem.eql(u8, local_query, "*")) {
        return candidates;
    }
    defer allocator.free(candidates);

    var filtered: std.ArrayList(dom.NodeId) = .empty;
    defer filtered.deinit(allocator);

    for (candidates) |node_id| {
        const candidate_namespace = host.domStore().namespaceUriForNode(node_id) orelse continue;
        if (!std.mem.eql(u8, namespace_query, "*") and !std.mem.eql(u8, candidate_namespace, namespace_query)) {
            continue;
        }

        if (!std.mem.eql(u8, local_query, "*")) {
            const candidate_tag = host.domStore().tagNameForNode(node_id) orelse continue;
            if (!std.ascii.eqlIgnoreCase(candidate_tag, local_query)) {
                continue;
            }
        }

        try filtered.append(allocator, node_id);
    }

    return try allocator.dupe(dom.NodeId, filtered.items);
}

fn classNameItems(
    allocator: std.mem.Allocator,
    host: anytype,
    root: dom.NodeId,
    class_names: ?[]const u8,
) errors.Result([]dom.NodeId) {
    const query = class_names orelse return allocator.alloc(dom.NodeId, 0);
    var required_tokens: std.ArrayList([]const u8) = .empty;
    errdefer required_tokens.deinit(allocator);

    var required_iter = std.mem.tokenizeAny(u8, query, " \t\r\n\x0c");
    while (required_iter.next()) |token| {
        try required_tokens.append(allocator, token);
    }
    if (required_tokens.items.len == 0) {
        required_tokens.deinit(allocator);
        return allocator.alloc(dom.NodeId, 0);
    }

    const candidates = try descendantElementIds(allocator, host, root);
    defer allocator.free(candidates);

    var filtered: std.ArrayList(dom.NodeId) = .empty;
    defer filtered.deinit(allocator);

    for (candidates) |node_id| {
        const class_attr = (try host.domStore().getAttribute(node_id, "class")) orelse continue;
        var actual_tokens: std.ArrayList([]const u8) = .empty;
        defer actual_tokens.deinit(allocator);

        var actual_iter = std.mem.tokenizeAny(u8, class_attr, " \t\r\n\x0c");
        while (actual_iter.next()) |token| {
            try actual_tokens.append(allocator, token);
        }

        var matches_all = true;
        for (required_tokens.items) |required_token| {
            if (!classListContains(actual_tokens.items, required_token)) {
                matches_all = false;
                break;
            }
        }

        if (matches_all) {
            try filtered.append(allocator, node_id);
        }
    }

    return try allocator.dupe(dom.NodeId, filtered.items);
}

fn documentImagesItems(
    allocator: std.mem.Allocator,
    host: anytype,
) errors.Result([]dom.NodeId) {
    return try documentSelectItems(allocator, host, "img");
}

fn documentLinksItems(
    allocator: std.mem.Allocator,
    host: anytype,
) errors.Result([]dom.NodeId) {
    return try documentSelectItems(allocator, host, "a[href], area[href]");
}

fn documentEmbedsItems(
    allocator: std.mem.Allocator,
    host: anytype,
) errors.Result([]dom.NodeId) {
    return try documentSelectItems(allocator, host, "embed");
}

fn documentPluginsItems(
    allocator: std.mem.Allocator,
    host: anytype,
) errors.Result([]dom.NodeId) {
    return try documentSelectItems(allocator, host, "embed");
}

fn documentAppletsItems(
    allocator: std.mem.Allocator,
    host: anytype,
) errors.Result([]dom.NodeId) {
    return try documentSelectItems(allocator, host, "applet");
}

fn documentAllItems(
    allocator: std.mem.Allocator,
    host: anytype,
) errors.Result([]dom.NodeId) {
    var filtered: std.ArrayList(dom.NodeId) = .empty;
    defer filtered.deinit(allocator);

    try appendAllElementIds(allocator, host, host.domStore().documentId(), &filtered);
    return try allocator.dupe(dom.NodeId, filtered.items);
}

fn documentStyleSheetsItems(
    allocator: std.mem.Allocator,
    host: anytype,
) errors.Result([]dom.NodeId) {
    var filtered: std.ArrayList(dom.NodeId) = .empty;
    defer filtered.deinit(allocator);

    try appendStyleSheetIds(allocator, host, host.domStore().documentId(), &filtered);
    return try allocator.dupe(dom.NodeId, filtered.items);
}

fn documentStyleSheetsKeys(
    allocator: std.mem.Allocator,
    host: anytype,
) errors.Result(Value) {
    const sheets = try documentStyleSheetsItems(allocator, host);
    defer allocator.free(sheets);
    return try collectionIteratorFromNodeIds(allocator, sheets, .element, true);
}

fn documentStyleSheetsValues(
    allocator: std.mem.Allocator,
    host: anytype,
) errors.Result(Value) {
    const sheets = try documentStyleSheetsItems(allocator, host);
    defer allocator.free(sheets);
    var items = try allocator.alloc(Value, sheets.len);
    errdefer allocator.free(items);
    for (sheets, 0..) |sheet_id, index| {
        items[index] = Value{ .style_sheet = sheet_id };
    }
    return try collectionIteratorFromValues(allocator, items);
}

fn documentStyleSheetsEntries(
    allocator: std.mem.Allocator,
    host: anytype,
) errors.Result(Value) {
    const sheets = try documentStyleSheetsItems(allocator, host);
    defer allocator.free(sheets);
    return try collectionEntriesFromStyleSheetIds(allocator, sheets);
}

fn documentStyleSheetsForEach(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    args: []*Expr,
) errors.Result(Value) {
    const callback_expr: *Expr = switch (args.len) {
        1, 2 => args[0],
        else => return error.ScriptRuntime,
    };
    const this_arg_expr: ?*Expr = if (args.len == 2) args[1] else null;

    const callback_value = try evalExpr(allocator, host, bindings, callback_expr);
    const callback = switch (callback_value) {
        .function => |function| function,
        else => return error.ScriptRuntime,
    };

    if (this_arg_expr) |expr| {
        _ = try evalExpr(allocator, host, bindings, expr);
    }

    const sheets = try documentStyleSheetsItems(allocator, host);
    defer allocator.free(sheets);

    for (sheets, 0..) |sheet_id, index| {
        const positional = [_]Value{
            .{ .style_sheet = sheet_id },
            .{ .number = @floatFromInt(index) },
            .{ .document_style_sheets = {} },
        };
        var function_bindings = try functionBindings(allocator, callback, positional[0..]);
        defer function_bindings.deinit(allocator);

        const source_name = try std.fmt.allocPrint(allocator, "stylesheetlist:forEach:{d}", .{index});
        defer allocator.free(source_name);

        try executeScriptSource(
            allocator,
            host,
            callback.body_source,
            source_name,
            function_bindings.items,
        );
    }

    return Value{ .undefined_value = {} };
}

fn cssRuleListCurrentValues(
    allocator: std.mem.Allocator,
    host: anytype,
    list: CssRuleListState,
) errors.Result([]Value) {
    return switch (list) {
        .sheet => |sheet_id| blk: {
            const tag_name = host.domStore().tagNameForNode(sheet_id) orelse return error.ScriptRuntime;
            if (std.mem.eql(u8, tag_name, "style")) {
                const css_text = try host.domStore().textContent(allocator, sheet_id);
                const rules = try parseStyleSheetRuleValues(allocator, css_text);
                defer allocator.free(rules);
                cssRuleValuesAnnotate(rules, sheet_id, null);
                const cloned_rules = try cssRuleValuesCloneForReturn(allocator, rules, sheet_id, null);
                break :blk cloned_rules;
            }
            if (std.mem.eql(u8, tag_name, "link")) {
                if (try isStyleSheetLinkElement(host, sheet_id)) {
                    break :blk try allocator.alloc(Value, 0);
                }
            }
            break :blk error.ScriptRuntime;
        },
        .items => |items| try allocator.dupe(Value, items),
    };
}

fn cssRuleValuesAnnotate(
    values: []Value,
    parent_style_sheet: ?dom.NodeId,
    parent_rule: ?*const CssRuleState,
) void {
    for (values) |*value| {
        switch (value.*) {
            .css_rule => |*rule| cssRuleSetParents(rule, parent_style_sheet, parent_rule),
            else => {},
        }
    }
}

fn cssRuleValuesCloneForReturn(
    allocator: std.mem.Allocator,
    values: []Value,
    parent_style_sheet: ?dom.NodeId,
    parent_rule: ?*const CssRuleState,
) errors.Result([]Value) {
    var cloned_values = try allocator.alloc(Value, values.len);
    errdefer allocator.free(cloned_values);

    for (values, 0..) |value, index| {
        cloned_values[index] = try cssRuleValueCloneForReturn(
            allocator,
            value,
            parent_style_sheet,
            parent_rule,
        );
    }

    return cloned_values;
}

fn cssRuleValueCloneForReturn(
    allocator: std.mem.Allocator,
    value: Value,
    parent_style_sheet: ?dom.NodeId,
    parent_rule: ?*const CssRuleState,
) errors.Result(Value) {
    return switch (value) {
        .css_rule => |rule| blk: {
            const cloned = try allocator.create(CssRuleState);
            errdefer allocator.destroy(cloned);
            cloned.* = rule;

            switch (cloned.*) {
                .style => |*style| {
                    style.parent_style_sheet = parent_style_sheet;
                    style.parent_rule = parent_rule;
                },
                .media => |*media| {
                    media.parent_style_sheet = parent_style_sheet;
                    media.parent_rule = parent_rule;
                    media.css_rules = try cssRuleValuesCloneForReturn(
                        allocator,
                        media.css_rules,
                        parent_style_sheet,
                        cloned,
                    );
                },
                .supports => |*supports| {
                    supports.parent_style_sheet = parent_style_sheet;
                    supports.parent_rule = parent_rule;
                    supports.css_rules = try cssRuleValuesCloneForReturn(
                        allocator,
                        supports.css_rules,
                        parent_style_sheet,
                        cloned,
                    );
                },
                .document => |*document| {
                    document.parent_style_sheet = parent_style_sheet;
                    document.parent_rule = parent_rule;
                    document.css_rules = try cssRuleValuesCloneForReturn(
                        allocator,
                        document.css_rules,
                        parent_style_sheet,
                        cloned,
                    );
                },
                .container => |*container| {
                    container.parent_style_sheet = parent_style_sheet;
                    container.parent_rule = parent_rule;
                    container.css_rules = try cssRuleValuesCloneForReturn(
                        allocator,
                        container.css_rules,
                        parent_style_sheet,
                        cloned,
                    );
                },
                .starting_style => |*starting_style| {
                    starting_style.parent_style_sheet = parent_style_sheet;
                    starting_style.parent_rule = parent_rule;
                    starting_style.css_rules = try cssRuleValuesCloneForReturn(
                        allocator,
                        starting_style.css_rules,
                        parent_style_sheet,
                        cloned,
                    );
                },
                .keyframes => |*keyframes| {
                    keyframes.parent_style_sheet = parent_style_sheet;
                    keyframes.parent_rule = parent_rule;
                    keyframes.css_rules = try cssRuleValuesCloneForReturn(
                        allocator,
                        keyframes.css_rules,
                        parent_style_sheet,
                        cloned,
                    );
                },
                .keyframe => |*keyframe| {
                    keyframe.parent_style_sheet = parent_style_sheet;
                    keyframe.parent_rule = parent_rule;
                },
                .font_face => |*font_face| {
                    font_face.parent_style_sheet = parent_style_sheet;
                    font_face.parent_rule = parent_rule;
                },
                .font_feature_values => |*font_feature_values| {
                    font_feature_values.parent_style_sheet = parent_style_sheet;
                    font_feature_values.parent_rule = parent_rule;
                },
                .font_palette_values => |*font_palette_values| {
                    font_palette_values.parent_style_sheet = parent_style_sheet;
                    font_palette_values.parent_rule = parent_rule;
                },
                .color_profile => |*color_profile| {
                    color_profile.parent_style_sheet = parent_style_sheet;
                    color_profile.parent_rule = parent_rule;
                },
                .scope => |*scope| {
                    scope.parent_style_sheet = parent_style_sheet;
                    scope.parent_rule = parent_rule;
                    scope.css_rules = try cssRuleValuesCloneForReturn(
                        allocator,
                        scope.css_rules,
                        parent_style_sheet,
                        cloned,
                    );
                },
                .page => |*page| {
                    page.parent_style_sheet = parent_style_sheet;
                    page.parent_rule = parent_rule;
                },
                .position_try => |*position_try| {
                    position_try.parent_style_sheet = parent_style_sheet;
                    position_try.parent_rule = parent_rule;
                },
                .layer => |*layer| {
                    layer.parent_style_sheet = parent_style_sheet;
                    layer.parent_rule = parent_rule;
                    layer.css_rules = try cssRuleValuesCloneForReturn(
                        allocator,
                        layer.css_rules,
                        parent_style_sheet,
                        cloned,
                    );
                },
                .counter_style => |*counter_style| {
                    counter_style.parent_style_sheet = parent_style_sheet;
                    counter_style.parent_rule = parent_rule;
                },
                .property => |*property_rule| {
                    property_rule.parent_style_sheet = parent_style_sheet;
                    property_rule.parent_rule = parent_rule;
                },
                .charset => |*charset_rule| {
                    charset_rule.parent_style_sheet = parent_style_sheet;
                    charset_rule.parent_rule = parent_rule;
                },
                .import => |*import_rule| {
                    import_rule.parent_style_sheet = parent_style_sheet;
                    import_rule.parent_rule = parent_rule;
                },
                .namespace => |*namespace_rule| {
                    namespace_rule.parent_style_sheet = parent_style_sheet;
                    namespace_rule.parent_rule = parent_rule;
                },
            }

            break :blk Value{ .css_rule = cloned.* };
        },
        else => value,
    };
}

fn cssRuleSetParents(
    rule: *CssRuleState,
    parent_style_sheet: ?dom.NodeId,
    parent_rule: ?*const CssRuleState,
) void {
    switch (rule.*) {
        .style => |*style| {
            style.parent_style_sheet = parent_style_sheet;
            style.parent_rule = parent_rule;
        },
        .media => |*media| {
            media.parent_style_sheet = parent_style_sheet;
            media.parent_rule = parent_rule;
            cssRuleValuesAnnotate(media.css_rules, parent_style_sheet, rule);
        },
        .supports => |*supports| {
            supports.parent_style_sheet = parent_style_sheet;
            supports.parent_rule = parent_rule;
            cssRuleValuesAnnotate(supports.css_rules, parent_style_sheet, rule);
        },
        .document => |*document| {
            document.parent_style_sheet = parent_style_sheet;
            document.parent_rule = parent_rule;
            cssRuleValuesAnnotate(document.css_rules, parent_style_sheet, rule);
        },
        .container => |*container| {
            container.parent_style_sheet = parent_style_sheet;
            container.parent_rule = parent_rule;
            cssRuleValuesAnnotate(container.css_rules, parent_style_sheet, rule);
        },
        .starting_style => |*starting_style| {
            starting_style.parent_style_sheet = parent_style_sheet;
            starting_style.parent_rule = parent_rule;
            cssRuleValuesAnnotate(starting_style.css_rules, parent_style_sheet, rule);
        },
        .keyframes => |*keyframes| {
            keyframes.parent_style_sheet = parent_style_sheet;
            keyframes.parent_rule = parent_rule;
            cssRuleValuesAnnotate(keyframes.css_rules, parent_style_sheet, rule);
        },
        .keyframe => |*keyframe| {
            keyframe.parent_style_sheet = parent_style_sheet;
            keyframe.parent_rule = parent_rule;
        },
        .font_face => |*font_face| {
            font_face.parent_style_sheet = parent_style_sheet;
            font_face.parent_rule = parent_rule;
        },
        .font_feature_values => |*font_feature_values| {
            font_feature_values.parent_style_sheet = parent_style_sheet;
            font_feature_values.parent_rule = parent_rule;
        },
        .font_palette_values => |*font_palette_values| {
            font_palette_values.parent_style_sheet = parent_style_sheet;
            font_palette_values.parent_rule = parent_rule;
        },
        .color_profile => |*color_profile| {
            color_profile.parent_style_sheet = parent_style_sheet;
            color_profile.parent_rule = parent_rule;
        },
        .scope => |*scope| {
            scope.parent_style_sheet = parent_style_sheet;
            scope.parent_rule = parent_rule;
            cssRuleValuesAnnotate(scope.css_rules, parent_style_sheet, rule);
        },
        .page => |*page| {
            page.parent_style_sheet = parent_style_sheet;
            page.parent_rule = parent_rule;
        },
        .position_try => |*position_try| {
            position_try.parent_style_sheet = parent_style_sheet;
            position_try.parent_rule = parent_rule;
        },
        .layer => |*layer| {
            layer.parent_style_sheet = parent_style_sheet;
            layer.parent_rule = parent_rule;
            cssRuleValuesAnnotate(layer.css_rules, parent_style_sheet, rule);
        },
        .counter_style => |*counter_style| {
            counter_style.parent_style_sheet = parent_style_sheet;
            counter_style.parent_rule = parent_rule;
        },
        .property => |*property_rule| {
            property_rule.parent_style_sheet = parent_style_sheet;
            property_rule.parent_rule = parent_rule;
        },
        .charset => |*charset_rule| {
            charset_rule.parent_style_sheet = parent_style_sheet;
            charset_rule.parent_rule = parent_rule;
        },
        .import => |*import_rule| {
            import_rule.parent_style_sheet = parent_style_sheet;
            import_rule.parent_rule = parent_rule;
        },
        .namespace => |*namespace_rule| {
            namespace_rule.parent_style_sheet = parent_style_sheet;
            namespace_rule.parent_rule = parent_rule;
        },
    }
}

fn cssRuleParentStyleSheet(rule: CssRuleState) ?dom.NodeId {
    return switch (rule) {
        .style => |style| style.parent_style_sheet,
        .media => |media| media.parent_style_sheet,
        .supports => |supports| supports.parent_style_sheet,
        .document => |document| document.parent_style_sheet,
        .container => |container| container.parent_style_sheet,
        .starting_style => |starting_style| starting_style.parent_style_sheet,
        .keyframes => |keyframes| keyframes.parent_style_sheet,
        .keyframe => |keyframe| keyframe.parent_style_sheet,
        .font_face => |font_face| font_face.parent_style_sheet,
        .font_feature_values => |font_feature_values| font_feature_values.parent_style_sheet,
        .font_palette_values => |font_palette_values| font_palette_values.parent_style_sheet,
        .color_profile => |color_profile| color_profile.parent_style_sheet,
        .scope => |scope| scope.parent_style_sheet,
        .page => |page| page.parent_style_sheet,
        .position_try => |position_try| position_try.parent_style_sheet,
        .layer => |layer| layer.parent_style_sheet,
        .counter_style => |counter_style| counter_style.parent_style_sheet,
        .property => |property_rule| property_rule.parent_style_sheet,
        .charset => |charset_rule| charset_rule.parent_style_sheet,
        .import => |import_rule| import_rule.parent_style_sheet,
        .namespace => |namespace_rule| namespace_rule.parent_style_sheet,
    };
}

fn cssRuleParentRule(rule: CssRuleState) ?*const CssRuleState {
    return switch (rule) {
        .style => |style| style.parent_rule,
        .media => |media| media.parent_rule,
        .supports => |supports| supports.parent_rule,
        .document => |document| document.parent_rule,
        .container => |container| container.parent_rule,
        .starting_style => |starting_style| starting_style.parent_rule,
        .keyframes => |keyframes| keyframes.parent_rule,
        .keyframe => |keyframe| keyframe.parent_rule,
        .font_face => |font_face| font_face.parent_rule,
        .font_feature_values => |font_feature_values| font_feature_values.parent_rule,
        .font_palette_values => |font_palette_values| font_palette_values.parent_rule,
        .color_profile => |color_profile| color_profile.parent_rule,
        .scope => |scope| scope.parent_rule,
        .page => |page| page.parent_rule,
        .position_try => |position_try| position_try.parent_rule,
        .layer => |layer| layer.parent_rule,
        .counter_style => |counter_style| counter_style.parent_rule,
        .property => |property_rule| property_rule.parent_rule,
        .charset => |charset_rule| charset_rule.parent_rule,
        .import => |import_rule| import_rule.parent_rule,
        .namespace => |namespace_rule| namespace_rule.parent_rule,
    };
}

fn cssRuleCssText(rule: Value) []const u8 {
    return switch (rule) {
        .css_rule => |state| switch (state) {
            .style => |style| style.css_text,
            .media => |media| media.css_text,
            .supports => |supports| supports.css_text,
            .document => |document| document.css_text,
            .container => |container| container.css_text,
            .starting_style => |starting_style| starting_style.css_text,
            .keyframes => |keyframes| keyframes.css_text,
            .keyframe => |keyframe| keyframe.css_text,
            .font_face => |font_face| font_face.css_text,
            .font_feature_values => |font_feature_values| font_feature_values.css_text,
            .font_palette_values => |font_palette_values| font_palette_values.css_text,
            .color_profile => |color_profile| color_profile.css_text,
            .scope => |scope| scope.css_text,
            .page => |page| page.css_text,
            .position_try => |position_try| position_try.css_text,
            .layer => |layer| layer.css_text,
            .counter_style => |counter_style| counter_style.css_text,
            .property => |property_rule| property_rule.css_text,
            .charset => |charset_rule| charset_rule.css_text,
            .import => |import_rule| import_rule.css_text,
            .namespace => |namespace_rule| namespace_rule.css_text,
        },
        else => unreachable,
    };
}

fn cssRuleType(rule: CssRuleState) usize {
    return switch (rule) {
        .style => 1,
        .charset => 2,
        .import => 3,
        .media => 4,
        .font_face => 5,
        .page => 6,
        .keyframes => 7,
        .keyframe => 8,
        .namespace => 10,
        .counter_style => 11,
        .supports => 12,
        .font_feature_values => 14,
        else => 0,
    };
}

fn cssRuleListKeys(
    allocator: std.mem.Allocator,
    host: anytype,
    list: CssRuleListState,
) errors.Result(Value) {
    const rules = try cssRuleListCurrentValues(allocator, host, list);
    defer allocator.free(rules);
    var items = try allocator.alloc(Value, rules.len);
    errdefer allocator.free(items);
    for (rules, 0..) |_, index| {
        items[index] = Value{ .number = @floatFromInt(index) };
    }
    return try collectionIteratorFromValues(allocator, items);
}

fn cssRuleListValues(
    allocator: std.mem.Allocator,
    host: anytype,
    list: CssRuleListState,
) errors.Result(Value) {
    const rules = try cssRuleListCurrentValues(allocator, host, list);
    return try collectionIteratorFromValues(allocator, rules);
}

fn cssRuleListEntries(
    allocator: std.mem.Allocator,
    host: anytype,
    list: CssRuleListState,
) errors.Result(Value) {
    const rules = try cssRuleListCurrentValues(allocator, host, list);
    defer allocator.free(rules);
    return try collectionEntriesFromValues(allocator, rules);
}

fn cssRuleListForEach(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    list: CssRuleListState,
    args: []*Expr,
) errors.Result(Value) {
    const callback_expr: *Expr = switch (args.len) {
        1, 2 => args[0],
        else => return error.ScriptRuntime,
    };
    const this_arg_expr: ?*Expr = if (args.len == 2) args[1] else null;

    const callback_value = try evalExpr(allocator, host, bindings, callback_expr);
    const callback = switch (callback_value) {
        .function => |function| function,
        else => return error.ScriptRuntime,
    };

    if (this_arg_expr) |expr| {
        _ = try evalExpr(allocator, host, bindings, expr);
    }

    const rules = try cssRuleListCurrentValues(allocator, host, list);
    defer allocator.free(rules);

    for (rules, 0..) |rule, index| {
        const positional = [_]Value{
            rule,
            .{ .number = @floatFromInt(index) },
            .{ .css_rule_list = list },
        };
        var function_bindings = try functionBindings(allocator, callback, positional[0..]);
        defer function_bindings.deinit(allocator);

        const source_name = try std.fmt.allocPrint(allocator, "cssrulelist:forEach:{d}", .{index});
        defer allocator.free(source_name);

        try executeScriptSource(
            allocator,
            host,
            callback.body_source,
            source_name,
            function_bindings.items,
        );
    }

    return Value{ .undefined_value = {} };
}

fn styleSheetInsertRule(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    sheet_id: dom.NodeId,
    args: []*Expr,
) errors.Result(Value) {
    if (args.len != 1 and args.len != 2) return error.ScriptRuntime;

    const tag_name = host.domStore().tagNameForNode(sheet_id) orelse return error.ScriptRuntime;
    if (!std.mem.eql(u8, tag_name, "style")) return error.ScriptRuntime;

    const rule_text_value = try evalExpr(allocator, host, bindings, args[0]);
    const rule_text = try asString(allocator, rule_text_value);

    const current_rules = try cssRuleListCurrentValues(allocator, host, .{ .sheet = sheet_id });
    defer allocator.free(current_rules);

    const parsed_rules = try parseStyleSheetRuleValues(allocator, rule_text);
    defer allocator.free(parsed_rules);
    if (parsed_rules.len != 1) return error.ScriptRuntime;

    const index = if (args.len == 2) index_value_blk: {
        const index_value = try evalExpr(allocator, host, bindings, args[1]);
        break :index_value_blk try asNodeListIndex(index_value);
    } else 0;
    if (index > current_rules.len) return error.ScriptRuntime;

    var serialized: std.ArrayList(u8) = .empty;
    defer serialized.deinit(allocator);

    var wrote_any = false;
    for (current_rules, 0..) |rule, current_index| {
        if (current_index == index) {
            if (wrote_any) try serialized.append(allocator, '\n');
            try serialized.appendSlice(allocator, cssRuleCssText(parsed_rules[0]));
            wrote_any = true;
        }

        if (wrote_any) try serialized.append(allocator, '\n');
        try serialized.appendSlice(allocator, cssRuleCssText(rule));
        wrote_any = true;
    }

    if (index == current_rules.len) {
        if (wrote_any) try serialized.append(allocator, '\n');
        try serialized.appendSlice(allocator, cssRuleCssText(parsed_rules[0]));
    }

    host.domStoreMut().setTextContent(sheet_id, serialized.items) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return Value{ .number = @floatFromInt(index) };
}

fn styleSheetDeleteRule(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    sheet_id: dom.NodeId,
    args: []*Expr,
) errors.Result(Value) {
    if (args.len != 1) return error.ScriptRuntime;

    const tag_name = host.domStore().tagNameForNode(sheet_id) orelse return error.ScriptRuntime;
    if (!std.mem.eql(u8, tag_name, "style")) return error.ScriptRuntime;

    const index_value = try evalExpr(allocator, host, bindings, args[0]);
    const index = try asNodeListIndex(index_value);

    const current_rules = try cssRuleListCurrentValues(allocator, host, .{ .sheet = sheet_id });
    defer allocator.free(current_rules);
    if (index >= current_rules.len) return error.ScriptRuntime;

    var serialized: std.ArrayList(u8) = .empty;
    defer serialized.deinit(allocator);

    var wrote_any = false;
    for (current_rules, 0..) |rule, current_index| {
        if (current_index == index) continue;
        if (wrote_any) try serialized.append(allocator, '\n');
        try serialized.appendSlice(allocator, cssRuleCssText(rule));
        wrote_any = true;
    }

    host.domStoreMut().setTextContent(sheet_id, serialized.items) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return Value{ .undefined_value = {} };
}

fn parseStyleSheetRuleValues(
    allocator: std.mem.Allocator,
    source: []const u8,
) errors.Result([]Value) {
    var rules: std.ArrayList(Value) = .empty;
    errdefer rules.deinit(allocator);

    var pos: usize = 0;
    while (true) {
        pos = try skipCssTrivia(source, pos);
        if (pos >= source.len) break;
        try rules.append(allocator, try parseStyleSheetRuleValue(allocator, source, &pos));
    }

    return try allocator.dupe(Value, rules.items);
}

fn parseStyleSheetRuleValue(
    allocator: std.mem.Allocator,
    source: []const u8,
    pos: *usize,
) errors.Result(Value) {
    if (source[pos.*] == '@') {
        return try parseStyleSheetAtRuleValue(allocator, source, pos);
    }
    return try parseStyleSheetQualifiedRuleValue(allocator, source, pos);
}

fn parseStyleSheetQualifiedRuleValue(
    allocator: std.mem.Allocator,
    source: []const u8,
    pos: *usize,
) errors.Result(Value) {
    _ = allocator;
    const selector_start = pos.*;
    const block_start = try parseCssRuleBlockStart(source, pos.*);
    const selector_text = std.mem.trim(u8, source[selector_start..block_start], " \t\r\n\x0c");
    if (selector_text.len == 0) return error.ScriptRuntime;

    const block_end = try parseCssRuleBlockEnd(source, block_start + 1);
    const css_text = std.mem.trim(u8, source[selector_start..block_end], " \t\r\n\x0c");
    pos.* = block_end;

    return Value{ .css_rule = .{
        .style = .{
            .selector_text = selector_text,
            .css_text = css_text,
        },
    } };
}

fn parseStyleSheetAtRuleValue(
    allocator: std.mem.Allocator,
    source: []const u8,
    pos: *usize,
) errors.Result(Value) {
    const at_start = pos.*;
    pos.* += 1;

    const keyword_start = pos.*;
    while (pos.* < source.len and (isIdentifierContinueByte(source[pos.*]) or source[pos.*] == '-')) {
        pos.* += 1;
    }
    if (keyword_start == pos.*) return error.ScriptRuntime;

    const keyword = source[keyword_start..pos.*];
    if (std.ascii.eqlIgnoreCase(keyword, "charset")) {
        return try parseStyleSheetCharsetRuleValue(allocator, source, at_start, pos);
    }
    if (std.ascii.eqlIgnoreCase(keyword, "import")) {
        return try parseStyleSheetImportRuleValue(allocator, source, at_start, pos);
    }
    if (std.ascii.eqlIgnoreCase(keyword, "namespace")) {
        return try parseStyleSheetNamespaceRuleValue(allocator, source, at_start, pos);
    }
    if (std.ascii.eqlIgnoreCase(keyword, "layer")) {
        return try parseStyleSheetLayerRuleValue(allocator, source, at_start, pos);
    }
    if (std.ascii.eqlIgnoreCase(keyword, "property")) {
        return try parseStyleSheetPropertyRuleValue(allocator, source, at_start, pos);
    }
    if (std.ascii.eqlIgnoreCase(keyword, "scope")) {
        return try parseStyleSheetScopeRuleValue(allocator, source, at_start, pos);
    }
    if (std.ascii.eqlIgnoreCase(keyword, "color-profile")) {
        return try parseStyleSheetColorProfileRuleValue(allocator, source, at_start, pos);
    }

    const rule_kind = if (std.ascii.eqlIgnoreCase(keyword, "media"))
        CssBlockAtRuleKind.media
    else if (std.ascii.eqlIgnoreCase(keyword, "supports"))
        CssBlockAtRuleKind.supports
    else if (std.ascii.eqlIgnoreCase(keyword, "document"))
        CssBlockAtRuleKind.document
    else if (std.ascii.eqlIgnoreCase(keyword, "container"))
        CssBlockAtRuleKind.container
    else if (std.ascii.eqlIgnoreCase(keyword, "starting-style"))
        CssBlockAtRuleKind.starting_style
    else if (std.ascii.eqlIgnoreCase(keyword, "position-try"))
        CssBlockAtRuleKind.position_try
    else if (std.ascii.eqlIgnoreCase(keyword, "keyframes"))
        CssBlockAtRuleKind.keyframes
    else if (std.ascii.eqlIgnoreCase(keyword, "font-face"))
        CssBlockAtRuleKind.font_face
    else if (std.ascii.eqlIgnoreCase(keyword, "font-feature-values"))
        CssBlockAtRuleKind.font_feature_values
    else if (std.ascii.eqlIgnoreCase(keyword, "font-palette-values"))
        CssBlockAtRuleKind.font_palette_values
    else if (std.ascii.eqlIgnoreCase(keyword, "page"))
        CssBlockAtRuleKind.page
    else if (std.ascii.eqlIgnoreCase(keyword, "counter-style"))
        CssBlockAtRuleKind.counter_style
    else
        return error.ScriptRuntime;

    const block_start = try parseCssRuleBlockStart(source, pos.*);
    const header_text = std.mem.trim(u8, source[pos.*..block_start], " \t\r\n\x0c");
    if (rule_kind != .font_face and rule_kind != .font_feature_values and rule_kind != .page and rule_kind != .counter_style and rule_kind != .starting_style and rule_kind != .position_try and header_text.len == 0) return error.ScriptRuntime;
    if (rule_kind == .font_palette_values and !std.mem.startsWith(u8, header_text, "--")) return error.ScriptRuntime;

    const block_end = try parseCssRuleBlockEnd(source, block_start + 1);
    const css_text = std.mem.trim(u8, source[at_start..block_end], " \t\r\n\x0c");
    pos.* = block_end;

    return switch (rule_kind) {
        .media => Value{ .css_rule = .{
            .media = .{
                .condition_text = header_text,
                .css_text = css_text,
                .css_rules = try parseStyleSheetRuleValues(allocator, source[block_start + 1 .. block_end - 1]),
            },
        } },
        .supports => Value{ .css_rule = .{
            .supports = .{
                .condition_text = header_text,
                .css_text = css_text,
                .css_rules = try parseStyleSheetRuleValues(allocator, source[block_start + 1 .. block_end - 1]),
            },
        } },
        .document => Value{ .css_rule = .{
            .document = .{
                .condition_text = header_text,
                .css_text = css_text,
                .css_rules = try parseStyleSheetRuleValues(allocator, source[block_start + 1 .. block_end - 1]),
            },
        } },
        .container => Value{ .css_rule = .{
            .container = .{
                .condition_text = header_text,
                .css_text = css_text,
                .css_rules = try parseStyleSheetRuleValues(allocator, source[block_start + 1 .. block_end - 1]),
            },
        } },
        .starting_style => Value{ .css_rule = .{
            .starting_style = .{
                .css_text = css_text,
                .css_rules = try parseStyleSheetRuleValues(allocator, source[block_start + 1 .. block_end - 1]),
            },
        } },
        .position_try => Value{ .css_rule = .{
            .position_try = .{
                .name = header_text,
                .css_text = css_text,
            },
        } },
        .keyframes => Value{ .css_rule = .{
            .keyframes = .{
                .name = header_text,
                .css_text = css_text,
                .css_rules = try parseKeyframesRuleValues(allocator, source[block_start + 1 .. block_end - 1]),
            },
        } },
        .font_face => Value{ .css_rule = .{
            .font_face = .{
                .css_text = css_text,
            },
        } },
        .font_feature_values => Value{ .css_rule = .{
            .font_feature_values = .{
                .font_family = header_text,
                .css_text = css_text,
            },
        } },
        .font_palette_values => Value{ .css_rule = .{
            .font_palette_values = .{
                .name = header_text,
                .css_text = css_text,
            },
        } },
        .page => Value{ .css_rule = .{
            .page = .{
                .selector_text = header_text,
                .css_text = css_text,
            },
        } },
        .counter_style => Value{ .css_rule = .{
            .counter_style = .{
                .name = header_text,
                .css_text = css_text,
            },
        } },
    };
}

const CssBlockAtRuleKind = enum {
    media,
    supports,
    document,
    container,
    starting_style,
    position_try,
    keyframes,
    font_face,
    font_feature_values,
    font_palette_values,
    page,
    counter_style,
};

fn parseStyleSheetImportRuleValue(
    allocator: std.mem.Allocator,
    source: []const u8,
    at_start: usize,
    pos: *usize,
) errors.Result(Value) {
    _ = allocator;
    const statement_end = try parseCssRuleStatementEnd(source, pos.*);
    const prelude = std.mem.trim(u8, source[pos.*..statement_end], " \t\r\n\x0c");
    if (prelude.len == 0) return error.ScriptRuntime;

    const parsed = try parseCssImportPrelude(prelude);
    const css_text = std.mem.trim(u8, source[at_start .. statement_end + 1], " \t\r\n\x0c");
    pos.* = statement_end + 1;

    return Value{ .css_rule = .{
        .import = .{
            .href = parsed.href,
            .media_text = parsed.media_text,
            .supports_text = parsed.supports_text,
            .layer_name = parsed.layer_name,
            .css_text = css_text,
        },
    } };
}

fn parseStyleSheetCharsetRuleValue(
    allocator: std.mem.Allocator,
    source: []const u8,
    at_start: usize,
    pos: *usize,
) errors.Result(Value) {
    _ = allocator;
    const statement_end = try parseCssRuleStatementEnd(source, pos.*);
    const prelude = std.mem.trim(u8, source[pos.*..statement_end], " \t\r\n\x0c");
    if (prelude.len == 0) return error.ScriptRuntime;

    const parsed = try parseCssCharsetPrelude(prelude);
    const css_text = std.mem.trim(u8, source[at_start .. statement_end + 1], " \t\r\n\x0c");
    pos.* = statement_end + 1;

    return Value{ .css_rule = .{
        .charset = .{
            .encoding = parsed.encoding,
            .css_text = css_text,
        },
    } };
}

const CssImportPrelude = struct {
    href: []const u8,
    media_text: []const u8,
    supports_text: []const u8,
    layer_name: []const u8,
};

const CssCharsetPrelude = struct {
    encoding: []const u8,
};

fn parseCssCharsetPrelude(prelude: []const u8) errors.Result(CssCharsetPrelude) {
    var pos: usize = 0;
    pos = try skipCssTrivia(prelude, pos);
    if (pos >= prelude.len) return error.ScriptRuntime;
    if (prelude[pos] != '"' and prelude[pos] != '\'') return error.ScriptRuntime;

    const quote = prelude[pos];
    pos += 1;
    const start = pos;
    while (pos < prelude.len) : (pos += 1) {
        const byte = prelude[pos];
        if (byte == '\\') {
            if (pos + 1 >= prelude.len) return error.ScriptRuntime;
            pos += 1;
            continue;
        }
        if (byte == quote) {
            const encoding = prelude[start..pos];
            pos += 1;
            pos = try skipCssTrivia(prelude, pos);
            if (pos != prelude.len) return error.ScriptRuntime;
            return .{ .encoding = encoding };
        }
    }

    return error.ScriptRuntime;
}

const CssNamespacePrelude = struct {
    prefix: []const u8,
    namespace_uri: []const u8,
};

const CssPropertyPrelude = struct {
    syntax: []const u8,
    inherits: bool,
    initial_value: []const u8,
};

const CssScopePrelude = struct {
    start_text: ?[]const u8,
    end_text: ?[]const u8,
};

fn parseStyleSheetPropertyRuleValue(
    allocator: std.mem.Allocator,
    source: []const u8,
    at_start: usize,
    pos: *usize,
) errors.Result(Value) {
    const block_start = try parseCssRuleBlockStart(source, pos.*);
    const name_text = std.mem.trim(u8, source[pos.*..block_start], " \t\r\n\x0c");
    if (!std.mem.startsWith(u8, name_text, "--")) return error.ScriptRuntime;
    if (name_text.len == 2) return error.ScriptRuntime;

    const block_end = try parseCssRuleBlockEnd(source, block_start + 1);
    const css_text = std.mem.trim(u8, source[at_start..block_end], " \t\r\n\x0c");
    pos.* = block_end;

    const body = std.mem.trim(u8, source[block_start + 1 .. block_end - 1], " \t\r\n\x0c");
    const parsed = try parseCssPropertyPrelude(allocator, body);
    return Value{ .css_rule = .{
        .property = .{
            .name = name_text,
            .syntax = parsed.syntax,
            .inherits = parsed.inherits,
            .initial_value = parsed.initial_value,
            .css_text = css_text,
        },
    } };
}

fn parseCssPropertyPrelude(allocator: std.mem.Allocator, prelude: []const u8) errors.Result(CssPropertyPrelude) {
    var entries = try parseStyleDeclarations(allocator, prelude);
    defer freeStyleDeclarations(allocator, &entries);

    var syntax: ?[]const u8 = null;
    errdefer if (syntax) |value| allocator.free(value);
    var inherits: ?bool = null;
    var initial_value: []const u8 = "";
    errdefer if (initial_value.len != 0) allocator.free(initial_value);

    for (entries.items) |entry| {
        if (std.mem.eql(u8, entry.name, "syntax")) {
            syntax = try allocator.dupe(u8, entry.value);
            continue;
        }
        if (std.mem.eql(u8, entry.name, "inherits")) {
            if (std.ascii.eqlIgnoreCase(entry.value, "true")) {
                inherits = true;
            } else if (std.ascii.eqlIgnoreCase(entry.value, "false")) {
                inherits = false;
            } else {
                return error.ScriptRuntime;
            }
            continue;
        }
        if (std.mem.eql(u8, entry.name, "initial-value")) {
            initial_value = try allocator.dupe(u8, entry.value);
            continue;
        }
    }

    const syntax_value = syntax orelse return error.ScriptRuntime;
    const inherits_value = inherits orelse return error.ScriptRuntime;
    return .{
        .syntax = syntax_value,
        .inherits = inherits_value,
        .initial_value = initial_value,
    };
}

fn parseStyleSheetLayerRuleValue(
    allocator: std.mem.Allocator,
    source: []const u8,
    at_start: usize,
    pos: *usize,
) errors.Result(Value) {
    const prelude = try parseCssLayerPreludeKind(source, pos.*);
    return switch (prelude.kind) {
        .block => blk: {
            const block_start = prelude.delimiter;
            const name_text = std.mem.trim(u8, source[pos.*..block_start], " \t\r\n\x0c");
            const block_end = try parseCssRuleBlockEnd(source, block_start + 1);
            const css_text = std.mem.trim(u8, source[at_start..block_end], " \t\r\n\x0c");
            pos.* = block_end;

            break :blk Value{ .css_rule = .{
                .layer = .{
                    .name_text = name_text,
                    .css_text = css_text,
                    .css_rules = try parseStyleSheetRuleValues(allocator, source[block_start + 1 .. block_end - 1]),
                },
            } };
        },
        .statement => blk: {
            const statement_end = prelude.delimiter;
            const name_text = std.mem.trim(u8, source[pos.*..statement_end], " \t\r\n\x0c");
            if (name_text.len == 0) return error.ScriptRuntime;
            const css_text = std.mem.trim(u8, source[at_start .. statement_end + 1], " \t\r\n\x0c");
            pos.* = statement_end + 1;

            break :blk Value{ .css_rule = .{
                .layer = .{
                    .name_text = name_text,
                    .css_text = css_text,
                    .css_rules = try allocator.alloc(Value, 0),
                    .is_statement = true,
                },
            } };
        },
    };
}

const CssLayerPreludeKind = struct {
    kind: enum { block, statement },
    delimiter: usize,
};

fn parseCssLayerPreludeKind(source: []const u8, start: usize) errors.Result(CssLayerPreludeKind) {
    var pos = start;
    var paren_depth: usize = 0;
    var in_string: ?u8 = null;
    while (pos < source.len) {
        const byte = source[pos];
        if (in_string) |quote| {
            if (byte == '\\') {
                if (pos + 1 >= source.len) return error.ScriptRuntime;
                pos += 2;
                continue;
            }
            if (byte == quote) in_string = null;
            pos += 1;
            continue;
        }
        if (byte == '"' or byte == '\'') {
            in_string = byte;
            pos += 1;
            continue;
        }
        if (byte == '/' and pos + 1 < source.len and source[pos + 1] == '*') {
            pos = try skipCssComment(source, pos);
            continue;
        }
        if (byte == '(') {
            paren_depth += 1;
            pos += 1;
            continue;
        }
        if (byte == ')') {
            if (paren_depth == 0) return error.ScriptRuntime;
            paren_depth -= 1;
            pos += 1;
            continue;
        }
        if (paren_depth == 0) {
            if (byte == '{') return .{ .kind = .block, .delimiter = pos };
            if (byte == ';') return .{ .kind = .statement, .delimiter = pos };
            if (byte == '}') return error.ScriptRuntime;
        }
        pos += 1;
    }
    return error.ScriptRuntime;
}

fn parseStyleSheetScopeRuleValue(
    allocator: std.mem.Allocator,
    source: []const u8,
    at_start: usize,
    pos: *usize,
) errors.Result(Value) {
    const block_start = try parseCssRuleBlockStart(source, pos.*);
    const prelude = std.mem.trim(u8, source[pos.*..block_start], " \t\r\n\x0c");
    if (prelude.len == 0) return error.ScriptRuntime;

    const parsed = try parseCssScopePrelude(prelude);
    const block_end = try parseCssRuleBlockEnd(source, block_start + 1);
    const css_text = std.mem.trim(u8, source[at_start..block_end], " \t\r\n\x0c");
    pos.* = block_end;

    return Value{ .css_rule = .{
        .scope = .{
            .start_text = parsed.start_text,
            .end_text = parsed.end_text,
            .css_text = css_text,
            .css_rules = try parseStyleSheetRuleValues(allocator, source[block_start + 1 .. block_end - 1]),
        },
    } };
}

fn parseCssScopePrelude(prelude: []const u8) errors.Result(CssScopePrelude) {
    var pos: usize = 0;
    pos = try skipCssTrivia(prelude, pos);
    if (pos >= prelude.len) return error.ScriptRuntime;

    if (isCssScopeToKeyword(prelude, pos)) {
        pos += 2;
        pos = try skipCssTrivia(prelude, pos);
        const end_text = try parseCssScopePreludeGroup(prelude, &pos);
        pos = try skipCssTrivia(prelude, pos);
        if (pos != prelude.len) return error.ScriptRuntime;
        return .{
            .start_text = null,
            .end_text = end_text,
        };
    }

    const start_text = try parseCssScopePreludeGroup(prelude, &pos);
    pos = try skipCssTrivia(prelude, pos);
    if (pos >= prelude.len) {
        return .{
            .start_text = start_text,
            .end_text = null,
        };
    }
    if (!isCssScopeToKeyword(prelude, pos)) return error.ScriptRuntime;
    pos += 2;
    pos = try skipCssTrivia(prelude, pos);
    const end_text = try parseCssScopePreludeGroup(prelude, &pos);
    pos = try skipCssTrivia(prelude, pos);
    if (pos != prelude.len) return error.ScriptRuntime;
    return .{
        .start_text = start_text,
        .end_text = end_text,
    };
}

fn parseCssScopePreludeGroup(prelude: []const u8, pos: *usize) errors.Result([]const u8) {
    if (pos.* >= prelude.len or prelude[pos.*] != '(') return error.ScriptRuntime;

    const text_start = pos.* + 1;
    var scan = text_start;
    var depth: usize = 1;
    var in_string: ?u8 = null;
    while (scan < prelude.len) {
        const byte = prelude[scan];
        if (in_string) |quote| {
            if (byte == '\\') {
                if (scan + 1 >= prelude.len) return error.ScriptRuntime;
                scan += 2;
                continue;
            }
            if (byte == quote) in_string = null;
            scan += 1;
            continue;
        }
        if (byte == '"' or byte == '\'') {
            in_string = byte;
            scan += 1;
            continue;
        }
        if (byte == '/' and scan + 1 < prelude.len and prelude[scan + 1] == '*') {
            scan = try skipCssComment(prelude, scan);
            continue;
        }
        if (byte == '(') {
            depth += 1;
            scan += 1;
            continue;
        }
        if (byte == ')') {
            depth -= 1;
            scan += 1;
            if (depth == 0) {
                const text = std.mem.trim(u8, prelude[text_start .. scan - 1], " \t\r\n\x0c");
                if (text.len == 0) return error.ScriptRuntime;
                pos.* = scan;
                return text;
            }
            continue;
        }
        scan += 1;
    }

    return error.ScriptRuntime;
}

fn isCssScopeToKeyword(prelude: []const u8, pos: usize) bool {
    if (pos + 2 > prelude.len) return false;
    if (!std.ascii.eqlIgnoreCase(prelude[pos .. pos + 2], "to")) return false;
    return pos + 2 == prelude.len or !isIdentifierContinueByte(prelude[pos + 2]);
}

fn parseCssImportPrelude(prelude: []const u8) errors.Result(CssImportPrelude) {
    var pos: usize = 0;
    pos = try skipCssTrivia(prelude, pos);
    if (pos >= prelude.len) return error.ScriptRuntime;

    const href: []const u8 = href_blk: {
        if (prelude[pos] == '"' or prelude[pos] == '\'') {
            const quote = prelude[pos];
            pos += 1;
            const start = pos;
            while (pos < prelude.len) : (pos += 1) {
                const byte = prelude[pos];
                if (byte == '\\') {
                    if (pos + 1 >= prelude.len) return error.ScriptRuntime;
                    pos += 2;
                    continue;
                }
                if (byte == quote) {
                    break;
                }
            }
            if (pos >= prelude.len or prelude[pos] != quote) return error.ScriptRuntime;
            const value = prelude[start..pos];
            pos += 1;
            break :href_blk value;
        }

        if (pos + 4 <= prelude.len and std.ascii.eqlIgnoreCase(prelude[pos .. pos + 4], "url(")) {
            pos += 4;
            pos = try skipCssTrivia(prelude, pos);
            if (pos >= prelude.len) return error.ScriptRuntime;

            if (prelude[pos] == '"' or prelude[pos] == '\'') {
                const quote = prelude[pos];
                pos += 1;
                const start = pos;
                while (pos < prelude.len) : (pos += 1) {
                    const byte = prelude[pos];
                    if (byte == '\\') {
                        if (pos + 1 >= prelude.len) return error.ScriptRuntime;
                        pos += 2;
                        continue;
                    }
                    if (byte == quote) {
                        break;
                    }
                }
                if (pos >= prelude.len or prelude[pos] != quote) return error.ScriptRuntime;
                const value = prelude[start..pos];
                pos += 1;
                pos = try skipCssTrivia(prelude, pos);
                if (pos >= prelude.len or prelude[pos] != ')') return error.ScriptRuntime;
                pos += 1;
                break :href_blk value;
            }

            const start = pos;
            while (pos < prelude.len and prelude[pos] != ')') : (pos += 1) {
                if (prelude[pos] == '\\') {
                    if (pos + 1 >= prelude.len) return error.ScriptRuntime;
                    pos += 2;
                    continue;
                }
            }
            if (pos >= prelude.len or prelude[pos] != ')') return error.ScriptRuntime;
            const value = std.mem.trim(u8, prelude[start..pos], " \t\r\n\x0c");
            if (value.len == 0) return error.ScriptRuntime;
            pos += 1;
            break :href_blk value;
        }

        return error.ScriptRuntime;
    };

    var supports_text: []const u8 = "";
    var layer_name: []const u8 = "";
    var has_supports = false;
    var has_layer = false;

    while (true) {
        pos = try skipCssTrivia(prelude, pos);
        if (pos >= prelude.len) break;

        if (!has_supports and isCssImportKeyword(prelude, pos, "supports")) {
            pos += "supports".len;
            pos = try skipCssTrivia(prelude, pos);
            if (pos >= prelude.len) return error.ScriptRuntime;
            supports_text = try parseCssScopePreludeGroup(prelude, &pos);
            has_supports = true;
            continue;
        }

        if (!has_layer and isCssImportKeyword(prelude, pos, "layer")) {
            pos += "layer".len;
            pos = try skipCssTrivia(prelude, pos);
            if (pos < prelude.len and prelude[pos] == '(') {
                layer_name = try parseCssScopePreludeGroup(prelude, &pos);
            } else {
                layer_name = "";
            }
            has_layer = true;
            continue;
        }

        break;
    }

    pos = try skipCssTrivia(prelude, pos);
    const media_text = std.mem.trim(u8, prelude[pos..], " \t\r\n\x0c");
    return .{
        .href = href,
        .media_text = media_text,
        .supports_text = supports_text,
        .layer_name = layer_name,
    };
}

fn isCssImportKeyword(prelude: []const u8, pos: usize, keyword: []const u8) bool {
    if (pos + keyword.len > prelude.len) return false;
    if (!std.ascii.eqlIgnoreCase(prelude[pos .. pos + keyword.len], keyword)) return false;
    return pos + keyword.len == prelude.len or !isIdentifierContinueByte(prelude[pos + keyword.len]);
}

fn parseStyleSheetNamespaceRuleValue(
    allocator: std.mem.Allocator,
    source: []const u8,
    at_start: usize,
    pos: *usize,
) errors.Result(Value) {
    _ = allocator;
    const statement_end = try parseCssRuleStatementEnd(source, pos.*);
    const prelude = std.mem.trim(u8, source[pos.*..statement_end], " \t\r\n\x0c");
    if (prelude.len == 0) return error.ScriptRuntime;

    const parsed = try parseCssNamespacePrelude(prelude);
    const css_text = std.mem.trim(u8, source[at_start .. statement_end + 1], " \t\r\n\x0c");
    pos.* = statement_end + 1;

    return Value{ .css_rule = .{
        .namespace = .{
            .prefix = parsed.prefix,
            .namespace_uri = parsed.namespace_uri,
            .css_text = css_text,
        },
    } };
}

fn parseStyleSheetFontPaletteValuesRuleValue(
    allocator: std.mem.Allocator,
    source: []const u8,
    at_start: usize,
    pos: *usize,
) errors.Result(Value) {
    _ = allocator;
    const block_start = try parseCssRuleBlockStart(source, pos.*);
    const name = std.mem.trim(u8, source[pos.*..block_start], " \t\r\n\x0c");
    if (name.len == 0 or !std.mem.startsWith(u8, name, "--")) return error.ScriptRuntime;

    const block_end = try parseCssRuleBlockEnd(source, block_start + 1);
    const css_text = std.mem.trim(u8, source[at_start..block_end], " \t\r\n\x0c");
    pos.* = block_end;

    return Value{ .css_rule = .{
        .font_palette_values = .{
            .name = name,
            .css_text = css_text,
        },
    } };
}

fn parseStyleSheetColorProfileRuleValue(
    allocator: std.mem.Allocator,
    source: []const u8,
    at_start: usize,
    pos: *usize,
) errors.Result(Value) {
    const block_start = try parseCssRuleBlockStart(source, pos.*);
    const name = std.mem.trim(u8, source[pos.*..block_start], " \t\r\n\x0c");
    if (name.len == 0) return error.ScriptRuntime;
    if (!isDashedIdent(name) and !std.ascii.eqlIgnoreCase(name, "device-cmyk")) return error.ScriptRuntime;

    const block_end = try parseCssRuleBlockEnd(source, block_start + 1);
    const css_text = std.mem.trim(u8, source[at_start..block_end], " \t\r\n\x0c");
    pos.* = block_end;

    const descriptors = std.mem.trim(u8, source[block_start + 1 .. block_end - 1], " \t\r\n\x0c");
    var entries = try parseStyleDeclarations(allocator, descriptors);
    defer freeStyleDeclarations(allocator, &entries);

    const src = try colorProfileDescriptorText(allocator, entries.items, "src");
    const rendering_intent = try colorProfileDescriptorText(allocator, entries.items, "rendering-intent");
    const components = try colorProfileDescriptorText(allocator, entries.items, "components");

    return Value{ .css_rule = .{
        .color_profile = .{
            .name = name,
            .src = src,
            .rendering_intent = rendering_intent,
            .components = components,
            .css_text = css_text,
        },
    } };
}

fn colorProfileDescriptorText(
    allocator: std.mem.Allocator,
    entries: []const StylePropertyEntry,
    name: []const u8,
) errors.Result([]const u8) {
    for (entries) |entry| {
        if (std.mem.eql(u8, entry.name, name)) {
            return allocator.dupe(u8, entry.value);
        }
    }
    return allocator.dupe(u8, "");
}

fn isDashedIdent(text: []const u8) bool {
    if (!std.mem.startsWith(u8, text, "--")) return false;
    if (text.len == 2) return false;
    for (text[2..]) |byte| {
        if (!isIdentifierContinueByte(byte)) return false;
    }
    return true;
}

fn parseCssNamespacePrelude(prelude: []const u8) errors.Result(CssNamespacePrelude) {
    var pos: usize = 0;
    pos = try skipCssTrivia(prelude, pos);
    if (pos >= prelude.len) return error.ScriptRuntime;

    var prefix: []const u8 = "";
    const prefix_start = pos;
    if (isIdentifierStartByte(prelude[pos])) {
        pos += 1;
        while (pos < prelude.len and isIdentifierContinueByte(prelude[pos])) {
            pos += 1;
        }
        const after_prefix = try skipCssTrivia(prelude, pos);
        if (after_prefix > pos) {
            prefix = prelude[prefix_start..pos];
            pos = after_prefix;
        } else {
            pos = prefix_start;
        }
    }

    pos = try skipCssTrivia(prelude, pos);
    if (pos >= prelude.len) return error.ScriptRuntime;

    const namespace_uri: []const u8 = href_blk: {
        if (prelude[pos] == '"' or prelude[pos] == '\'') {
            const quote = prelude[pos];
            pos += 1;
            const start = pos;
            while (pos < prelude.len) : (pos += 1) {
                const byte = prelude[pos];
                if (byte == '\\') {
                    if (pos + 1 >= prelude.len) return error.ScriptRuntime;
                    pos += 2;
                    continue;
                }
                if (byte == quote) {
                    break;
                }
            }
            if (pos >= prelude.len or prelude[pos] != quote) return error.ScriptRuntime;
            const value = prelude[start..pos];
            pos += 1;
            break :href_blk value;
        }

        if (pos + 4 <= prelude.len and std.ascii.eqlIgnoreCase(prelude[pos .. pos + 4], "url(")) {
            pos += 4;
            pos = try skipCssTrivia(prelude, pos);
            if (pos >= prelude.len) return error.ScriptRuntime;

            if (prelude[pos] == '"' or prelude[pos] == '\'') {
                const quote = prelude[pos];
                pos += 1;
                const start = pos;
                while (pos < prelude.len) : (pos += 1) {
                    const byte = prelude[pos];
                    if (byte == '\\') {
                        if (pos + 1 >= prelude.len) return error.ScriptRuntime;
                        pos += 2;
                        continue;
                    }
                    if (byte == quote) {
                        break;
                    }
                }
                if (pos >= prelude.len or prelude[pos] != quote) return error.ScriptRuntime;
                const value = prelude[start..pos];
                pos += 1;
                pos = try skipCssTrivia(prelude, pos);
                if (pos >= prelude.len or prelude[pos] != ')') return error.ScriptRuntime;
                pos += 1;
                break :href_blk value;
            }

            const start = pos;
            while (pos < prelude.len and prelude[pos] != ')') : (pos += 1) {
                if (prelude[pos] == '\\') {
                    if (pos + 1 >= prelude.len) return error.ScriptRuntime;
                    pos += 2;
                    continue;
                }
            }
            if (pos >= prelude.len or prelude[pos] != ')') return error.ScriptRuntime;
            const value = std.mem.trim(u8, prelude[start..pos], " \t\r\n\x0c");
            if (value.len == 0) return error.ScriptRuntime;
            pos += 1;
            break :href_blk value;
        }

        return error.ScriptRuntime;
    };

    pos = try skipCssTrivia(prelude, pos);
    if (pos != prelude.len) return error.ScriptRuntime;

    return .{
        .prefix = prefix,
        .namespace_uri = namespace_uri,
    };
}

fn parseCssRuleStatementEnd(
    source: []const u8,
    start: usize,
) errors.Result(usize) {
    var pos = start;
    var paren_depth: usize = 0;
    var in_string: ?u8 = null;
    while (pos < source.len) {
        const byte = source[pos];
        if (in_string) |quote| {
            if (byte == '\\') {
                if (pos + 1 >= source.len) return error.ScriptRuntime;
                pos += 2;
                continue;
            }
            if (byte == quote) in_string = null;
            pos += 1;
            continue;
        }
        if (byte == '"' or byte == '\'') {
            in_string = byte;
            pos += 1;
            continue;
        }
        if (byte == '/' and pos + 1 < source.len and source[pos + 1] == '*') {
            pos = try skipCssComment(source, pos);
            continue;
        }
        if (byte == '(') {
            paren_depth += 1;
            pos += 1;
            continue;
        }
        if (byte == ')') {
            if (paren_depth == 0) return error.ScriptRuntime;
            paren_depth -= 1;
            pos += 1;
            continue;
        }
        if (byte == ';' and paren_depth == 0) return pos;
        if (byte == '{' or byte == '}') return error.ScriptRuntime;
        pos += 1;
    }
    return error.ScriptRuntime;
}

fn parseKeyframesRuleValues(
    allocator: std.mem.Allocator,
    source: []const u8,
) errors.Result([]Value) {
    var rules: std.ArrayList(Value) = .empty;
    errdefer rules.deinit(allocator);

    var pos: usize = 0;
    while (true) {
        pos = try skipCssTrivia(source, pos);
        if (pos >= source.len) break;
        try rules.append(allocator, try parseKeyframesRuleValue(allocator, source, &pos));
    }

    return try allocator.dupe(Value, rules.items);
}

fn parseKeyframesRuleValue(
    allocator: std.mem.Allocator,
    source: []const u8,
    pos: *usize,
) errors.Result(Value) {
    _ = allocator;
    if (source[pos.*] == '@') return error.ScriptRuntime;

    const key_start = pos.*;
    const block_start = try parseCssRuleBlockStart(source, pos.*);
    const key_text = std.mem.trim(u8, source[key_start..block_start], " \t\r\n\x0c");
    if (key_text.len == 0) return error.ScriptRuntime;

    const block_end = try parseCssRuleBlockEnd(source, block_start + 1);
    const css_text = std.mem.trim(u8, source[key_start..block_end], " \t\r\n\x0c");
    pos.* = block_end;

    return Value{ .css_rule = .{
        .keyframe = .{
            .key_text = key_text,
            .css_text = css_text,
        },
    } };
}

fn parseCssRuleBlockStart(
    source: []const u8,
    start: usize,
) errors.Result(usize) {
    var pos = start;
    var in_string: ?u8 = null;
    while (pos < source.len) {
        const byte = source[pos];
        if (in_string) |quote| {
            if (byte == '\\') {
                if (pos + 1 >= source.len) return error.ScriptRuntime;
                pos += 2;
                continue;
            }
            if (byte == quote) in_string = null;
            pos += 1;
            continue;
        }
        if (byte == '"' or byte == '\'') {
            in_string = byte;
            pos += 1;
            continue;
        }
        if (byte == '/' and pos + 1 < source.len and source[pos + 1] == '*') {
            pos = try skipCssComment(source, pos);
            continue;
        }
        if (byte == '{') return pos;
        if (byte == '}') return error.ScriptRuntime;
        pos += 1;
    }
    return error.ScriptRuntime;
}

fn parseCssRuleBlockEnd(
    source: []const u8,
    start: usize,
) errors.Result(usize) {
    var pos = start;
    var depth: usize = 1;
    var in_string: ?u8 = null;
    while (pos < source.len) {
        const byte = source[pos];
        if (in_string) |quote| {
            if (byte == '\\') {
                if (pos + 1 >= source.len) return error.ScriptRuntime;
                pos += 2;
                continue;
            }
            if (byte == quote) in_string = null;
            pos += 1;
            continue;
        }
        if (byte == '"' or byte == '\'') {
            in_string = byte;
            pos += 1;
            continue;
        }
        if (byte == '/' and pos + 1 < source.len and source[pos + 1] == '*') {
            pos = try skipCssComment(source, pos);
            continue;
        }
        if (byte == '{') {
            depth += 1;
            pos += 1;
            continue;
        }
        if (byte == '}') {
            depth -= 1;
            pos += 1;
            if (depth == 0) return pos;
            continue;
        }
        pos += 1;
    }
    return error.ScriptRuntime;
}

fn skipCssTrivia(
    source: []const u8,
    start: usize,
) errors.Result(usize) {
    var pos = start;
    while (pos < source.len) {
        if (isCssWhitespace(source[pos]) or source[pos] == ';') {
            pos += 1;
            continue;
        }
        if (source[pos] == '/' and pos + 1 < source.len and source[pos + 1] == '*') {
            pos = try skipCssComment(source, pos);
            continue;
        }
        break;
    }
    return pos;
}

fn skipCssComment(
    source: []const u8,
    start: usize,
) errors.Result(usize) {
    if (start + 1 >= source.len or source[start] != '/' or source[start + 1] != '*') return error.ScriptRuntime;
    var pos = start + 2;
    while (pos + 1 < source.len) : (pos += 1) {
        if (source[pos] == '*' and source[pos + 1] == '/') {
            return pos + 2;
        }
    }
    return error.ScriptRuntime;
}

fn isCssWhitespace(byte: u8) bool {
    return switch (byte) {
        ' ', '\t', '\n', '\r', 0x0c => true,
        else => false,
    };
}

fn appendAllElementIds(
    allocator: std.mem.Allocator,
    host: anytype,
    node_id: dom.NodeId,
    filtered: *std.ArrayList(dom.NodeId),
) errors.Result(void) {
    const children = host.domStore().childIds(node_id);
    for (children) |child_id| {
        if (host.domStore().tagNameForNode(child_id) != null) {
            try filtered.append(allocator, child_id);
        }
        try appendAllElementIds(allocator, host, child_id, filtered);
    }
}

fn appendStyleSheetIds(
    allocator: std.mem.Allocator,
    host: anytype,
    node_id: dom.NodeId,
    filtered: *std.ArrayList(dom.NodeId),
) errors.Result(void) {
    const children = host.domStore().childIds(node_id);
    for (children) |child_id| {
        const tag_name = host.domStore().tagNameForNode(child_id) orelse {
            try appendStyleSheetIds(allocator, host, child_id, filtered);
            continue;
        };

        if (std.mem.eql(u8, tag_name, "style") or try isStyleSheetLinkElement(host, child_id)) {
            try filtered.append(allocator, child_id);
        }
        try appendStyleSheetIds(allocator, host, child_id, filtered);
    }
}

fn isStyleSheetLinkElement(host: anytype, element_id: dom.NodeId) errors.Result(bool) {
    const tag_name = host.domStore().tagNameForNode(element_id) orelse return false;
    if (!std.mem.eql(u8, tag_name, "link")) return false;

    const rel_value = (try host.domStore().getAttribute(element_id, "rel")) orelse return false;
    var rels = std.mem.tokenizeAny(u8, rel_value, " \t\r\n\x0c");
    while (rels.next()) |rel| {
        if (std.ascii.eqlIgnoreCase(rel, "stylesheet")) return true;
    }
    return false;
}

fn isLabelableElement(host: anytype, element_id: dom.NodeId) errors.Result(bool) {
    const tag_name = host.domStore().tagNameForNode(element_id) orelse return false;
    if (std.mem.eql(u8, tag_name, "input")) {
        const input_type = (try host.domStore().getAttribute(element_id, "type")) orelse "";
        return !std.ascii.eqlIgnoreCase(input_type, "hidden");
    }

    return std.mem.eql(u8, tag_name, "button") or
        std.mem.eql(u8, tag_name, "fieldset") or
        std.mem.eql(u8, tag_name, "meter") or
        std.mem.eql(u8, tag_name, "output") or
        std.mem.eql(u8, tag_name, "progress") or
        std.mem.eql(u8, tag_name, "select") or
        std.mem.eql(u8, tag_name, "textarea");
}

fn elementSupportsPlaceholderProperty(host: anytype, element_id: dom.NodeId) errors.Result(bool) {
    const tag_name = host.domStore().tagNameForNode(element_id) orelse return false;
    return std.mem.eql(u8, tag_name, "input") or
        std.mem.eql(u8, tag_name, "textarea");
}

fn elementSupportsDisabledProperty(host: anytype, element_id: dom.NodeId) errors.Result(bool) {
    const tag_name = host.domStore().tagNameForNode(element_id) orelse return false;
    return std.mem.eql(u8, tag_name, "button") or
        std.mem.eql(u8, tag_name, "fieldset") or
        std.mem.eql(u8, tag_name, "input") or
        std.mem.eql(u8, tag_name, "optgroup") or
        std.mem.eql(u8, tag_name, "option") or
        std.mem.eql(u8, tag_name, "select") or
        std.mem.eql(u8, tag_name, "textarea");
}

fn elementSupportsRequiredProperty(host: anytype, element_id: dom.NodeId) errors.Result(bool) {
    const tag_name = host.domStore().tagNameForNode(element_id) orelse return false;
    return std.mem.eql(u8, tag_name, "input") or
        std.mem.eql(u8, tag_name, "select") or
        std.mem.eql(u8, tag_name, "textarea");
}

fn isTabIndexFocusableElement(host: anytype, element_id: dom.NodeId) errors.Result(bool) {
    const tag_name = host.domStore().tagNameForNode(element_id) orelse return false;
    if (std.mem.eql(u8, tag_name, "input")) {
        const input_type = (try host.domStore().getAttribute(element_id, "type")) orelse "";
        return !std.ascii.eqlIgnoreCase(input_type, "hidden");
    }

    if (std.mem.eql(u8, tag_name, "button") or
        std.mem.eql(u8, tag_name, "select") or
        std.mem.eql(u8, tag_name, "textarea") or
        std.mem.eql(u8, tag_name, "iframe") or
        std.mem.eql(u8, tag_name, "object") or
        std.mem.eql(u8, tag_name, "embed") or
        std.mem.eql(u8, tag_name, "summary"))
    {
        return true;
    }

    if (std.mem.eql(u8, tag_name, "a") or std.mem.eql(u8, tag_name, "area")) {
        return (try host.domStore().getAttribute(element_id, "href")) != null;
    }

    return false;
}

fn isDescendantOf(host: anytype, node_id: dom.NodeId, ancestor_id: dom.NodeId) bool {
    var current = host.domStore().nodeAt(node_id) orelse return false;
    var parent_id = current.parent;
    while (parent_id) |id| {
        if (id.index == ancestor_id.index and id.generation == ancestor_id.generation) {
            return true;
        }
        current = host.domStore().nodeAt(id) orelse return false;
        parent_id = current.parent;
    }
    return false;
}

fn elementTabIndexValue(
    allocator: std.mem.Allocator,
    host: anytype,
    element_id: dom.NodeId,
) errors.Result(i64) {
    const attr = try host.domStore().getAttribute(element_id, "tabindex");
    if (attr) |text| {
        const trimmed = std.mem.trim(u8, text, " \t\r\n");
        if (trimmed.len == 0) return -1;
        return std.fmt.parseInt(i64, trimmed, 10) catch return -1;
    }
    if (try isTabIndexFocusableElement(host, element_id)) return 0;
    _ = allocator;
    return -1;
}

fn elementContentEditableText(
    allocator: std.mem.Allocator,
    host: anytype,
    element_id: dom.NodeId,
) errors.Result([]const u8) {
    const attr = try host.domStore().getAttribute(element_id, "contenteditable");
    if (attr) |text| {
        if (text.len == 0) return "true";
        return text;
    }
    _ = allocator;
    return "inherit";
}

fn elementIsContentEditable(
    host: anytype,
    element_id: dom.NodeId,
) errors.Result(bool) {
    var current_id: ?dom.NodeId = element_id;
    while (current_id) |id| {
        const node = host.domStore().nodeAt(id) orelse return false;
        const attr = try host.domStore().getAttribute(id, "contenteditable");
        if (attr) |text| {
            if (text.len == 0 or std.ascii.eqlIgnoreCase(text, "true")) return true;
            if (std.ascii.eqlIgnoreCase(text, "false")) return false;
            if (std.ascii.eqlIgnoreCase(text, "inherit")) {
                current_id = node.parent;
                continue;
            }
            return true;
        }
        current_id = node.parent;
    }
    return false;
}

fn elementTranslateValue(host: anytype, element_id: dom.NodeId) errors.Result(bool) {
    var current_id: ?dom.NodeId = element_id;
    while (current_id) |id| {
        const node = host.domStore().nodeAt(id) orelse return true;
        const attr = try host.domStore().getAttribute(id, "translate");
        if (attr) |text| {
            if (std.ascii.eqlIgnoreCase(text, "no")) return false;
            if (std.ascii.eqlIgnoreCase(text, "yes")) return true;
            return true;
        }
        current_id = node.parent;
    }
    return true;
}

fn elementSpellcheckValue(host: anytype, element_id: dom.NodeId) errors.Result(bool) {
    var current_id: ?dom.NodeId = element_id;
    while (current_id) |id| {
        const node = host.domStore().nodeAt(id) orelse return true;
        const attr = try host.domStore().getAttribute(id, "spellcheck");
        if (attr) |text| {
            if (std.ascii.eqlIgnoreCase(text, "false")) return false;
            if (std.ascii.eqlIgnoreCase(text, "true")) return true;
            return true;
        }
        current_id = node.parent;
    }
    return true;
}

fn elementLabelsIds(
    allocator: std.mem.Allocator,
    host: anytype,
    element_id: dom.NodeId,
) errors.Result([]dom.NodeId) {
    if (!try isLabelableElement(host, element_id)) {
        return error.ScriptRuntime;
    }

    const target_id = (try host.domStore().getAttribute(element_id, "id")) orelse "";
    const labels = host.domStore().select(allocator, "label") catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    defer allocator.free(labels);

    var filtered: std.ArrayList(dom.NodeId) = .empty;
    defer filtered.deinit(allocator);

    for (labels) |label_id| {
        const explicit = if (target_id.len == 0) false else blk: {
            const for_value = (try host.domStore().getAttribute(label_id, "for")) orelse "";
            break :blk std.mem.eql(u8, for_value, target_id);
        };
        const implicit = isDescendantOf(host, element_id, label_id);
        if (explicit or implicit) {
            try filtered.append(allocator, label_id);
        }
    }

    return try allocator.dupe(dom.NodeId, filtered.items);
}

fn mapAreasItems(
    allocator: std.mem.Allocator,
    host: anytype,
    map_id: dom.NodeId,
) errors.Result([]dom.NodeId) {
    return host.domStore().selectWithin(allocator, map_id, "area") catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
}

fn tableBodiesItems(
    allocator: std.mem.Allocator,
    host: anytype,
    table_id: dom.NodeId,
) errors.Result([]dom.NodeId) {
    const children = host.domStore().childIds(table_id);

    var filtered: std.ArrayList(dom.NodeId) = .empty;
    defer filtered.deinit(allocator);

    for (children) |child_id| {
        const tag_name = host.domStore().tagNameForNode(child_id) orelse continue;
        if (std.mem.eql(u8, tag_name, "tbody")) {
            try filtered.append(allocator, child_id);
        }
    }

    return try allocator.dupe(dom.NodeId, filtered.items);
}

fn tableRowsItems(
    allocator: std.mem.Allocator,
    host: anytype,
    table_id: dom.NodeId,
) errors.Result([]dom.NodeId) {
    const tag_name = host.domStore().tagNameForNode(table_id) orelse return error.ScriptRuntime;

    var filtered: std.ArrayList(dom.NodeId) = .empty;
    defer filtered.deinit(allocator);

    const children = host.domStore().childIds(table_id);
    if (std.mem.eql(u8, tag_name, "table")) {
        for (children) |child_id| {
            const child_tag_name = host.domStore().tagNameForNode(child_id) orelse continue;
            if (std.mem.eql(u8, child_tag_name, "tr")) {
                try filtered.append(allocator, child_id);
                continue;
            }

            if (std.mem.eql(u8, child_tag_name, "thead") or
                std.mem.eql(u8, child_tag_name, "tbody") or
                std.mem.eql(u8, child_tag_name, "tfoot"))
            {
                for (host.domStore().childIds(child_id)) |row_id| {
                    const row_tag_name = host.domStore().tagNameForNode(row_id) orelse continue;
                    if (std.mem.eql(u8, row_tag_name, "tr")) {
                        try filtered.append(allocator, row_id);
                    }
                }
            }
        }
    } else if (std.mem.eql(u8, tag_name, "thead") or
        std.mem.eql(u8, tag_name, "tbody") or
        std.mem.eql(u8, tag_name, "tfoot"))
    {
        for (children) |child_id| {
            const child_tag_name = host.domStore().tagNameForNode(child_id) orelse continue;
            if (std.mem.eql(u8, child_tag_name, "tr")) {
                try filtered.append(allocator, child_id);
            }
        }
    } else {
        return error.ScriptRuntime;
    }

    return try allocator.dupe(dom.NodeId, filtered.items);
}

fn rowCellsItems(
    allocator: std.mem.Allocator,
    host: anytype,
    row_id: dom.NodeId,
) errors.Result([]dom.NodeId) {
    const tag_name = host.domStore().tagNameForNode(row_id) orelse return error.ScriptRuntime;
    if (!std.mem.eql(u8, tag_name, "tr")) {
        return error.ScriptRuntime;
    }

    var filtered: std.ArrayList(dom.NodeId) = .empty;
    defer filtered.deinit(allocator);

    for (host.domStore().childIds(row_id)) |child_id| {
        const child_tag_name = host.domStore().tagNameForNode(child_id) orelse continue;
        if (std.mem.eql(u8, child_tag_name, "td") or std.mem.eql(u8, child_tag_name, "th")) {
            try filtered.append(allocator, child_id);
        }
    }

    return try allocator.dupe(dom.NodeId, filtered.items);
}

fn selectedOptionsItems(
    allocator: std.mem.Allocator,
    host: anytype,
    select_id: dom.NodeId,
) errors.Result([]dom.NodeId) {
    const items = try selectOptionsItems(allocator, host, select_id);
    defer allocator.free(items);

    var match_count: usize = 0;
    for (items) |item| {
        const present = host.domStore().hasAttribute(item, "selected") catch |err| switch (err) {
            error.OutOfMemory => return error.OutOfMemory,
            else => return error.ScriptRuntime,
        };
        if (present) {
            match_count += 1;
        }
    }

    const matches = try allocator.alloc(dom.NodeId, match_count);
    var write_index: usize = 0;
    for (items) |item| {
        const present = host.domStore().hasAttribute(item, "selected") catch |err| switch (err) {
            error.OutOfMemory => return error.OutOfMemory,
            else => return error.ScriptRuntime,
        };
        if (present) {
            matches[write_index] = item;
            write_index += 1;
        }
    }

    return matches;
}

fn radioNodeListCurrentIds(
    allocator: std.mem.Allocator,
    host: anytype,
    list: RadioNodeList,
) errors.Result([]dom.NodeId) {
    const items = try formElementsItems(allocator, host, list.root);
    defer allocator.free(items);

    var match_count: usize = 0;
    for (items) |item| {
        const item_name = (try host.domStore().getAttribute(item, "name")) orelse "";
        if (std.mem.eql(u8, item_name, list.name)) {
            match_count += 1;
        }
    }

    const matches = try allocator.alloc(dom.NodeId, match_count);
    var write_index: usize = 0;
    for (items) |item| {
        const item_name = (try host.domStore().getAttribute(item, "name")) orelse "";
        if (std.mem.eql(u8, item_name, list.name)) {
            matches[write_index] = item;
            write_index += 1;
        }
    }

    return matches;
}

fn radioNodeListValue(
    allocator: std.mem.Allocator,
    host: anytype,
    list: RadioNodeList,
) errors.Result([]u8) {
    const items = try radioNodeListCurrentIds(allocator, host, list);
    defer allocator.free(items);

    for (items) |item| {
        const node = host.domStore().nodeAt(item) orelse continue;
        const element = switch (node.kind) {
            .element => |element| element,
            else => continue,
        };
        if (!std.mem.eql(u8, element.tag_name, "input")) continue;
        const input_type = (try host.domStore().getAttribute(item, "type")) orelse "text";
        if (!std.mem.eql(u8, input_type, "radio")) continue;
        if (host.domStore().checkedForNode(item) orelse false) {
            const current_value = (try host.domStore().getAttribute(item, "value")) orelse "on";
            return allocator.dupe(u8, current_value);
        }
    }

    return "";
}

fn radioNodeListSetValue(
    allocator: std.mem.Allocator,
    host: anytype,
    list: RadioNodeList,
    value: []const u8,
) errors.Result(void) {
    const items = try radioNodeListCurrentIds(allocator, host, list);
    defer allocator.free(items);

    var selected: ?dom.NodeId = null;
    for (items) |item| {
        const node = host.domStore().nodeAt(item) orelse continue;
        const element = switch (node.kind) {
            .element => |element| element,
            else => continue,
        };
        if (!std.mem.eql(u8, element.tag_name, "input")) continue;
        const input_type = (try host.domStore().getAttribute(item, "type")) orelse "text";
        if (!std.mem.eql(u8, input_type, "radio")) continue;

        const current_value = (try host.domStore().getAttribute(item, "value")) orelse "on";
        const matches = if (std.mem.eql(u8, value, "on"))
            std.mem.eql(u8, current_value, "on")
        else
            std.mem.eql(u8, current_value, value);
        if (matches) {
            selected = item;
            break;
        }
    }

    for (items) |item| {
        const node = host.domStore().nodeAt(item) orelse continue;
        const element = switch (node.kind) {
            .element => |element| element,
            else => continue,
        };
        if (!std.mem.eql(u8, element.tag_name, "input")) continue;
        const input_type = (try host.domStore().getAttribute(item, "type")) orelse "text";
        if (!std.mem.eql(u8, input_type, "radio")) continue;
        const checked = if (selected) |chosen| item.index == chosen.index else false;
        try host.domStoreMut().setFormControlChecked(item, checked);
    }

    return;
}

fn radioNodeListKeys(
    allocator: std.mem.Allocator,
    host: anytype,
    list: RadioNodeList,
) errors.Result(Value) {
    const items = try radioNodeListCurrentIds(allocator, host, list);
    defer allocator.free(items);
    return try collectionIteratorFromNodeIds(allocator, items, .element, true);
}

fn radioNodeListValues(
    allocator: std.mem.Allocator,
    host: anytype,
    list: RadioNodeList,
) errors.Result(Value) {
    const items = try radioNodeListCurrentIds(allocator, host, list);
    defer allocator.free(items);
    return try collectionIteratorFromNodeIds(allocator, items, .element, false);
}

fn radioNodeListEntries(
    allocator: std.mem.Allocator,
    host: anytype,
    list: RadioNodeList,
) errors.Result(Value) {
    const items = try radioNodeListCurrentIds(allocator, host, list);
    defer allocator.free(items);
    return try collectionEntriesFromNodeIds(allocator, items, .element);
}

fn radioNodeListForEach(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    list: RadioNodeList,
    args: []*Expr,
) errors.Result(Value) {
    const callback_expr: *Expr = switch (args.len) {
        1, 2 => args[0],
        else => return error.ScriptRuntime,
    };
    const this_arg_expr: ?*Expr = if (args.len == 2) args[1] else null;

    const callback_value = try evalExpr(allocator, host, bindings, callback_expr);
    const callback = switch (callback_value) {
        .function => |function| function,
        else => return error.ScriptRuntime,
    };

    if (this_arg_expr) |expr| {
        _ = try evalExpr(allocator, host, bindings, expr);
    }

    const items = try radioNodeListCurrentIds(allocator, host, list);
    defer allocator.free(items);

    for (items, 0..) |item, index| {
        const positional = [_]Value{
            .{ .element = item },
            .{ .number = @floatFromInt(index) },
            .{ .radio_node_list = list },
        };
        var function_bindings = try functionBindings(allocator, callback, positional[0..]);
        defer function_bindings.deinit(allocator);

        const source_name = try std.fmt.allocPrint(allocator, "radionodelist:forEach:{d}", .{index});
        defer allocator.free(source_name);

        try executeScriptSource(
            allocator,
            host,
            callback.body_source,
            source_name,
            function_bindings.items,
        );
    }

    return Value{ .undefined_value = {} };
}

pub fn functionBindings(
    allocator: std.mem.Allocator,
    function: ScriptFunction,
    positional: []const Value,
) errors.Result(std.ArrayList(Binding)) {
    var bindings_out: std.ArrayList(Binding) = .empty;
    errdefer bindings_out.deinit(allocator);

    for (function.params, 0..) |param, index| {
        try bindings_out.append(allocator, .{
            .name = param,
            .value = if (index < positional.len)
                positional[index]
            else
                Value{ .undefined_value = {} },
        });
    }

    return bindings_out;
}

fn eventListenerBindings(
    allocator: std.mem.Allocator,
    function: ScriptFunction,
    event: *ScriptEvent,
) errors.Result(std.ArrayList(Binding)) {
    var bindings_out: std.ArrayList(Binding) = .empty;
    errdefer bindings_out.deinit(allocator);

    try bindings_out.append(allocator, .{
        .name = "event",
        .value = .{ .event = event },
    });

    for (function.params, 0..) |param, index| {
        try bindings_out.append(allocator, .{
            .name = param,
            .value = if (index == 0)
                .{ .event = event }
            else
                .{ .undefined_value = {} },
        });
    }

    return bindings_out;
}

fn registerListener(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    target: ListenerTarget,
    args: []*Expr,
) errors.Result(Value) {
    if (args.len < 2 or args.len > 3) return error.ScriptRuntime;

    const event_value = try evalExpr(allocator, host, bindings, args[0]);
    const event_type = try asString(allocator, event_value);

    const handler_value = try evalExpr(allocator, host, bindings, args[1]);
    const handler = switch (handler_value) {
        .function => |function| function,
        else => return error.ScriptRuntime,
    };

    const capture = if (args.len == 3) blk: {
        const capture_value = try evalExpr(allocator, host, bindings, args[2]);
        break :blk isTruthy(capture_value);
    } else false;

    try host.registerEventListener(target, event_type, capture, handler);
    return Value{ .undefined_value = {} };
}

fn elementAppendChild(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    element: dom.NodeId,
    args: []*Expr,
) errors.Result(Value) {
    if (args.len != 1) return error.ScriptRuntime;

    const child_value = try evalNodeValue(allocator, host, bindings, args[0]);
    const child = try nodeValueId(child_value);
    _ = host.domStoreMut().appendChild(element, child) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return child_value;
}

fn elementInsertBefore(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    element: dom.NodeId,
    args: []*Expr,
) errors.Result(Value) {
    if (args.len != 2) return error.ScriptRuntime;

    const child_value = try evalNodeValue(allocator, host, bindings, args[0]);
    const child = try nodeValueId(child_value);
    const reference = try evalOptionalNodeHandle(allocator, host, bindings, args[1]);
    _ = host.domStoreMut().insertBefore(element, child, reference) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return child_value;
}

fn elementReplaceChild(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    element: dom.NodeId,
    args: []*Expr,
) errors.Result(Value) {
    if (args.len != 2) return error.ScriptRuntime;

    const new_child_value = try evalNodeValue(allocator, host, bindings, args[0]);
    const new_child = try nodeValueId(new_child_value);
    const old_child_value = try evalNodeValue(allocator, host, bindings, args[1]);
    const old_child = try nodeValueId(old_child_value);
    _ = host.domStoreMut().replaceChild(element, new_child, old_child) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return old_child_value;
}

fn elementReplaceChildren(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    element: dom.NodeId,
    args: []*Expr,
) errors.Result(Value) {
    var children = try evalElementArguments(allocator, host, bindings, args);
    defer children.deinit(allocator);
    host.domStoreMut().replaceChildren(element, children.items) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return Value{ .undefined_value = {} };
}

fn elementAppend(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    element: dom.NodeId,
    args: []*Expr,
) errors.Result(Value) {
    var children = try evalElementArguments(allocator, host, bindings, args);
    defer children.deinit(allocator);
    host.domStoreMut().appendChildren(element, children.items) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return Value{ .undefined_value = {} };
}

fn elementPrepend(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    element: dom.NodeId,
    args: []*Expr,
) errors.Result(Value) {
    var children = try evalElementArguments(allocator, host, bindings, args);
    defer children.deinit(allocator);
    host.domStoreMut().prependChildren(element, children.items) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return Value{ .undefined_value = {} };
}

fn elementBefore(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    element: dom.NodeId,
    args: []*Expr,
) errors.Result(Value) {
    var children = try evalElementArguments(allocator, host, bindings, args);
    defer children.deinit(allocator);

    const node = host.domStore().nodeAt(element) orelse return error.ScriptRuntime;
    const parent = node.parent orelse return Value{ .undefined_value = {} };
    host.domStoreMut().insertChildrenBefore(parent, element, children.items) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return Value{ .undefined_value = {} };
}

fn elementAfter(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    element: dom.NodeId,
    args: []*Expr,
) errors.Result(Value) {
    var children = try evalElementArguments(allocator, host, bindings, args);
    defer children.deinit(allocator);

    const node = host.domStore().nodeAt(element) orelse return error.ScriptRuntime;
    const parent = node.parent orelse return Value{ .undefined_value = {} };
    host.domStoreMut().insertChildrenAfter(parent, element, children.items) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return Value{ .undefined_value = {} };
}

fn elementRemove(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    element: dom.NodeId,
    args: []*Expr,
) errors.Result(Value) {
    _ = allocator;
    _ = bindings;
    if (args.len != 0) return error.ScriptRuntime;
    host.domStoreMut().removeNode(element) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return Value{ .undefined_value = {} };
}

fn nodeReplaceWith(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    node_id: dom.NodeId,
    args: []*Expr,
) errors.Result(Value) {
    var children = try evalNodeArguments(allocator, host, bindings, args);
    defer children.deinit(allocator);

    const node = host.domStore().nodeAt(node_id) orelse return error.ScriptRuntime;
    const parent = node.parent orelse return Value{ .undefined_value = {} };
    host.domStoreMut().insertChildrenBefore(parent, node_id, children.items) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    host.domStoreMut().removeNode(node_id) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return Value{ .undefined_value = {} };
}

fn evalElementHandle(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    expr: *Expr,
) errors.Result(dom.NodeId) {
    const value = try evalExpr(allocator, host, bindings, expr);
    return switch (value) {
        .element => |element| element,
        else => error.ScriptRuntime,
    };
}

fn evalNodeHandle(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    expr: *Expr,
) errors.Result(dom.NodeId) {
    const value = try evalExpr(allocator, host, bindings, expr);
    return switch (value) {
        .element => |element| element,
        .node => |node_id| node_id,
        else => error.ScriptRuntime,
    };
}

fn evalNodeValue(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    expr: *Expr,
) errors.Result(Value) {
    const value = try evalExpr(allocator, host, bindings, expr);
    return switch (value) {
        .element, .node, .template_content => value,
        else => error.ScriptRuntime,
    };
}

fn nodeContainsValue(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    container_is_fragment: bool,
    container_id: dom.NodeId,
    expr: *Expr,
) errors.Result(bool) {
    const value = try evalExpr(allocator, host, bindings, expr);
    return if (container_is_fragment) switch (value) {
        .document => false,
        .element => |element| host.domStore().nodeContainsFragment(container_id, element),
        .node => |node_id| host.domStore().nodeContainsFragment(container_id, node_id),
        .template_content => |fragment_id| sameNodeId(container_id, fragment_id),
        .null_value, .undefined_value => false,
        else => error.ScriptRuntime,
    } else switch (value) {
        .document => host.domStore().nodeContains(container_id, host.domStore().documentId()),
        .element => |element| host.domStore().nodeContains(container_id, element),
        .node => |node_id| host.domStore().nodeContains(container_id, node_id),
        .template_content => false,
        .null_value, .undefined_value => false,
        else => error.ScriptRuntime,
    };
}

fn nodeSameNodeValue(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    object: Value,
    args: []*Expr,
) errors.Result(Value) {
    if (args.len != 1) return error.ScriptRuntime;
    const other = try evalExpr(allocator, host, bindings, args[0]);
    const same = switch (object) {
        .document => switch (other) {
            .null_value, .undefined_value => false,
            .document => true,
            .element, .node, .template_content => false,
            else => return error.ScriptRuntime,
        },
        .element => |element| switch (other) {
            .null_value, .undefined_value => false,
            .document => false,
            .element => |other_element| sameNodeId(element, other_element),
            .node => |other_node| sameNodeId(element, other_node),
            .template_content => false,
            else => return error.ScriptRuntime,
        },
        .node => |node_id| switch (other) {
            .null_value, .undefined_value => false,
            .document => false,
            .element => |other_element| sameNodeId(node_id, other_element),
            .node => |other_node| sameNodeId(node_id, other_node),
            .template_content => false,
            else => return error.ScriptRuntime,
        },
        .template_content => |fragment_id| switch (other) {
            .null_value, .undefined_value => false,
            .template_content => |other_fragment_id| sameNodeId(fragment_id, other_fragment_id),
            .document, .element, .node => false,
            else => return error.ScriptRuntime,
        },
        else => return error.ScriptRuntime,
    };
    return Value{ .boolean = same };
}

fn nodeEqualNodeValue(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    object: Value,
    args: []*Expr,
) errors.Result(Value) {
    if (args.len != 1) return error.ScriptRuntime;
    const other = try evalExpr(allocator, host, bindings, args[0]);
    const store = host.domStore();
    const equal = switch (object) {
        .document => switch (other) {
            .null_value, .undefined_value => false,
            .document => true,
            .element => |other_element| dom.nodeIsEqualNode(store, store.documentId(), other_element),
            .node => |other_node| dom.nodeIsEqualNode(store, store.documentId(), other_node),
            .template_content => false,
            else => return error.ScriptRuntime,
        },
        .element => |element| switch (other) {
            .null_value, .undefined_value => false,
            .document => dom.nodeIsEqualNode(store, element, store.documentId()),
            .element => |other_element| dom.nodeIsEqualNode(store, element, other_element),
            .node => |other_node| dom.nodeIsEqualNode(store, element, other_node),
            .template_content => false,
            else => return error.ScriptRuntime,
        },
        .node => |node_id| switch (other) {
            .null_value, .undefined_value => false,
            .document => dom.nodeIsEqualNode(store, node_id, store.documentId()),
            .element => |other_element| dom.nodeIsEqualNode(store, node_id, other_element),
            .node => |other_node| dom.nodeIsEqualNode(store, node_id, other_node),
            .template_content => false,
            else => return error.ScriptRuntime,
        },
        .template_content => |fragment_id| switch (other) {
            .null_value, .undefined_value => false,
            .template_content => |other_fragment_id| dom.templateContentIsEqualNode(store, fragment_id, other_fragment_id),
            .document, .element, .node => false,
            else => return error.ScriptRuntime,
        },
        else => return error.ScriptRuntime,
    };
    return Value{ .boolean = equal };
}

fn sameNodeId(left: dom.NodeId, right: dom.NodeId) bool {
    return left.index == right.index and left.generation == right.generation;
}

fn compareDocumentPositionDisconnected(left_root: dom.NodeId, right_root: dom.NodeId) u16 {
    const DISCONNECTED: u16 = 0x01;
    const FOLLOWING: u16 = 0x04;
    const IMPLEMENTATION_SPECIFIC: u16 = 0x20;
    return if (left_root.index < right_root.index)
        DISCONNECTED | IMPLEMENTATION_SPECIFIC | FOLLOWING
    else
        DISCONNECTED | IMPLEMENTATION_SPECIFIC | 0x02;
}

fn nodeCompareDocumentPositionValue(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    reference_value: Value,
    expr: *Expr,
) errors.Result(u16) {
    const FOLLOWING: u16 = 0x04;
    const CONTAINS: u16 = 0x08;
    const CONTAINED_BY: u16 = 0x10;

    const reference_root = switch (reference_value) {
        .document => host.domStore().documentId(),
        .element => |element| element,
        .node => |node_id| node_id,
        .template_content => |fragment_id| fragment_id,
        else => return error.ScriptRuntime,
    };

    const value = try evalExpr(allocator, host, bindings, expr);
    return switch (reference_value) {
        .template_content => |fragment_id| switch (value) {
            .template_content => |other_fragment_id| if (sameNodeId(fragment_id, other_fragment_id)) 0 else compareDocumentPositionDisconnected(fragment_id, other_fragment_id),
            .document => compareDocumentPositionDisconnected(fragment_id, host.domStore().documentId()),
            .element => |element| blk: {
                if (dom.comparisonRoot(host.domStore(), element)) |element_root| {
                    if (sameNodeId(element_root, fragment_id)) break :blk CONTAINED_BY | FOLLOWING;
                    break :blk compareDocumentPositionDisconnected(fragment_id, element_root);
                }
                break :blk compareDocumentPositionDisconnected(fragment_id, element);
            },
            .node => |node_id| blk: {
                if (dom.comparisonRoot(host.domStore(), node_id)) |node_root| {
                    if (sameNodeId(node_root, fragment_id)) break :blk CONTAINED_BY | FOLLOWING;
                    break :blk compareDocumentPositionDisconnected(fragment_id, node_root);
                }
                break :blk compareDocumentPositionDisconnected(fragment_id, node_id);
            },
            .null_value, .undefined_value => return error.ScriptRuntime,
            else => return error.ScriptRuntime,
        },
        else => switch (value) {
            .document => dom.compareDocumentPosition(host.domStore(), reference_root, host.domStore().documentId()),
            .element => |element| dom.compareDocumentPosition(host.domStore(), reference_root, element),
            .node => |node_id| dom.compareDocumentPosition(host.domStore(), reference_root, node_id),
            .template_content => |fragment_id| blk: {
                if (dom.comparisonRoot(host.domStore(), reference_root)) |reference_comparison_root| {
                    if (sameNodeId(reference_comparison_root, fragment_id)) break :blk CONTAINS | 0x02;
                    break :blk compareDocumentPositionDisconnected(reference_comparison_root, fragment_id);
                }
                break :blk compareDocumentPositionDisconnected(reference_root, fragment_id);
            },
            .null_value, .undefined_value => return error.ScriptRuntime,
            else => return error.ScriptRuntime,
        },
    };
}

fn evalOptionalNodeHandle(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    expr: *Expr,
) errors.Result(?dom.NodeId) {
    const value = try evalExpr(allocator, host, bindings, expr);
    return switch (value) {
        .element => |element| element,
        .node => |node_id| node_id,
        .template_content => |element| element,
        .null_value, .undefined_value => null,
        else => error.ScriptRuntime,
    };
}

fn nodeAppendChildValue(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    parent: dom.NodeId,
    expr: *Expr,
) errors.Result(Value) {
    const child_value = try evalNodeValue(allocator, host, bindings, expr);
    const child = try nodeValueId(child_value);
    _ = host.domStoreMut().appendChild(parent, child) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return child_value;
}

fn nodeInsertBeforeValue(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    parent: dom.NodeId,
    child_expr: *Expr,
    reference_expr: *Expr,
) errors.Result(Value) {
    const child_value = try evalNodeValue(allocator, host, bindings, child_expr);
    const child = try nodeValueId(child_value);
    const reference = try evalOptionalNodeHandle(allocator, host, bindings, reference_expr);
    _ = host.domStoreMut().insertBefore(parent, child, reference) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return child_value;
}

fn nodeReplaceChildValue(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    parent: dom.NodeId,
    new_child_expr: *Expr,
    old_child_expr: *Expr,
) errors.Result(Value) {
    const new_child_value = try evalNodeValue(allocator, host, bindings, new_child_expr);
    const new_child = try nodeValueId(new_child_value);
    const old_child_value = try evalNodeValue(allocator, host, bindings, old_child_expr);
    const old_child = try nodeValueId(old_child_value);
    _ = host.domStoreMut().replaceChild(parent, new_child, old_child) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return old_child_value;
}

fn nodeRemoveChildValue(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    parent: dom.NodeId,
    child_expr: *Expr,
) errors.Result(Value) {
    const child_value = try evalNodeValue(allocator, host, bindings, child_expr);
    const child = try nodeValueId(child_value);
    const child_record = host.domStore().nodeAt(child) orelse return error.ScriptRuntime;
    const actual_parent = child_record.parent orelse return error.ScriptRuntime;
    if (!sameNodeId(actual_parent, parent)) return error.ScriptRuntime;
    host.domStoreMut().removeNode(child) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return child_value;
}

fn nodeValueId(value: Value) errors.Result(dom.NodeId) {
    return switch (value) {
        .element => |element| element,
        .node => |node_id| node_id,
        .template_content => |element| element,
        else => error.ScriptRuntime,
    };
}

fn evalNodeArguments(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    args: []*Expr,
) errors.Result(std.ArrayList(dom.NodeId)) {
    var children: std.ArrayList(dom.NodeId) = .empty;
    errdefer children.deinit(allocator);

    for (args) |expr| {
        try children.append(allocator, try evalNodeHandle(allocator, host, bindings, expr));
    }

    return children;
}

fn evalElementArguments(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    args: []*Expr,
) errors.Result(std.ArrayList(dom.NodeId)) {
    var children: std.ArrayList(dom.NodeId) = .empty;
    errdefer children.deinit(allocator);

    for (args) |expr| {
        try children.append(allocator, try evalElementHandle(allocator, host, bindings, expr));
    }

    return children;
}

fn originFromUrl(allocator: std.mem.Allocator, url: []const u8) errors.Result([]const u8) {
    const colon_index = std.mem.indexOfScalar(u8, url, ':') orelse return allocator.dupe(u8, "null");
    if (colon_index + 2 >= url.len) return allocator.dupe(u8, "null");

    const scheme = url[0..colon_index];
    const after_colon = url[colon_index + 1 ..];
    if (!std.mem.startsWith(u8, after_colon, "//")) {
        return allocator.dupe(u8, "null");
    }

    const authority_start = colon_index + 3;
    const remainder = url[authority_start..];
    const authority_end = std.mem.indexOfAny(u8, remainder, "/?#") orelse remainder.len;
    const authority = remainder[0..authority_end];
    if (authority.len == 0) {
        return allocator.dupe(u8, "null");
    }

    var out = try allocator.alloc(u8, scheme.len + 3 + authority.len);
    for (scheme, 0..) |byte, index| {
        out[index] = std.ascii.toLower(byte);
    }
    out[scheme.len] = ':';
    out[scheme.len + 1] = '/';
    out[scheme.len + 2] = '/';
    @memcpy(out[scheme.len + 3 ..], authority);
    return out;
}

const LocationAuthorityBounds = struct {
    start: usize,
    end: usize,
};

const LocationPathBounds = struct {
    start: usize,
    end: usize,
};

const LocationAuthorityParts = struct {
    username: []const u8,
    password: []const u8,
    host: []const u8,
    hostname: []const u8,
    port: []const u8,
};

fn lowercaseDupe(allocator: std.mem.Allocator, text: []const u8) errors.Result([]const u8) {
    var out = try allocator.alloc(u8, text.len);
    for (text, 0..) |byte, index| {
        out[index] = std.ascii.toLower(byte);
    }
    return out;
}

fn locationAuthorityBounds(url: []const u8) ?LocationAuthorityBounds {
    const scheme_end = std.mem.indexOfScalar(u8, url, ':') orelse return null;
    const after_colon = url[scheme_end + 1 ..];
    if (!std.mem.startsWith(u8, after_colon, "//")) {
        return null;
    }

    const authority_start = scheme_end + 3;
    const remainder = url[authority_start..];
    const authority_end = std.mem.indexOfAny(u8, remainder, "/?#") orelse remainder.len;
    return .{
        .start = authority_start,
        .end = authority_start + authority_end,
    };
}

fn locationAuthority(url: []const u8) ?[]const u8 {
    const bounds = locationAuthorityBounds(url) orelse return null;
    return url[bounds.start..bounds.end];
}

fn locationAuthorityParts(authority: []const u8) ?LocationAuthorityParts {
    const userinfo_end = std.mem.lastIndexOfScalar(u8, authority, '@') orelse authority.len;
    const userinfo = if (userinfo_end < authority.len) authority[0..userinfo_end] else "";
    const host_port = if (userinfo_end < authority.len) authority[userinfo_end + 1 ..] else authority;
    if (host_port.len == 0) return null;

    const userinfo_colon = std.mem.indexOfScalar(u8, userinfo, ':');
    const username = if (userinfo_colon) |index| userinfo[0..index] else userinfo;
    const password = if (userinfo_colon) |index| userinfo[index + 1 ..] else "";

    if (host_port[0] == '[') {
        const end_bracket = std.mem.indexOfScalar(u8, host_port, ']') orelse return null;
        const hostname = host_port[1..end_bracket];
        const port = if (end_bracket + 1 < host_port.len and host_port[end_bracket + 1] == ':')
            host_port[end_bracket + 2 ..]
        else
            "";
        return .{
            .username = username,
            .password = password,
            .host = host_port,
            .hostname = hostname,
            .port = port,
        };
    }

    const colon_index = std.mem.indexOfScalar(u8, host_port, ':');
    const hostname = if (colon_index) |index| host_port[0..index] else host_port;
    const port = if (colon_index) |index| host_port[index + 1 ..] else "";
    return .{
        .username = username,
        .password = password,
        .host = host_port,
        .hostname = hostname,
        .port = port,
    };
}

fn locationProtocolFromUrl(allocator: std.mem.Allocator, url: []const u8) errors.Result([]const u8) {
    const colon_index = std.mem.indexOfScalar(u8, url, ':') orelse return allocator.dupe(u8, "");
    const scheme = url[0..colon_index];
    var out = try allocator.alloc(u8, scheme.len + 1);
    for (scheme, 0..) |byte, index| {
        out[index] = std.ascii.toLower(byte);
    }
    out[scheme.len] = ':';
    return out;
}

fn locationAuthorityHostFromParts(
    allocator: std.mem.Allocator,
    parts: LocationAuthorityParts,
) errors.Result([]const u8) {
    const hostname = try lowercaseDupe(allocator, parts.hostname);
    defer allocator.free(hostname);

    var out: std.ArrayList(u8) = .empty;
    errdefer out.deinit(allocator);

    if (std.mem.indexOfScalar(u8, parts.hostname, ':') != null) {
        try out.append(allocator, '[');
        try out.appendSlice(allocator, hostname);
        try out.append(allocator, ']');
    } else {
        try out.appendSlice(allocator, hostname);
    }

    if (parts.port.len != 0) {
        try out.append(allocator, ':');
        try out.appendSlice(allocator, parts.port);
    }

    return try out.toOwnedSlice(allocator);
}

fn locationHostFromUrl(allocator: std.mem.Allocator, url: []const u8) errors.Result([]const u8) {
    const authority = locationAuthority(url) orelse return allocator.dupe(u8, "");
    const parts = locationAuthorityParts(authority) orelse return allocator.dupe(u8, "");
    return try locationAuthorityHostFromParts(allocator, parts);
}

fn locationHostnameFromUrl(allocator: std.mem.Allocator, url: []const u8) errors.Result([]const u8) {
    const authority = locationAuthority(url) orelse return allocator.dupe(u8, "");
    const parts = locationAuthorityParts(authority) orelse return allocator.dupe(u8, "");
    return try lowercaseDupe(allocator, parts.hostname);
}

fn locationPortFromUrl(url: []const u8) errors.Result([]const u8) {
    const authority = locationAuthority(url) orelse return "";
    const parts = locationAuthorityParts(authority) orelse return "";
    return parts.port;
}

fn locationUsernameFromUrl(url: []const u8) errors.Result([]const u8) {
    const authority = locationAuthority(url) orelse return "";
    const parts = locationAuthorityParts(authority) orelse return "";
    return parts.username;
}

fn locationPasswordFromUrl(url: []const u8) errors.Result([]const u8) {
    const authority = locationAuthority(url) orelse return "";
    const parts = locationAuthorityParts(authority) orelse return "";
    return parts.password;
}

fn locationPathBounds(url: []const u8) ?LocationPathBounds {
    const scheme_end = std.mem.indexOfScalar(u8, url, ':') orelse return null;
    var path_start = scheme_end + 1;

    if (path_start < url.len and std.mem.startsWith(u8, url[path_start..], "//")) {
        path_start += 2;
        const authority_end = std.mem.indexOfAny(u8, url[path_start..], "/?#") orelse url.len - path_start;
        path_start += authority_end;
    }

    const path_end = std.mem.indexOfAny(u8, url[path_start..], "?#") orelse url.len - path_start;
    return .{ .start = path_start, .end = path_start + path_end };
}

fn locationPathnameFromUrl(url: []const u8) errors.Result([]const u8) {
    const bounds = locationPathBounds(url) orelse return "/";
    const path = url[bounds.start..bounds.end];
    if (path.len == 0) return "/";
    return path;
}

fn locationSearchBounds(url: []const u8) ?LocationPathBounds {
    const path_bounds = locationPathBounds(url) orelse LocationPathBounds{
        .start = url.len,
        .end = url.len,
    };
    if (path_bounds.end >= url.len or url[path_bounds.end] != '?') {
        return null;
    }

    const search_start = path_bounds.end + 1;
    const search_end = std.mem.indexOfScalar(u8, url[search_start..], '#') orelse url.len - search_start;
    return .{ .start = search_start, .end = search_start + search_end };
}

fn locationSearchFromUrl(allocator: std.mem.Allocator, url: []const u8) errors.Result([]const u8) {
    const bounds = locationSearchBounds(url) orelse return allocator.dupe(u8, "");
    const search = url[bounds.start..bounds.end];
    var out = try allocator.alloc(u8, search.len + 1);
    out[0] = '?';
    if (search.len != 0) {
        @memcpy(out[1..], search);
    }
    return out;
}

fn normalizeLocationHostnameForAuthority(
    allocator: std.mem.Allocator,
    hostname: []const u8,
) errors.Result([]const u8) {
    const trimmed = std.mem.trim(u8, hostname, " \t\r\n");
    if (trimmed.len == 0) return error.ScriptRuntime;

    const stripped = if (trimmed.len >= 2 and trimmed[0] == '[' and trimmed[trimmed.len - 1] == ']')
        trimmed[1 .. trimmed.len - 1]
    else
        trimmed;
    const needs_brackets =
        (trimmed.len >= 2 and trimmed[0] == '[' and trimmed[trimmed.len - 1] == ']') or
        std.mem.indexOfScalar(u8, stripped, ':') != null;

    const extra: usize = if (needs_brackets) 2 else 0;
    var out = try allocator.alloc(u8, stripped.len + extra);
    var out_index: usize = 0;
    if (needs_brackets) {
        out[0] = '[';
        out_index = 1;
    }
    for (stripped, 0..) |byte, index| {
        out[out_index + index] = std.ascii.toLower(byte);
    }
    out_index += stripped.len;
    if (needs_brackets) {
        out[out_index] = ']';
    }
    return out;
}

fn locationAuthorityString(
    allocator: std.mem.Allocator,
    username: []const u8,
    password: []const u8,
    host: []const u8,
    port: []const u8,
) errors.Result([]const u8) {
    var out: std.ArrayList(u8) = .empty;
    errdefer out.deinit(allocator);

    if (username.len != 0 or password.len != 0) {
        try out.appendSlice(allocator, username);
        if (password.len != 0 or username.len == 0) {
            try out.append(allocator, ':');
            try out.appendSlice(allocator, password);
        }
        try out.append(allocator, '@');
    }

    try out.appendSlice(allocator, host);
    if (port.len != 0) {
        try out.append(allocator, ':');
        try out.appendSlice(allocator, port);
    }

    return try out.toOwnedSlice(allocator);
}

fn locationAuthorityHostFromInput(
    allocator: std.mem.Allocator,
    authority: []const u8,
) errors.Result([]const u8) {
    const parts = locationAuthorityParts(authority) orelse return error.ScriptRuntime;
    const hostname = try normalizeLocationHostnameForAuthority(allocator, parts.hostname);
    defer allocator.free(hostname);
    return try locationAuthorityString(allocator, "", "", hostname, parts.port);
}

fn locationUrlWithAuthority(
    allocator: std.mem.Allocator,
    current_url: []const u8,
    authority: []const u8,
) errors.Result([]const u8) {
    const trimmed = std.mem.trim(u8, authority, " \t\r\n");
    if (trimmed.len == 0) return error.ScriptRuntime;

    const bounds = locationAuthorityBounds(current_url) orelse return error.ScriptRuntime;
    var out = try allocator.alloc(u8, current_url.len - (bounds.end - bounds.start) + trimmed.len);
    @memcpy(out[0..bounds.start], current_url[0..bounds.start]);
    @memcpy(out[bounds.start .. bounds.start + trimmed.len], trimmed);
    @memcpy(out[bounds.start + trimmed.len ..], current_url[bounds.end..]);
    return out;
}

fn locationUrlWithProtocol(
    allocator: std.mem.Allocator,
    current_url: []const u8,
    protocol: []const u8,
) errors.Result([]const u8) {
    const trimmed = std.mem.trim(u8, protocol, " \t\r\n");
    const normalized = if (trimmed.len > 0 and trimmed[trimmed.len - 1] == ':')
        trimmed[0 .. trimmed.len - 1]
    else
        trimmed;
    if (normalized.len == 0) return error.ScriptRuntime;

    const current_scheme_end = std.mem.indexOfScalar(u8, current_url, ':') orelse return error.ScriptRuntime;
    const rest = current_url[current_scheme_end + 1 ..];
    var out = try allocator.alloc(u8, normalized.len + 1 + rest.len);
    for (normalized, 0..) |byte, index| {
        out[index] = std.ascii.toLower(byte);
    }
    out[normalized.len] = ':';
    @memcpy(out[normalized.len + 1 ..], rest);
    return out;
}

fn locationUrlWithHost(
    allocator: std.mem.Allocator,
    current_url: []const u8,
    host: []const u8,
) errors.Result([]const u8) {
    const trimmed = std.mem.trim(u8, host, " \t\r\n");
    if (trimmed.len == 0) return error.ScriptRuntime;

    const next_host = try locationAuthorityHostFromInput(allocator, trimmed);
    defer allocator.free(next_host);

    const current_authority = locationAuthority(current_url) orelse return error.ScriptRuntime;
    const current_parts = locationAuthorityParts(current_authority) orelse return error.ScriptRuntime;
    const next_authority = try locationAuthorityString(
        allocator,
        current_parts.username,
        current_parts.password,
        next_host,
        "",
    );
    defer allocator.free(next_authority);

    return try locationUrlWithAuthority(allocator, current_url, next_authority);
}

fn locationUrlWithHostname(
    allocator: std.mem.Allocator,
    current_url: []const u8,
    hostname: []const u8,
) errors.Result([]const u8) {
    const normalized = try normalizeLocationHostnameForAuthority(allocator, hostname);
    defer allocator.free(normalized);

    const current_authority = locationAuthority(current_url) orelse return error.ScriptRuntime;
    const current_parts = locationAuthorityParts(current_authority) orelse return error.ScriptRuntime;
    const next_authority = try locationAuthorityString(
        allocator,
        current_parts.username,
        current_parts.password,
        normalized,
        current_parts.port,
    );
    defer allocator.free(next_authority);

    return try locationUrlWithAuthority(allocator, current_url, next_authority);
}

fn locationUrlWithPort(
    allocator: std.mem.Allocator,
    current_url: []const u8,
    port: []const u8,
) errors.Result([]const u8) {
    const trimmed = std.mem.trim(u8, port, " \t\r\n");
    if (trimmed.len != 0) {
        for (trimmed) |byte| {
            if (!std.ascii.isDigit(byte)) return error.ScriptRuntime;
        }
    }

    const current_authority = locationAuthority(current_url) orelse return error.ScriptRuntime;
    const current_parts = locationAuthorityParts(current_authority) orelse return error.ScriptRuntime;
    const hostname = try normalizeLocationHostnameForAuthority(allocator, current_parts.hostname);
    defer allocator.free(hostname);
    const next_authority = try locationAuthorityString(
        allocator,
        current_parts.username,
        current_parts.password,
        hostname,
        trimmed,
    );
    defer allocator.free(next_authority);

    return try locationUrlWithAuthority(allocator, current_url, next_authority);
}

fn locationUrlWithUsername(
    allocator: std.mem.Allocator,
    current_url: []const u8,
    username: []const u8,
) errors.Result([]const u8) {
    const trimmed = std.mem.trim(u8, username, " \t\r\n");
    const current_authority = locationAuthority(current_url) orelse return error.ScriptRuntime;
    const current_parts = locationAuthorityParts(current_authority) orelse return error.ScriptRuntime;
    const next_authority = try locationAuthorityString(
        allocator,
        trimmed,
        current_parts.password,
        current_parts.host,
        "",
    );
    defer allocator.free(next_authority);

    return try locationUrlWithAuthority(allocator, current_url, next_authority);
}

fn locationUrlWithPassword(
    allocator: std.mem.Allocator,
    current_url: []const u8,
    password: []const u8,
) errors.Result([]const u8) {
    const trimmed = std.mem.trim(u8, password, " \t\r\n");
    const current_authority = locationAuthority(current_url) orelse return error.ScriptRuntime;
    const current_parts = locationAuthorityParts(current_authority) orelse return error.ScriptRuntime;
    const next_authority = try locationAuthorityString(
        allocator,
        current_parts.username,
        trimmed,
        current_parts.host,
        "",
    );
    defer allocator.free(next_authority);

    return try locationUrlWithAuthority(allocator, current_url, next_authority);
}

fn locationUrlWithPathname(
    allocator: std.mem.Allocator,
    current_url: []const u8,
    pathname: []const u8,
) errors.Result([]const u8) {
    const bounds = locationPathBounds(current_url) orelse LocationPathBounds{
        .start = current_url.len,
        .end = current_url.len,
    };
    var out: std.ArrayList(u8) = .empty;
    errdefer out.deinit(allocator);

    try out.appendSlice(allocator, current_url[0..bounds.start]);
    const normalized = pathname;
    if (normalized.len == 0) {
        try out.append(allocator, '/');
    } else if (normalized[0] == '/') {
        try out.appendSlice(allocator, normalized);
    } else {
        try out.append(allocator, '/');
        try out.appendSlice(allocator, normalized);
    }
    try out.appendSlice(allocator, current_url[bounds.end..]);
    return try out.toOwnedSlice(allocator);
}

fn locationUrlWithSearch(
    allocator: std.mem.Allocator,
    current_url: []const u8,
    search: []const u8,
) errors.Result([]const u8) {
    const path_bounds = locationPathBounds(current_url) orelse LocationPathBounds{
        .start = current_url.len,
        .end = current_url.len,
    };
    const hash_start = std.mem.indexOfScalar(u8, current_url[path_bounds.end..], '#') orelse current_url.len - path_bounds.end;
    var out: std.ArrayList(u8) = .empty;
    errdefer out.deinit(allocator);

    try out.appendSlice(allocator, current_url[0..path_bounds.end]);
    const normalized = std.mem.trim(u8, search, " \t\r\n");
    if (normalized.len == 0) {
        try out.appendSlice(allocator, current_url[path_bounds.end + hash_start ..]);
        return try out.toOwnedSlice(allocator);
    }

    try out.append(allocator, '?');
    if (normalized[0] == '?') {
        try out.appendSlice(allocator, normalized[1..]);
    } else {
        try out.appendSlice(allocator, normalized);
    }
    try out.appendSlice(allocator, current_url[path_bounds.end + hash_start ..]);
    return try out.toOwnedSlice(allocator);
}

fn evalBinaryAdd(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    binary: BinaryAddExpr,
) errors.Result(Value) {
    const left = try evalExpr(allocator, host, bindings, binary.left);
    const right = try evalExpr(allocator, host, bindings, binary.right);

    return switch (left) {
        .number => |left_number| switch (right) {
            .number => |right_number| Value{ .number = left_number + right_number },
            else => Value{ .string = try concatAsStrings(allocator, left, right) },
        },
        else => Value{ .string = try concatAsStrings(allocator, left, right) },
    };
}

fn concatAsStrings(
    allocator: std.mem.Allocator,
    left: Value,
    right: Value,
) errors.Result([]const u8) {
    const left_text = try asString(allocator, left);
    const right_text = try asString(allocator, right);
    var out: std.ArrayList(u8) = .empty;
    errdefer out.deinit(allocator);
    try out.appendSlice(allocator, left_text);
    try out.appendSlice(allocator, right_text);
    return try allocator.dupe(u8, out.items);
}

fn asString(allocator: std.mem.Allocator, value: Value) errors.Result([]const u8) {
    return switch (value) {
        .undefined_value => "undefined",
        .null_value => "null",
        .boolean => |flag| if (flag) "true" else "false",
        .number => |number| try std.fmt.allocPrint(allocator, "{d}", .{number}),
        .string => |text| text,
        .location => |location| location.current_url,
        .element => "[object Element]",
        .node => "[object Node]",
        .template_content => "[object DocumentFragment]",
        .class_list => "[object DOMTokenList]",
        .dataset => "[object DOMStringMap]",
        .node_list => "[object NodeList]",
        .collection_iterator => "[object Object]",
        .iterator_result => "[object Object]",
        .html_collection => |collection| switch (collection.kind) {
            .navigator_plugins => "[object PluginArray]",
            else => "[object HTMLCollection]",
        },
        .document_scripts => "[object HTMLCollection]",
        .document_anchors => "[object HTMLCollection]",
        .document_style_sheets => "[object StyleSheetList]",
        .css_rule_list => "[object CSSRuleList]",
        .css_rule => |rule| switch (rule) {
            .style => "[object CSSStyleRule]",
            .charset => "[object CSSCharsetRule]",
            .media => "[object CSSMediaRule]",
            .supports => "[object CSSSupportsRule]",
            .document => "[object CSSDocumentRule]",
            .container => "[object CSSContainerRule]",
            .starting_style => "[object CSSStartingStyleRule]",
            .keyframes => "[object CSSKeyframesRule]",
            .keyframe => "[object CSSKeyframeRule]",
            .font_face => "[object CSSFontFaceRule]",
            .font_feature_values => "[object CSSFontFeatureValuesRule]",
            .font_palette_values => "[object CSSFontPaletteValuesRule]",
            .color_profile => "[object CSSColorProfileRule]",
            .scope => "[object CSSScopeRule]",
            .page => "[object CSSPageRule]",
            .position_try => "[object CSSPositionTryRule]",
            .layer => |layer| if (layer.is_statement) "[object CSSLayerStatementRule]" else "[object CSSLayerBlockRule]",
            .counter_style => "[object CSSCounterStyleRule]",
            .property => "[object CSSPropertyRule]",
            .import => "[object CSSImportRule]",
            .namespace => "[object CSSNamespaceRule]",
        },
        .style_sheet => "[object CSSStyleSheet]",
        .style_declaration => |style| try styleDeclarationCssText(allocator, style),
        .radio_node_list => "[object RadioNodeList]",
        .media_query_list => "[object MediaQueryList]",
        .string_list => "[object DOMStringList]",
        .math => "[object Math]",
        .crypto => "[object Crypto]",
        .navigator => "[object Navigator]",
        .mime_type_array => "[object MimeTypeArray]",
        .performance => "[object Performance]",
        .screen => "[object Screen]",
        .screen_orientation => "[object ScreenOrientation]",
        .storage => "[object Storage]",
        .history => "[object History]",
        .media_list => |media| try media.currentText(),
        .collection_entry => "[object IteratorEntry]",
        .event => "[object Event]",
        .document => "[object Document]",
        .window => "[object Window]",
        .function => "[function]",
    };
}

fn valueForListenerTarget(target: ListenerTarget) Value {
    return switch (target) {
        .document => .{ .document = {} },
        .window => .{ .window = {} },
        .element => |element| .{ .element = element },
    };
}

fn isTruthy(value: Value) bool {
    return switch (value) {
        .undefined_value, .null_value => false,
        .boolean => |flag| flag,
        .number => |number| number != 0,
        .string => |text| text.len != 0,
        .element, .node, .template_content, .class_list, .dataset, .node_list, .collection_iterator, .iterator_result, .collection_entry, .html_collection, .document_scripts, .document_anchors, .document_style_sheets, .css_rule_list, .css_rule, .style_sheet, .style_declaration, .radio_node_list, .media_query_list, .string_list, .media_list, .math, .crypto, .navigator, .mime_type_array, .performance, .screen, .screen_orientation, .storage, .location, .history, .event, .document, .window, .function => true,
    };
}

fn makeLocationValue(allocator: std.mem.Allocator, host: anytype) errors.Result(Value) {
    const Host = @TypeOf(host);
    const state = try allocator.create(LocationState);
    state.* = .{
        .host = @ptrCast(host),
        .current_url = host.currentLocationUrl(),
        .current_url_fn = struct {
            fn call(ptr: *anyopaque) []const u8 {
                const typed: Host = @ptrCast(@alignCast(ptr));
                return typed.currentLocationUrl();
            }
        }.call,
        .hash_fn = struct {
            fn call(ptr: *anyopaque, alloc: std.mem.Allocator) errors.Result([]const u8) {
                const typed: Host = @ptrCast(@alignCast(ptr));
                return typed.locationHash(alloc);
            }
        }.call,
        .assign_fn = struct {
            fn call(ptr: *anyopaque, url: []const u8) errors.Result(void) {
                const typed: Host = @ptrCast(@alignCast(ptr));
                return typed.assignLocation(url);
            }
        }.call,
        .replace_fn = struct {
            fn call(ptr: *anyopaque, url: []const u8) errors.Result(void) {
                const typed: Host = @ptrCast(@alignCast(ptr));
                return typed.replaceLocation(url);
            }
        }.call,
        .reload_fn = struct {
            fn call(ptr: *anyopaque) errors.Result(void) {
                const typed: Host = @ptrCast(@alignCast(ptr));
                return typed.reloadLocation();
            }
        }.call,
        .set_hash_fn = struct {
            fn call(ptr: *anyopaque, value: []const u8) errors.Result(void) {
                const typed: Host = @ptrCast(@alignCast(ptr));
                return typed.setLocationHash(value);
            }
        }.call,
        .ancestor_origins = .{ .items = &.{} },
    };
    return .{ .location = state };
}

fn makePerformanceValue(allocator: std.mem.Allocator, host: anytype) errors.Result(Value) {
    const Host = @TypeOf(host);
    const state = try allocator.create(PerformanceState);
    state.* = .{
        .host = @ptrCast(host),
        .now_fn = struct {
            fn call(ptr: *anyopaque) i64 {
                const typed: Host = @ptrCast(@alignCast(ptr));
                return typed.nowMs();
            }
        }.call,
        .time_origin = 0,
    };
    return .{ .performance = state };
}

fn makeCryptoValue(allocator: std.mem.Allocator, host: anytype) errors.Result(Value) {
    const Host = @TypeOf(host);
    const state = try allocator.create(CryptoState);
    state.* = .{
        .host = @ptrCast(host),
        .random_uuid_fn = struct {
            fn call(ptr: *anyopaque, alloc: std.mem.Allocator) errors.Result([]const u8) {
                const typed: Host = @ptrCast(@alignCast(ptr));
                return typed.cryptoRandomUUID(alloc);
            }
        }.call,
    };
    return .{ .crypto = state };
}

fn makeMathValue(allocator: std.mem.Allocator, host: anytype) errors.Result(Value) {
    const Host = @TypeOf(host);
    const state = try allocator.create(MathState);
    state.* = .{
        .host = @ptrCast(host),
        .random_fn = struct {
            fn call(ptr: *anyopaque) f64 {
                const typed: Host = @ptrCast(@alignCast(ptr));
                return typed.mathRandom();
            }
        }.call,
    };
    return .{ .math = state };
}

fn makeHistoryValue(host: anytype) Value {
    const Host = @TypeOf(host);
    return .{
        .history = .{
            .host = @ptrCast(host),
            .length_fn = struct {
                fn call(ptr: *anyopaque) usize {
                    const typed: Host = @ptrCast(@alignCast(ptr));
                    return typed.historyLength();
                }
            }.call,
            .state_fn = struct {
                fn call(ptr: *anyopaque) ?[]const u8 {
                    const typed: Host = @ptrCast(@alignCast(ptr));
                    return typed.historyState();
                }
            }.call,
            .scroll_restoration_fn = struct {
                fn call(ptr: *anyopaque) []const u8 {
                    const typed: Host = @ptrCast(@alignCast(ptr));
                    return typed.historyScrollRestoration();
                }
            }.call,
            .set_scroll_restoration_fn = struct {
                fn call(ptr: *anyopaque, value: []const u8) errors.Result(void) {
                    const typed: Host = @ptrCast(@alignCast(ptr));
                    return typed.setHistoryScrollRestoration(value);
                }
            }.call,
            .back_fn = struct {
                fn call(ptr: *anyopaque) errors.Result(void) {
                    const typed: Host = @ptrCast(@alignCast(ptr));
                    return typed.historyBack();
                }
            }.call,
            .forward_fn = struct {
                fn call(ptr: *anyopaque) errors.Result(void) {
                    const typed: Host = @ptrCast(@alignCast(ptr));
                    return typed.historyForward();
                }
            }.call,
            .go_fn = struct {
                fn call(ptr: *anyopaque, delta: isize) errors.Result(void) {
                    const typed: Host = @ptrCast(@alignCast(ptr));
                    return typed.historyGo(delta);
                }
            }.call,
            .push_state_fn = struct {
                fn call(ptr: *anyopaque, state: ?[]const u8, url: []const u8) errors.Result(void) {
                    const typed: Host = @ptrCast(@alignCast(ptr));
                    return typed.historyPushState(state, url);
                }
            }.call,
            .replace_state_fn = struct {
                fn call(ptr: *anyopaque, state: ?[]const u8, url: []const u8) errors.Result(void) {
                    const typed: Host = @ptrCast(@alignCast(ptr));
                    return typed.historyReplaceState(state, url);
                }
            }.call,
        },
    };
}

fn makeStorageValue(allocator: std.mem.Allocator, host: anytype, target: StorageTarget) errors.Result(Value) {
    const Host = @TypeOf(host);
    const state = try allocator.create(StorageState);
    state.* = .{
        .host = @ptrCast(host),
        .target = target,
        .length_fn = struct {
            fn call(ptr: *anyopaque, storage_target: StorageTarget) usize {
                const typed: Host = @ptrCast(@alignCast(ptr));
                return typed.storageLength(storage_target);
            }
        }.call,
        .get_item_fn = struct {
            fn call(ptr: *anyopaque, storage_target: StorageTarget, key: []const u8) ?[]const u8 {
                const typed: Host = @ptrCast(@alignCast(ptr));
                return typed.storageGetItem(storage_target, key);
            }
        }.call,
        .set_item_fn = struct {
            fn call(ptr: *anyopaque, storage_target: StorageTarget, key: []const u8, value: []const u8) errors.Result(void) {
                const typed: Host = @ptrCast(@alignCast(ptr));
                return typed.storageSetItem(storage_target, key, value);
            }
        }.call,
        .remove_item_fn = struct {
            fn call(ptr: *anyopaque, storage_target: StorageTarget, key: []const u8) errors.Result(void) {
                const typed: Host = @ptrCast(@alignCast(ptr));
                return typed.storageRemoveItem(storage_target, key);
            }
        }.call,
        .clear_fn = struct {
            fn call(ptr: *anyopaque, storage_target: StorageTarget) errors.Result(void) {
                const typed: Host = @ptrCast(@alignCast(ptr));
                return typed.storageClear(storage_target);
            }
        }.call,
        .key_fn = struct {
            fn call(ptr: *anyopaque, storage_target: StorageTarget, index: usize) ?[]const u8 {
                const typed: Host = @ptrCast(@alignCast(ptr));
                return typed.storageKey(storage_target, index);
            }
        }.call,
    };
    return .{ .storage = state };
}

fn makeStyleDeclarationValue(allocator: std.mem.Allocator, host: anytype, element: dom.NodeId) errors.Result(Value) {
    const Host = @TypeOf(host);
    const state = try allocator.create(StyleDeclarationState);
    state.* = .{
        .host = @ptrCast(host),
        .element = element,
        .get_attribute_fn = struct {
            fn call(
                ptr: *anyopaque,
                node: dom.NodeId,
                name: []const u8,
                alloc: std.mem.Allocator,
            ) errors.Result(?[]const u8) {
                const typed: Host = @ptrCast(@alignCast(ptr));
                _ = alloc;
                return typed.domStore().getAttribute(node, name);
            }
        }.call,
        .set_attribute_fn = struct {
            fn call(
                ptr: *anyopaque,
                node: dom.NodeId,
                name: []const u8,
                value: []const u8,
            ) errors.Result(void) {
                const typed: Host = @ptrCast(@alignCast(ptr));
                return typed.domStoreMut().setAttribute(node, name, value);
            }
        }.call,
    };
    return .{ .style_declaration = state };
}

fn makeCssRuleStyleDeclarationValue(allocator: std.mem.Allocator, css_text: []const u8) errors.Result(Value) {
    const host_state = try allocator.create(CssRuleStyleDeclarationState);
    errdefer allocator.destroy(host_state);
    host_state.* = .{
        .css_text = css_text,
    };

    const state = try allocator.create(StyleDeclarationState);
    errdefer allocator.destroy(state);
    state.* = .{
        .host = @ptrCast(host_state),
        .element = dom.NodeId.new(0, 0),
        .get_attribute_fn = struct {
            fn call(
                ptr: *anyopaque,
                node: dom.NodeId,
                name: []const u8,
                alloc: std.mem.Allocator,
            ) errors.Result(?[]const u8) {
                _ = node;
                _ = alloc;
                const typed: *CssRuleStyleDeclarationState = @ptrCast(@alignCast(ptr));
                if (std.mem.eql(u8, name, "style")) {
                    return typed.css_text;
                }
                return null;
            }
        }.call,
        .set_attribute_fn = struct {
            fn call(
                ptr: *anyopaque,
                node: dom.NodeId,
                name: []const u8,
                value: []const u8,
            ) errors.Result(void) {
                _ = ptr;
                _ = node;
                _ = name;
                _ = value;
                return error.ScriptRuntime;
            }
        }.call,
    };

    return .{ .style_declaration = state };
}

fn cssRuleStyleDeclarationText(source: []const u8) errors.Result([]const u8) {
    const block_start = try parseCssRuleBlockStart(source, 0);
    const block_end = try parseCssRuleBlockEnd(source, block_start + 1);
    return source[block_start + 1 .. block_end - 1];
}

fn styleDeclarationCurrentText(
    allocator: std.mem.Allocator,
    state: *StyleDeclarationState,
) errors.Result([]const u8) {
    const raw = try state.get_attribute_fn(state.host, state.element, "style", allocator);
    if (raw) |text| return allocator.dupe(u8, text);
    return allocator.dupe(u8, "");
}

fn styleDeclarationEntries(
    allocator: std.mem.Allocator,
    state: *StyleDeclarationState,
) errors.Result(std.ArrayList(StylePropertyEntry)) {
    const raw = try styleDeclarationCurrentText(allocator, state);
    defer allocator.free(raw);
    return try parseStyleDeclarations(allocator, raw);
}

fn styleDeclarationCssText(
    allocator: std.mem.Allocator,
    state: *StyleDeclarationState,
) errors.Result([]const u8) {
    var entries = try styleDeclarationEntries(allocator, state);
    defer freeStyleDeclarations(allocator, &entries);
    return try serializeStyleDeclarations(allocator, entries.items);
}

fn styleDeclarationLength(
    allocator: std.mem.Allocator,
    state: *StyleDeclarationState,
) errors.Result(usize) {
    var entries = try styleDeclarationEntries(allocator, state);
    defer freeStyleDeclarations(allocator, &entries);
    return entries.items.len;
}

fn styleDeclarationItem(
    allocator: std.mem.Allocator,
    state: *StyleDeclarationState,
    index: usize,
) errors.Result([]const u8) {
    var entries = try styleDeclarationEntries(allocator, state);
    defer freeStyleDeclarations(allocator, &entries);
    if (index >= entries.items.len) return allocator.dupe(u8, "");
    return allocator.dupe(u8, entries.items[index].name);
}

fn styleDeclarationGetPropertyValue(
    allocator: std.mem.Allocator,
    state: *StyleDeclarationState,
    property: []const u8,
) errors.Result([]const u8) {
    const name = try stylePropertyName(allocator, property);
    defer allocator.free(name);

    var entries = try styleDeclarationEntries(allocator, state);
    defer freeStyleDeclarations(allocator, &entries);
    for (entries.items) |entry| {
        if (std.mem.eql(u8, entry.name, name)) {
            return allocator.dupe(u8, entry.value);
        }
    }
    return allocator.dupe(u8, "");
}

fn styleDeclarationGetPropertyPriority(
    allocator: std.mem.Allocator,
    state: *StyleDeclarationState,
    property: []const u8,
) errors.Result([]const u8) {
    const name = try stylePropertyName(allocator, property);
    defer allocator.free(name);

    var entries = try styleDeclarationEntries(allocator, state);
    defer freeStyleDeclarations(allocator, &entries);
    for (entries.items) |entry| {
        if (std.mem.eql(u8, entry.name, name)) {
            if (entry.important) {
                return allocator.dupe(u8, "important");
            }
            return allocator.dupe(u8, "");
        }
    }
    return allocator.dupe(u8, "");
}

fn styleDeclarationSetCssText(
    allocator: std.mem.Allocator,
    state: *StyleDeclarationState,
    text: []const u8,
) errors.Result(void) {
    var entries = try parseStyleDeclarations(allocator, text);
    defer freeStyleDeclarations(allocator, &entries);
    const css_text = try serializeStyleDeclarations(allocator, entries.items);
    defer allocator.free(css_text);
    try state.set_attribute_fn(state.host, state.element, "style", css_text);
    return;
}

fn styleDeclarationSetProperty(
    allocator: std.mem.Allocator,
    state: *StyleDeclarationState,
    property: []const u8,
    value: []const u8,
    priority: ?[]const u8,
) errors.Result(void) {
    const name = try stylePropertyName(allocator, property);
    defer allocator.free(name);

    const parsed = try styleDeclarationValueFromText(allocator, value, priority);
    defer allocator.free(parsed.value);
    if (std.mem.indexOfScalar(u8, parsed.value, ';') != null) return error.ScriptRuntime;

    var entries = try styleDeclarationEntries(allocator, state);
    defer freeStyleDeclarations(allocator, &entries);

    if (parsed.value.len == 0) {
        var index: usize = 0;
        while (index < entries.items.len) : (index += 1) {
            if (std.mem.eql(u8, entries.items[index].name, name)) {
                allocator.free(entries.items[index].name);
                allocator.free(entries.items[index].value);
                _ = entries.orderedRemove(index);
                break;
            }
        }
    } else {
        const value_copy = try allocator.dupe(u8, parsed.value);
        errdefer allocator.free(value_copy);
        var index: usize = 0;
        while (index < entries.items.len) : (index += 1) {
            if (std.mem.eql(u8, entries.items[index].name, name)) {
                allocator.free(entries.items[index].value);
                entries.items[index].value = value_copy;
                entries.items[index].important = parsed.important;
                break;
            }
        }

        if (index == entries.items.len) {
            const name_copy = try allocator.dupe(u8, name);
            errdefer allocator.free(name_copy);
            try entries.append(allocator, .{
                .name = name_copy,
                .value = value_copy,
                .important = parsed.important,
            });
        }
    }

    const css_text = try serializeStyleDeclarations(allocator, entries.items);
    defer allocator.free(css_text);
    try state.set_attribute_fn(state.host, state.element, "style", css_text);
    return;
}

fn styleDeclarationRemoveProperty(
    allocator: std.mem.Allocator,
    state: *StyleDeclarationState,
    property: []const u8,
) errors.Result([]const u8) {
    const name = try stylePropertyName(allocator, property);
    defer allocator.free(name);

    var entries = try styleDeclarationEntries(allocator, state);
    defer freeStyleDeclarations(allocator, &entries);

    var removed_value = try allocator.dupe(u8, "");
    errdefer allocator.free(removed_value);
    var index: usize = 0;
    while (index < entries.items.len) : (index += 1) {
        if (std.mem.eql(u8, entries.items[index].name, name)) {
            allocator.free(removed_value);
            removed_value = try allocator.dupe(u8, entries.items[index].value);
            allocator.free(entries.items[index].name);
            allocator.free(entries.items[index].value);
            _ = entries.orderedRemove(index);
            break;
        }
    }

    const css_text = try serializeStyleDeclarations(allocator, entries.items);
    defer allocator.free(css_text);
    try state.set_attribute_fn(state.host, state.element, "style", css_text);
    return removed_value;
}

fn freeStyleDeclarations(allocator: std.mem.Allocator, entries: *std.ArrayList(StylePropertyEntry)) void {
    for (entries.items) |entry| {
        allocator.free(entry.name);
        allocator.free(entry.value);
    }
    entries.deinit(allocator);
}

fn parseStyleDeclarations(
    allocator: std.mem.Allocator,
    text: []const u8,
) errors.Result(std.ArrayList(StylePropertyEntry)) {
    var entries: std.ArrayList(StylePropertyEntry) = .empty;
    errdefer freeStyleDeclarations(allocator, &entries);

    const stripped = try stripCssComments(allocator, text);
    defer allocator.free(stripped);

    var parts = std.mem.splitScalar(u8, stripped, ';');
    while (parts.next()) |part| {
        const declaration = std.mem.trim(u8, part, " \t\r\n\x0c");
        if (declaration.len == 0) continue;

        const colon_index = std.mem.indexOfScalar(u8, declaration, ':') orelse return error.ScriptRuntime;
        const name_source = std.mem.trim(u8, declaration[0..colon_index], " \t\r\n\x0c");
        const value_source = std.mem.trim(u8, declaration[colon_index + 1 ..], " \t\r\n\x0c");
        const name = try stylePropertyName(allocator, name_source);
        errdefer allocator.free(name);
        const parsed_value = try styleDeclarationValueFromText(allocator, value_source, null);
        defer allocator.free(parsed_value.value);
        const value = try allocator.dupe(u8, parsed_value.value);
        errdefer allocator.free(value);

        var index: usize = 0;
        while (index < entries.items.len) : (index += 1) {
            if (std.mem.eql(u8, entries.items[index].name, name)) {
                allocator.free(entries.items[index].value);
                allocator.free(name);
                entries.items[index].value = value;
                entries.items[index].important = parsed_value.important;
                break;
            }
        }

        if (index == entries.items.len) {
            try entries.append(allocator, .{
                .name = name,
                .value = value,
                .important = parsed_value.important,
            });
        }
    }

    return entries;
}

fn serializeStyleDeclarations(
    allocator: std.mem.Allocator,
    entries: []const StylePropertyEntry,
) errors.Result([]const u8) {
    var out: std.ArrayList(u8) = .empty;
    errdefer out.deinit(allocator);

    for (entries, 0..) |entry, index| {
        if (index > 0) {
            try out.appendSlice(allocator, " ");
        }
        try out.appendSlice(allocator, entry.name);
        try out.appendSlice(allocator, ": ");
        try out.appendSlice(allocator, entry.value);
        if (entry.important) {
            try out.appendSlice(allocator, " !important");
        }
        try out.append(allocator, ';');
    }

    return try allocator.dupe(u8, out.items);
}

fn stripCssComments(
    allocator: std.mem.Allocator,
    text: []const u8,
) errors.Result([]const u8) {
    var out: std.ArrayList(u8) = .empty;
    errdefer out.deinit(allocator);

    var index: usize = 0;
    while (index < text.len) {
        if (index + 1 < text.len and text[index] == '/' and text[index + 1] == '*') {
            const rest = text[index + 2 ..];
            const comment_end = std.mem.indexOf(u8, rest, "*/") orelse return error.ScriptRuntime;
            index += 2 + comment_end + 2;
            continue;
        }

        try out.append(allocator, text[index]);
        index += 1;
    }

    return try allocator.dupe(u8, out.items);
}

fn styleDeclarationValueFromText(
    allocator: std.mem.Allocator,
    text: []const u8,
    priority: ?[]const u8,
) errors.Result(StyleValueEntry) {
    const trimmed_priority = if (priority) |text_value|
        std.mem.trim(u8, text_value, " \t\r\n\x0c")
    else
        "";
    if (trimmed_priority.len != 0 and !std.ascii.eqlIgnoreCase(trimmed_priority, "important")) {
        return error.ScriptRuntime;
    }

    const trimmed_value = std.mem.trim(u8, text, " \t\r\n\x0c");
    var value_text = trimmed_value;
    var important = trimmed_priority.len != 0;
    if (endsWithImportant(trimmed_value)) {
        important = true;
        value_text = std.mem.trimRight(u8, trimmed_value[0 .. trimmed_value.len - "!important".len], " \t\r\n\x0c");
    }

    return .{
        .value = try allocator.dupe(u8, value_text),
        .important = important,
    };
}

fn endsWithImportant(text: []const u8) bool {
    const suffix = "!important";
    if (text.len < suffix.len) return false;
    return std.ascii.eqlIgnoreCase(text[text.len - suffix.len ..], suffix);
}

fn stylePropertyName(
    allocator: std.mem.Allocator,
    text: []const u8,
) errors.Result([]const u8) {
    const trimmed = std.mem.trim(u8, text, " \t\r\n\x0c");
    if (trimmed.len == 0) return error.ScriptRuntime;
    if (std.mem.eql(u8, trimmed, "cssFloat")) return allocator.dupe(u8, "float");

    const has_hyphen = std.mem.indexOfScalar(u8, trimmed, '-') != null;
    var has_lowercase = false;
    for (trimmed) |byte| {
        if (std.ascii.isLower(byte)) {
            has_lowercase = true;
            break;
        }
    }
    const camel_case = !has_hyphen and has_lowercase;

    var out: std.ArrayList(u8) = .empty;
    errdefer out.deinit(allocator);

    for (trimmed, 0..) |byte, index| {
        switch (byte) {
            'A'...'Z' => {
                if (camel_case and index > 0) {
                    try out.append(allocator, '-');
                }
                try out.append(allocator, std.ascii.toLower(byte));
            },
            'a'...'z', '0'...'9', '-', '_' => try out.append(allocator, std.ascii.toLower(byte)),
            else => return error.ScriptRuntime,
        }
    }

    if (out.items.len == 0) return error.ScriptRuntime;
    return try allocator.dupe(u8, out.items);
}

fn historyDeltaFromExpr(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    expr: *Expr,
) errors.Result(isize) {
    const value = try evalExpr(allocator, host, bindings, expr);
    return try historyDeltaFromValue(allocator, value);
}

fn historyDeltaFromValue(allocator: std.mem.Allocator, value: Value) errors.Result(isize) {
    return switch (value) {
        .number => |number| blk: {
            if (!std.math.isFinite(number)) return error.ScriptRuntime;
            if (std.math.round(number) != number) return error.ScriptRuntime;
            const as_i64 = @as(i64, @intFromFloat(number));
            break :blk std.math.cast(isize, as_i64) orelse error.ScriptRuntime;
        },
        else => blk: {
            const text = try asString(allocator, value);
            const trimmed = std.mem.trim(u8, text, " \t\r\n");
            break :blk std.fmt.parseInt(isize, trimmed, 10) catch error.ScriptRuntime;
        },
    };
}

fn scrollCoordinate(value: Value, method: []const u8) errors.Result(i64) {
    return switch (value) {
        .number => |number| blk: {
            if (!std.math.isFinite(number)) return error.ScriptRuntime;
            if (std.math.round(number) != number) return error.ScriptRuntime;
            const min = @as(f64, @floatFromInt(std.math.minInt(i64)));
            const max = @as(f64, @floatFromInt(std.math.maxInt(i64)));
            if (number < min or number > max) return error.ScriptRuntime;
            break :blk @as(i64, @intFromFloat(number));
        },
        .string => |text| std.fmt.parseInt(i64, text, 10) catch {
            _ = method;
            return error.ScriptRuntime;
        },
        else => {
            _ = method;
            return error.ScriptRuntime;
        },
    };
}

fn historyStateFromValue(allocator: std.mem.Allocator, value: Value) errors.Result(?[]const u8) {
    return switch (value) {
        .undefined_value, .null_value => null,
        else => try asString(allocator, value),
    };
}

fn selectionIndexFromValue(allocator: std.mem.Allocator, value: Value) errors.Result(usize) {
    return switch (value) {
        .number => |number| blk: {
            if (!std.math.isFinite(number)) return error.ScriptRuntime;
            if (std.math.round(number) != number) return error.ScriptRuntime;
            if (number < 0) return error.ScriptRuntime;
            const max_index: f64 = @floatFromInt(std.math.maxInt(usize));
            if (number > max_index) return error.ScriptRuntime;
            break :blk @as(usize, @intFromFloat(number));
        },
        else => blk: {
            const text = try asString(allocator, value);
            const trimmed = std.mem.trim(u8, text, " \t\r\n");
            if (trimmed.len == 0) return error.ScriptRuntime;
            const parsed = std.fmt.parseInt(i64, trimmed, 10) catch return error.ScriptRuntime;
            if (parsed < 0) return error.ScriptRuntime;
            break :blk @as(usize, @intCast(parsed));
        },
    };
}

fn tabIndexFromValue(allocator: std.mem.Allocator, value: Value) errors.Result(i64) {
    return switch (value) {
        .number => |number| blk: {
            if (!std.math.isFinite(number)) return error.ScriptRuntime;
            if (std.math.round(number) != number) return error.ScriptRuntime;
            const min_index: f64 = @floatFromInt(std.math.minInt(i64));
            const max_index: f64 = @floatFromInt(std.math.maxInt(i64));
            if (number < min_index or number > max_index) return error.ScriptRuntime;
            break :blk @as(i64, @intFromFloat(number));
        },
        else => blk: {
            const text = try asString(allocator, value);
            const trimmed = std.mem.trim(u8, text, " \t\r\n");
            if (trimmed.len == 0) return error.ScriptRuntime;
            break :blk std.fmt.parseInt(i64, trimmed, 10) catch error.ScriptRuntime;
        },
    };
}

fn selectionDirectionFromString(text: []const u8) ?dom.SelectionDirection {
    if (std.ascii.eqlIgnoreCase(text, "forward")) return .forward;
    if (std.ascii.eqlIgnoreCase(text, "backward")) return .backward;
    if (std.ascii.eqlIgnoreCase(text, "none")) return .none;
    return null;
}

fn selectionDirectionName(direction: dom.SelectionDirection) []const u8 {
    return switch (direction) {
        .forward => "forward",
        .backward => "backward",
        .none => "none",
    };
}

fn timerDelayFromExpr(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    expr: *Expr,
) errors.Result(i64) {
    const value = try evalExpr(allocator, host, bindings, expr);
    return try timerDelayFromValue(allocator, value);
}

fn timerDelayFromValue(allocator: std.mem.Allocator, value: Value) errors.Result(i64) {
    return switch (value) {
        .number => |number| blk: {
            if (!std.math.isFinite(number)) return error.ScriptRuntime;
            if (std.math.round(number) != number) return error.ScriptRuntime;
            const min = @as(f64, @floatFromInt(std.math.minInt(i64)));
            const max = @as(f64, @floatFromInt(std.math.maxInt(i64)));
            if (number < min or number > max) return error.ScriptRuntime;
            const delay = @as(i64, @intFromFloat(number));
            break :blk if (delay < 0) 0 else delay;
        },
        else => blk: {
            const text = try asString(allocator, value);
            const trimmed = std.mem.trim(u8, text, " \t\r\n");
            const parsed = std.fmt.parseInt(i64, trimmed, 10) catch return error.ScriptRuntime;
            break :blk if (parsed < 0) 0 else parsed;
        },
    };
}

fn timerIdFromExpr(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    expr: *Expr,
) errors.Result(?u64) {
    const value = try evalExpr(allocator, host, bindings, expr);
    return timerIdFromValue(allocator, value);
}

fn timerIdFromValue(allocator: std.mem.Allocator, value: Value) errors.Result(?u64) {
    return switch (value) {
        .number => |number| blk: {
            if (!std.math.isFinite(number)) break :blk null;
            if (std.math.round(number) != number) break :blk null;
            const min = @as(f64, @floatFromInt(@as(i64, 0)));
            const max = @as(f64, @floatFromInt(std.math.maxInt(u64)));
            if (number < min or number > max) break :blk null;
            const id = @as(u64, @intFromFloat(number));
            break :blk id;
        },
        else => blk: {
            const text = try asString(allocator, value);
            const trimmed = std.mem.trim(u8, text, " \t\r\n");
            const parsed = std.fmt.parseInt(i64, trimmed, 10) catch return null;
            if (parsed < 0) break :blk null;
            break :blk @as(u64, @intCast(parsed));
        },
    };
}

fn classListTokens(
    allocator: std.mem.Allocator,
    host: anytype,
    element: dom.NodeId,
) errors.Result(std.ArrayList([]const u8)) {
    var tokens: std.ArrayList([]const u8) = .empty;
    errdefer tokens.deinit(allocator);

    const class_value = (try host.domStore().getAttribute(element, "class")) orelse "";
    var iter = std.mem.tokenizeAny(u8, class_value, " \t\r\n\x0c");
    while (iter.next()) |candidate| {
        if (classListContains(tokens.items, candidate)) continue;
        try tokens.append(allocator, candidate);
    }

    return tokens;
}

fn classListContains(tokens: []const []const u8, token: []const u8) bool {
    for (tokens) |candidate| {
        if (std.mem.eql(u8, candidate, token)) return true;
    }
    return false;
}

fn validateClassListToken(
    allocator: std.mem.Allocator,
    value: Value,
) errors.Result([]const u8) {
    const token = try asString(allocator, value);
    const trimmed = std.mem.trim(u8, token, " \t\r\n\x0c");
    if (trimmed.len == 0 or trimmed.len != token.len) return error.ScriptRuntime;
    for (trimmed) |byte| {
        if (isWhitespaceByte(byte)) return error.ScriptRuntime;
    }
    return trimmed;
}

fn writeClassListTokens(
    allocator: std.mem.Allocator,
    host: anytype,
    element: dom.NodeId,
    tokens: []const []const u8,
) errors.Result(void) {
    var joined: std.ArrayList(u8) = .empty;
    errdefer joined.deinit(allocator);

    for (tokens, 0..) |token, index| {
        if (index > 0) {
            try joined.appendSlice(allocator, " ");
        }
        try joined.appendSlice(allocator, token);
    }

    host.domStoreMut().setAttribute(element, "class", joined.items) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return;
}

fn datasetAttributeName(
    allocator: std.mem.Allocator,
    property: []const u8,
) errors.Result([]const u8) {
    const trimmed = std.mem.trim(u8, property, " \t\r\n\x0c");
    if (trimmed.len == 0) return error.ScriptRuntime;

    var out: std.ArrayList(u8) = .empty;
    errdefer out.deinit(allocator);
    try out.appendSlice(allocator, "data-");

    for (trimmed) |byte| {
        switch (byte) {
            'A'...'Z' => {
                try out.append(allocator, '-');
                try out.append(allocator, std.ascii.toLower(byte));
            },
            'a'...'z', '0'...'9', '_', '$' => try out.append(allocator, byte),
            else => return error.ScriptRuntime,
        }
    }

    return try allocator.dupe(u8, out.items);
}

fn asNodeListIndex(value: Value) errors.Result(usize) {
    return switch (value) {
        .number => |number| blk: {
            if (!std.math.isFinite(number)) return error.ScriptRuntime;
            if (number < 0) return error.ScriptRuntime;
            const floored = std.math.floor(number);
            if (floored != number) return error.ScriptRuntime;
            const max_index: f64 = @floatFromInt(std.math.maxInt(usize));
            if (number > max_index) return error.ScriptRuntime;
            break :blk @as(usize, @intFromFloat(number));
        },
        else => return error.ScriptRuntime,
    };
}

fn optionalNodeListIndex(value: Value) ?usize {
    return switch (value) {
        .number => |number| blk: {
            if (!std.math.isFinite(number)) break :blk null;
            if (number < 0) break :blk null;
            const floored = std.math.floor(number);
            if (floored != number) break :blk null;
            const max_index: f64 = @floatFromInt(std.math.maxInt(usize));
            if (number > max_index) break :blk null;
            break :blk @as(usize, @intFromFloat(number));
        },
        else => null,
    };
}

fn parseNumberLiteral(text: []const u8) errors.Result(f64) {
    return std.fmt.parseFloat(f64, text) catch error.ScriptParse;
}

fn isWhitespaceByte(byte: u8) bool {
    return switch (byte) {
        ' ', '\t', '\n', '\r', 0x0c => true,
        else => false,
    };
}

fn isIdentifierStartByte(byte: u8) bool {
    return std.ascii.isAlphabetic(byte) or byte == '_' or byte == '$';
}

fn isIdentifierContinueByte(byte: u8) bool {
    return std.ascii.isAlphanumeric(byte) or byte == '_' or byte == '$';
}
