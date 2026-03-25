const std = @import("std");

const errors = @import("errors.zig");

pub const NodeId = struct {
    index: u32,
    generation: u32,

    pub fn new(index: u32, generation: u32) NodeId {
        return .{ .index = index, .generation = generation };
    }
};

pub const Attribute = struct {
    name: []const u8,
    value: []const u8,
};

pub const ElementData = struct {
    tag_name: []const u8,
    namespace_uri: []const u8 = "http://www.w3.org/1999/xhtml",
    attributes: std.ArrayListUnmanaged(Attribute) = .{},
    dialog_return_value: []const u8 = "",
    output_default_value: ?[]const u8 = null,
    popover_visible: bool = false,
};

pub const NodeKind = union(enum) {
    document,
    element: ElementData,
    text: []const u8,
    comment: []const u8,
};

pub const SelectionDirection = enum {
    forward,
    backward,
    none,
};

pub const SelectionState = struct {
    start: usize,
    end: usize,
    direction: SelectionDirection = .none,
};

pub const RangeTextSelectionMode = enum {
    preserve,
    select,
    start,
    end,
};

pub const PopoverState = enum {
    no_popover,
    auto,
    hint,
    manual,
};

pub const MediaPlaybackState = struct {
    current_time: f64 = 0.0,
    duration: f64 = std.math.nan(f64),
    volume: f64 = 1.0,
    default_playback_rate: f64 = 1.0,
    playback_rate: f64 = 1.0,
    preserves_pitch: bool = true,
    paused: bool = true,
    seeking: bool = false,
    ended: bool = false,
    ready_state: u8 = 0,
    network_state: u8 = 0,
};

pub const TextTrackMode = enum {
    disabled,
    hidden,
    showing,
};

pub const TextTrackState = struct {
    element: NodeId,
    mode: TextTrackMode = .disabled,
};

const SerializationNamespace = enum {
    html,
    svg,
    mathml,
};

const HtmlNamespaceUri = "http://www.w3.org/1999/xhtml";
const SvgNamespaceUri = "http://www.w3.org/2000/svg";
const MathMlNamespaceUri = "http://www.w3.org/1998/Math/MathML";

fn namespaceUriForSerializationNamespace(namespace: SerializationNamespace) []const u8 {
    return switch (namespace) {
        .html => HtmlNamespaceUri,
        .svg => SvgNamespaceUri,
        .mathml => MathMlNamespaceUri,
    };
}

fn serializationNamespaceFromUri(namespace_uri: []const u8) SerializationNamespace {
    if (std.mem.eql(u8, namespace_uri, SvgNamespaceUri)) return .svg;
    if (std.mem.eql(u8, namespace_uri, MathMlNamespaceUri)) return .mathml;
    return .html;
}

const AdjacentInsertion = struct {
    parent: NodeId,
    index: usize,
};

pub const NodeRecord = struct {
    id: NodeId,
    parent: ?NodeId,
    children: std.ArrayListUnmanaged(NodeId) = .{},
    kind: NodeKind,
};

const SelectorAttributeCaseSensitivity = enum {
    case_sensitive,
    case_insensitive,
};

const SelectorCombinator = enum {
    descendant,
    child,
    adjacent_sibling,
    general_sibling,
};

const SelectorDirValue = enum {
    ltr,
    rtl,
};

const SelectorPseudoClass = union(enum) {
    scope,
    root,
    empty,
    first_child,
    last_child,
    only_child,
    first_of_type,
    last_of_type,
    only_of_type,
    checked,
    disabled,
    enabled,
    required,
    optional,
    link,
    any_link,
    defined,
    placeholder_shown,
    blank,
    indeterminate,
    default,
    valid,
    invalid,
    in_range,
    out_of_range,
    read_only,
    read_write,
    target,
    focus,
    focus_visible,
    focus_within,
    popover_open,
    lang: std.ArrayListUnmanaged([]const u8),
    dir: SelectorDirValue,
    not: std.ArrayListUnmanaged(SelectorChain),
    is: std.ArrayListUnmanaged(SelectorChain),
    where: std.ArrayListUnmanaged(SelectorChain),
    has: std.ArrayListUnmanaged(SelectorRelativeSelector),
    nth_child: SelectorNthChildPattern,
    nth_last_child: SelectorNthChildPattern,
    nth_of_type: SelectorNthChildPattern,
    nth_last_of_type: SelectorNthChildPattern,

    fn deinit(self: *SelectorPseudoClass, allocator: std.mem.Allocator) void {
        switch (self.*) {
            .lang => |*langs| {
                langs.deinit(allocator);
            },
            .has => |*relative_selectors| {
                for (relative_selectors.items) |*relative_selector| {
                    relative_selector.deinit(allocator);
                }
                relative_selectors.deinit(allocator);
            },
            .not, .is, .where => |*selectors| {
                for (selectors.items) |*chain| {
                    chain.deinit(allocator);
                }
                selectors.deinit(allocator);
            },
            .nth_child, .nth_last_child, .nth_of_type, .nth_last_of_type => |*pattern| {
                pattern.deinit(allocator);
            },
            else => {},
        }
    }
};

const SelectorAttributeOperator = enum {
    exists,
    equals,
    prefix,
    suffix,
    contains,
    contains_word,
    hyphen_prefix,
};

const SelectorAttribute = struct {
    name: []const u8,
    operator: SelectorAttributeOperator,
    value: ?[]const u8 = null,
    case_sensitivity: SelectorAttributeCaseSensitivity = .case_sensitive,
};

const SelectorQuery = struct {
    universal: bool = false,
    tag: ?[]const u8 = null,
    id: ?[]const u8 = null,
    classes: std.ArrayListUnmanaged([]const u8) = .{},
    attributes: std.ArrayListUnmanaged(SelectorAttribute) = .{},
    pseudos: std.ArrayListUnmanaged(SelectorPseudoClass) = .{},

    fn deinit(self: *SelectorQuery, allocator: std.mem.Allocator) void {
        self.classes.deinit(allocator);
        self.attributes.deinit(allocator);
        for (self.pseudos.items) |*pseudo| {
            pseudo.deinit(allocator);
        }
        self.pseudos.deinit(allocator);
    }
};

const SelectorChain = struct {
    parts: std.ArrayListUnmanaged(SelectorQuery) = .{},
    relations: std.ArrayListUnmanaged(SelectorCombinator) = .{},

    fn deinit(self: *SelectorChain, allocator: std.mem.Allocator) void {
        for (self.parts.items) |*part| {
            part.deinit(allocator);
        }
        self.parts.deinit(allocator);
        self.relations.deinit(allocator);
    }
};

const SelectorNthChildPattern = struct {
    step: isize,
    offset: isize,
    of_selectors: ?std.ArrayListUnmanaged(SelectorChain) = null,

    fn deinit(self: *SelectorNthChildPattern, allocator: std.mem.Allocator) void {
        if (self.of_selectors) |*selectors| {
            for (selectors.items) |*chain| {
                chain.deinit(allocator);
            }
            selectors.deinit(allocator);
        }
    }
};

const SelectorRelativeSelector = struct {
    combinator: ?SelectorCombinator = null,
    chain: SelectorChain,

    fn deinit(self: *SelectorRelativeSelector, allocator: std.mem.Allocator) void {
        self.chain.deinit(allocator);
    }
};

pub const DomStore = struct {
    allocator: std.mem.Allocator,
    arena: std.heap.ArenaAllocator,
    nodes: std.ArrayListUnmanaged(NodeRecord) = .{},
    source_html: ?[]const u8 = null,
    document_title: []const u8 = "",
    focused_node: ?NodeId = null,
    target_fragment: ?[]const u8 = null,
    selection: std.AutoHashMapUnmanaged(NodeId, SelectionState) = .{},
    document_selection_cleared: bool = false,
    media_muted: std.AutoHashMapUnmanaged(NodeId, bool) = .{},
    media_playback: std.AutoHashMapUnmanaged(NodeId, MediaPlaybackState) = .{},
    text_track: std.AutoHashMapUnmanaged(NodeId, TextTrackState) = .{},
    indeterminate: std.AutoHashMapUnmanaged(NodeId, void) = .{},
    custom_validity: std.AutoHashMapUnmanaged(NodeId, []const u8) = .{},

    pub fn init(allocator: std.mem.Allocator) errors.Result(DomStore) {
        var arena = std.heap.ArenaAllocator.init(allocator);
        errdefer arena.deinit();

        var store = DomStore{
            .allocator = allocator,
            .arena = arena,
            .nodes = .{},
            .source_html = null,
            .document_title = "",
            .focused_node = null,
            .target_fragment = null,
            .selection = .{},
            .document_selection_cleared = false,
            .media_muted = .{},
            .media_playback = .{},
            .text_track = .{},
            .indeterminate = .{},
            .custom_validity = .{},
        };
        const arena_alloc = store.arena.allocator();
        try store.nodes.append(arena_alloc, .{
            .id = NodeId.new(0, 0),
            .parent = null,
            .children = .{},
            .kind = .document,
        });
        return store;
    }

    pub fn deinit(self: *DomStore) void {
        var custom_values = self.custom_validity.valueIterator();
        while (custom_values.next()) |message| {
            self.allocator.free(message.*);
        }
        self.media_playback.deinit(self.allocator);
        self.media_muted.deinit(self.allocator);
        self.text_track.deinit(self.allocator);
        self.indeterminate.deinit(self.allocator);
        self.custom_validity.deinit(self.allocator);
        self.selection.deinit(self.allocator);
        self.arena.deinit();
    }

    pub fn documentId(self: *const DomStore) NodeId {
        _ = self;
        return NodeId.new(0, 0);
    }

    pub fn documentElement(self: *const DomStore) ?NodeId {
        return documentElementId(self);
    }

    pub fn documentHead(self: *const DomStore) ?NodeId {
        const root = documentElementId(self) orelse return null;
        const root_tag = self.tagNameForNode(root) orelse return null;
        if (!std.mem.eql(u8, root_tag, "html")) return null;

        const root_node = self.nodeAt(root) orelse return null;
        for (root_node.children.items) |child_id| {
            const child_tag = self.tagNameForNode(child_id) orelse continue;
            if (std.mem.eql(u8, child_tag, "head")) {
                return child_id;
            }
        }

        return null;
    }

    pub fn documentBody(self: *const DomStore) ?NodeId {
        const root = documentElementId(self) orelse return null;
        const root_tag = self.tagNameForNode(root) orelse return null;
        if (!std.mem.eql(u8, root_tag, "html")) return null;

        const root_node = self.nodeAt(root) orelse return null;
        for (root_node.children.items) |child_id| {
            const child_tag = self.tagNameForNode(child_id) orelse continue;
            if (std.mem.eql(u8, child_tag, "body")) {
                return child_id;
            }
        }

        return null;
    }

    pub fn documentTitle(self: *const DomStore) []const u8 {
        return self.document_title;
    }

    pub fn sourceHtml(self: *const DomStore) ?[]const u8 {
        return self.source_html;
    }

    pub fn focusedNode(self: *const DomStore) ?NodeId {
        return self.focused_node;
    }

    pub fn setFocusedNode(self: *DomStore, focused_node: ?NodeId) void {
        self.focused_node = focused_node;
    }

    pub fn setCustomValidity(self: *DomStore, node_id: NodeId, message: []const u8) errors.Result(void) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return error.DomError,
        };

        if (!std.mem.eql(u8, element.tag_name, "input") and
            !std.mem.eql(u8, element.tag_name, "select") and
            !std.mem.eql(u8, element.tag_name, "textarea") and
            !std.mem.eql(u8, element.tag_name, "output") and
            !std.mem.eql(u8, element.tag_name, "object"))
        {
            return error.DomError;
        }

        if (message.len == 0) {
            if (self.custom_validity.fetchRemove(node_id)) |entry| {
                self.allocator.free(entry.value);
            }
            return;
        }

        const message_copy = try self.allocator.dupe(u8, message);
        errdefer self.allocator.free(message_copy);

        if (self.custom_validity.getPtr(node_id)) |stored_message| {
            const previous = stored_message.*;
            stored_message.* = message_copy;
            self.allocator.free(previous);
            return;
        }

        try self.custom_validity.put(self.allocator, node_id, message_copy);
    }

    pub fn mediaVolumeForNode(self: *const DomStore, node_id: NodeId) errors.Result(f64) {
        const state = try self.mediaPlaybackStateForNode(node_id);
        return state.volume;
    }

    pub fn mediaCurrentTimeForNode(self: *const DomStore, node_id: NodeId) errors.Result(f64) {
        const state = try self.mediaPlaybackStateForNode(node_id);
        return state.current_time;
    }

    pub fn mediaDurationForNode(self: *const DomStore, node_id: NodeId) errors.Result(f64) {
        const state = try self.mediaPlaybackStateForNode(node_id);
        return state.duration;
    }

    pub fn mediaPausedForNode(self: *const DomStore, node_id: NodeId) errors.Result(bool) {
        const state = try self.mediaPlaybackStateForNode(node_id);
        return state.paused;
    }

    pub fn mediaSeekingForNode(self: *const DomStore, node_id: NodeId) errors.Result(bool) {
        const state = try self.mediaPlaybackStateForNode(node_id);
        return state.seeking;
    }

    pub fn mediaEndedForNode(self: *const DomStore, node_id: NodeId) errors.Result(bool) {
        const state = try self.mediaPlaybackStateForNode(node_id);
        return state.ended;
    }

    pub fn mediaReadyStateForNode(self: *const DomStore, node_id: NodeId) errors.Result(u8) {
        const state = try self.mediaPlaybackStateForNode(node_id);
        return state.ready_state;
    }

    pub fn mediaNetworkStateForNode(self: *const DomStore, node_id: NodeId) errors.Result(u8) {
        const state = try self.mediaPlaybackStateForNode(node_id);
        return state.network_state;
    }

    pub fn setMediaCurrentTime(self: *DomStore, node_id: NodeId, current_time: f64) errors.Result(void) {
        if (!std.math.isFinite(current_time)) return error.DomError;
        if (current_time < 0.0) return error.DomError;
        const state = try self.ensureMediaPlaybackState(node_id);
        state.current_time = current_time;
    }

    pub fn setMediaVolume(self: *DomStore, node_id: NodeId, volume: f64) errors.Result(void) {
        if (!std.math.isFinite(volume)) return error.DomError;
        if (volume < 0.0 or volume > 1.0) return error.DomError;
        const state = try self.ensureMediaPlaybackState(node_id);
        state.volume = volume;
    }

    pub fn mediaMutedForNode(self: *const DomStore, node_id: NodeId) errors.Result(bool) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return error.DomError,
        };
        if (!self.isMediaElementNode(element.tag_name)) return error.DomError;
        if (self.media_muted.get(node_id)) |muted| return muted;
        return self.hasAttribute(node_id, "muted");
    }

    pub fn setMediaMuted(self: *DomStore, node_id: NodeId, muted: bool) errors.Result(void) {
        const state = try self.ensureMediaPlaybackState(node_id);
        _ = state;
        try self.media_muted.put(self.allocator, node_id, muted);
    }

    pub fn mediaDefaultPlaybackRateForNode(self: *const DomStore, node_id: NodeId) errors.Result(f64) {
        const state = try self.mediaPlaybackStateForNode(node_id);
        return state.default_playback_rate;
    }

    pub fn setMediaDefaultPlaybackRate(self: *DomStore, node_id: NodeId, playback_rate: f64) errors.Result(void) {
        if (!std.math.isFinite(playback_rate)) return error.DomError;
        const state = try self.ensureMediaPlaybackState(node_id);
        state.default_playback_rate = playback_rate;
    }

    pub fn mediaPlaybackRateForNode(self: *const DomStore, node_id: NodeId) errors.Result(f64) {
        const state = try self.mediaPlaybackStateForNode(node_id);
        return state.playback_rate;
    }

    pub fn setMediaPlaybackRate(self: *DomStore, node_id: NodeId, playback_rate: f64) errors.Result(void) {
        if (!std.math.isFinite(playback_rate)) return error.DomError;
        const state = try self.ensureMediaPlaybackState(node_id);
        state.playback_rate = playback_rate;
    }

    pub fn mediaPreservesPitchForNode(self: *const DomStore, node_id: NodeId) errors.Result(bool) {
        const state = try self.mediaPlaybackStateForNode(node_id);
        return state.preserves_pitch;
    }

    pub fn setMediaPreservesPitch(self: *DomStore, node_id: NodeId, preserves_pitch: bool) errors.Result(void) {
        const state = try self.ensureMediaPlaybackState(node_id);
        state.preserves_pitch = preserves_pitch;
    }

    pub fn dialogReturnValue(self: *const DomStore, node_id: NodeId) errors.Result([]const u8) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return error.DomError,
        };
        if (!std.mem.eql(u8, element.tag_name, "dialog")) return error.DomError;
        return element.dialog_return_value;
    }

    pub fn setDialogReturnValue(self: *DomStore, node_id: NodeId, value: []const u8) errors.Result(void) {
        const node = self.nodeAtMut(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |*element| element,
            else => return error.DomError,
        };
        if (!std.mem.eql(u8, element.tag_name, "dialog")) return error.DomError;
        element.dialog_return_value = try duplicateString(self, value);
        return;
    }

    pub fn outputDefaultValue(self: *const DomStore, allocator: std.mem.Allocator, node_id: NodeId) errors.Result([]const u8) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return error.DomError,
        };
        if (!std.mem.eql(u8, element.tag_name, "output")) return error.DomError;
        if (element.output_default_value) |default_value| {
            return allocator.dupe(u8, default_value);
        }
        return self.textContent(allocator, node_id);
    }

    pub fn outputHasDefaultValueOverride(self: *const DomStore, node_id: NodeId) errors.Result(bool) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return error.DomError,
        };
        if (!std.mem.eql(u8, element.tag_name, "output")) return error.DomError;
        return element.output_default_value != null;
    }

    pub fn setOutputDefaultValue(self: *DomStore, node_id: NodeId, value: []const u8) errors.Result(void) {
        const node = self.nodeAtMut(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |*element| element,
            else => return error.DomError,
        };
        if (!std.mem.eql(u8, element.tag_name, "output")) return error.DomError;
        element.output_default_value = try duplicateString(self, value);
        return;
    }

    pub fn validationMessageForNode(
        self: *const DomStore,
        allocator: std.mem.Allocator,
        node_id: NodeId,
    ) errors.Result([]const u8) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return error.DomError,
        };

        if (!std.mem.eql(u8, element.tag_name, "input") and
            !std.mem.eql(u8, element.tag_name, "select") and
            !std.mem.eql(u8, element.tag_name, "textarea") and
            !std.mem.eql(u8, element.tag_name, "output") and
            !std.mem.eql(u8, element.tag_name, "object"))
        {
            return error.DomError;
        }

        if (isBarredFromConstraintValidation(self, node_id)) {
            return allocator.dupe(u8, "");
        }

        if (self.custom_validity.get(node_id)) |message| {
            return allocator.dupe(u8, message);
        }

        if (isInvalidPseudoClass(self, node_id)) {
            return allocator.dupe(u8, "Constraints not satisfied");
        }

        return allocator.dupe(u8, "");
    }

    pub fn activeElement(self: *const DomStore) ?NodeId {
        if (self.focused_node) |focused| {
            return focused;
        }

        if (self.documentBody()) |body| {
            return body;
        }

        return self.documentElement();
    }

    pub fn targetFragment(self: *const DomStore) ?[]const u8 {
        return self.target_fragment;
    }

    pub fn setTargetFragment(self: *DomStore, fragment: ?[]const u8) errors.Result(void) {
        self.target_fragment = if (fragment) |value|
            try self.arena.allocator().dupe(u8, value)
        else
            null;
        return;
    }

    pub fn setDocumentTitle(self: *DomStore, title: []const u8) errors.Result(void) {
        if (self.findFirstElementByTagNameInSubtree(self.documentId(), "title")) |title_id| {
            try self.setTextContent(title_id, title);
            return;
        }

        self.document_title = try duplicateString(self, title);
        return;
    }

    pub fn records(self: *const DomStore) []const NodeRecord {
        return self.nodes.items;
    }

    pub fn nodeCount(self: *const DomStore) usize {
        return self.nodes.items.len;
    }

    pub fn nodeAt(self: *const DomStore, node_id: NodeId) ?*const NodeRecord {
        const index: usize = @intCast(node_id.index);
        if (index >= self.nodes.items.len) return null;
        return &self.nodes.items[index];
    }

    pub fn nodeAtMut(self: *DomStore, node_id: NodeId) ?*NodeRecord {
        const index: usize = @intCast(node_id.index);
        if (index >= self.nodes.items.len) return null;
        return &self.nodes.items[index];
    }

    pub fn childIds(self: *const DomStore, node_id: NodeId) []const NodeId {
        if (self.nodeAt(node_id)) |node| {
            return node.children.items;
        }
        return &.{};
    }

    pub fn findElementById(self: *const DomStore, id: []const u8) ?NodeId {
        return self.findElementByIdInSubtree(self.documentId(), id);
    }

    pub fn findElementByIdWithin(self: *const DomStore, node_id: NodeId, id: []const u8) ?NodeId {
        const node = self.nodeAt(node_id) orelse return null;
        for (node.children.items) |child_id| {
            if (self.findElementByIdInSubtree(child_id, id)) |found| {
                return found;
            }
        }
        return null;
    }

    pub fn tagNameForNode(self: *const DomStore, node_id: NodeId) ?[]const u8 {
        const node = self.nodeAt(node_id) orelse return null;
        return switch (node.kind) {
            .element => |element| element.tag_name,
            else => null,
        };
    }

    pub fn namespaceUriForNode(self: *const DomStore, node_id: NodeId) ?[]const u8 {
        const node = self.nodeAt(node_id) orelse return null;
        return switch (node.kind) {
            .element => |element| element.namespace_uri,
            else => null,
        };
    }

    pub fn nodeNameForNode(self: *const DomStore, node_id: NodeId) ?[]const u8 {
        const node = self.nodeAt(node_id) orelse return null;
        return switch (node.kind) {
            .document => "#document",
            .element => |element| element.tag_name,
            .text => "#text",
            .comment => "#comment",
        };
    }

    pub fn nodeTypeForNode(self: *const DomStore, node_id: NodeId) ?u8 {
        const node = self.nodeAt(node_id) orelse return null;
        return switch (node.kind) {
            .document => 9,
            .element => 1,
            .text => 3,
            .comment => 8,
        };
    }

    pub fn select(self: *const DomStore, allocator: std.mem.Allocator, selector: []const u8) errors.Result([]NodeId) {
        return try self.selectWithin(allocator, self.documentId(), selector);
    }

    pub fn selectWithin(
        self: *const DomStore,
        allocator: std.mem.Allocator,
        root_id: NodeId,
        selector: []const u8,
    ) errors.Result([]NodeId) {
        var chains = try self.parseSelectorChains(allocator, selector);
        defer self.deinitSelectorChains(allocator, &chains);

        var results: std.ArrayList(NodeId) = .empty;
        errdefer results.deinit(allocator);

        const scope_root = selectorScopeRootForSearch(self, root_id);
        try collectSelectorMatchesWithin(self, root_id, chains.items, scope_root, &results, allocator);

        const owned = try allocator.dupe(NodeId, results.items);
        results.deinit(allocator);
        return owned;
    }

    pub fn querySelector(self: *const DomStore, allocator: std.mem.Allocator, selector: []const u8) errors.Result(?NodeId) {
        return self.querySelectorWithin(allocator, self.documentId(), selector);
    }

    pub fn querySelectorWithin(
        self: *const DomStore,
        allocator: std.mem.Allocator,
        root_id: NodeId,
        selector: []const u8,
    ) errors.Result(?NodeId) {
        var chains = try self.parseSelectorChains(allocator, selector);
        defer self.deinitSelectorChains(allocator, &chains);

        const scope_root = selectorScopeRootForSearch(self, root_id);
        return self.findFirstMatchingDescendant(root_id, chains.items, scope_root);
    }

    pub fn matchesSelector(
        self: *const DomStore,
        allocator: std.mem.Allocator,
        node_id: NodeId,
        selector: []const u8,
    ) errors.Result(bool) {
        var chains = try self.parseSelectorChains(allocator, selector);
        defer self.deinitSelectorChains(allocator, &chains);

        return nodeMatchesAnyChain(self, node_id, chains.items, node_id);
    }

    pub fn closestSelector(
        self: *const DomStore,
        allocator: std.mem.Allocator,
        node_id: NodeId,
        selector: []const u8,
    ) errors.Result(?NodeId) {
        var chains = try self.parseSelectorChains(allocator, selector);
        defer self.deinitSelectorChains(allocator, &chains);

        var current = node_id;
        while (true) {
            if (nodeMatchesAnyChain(self, current, chains.items, node_id)) {
                return current;
            }
            current = parentOf(self, current) orelse break;
        }

        return null;
    }

    pub fn bootstrapHtml(self: *DomStore, html: []const u8) errors.Result(void) {
        var parsed = try DomStore.init(self.allocator);
        errdefer parsed.deinit();

        const parsed_alloc = parsed.arena.allocator();
        parsed.source_html = try parsed_alloc.dupe(u8, html);
        var parser = HtmlParser.init(html);
        try parser.parseInto(&parsed);
        try parsed.syncDocumentTitleFromDom();

        self.deinit();
        self.* = parsed;
    }

    pub fn dumpDom(self: *const DomStore, allocator: std.mem.Allocator) errors.Result([]u8) {
        var output: std.ArrayList(u8) = .empty;
        errdefer output.deinit(allocator);

        try self.dumpNode(self.documentId(), 0, &output, allocator);
        const result = try allocator.dupe(u8, output.items);
        output.deinit(allocator);
        return result;
    }

    pub fn innerHtml(self: *const DomStore, allocator: std.mem.Allocator, node_id: NodeId) errors.Result([]u8) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        switch (node.kind) {
            .document, .element => {},
            else => return error.DomError,
        }

        var output: std.ArrayList(u8) = .empty;
        errdefer output.deinit(allocator);

        const raw_text_context = switch (node.kind) {
            .element => |element| isRawTextElement(element.tag_name),
            else => false,
        };

        for (node.children.items) |child_id| {
            try self.serializeHtmlNodeWithContext(child_id, &output, allocator, raw_text_context);
        }

        const result = try allocator.dupe(u8, output.items);
        output.deinit(allocator);
        return result;
    }

    pub fn setInnerHtml(self: *DomStore, node_id: NodeId, html: []const u8) errors.Result(void) {
        const node = self.nodeAtMut(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |*element| element,
            else => return error.DomError,
        };
        if (isVoidElement(element.tag_name)) {
            return error.DomError;
        }

        var fragment_store = try DomStore.init(self.allocator);
        defer fragment_store.deinit();

        const fragment_root = try self.parseHtmlFragmentIntoStore(&fragment_store, node_id, html);
        const fragment_children = fragment_store.childIds(fragment_root);

        const old_children = node.children.items;
        for (old_children) |old_child| {
            if (self.nodeAtMut(old_child)) |child_record| {
                child_record.parent = null;
            }
        }
        node.children.items.len = 0;

        try self.cloneFragmentChildrenInto(&fragment_store, fragment_children, node_id, 0, true);
        try self.syncDocumentTitleFromDom();
        if (std.mem.eql(u8, element.tag_name, "textarea")) {
            try self.resetSelectionToEnd(node_id);
        }
        return;
    }

    pub fn createContextualFragment(self: *DomStore, context_parent: NodeId, html: []const u8) errors.Result(NodeId) {
        var fragment_store = try DomStore.init(self.allocator);
        defer fragment_store.deinit();

        const fragment_root = try self.parseHtmlFragmentIntoStore(&fragment_store, context_parent, html);
        const fragment_children = fragment_store.childIds(fragment_root);

        const fragment = try self.createElementDetached("template");
        try self.cloneFragmentChildrenInto(&fragment_store, fragment_children, fragment, 0, true);
        return fragment;
    }

    pub fn clearDocument(self: *DomStore) errors.Result(void) {
        const document_id = self.documentId();
        const document = self.nodeAtMut(document_id) orelse return error.DomError;

        const old_children = document.children.items;
        for (old_children) |child_id| {
            if (self.focused_node) |focused| {
                if (sameNodeId(focused, child_id) or self.nodeIsDescendantOf(focused, child_id)) {
                    self.focused_node = null;
                }
            }

            self.clearSelectionStateForSubtree(child_id);
            self.clearCustomValidityStateForSubtree(child_id);

            if (self.nodeAtMut(child_id)) |child_record| {
                child_record.parent = null;
            }
        }

        document.children.items.len = 0;
        self.document_selection_cleared = false;
        return;
    }

    pub fn appendHtmlToDocument(self: *DomStore, html: []const u8) errors.Result(void) {
        var fragment_store = try DomStore.init(self.allocator);
        defer fragment_store.deinit();

        const fragment_root = try self.parseHtmlFragmentIntoStore(&fragment_store, self.documentId(), html);
        const fragment_children = fragment_store.childIds(fragment_root);

        const insertion_index = try self.childCount(self.documentId());
        try self.cloneFragmentChildrenInto(&fragment_store, fragment_children, self.documentId(), insertion_index, true);
        try self.syncDocumentTitleFromDom();
        return;
    }

    pub fn outerHtml(self: *const DomStore, allocator: std.mem.Allocator, node_id: NodeId) errors.Result([]u8) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        switch (node.kind) {
            .element => {},
            else => return error.DomError,
        }

        var output: std.ArrayList(u8) = .empty;
        errdefer output.deinit(allocator);

        try self.serializeHtmlNode(node_id, &output, allocator);
        const result = try allocator.dupe(u8, output.items);
        output.deinit(allocator);
        return result;
    }

    pub fn setOuterHtml(self: *DomStore, node_id: NodeId, html: []const u8) errors.Result(void) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        switch (node.kind) {
            .element => {},
            .document => return error.DomError,
            else => return error.DomError,
        }

        const parent_id = parentOf(self, node_id) orelse return;
        const insertion_index = try self.childIndex(parent_id, node_id);

        var fragment_store = try DomStore.init(self.allocator);
        defer fragment_store.deinit();

        const fragment_root = try self.parseHtmlFragmentIntoStore(&fragment_store, parent_id, html);
        const fragment_children = fragment_store.childIds(fragment_root);

        try self.removeNode(node_id);
        try self.cloneFragmentChildrenInto(&fragment_store, fragment_children, parent_id, insertion_index, true);
        try self.syncDocumentTitleFromDom();
        return;
    }

    pub fn setOuterText(self: *DomStore, node_id: NodeId, text: []const u8) errors.Result(void) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        switch (node.kind) {
            .element => {},
            else => return error.DomError,
        }

        const parent_id = parentOf(self, node_id) orelse {
            try self.setTextContent(node_id, text);
            return;
        };

        const replacement = try self.createTextNode(text);
        _ = try self.replaceChild(parent_id, replacement, node_id);
        try self.syncDocumentTitleFromDom();
    }

    pub fn insertAdjacentHtml(
        self: *DomStore,
        node_id: NodeId,
        position: []const u8,
        html: []const u8,
    ) errors.Result(void) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |*element| element,
            else => return error.DomError,
        };

        if (std.mem.eql(u8, position, "beforebegin")) {
            const parent_id = parentOf(self, node_id) orelse return error.DomError;
            const insertion_index = try self.childIndex(parent_id, node_id);

            var fragment_store = try DomStore.init(self.allocator);
            defer fragment_store.deinit();

            const fragment_root = try self.parseHtmlFragmentIntoStore(&fragment_store, parent_id, html);
            const fragment_children = fragment_store.childIds(fragment_root);
            try self.cloneFragmentChildrenInto(&fragment_store, fragment_children, parent_id, insertion_index, true);
            try self.syncDocumentTitleFromDom();
            return;
        }

        if (std.mem.eql(u8, position, "afterbegin")) {
            if (isVoidElement(element.tag_name)) {
                return error.DomError;
            }

            var fragment_store = try DomStore.init(self.allocator);
            defer fragment_store.deinit();

            const fragment_root = try self.parseHtmlFragmentIntoStore(&fragment_store, node_id, html);
            const fragment_children = fragment_store.childIds(fragment_root);
            try self.cloneFragmentChildrenInto(&fragment_store, fragment_children, node_id, 0, true);
            try self.syncDocumentTitleFromDom();
            if (std.mem.eql(u8, element.tag_name, "textarea")) {
                try self.resetSelectionToEnd(node_id);
            }
            return;
        }

        if (std.mem.eql(u8, position, "beforeend")) {
            if (isVoidElement(element.tag_name)) {
                return error.DomError;
            }

            const insertion_index = try self.childCount(node_id);

            var fragment_store = try DomStore.init(self.allocator);
            defer fragment_store.deinit();

            const fragment_root = try self.parseHtmlFragmentIntoStore(&fragment_store, node_id, html);
            const fragment_children = fragment_store.childIds(fragment_root);
            try self.cloneFragmentChildrenInto(&fragment_store, fragment_children, node_id, insertion_index, true);
            try self.syncDocumentTitleFromDom();
            if (std.mem.eql(u8, element.tag_name, "textarea")) {
                try self.resetSelectionToEnd(node_id);
            }
            return;
        }

        if (std.mem.eql(u8, position, "afterend")) {
            const parent_id = parentOf(self, node_id) orelse return error.DomError;
            const insertion_index = try self.childIndex(parent_id, node_id) + 1;

            var fragment_store = try DomStore.init(self.allocator);
            defer fragment_store.deinit();

            const fragment_root = try self.parseHtmlFragmentIntoStore(&fragment_store, parent_id, html);
            const fragment_children = fragment_store.childIds(fragment_root);
            try self.cloneFragmentChildrenInto(&fragment_store, fragment_children, parent_id, insertion_index, true);
            try self.syncDocumentTitleFromDom();
            return;
        }

        return error.DomError;
    }

    pub fn insertAdjacentElement(
        self: *DomStore,
        node_id: NodeId,
        position: []const u8,
        child: NodeId,
    ) errors.Result(NodeId) {
        const insertion = try self.adjacentInsertionPoint(node_id, position);
        try self.insertChildrenAt(insertion.parent, insertion.index, &.{child});
        return child;
    }

    pub fn insertAdjacentText(
        self: *DomStore,
        node_id: NodeId,
        position: []const u8,
        text: []const u8,
    ) errors.Result(NodeId) {
        const insertion = try self.adjacentInsertionPoint(node_id, position);
        const created = try self.addText(insertion.parent, text);
        try self.moveAppendedChildToIndex(insertion.parent, created, insertion.index);
        return created;
    }

    fn adjacentInsertionPoint(
        self: *const DomStore,
        node_id: NodeId,
        position: []const u8,
    ) errors.Result(AdjacentInsertion) {
        const node = self.nodeAt(node_id) orelse return error.DomError;

        if (std.mem.eql(u8, position, "beforebegin")) {
            const parent_id = parentOf(self, node_id) orelse return error.DomError;
            const insertion_index = try self.childIndex(parent_id, node_id);
            return .{ .parent = parent_id, .index = insertion_index };
        }

        if (std.mem.eql(u8, position, "afterend")) {
            const parent_id = parentOf(self, node_id) orelse return error.DomError;
            const insertion_index = try self.childIndex(parent_id, node_id) + 1;
            return .{ .parent = parent_id, .index = insertion_index };
        }

        const element = switch (node.kind) {
            .element => |*element| element,
            else => return error.DomError,
        };

        if (isVoidElement(element.tag_name)) {
            return error.DomError;
        }

        if (std.mem.eql(u8, position, "afterbegin")) {
            return .{ .parent = node_id, .index = 0 };
        }

        if (std.mem.eql(u8, position, "beforeend")) {
            return .{ .parent = node_id, .index = try self.childCount(node_id) };
        }

        return error.DomError;
    }

    pub fn textContent(
        self: *const DomStore,
        allocator: std.mem.Allocator,
        node_id: NodeId,
    ) errors.Result([]u8) {
        var output: std.ArrayList(u8) = .empty;
        errdefer output.deinit(allocator);

        try self.appendTextContent(node_id, &output, allocator);
        const result = try allocator.dupe(u8, output.items);
        output.deinit(allocator);
        return result;
    }

    pub fn setTextContent(self: *DomStore, node_id: NodeId, text: []const u8) errors.Result(void) {
        const node = self.nodeAtMut(node_id) orelse return error.HtmlParse;
        switch (node.kind) {
            .document, .element => {
                const old_children = node.children.items;
                for (old_children) |child_id| {
                    if (self.nodeAtMut(child_id)) |child| {
                        child.parent = null;
                    }
                }

                node.children.items.len = 0;
                if (text.len > 0) {
                    _ = try self.addText(node_id, text);
                }
                try self.syncDocumentTitleFromDom();
                if (self.isTextareaNode(node_id)) {
                    try self.setSelectionRange(node_id, text.len, text.len, .none);
                }
            },
            else => return error.HtmlParse,
        }
    }

    pub fn setNodeValue(self: *DomStore, node_id: NodeId, text: []const u8) errors.Result(void) {
        const node = self.nodeAtMut(node_id) orelse return error.HtmlParse;
        const arena_alloc = self.arena.allocator();
        switch (node.kind) {
            .text => node.kind = .{ .text = try arena_alloc.dupe(u8, text) },
            .comment => node.kind = .{ .comment = try arena_alloc.dupe(u8, text) },
            else => return error.DomError,
        }
        try self.syncDocumentTitleFromDom();
    }

    pub fn characterDataText(self: *const DomStore, node_id: NodeId) errors.Result([]const u8) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        return switch (node.kind) {
            .text => |text| text,
            .comment => |comment| comment,
            else => return error.DomError,
        };
    }

    pub fn appendCharacterData(self: *DomStore, node_id: NodeId, text: []const u8) errors.Result(void) {
        const current = try self.characterDataText(node_id);
        const merged = try self.concatText(current, text);
        try self.setNodeValue(node_id, merged);
    }

    pub fn insertCharacterData(
        self: *DomStore,
        node_id: NodeId,
        offset: usize,
        text: []const u8,
    ) errors.Result(void) {
        const current = try self.characterDataText(node_id);
        if (offset > current.len) return error.DomError;

        const merged_prefix = try self.concatText(current[0..offset], text);
        const merged = try self.concatText(merged_prefix, current[offset..]);
        try self.setNodeValue(node_id, merged);
    }

    pub fn deleteCharacterData(
        self: *DomStore,
        node_id: NodeId,
        offset: usize,
        count: usize,
    ) errors.Result(void) {
        const current = try self.characterDataText(node_id);
        if (offset > current.len) return error.DomError;

        const remaining = current.len - offset;
        const delete_count = if (count > remaining) remaining else count;
        const end = offset + delete_count;
        const merged = try self.concatText(current[0..offset], current[end..]);
        try self.setNodeValue(node_id, merged);
    }

    pub fn replaceCharacterData(
        self: *DomStore,
        node_id: NodeId,
        offset: usize,
        count: usize,
        text: []const u8,
    ) errors.Result(void) {
        const current = try self.characterDataText(node_id);
        if (offset > current.len) return error.DomError;

        const remaining = current.len - offset;
        const delete_count = if (count > remaining) remaining else count;
        const end = offset + delete_count;
        const merged_prefix = try self.concatText(current[0..offset], text);
        const merged = try self.concatText(merged_prefix, current[end..]);
        try self.setNodeValue(node_id, merged);
    }

    pub fn wholeText(self: *const DomStore, allocator: std.mem.Allocator, node_id: NodeId) errors.Result([]const u8) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        const text = switch (node.kind) {
            .text => |value| value,
            else => return error.DomError,
        };

        const parent_id = node.parent orelse return allocator.dupe(u8, text);
        const parent = self.nodeAt(parent_id) orelse return error.DomError;
        const index = try self.childIndex(parent_id, node_id);
        var start_index = index;
        while (start_index > 0) {
            const sibling_id = parent.children.items[start_index - 1];
            const sibling = self.nodeAt(sibling_id) orelse return error.DomError;
            switch (sibling.kind) {
                .text => start_index -= 1,
                else => break,
            }
        }

        var end_index = index;
        while (end_index + 1 < parent.children.items.len) {
            const sibling_id = parent.children.items[end_index + 1];
            const sibling = self.nodeAt(sibling_id) orelse return error.DomError;
            switch (sibling.kind) {
                .text => end_index += 1,
                else => break,
            }
        }

        var out: std.ArrayList(u8) = .empty;
        errdefer out.deinit(allocator);
        for (parent.children.items[start_index .. end_index + 1]) |sibling_id| {
            const sibling = self.nodeAt(sibling_id) orelse return error.DomError;
            switch (sibling.kind) {
                .text => |value| try out.appendSlice(allocator, value),
                else => return error.DomError,
            }
        }
        return try out.toOwnedSlice(allocator);
    }

    pub fn splitTextNode(self: *DomStore, node_id: NodeId, offset: usize) errors.Result(NodeId) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        const text_value = switch (node.kind) {
            .text => |text| text,
            else => return error.DomError,
        };
        if (offset > text_value.len) return error.DomError;

        const parent_id = node.parent;
        const head = text_value[0..offset];
        const tail = text_value[offset..];
        try self.setNodeValue(node_id, head);

        if (parent_id) |attached_parent| {
            const split = try self.addText(attached_parent, tail);
            try self.insertChildrenAfter(attached_parent, node_id, &.{split});
            return split;
        }

        return try self.createTextNode(tail);
    }

    pub fn normalize(self: *DomStore, node_id: NodeId) errors.Result(void) {
        const node = self.nodeAtMut(node_id) orelse return error.HtmlParse;
        switch (node.kind) {
            .document, .element => {
                const old_children = node.children.items;
                const arena_alloc = self.arena.allocator();
                var new_children: std.ArrayListUnmanaged(NodeId) = .{};
                var previous_text: ?NodeId = null;

                for (old_children) |child_id| {
                    const child = self.nodeAt(child_id) orelse return error.HtmlParse;
                    switch (child.kind) {
                        .document, .element => {
                            try self.normalize(child_id);
                            try new_children.append(arena_alloc, child_id);
                            previous_text = null;
                        },
                        .text => |text| {
                            if (text.len == 0) {
                                if (self.nodeAtMut(child_id)) |removed| {
                                    removed.parent = null;
                                }
                                continue;
                            }

                            if (previous_text) |previous_id| {
                                const previous = self.nodeAtMut(previous_id) orelse return error.HtmlParse;
                                const previous_text_value = switch (previous.kind) {
                                    .text => |value| value,
                                    else => return error.HtmlParse,
                                };
                                const merged = try self.concatText(previous_text_value, text);
                                previous.kind = .{ .text = merged };
                                if (self.nodeAtMut(child_id)) |removed| {
                                    removed.parent = null;
                                }
                                continue;
                            }

                            try new_children.append(arena_alloc, child_id);
                            previous_text = child_id;
                        },
                        .comment => {
                            try new_children.append(arena_alloc, child_id);
                            previous_text = null;
                        },
                    }
                }

                node.children = new_children;
                try self.syncDocumentTitleFromDom();
            },
            .text, .comment => {},
        }
        return;
    }

    pub fn getAttribute(
        self: *const DomStore,
        node_id: NodeId,
        name: []const u8,
    ) errors.Result(?[]const u8) {
        const trimmed = trimAttributeName(name) orelse return error.DomError;
        const node = self.nodeAt(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return error.DomError,
        };

        return elementAttributeValue(element, trimmed);
    }

    pub fn hasAttribute(
        self: *const DomStore,
        node_id: NodeId,
        name: []const u8,
    ) errors.Result(bool) {
        const trimmed = trimAttributeName(name) orelse return error.DomError;
        return (try self.getAttribute(node_id, trimmed)) != null;
    }

    pub fn popoverStateForNode(self: *const DomStore, node_id: NodeId) ?PopoverState {
        const node = self.nodeAt(node_id) orelse return null;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return null,
        };
        if (!std.mem.eql(u8, element.namespace_uri, "http://www.w3.org/1999/xhtml")) return null;
        return popoverStateFromAttribute(elementAttributeValue(element, "popover"));
    }

    pub fn popoverVisibleForNode(self: *const DomStore, node_id: NodeId) ?bool {
        const node = self.nodeAt(node_id) orelse return null;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return null,
        };
        if (!std.mem.eql(u8, element.namespace_uri, "http://www.w3.org/1999/xhtml")) return null;
        return element.popover_visible;
    }

    pub fn setPopoverVisible(self: *DomStore, node_id: NodeId, visible: bool) errors.Result(void) {
        const node = self.nodeAtMut(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |*element| element,
            else => return error.DomError,
        };
        if (!std.mem.eql(u8, element.namespace_uri, "http://www.w3.org/1999/xhtml")) {
            return error.DomError;
        }
        element.popover_visible = visible;
        return;
    }

    pub fn setAttribute(
        self: *DomStore,
        node_id: NodeId,
        name: []const u8,
        value: []const u8,
    ) errors.Result(void) {
        const trimmed = trimAttributeName(name) orelse return error.DomError;
        const node = self.nodeAtMut(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |*element| element,
            else => return error.DomError,
        };

        const arena_alloc = self.arena.allocator();
        const value_copy = try duplicateString(self, value);
        if (findAttributeIndexByName(element.attributes.items, trimmed)) |index| {
            if (std.ascii.eqlIgnoreCase(trimmed, "popover")) {
                const old_state = popoverStateFromAttribute(element.attributes.items[index].value);
                const new_state = popoverStateFromAttribute(value);
                if (element.popover_visible and old_state != new_state) {
                    element.popover_visible = false;
                }
            }
            element.attributes.items[index].value = value_copy;
            return;
        }

        if (std.ascii.eqlIgnoreCase(trimmed, "popover")) {
            const new_state = popoverStateFromAttribute(value);
            if (element.popover_visible and new_state != .no_popover) {
                element.popover_visible = false;
            }
        }

        try element.attributes.append(arena_alloc, .{
            .name = try duplicateLowercase(self, trimmed),
            .value = value_copy,
        });
        return;
    }

    pub fn removeAttribute(
        self: *DomStore,
        node_id: NodeId,
        name: []const u8,
    ) errors.Result(void) {
        const trimmed = trimAttributeName(name) orelse return error.DomError;
        const node = self.nodeAtMut(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |*element| element,
            else => return error.DomError,
        };

        if (findAttributeIndexByName(element.attributes.items, trimmed)) |index| {
            if (std.ascii.eqlIgnoreCase(trimmed, "popover")) {
                const old_state = popoverStateFromAttribute(element.attributes.items[index].value);
                if (element.popover_visible and old_state != .no_popover) {
                    element.popover_visible = false;
                }
            }
            _ = element.attributes.orderedRemove(index);
        }
        return;
    }

    pub fn toggleAttribute(
        self: *DomStore,
        node_id: NodeId,
        name: []const u8,
        force: ?bool,
    ) errors.Result(bool) {
        const trimmed = trimAttributeName(name) orelse return error.DomError;
        const node = self.nodeAtMut(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |*element| element,
            else => return error.DomError,
        };
        const arena_alloc = self.arena.allocator();
        const index = findAttributeIndexByName(element.attributes.items, trimmed);

        if (force) |forced| {
            if (!forced) {
                if (index) |found| {
                    if (std.ascii.eqlIgnoreCase(trimmed, "popover")) {
                        const old_state = popoverStateFromAttribute(element.attributes.items[found].value);
                        if (element.popover_visible and old_state != .no_popover) {
                            element.popover_visible = false;
                        }
                    }
                    _ = element.attributes.orderedRemove(found);
                }
                return false;
            }

            if (index == null) {
                if (std.ascii.eqlIgnoreCase(trimmed, "popover")) {
                    if (element.popover_visible) {
                        element.popover_visible = false;
                    }
                }
                try element.attributes.append(arena_alloc, .{
                    .name = try duplicateLowercase(self, trimmed),
                    .value = try duplicateString(self, ""),
                });
            }
            return true;
        }

        if (index) |found| {
            if (std.ascii.eqlIgnoreCase(trimmed, "popover")) {
                const old_state = popoverStateFromAttribute(element.attributes.items[found].value);
                if (element.popover_visible and old_state != .no_popover) {
                    element.popover_visible = false;
                }
            }
            _ = element.attributes.orderedRemove(found);
            return false;
        }

        if (std.ascii.eqlIgnoreCase(trimmed, "popover")) {
            if (element.popover_visible) {
                element.popover_visible = false;
            }
        }

        try element.attributes.append(arena_alloc, .{
            .name = try duplicateLowercase(self, trimmed),
            .value = try duplicateString(self, ""),
        });
        return true;
    }

    pub fn valueForNode(
        self: *const DomStore,
        allocator: std.mem.Allocator,
        node_id: NodeId,
    ) errors.Result([]u8) {
        const node = self.nodeAt(node_id) orelse return error.HtmlParse;
        return switch (node.kind) {
            .document => self.textContent(allocator, node_id),
            .element => |element| blk: {
                if (std.mem.eql(u8, element.tag_name, "select")) {
                    break :blk self.selectValueForNode(allocator, node_id);
                }
                if (std.mem.eql(u8, element.tag_name, "option")) {
                    break :blk self.optionValueForNode(allocator, node_id);
                }
                if (std.mem.eql(u8, element.tag_name, "button")) {
                    const text = (try self.getAttribute(node_id, "value")) orelse "";
                    break :blk allocator.dupe(u8, text);
                }
                if (std.mem.eql(u8, element.tag_name, "textarea")) {
                    break :blk self.textContent(allocator, node_id);
                }
                if (std.mem.eql(u8, element.tag_name, "input")) {
                    break :blk self.inputValueForNode(allocator, node_id);
                }
                break :blk self.textContent(allocator, node_id);
            },
            .text => |text| allocator.dupe(u8, text),
            .comment => allocator.dupe(u8, ""),
        };
    }

    pub fn checkedForNode(self: *const DomStore, node_id: NodeId) ?bool {
        const node = self.nodeAt(node_id) orelse return null;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return null,
        };

        if (std.mem.eql(u8, element.tag_name, "input") and isCheckableInputType(elementAttributeValue(element, "type"))) {
            return elementAttributeValue(element, "checked") != null;
        }

        return null;
    }

    pub fn setFormControlValue(self: *DomStore, node_id: NodeId, value: []const u8) errors.Result(void) {
        const node = self.nodeAtMut(node_id) orelse return error.HtmlParse;
        const element = switch (node.kind) {
            .element => |*element| element,
            else => return error.HtmlParse,
        };

        if (std.mem.eql(u8, element.tag_name, "textarea")) {
            try self.setTextContent(node_id, value);
            return;
        }

        if (std.mem.eql(u8, element.tag_name, "input") and std.mem.eql(u8, elementAttributeValue(element.*, "type") orelse "text", "range")) {
            const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
            const candidate = if (trimmed.len == 0) null else std.fmt.parseFloat(f64, trimmed) catch null;
            const normalized = normalizedRangeValue(self, node_id, candidate) orelse return error.DomError;
            const text = try std.fmt.allocPrint(self.allocator, "{d}", .{normalized});
            defer self.allocator.free(text);
            const arena_alloc = self.arena.allocator();
            try upsertAttribute(arena_alloc, &element.attributes, "value", try duplicateString(self, text));
            return;
        }

        if (std.mem.eql(u8, element.tag_name, "input") and isTextInputType(elementAttributeValue(element.*, "type"))) {
            const arena_alloc = self.arena.allocator();
            try upsertAttribute(arena_alloc, &element.attributes, "value", try duplicateString(self, value));
            if (isSelectionInputType(elementAttributeValue(element.*, "type"))) {
                try self.setSelectionRange(node_id, value.len, value.len, .none);
            }
            return;
        }

        if (std.mem.eql(u8, element.tag_name, "option")) {
            const arena_alloc = self.arena.allocator();
            try upsertAttribute(arena_alloc, &element.attributes, "value", try duplicateString(self, value));
            return;
        }

        if (std.mem.eql(u8, element.tag_name, "input") and isCheckableInputType(elementAttributeValue(element.*, "type"))) {
            return error.DomError;
        }

        if (std.mem.eql(u8, element.tag_name, "select")) {
            try self.setSelectValue(node_id, value);
            return;
        }

        return error.DomError;
    }

    pub fn setFormControlChecked(self: *DomStore, node_id: NodeId, checked: bool) errors.Result(void) {
        const node = self.nodeAtMut(node_id) orelse return error.HtmlParse;
        const element = switch (node.kind) {
            .element => |*element| element,
            else => return error.HtmlParse,
        };

        if (!std.mem.eql(u8, element.tag_name, "input") or !isCheckableInputType(elementAttributeValue(element.*, "type"))) {
            return error.DomError;
        }

        if (checked) {
            const arena_alloc = self.arena.allocator();
            try upsertAttribute(arena_alloc, &element.attributes, "checked", try duplicateString(self, ""));
        } else {
            removeAttributeByName(&element.attributes, "checked");
        }

        return;
    }

    pub fn indeterminateForNode(self: *const DomStore, node_id: NodeId) ?bool {
        const node = self.nodeAt(node_id) orelse return null;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return null,
        };

        if (!std.mem.eql(u8, element.tag_name, "input")) return null;
        return self.indeterminate.contains(node_id);
    }

    pub fn setFormControlIndeterminate(self: *DomStore, node_id: NodeId, indeterminate: bool) errors.Result(void) {
        const node = self.nodeAtMut(node_id) orelse return error.HtmlParse;
        const element = switch (node.kind) {
            .element => |*element| element,
            else => return error.HtmlParse,
        };

        if (!std.mem.eql(u8, element.tag_name, "input")) {
            return error.DomError;
        }

        if (indeterminate) {
            try self.indeterminate.put(self.allocator, node_id, {});
        } else {
            _ = self.indeterminate.remove(node_id);
        }

        return;
    }

    pub fn setSelectValue(self: *DomStore, node_id: NodeId, value: []const u8) errors.Result(void) {
        const node = self.nodeAt(node_id) orelse return error.HtmlParse;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return error.HtmlParse,
        };

        if (!std.mem.eql(u8, element.tag_name, "select")) {
            return error.DomError;
        }

        var descendants: std.ArrayList(NodeId) = .empty;
        defer descendants.deinit(self.allocator);
        try self.collectSubtreeNodes(node_id, &descendants, self.allocator);

        var found = false;
        for (descendants.items) |descendant_id| {
            if (!self.isOptionNode(descendant_id)) continue;
            const option_value = try self.optionValueForNode(self.allocator, descendant_id);
            defer self.allocator.free(option_value);
            if (std.mem.eql(u8, option_value, value)) {
                found = true;
                break;
            }
        }

        if (!found) return error.HtmlParse;

        for (descendants.items) |descendant_id| {
            if (!self.isOptionNode(descendant_id)) continue;
            const option_value = try self.optionValueForNode(self.allocator, descendant_id);
            defer self.allocator.free(option_value);
            const selected = std.mem.eql(u8, option_value, value);
            try self.setOptionSelected(descendant_id, selected);
        }

        return;
    }

    pub fn setFileInputFiles(
        self: *DomStore,
        node_id: NodeId,
        files: []const []const u8,
    ) errors.Result(void) {
        const node = self.nodeAtMut(node_id) orelse return error.HtmlParse;
        const element = switch (node.kind) {
            .element => |*element| element,
            else => return error.HtmlParse,
        };

        if (!std.mem.eql(u8, element.tag_name, "input") or !isFileInputType(elementAttributeValue(element.*, "type"))) {
            return error.DomError;
        }

        var joined: std.ArrayList(u8) = .empty;
        defer joined.deinit(self.allocator);

        for (files, 0..) |file, index| {
            if (index > 0) {
                try joined.appendSlice(self.allocator, ", ");
            }
            try joined.appendSlice(self.allocator, file);
        }

        const arena_alloc = self.arena.allocator();
        try upsertAttribute(
            arena_alloc,
            &element.attributes,
            "value",
            try duplicateString(self, joined.items),
        );
        return;
    }

    pub fn isSelectionControlNode(self: *const DomStore, node_id: NodeId) bool {
        const node = self.nodeAt(node_id) orelse return false;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return false,
        };

        if (std.mem.eql(u8, element.tag_name, "textarea")) {
            return true;
        }

        if (!std.mem.eql(u8, element.tag_name, "input")) {
            return false;
        }

        return isSelectionInputType(elementAttributeValue(element, "type"));
    }

    pub fn selectionStateForNode(self: *const DomStore, node_id: NodeId) errors.Result(?SelectionState) {
        const length = try self.selectionControlValueLength(node_id) orelse return null;
        const stored = self.selection.get(node_id) orelse return .{
            .start = length,
            .end = length,
            .direction = .none,
        };
        return normalizeSelectionState(stored, length);
    }

    pub fn documentSelectionCleared(self: *const DomStore) bool {
        return self.document_selection_cleared;
    }

    pub fn setDocumentSelectionCleared(self: *DomStore, cleared: bool) void {
        self.document_selection_cleared = cleared;
    }

    pub fn setSelectionRange(
        self: *DomStore,
        node_id: NodeId,
        start: usize,
        end: usize,
        direction: SelectionDirection,
    ) errors.Result(void) {
        const length = try self.selectionControlValueLength(node_id) orelse return error.DomError;
        const normalized = normalizeSelectionRange(start, end, length, direction);
        try self.selection.put(self.allocator, node_id, normalized);
        self.document_selection_cleared = false;
        return;
    }

    pub fn setRangeText(
        self: *DomStore,
        node_id: NodeId,
        replacement: []const u8,
        start: ?usize,
        end: ?usize,
        selection_mode: RangeTextSelectionMode,
    ) errors.Result(void) {
        const current_value = try self.valueForNode(self.allocator, node_id);
        defer self.allocator.free(current_value);

        const selection: SelectionState = if (try self.selectionStateForNode(node_id)) |state| state else SelectionState{
            .start = current_value.len,
            .end = current_value.len,
            .direction = .none,
        };
        const selection_start = selection.start;
        const selection_end = selection.end;
        const range_start = start orelse selection_start;
        const range_end = end orelse selection_end;

        if (range_start > range_end or range_end > current_value.len) {
            return error.DomError;
        }

        const prefix = try self.concatText(current_value[0..range_start], replacement);
        const updated = try self.concatText(prefix, current_value[range_end..]);

        try self.setFormControlValue(node_id, updated);

        const delta: isize = @as(isize, @intCast(replacement.len)) - @as(isize, @intCast(range_end - range_start));
        const new_value_length = updated.len;

        const next_state: SelectionState = switch (selection_mode) {
            .select => .{
                .start = range_start,
                .end = range_start + replacement.len,
                .direction = .none,
            },
            .start => .{
                .start = range_start,
                .end = range_start,
                .direction = .none,
            },
            .end => .{
                .start = range_start + replacement.len,
                .end = range_start + replacement.len,
                .direction = .none,
            },
            .preserve => blk: {
                var next_start = selection_start;
                var next_end = selection_end;

                if (selection_end <= range_start) {
                    // unchanged
                } else if (selection_start >= range_end) {
                    next_start = shiftSelectionIndex(selection_start, delta);
                    next_end = shiftSelectionIndex(selection_end, delta);
                } else {
                    if (selection_start < range_start) {
                        next_start = selection_start;
                    } else {
                        next_start = range_start;
                    }

                    if (selection_end <= range_end) {
                        next_end = range_start + replacement.len;
                    } else {
                        next_end = shiftSelectionIndex(selection_end, delta);
                    }
                }

                break :blk normalizeSelectionRange(next_start, next_end, new_value_length, selection.direction);
            },
        };

        try self.setSelectionRange(node_id, next_state.start, next_state.end, next_state.direction);
        return;
    }

    pub fn stepInputValue(self: *DomStore, node_id: NodeId, delta_steps: i64) errors.Result(void) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return error.DomError,
        };

        const input_type = elementAttributeValue(element, "type");
        const kind = inputValueAsNumberKind(input_type) orelse return error.DomError;

        switch (kind) {
            .number, .range => {
                const limits = numericRangeLimits(self, node_id);
                const raw_value = elementAttributeValue(element, "value") orelse "";
                const trimmed_value = std.mem.trim(u8, raw_value, " \t\r\n\x0c");
                const current_value = if (trimmed_value.len > 0) blk: {
                    break :blk std.fmt.parseFloat(f64, trimmed_value) catch return error.DomError;
                } else blk: {
                    if (std.mem.eql(u8, input_type orelse "text", "range")) {
                        const min = limits.?.min orelse 0.0;
                        const max = limits.?.max orelse 100.0;
                        break :blk (min + max) / 2.0;
                    }
                    break :blk if (limits) |bounds| bounds.min orelse 0.0 else 0.0;
                };

                const step_value = if (elementAttributeValue(element, "step")) |value| blk: {
                    const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
                    if (trimmed.len == 0) break :blk 1.0;
                    if (std.ascii.eqlIgnoreCase(trimmed, "any")) return error.DomError;
                    const parsed = std.fmt.parseFloat(f64, trimmed) catch return error.DomError;
                    if (!std.math.isFinite(parsed) or parsed <= 0) return error.DomError;
                    break :blk parsed;
                } else 1.0;

                var next_value = current_value + (step_value * @as(f64, @floatFromInt(delta_steps)));
                if (limits) |bounds| {
                    if (bounds.min) |min| {
                        if (next_value < min) next_value = min;
                    }
                    if (bounds.max) |max| {
                        if (next_value > max) next_value = max;
                    }
                }

                const text = try std.fmt.allocPrint(self.allocator, "{d}", .{next_value});
                defer self.allocator.free(text);
                try self.setFormControlValue(node_id, text);
                return;
            },
            .date, .datetime_local, .week, .time => {
                const current_value = try inputStepCurrentValue(self, node_id, kind);
                const step_value = try inputStepScaleValue(self, node_id, kind);
                const next_value = current_value + (step_value * @as(f64, @floatFromInt(delta_steps)));
                const text = try inputValueTextFromNumber(self, node_id, kind, next_value);
                defer self.allocator.free(text);
                try self.setFormControlValue(node_id, text);
                return;
            },
            .month => {
                const current_value = try inputStepCurrentValue(self, node_id, kind);
                const step_value = try inputStepScaleValue(self, node_id, kind);
                const calendar = calendarFromMilliseconds(current_value);
                const current_month_index = (@as(i64, calendar.year) * 12) + @as(i64, calendar.month) - 1;
                const stepped_months = @as(f64, @floatFromInt(current_month_index)) +
                    (step_value * @as(f64, @floatFromInt(delta_steps)));
                const next_month_index = @as(i64, @intFromFloat(std.math.round(stepped_months)));
                const year = @divFloor(next_month_index, 12);
                const month = @as(u8, @intCast(@mod(next_month_index, 12) + 1));
                const days = daysFromCivil(year, month, 1);
                const next_value = @as(f64, @floatFromInt(days)) * @as(f64, @floatFromInt(millisPerDay()));
                const text = try inputValueTextFromNumber(self, node_id, kind, next_value);
                defer self.allocator.free(text);
                try self.setFormControlValue(node_id, text);
                return;
            },
        }
    }

    pub fn appendChild(self: *DomStore, parent: NodeId, child: NodeId) errors.Result(NodeId) {
        try self.insertChildrenAt(parent, try self.childCount(parent), &.{child});
        return child;
    }

    pub fn insertBefore(
        self: *DomStore,
        parent: NodeId,
        child: NodeId,
        reference: ?NodeId,
    ) errors.Result(NodeId) {
        if (reference) |reference_id| {
            if (sameNodeId(child, reference_id)) return error.DomError;
            try self.insertChildrenBefore(parent, reference_id, &.{child});
            return child;
        }

        return try self.appendChild(parent, child);
    }

    pub fn replaceChild(
        self: *DomStore,
        parent: NodeId,
        new_child: NodeId,
        old_child: NodeId,
    ) errors.Result(NodeId) {
        if (sameNodeId(new_child, old_child)) return old_child;
        const old_parent = parentOf(self, old_child) orelse return error.DomError;
        if (!sameNodeId(old_parent, parent)) return error.DomError;

        try self.insertChildrenBefore(parent, old_child, &.{new_child});
        try self.removeNode(old_child);
        return old_child;
    }

    pub fn replaceChildren(
        self: *DomStore,
        parent: NodeId,
        children: []const NodeId,
    ) errors.Result(void) {
        try self.validateMutationChildren(parent, children);

        const parent_record = self.nodeAtMut(parent) orelse return error.DomError;
        const old_children = try self.allocator.dupe(NodeId, parent_record.children.items);
        defer self.allocator.free(old_children);

        try self.insertChildrenAt(parent, 0, children);

        var index = old_children.len;
        while (index > 0) {
            index -= 1;
            const old_child = old_children[index];
            if (!sliceContainsNodeId(children, old_child)) {
                try self.removeNode(old_child);
            }
        }
        if (self.isTextareaNode(parent)) {
            try self.resetSelectionToEnd(parent);
        }
        return;
    }

    pub fn removeChild(self: *DomStore, parent: NodeId, child: NodeId) errors.Result(NodeId) {
        const child_parent = parentOf(self, child) orelse return error.DomError;
        if (!sameNodeId(child_parent, parent)) return error.DomError;
        try self.removeNode(child);
        return child;
    }

    pub fn appendChildren(
        self: *DomStore,
        parent: NodeId,
        children: []const NodeId,
    ) errors.Result(void) {
        try self.insertChildrenAt(parent, try self.childCount(parent), children);
        return;
    }

    pub fn prependChildren(
        self: *DomStore,
        parent: NodeId,
        children: []const NodeId,
    ) errors.Result(void) {
        try self.insertChildrenAt(parent, 0, children);
        return;
    }

    pub fn insertChildrenBefore(
        self: *DomStore,
        parent: NodeId,
        reference: NodeId,
        children: []const NodeId,
    ) errors.Result(void) {
        const reference_parent = parentOf(self, reference) orelse return;
        if (!sameNodeId(reference_parent, parent)) return error.DomError;
        try self.insertChildrenAt(parent, try self.childIndex(parent, reference), children);
        return;
    }

    pub fn insertChildrenAfter(
        self: *DomStore,
        parent: NodeId,
        reference: NodeId,
        children: []const NodeId,
    ) errors.Result(void) {
        const reference_parent = parentOf(self, reference) orelse return;
        if (!sameNodeId(reference_parent, parent)) return error.DomError;
        const index = try self.childIndex(parent, reference);
        try self.insertChildrenAt(parent, index + 1, children);
        return;
    }

    pub fn removeNode(self: *DomStore, node_id: NodeId) errors.Result(void) {
        if (sameNodeId(node_id, self.documentId())) {
            return error.DomError;
        }

        const parent_id = parentOf(self, node_id) orelse return;
        const parent_index = try self.childIndex(parent_id, node_id);
        const parent_record = self.nodeAtMut(parent_id) orelse return error.DomError;
        if (parent_index >= parent_record.children.items.len or !sameNodeId(parent_record.children.items[parent_index], node_id)) {
            return error.DomError;
        }
        _ = parent_record.children.orderedRemove(parent_index);

        if (self.focused_node) |focused| {
            if (sameNodeId(focused, node_id) or self.nodeIsDescendantOf(focused, node_id)) {
                self.focused_node = null;
            }
        }

        self.clearSelectionStateForSubtree(node_id);
        self.clearIndeterminateStateForSubtree(node_id);
        self.clearCustomValidityStateForSubtree(node_id);
        self.clearTextTrackStateForSubtree(node_id);

        const record = self.nodeAtMut(node_id) orelse return error.DomError;
        record.parent = null;
        try self.syncDocumentTitleFromDom();
        return;
    }

    fn dumpNode(
        self: *const DomStore,
        node_id: NodeId,
        indent: usize,
        output: *std.ArrayList(u8),
        allocator: std.mem.Allocator,
    ) errors.Result(void) {
        const node = self.nodeAt(node_id) orelse return error.HtmlParse;
        try writeIndent(output, allocator, indent);

        switch (node.kind) {
            .document => {
                try output.appendSlice(allocator, "#document\n");
                for (node.children.items) |child| {
                    try self.dumpNode(child, indent + 1, output, allocator);
                }
            },
            .text => |text| {
                try output.append(allocator, '"');
                try writeEscapedText(output, allocator, text);
                try output.appendSlice(allocator, "\"\n");
            },
            .comment => |comment| {
                try output.appendSlice(allocator, "<!-- ");
                try output.appendSlice(allocator, comment);
                try output.appendSlice(allocator, " -->\n");
            },
            .element => |element| {
                try output.appendSlice(allocator, "<");
                try output.appendSlice(allocator, element.tag_name);
                if (element.attributes.items.len > 0) {
                    for (element.attributes.items) |attribute| {
                        try output.append(allocator, ' ');
                        try output.appendSlice(allocator, attribute.name);
                        if (attribute.value.len > 0) {
                            try output.appendSlice(allocator, "=\"");
                            try writeEscapedAttr(output, allocator, attribute.value);
                            try output.appendSlice(allocator, "\"");
                        }
                    }
                }

                if (isVoidElement(element.tag_name)) {
                    try output.appendSlice(allocator, " />\n");
                    return;
                }

                try output.appendSlice(allocator, ">\n");
                for (node.children.items) |child| {
                    try self.dumpNode(child, indent + 1, output, allocator);
                }
                try writeIndent(output, allocator, indent);
                try output.appendSlice(allocator, "</");
                try output.appendSlice(allocator, element.tag_name);
                try output.appendSlice(allocator, ">\n");
            },
        }
    }

    fn appendTextContent(
        self: *const DomStore,
        node_id: NodeId,
        output: *std.ArrayList(u8),
        allocator: std.mem.Allocator,
    ) errors.Result(void) {
        const node = self.nodeAt(node_id) orelse return error.HtmlParse;
        switch (node.kind) {
            .document, .element => {
                for (node.children.items) |child_id| {
                    try self.appendTextContent(child_id, output, allocator);
                }
            },
            .text => |text| {
                try output.appendSlice(allocator, text);
            },
            .comment => {},
        }
    }

    fn concatText(self: *DomStore, left: []const u8, right: []const u8) errors.Result([]const u8) {
        const arena_alloc = self.arena.allocator();
        const merged = try arena_alloc.alloc(u8, left.len + right.len);
        @memcpy(merged[0..left.len], left);
        @memcpy(merged[left.len..], right);
        return merged;
    }

    fn parseHtmlFragmentIntoStore(
        self: *const DomStore,
        fragment_store: *DomStore,
        context_parent: NodeId,
        html: []const u8,
    ) errors.Result(NodeId) {
        const context_tag = try self.fragmentContextTagName(context_parent);
        const context_tag_copy = try duplicateString(fragment_store, context_tag);
        const fragment_root = try fragment_store.addElement(fragment_store.documentId(), context_tag_copy, .{}, null);

        var parser = HtmlParser.init(html);
        try parser.parseFragmentInto(fragment_store, fragment_root);
        return fragment_root;
    }

    fn fragmentContextTagName(self: *const DomStore, context_parent: NodeId) errors.Result([]const u8) {
        const node = self.nodeAt(context_parent) orelse return error.DomError;
        return switch (node.kind) {
            .document => "div",
            .element => |element| element.tag_name,
            else => return error.DomError,
        };
    }

    fn cloneFragmentChildrenInto(
        self: *DomStore,
        fragment_store: *const DomStore,
        fragment_children: []const NodeId,
        parent: NodeId,
        insertion_index: usize,
        deep: bool,
    ) errors.Result(void) {
        var next_index = insertion_index;
        for (fragment_children) |child_id| {
            _ = try self.cloneSubtreeAt(fragment_store, child_id, parent, next_index, deep);
            next_index += 1;
        }
    }

    fn cloneSubtreeAt(
        self: *DomStore,
        source: *const DomStore,
        source_node_id: NodeId,
        parent: NodeId,
        insertion_index: usize,
        deep: bool,
    ) errors.Result(NodeId) {
        const source_node = source.nodeAt(source_node_id) orelse return error.DomError;
        const created = switch (source_node.kind) {
            .document => return error.DomError,
            .text => |text| try self.addText(parent, text),
            .comment => |comment| try self.addComment(parent, comment),
            .element => |element| blk: {
                var attributes: std.ArrayListUnmanaged(Attribute) = .{};
                for (element.attributes.items) |attribute| {
                    try attributes.append(self.arena.allocator(), .{
                        .name = try duplicateString(self, attribute.name),
                        .value = try duplicateString(self, attribute.value),
                    });
                }
                const node_id = try self.addElement(
                    parent,
                    try duplicateString(self, element.tag_name),
                    attributes,
                    element.namespace_uri,
                );
                const cloned_element = self.nodeAtMut(node_id) orelse return error.DomError;
                if (cloned_element.kind == .element) {
                    cloned_element.kind.element.dialog_return_value =
                        try duplicateString(self, element.dialog_return_value);
                    cloned_element.kind.element.output_default_value = if (element.output_default_value) |default_value|
                        try duplicateString(self, default_value)
                    else
                        null;
                    cloned_element.kind.element.popover_visible = element.popover_visible;
                }
                break :blk node_id;
            },
        };

        try self.moveAppendedChildToIndex(parent, created, insertion_index);

        if (deep and source_node.kind == .element) {
            const source_children = source_node.children.items;
            var child_index: usize = 0;
            while (child_index < source_children.len) : (child_index += 1) {
                _ = try self.cloneSubtreeAt(source, source_children[child_index], created, child_index, true);
            }
        }

        return created;
    }

    pub fn cloneNode(self: *DomStore, source_node_id: NodeId, deep: bool) errors.Result(NodeId) {
        const source_node = self.nodeAt(source_node_id) orelse return error.DomError;
        if (source_node.kind == .document) {
            return error.DomError;
        }

        var fragment_store = try DomStore.init(self.allocator);
        defer fragment_store.deinit();

        const fragment_root = try fragment_store.cloneSubtreeAt(self, source_node_id, fragment_store.documentId(), 0, deep);
        const insertion_index = try self.childCount(self.documentId());
        const cloned = try self.cloneSubtreeAt(&fragment_store, fragment_root, self.documentId(), insertion_index, deep);
        try self.removeNode(cloned);
        return cloned;
    }

    fn moveAppendedChildToIndex(
        self: *DomStore,
        parent: NodeId,
        child: NodeId,
        insertion_index: usize,
    ) errors.Result(void) {
        const arena_alloc = self.arena.allocator();
        const parent_record = self.nodeAtMut(parent) orelse return error.DomError;
        if (parent_record.children.items.len == 0) return error.DomError;

        const last_index = parent_record.children.items.len - 1;
        if (!sameNodeId(parent_record.children.items[last_index], child)) {
            return error.DomError;
        }

        const moved = parent_record.children.orderedRemove(last_index);
        const target_index = if (insertion_index > parent_record.children.items.len) parent_record.children.items.len else insertion_index;
        try parent_record.children.insert(arena_alloc, target_index, moved);
    }

    fn serializeHtmlNode(
        self: *const DomStore,
        node_id: NodeId,
        output: *std.ArrayList(u8),
        allocator: std.mem.Allocator,
    ) errors.Result(void) {
        try self.serializeHtmlNodeWithContext(node_id, output, allocator, false);
    }

    fn serializeHtmlNodeWithContext(
        self: *const DomStore,
        node_id: NodeId,
        output: *std.ArrayList(u8),
        allocator: std.mem.Allocator,
        raw_text_context: bool,
    ) errors.Result(void) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        switch (node.kind) {
            .document => {
                for (node.children.items) |child_id| {
                    try self.serializeHtmlNodeWithContext(child_id, output, allocator, raw_text_context);
                }
            },
            .text => |text| {
                if (raw_text_context) {
                    try output.appendSlice(allocator, text);
                } else {
                    try appendEscapedHtmlText(output, allocator, text);
                }
            },
            .comment => |comment| {
                try output.appendSlice(allocator, "<!--");
                try output.appendSlice(allocator, comment);
                try output.appendSlice(allocator, "-->");
            },
            .element => |element| {
                const namespace = self.serializationNamespaceForNode(node_id);
                const tag_name = serializedElementName(namespace, element.tag_name);

                try output.appendSlice(allocator, "<");
                try output.appendSlice(allocator, tag_name);
                try self.serializeHtmlAttributes(namespace, &element, output, allocator);

                if (isVoidElement(element.tag_name)) {
                    if (node.children.items.len > 0) {
                        return error.DomError;
                    }
                    try output.appendSlice(allocator, ">");
                    return;
                }

                try output.appendSlice(allocator, ">");
                const child_raw_text_context = isRawTextElement(element.tag_name);
                for (node.children.items) |child_id| {
                    try self.serializeHtmlNodeWithContext(child_id, output, allocator, child_raw_text_context);
                }
                try output.appendSlice(allocator, "</");
                try output.appendSlice(allocator, tag_name);
                try output.appendSlice(allocator, ">");
            },
        }
    }

    fn serializeHtmlAttributes(
        self: *const DomStore,
        namespace: SerializationNamespace,
        element: *const ElementData,
        output: *std.ArrayList(u8),
        allocator: std.mem.Allocator,
    ) errors.Result(void) {
        _ = self;
        if (element.attributes.items.len == 0) {
            return;
        }

        var indices: std.ArrayList(usize) = .empty;
        defer indices.deinit(allocator);
        for (0..element.attributes.items.len) |index| {
            try indices.append(allocator, index);
        }

        var index: usize = 0;
        while (index < indices.items.len) : (index += 1) {
            var smallest = index;
            var scan = index + 1;
            while (scan < indices.items.len) : (scan += 1) {
                if (lessThanBytes(
                    element.attributes.items[indices.items[scan]].name,
                    element.attributes.items[indices.items[smallest]].name,
                )) {
                    smallest = scan;
                }
            }
            if (smallest != index) {
                const temp = indices.items[index];
                indices.items[index] = indices.items[smallest];
                indices.items[smallest] = temp;
            }
        }

        for (indices.items) |attr_index| {
            const attribute = element.attributes.items[attr_index];
            try output.appendSlice(allocator, " ");
            try output.appendSlice(allocator, serializedAttributeName(namespace, element, attribute.name));
            if (attribute.value.len == 0) {
                continue;
            }

            const has_double_quote = std.mem.indexOfScalar(u8, attribute.value, '"') != null;
            const has_single_quote = std.mem.indexOfScalar(u8, attribute.value, '\'') != null;
            const quote: u8 = if (!has_double_quote) '"' else if (!has_single_quote) '\'' else '"';
            try output.append(allocator, '=');
            try output.append(allocator, quote);
            try writeEscapedAttrValue(output, allocator, attribute.value, quote);
            try output.append(allocator, quote);
        }
    }

    fn inputValueForNode(self: *const DomStore, allocator: std.mem.Allocator, node_id: NodeId) errors.Result([]u8) {
        const node = self.nodeAt(node_id) orelse return allocator.dupe(u8, "");
        const element = switch (node.kind) {
            .element => |element| element,
            else => return allocator.dupe(u8, ""),
        };
        const input_type = elementAttributeValue(element, "type");
        if (std.mem.eql(u8, element.tag_name, "input") and std.mem.eql(u8, input_type orelse "text", "range")) {
            const raw_value = elementAttributeValue(element, "value") orelse "";
            const trimmed = std.mem.trim(u8, raw_value, " \t\r\n\x0c");
            const candidate = if (trimmed.len == 0) null else std.fmt.parseFloat(f64, trimmed) catch null;
            const normalized = normalizedRangeValue(self, node_id, candidate) orelse return allocator.dupe(u8, "");
            return std.fmt.allocPrint(allocator, "{d}", .{normalized});
        }
        return if (elementAttributeValue(element, "value")) |value|
            allocator.dupe(u8, value)
        else
            allocator.dupe(u8, "");
    }

    pub fn inputValueAsNumber(self: *const DomStore, node_id: NodeId) ?f64 {
        const node = self.nodeAt(node_id) orelse return null;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return null,
        };
        if (!std.mem.eql(u8, element.tag_name, "input")) return null;

        const input_type = elementAttributeValue(element, "type");
        const kind = inputValueAsNumberKind(input_type) orelse return null;
        const text = elementAttributeValue(element, "value") orelse "";
        return inputValueAsNumberFromText(self, node_id, kind, text);
    }

    pub fn inputValueAsDate(self: *const DomStore, node_id: NodeId) ?f64 {
        const node = self.nodeAt(node_id) orelse return null;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return null,
        };
        if (!std.mem.eql(u8, element.tag_name, "input")) return null;

        const input_type = elementAttributeValue(element, "type");
        const kind = inputValueAsDateKind(input_type) orelse return null;
        const text = elementAttributeValue(element, "value") orelse "";
        return inputValueAsDateFromText(kind, text);
    }

    pub fn setInputValueAsNumber(self: *DomStore, node_id: NodeId, value: f64) errors.Result(void) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return error.DomError,
        };
        if (!std.mem.eql(u8, element.tag_name, "input")) return error.DomError;

        const input_type = elementAttributeValue(element, "type");
        const kind = inputValueAsNumberKind(input_type) orelse return error.DomError;
        const text = try inputValueTextFromNumber(self, node_id, kind, value);
        defer self.allocator.free(text);
        try self.setAttribute(node_id, "value", text);
    }

    pub fn setInputValueAsDate(self: *DomStore, node_id: NodeId, value: ?f64) errors.Result(void) {
        const node = self.nodeAtMut(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |*element| element,
            else => return error.DomError,
        };
        if (!std.mem.eql(u8, element.tag_name, "input")) return error.DomError;

        const input_type = elementAttributeValue(element.*, "type");
        const kind = inputValueAsDateKind(input_type) orelse return error.DomError;
        const text = try if (value) |ms| switch (kind) {
            .date => if (!std.math.isFinite(ms)) allocatorDupe(self, "") else formatDateValue(self.allocator, ms),
            .datetime_local => if (!std.math.isFinite(ms)) allocatorDupe(self, "") else formatDateTimeLocalValue(self.allocator, ms),
            .month => if (!std.math.isFinite(ms)) allocatorDupe(self, "") else formatMonthValue(self.allocator, ms),
            .week => if (!std.math.isFinite(ms)) allocatorDupe(self, "") else formatWeekValue(self.allocator, ms),
            .time => if (!std.math.isFinite(ms)) allocatorDupe(self, "") else formatTimeValue(self.allocator, ms),
        } else allocatorDupe(self, "");
        defer self.allocator.free(text);

        const arena_alloc = self.arena.allocator();
        const copy = try duplicateString(self, text);
        try upsertAttribute(arena_alloc, &element.attributes, "value", copy);
    }

    fn optionValueForNode(self: *const DomStore, allocator: std.mem.Allocator, node_id: NodeId) errors.Result([]u8) {
        const node = self.nodeAt(node_id) orelse return allocator.dupe(u8, "");
        const element = switch (node.kind) {
            .element => |element| element,
            else => return allocator.dupe(u8, ""),
        };

        if (elementAttributeValue(element, "value")) |value| {
            return allocator.dupe(u8, value);
        }

        return self.textContent(allocator, node_id);
    }

    fn selectValueForNode(self: *const DomStore, allocator: std.mem.Allocator, node_id: NodeId) errors.Result([]u8) {
        const node = self.nodeAt(node_id) orelse return allocator.dupe(u8, "");
        const element = switch (node.kind) {
            .element => |element| element,
            else => return allocator.dupe(u8, ""),
        };
        if (!std.mem.eql(u8, element.tag_name, "select")) return allocator.dupe(u8, "");

        var descendants: std.ArrayList(NodeId) = .empty;
        defer descendants.deinit(self.allocator);
        try self.collectSubtreeNodes(node_id, &descendants, self.allocator);

        var first_option: ?[]u8 = null;
        defer if (first_option) |value| allocator.free(value);
        for (descendants.items) |descendant_id| {
            if (!self.isOptionNode(descendant_id)) continue;
            const option_value = try self.optionValueForNode(allocator, descendant_id);
            defer allocator.free(option_value);
            if (first_option == null) {
                first_option = try allocator.dupe(u8, option_value);
            }
            if (self.optionSelectedForNode(descendant_id) == true) {
                return allocator.dupe(u8, option_value);
            }
        }

        if (first_option) |value| {
            first_option = null;
            return value;
        }

        return allocator.dupe(u8, "");
    }

    pub fn selectedIndexForNode(self: *const DomStore, node_id: NodeId) errors.Result(isize) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return error.DomError,
        };
        if (!std.mem.eql(u8, element.tag_name, "select")) return error.DomError;

        var descendants: std.ArrayList(NodeId) = .empty;
        defer descendants.deinit(self.allocator);
        try self.collectSubtreeNodes(node_id, &descendants, self.allocator);

        var option_index: isize = 0;
        for (descendants.items) |descendant_id| {
            if (!self.isOptionNode(descendant_id)) continue;
            if (self.optionSelectedForNode(descendant_id) == true) {
                return option_index;
            }
            option_index += 1;
        }

        return -1;
    }

    pub fn optionSelectedForNode(self: *const DomStore, node_id: NodeId) ?bool {
        const node = self.nodeAt(node_id) orelse return null;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return null,
        };
        if (!std.mem.eql(u8, element.tag_name, "option")) return null;
        return elementAttributeValue(element, "selected") != null;
    }

    fn isOptionNode(self: *const DomStore, node_id: NodeId) bool {
        const tag_name = self.tagNameForNode(node_id) orelse return false;
        return std.mem.eql(u8, tag_name, "option");
    }

    pub fn setOptionSelected(self: *DomStore, node_id: NodeId, selected: bool) errors.Result(void) {
        const node = self.nodeAtMut(node_id) orelse return error.HtmlParse;
        const element = switch (node.kind) {
            .element => |*element| element,
            else => return error.HtmlParse,
        };

        if (!std.mem.eql(u8, element.tag_name, "option")) {
            return error.DomError;
        }

        if (selected) {
            const arena_alloc = self.arena.allocator();
            try upsertAttribute(arena_alloc, &element.attributes, "selected", try duplicateString(self, ""));
        } else {
            removeAttributeByName(&element.attributes, "selected");
        }
        return;
    }

    pub fn setSelectSelectedIndex(self: *DomStore, node_id: NodeId, selected_index: isize) errors.Result(void) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return error.DomError,
        };
        if (!std.mem.eql(u8, element.tag_name, "select")) return error.DomError;
        if (selected_index < -1) return error.DomError;

        var descendants: std.ArrayList(NodeId) = .empty;
        defer descendants.deinit(self.allocator);
        try self.collectSubtreeNodes(node_id, &descendants, self.allocator);

        var option_index: isize = 0;
        for (descendants.items) |descendant_id| {
            if (!self.isOptionNode(descendant_id)) continue;
            try self.setOptionSelected(descendant_id, selected_index >= 0 and option_index == selected_index);
            option_index += 1;
        }

        return;
    }

    fn collectSubtreeNodes(
        self: *const DomStore,
        node_id: NodeId,
        output: *std.ArrayList(NodeId),
        allocator: std.mem.Allocator,
    ) errors.Result(void) {
        try output.append(allocator, node_id);
        const node = self.nodeAt(node_id) orelse return error.HtmlParse;
        for (node.children.items) |child_id| {
            try self.collectSubtreeNodes(child_id, output, allocator);
        }
    }

    pub fn collectInvalidSubtreeNodes(
        self: *const DomStore,
        node_id: NodeId,
        output: *std.ArrayList(NodeId),
        allocator: std.mem.Allocator,
    ) errors.Result(void) {
        if (isInvalidPseudoClass(self, node_id)) {
            try output.append(allocator, node_id);
        }

        const node = self.nodeAt(node_id) orelse return error.HtmlParse;
        for (node.children.items) |child_id| {
            try self.collectInvalidSubtreeNodes(child_id, output, allocator);
        }
    }

    fn serializationNamespaceForNode(self: *const DomStore, node_id: NodeId) SerializationNamespace {
        const node = self.nodeAt(node_id) orelse return .html;
        return switch (node.kind) {
            .document => .html,
            .element => |element| serializationNamespaceFromUri(element.namespace_uri),
            .text, .comment => {
                const parent_id = node.parent orelse return .html;
                return self.serializationNamespaceForNode(parent_id);
            },
        };
    }

    fn findElementByIdInSubtree(self: *const DomStore, node_id: NodeId, id: []const u8) ?NodeId {
        const node = self.nodeAt(node_id) orelse return null;
        const element = switch (node.kind) {
            .element => |element| element,
            else => null,
        };
        if (element) |element_data| {
            const actual_id = elementAttributeValue(element_data, "id") orelse return null;
            if (std.mem.eql(u8, actual_id, id)) {
                return node.id;
            }
        }

        for (node.children.items) |child_id| {
            if (self.findElementByIdInSubtree(child_id, id)) |found| {
                return found;
            }
        }

        return null;
    }

    fn findElementByNameInSubtree(self: *const DomStore, node_id: NodeId, name: []const u8) ?NodeId {
        const node = self.nodeAt(node_id) orelse return null;
        const element = switch (node.kind) {
            .element => |element| element,
            else => null,
        };
        if (element) |element_data| {
            const actual_name = elementAttributeValue(element_data, "name") orelse return null;
            if (std.mem.eql(u8, actual_name, name)) {
                return node.id;
            }
        }

        for (node.children.items) |child_id| {
            if (self.findElementByNameInSubtree(child_id, name)) |found| {
                return found;
            }
        }

        return null;
    }

    fn findFirstElementByTagNameInSubtree(self: *const DomStore, node_id: NodeId, tag_name: []const u8) ?NodeId {
        const node = self.nodeAt(node_id) orelse return null;
        if (switch (node.kind) {
            .element => |element| std.mem.eql(u8, element.tag_name, tag_name),
            else => false,
        }) {
            return node.id;
        }

        for (node.children.items) |child_id| {
            if (self.findFirstElementByTagNameInSubtree(child_id, tag_name)) |found| {
                return found;
            }
        }

        return null;
    }

    fn syncDocumentTitleFromDom(self: *DomStore) errors.Result(void) {
        const title_id = self.findFirstElementByTagNameInSubtree(self.documentId(), "title") orelse return;
        const title_text = try self.textContent(self.allocator, title_id);
        defer self.allocator.free(title_text);
        if (std.mem.eql(u8, title_text, self.document_title)) {
            return;
        }

        self.document_title = try duplicateString(self, title_text);
        return;
    }

    fn clearSelectionStateForSubtree(self: *DomStore, node_id: NodeId) void {
        _ = self.selection.remove(node_id);
        const node = self.nodeAt(node_id) orelse return;
        for (node.children.items) |child_id| {
            self.clearSelectionStateForSubtree(child_id);
        }
    }

    fn clearIndeterminateStateForSubtree(self: *DomStore, node_id: NodeId) void {
        _ = self.indeterminate.remove(node_id);
        const node = self.nodeAt(node_id) orelse return;
        for (node.children.items) |child_id| {
            self.clearIndeterminateStateForSubtree(child_id);
        }
    }

    fn clearCustomValidityStateForSubtree(self: *DomStore, node_id: NodeId) void {
        if (self.custom_validity.fetchRemove(node_id)) |entry| {
            self.allocator.free(entry.value);
        }
        const node = self.nodeAt(node_id) orelse return;
        for (node.children.items) |child_id| {
            self.clearCustomValidityStateForSubtree(child_id);
        }
    }

    fn clearTextTrackStateForSubtree(self: *DomStore, node_id: NodeId) void {
        _ = self.text_track.remove(node_id);
        const node = self.nodeAt(node_id) orelse return;
        for (node.children.items) |child_id| {
            self.clearTextTrackStateForSubtree(child_id);
        }
    }

    fn mediaPlaybackStateForNode(self: *const DomStore, node_id: NodeId) errors.Result(MediaPlaybackState) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return error.DomError,
        };
        if (!self.isMediaElementNode(element.tag_name)) return error.DomError;
        return self.media_playback.get(node_id) orelse MediaPlaybackState{};
    }

    fn ensureMediaPlaybackState(self: *DomStore, node_id: NodeId) errors.Result(*MediaPlaybackState) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return error.DomError,
        };
        if (!self.isMediaElementNode(element.tag_name)) return error.DomError;

        if (self.media_playback.getPtr(node_id)) |state| return state;
        try self.media_playback.put(self.allocator, node_id, .{});
        return self.media_playback.getPtr(node_id).?;
    }

    pub fn textTrackStateForNode(self: *const DomStore, node_id: NodeId) errors.Result(TextTrackState) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return error.DomError,
        };
        if (!std.mem.eql(u8, element.tag_name, "track")) return error.DomError;
        return self.text_track.get(node_id) orelse TextTrackState{ .element = node_id };
    }

    pub fn ensureTextTrackState(self: *DomStore, node_id: NodeId) errors.Result(*TextTrackState) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return error.DomError,
        };
        if (!std.mem.eql(u8, element.tag_name, "track")) return error.DomError;

        if (self.text_track.getPtr(node_id)) |state| return state;
        try self.text_track.put(self.allocator, node_id, .{ .element = node_id });
        return self.text_track.getPtr(node_id).?;
    }

    fn isTextareaNode(self: *const DomStore, node_id: NodeId) bool {
        const node = self.nodeAt(node_id) orelse return false;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return false,
        };
        return std.mem.eql(u8, element.tag_name, "textarea");
    }

    fn isMediaElementNode(self: *const DomStore, tag_name: []const u8) bool {
        _ = self;
        return std.ascii.eqlIgnoreCase(tag_name, "audio") or std.ascii.eqlIgnoreCase(tag_name, "video");
    }

    fn resetSelectionToEnd(self: *DomStore, node_id: NodeId) errors.Result(void) {
        const length = try self.selectionControlValueLength(node_id) orelse return error.DomError;
        try self.setSelectionRange(node_id, length, length, .none);
        return;
    }

    fn selectionControlValueLength(self: *const DomStore, node_id: NodeId) errors.Result(?usize) {
        const node = self.nodeAt(node_id) orelse return error.DomError;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return null,
        };

        if (std.mem.eql(u8, element.tag_name, "textarea")) {
            const text = try self.valueForNode(self.allocator, node_id);
            defer self.allocator.free(text);
            return text.len;
        }

        if (std.mem.eql(u8, element.tag_name, "input") and isSelectionInputType(elementAttributeValue(element, "type"))) {
            const value = try self.valueForNode(self.allocator, node_id);
            defer self.allocator.free(value);
            return value.len;
        }

        return null;
    }

    fn normalizeSelectionState(state: SelectionState, length: usize) SelectionState {
        const clamped_start = if (state.start > length) length else state.start;
        const clamped_end = if (state.end > length) length else state.end;
        if (clamped_start > clamped_end) {
            return .{
                .start = clamped_end,
                .end = clamped_end,
                .direction = state.direction,
            };
        }

        return .{
            .start = clamped_start,
            .end = clamped_end,
            .direction = state.direction,
        };
    }

    fn normalizeSelectionRange(
        start: usize,
        end: usize,
        length: usize,
        direction: SelectionDirection,
    ) SelectionState {
        const clamped_start = if (start > length) length else start;
        const clamped_end = if (end > length) length else end;
        if (clamped_start > clamped_end) {
            return .{
                .start = clamped_end,
                .end = clamped_end,
                .direction = direction,
            };
        }

        return .{
            .start = clamped_start,
            .end = clamped_end,
            .direction = direction,
        };
    }

    fn shiftSelectionIndex(index: usize, delta: isize) usize {
        if (delta >= 0) {
            return index + @as(usize, @intCast(delta));
        }

        const amount = @as(usize, @intCast(-delta));
        return index - amount;
    }

    fn targetNodeForFragment(self: *const DomStore, fragment: []const u8) ?NodeId {
        if (fragment.len == 0) return null;
        if (self.findElementById(fragment)) |node_id| {
            return node_id;
        }
        return self.findElementByNameInSubtree(self.documentId(), fragment);
    }

    fn nodeIsDescendantOf(self: *const DomStore, node_id: NodeId, ancestor_id: NodeId) bool {
        var current = parentOf(self, node_id);
        while (current) |current_id| {
            if (sameNodeId(current_id, ancestor_id)) return true;
            current = parentOf(self, current_id);
        }
        return false;
    }

    pub fn nodeContains(self: *const DomStore, container_id: NodeId, target_id: NodeId) bool {
        return comparisonContains(self, container_id, target_id);
    }

    pub fn nodeContainsFragment(self: *const DomStore, fragment_root_id: NodeId, target_id: NodeId) bool {
        if (sameNodeId(fragment_root_id, target_id)) return true;
        const target_root = comparisonRoot(self, target_id) orelse return false;
        return sameNodeId(target_root, fragment_root_id);
    }

    fn ensureMutationParent(self: *const DomStore, parent: NodeId) errors.Result(void) {
        const node = self.nodeAt(parent) orelse return error.DomError;
        switch (node.kind) {
            .document, .element => return,
            else => return error.DomError,
        }
    }

    fn childCount(self: *const DomStore, parent: NodeId) errors.Result(usize) {
        try self.ensureMutationParent(parent);
        const node = self.nodeAt(parent) orelse return error.DomError;
        return node.children.items.len;
    }

    fn childIndex(self: *const DomStore, parent: NodeId, child: NodeId) errors.Result(usize) {
        try self.ensureMutationParent(parent);
        const node = self.nodeAt(parent) orelse return error.DomError;
        for (node.children.items, 0..) |candidate, index| {
            if (sameNodeId(candidate, child)) {
                return index;
            }
        }
        return error.DomError;
    }

    fn validateMutationChildren(self: *const DomStore, parent: NodeId, children: []const NodeId) errors.Result(void) {
        try self.ensureMutationParent(parent);

        for (children, 0..) |child, index| {
            const node = self.nodeAt(child) orelse return error.DomError;
            if (sameNodeId(child, self.documentId())) return error.DomError;
            if (sameNodeId(child, parent)) return error.DomError;
            switch (node.kind) {
                .document => return error.DomError,
                else => {},
            }

            for (children[0..index]) |previous| {
                if (sameNodeId(previous, child)) return error.DomError;
            }

            var ancestor = parentOf(self, parent);
            while (ancestor) |ancestor_id| {
                if (sameNodeId(ancestor_id, child)) return error.DomError;
                ancestor = parentOf(self, ancestor_id);
            }
        }

        return;
    }

    fn insertChildrenAt(
        self: *DomStore,
        parent: NodeId,
        insertion_index: usize,
        children: []const NodeId,
    ) errors.Result(void) {
        try self.validateMutationChildren(parent, children);

        if (children.len == 0) {
            return;
        }

        const parent_len = try self.childCount(parent);
        const insertion_point = if (insertion_index > parent_len) parent_len else insertion_index;
        var adjusted_insertion_index = insertion_point;
        var moved_before_insertion: usize = 0;

        const ChildMove = struct {
            old_parent: NodeId,
            old_index: usize,
            child: NodeId,
        };

        var pending: std.ArrayList(ChildMove) = .empty;
        defer pending.deinit(self.allocator);

        for (children) |child| {
            const old_parent = parentOf(self, child) orelse continue;
            const old_index = try self.childIndex(old_parent, child);
            if (sameNodeId(old_parent, parent) and old_index < insertion_point) {
                moved_before_insertion += 1;
            }
            try pending.append(self.allocator, .{
                .old_parent = old_parent,
                .old_index = old_index,
                .child = child,
            });
        }

        if (moved_before_insertion > adjusted_insertion_index) {
            adjusted_insertion_index = 0;
        } else {
            adjusted_insertion_index -= moved_before_insertion;
        }

        while (pending.items.len > 0) {
            var chosen_index: usize = 0;
            var chosen_move = pending.items[0];
            for (pending.items[1..], 1..) |move, index| {
                if (move.old_index > chosen_move.old_index) {
                    chosen_index = index;
                    chosen_move = move;
                }
            }

            _ = pending.swapRemove(chosen_index);

            const old_parent_record = self.nodeAtMut(chosen_move.old_parent) orelse return error.DomError;
            if (chosen_move.old_index >= old_parent_record.children.items.len or !sameNodeId(old_parent_record.children.items[chosen_move.old_index], chosen_move.child)) {
                return error.DomError;
            }
            _ = old_parent_record.children.orderedRemove(chosen_move.old_index);

            const moved_record = self.nodeAtMut(chosen_move.child) orelse return error.DomError;
            moved_record.parent = null;
        }

        var insertion_pos = adjusted_insertion_index;
        const arena_alloc = self.arena.allocator();
        for (children) |child| {
            const parent_record = self.nodeAtMut(parent) orelse return error.DomError;
            try parent_record.children.insert(arena_alloc, insertion_pos, child);

            const record = self.nodeAtMut(child) orelse return error.DomError;
            record.parent = parent;
            insertion_pos += 1;
        }

        try self.syncDocumentTitleFromDom();
        if (self.isTextareaNode(parent)) {
            try self.resetSelectionToEnd(parent);
        }
        return;
    }

    fn addElement(
        self: *DomStore,
        parent: NodeId,
        tag_name: []const u8,
        attributes: std.ArrayListUnmanaged(Attribute),
        namespace_uri: ?[]const u8,
    ) errors.Result(NodeId) {
        const arena_alloc = self.arena.allocator();
        const node_id = NodeId.new(@intCast(self.nodes.items.len), 0);
        const parent_node = self.nodeAt(parent) orelse return error.HtmlParse;
        const parent_namespace = self.serializationNamespaceForNode(parent);
        const child_namespace = namespace_uri orelse namespaceUriForSerializationNamespace(
            serializationNamespaceForChild(parent_node, parent_namespace, tag_name),
        );
        try self.nodes.append(arena_alloc, .{
            .id = node_id,
            .parent = parent,
            .children = .{},
            .kind = .{ .element = .{
                .tag_name = tag_name,
                .namespace_uri = child_namespace,
                .attributes = attributes,
            } },
        });
        try self.attachNode(parent, node_id);
        return node_id;
    }

    fn addText(self: *DomStore, parent: NodeId, text: []const u8) errors.Result(NodeId) {
        const arena_alloc = self.arena.allocator();
        const text_copy = try arena_alloc.dupe(u8, text);
        const node_id = NodeId.new(@intCast(self.nodes.items.len), 0);
        try self.nodes.append(arena_alloc, .{
            .id = node_id,
            .parent = parent,
            .children = .{},
            .kind = .{ .text = text_copy },
        });
        try self.attachNode(parent, node_id);
        return node_id;
    }

    fn addComment(self: *DomStore, parent: NodeId, text: []const u8) errors.Result(NodeId) {
        const arena_alloc = self.arena.allocator();
        const text_copy = try arena_alloc.dupe(u8, text);
        const node_id = NodeId.new(@intCast(self.nodes.items.len), 0);
        try self.nodes.append(arena_alloc, .{
            .id = node_id,
            .parent = parent,
            .children = .{},
            .kind = .{ .comment = text_copy },
        });
        try self.attachNode(parent, node_id);
        return node_id;
    }

    pub fn createElementDetached(self: *DomStore, tag_name: []const u8) errors.Result(NodeId) {
        return self.createElementDetachedNS(tag_name, HtmlNamespaceUri);
    }

    pub fn createElementNS(self: *DomStore, namespace_uri: []const u8, tag_name: []const u8) errors.Result(NodeId) {
        if (!std.mem.eql(u8, namespace_uri, HtmlNamespaceUri) and
            !std.mem.eql(u8, namespace_uri, SvgNamespaceUri) and
            !std.mem.eql(u8, namespace_uri, MathMlNamespaceUri))
        {
            return error.DomError;
        }

        return self.createElementDetachedNS(tag_name, namespace_uri);
    }

    fn createElementDetachedNS(self: *DomStore, tag_name: []const u8, namespace_uri: []const u8) errors.Result(NodeId) {
        if (!isValidDetachedElementTagName(tag_name)) {
            return error.DomError;
        }

        const arena_alloc = self.arena.allocator();
        const node_id = NodeId.new(@intCast(self.nodes.items.len), 0);
        const stored_tag = if (std.mem.eql(u8, namespace_uri, HtmlNamespaceUri))
            try duplicateLowercase(self, tag_name)
        else
            try duplicateString(self, tag_name);
        try self.nodes.append(arena_alloc, .{
            .id = node_id,
            .parent = null,
            .children = .{},
            .kind = .{ .element = .{
                .tag_name = stored_tag,
                .namespace_uri = namespace_uri,
                .attributes = .{},
            } },
        });
        return node_id;
    }

    pub fn createTextNode(self: *DomStore, text: []const u8) errors.Result(NodeId) {
        const arena_alloc = self.arena.allocator();
        const text_copy = try arena_alloc.dupe(u8, text);
        const node_id = NodeId.new(@intCast(self.nodes.items.len), 0);
        try self.nodes.append(arena_alloc, .{
            .id = node_id,
            .parent = null,
            .children = .{},
            .kind = .{ .text = text_copy },
        });
        return node_id;
    }

    pub fn createComment(self: *DomStore, text: []const u8) errors.Result(NodeId) {
        const arena_alloc = self.arena.allocator();
        const text_copy = try arena_alloc.dupe(u8, text);
        const node_id = NodeId.new(@intCast(self.nodes.items.len), 0);
        try self.nodes.append(arena_alloc, .{
            .id = node_id,
            .parent = null,
            .children = .{},
            .kind = .{ .comment = text_copy },
        });
        return node_id;
    }

    fn attachNode(self: *DomStore, parent: NodeId, child: NodeId) errors.Result(void) {
        const parent_index: usize = @intCast(parent.index);
        if (parent_index >= self.nodes.items.len) return error.HtmlParse;
        const arena_alloc = self.arena.allocator();
        try self.nodes.items[parent_index].children.append(arena_alloc, child);
    }

    fn setSourceHtml(self: *DomStore, html: []const u8) errors.Result(void) {
        const arena_alloc = self.arena.allocator();
        self.source_html = try arena_alloc.dupe(u8, html);
    }

    fn parseSelectorChains(self: *const DomStore, allocator: std.mem.Allocator, selector: []const u8) errors.Result(std.ArrayList(SelectorChain)) {
        _ = self;
        const trimmed = std.mem.trim(u8, selector, " \t\r\n");
        if (trimmed.len == 0) return error.HtmlParse;

        var chains: std.ArrayList(SelectorChain) = .empty;
        errdefer {
            for (chains.items) |*chain| {
                chain.deinit(allocator);
            }
            chains.deinit(allocator);
        }

        try appendSelectorChains(allocator, trimmed, &chains);
        if (chains.items.len == 0) return error.HtmlParse;
        return chains;
    }

    fn deinitSelectorChains(self: *const DomStore, allocator: std.mem.Allocator, chains: *std.ArrayList(SelectorChain)) void {
        _ = self;
        for (chains.items) |*chain| {
            chain.deinit(allocator);
        }
        chains.deinit(allocator);
    }

    fn findFirstMatchingDescendant(
        self: *const DomStore,
        node_id: NodeId,
        chains: []const SelectorChain,
        scope_root: ?NodeId,
    ) errors.Result(?NodeId) {
        const node = self.nodeAt(node_id) orelse return error.HtmlParse;
        for (node.children.items) |child_id| {
            const child = self.nodeAt(child_id) orelse return error.HtmlParse;
            switch (child.kind) {
                .element => {
                    if (nodeMatchesAnyChain(self, child_id, chains, scope_root)) {
                        return child_id;
                    }
                },
                else => {},
            }

            if (try self.findFirstMatchingDescendant(child_id, chains, scope_root)) |found| {
                return found;
            }
        }

        return null;
    }
};

const HtmlParser = struct {
    input: []const u8,
    bytes: []const u8,
    pos: usize,

    fn init(input: []const u8) HtmlParser {
        return .{
            .input = input,
            .bytes = input,
            .pos = 0,
        };
    }

    fn parseInto(self: *HtmlParser, store: *DomStore) errors.Result(void) {
        var stack: std.ArrayListUnmanaged(NodeId) = .{};
        const arena_alloc = store.arena.allocator();
        try stack.append(arena_alloc, store.documentId());
        try self.parseIntoWithStack(store, &stack, 1);
    }

    fn parseFragmentInto(self: *HtmlParser, store: *DomStore, parent: NodeId) errors.Result(void) {
        var stack: std.ArrayListUnmanaged(NodeId) = .{};
        const arena_alloc = store.arena.allocator();
        try stack.append(arena_alloc, store.documentId());
        try stack.append(arena_alloc, parent);
        try self.parseIntoWithStack(store, &stack, 2);
    }

    fn parseIntoWithStack(
        self: *HtmlParser,
        store: *DomStore,
        stack: *std.ArrayListUnmanaged(NodeId),
        expected_stack_len: usize,
    ) errors.Result(void) {
        while (self.pos < self.bytes.len) {
            const current_parent = stack.items[stack.items.len - 1];
            if (store.tagNameForNode(current_parent)) |tag_name| {
                if (isRawTextElement(tag_name)) {
                    const rest = self.input[self.pos..];
                    const closing_tag = try std.fmt.allocPrint(store.arena.allocator(), "</{s}>", .{tag_name});
                    if (findCaseInsensitive(rest, closing_tag)) |offset| {
                        if (offset > 0) {
                            _ = try store.addText(current_parent, rest[0..offset]);
                            self.pos += offset;
                            continue;
                        }
                    } else {
                        if (rest.len > 0) {
                            _ = try store.addText(current_parent, rest);
                        }
                        self.pos = self.bytes.len;
                        break;
                    }
                }
            }

            if (self.currentByte() == '<') {
                if (self.startsWith("<!--")) {
                    try self.parseComment(store, current_parent);
                    continue;
                }

                if (self.startsWith("</")) {
                    try self.parseClosingTag(store, stack, expected_stack_len);
                    continue;
                }

                if (self.startsWith("<!")) {
                    try self.parseDeclaration();
                    continue;
                }

                try self.parseStartTag(store, stack);
                continue;
            }

            try self.parseText(store, current_parent);
        }

        if (stack.items.len != expected_stack_len) {
            return error.HtmlParse;
        }

        return;
    }

    fn startsWith(self: *HtmlParser, pattern: []const u8) bool {
        return std.mem.startsWith(u8, self.bytes[self.pos..], pattern);
    }

    fn currentByte(self: *HtmlParser) ?u8 {
        return if (self.pos < self.bytes.len) self.bytes[self.pos] else null;
    }

    fn skipWhitespace(self: *HtmlParser) void {
        while (self.currentByte()) |byte| {
            if (!isHtmlWhitespace(byte)) break;
            self.pos += 1;
        }
    }

    fn parseText(self: *HtmlParser, store: *DomStore, parent: NodeId) errors.Result(void) {
        const rest = self.input[self.pos..];
        const next_tag = std.mem.indexOfScalar(u8, rest, '<') orelse rest.len;
        const text = try decodeHtmlCharacterReferences(store, rest[0..next_tag]);
        self.pos += next_tag;
        if (text.len > 0) {
            _ = try store.addText(parent, text);
        }
    }

    fn parseComment(self: *HtmlParser, store: *DomStore, parent: NodeId) errors.Result(void) {
        self.pos += 4;
        const rest = self.input[self.pos..];
        const end = std.mem.indexOf(u8, rest, "-->") orelse return error.HtmlParse;
        const comment = rest[0..end];
        self.pos += end + 3;
        _ = try store.addComment(parent, comment);
    }

    fn parseDeclaration(self: *HtmlParser) errors.Result(void) {
        self.pos += 2;
        const rest = self.input[self.pos..];
        const end = std.mem.indexOfScalar(u8, rest, '>') orelse return error.HtmlParse;
        self.pos += end + 1;
    }

    fn parseStartTag(self: *HtmlParser, store: *DomStore, stack: *std.ArrayListUnmanaged(NodeId)) errors.Result(void) {
        self.pos += 1;
        if (self.currentByte() == null or !isTagNameByte(self.currentByte().?)) {
            return error.HtmlParse;
        }

        const tag_name = try self.parseNameToken(store);
        var attributes: std.ArrayListUnmanaged(Attribute) = .{};

        while (true) {
            self.skipWhitespace();
            if (self.pos >= self.bytes.len) return error.HtmlParse;

            if (self.startsWith("/>")) {
                self.pos += 2;
                _ = try store.addElement(stack.items[stack.items.len - 1], tag_name, attributes, null);
                return;
            }

            if (self.currentByte() == '>') {
                self.pos += 1;
                const node_id = try store.addElement(stack.items[stack.items.len - 1], tag_name, attributes, null);
                if (!isVoidElement(tag_name)) {
                    try stack.append(store.arena.allocator(), node_id);
                }
                return;
            }

            const attr_name = try self.parseNameToken(store);
            self.skipWhitespace();

            const attr_value = if (self.currentByte() == '=') blk: {
                self.pos += 1;
                self.skipWhitespace();
                break :blk try self.parseAttributeValue(store);
            } else blk: {
                break :blk try duplicateString(store, "");
            };

            try upsertAttribute(store.arena.allocator(), &attributes, attr_name, attr_value);
        }
    }

    fn parseClosingTag(
        self: *HtmlParser,
        store: *DomStore,
        stack: *std.ArrayListUnmanaged(NodeId),
        min_stack_len: usize,
    ) errors.Result(void) {
        self.pos += 2;
        self.skipWhitespace();
        if (self.pos >= self.bytes.len or !isTagNameByte(self.currentByte().?)) {
            return error.HtmlParse;
        }

        const closing_name = try self.parseNameToken(store);
        self.skipWhitespace();
        if (self.currentByte() != '>') return error.HtmlParse;
        self.pos += 1;

        if (stack.items.len <= min_stack_len) {
            return error.HtmlParse;
        }

        const open_id = stack.items[stack.items.len - 1];
        stack.items.len -= 1;
        const open_name = store.tagNameForNode(open_id) orelse return error.HtmlParse;
        if (!std.mem.eql(u8, open_name, closing_name)) {
            return error.HtmlParse;
        }
    }

    fn parseNameToken(self: *HtmlParser, store: *DomStore) errors.Result([]const u8) {
        const start = self.pos;
        while (self.currentByte()) |byte| {
            if (!isTagNameByte(byte)) break;
            self.pos += 1;
        }
        if (self.pos == start) return error.HtmlParse;
        return duplicateLowercase(store, self.input[start..self.pos]);
    }

    fn parseAttributeValue(self: *HtmlParser, store: *DomStore) errors.Result([]const u8) {
        const current = self.currentByte() orelse return error.HtmlParse;
        if (current == '"' or current == '\'') {
            const quote = current;
            self.pos += 1;
            const rest = self.input[self.pos..];
            const end = std.mem.indexOfScalar(u8, rest, quote) orelse return error.HtmlParse;
            const value = try decodeHtmlCharacterReferences(store, rest[0..end]);
            self.pos += end + 1;
            return value;
        }

        const start = self.pos;
        while (self.currentByte()) |byte| {
            if (isHtmlWhitespace(byte) or byte == '>') break;
            self.pos += 1;
        }
        if (self.pos == start) return error.HtmlParse;
        return decodeHtmlCharacterReferences(store, self.input[start..self.pos]);
    }
};

fn duplicateString(store: *DomStore, value: []const u8) errors.Result([]const u8) {
    return try store.arena.allocator().dupe(u8, value);
}

fn decodeHtmlCharacterReferences(store: *DomStore, value: []const u8) errors.Result([]const u8) {
    if (std.mem.indexOfScalar(u8, value, '&') == null) {
        return duplicateString(store, value);
    }

    var output: std.ArrayList(u8) = .empty;
    errdefer output.deinit(store.arena.allocator());

    var index: usize = 0;
    while (index < value.len) {
        if (value[index] != '&') {
            try output.append(store.arena.allocator(), value[index]);
            index += 1;
            continue;
        }

        if (try decodeHtmlCharacterReferenceInto(store, value, &index, &output)) {
            continue;
        }

        try output.append(store.arena.allocator(), '&');
        index += 1;
    }

    return try output.toOwnedSlice(store.arena.allocator());
}

fn decodeHtmlCharacterReferenceInto(
    store: *DomStore,
    value: []const u8,
    index: *usize,
    output: *std.ArrayList(u8),
) errors.Result(bool) {
    const start = index.*;
    if (start + 1 >= value.len or value[start] != '&') return false;

    const allocator = store.arena.allocator();
    if (value[start + 1] == '#') {
        var pos = start + 2;
        var base: u8 = 10;
        if (pos < value.len and (value[pos] == 'x' or value[pos] == 'X')) {
            base = 16;
            pos += 1;
        }

        const digits_start = pos;
        while (pos < value.len) : (pos += 1) {
            const byte = value[pos];
            if (base == 10) {
                if (!std.ascii.isDigit(byte)) break;
            } else if (!isHexDigit(byte)) {
                break;
            }
        }
        if (pos == digits_start) return false;

        const number = std.fmt.parseInt(u21, value[digits_start..pos], base) catch return false;
        if (pos < value.len and value[pos] == ';') pos += 1;
        try appendUtf8Codepoint(output, allocator, number);
        index.* = pos;
        return true;
    }

    const named_references = [_]struct { name: []const u8, replacement: []const u8 }{
        .{ .name = "amp", .replacement = "&" },
        .{ .name = "AMP", .replacement = "&" },
        .{ .name = "lt", .replacement = "<" },
        .{ .name = "LT", .replacement = "<" },
        .{ .name = "gt", .replacement = ">" },
        .{ .name = "GT", .replacement = ">" },
        .{ .name = "quot", .replacement = "\"" },
        .{ .name = "QUOT", .replacement = "\"" },
        .{ .name = "nbsp", .replacement = "\xC2\xA0" },
        .{ .name = "NBSP", .replacement = "\xC2\xA0" },
        .{ .name = "copy", .replacement = "\xC2\xA9" },
        .{ .name = "COPY", .replacement = "\xC2\xA9" },
        .{ .name = "reg", .replacement = "\xC2\xAE" },
        .{ .name = "REG", .replacement = "\xC2\xAE" },
        .{ .name = "apos", .replacement = "'" },
        .{ .name = "APOS", .replacement = "'" },
    };

    for (named_references) |reference| {
        if (!startsWithCaseInsensitive(value[start + 1 ..], reference.name)) continue;
        const after_name = start + 1 + reference.name.len;
        if (after_name < value.len and value[after_name] != ';' and std.ascii.isAlphanumeric(value[after_name])) {
            continue;
        }
        try output.appendSlice(allocator, reference.replacement);
        index.* = if (after_name < value.len and value[after_name] == ';') after_name + 1 else after_name;
        return true;
    }

    return false;
}

fn appendUtf8Codepoint(output: *std.ArrayList(u8), allocator: std.mem.Allocator, codepoint: u21) errors.Result(void) {
    var buf: [4]u8 = undefined;
    const len = std.unicode.utf8Encode(codepoint, &buf) catch return error.HtmlParse;
    try output.appendSlice(allocator, buf[0..len]);
}

fn startsWithCaseInsensitive(haystack: []const u8, needle: []const u8) bool {
    if (haystack.len < needle.len) return false;
    for (needle, 0..) |needle_byte, index| {
        if (std.ascii.toLower(haystack[index]) != std.ascii.toLower(needle_byte)) return false;
    }
    return true;
}

fn duplicateLowercase(store: *DomStore, value: []const u8) errors.Result([]const u8) {
    const out = try store.arena.allocator().alloc(u8, value.len);
    for (value, 0..) |byte, i| {
        out[i] = std.ascii.toLower(byte);
    }
    return out;
}

fn isValidDetachedElementTagName(name: []const u8) bool {
    if (name.len == 0) return false;
    for (name) |byte| {
        if (!isTagNameByte(byte)) return false;
    }
    return true;
}

fn trimAttributeName(name: []const u8) ?[]const u8 {
    const trimmed = std.mem.trim(u8, name, " \t\r\n");
    return if (trimmed.len == 0) null else trimmed;
}

fn popoverStateFromAttribute(value: ?[]const u8) PopoverState {
    const raw = value orelse return .no_popover;
    const trimmed = std.mem.trim(u8, raw, " \t\r\n\x0c");
    if (trimmed.len == 0) return .auto;
    if (std.ascii.eqlIgnoreCase(trimmed, "auto")) return .auto;
    if (std.ascii.eqlIgnoreCase(trimmed, "hint")) return .hint;
    if (std.ascii.eqlIgnoreCase(trimmed, "manual")) return .manual;
    return .manual;
}

fn findAttributeIndexByName(attributes: []const Attribute, name: []const u8) ?usize {
    for (attributes, 0..) |attribute, index| {
        if (asciiEqualIgnoreCase(attribute.name, name)) {
            return index;
        }
    }
    return null;
}

fn upsertAttribute(
    allocator: std.mem.Allocator,
    attributes: *std.ArrayListUnmanaged(Attribute),
    name: []const u8,
    value: []const u8,
) errors.Result(void) {
    for (attributes.items, 0..) |*attribute, index| {
        _ = index;
        if (std.mem.eql(u8, attribute.name, name)) {
            attribute.value = value;
            return;
        }
    }
    try attributes.append(allocator, .{
        .name = name,
        .value = value,
    });
}

fn removeAttributeByName(attributes: *std.ArrayListUnmanaged(Attribute), name: []const u8) void {
    var index: usize = 0;
    while (index < attributes.items.len) : (index += 1) {
        if (std.mem.eql(u8, attributes.items[index].name, name)) {
            _ = attributes.orderedRemove(index);
            return;
        }
    }
}

fn isHtmlWhitespace(byte: u8) bool {
    return switch (byte) {
        ' ', '\n', '\r', '\t', 0x0c => true,
        else => false,
    };
}

fn isHexDigit(byte: u8) bool {
    return std.ascii.isDigit(byte) or (byte >= 'a' and byte <= 'f') or (byte >= 'A' and byte <= 'F');
}

fn isTagNameByte(byte: u8) bool {
    return std.ascii.isAlphanumeric(byte) or byte == '-' or byte == '_' or byte == ':';
}

fn isTextInputType(input_type: ?[]const u8) bool {
    const value = input_type orelse "text";
    return std.mem.eql(u8, value, "text") or std.mem.eql(u8, value, "search") or std.mem.eql(u8, value, "url") or std.mem.eql(u8, value, "tel") or std.mem.eql(u8, value, "email") or std.mem.eql(u8, value, "password") or std.mem.eql(u8, value, "number") or std.mem.eql(u8, value, "date") or std.mem.eql(u8, value, "datetime-local") or std.mem.eql(u8, value, "month") or std.mem.eql(u8, value, "week") or std.mem.eql(u8, value, "time") or std.mem.eql(u8, value, "color");
}

fn isSelectionInputType(input_type: ?[]const u8) bool {
    const value = input_type orelse "text";
    return std.mem.eql(u8, value, "text") or std.mem.eql(u8, value, "search") or std.mem.eql(u8, value, "url") or std.mem.eql(u8, value, "tel") or std.mem.eql(u8, value, "password");
}

fn isPatternInputType(input_type: ?[]const u8) bool {
    const value = input_type orelse "text";
    return std.mem.eql(u8, value, "text") or
        std.mem.eql(u8, value, "search") or
        std.mem.eql(u8, value, "url") or
        std.mem.eql(u8, value, "tel") or
        std.mem.eql(u8, value, "email") or
        std.mem.eql(u8, value, "password");
}

fn isLengthInputType(input_type: ?[]const u8) bool {
    const value = input_type orelse "text";
    return std.mem.eql(u8, value, "text") or std.mem.eql(u8, value, "search") or std.mem.eql(u8, value, "url") or std.mem.eql(u8, value, "tel") or std.mem.eql(u8, value, "email") or std.mem.eql(u8, value, "password");
}

fn isCheckableInputType(input_type: ?[]const u8) bool {
    const value = input_type orelse "text";
    return std.mem.eql(u8, value, "checkbox") or std.mem.eql(u8, value, "radio");
}

fn isFileInputType(input_type: ?[]const u8) bool {
    const value = input_type orelse "text";
    return std.mem.eql(u8, value, "file");
}

fn isSelectorTokenByte(byte: u8) bool {
    return std.ascii.isAlphanumeric(byte) or byte == '-' or byte == '_';
}

fn isSelectorCombinatorByte(byte: u8) bool {
    return byte == '>' or byte == '+' or byte == '~' or byte == ',';
}

fn isVoidElement(tag_name: []const u8) bool {
    return std.mem.eql(u8, tag_name, "area") or std.mem.eql(u8, tag_name, "base") or std.mem.eql(u8, tag_name, "br") or std.mem.eql(u8, tag_name, "col") or std.mem.eql(u8, tag_name, "embed") or std.mem.eql(u8, tag_name, "hr") or std.mem.eql(u8, tag_name, "img") or std.mem.eql(u8, tag_name, "input") or std.mem.eql(u8, tag_name, "link") or std.mem.eql(u8, tag_name, "meta") or std.mem.eql(u8, tag_name, "param") or std.mem.eql(u8, tag_name, "source") or std.mem.eql(u8, tag_name, "track") or std.mem.eql(u8, tag_name, "wbr");
}

fn isRawTextElement(tag_name: []const u8) bool {
    return std.mem.eql(u8, tag_name, "script") or std.mem.eql(u8, tag_name, "style");
}

fn findCaseInsensitive(haystack: []const u8, needle: []const u8) ?usize {
    if (needle.len == 0) return 0;
    if (haystack.len < needle.len) return null;

    var offset: usize = 0;
    while (offset + needle.len <= haystack.len) : (offset += 1) {
        var matched = true;
        for (needle, 0..) |needle_byte, i| {
            if (std.ascii.toLower(haystack[offset + i]) != std.ascii.toLower(needle_byte)) {
                matched = false;
                break;
            }
        }
        if (matched) return offset;
    }
    return null;
}

fn writeIndent(
    output: *std.ArrayList(u8),
    allocator: std.mem.Allocator,
    indent: usize,
) errors.Result(void) {
    var i: usize = 0;
    while (i < indent) : (i += 1) {
        try output.appendSlice(allocator, "  ");
    }
}

fn writeEscapedText(
    output: *std.ArrayList(u8),
    allocator: std.mem.Allocator,
    value: []const u8,
) errors.Result(void) {
    for (value) |byte| {
        switch (byte) {
            '\\' => try output.appendSlice(allocator, "\\\\"),
            '"' => try output.appendSlice(allocator, "\\\""),
            '\n' => try output.appendSlice(allocator, "\\n"),
            '\r' => try output.appendSlice(allocator, "\\r"),
            '\t' => try output.appendSlice(allocator, "\\t"),
            else => try output.append(allocator, byte),
        }
    }
}

fn writeEscapedAttr(
    output: *std.ArrayList(u8),
    allocator: std.mem.Allocator,
    value: []const u8,
) errors.Result(void) {
    try writeEscapedAttrValue(output, allocator, value, '"');
}

fn writeEscapedAttrValue(
    output: *std.ArrayList(u8),
    allocator: std.mem.Allocator,
    value: []const u8,
    quote: u8,
) errors.Result(void) {
    for (value) |byte| {
        switch (byte) {
            '&' => try output.appendSlice(allocator, "&amp;"),
            '<' => try output.appendSlice(allocator, "&lt;"),
            '>' => try output.appendSlice(allocator, "&gt;"),
            '"' => if (quote == '"') {
                try output.appendSlice(allocator, "&quot;");
            } else {
                try output.append(allocator, byte);
            },
            '\'' => if (quote == '\'') {
                try output.appendSlice(allocator, "&apos;");
            } else {
                try output.append(allocator, byte);
            },
            else => try output.append(allocator, byte),
        }
    }
}

fn appendEscapedHtmlText(
    output: *std.ArrayList(u8),
    allocator: std.mem.Allocator,
    value: []const u8,
) errors.Result(void) {
    for (value) |byte| {
        switch (byte) {
            '&' => try output.appendSlice(allocator, "&amp;"),
            '<' => try output.appendSlice(allocator, "&lt;"),
            '>' => try output.appendSlice(allocator, "&gt;"),
            else => try output.append(allocator, byte),
        }
    }
}

fn lessThanBytes(left: []const u8, right: []const u8) bool {
    const limit = if (left.len < right.len) left.len else right.len;
    var index: usize = 0;
    while (index < limit) : (index += 1) {
        if (left[index] < right[index]) return true;
        if (left[index] > right[index]) return false;
    }
    return left.len < right.len;
}

fn appendSelectorChains(
    allocator: std.mem.Allocator,
    selector: []const u8,
    chains: *std.ArrayList(SelectorChain),
) errors.Result(void) {
    var start: usize = 0;
    var paren_depth: usize = 0;
    var bracket_depth: usize = 0;
    var quote: ?u8 = null;

    var index: usize = 0;
    while (index < selector.len) : (index += 1) {
        const byte = selector[index];
        if (quote) |current_quote| {
            if (byte == current_quote) {
                quote = null;
            }
            continue;
        }

        switch (byte) {
            '"', '\'' => quote = byte,
            '[' => bracket_depth += 1,
            ']' => {
                if (bracket_depth == 0) return error.HtmlParse;
                bracket_depth -= 1;
            },
            '(' => paren_depth += 1,
            ')' => {
                if (paren_depth == 0) return error.HtmlParse;
                paren_depth -= 1;
            },
            ',' => {
                if (bracket_depth != 0 or paren_depth != 0) continue;
                const item = std.mem.trim(u8, selector[start..index], " \t\r\n");
                if (item.len == 0) return error.HtmlParse;
                try appendSelectorChain(allocator, chains, item);
                start = index + 1;
            },
            else => {},
        }
    }

    if (quote != null or bracket_depth != 0 or paren_depth != 0) return error.HtmlParse;

    const item = std.mem.trim(u8, selector[start..], " \t\r\n");
    if (item.len == 0) return error.HtmlParse;
    try appendSelectorChain(allocator, chains, item);
}

fn appendSelectorChain(
    allocator: std.mem.Allocator,
    chains: *std.ArrayList(SelectorChain),
    selector: []const u8,
) errors.Result(void) {
    var chain = try parseSelectorChain(allocator, selector);
    errdefer chain.deinit(allocator);
    try chains.append(allocator, chain);
}

fn parseSelectorChain(
    allocator: std.mem.Allocator,
    selector: []const u8,
) errors.Result(SelectorChain) {
    var chain: SelectorChain = .{};
    errdefer chain.deinit(allocator);

    var pos: usize = 0;
    try chain.parts.append(allocator, try parseSelectorCompound(allocator, selector, &pos));

    while (pos < selector.len) {
        const had_whitespace = skipSelectorWhitespace(selector, &pos);
        if (pos >= selector.len) break;

        const relation = switch (selector[pos]) {
            '>' => blk: {
                pos += 1;
                break :blk SelectorCombinator.child;
            },
            '+' => blk: {
                pos += 1;
                break :blk SelectorCombinator.adjacent_sibling;
            },
            '~' => blk: {
                pos += 1;
                break :blk SelectorCombinator.general_sibling;
            },
            ',' => return error.HtmlParse,
            else => blk: {
                if (!had_whitespace) return error.HtmlParse;
                break :blk SelectorCombinator.descendant;
            },
        };

        _ = skipSelectorWhitespace(selector, &pos);
        if (pos >= selector.len) return error.HtmlParse;

        try chain.relations.append(allocator, relation);
        try chain.parts.append(allocator, try parseSelectorCompound(allocator, selector, &pos));
    }

    return chain;
}

fn parseSelectorCompound(
    allocator: std.mem.Allocator,
    selector: []const u8,
    pos: *usize,
) errors.Result(SelectorQuery) {
    var query: SelectorQuery = .{};
    errdefer query.deinit(allocator);

    var saw_token = false;
    while (pos.* < selector.len) {
        const byte = selector[pos.*];
        if (isHtmlWhitespace(byte) or isSelectorCombinatorByte(byte)) break;

        switch (byte) {
            '*' => {
                pos.* += 1;
                if (query.universal or query.tag != null or query.id != null or query.classes.items.len > 0 or query.attributes.items.len > 0 or query.pseudos.items.len > 0) {
                    return error.HtmlParse;
                }
                query.universal = true;
                saw_token = true;
            },
            '#' => {
                pos.* += 1;
                const token = try parseSelectorToken(selector, pos);
                if (query.id != null) return error.HtmlParse;
                query.id = token;
                saw_token = true;
            },
            '.' => {
                pos.* += 1;
                const token = try parseSelectorToken(selector, pos);
                try query.classes.append(allocator, token);
                saw_token = true;
            },
            ':' => {
                pos.* += 1;
                const pseudo = try parseSelectorPseudoClass(allocator, selector, pos);
                try query.pseudos.append(allocator, pseudo);
                saw_token = true;
            },
            '[' => {
                try parseSelectorAttribute(allocator, selector, pos, &query);
                saw_token = true;
            },
            else => {
                if (!isSelectorTokenByte(byte)) return error.HtmlParse;
                const token = try parseSelectorToken(selector, pos);
                if (query.tag != null or query.universal) return error.HtmlParse;
                query.tag = token;
                saw_token = true;
            },
        }
    }

    if (!saw_token) return error.HtmlParse;
    return query;
}

fn parseSelectorToken(selector: []const u8, pos: *usize) errors.Result([]const u8) {
    const start = pos.*;
    while (pos.* < selector.len) {
        const byte = selector[pos.*];
        if (!isSelectorTokenByte(byte)) break;
        pos.* += 1;
    }
    if (pos.* == start) return error.HtmlParse;
    return selector[start..pos.*];
}

fn parseSelectorAttribute(
    allocator: std.mem.Allocator,
    selector: []const u8,
    pos: *usize,
    query: *SelectorQuery,
) errors.Result(void) {
    if (selector[pos.*] != '[') return error.HtmlParse;
    pos.* += 1;
    _ = skipSelectorWhitespace(selector, pos);

    const name = try parseSelectorToken(selector, pos);
    _ = skipSelectorWhitespace(selector, pos);

    var operator: SelectorAttributeOperator = .exists;
    var value: ?[]const u8 = null;
    var case_sensitivity: SelectorAttributeCaseSensitivity = .case_sensitive;

    if (pos.* < selector.len and selector[pos.*] != ']') {
        operator = try parseSelectorAttributeOperator(selector, pos);
        _ = skipSelectorWhitespace(selector, pos);
        value = try parseSelectorAttributeValue(selector, pos);
        _ = skipSelectorWhitespace(selector, pos);

        if (pos.* < selector.len and (selector[pos.*] == 'i' or selector[pos.*] == 's')) {
            case_sensitivity = if (selector[pos.*] == 'i')
                .case_insensitive
            else
                .case_sensitive;
            pos.* += 1;
            _ = skipSelectorWhitespace(selector, pos);
        }
    }

    if (pos.* >= selector.len or selector[pos.*] != ']') return error.HtmlParse;
    pos.* += 1;

    try query.attributes.append(allocator, .{
        .name = name,
        .operator = operator,
        .value = value,
        .case_sensitivity = case_sensitivity,
    });
}

fn parseSelectorAttributeOperator(
    selector: []const u8,
    pos: *usize,
) errors.Result(SelectorAttributeOperator) {
    if (pos.* >= selector.len) return error.HtmlParse;

    return switch (selector[pos.*]) {
        '=' => blk: {
            pos.* += 1;
            break :blk .equals;
        },
        '^' => blk: {
            if (pos.* + 1 >= selector.len or selector[pos.* + 1] != '=') return error.HtmlParse;
            pos.* += 2;
            break :blk .prefix;
        },
        '$' => blk: {
            if (pos.* + 1 >= selector.len or selector[pos.* + 1] != '=') return error.HtmlParse;
            pos.* += 2;
            break :blk .suffix;
        },
        '*' => blk: {
            if (pos.* + 1 >= selector.len or selector[pos.* + 1] != '=') return error.HtmlParse;
            pos.* += 2;
            break :blk .contains;
        },
        '~' => blk: {
            if (pos.* + 1 >= selector.len or selector[pos.* + 1] != '=') return error.HtmlParse;
            pos.* += 2;
            break :blk .contains_word;
        },
        '|' => blk: {
            if (pos.* + 1 >= selector.len or selector[pos.* + 1] != '=') return error.HtmlParse;
            pos.* += 2;
            break :blk .hyphen_prefix;
        },
        else => error.HtmlParse,
    };
}

fn parseSelectorAttributeValue(
    selector: []const u8,
    pos: *usize,
) errors.Result([]const u8) {
    if (pos.* >= selector.len) return error.HtmlParse;

    const current = selector[pos.*];
    if (current == '"' or current == '\'') {
        const quote = current;
        pos.* += 1;
        const start = pos.*;
        while (pos.* < selector.len and selector[pos.*] != quote) {
            pos.* += 1;
        }
        if (pos.* >= selector.len) return error.HtmlParse;
        const value = selector[start..pos.*];
        pos.* += 1;
        return value;
    }

    const start = pos.*;
    while (pos.* < selector.len) {
        const byte = selector[pos.*];
        if (isHtmlWhitespace(byte) or byte == ']') break;
        pos.* += 1;
    }
    if (pos.* == start) return error.HtmlParse;
    return selector[start..pos.*];
}

fn parseSelectorPseudoClass(
    allocator: std.mem.Allocator,
    selector: []const u8,
    pos: *usize,
) errors.Result(SelectorPseudoClass) {
    const name = try parseSelectorToken(selector, pos);
    if (name.len == 0) return error.HtmlParse;

    if (asciiEqualIgnoreCase(name, "has")) {
        const argument = try parseSelectorParenthesizedArgument(selector, pos);
        const relative_selectors = try parseSelectorRelativeSelectors(allocator, argument);
        return .{ .has = relative_selectors };
    }

    if (asciiEqualIgnoreCase(name, "lang")) {
        const langs = try parseSelectorLangArgument(allocator, selector, pos);
        return .{ .lang = langs };
    }

    if (asciiEqualIgnoreCase(name, "dir")) {
        const direction = try parseSelectorDirArgument(selector, pos);
        return .{ .dir = direction };
    }

    if (asciiEqualIgnoreCase(name, "not")) {
        const argument = try parseSelectorParenthesizedArgument(selector, pos);
        const selectors = try parseSelectorChainListArgument(allocator, argument);
        return .{ .not = selectors };
    }

    if (asciiEqualIgnoreCase(name, "is")) {
        const argument = try parseSelectorParenthesizedArgument(selector, pos);
        const selectors = try parseSelectorChainListArgument(allocator, argument);
        return .{ .is = selectors };
    }

    if (asciiEqualIgnoreCase(name, "where")) {
        const argument = try parseSelectorParenthesizedArgument(selector, pos);
        const selectors = try parseSelectorChainListArgument(allocator, argument);
        return .{ .where = selectors };
    }

    if (asciiEqualIgnoreCase(name, "nth-child")) {
        const pattern = try parseSelectorNthChildArgument(allocator, selector, pos);
        return .{ .nth_child = pattern };
    }

    if (asciiEqualIgnoreCase(name, "nth-last-child")) {
        const pattern = try parseSelectorNthChildArgument(allocator, selector, pos);
        return .{ .nth_last_child = pattern };
    }

    if (asciiEqualIgnoreCase(name, "nth-of-type")) {
        const pattern = try parseSelectorNthChildArgument(allocator, selector, pos);
        return .{ .nth_of_type = pattern };
    }

    if (asciiEqualIgnoreCase(name, "nth-last-of-type")) {
        const pattern = try parseSelectorNthChildArgument(allocator, selector, pos);
        return .{ .nth_last_of_type = pattern };
    }

    if (asciiEqualIgnoreCase(name, "popover-open")) return .popover_open;

    if (pos.* < selector.len and selector[pos.*] == '(') return error.HtmlParse;

    return if (asciiEqualIgnoreCase(name, "scope"))
        .scope
    else if (asciiEqualIgnoreCase(name, "root"))
        .root
    else if (asciiEqualIgnoreCase(name, "empty"))
        .empty
    else if (asciiEqualIgnoreCase(name, "first-child"))
        .first_child
    else if (asciiEqualIgnoreCase(name, "last-child"))
        .last_child
    else if (asciiEqualIgnoreCase(name, "only-child"))
        .only_child
    else if (asciiEqualIgnoreCase(name, "first-of-type"))
        .first_of_type
    else if (asciiEqualIgnoreCase(name, "last-of-type"))
        .last_of_type
    else if (asciiEqualIgnoreCase(name, "only-of-type"))
        .only_of_type
    else if (asciiEqualIgnoreCase(name, "checked"))
        .checked
    else if (asciiEqualIgnoreCase(name, "disabled"))
        .disabled
    else if (asciiEqualIgnoreCase(name, "enabled"))
        .enabled
    else if (asciiEqualIgnoreCase(name, "required"))
        .required
    else if (asciiEqualIgnoreCase(name, "optional"))
        .optional
    else if (asciiEqualIgnoreCase(name, "link"))
        .link
    else if (asciiEqualIgnoreCase(name, "any-link"))
        .any_link
    else if (asciiEqualIgnoreCase(name, "defined"))
        .defined
    else if (asciiEqualIgnoreCase(name, "placeholder-shown"))
        .placeholder_shown
    else if (asciiEqualIgnoreCase(name, "blank"))
        .blank
    else if (asciiEqualIgnoreCase(name, "indeterminate"))
        .indeterminate
    else if (asciiEqualIgnoreCase(name, "default"))
        .default
    else if (asciiEqualIgnoreCase(name, "valid"))
        .valid
    else if (asciiEqualIgnoreCase(name, "invalid"))
        .invalid
    else if (asciiEqualIgnoreCase(name, "in-range"))
        .in_range
    else if (asciiEqualIgnoreCase(name, "out-of-range"))
        .out_of_range
    else if (asciiEqualIgnoreCase(name, "read-only"))
        .read_only
    else if (asciiEqualIgnoreCase(name, "read-write"))
        .read_write
    else if (asciiEqualIgnoreCase(name, "target"))
        .target
    else if (asciiEqualIgnoreCase(name, "focus"))
        .focus
    else if (asciiEqualIgnoreCase(name, "focus-visible"))
        .focus_visible
    else if (asciiEqualIgnoreCase(name, "focus-within"))
        .focus_within
    else {
        return error.HtmlParse;
    };
}

fn parseSelectorLangArgument(
    allocator: std.mem.Allocator,
    selector: []const u8,
    pos: *usize,
) errors.Result(std.ArrayListUnmanaged([]const u8)) {
    const argument = try parseSelectorParenthesizedArgument(selector, pos);
    const trimmed = std.mem.trim(u8, argument, " \t\r\n\x0c");
    if (trimmed.len == 0) return error.HtmlParse;

    var langs: std.ArrayListUnmanaged([]const u8) = .{};
    errdefer langs.deinit(allocator);

    var items = std.mem.splitScalar(u8, trimmed, ',');
    while (items.next()) |item_raw| {
        const item = std.mem.trim(u8, item_raw, " \t\r\n\x0c");
        if (item.len == 0) return error.HtmlParse;

        var item_pos: usize = 0;
        const lang = try parseSelectorToken(item, &item_pos);
        if (item_pos != item.len) return error.HtmlParse;

        for (lang) |byte| {
            if (!std.ascii.isAlphanumeric(byte) and byte != '-') return error.HtmlParse;
        }

        try langs.append(allocator, lang);
    }

    if (langs.items.len == 0) return error.HtmlParse;
    return langs;
}

fn parseSelectorDirArgument(
    selector: []const u8,
    pos: *usize,
) errors.Result(SelectorDirValue) {
    const argument = try parseSelectorParenthesizedArgument(selector, pos);
    const trimmed = std.mem.trim(u8, argument, " \t\r\n\x0c");
    if (trimmed.len == 0) return error.HtmlParse;

    var item_pos: usize = 0;
    const dir = try parseSelectorToken(trimmed, &item_pos);
    if (item_pos != trimmed.len) return error.HtmlParse;

    if (asciiEqualIgnoreCase(dir, "ltr")) return .ltr;
    if (asciiEqualIgnoreCase(dir, "rtl")) return .rtl;
    return error.HtmlParse;
}

const NthChildArgumentSplit = struct {
    formula: []const u8,
    of_selectors: ?[]const u8,
};

fn parseSelectorNthChildArgument(
    allocator: std.mem.Allocator,
    selector: []const u8,
    pos: *usize,
) errors.Result(SelectorNthChildPattern) {
    const argument = try parseSelectorParenthesizedArgument(selector, pos);
    const split = try splitNthChildArgument(argument);

    var formula: std.ArrayList(u8) = .empty;
    defer formula.deinit(allocator);

    for (split.formula) |byte| {
        if (!isHtmlWhitespace(byte)) {
            try formula.append(allocator, std.ascii.toLower(byte));
        }
    }

    if (formula.items.len == 0) return error.HtmlParse;

    var pattern = if (std.mem.eql(u8, formula.items, "odd")) SelectorNthChildPattern{
        .step = 2,
        .offset = 1,
        .of_selectors = null,
    } else if (std.mem.eql(u8, formula.items, "even")) SelectorNthChildPattern{
        .step = 2,
        .offset = 0,
        .of_selectors = null,
    } else if (std.mem.indexOfScalar(u8, formula.items, 'n')) |n_index| blk: {
        if (std.mem.indexOfScalar(u8, formula.items[n_index + 1 ..], 'n') != null) {
            return error.HtmlParse;
        }

        const step = switch (formula.items[0..n_index].len) {
            0 => 1,
            1 => switch (formula.items[0]) {
                '+' => 1,
                '-' => -1,
                else => std.fmt.parseInt(isize, formula.items[0..n_index], 10) catch return error.HtmlParse,
            },
            else => std.fmt.parseInt(isize, formula.items[0..n_index], 10) catch return error.HtmlParse,
        };

        const offset = if (formula.items[n_index + 1 ..].len == 0)
            0
        else
            std.fmt.parseInt(isize, formula.items[n_index + 1 ..], 10) catch return error.HtmlParse;

        break :blk SelectorNthChildPattern{
            .step = step,
            .offset = offset,
            .of_selectors = null,
        };
    } else blk: {
        const offset = std.fmt.parseInt(isize, formula.items, 10) catch return error.HtmlParse;
        break :blk SelectorNthChildPattern{
            .step = 0,
            .offset = offset,
            .of_selectors = null,
        };
    };

    if (split.of_selectors) |of_selectors_source| {
        pattern.of_selectors = try parseSelectorNthChildOfSelectors(allocator, of_selectors_source);
    }

    return pattern;
}

fn splitNthChildArgument(argument: []const u8) errors.Result(NthChildArgumentSplit) {
    const trimmed = std.mem.trim(u8, argument, " \t\r\n\x0c");
    if (trimmed.len == 0) return error.HtmlParse;

    var quote: ?u8 = null;
    var bracket_depth: usize = 0;
    var paren_depth: usize = 0;
    var index: usize = 0;
    while (index < trimmed.len) : (index += 1) {
        const byte = trimmed[index];
        if (quote) |current_quote| {
            if (byte == current_quote) {
                quote = null;
            }
            continue;
        }

        switch (byte) {
            '"', '\'' => quote = byte,
            '[' => bracket_depth += 1,
            ']' => {
                if (bracket_depth == 0) return error.HtmlParse;
                bracket_depth -= 1;
            },
            '(' => paren_depth += 1,
            ')' => {
                if (paren_depth == 0) return error.HtmlParse;
                paren_depth -= 1;
            },
            else => {
                if (paren_depth != 0 or bracket_depth != 0) continue;
                if (!isHtmlWhitespace(byte)) continue;

                const formula_end = index;
                var lookahead = index;
                while (lookahead < trimmed.len and isHtmlWhitespace(trimmed[lookahead])) : (lookahead += 1) {}
                if (!isOfKeyword(trimmed, lookahead)) continue;

                const of_start = lookahead + 2;
                if (of_start >= trimmed.len) return error.HtmlParse;
                return .{
                    .formula = std.mem.trim(u8, trimmed[0..formula_end], " \t\r\n\x0c"),
                    .of_selectors = std.mem.trim(u8, trimmed[of_start..], " \t\r\n\x0c"),
                };
            },
        }
    }

    if (quote != null or paren_depth != 0 or bracket_depth != 0) return error.HtmlParse;

    return .{
        .formula = trimmed,
        .of_selectors = null,
    };
}

fn isOfKeyword(bytes: []const u8, pos: usize) bool {
    if (pos + 1 >= bytes.len) return false;
    if (!asciiEqualIgnoreCase(bytes[pos .. pos + 2], "of")) return false;
    if (pos + 2 >= bytes.len) return true;
    return isHtmlWhitespace(bytes[pos + 2]);
}

fn parseSelectorNthChildOfSelectors(
    allocator: std.mem.Allocator,
    selector: []const u8,
) errors.Result(std.ArrayListUnmanaged(SelectorChain)) {
    return parseSelectorChainListArgument(allocator, selector);
}

fn parseSelectorChainListArgument(
    allocator: std.mem.Allocator,
    selector: []const u8,
) errors.Result(std.ArrayListUnmanaged(SelectorChain)) {
    var chains: std.ArrayList(SelectorChain) = .empty;
    errdefer {
        for (chains.items) |*chain| {
            chain.deinit(allocator);
        }
        chains.deinit(allocator);
    }

    try appendSelectorChains(allocator, selector, &chains);
    if (chains.items.len == 0) return error.HtmlParse;
    const owned = try chains.toOwnedSlice(allocator);
    return .{
        .items = owned,
        .capacity = owned.len,
    };
}

fn parseSelectorParenthesizedArgument(
    selector: []const u8,
    pos: *usize,
) errors.Result([]const u8) {
    if (pos.* >= selector.len or selector[pos.*] != '(') return error.HtmlParse;
    pos.* += 1;

    const start = pos.*;
    var depth: usize = 1;
    var bracket_depth: usize = 0;
    var quote: ?u8 = null;

    while (pos.* < selector.len) {
        const byte = selector[pos.*];
        if (quote) |current_quote| {
            if (byte == current_quote) {
                quote = null;
            }
            pos.* += 1;
            continue;
        }

        switch (byte) {
            '"', '\'' => {
                quote = byte;
                pos.* += 1;
            },
            '[' => {
                bracket_depth += 1;
                pos.* += 1;
            },
            ']' => {
                if (bracket_depth == 0) return error.HtmlParse;
                bracket_depth -= 1;
                pos.* += 1;
            },
            '(' => {
                if (bracket_depth == 0) {
                    depth += 1;
                }
                pos.* += 1;
            },
            ')' => {
                if (bracket_depth == 0) {
                    depth -= 1;
                    if (depth == 0) {
                        const argument = selector[start..pos.*];
                        pos.* += 1;
                        return argument;
                    }
                }
                pos.* += 1;
            },
            else => {
                pos.* += 1;
            },
        }
    }

    return error.HtmlParse;
}

fn parseSelectorRelativeSelectors(
    allocator: std.mem.Allocator,
    selector: []const u8,
) errors.Result(std.ArrayListUnmanaged(SelectorRelativeSelector)) {
    var relative_selectors: std.ArrayListUnmanaged(SelectorRelativeSelector) = .{};
    errdefer {
        for (relative_selectors.items) |*relative_selector| {
            relative_selector.deinit(allocator);
        }
        relative_selectors.deinit(allocator);
    }

    const trimmed = std.mem.trim(u8, selector, " \t\r\n");
    if (trimmed.len == 0) return error.HtmlParse;

    var start: usize = 0;
    var paren_depth: usize = 0;
    var bracket_depth: usize = 0;
    var quote: ?u8 = null;
    var index: usize = 0;
    while (index < trimmed.len) : (index += 1) {
        const byte = trimmed[index];
        if (quote) |current_quote| {
            if (byte == current_quote) {
                quote = null;
            }
            continue;
        }

        switch (byte) {
            '"', '\'' => quote = byte,
            '[' => bracket_depth += 1,
            ']' => {
                if (bracket_depth == 0) return error.HtmlParse;
                bracket_depth -= 1;
            },
            '(' => paren_depth += 1,
            ')' => {
                if (paren_depth == 0) return error.HtmlParse;
                paren_depth -= 1;
            },
            ',' => {
                if (paren_depth != 0 or bracket_depth != 0) continue;
                const item = std.mem.trim(u8, trimmed[start..index], " \t\r\n");
                if (item.len == 0) return error.HtmlParse;

                var relative_selector = try parseSelectorRelativeSelectorItem(allocator, item);
                errdefer relative_selector.deinit(allocator);
                try relative_selectors.append(allocator, relative_selector);
                start = index + 1;
            },
            else => {},
        }
    }

    if (quote != null or paren_depth != 0 or bracket_depth != 0) return error.HtmlParse;

    const item = std.mem.trim(u8, trimmed[start..], " \t\r\n");
    if (item.len == 0) return error.HtmlParse;

    var relative_selector = try parseSelectorRelativeSelectorItem(allocator, item);
    errdefer relative_selector.deinit(allocator);
    try relative_selectors.append(allocator, relative_selector);

    return relative_selectors;
}

fn parseSelectorRelativeSelectorItem(
    allocator: std.mem.Allocator,
    selector: []const u8,
) errors.Result(SelectorRelativeSelector) {
    const trimmed = std.mem.trim(u8, selector, " \t\r\n");
    if (trimmed.len == 0) return error.HtmlParse;

    var pos: usize = 0;
    const had_whitespace = skipSelectorWhitespace(trimmed, &pos);
    _ = had_whitespace;

    var combinator: ?SelectorCombinator = null;
    if (pos < trimmed.len) {
        switch (trimmed[pos]) {
            '>' => {
                combinator = .child;
                pos += 1;
            },
            '+' => {
                combinator = .adjacent_sibling;
                pos += 1;
            },
            '~' => {
                combinator = .general_sibling;
                pos += 1;
            },
            ',' => return error.HtmlParse,
            else => {},
        }
    }

    if (combinator != null) {
        _ = skipSelectorWhitespace(trimmed, &pos);
        if (pos >= trimmed.len) return error.HtmlParse;
    }

    const remainder = std.mem.trim(u8, trimmed[pos..], " \t\r\n");
    if (remainder.len == 0) return error.HtmlParse;

    var chain = try parseSelectorChain(allocator, remainder);
    errdefer chain.deinit(allocator);

    return .{
        .combinator = combinator,
        .chain = chain,
    };
}

fn skipSelectorWhitespace(selector: []const u8, pos: *usize) bool {
    const start = pos.*;
    while (pos.* < selector.len and isHtmlWhitespace(selector[pos.*])) {
        pos.* += 1;
    }

    return pos.* != start;
}

fn parentOf(self: *const DomStore, node_id: NodeId) ?NodeId {
    const node = self.nodeAt(node_id) orelse return null;
    return node.parent;
}

pub fn comparisonRoot(self: *const DomStore, node_id: NodeId) ?NodeId {
    var current = node_id;
    while (true) {
        const node = self.nodeAt(current) orelse return null;
        if (node.parent) |parent_id| {
            const parent = self.nodeAt(parent_id) orelse return null;
            if (parent.kind == .element) {
                if (self.tagNameForNode(parent_id)) |tag_name| {
                    if (std.mem.eql(u8, tag_name, "template")) {
                        return parent_id;
                    }
                }
            }
            current = parent_id;
        } else {
            return current;
        }
    }
}

fn comparisonDepth(self: *const DomStore, node_id: NodeId, root_id: NodeId) ?usize {
    var depth: usize = 0;
    var current = node_id;
    while (!sameNodeId(current, root_id)) {
        current = parentOf(self, current) orelse return null;
        depth += 1;
    }
    return depth;
}

fn comparisonContains(self: *const DomStore, container_id: NodeId, target_id: NodeId) bool {
    if (sameNodeId(container_id, target_id)) return true;
    const container_root = comparisonRoot(self, container_id) orelse return false;
    const target_root = comparisonRoot(self, target_id) orelse return false;
    if (!sameNodeId(container_root, target_root)) return false;

    var current = target_id;
    while (!sameNodeId(current, container_root)) {
        current = parentOf(self, current) orelse return false;
        if (sameNodeId(current, container_id)) return true;
    }
    return false;
}

fn comparisonOrder(
    self: *const DomStore,
    left_id: NodeId,
    right_id: NodeId,
    root_id: NodeId,
) ?bool {
    var left = left_id;
    var right = right_id;
    var left_depth = comparisonDepth(self, left_id, root_id) orelse return null;
    var right_depth = comparisonDepth(self, right_id, root_id) orelse return null;
    while (left_depth > right_depth) {
        left = parentOf(self, left) orelse return null;
        left_depth -= 1;
    }
    while (right_depth > left_depth) {
        right = parentOf(self, right) orelse return null;
        right_depth -= 1;
    }

    while (true) {
        const left_parent = parentOf(self, left) orelse return null;
        const right_parent = parentOf(self, right) orelse return null;
        if (sameNodeId(left_parent, right_parent)) break;
        left = left_parent;
        right = right_parent;
    }

    const parent_id = parentOf(self, left) orelse return null;
    const left_index = self.childIndex(parent_id, left) catch return null;
    const right_index = self.childIndex(parent_id, right) catch return null;
    return left_index < right_index;
}

pub fn firstChild(self: *const DomStore, node_id: NodeId) ?NodeId {
    const node = self.nodeAt(node_id) orelse return null;
    return if (node.children.items.len > 0) node.children.items[0] else null;
}

pub fn lastChild(self: *const DomStore, node_id: NodeId) ?NodeId {
    const node = self.nodeAt(node_id) orelse return null;
    if (node.children.items.len == 0) return null;
    return node.children.items[node.children.items.len - 1];
}

pub fn hasChildNodes(self: *const DomStore, node_id: NodeId) bool {
    const node = self.nodeAt(node_id) orelse return false;
    return node.children.items.len != 0;
}

pub fn isConnected(self: *const DomStore, node_id: NodeId) bool {
    var current = node_id;
    while (true) {
        if (sameNodeId(current, self.documentId())) {
            return true;
        }
        current = parentOf(self, current) orelse return false;
    }
}

fn documentElementId(self: *const DomStore) ?NodeId {
    const document = self.nodeAt(self.documentId()) orelse return null;
    for (document.children.items) |child_id| {
        if (self.tagNameForNode(child_id) != null) {
            return child_id;
        }
    }
    return null;
}

fn selectorScopeRootForSearch(self: *const DomStore, root_id: NodeId) ?NodeId {
    if (sameNodeId(root_id, self.documentId())) {
        return documentElementId(self);
    }
    return root_id;
}

pub fn previousElementSibling(self: *const DomStore, node_id: NodeId) ?NodeId {
    const parent_id = parentOf(self, node_id) orelse return null;
    const parent = self.nodeAt(parent_id) orelse return null;
    var index = self.childIndex(parent_id, node_id) catch return null;
    while (index > 0) {
        index -= 1;
        const sibling_id = parent.children.items[index];
        if (self.tagNameForNode(sibling_id) != null) {
            return sibling_id;
        }
    }
    return null;
}

pub fn compareDocumentPosition(self: *const DomStore, node_id: NodeId, other_id: NodeId) u16 {
    const DISCONNECTED: u16 = 0x01;
    const PRECEDING: u16 = 0x02;
    const FOLLOWING: u16 = 0x04;
    const CONTAINS: u16 = 0x08;
    const CONTAINED_BY: u16 = 0x10;
    const IMPLEMENTATION_SPECIFIC: u16 = 0x20;

    if (sameNodeId(node_id, other_id)) return 0;

    const node_root = comparisonRoot(self, node_id) orelse return DISCONNECTED | IMPLEMENTATION_SPECIFIC | FOLLOWING;
    const other_root = comparisonRoot(self, other_id) orelse return DISCONNECTED | IMPLEMENTATION_SPECIFIC | PRECEDING;
    if (!sameNodeId(node_root, other_root)) {
        return if (node_root.index < other_root.index)
            DISCONNECTED | IMPLEMENTATION_SPECIFIC | FOLLOWING
        else
            DISCONNECTED | IMPLEMENTATION_SPECIFIC | PRECEDING;
    }

    if (comparisonContains(self, node_id, other_id)) {
        return CONTAINED_BY | FOLLOWING;
    }

    if (comparisonContains(self, other_id, node_id)) {
        return CONTAINS | PRECEDING;
    }

    return if (comparisonOrder(self, node_id, other_id, node_root) orelse false) FOLLOWING else PRECEDING;
}

pub fn nodeIsEqualNode(self: *const DomStore, node_id: NodeId, other_id: NodeId) bool {
    const node = self.nodeAt(node_id) orelse return false;
    const other = self.nodeAt(other_id) orelse return false;

    return switch (node.kind) {
        .document => switch (other.kind) {
            .document => nodeChildrenEqual(self, node.children.items, other.children.items),
            else => false,
        },
        .element => |left| switch (other.kind) {
            .element => |right| elementDataEqual(left, right) and nodeChildrenEqual(self, node.children.items, other.children.items),
            else => false,
        },
        .text => |left| switch (other.kind) {
            .text => |right| std.mem.eql(u8, left, right),
            else => false,
        },
        .comment => |left| switch (other.kind) {
            .comment => |right| std.mem.eql(u8, left, right),
            else => false,
        },
    };
}

pub fn templateContentIsEqualNode(self: *const DomStore, left: NodeId, right: NodeId) bool {
    const left_record = self.nodeAt(left) orelse return false;
    const right_record = self.nodeAt(right) orelse return false;

    return switch (left_record.kind) {
        .element => switch (right_record.kind) {
            .element => nodeChildrenEqual(self, left_record.children.items, right_record.children.items),
            else => false,
        },
        else => false,
    };
}

pub fn nextElementSibling(self: *const DomStore, node_id: NodeId) ?NodeId {
    const parent_id = parentOf(self, node_id) orelse return null;
    const parent = self.nodeAt(parent_id) orelse return null;
    const start_index = self.childIndex(parent_id, node_id) catch return null;
    var index = start_index + 1;
    while (index < parent.children.items.len) : (index += 1) {
        const sibling_id = parent.children.items[index];
        if (self.tagNameForNode(sibling_id) != null) {
            return sibling_id;
        }
    }
    return null;
}

pub fn previousSibling(self: *const DomStore, node_id: NodeId) ?NodeId {
    const parent_id = parentOf(self, node_id) orelse return null;
    const parent = self.nodeAt(parent_id) orelse return null;
    const index = self.childIndex(parent_id, node_id) catch return null;
    if (index == 0) return null;
    return parent.children.items[index - 1];
}

pub fn nextSibling(self: *const DomStore, node_id: NodeId) ?NodeId {
    const parent_id = parentOf(self, node_id) orelse return null;
    const parent = self.nodeAt(parent_id) orelse return null;
    const index = self.childIndex(parent_id, node_id) catch return null;
    const next_index = index + 1;
    if (next_index >= parent.children.items.len) return null;
    return parent.children.items[next_index];
}

pub fn firstElementChild(self: *const DomStore, parent_id: NodeId) ?NodeId {
    const parent = self.nodeAt(parent_id) orelse return null;
    for (parent.children.items) |child_id| {
        if (self.tagNameForNode(child_id) != null) {
            return child_id;
        }
    }
    return null;
}

pub fn lastElementChild(self: *const DomStore, parent_id: NodeId) ?NodeId {
    const parent = self.nodeAt(parent_id) orelse return null;
    var index = parent.children.items.len;
    while (index > 0) {
        index -= 1;
        const child_id = parent.children.items[index];
        if (self.tagNameForNode(child_id) != null) {
            return child_id;
        }
    }
    return null;
}

pub fn childElementCount(self: *const DomStore, parent_id: NodeId) usize {
    const parent = self.nodeAt(parent_id) orelse return 0;
    var count: usize = 0;
    for (parent.children.items) |child_id| {
        if (self.tagNameForNode(child_id) != null) {
            count += 1;
        }
    }
    return count;
}

fn isOnlyElementChild(self: *const DomStore, node_id: NodeId) bool {
    const parent_id = parentOf(self, node_id) orelse return false;
    return sameNodeId(firstElementChild(self, parent_id) orelse return false, node_id) and sameNodeId(lastElementChild(self, parent_id) orelse return false, node_id);
}

fn isFirstElementChild(self: *const DomStore, node_id: NodeId) bool {
    const parent_id = parentOf(self, node_id) orelse return false;
    return sameNodeId(firstElementChild(self, parent_id) orelse return false, node_id);
}

fn isLastElementChild(self: *const DomStore, node_id: NodeId) bool {
    const parent_id = parentOf(self, node_id) orelse return false;
    return sameNodeId(lastElementChild(self, parent_id) orelse return false, node_id);
}

fn isFirstElementOfType(self: *const DomStore, node_id: NodeId) bool {
    const parent_id = parentOf(self, node_id) orelse return false;
    const node_tag = self.tagNameForNode(node_id) orelse return false;
    const parent = self.nodeAt(parent_id) orelse return false;
    for (parent.children.items) |child_id| {
        if (!sameNodeId(child_id, node_id) and self.tagNameForNode(child_id) != null and std.mem.eql(u8, self.tagNameForNode(child_id).?, node_tag)) {
            return false;
        }
        if (sameNodeId(child_id, node_id)) {
            return true;
        }
    }
    return false;
}

fn isLastElementOfType(self: *const DomStore, node_id: NodeId) bool {
    const parent_id = parentOf(self, node_id) orelse return false;
    const node_tag = self.tagNameForNode(node_id) orelse return false;
    const parent = self.nodeAt(parent_id) orelse return false;
    var seen_self = false;
    var index = parent.children.items.len;
    while (index > 0) {
        index -= 1;
        const child_id = parent.children.items[index];
        if (!sameNodeId(child_id, node_id) and self.tagNameForNode(child_id) != null and std.mem.eql(u8, self.tagNameForNode(child_id).?, node_tag)) {
            return false;
        }
        if (sameNodeId(child_id, node_id)) {
            seen_self = true;
            break;
        }
    }
    return seen_self;
}

fn isOnlyElementOfType(self: *const DomStore, node_id: NodeId) bool {
    const node_tag = self.tagNameForNode(node_id) orelse return false;
    const parent_id = parentOf(self, node_id) orelse return false;
    const parent = self.nodeAt(parent_id) orelse return false;
    var seen_self = false;
    for (parent.children.items) |child_id| {
        const child_tag = self.tagNameForNode(child_id) orelse continue;
        if (!std.mem.eql(u8, child_tag, node_tag)) continue;
        if (sameNodeId(child_id, node_id)) {
            seen_self = true;
            continue;
        }
        return false;
    }
    return seen_self;
}

fn isEmptyElement(self: *const DomStore, node_id: NodeId) bool {
    const node = self.nodeAt(node_id) orelse return false;
    switch (node.kind) {
        .element => {},
        else => return false,
    }

    for (node.children.items) |child_id| {
        const child = self.nodeAt(child_id) orelse return false;
        switch (child.kind) {
            .comment => {},
            else => return false,
        }
    }
    return true;
}

fn isLinkPseudo(self: *const DomStore, node_id: NodeId) bool {
    const node = self.nodeAt(node_id) orelse return false;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return false,
    };
    if (std.mem.eql(u8, element.tag_name, "a") or std.mem.eql(u8, element.tag_name, "area")) {
        return elementAttributeValue(element, "href") != null;
    }
    return false;
}

fn isDefinedPseudoClass(self: *const DomStore, node_id: NodeId) bool {
    const node = self.nodeAt(node_id) orelse return false;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return false,
    };

    if (self.serializationNamespaceForNode(node_id) != .html) return true;
    return std.mem.indexOfScalar(u8, element.tag_name, '-') == null;
}

fn isCheckedPseudo(self: *const DomStore, node_id: NodeId) bool {
    if (self.checkedForNode(node_id)) |checked| {
        return checked;
    }
    return self.optionSelectedForNode(node_id) == true;
}

fn isDisabledPseudo(self: *const DomStore, node_id: NodeId) bool {
    const node = self.nodeAt(node_id) orelse return false;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return false,
    };

    if (!isFormStatePseudoCandidate(element.tag_name)) return false;
    return elementAttributeValue(element, "disabled") != null;
}

fn isEnabledPseudo(self: *const DomStore, node_id: NodeId) bool {
    const node = self.nodeAt(node_id) orelse return false;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return false,
    };

    if (!isFormStatePseudoCandidate(element.tag_name)) return false;
    return elementAttributeValue(element, "disabled") == null;
}

fn isRequiredPseudo(self: *const DomStore, node_id: NodeId) bool {
    const node = self.nodeAt(node_id) orelse return false;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return false,
    };

    if (!isRequiredPseudoCandidate(element.tag_name)) return false;
    return elementAttributeValue(element, "required") != null;
}

fn isOptionalPseudo(self: *const DomStore, node_id: NodeId) bool {
    const node = self.nodeAt(node_id) orelse return false;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return false,
    };

    if (!isRequiredPseudoCandidate(element.tag_name)) return false;
    return elementAttributeValue(element, "required") == null;
}

fn isPlaceholderShownPseudo(self: *const DomStore, node_id: NodeId) bool {
    const node = self.nodeAt(node_id) orelse return false;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return false,
    };

    if (!(std.mem.eql(u8, element.tag_name, "input") or std.mem.eql(u8, element.tag_name, "textarea"))) {
        return false;
    }

    if (elementAttributeValue(element, "placeholder") == null) {
        return false;
    }

    if (std.mem.eql(u8, element.tag_name, "input")) {
        const value = elementAttributeValue(element, "value") orelse "";
        return value.len == 0;
    }

    return isEmptyElement(self, node_id);
}

fn isBlankPseudoClass(self: *const DomStore, node_id: NodeId) bool {
    const node = self.nodeAt(node_id) orelse return false;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return false,
    };

    if (isContentEditableElement(element)) {
        const value = self.textContent(self.allocator, node_id) catch return false;
        defer self.allocator.free(value);
        return std.mem.trim(u8, value, " \t\r\n\x0c").len == 0;
    }

    if (std.mem.eql(u8, element.tag_name, "textarea")) {
        const value = self.textContent(self.allocator, node_id) catch return false;
        defer self.allocator.free(value);
        return std.mem.trim(u8, value, " \t\r\n\x0c").len == 0;
    }

    if (!std.mem.eql(u8, element.tag_name, "input")) {
        return false;
    }

    const input_type = elementAttributeValue(element, "type");
    if (!isTextInputType(input_type)) {
        return false;
    }

    const value = elementAttributeValue(element, "value") orelse "";
    return std.mem.trim(u8, value, " \t\r\n\x0c").len == 0;
}

fn isIndeterminatePseudoClass(self: *const DomStore, node_id: NodeId) bool {
    const node = self.nodeAt(node_id) orelse return false;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return false,
    };

    if (std.mem.eql(u8, element.tag_name, "progress")) {
        return elementAttributeValue(element, "value") == null;
    }

    if (!std.mem.eql(u8, element.tag_name, "input")) {
        return false;
    }

    if (self.indeterminate.contains(node_id)) {
        return true;
    }

    if (!isCheckableInputType(elementAttributeValue(element, "type"))) {
        return false;
    }

    const name = elementAttributeValue(element, "name") orelse return false;
    const scope_root = radioGroupScopeRoot(self, node_id);
    var descendants: std.ArrayList(NodeId) = .empty;
    defer descendants.deinit(self.allocator);
    self.collectSubtreeNodes(scope_root, &descendants, self.allocator) catch return false;

    for (descendants.items) |descendant_id| {
        const descendant = self.nodeAt(descendant_id) orelse continue;
        const descendant_element = switch (descendant.kind) {
            .element => |descendant_element| descendant_element,
            else => continue,
        };

        if (!std.mem.eql(u8, descendant_element.tag_name, "input")) continue;
        if (!isCheckableInputType(elementAttributeValue(descendant_element, "type"))) continue;
        if (!std.mem.eql(u8, elementAttributeValue(descendant_element, "name") orelse "", name)) continue;
        if (self.checkedForNode(descendant_id) == true) return false;
    }

    return true;
}

fn isDefaultPseudoClass(self: *const DomStore, node_id: NodeId) bool {
    const node = self.nodeAt(node_id) orelse return false;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return false,
    };

    if (std.mem.eql(u8, element.tag_name, "option")) {
        return self.optionSelectedForNode(node_id) == true;
    }

    if (std.mem.eql(u8, element.tag_name, "input")) {
        const input_type = elementAttributeValue(element, "type");
        if (isCheckableInputType(input_type)) {
            return self.checkedForNode(node_id) == true;
        }

        if (std.mem.eql(u8, input_type orelse "text", "submit") or std.mem.eql(u8, input_type orelse "text", "image")) {
            return isDefaultSubmitButton(self, node_id);
        }
        return false;
    }

    if (std.mem.eql(u8, element.tag_name, "button")) {
        return isDefaultSubmitButton(self, node_id);
    }

    return false;
}

fn isValidPseudoClass(self: *const DomStore, node_id: NodeId) bool {
    if (!isValidityFormControlCandidate(self, node_id)) return false;
    return !isInvalidPseudoClass(self, node_id);
}

pub fn formControlCheckValidity(self: *const DomStore, node_id: NodeId) bool {
    return !isInvalidPseudoClass(self, node_id);
}

pub fn willValidateForNode(self: *const DomStore, node_id: NodeId) bool {
    return isValidityFormControlCandidate(self, node_id) and !isBarredFromConstraintValidation(self, node_id);
}

pub fn validityStateForNode(self: *const DomStore, allocator: std.mem.Allocator, node_id: NodeId) errors.Result(ValidityStateInfo) {
    const default_state: ValidityStateInfo = .{
        .valid = true,
        .value_missing = false,
        .type_mismatch = false,
        .pattern_mismatch = false,
        .too_long = false,
        .too_short = false,
        .range_underflow = false,
        .range_overflow = false,
        .step_mismatch = false,
        .bad_input = false,
        .custom_error = self.custom_validity.contains(node_id),
    };

    const node = self.nodeAt(node_id) orelse return default_state;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return default_state,
    };

    if (isBarredFromConstraintValidation(self, node_id)) {
        return default_state;
    }

    var state = default_state;

    if (std.mem.eql(u8, element.tag_name, "textarea")) {
        const value = self.textContent(allocator, node_id) catch |err| switch (err) {
            error.OutOfMemory => return error.OutOfMemory,
            else => return default_state,
        };
        defer allocator.free(value);

        state.value_missing = elementAttributeValue(element, "required") != null and value.len == 0;
        if (textLengthLimits(self, node_id)) |limits| {
            if (limits.min) |min| {
                state.too_short = value.len > 0 and value.len < min;
            }
            if (limits.max) |max| {
                state.too_long = value.len > max;
            }
        }
        state.valid = !(state.custom_error or state.value_missing or state.type_mismatch or state.pattern_mismatch or state.too_long or state.too_short or state.range_underflow or state.range_overflow or state.step_mismatch or state.bad_input);
        return state;
    }

    if (std.mem.eql(u8, element.tag_name, "select")) {
        const value = self.selectValueForNode(allocator, node_id) catch |err| switch (err) {
            error.OutOfMemory => return error.OutOfMemory,
            else => return default_state,
        };
        defer allocator.free(value);

        state.value_missing = elementAttributeValue(element, "required") != null and value.len == 0;
        state.valid = !(state.custom_error or state.value_missing or state.type_mismatch or state.pattern_mismatch or state.too_long or state.too_short or state.range_underflow or state.range_overflow or state.step_mismatch or state.bad_input);
        return state;
    }

    if (std.mem.eql(u8, element.tag_name, "object")) {
        state.valid = !state.custom_error;
        return state;
    }

    if (!std.mem.eql(u8, element.tag_name, "input")) {
        return default_state;
    }

    const input_type = elementAttributeValue(element, "type");
    if (std.mem.eql(u8, input_type orelse "text", "hidden")) {
        return default_state;
    }

    if (isCheckableInputType(input_type)) {
        state.value_missing = elementAttributeValue(element, "required") != null and self.checkedForNode(node_id) != true;
        state.valid = !(state.custom_error or state.value_missing or state.type_mismatch or state.pattern_mismatch or state.too_long or state.too_short or state.range_underflow or state.range_overflow or state.step_mismatch or state.bad_input);
        return state;
    }

    if (isRangeInputType(input_type)) {
        const value = self.inputValueForNode(allocator, node_id) catch |err| switch (err) {
            error.OutOfMemory => return error.OutOfMemory,
            else => return default_state,
        };
        defer allocator.free(value);

        const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
        if (elementAttributeValue(element, "required") != null and trimmed.len == 0) {
            state.value_missing = true;
        } else if (trimmed.len > 0) {
            const parsed = std.fmt.parseFloat(f64, trimmed) catch null;
            if (parsed == null) {
                state.bad_input = true;
            } else {
                const current_value = parsed.?;
                if (numericRangeLimits(self, node_id)) |bounds| {
                    if (bounds.min) |min| {
                        state.range_underflow = current_value < min;
                    }
                    if (bounds.max) |max| {
                        state.range_overflow = current_value > max;
                    }
                }
                if (numericStepMismatch(self, node_id, current_value)) {
                    state.step_mismatch = true;
                }
            }
        }
        state.valid = !(state.custom_error or state.value_missing or state.type_mismatch or state.pattern_mismatch or state.too_long or state.too_short or state.range_underflow or state.range_overflow or state.step_mismatch or state.bad_input);
        return state;
    }

    if (isFileInputType(input_type) or isTextInputType(input_type)) {
        const value = self.inputValueForNode(allocator, node_id) catch |err| switch (err) {
            error.OutOfMemory => return error.OutOfMemory,
            else => return default_state,
        };
        defer allocator.free(value);

        if (elementAttributeValue(element, "required") != null and value.len == 0) {
            state.value_missing = true;
        }
        if (inputTypeMismatch(self, allocator, node_id, input_type, value)) {
            state.type_mismatch = true;
        }
        if (try inputPatternMismatch(self, allocator, node_id, input_type, value)) {
            state.pattern_mismatch = true;
        }
        if (isLengthInputType(input_type)) {
            if (textLengthLimits(self, node_id)) |limits| {
                if (limits.min) |min| {
                    state.too_short = value.len > 0 and value.len < min;
                }
                if (limits.max) |max| {
                    state.too_long = value.len > max;
                }
            }
        }
        state.valid = !(state.custom_error or state.value_missing or state.type_mismatch or state.pattern_mismatch or state.too_long or state.too_short or state.range_underflow or state.range_overflow or state.step_mismatch or state.bad_input);
        return state;
    }

    return default_state;
}

pub fn formCheckValidity(self: *const DomStore, form_id: NodeId) bool {
    var descendants: std.ArrayList(NodeId) = .empty;
    defer descendants.deinit(self.allocator);
    self.collectSubtreeNodes(form_id, &descendants, self.allocator) catch return false;

    for (descendants.items) |candidate_id| {
        if (isInvalidPseudoClass(self, candidate_id)) return false;
    }
    return true;
}

fn isInvalidPseudoClass(self: *const DomStore, node_id: NodeId) bool {
    const node = self.nodeAt(node_id) orelse return false;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return false,
    };

    if (isBarredFromConstraintValidation(self, node_id)) {
        return false;
    }

    if (self.custom_validity.contains(node_id)) {
        return true;
    }

    if (std.mem.eql(u8, element.tag_name, "textarea")) {
        const value = self.textContent(self.allocator, node_id) catch return false;
        defer self.allocator.free(value);
        if (elementAttributeValue(element, "required") != null and value.len == 0) {
            return true;
        }
        if (textLengthLimits(self, node_id)) |limits| {
            if (limits.min) |min| {
                if (value.len > 0 and value.len < min) return true;
            }
            if (limits.max) |max| {
                if (value.len > max) return true;
            }
        }
        return false;
    }

    if (std.mem.eql(u8, element.tag_name, "select")) {
        const value = self.selectValueForNode(self.allocator, node_id) catch return false;
        defer self.allocator.free(value);
        return elementAttributeValue(element, "required") != null and value.len == 0;
    }

    if (!std.mem.eql(u8, element.tag_name, "input")) {
        return false;
    }

    const input_type = elementAttributeValue(element, "type");
    if (std.mem.eql(u8, input_type orelse "text", "hidden")) {
        return false;
    }

    if (isCheckableInputType(input_type)) {
        return elementAttributeValue(element, "required") != null and self.checkedForNode(node_id) != true;
    }

    if (isRangeInputType(input_type)) {
        const value = self.inputValueForNode(self.allocator, node_id) catch return false;
        defer self.allocator.free(value);
        const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
        if (trimmed.len == 0) {
            return elementAttributeValue(element, "required") != null;
        }
        if (numericRangeValue(self, node_id) == null) {
            return true;
        }
        const parsed = std.fmt.parseFloat(f64, trimmed) catch return true;
        if (numericStepMismatch(self, node_id, parsed)) {
            return true;
        }
        return isOutOfRangePseudoClass(self, node_id);
    }

    if (isFileInputType(input_type) or isTextInputType(input_type)) {
        const value = self.inputValueForNode(self.allocator, node_id) catch return false;
        defer self.allocator.free(value);
        if (elementAttributeValue(element, "required") != null and value.len == 0) {
            return true;
        }
        if (inputTypeMismatch(self, self.allocator, node_id, input_type, value)) {
            return true;
        }
        if (inputPatternMismatch(self, self.allocator, node_id, input_type, value) catch return false) {
            return true;
        }
        if (isLengthInputType(input_type)) {
            if (textLengthLimits(self, node_id)) |limits| {
                if (limits.min) |min| {
                    if (value.len > 0 and value.len < min) return true;
                }
                if (limits.max) |max| {
                    if (value.len > max) return true;
                }
            }
        }
        return false;
    }

    return false;
}

fn isInRangePseudoClass(self: *const DomStore, node_id: NodeId) bool {
    const current_value = numericRangeValue(self, node_id) orelse return false;
    const bounds = numericRangeLimits(self, node_id) orelse return false;

    if (bounds.min) |min| {
        if (current_value < min) return false;
    }
    if (bounds.max) |max| {
        if (current_value > max) return false;
    }

    return true;
}

fn isOutOfRangePseudoClass(self: *const DomStore, node_id: NodeId) bool {
    const current_value = numericRangeValue(self, node_id) orelse return false;
    const bounds = numericRangeLimits(self, node_id) orelse return false;

    if (bounds.min) |min| {
        if (current_value < min) return true;
    }
    if (bounds.max) |max| {
        if (current_value > max) return true;
    }

    return false;
}

fn isReadOnlyPseudoClass(self: *const DomStore, node_id: NodeId) bool {
    return !isReadWritePseudoClass(self, node_id);
}

fn isReadWritePseudoClass(self: *const DomStore, node_id: NodeId) bool {
    const node = self.nodeAt(node_id) orelse return false;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return false,
    };

    if (isContentEditableElement(element)) {
        return true;
    }

    if (std.mem.eql(u8, element.tag_name, "textarea")) {
        return elementAttributeValue(element, "disabled") == null and elementAttributeValue(element, "readonly") == null;
    }

    if (std.mem.eql(u8, element.tag_name, "input")) {
        const input_type = elementAttributeValue(element, "type");
        return isTextInputType(input_type) and elementAttributeValue(element, "disabled") == null and elementAttributeValue(element, "readonly") == null;
    }

    return false;
}

fn isValidityFormControlCandidate(self: *const DomStore, node_id: NodeId) bool {
    const node = self.nodeAt(node_id) orelse return false;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return false,
    };

    if (std.mem.eql(u8, element.tag_name, "textarea") or
        std.mem.eql(u8, element.tag_name, "select") or
        std.mem.eql(u8, element.tag_name, "output") or
        std.mem.eql(u8, element.tag_name, "object"))
    {
        return true;
    }

    if (!std.mem.eql(u8, element.tag_name, "input")) {
        return false;
    }

    const input_type = elementAttributeValue(element, "type");
    return !std.mem.eql(u8, input_type orelse "text", "hidden") and (isTextInputType(input_type) or isCheckableInputType(input_type) or isFileInputType(input_type));
}

fn isBarredFromConstraintValidation(self: *const DomStore, node_id: NodeId) bool {
    const node = self.nodeAt(node_id) orelse return true;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return true,
    };

    if (elementAttributeValue(element, "disabled") != null) {
        return true;
    }

    if (std.mem.eql(u8, element.tag_name, "textarea")) {
        return elementAttributeValue(element, "readonly") != null;
    }

    if (!std.mem.eql(u8, element.tag_name, "input")) {
        return false;
    }

    const input_type = elementAttributeValue(element, "type");
    if (std.mem.eql(u8, input_type orelse "text", "hidden")) {
        return true;
    }

    return elementAttributeValue(element, "readonly") != null and isTextInputType(input_type);
}

fn isDefaultSubmitButton(self: *const DomStore, node_id: NodeId) bool {
    const form_id = formAncestorOf(self, node_id) orelse return false;
    var descendants: std.ArrayList(NodeId) = .empty;
    defer descendants.deinit(self.allocator);
    self.collectSubtreeNodes(form_id, &descendants, self.allocator) catch return false;

    for (descendants.items) |candidate_id| {
        if (isSubmitButtonCandidate(self, candidate_id)) {
            return sameNodeId(candidate_id, node_id);
        }
    }

    return false;
}

fn isSubmitButtonCandidate(self: *const DomStore, node_id: NodeId) bool {
    const node = self.nodeAt(node_id) orelse return false;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return false,
    };

    if (elementAttributeValue(element, "disabled") != null) {
        return false;
    }

    if (std.mem.eql(u8, element.tag_name, "button")) {
        return !std.mem.eql(u8, elementAttributeValue(element, "type") orelse "submit", "button");
    }

    if (std.mem.eql(u8, element.tag_name, "input")) {
        const input_type = elementAttributeValue(element, "type") orelse "text";
        return std.mem.eql(u8, input_type, "submit") or std.mem.eql(u8, input_type, "image");
    }

    return false;
}

fn formAncestorOf(self: *const DomStore, node_id: NodeId) ?NodeId {
    var current = parentOf(self, node_id);
    while (current) |ancestor_id| {
        const node = self.nodeAt(ancestor_id) orelse break;
        if (switch (node.kind) {
            .element => |element| std.mem.eql(u8, element.tag_name, "form"),
            else => false,
        }) {
            return ancestor_id;
        }
        current = parentOf(self, ancestor_id);
    }

    return null;
}

fn radioGroupScopeRoot(self: *const DomStore, node_id: NodeId) NodeId {
    var current = parentOf(self, node_id);
    while (current) |ancestor_id| {
        const node = self.nodeAt(ancestor_id) orelse break;
        if (switch (node.kind) {
            .element => |element| std.mem.eql(u8, element.tag_name, "form"),
            else => false,
        }) {
            return ancestor_id;
        }
        current = parentOf(self, ancestor_id);
    }

    return documentElementId(self) orelse self.documentId();
}

fn isContentEditableElement(element: ElementData) bool {
    const value = elementAttributeValue(element, "contenteditable") orelse return false;
    const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
    return trimmed.len == 0 or asciiEqualIgnoreCase(trimmed, "true") or asciiEqualIgnoreCase(trimmed, "plaintext-only");
}

const NumericRangeLimits = struct {
    min: ?f64,
    max: ?f64,
};

const TextLengthLimits = struct {
    min: ?usize,
    max: ?usize,
};

pub const ValidityStateInfo = struct {
    valid: bool,
    value_missing: bool,
    type_mismatch: bool,
    pattern_mismatch: bool,
    too_long: bool,
    too_short: bool,
    range_underflow: bool,
    range_overflow: bool,
    step_mismatch: bool,
    bad_input: bool,
    custom_error: bool,
};

fn isRangeInputType(input_type: ?[]const u8) bool {
    const value = input_type orelse "text";
    return std.mem.eql(u8, value, "number") or std.mem.eql(u8, value, "range");
}

fn numericRangeValue(self: *const DomStore, node_id: NodeId) ?f64 {
    const node = self.nodeAt(node_id) orelse return null;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return null,
    };

    const input_type = elementAttributeValue(element, "type");
    if (!isRangeInputType(input_type)) return null;

    const value = elementAttributeValue(element, "value") orelse return null;
    const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
    if (trimmed.len == 0) return null;
    return std.fmt.parseFloat(f64, trimmed) catch return null;
}

fn numericRangeLimits(self: *const DomStore, node_id: NodeId) ?NumericRangeLimits {
    const node = self.nodeAt(node_id) orelse return null;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return null,
    };

    const input_type = elementAttributeValue(element, "type");
    if (!isRangeInputType(input_type)) return null;

    const min = if (elementAttributeValue(element, "min")) |value| blk: {
        const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
        if (trimmed.len == 0) break :blk null;
        break :blk std.fmt.parseFloat(f64, trimmed) catch null;
    } else null;

    const max = if (elementAttributeValue(element, "max")) |value| blk: {
        const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
        if (trimmed.len == 0) break :blk null;
        break :blk std.fmt.parseFloat(f64, trimmed) catch null;
    } else null;

    if (std.mem.eql(u8, input_type orelse "text", "number") and min == null and max == null) {
        return null;
    }

    return .{
        .min = if (std.mem.eql(u8, input_type orelse "text", "range")) (min orelse 0.0) else min,
        .max = if (std.mem.eql(u8, input_type orelse "text", "range")) (max orelse 100.0) else max,
    };
}

fn numericStepMismatch(self: *const DomStore, node_id: NodeId, current_value: f64) bool {
    const step = numericStepValue(self, node_id) orelse return false;
    const base = numericStepBase(self, node_id);

    const quotient = (current_value - base) / step;
    return std.math.round(quotient) != quotient;
}

fn numericStepValue(self: *const DomStore, node_id: NodeId) ?f64 {
    const node = self.nodeAt(node_id) orelse return null;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return null,
    };

    const input_type = elementAttributeValue(element, "type");
    if (!isRangeInputType(input_type)) return null;

    const step = if (elementAttributeValue(element, "step")) |value| blk: {
        const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
        if (trimmed.len == 0) break :blk 1.0;
        if (std.ascii.eqlIgnoreCase(trimmed, "any")) return null;
        const parsed = std.fmt.parseFloat(f64, trimmed) catch return null;
        if (!std.math.isFinite(parsed) or parsed <= 0) return null;
        break :blk parsed;
    } else 1.0;

    return step;
}

fn numericStepBase(self: *const DomStore, node_id: NodeId) f64 {
    const node = self.nodeAt(node_id) orelse return 0.0;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return 0.0,
    };

    const input_type = elementAttributeValue(element, "type");
    if (!isRangeInputType(input_type)) return 0.0;

    return if (elementAttributeValue(element, "min")) |value| blk: {
        const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
        if (trimmed.len == 0) break :blk 0.0;
        break :blk std.fmt.parseFloat(f64, trimmed) catch 0.0;
    } else 0.0;
}

const InputValueAsNumberKind = enum {
    number,
    range,
    date,
    datetime_local,
    month,
    week,
    time,
};

fn inputValueAsNumberKind(input_type: ?[]const u8) ?InputValueAsNumberKind {
    const value = input_type orelse "text";
    if (std.ascii.eqlIgnoreCase(value, "number")) return .number;
    if (std.ascii.eqlIgnoreCase(value, "range")) return .range;
    if (std.ascii.eqlIgnoreCase(value, "date")) return .date;
    if (std.ascii.eqlIgnoreCase(value, "datetime-local")) return .datetime_local;
    if (std.ascii.eqlIgnoreCase(value, "month")) return .month;
    if (std.ascii.eqlIgnoreCase(value, "week")) return .week;
    if (std.ascii.eqlIgnoreCase(value, "time")) return .time;
    return null;
}

const InputValueAsDateKind = enum {
    date,
    datetime_local,
    month,
    week,
    time,
};

fn inputValueAsDateKind(input_type: ?[]const u8) ?InputValueAsDateKind {
    const value = input_type orelse "text";
    if (std.ascii.eqlIgnoreCase(value, "date")) return .date;
    if (std.ascii.eqlIgnoreCase(value, "datetime-local")) return .datetime_local;
    if (std.ascii.eqlIgnoreCase(value, "month")) return .month;
    if (std.ascii.eqlIgnoreCase(value, "week")) return .week;
    if (std.ascii.eqlIgnoreCase(value, "time")) return .time;
    return null;
}

fn inputValueAsDateFromText(kind: InputValueAsDateKind, text: []const u8) ?f64 {
    return switch (kind) {
        .date => parseDateValue(text),
        .datetime_local => parseDateTimeLocalValue(text),
        .month => parseMonthValue(text),
        .week => parseWeekValue(text),
        .time => parseTimeValue(text),
    };
}

fn inputValueAsNumberFromText(self: *const DomStore, node_id: NodeId, kind: InputValueAsNumberKind, text: []const u8) ?f64 {
    return switch (kind) {
        .number => blk: {
            const trimmed = std.mem.trim(u8, text, " \t\r\n\x0c");
            if (trimmed.len == 0) break :blk std.math.nan(f64);
            const parsed = std.fmt.parseFloat(f64, trimmed) catch break :blk std.math.nan(f64);
            break :blk if (std.math.isFinite(parsed)) parsed else std.math.nan(f64);
        },
        .range => blk: {
            const trimmed = std.mem.trim(u8, text, " \t\r\n\x0c");
            const current = if (trimmed.len == 0) null else std.fmt.parseFloat(f64, trimmed) catch null;
            break :blk normalizedRangeValue(self, node_id, current) orelse std.math.nan(f64);
        },
        .date => parseDateValue(text) orelse std.math.nan(f64),
        .datetime_local => parseDateTimeLocalValue(text) orelse std.math.nan(f64),
        .month => parseMonthValue(text) orelse std.math.nan(f64),
        .week => parseWeekValue(text) orelse std.math.nan(f64),
        .time => parseTimeValue(text) orelse std.math.nan(f64),
    };
}

fn inputStepCurrentValue(self: *const DomStore, node_id: NodeId, kind: InputValueAsNumberKind) errors.Result(f64) {
    const node = self.nodeAt(node_id) orelse return error.DomError;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return error.DomError,
    };

    if (elementAttributeValue(element, "min")) |value| {
        const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
        if (trimmed.len != 0) {
            if (inputValueAsNumberFromText(self, node_id, kind, trimmed)) |parsed| {
                if (std.math.isFinite(parsed)) return parsed;
            }
        }
    }

    if (self.inputValueAsNumber(node_id)) |current| {
        if (std.math.isFinite(current)) return current;
    }

    return 0.0;
}

fn inputStepScaleValue(self: *const DomStore, node_id: NodeId, kind: InputValueAsNumberKind) errors.Result(f64) {
    const node = self.nodeAt(node_id) orelse return error.DomError;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return error.DomError,
    };

    const default_step: f64 = switch (kind) {
        .number, .range, .date, .month, .week => 1.0,
        .datetime_local, .time => 60.0,
    };

    const raw_step = if (elementAttributeValue(element, "step")) |value| value else return switch (kind) {
        .number, .range => default_step,
        .date => default_step * @as(f64, @floatFromInt(millisPerDay())),
        .datetime_local, .time => default_step * 1000.0,
        .month => default_step,
        .week => default_step * @as(f64, @floatFromInt(millisPerDay())) * 7.0,
    };
    const trimmed = std.mem.trim(u8, raw_step, " \t\r\n\x0c");
    if (trimmed.len == 0) {
        return switch (kind) {
            .number, .range => default_step,
            .date => default_step * @as(f64, @floatFromInt(millisPerDay())),
            .datetime_local, .time => default_step * 1000.0,
            .month => default_step,
            .week => default_step * @as(f64, @floatFromInt(millisPerDay())) * 7.0,
        };
    }
    if (std.ascii.eqlIgnoreCase(trimmed, "any")) return error.DomError;
    const parsed = std.fmt.parseFloat(f64, trimmed) catch return error.DomError;
    if (!std.math.isFinite(parsed) or parsed <= 0) return error.DomError;

    return switch (kind) {
        .number, .range => parsed,
        .date => parsed * @as(f64, @floatFromInt(millisPerDay())),
        .datetime_local, .time => parsed * 1000.0,
        .month => parsed,
        .week => parsed * @as(f64, @floatFromInt(millisPerDay())) * 7.0,
    };
}

fn inputValueTextFromNumber(
    self: *const DomStore,
    node_id: NodeId,
    kind: InputValueAsNumberKind,
    value: f64,
) errors.Result([]u8) {
    return switch (kind) {
        .number => blk: {
            if (!std.math.isFinite(value)) break :blk allocatorDupe(self, "");
            break :blk try std.fmt.allocPrint(self.allocator, "{d}", .{value});
        },
        .range => blk: {
            const normalized = normalizedRangeValue(self, node_id, if (std.math.isFinite(value)) value else null) orelse return error.DomError;
            break :blk try std.fmt.allocPrint(self.allocator, "{d}", .{normalized});
        },
        .date => if (!std.math.isFinite(value)) allocatorDupe(self, "") else formatDateValue(self.allocator, value),
        .datetime_local => if (!std.math.isFinite(value)) allocatorDupe(self, "") else formatDateTimeLocalValue(self.allocator, value),
        .month => if (!std.math.isFinite(value)) allocatorDupe(self, "") else formatMonthValue(self.allocator, value),
        .week => if (!std.math.isFinite(value)) allocatorDupe(self, "") else formatWeekValue(self.allocator, value),
        .time => if (!std.math.isFinite(value)) allocatorDupe(self, "") else formatTimeValue(self.allocator, value),
    };
}

fn allocatorDupe(allocator_owner: *const DomStore, text: []const u8) errors.Result([]u8) {
    return allocator_owner.allocator.dupe(u8, text);
}

fn normalizedRangeValue(self: *const DomStore, node_id: NodeId, candidate: ?f64) ?f64 {
    const limits = numericRangeLimits(self, node_id) orelse return null;

    if (limits.min) |min| {
        if (limits.max) |max| {
            if (max < min) return min;
        }
    }

    var value = candidate orelse defaultRangeValue(limits);
    if (!std.math.isFinite(value)) {
        value = defaultRangeValue(limits);
    }

    if (limits.min) |min| {
        if (value < min) value = min;
    }
    if (limits.max) |max| {
        if (value > max) value = max;
    }

    if (numericStepValue(self, node_id)) |step| {
        const base = numericStepBase(self, node_id);
        value = std.math.round((value - base) / step) * step + base;
        if (limits.min) |min| {
            if (value < min) value = min;
        }
        if (limits.max) |max| {
            if (value > max) value = max;
        }
    }

    return value;
}

fn defaultRangeValue(limits: NumericRangeLimits) f64 {
    const min = limits.min orelse 0.0;
    const max = limits.max orelse 100.0;
    if (max < min) return min;
    return (min + max) / 2.0;
}

fn parseUnsignedComponent(text: []const u8) ?u32 {
    return std.fmt.parseInt(u32, text, 10) catch null;
}

fn parseDateValue(text: []const u8) ?f64 {
    if (text.len != 10) return null;
    if (text[4] != '-' or text[7] != '-') return null;
    const year = parseUnsignedComponent(text[0..4]) orelse return null;
    const month = parseUnsignedComponent(text[5..7]) orelse return null;
    const day = parseUnsignedComponent(text[8..10]) orelse return null;
    if (month < 1 or month > 12) return null;
    if (day < 1 or day > daysInMonth(@as(i64, @intCast(year)), @as(u8, @intCast(month)))) return null;
    const days = daysFromCivil(@as(i64, @intCast(year)), @as(u8, @intCast(month)), @as(u8, @intCast(day)));
    return @as(f64, @floatFromInt(days)) * @as(f64, @floatFromInt(millisPerDay()));
}

fn parseMonthValue(text: []const u8) ?f64 {
    if (text.len != 7) return null;
    if (text[4] != '-') return null;
    const year = parseUnsignedComponent(text[0..4]) orelse return null;
    const month = parseUnsignedComponent(text[5..7]) orelse return null;
    if (month < 1 or month > 12) return null;
    const days = daysFromCivil(@as(i64, @intCast(year)), @as(u8, @intCast(month)), 1);
    return @as(f64, @floatFromInt(days)) * @as(f64, @floatFromInt(millisPerDay()));
}

fn parseTimeValue(text: []const u8) ?f64 {
    const dot_index = std.mem.indexOfScalar(u8, text, '.') orelse text.len;
    const main = text[0..dot_index];
    const fraction = if (dot_index < text.len) text[dot_index + 1 ..] else "";

    if (main.len != 5 and main.len != 8) return null;
    if (main[2] != ':') return null;
    const hour = parseUnsignedComponent(main[0..2]) orelse return null;
    const minute = parseUnsignedComponent(main[3..5]) orelse return null;
    var second: u32 = 0;
    if (main.len == 8) {
        if (main[5] != ':') return null;
        second = parseUnsignedComponent(main[6..8]) orelse return null;
    }
    if (hour > 23 or minute > 59 or second > 59) return null;

    var millisecond: u32 = 0;
    if (fraction.len != 0) {
        if (fraction.len > 3) return null;
        millisecond = parseUnsignedComponent(fraction) orelse return null;
        millisecond *= switch (fraction.len) {
            1 => 100,
            2 => 10,
            else => 1,
        };
    }

    const total_ms = (@as(i64, @intCast(hour)) * 3600000) +
        (@as(i64, @intCast(minute)) * 60000) +
        (@as(i64, @intCast(second)) * 1000) +
        @as(i64, @intCast(millisecond));
    return @as(f64, @floatFromInt(total_ms));
}

fn parseDateTimeLocalValue(text: []const u8) ?f64 {
    const separator = std.mem.indexOfScalar(u8, text, 'T') orelse return null;
    if (separator != 10) return null;
    const date = parseDateValue(text[0..10]) orelse return null;
    const time = parseTimeValue(text[11..]) orelse return null;
    return date + time;
}

fn parseWeekValue(text: []const u8) ?f64 {
    if (text.len != 8) return null;
    if (text[4] != '-' or (text[5] != 'W' and text[5] != 'w')) return null;
    const year = parseUnsignedComponent(text[0..4]) orelse return null;
    const week = parseUnsignedComponent(text[6..8]) orelse return null;
    if (week < 1 or week > weeksInIsoWeekYear(@as(i64, @intCast(year)))) return null;
    const days = isoWeekStartDays(@as(i64, @intCast(year)), @as(u8, @intCast(week)));
    return @as(f64, @floatFromInt(days)) * @as(f64, @floatFromInt(millisPerDay()));
}

fn formatDateValue(allocator: std.mem.Allocator, value: f64) errors.Result([]u8) {
    const calendar = calendarFromMilliseconds(value);
    const year_text = try formatYearText(allocator, calendar.year);
    defer allocator.free(year_text);
    return try std.fmt.allocPrint(allocator, "{s}-{d:0>2}-{d:0>2}", .{ year_text, calendar.month, calendar.day });
}

fn formatMonthValue(allocator: std.mem.Allocator, value: f64) errors.Result([]u8) {
    const calendar = calendarFromMilliseconds(value);
    const year_text = try formatYearText(allocator, calendar.year);
    defer allocator.free(year_text);
    return try std.fmt.allocPrint(allocator, "{s}-{d:0>2}", .{ year_text, calendar.month });
}

fn formatTimeValue(allocator: std.mem.Allocator, value: f64) errors.Result([]u8) {
    const time = timeFromMilliseconds(value);
    return try formatTimeParts(allocator, time.hour, time.minute, time.second, time.millisecond);
}

fn formatDateTimeLocalValue(allocator: std.mem.Allocator, value: f64) errors.Result([]u8) {
    const calendar = calendarFromMilliseconds(value);
    const time = timeFromMilliseconds(value);
    const year_text = try formatYearText(allocator, calendar.year);
    defer allocator.free(year_text);
    const date_text = try std.fmt.allocPrint(allocator, "{s}-{d:0>2}-{d:0>2}", .{ year_text, calendar.month, calendar.day });
    defer allocator.free(date_text);
    const time_text = try formatTimeParts(allocator, time.hour, time.minute, time.second, time.millisecond);
    defer allocator.free(time_text);
    return try std.fmt.allocPrint(allocator, "{s}T{s}", .{ date_text, time_text });
}

pub fn formatDateTimeIsoValue(allocator: std.mem.Allocator, value: f64) errors.Result([]u8) {
    const calendar = calendarFromMilliseconds(value);
    const time = timeFromMilliseconds(value);
    const year_text = try formatYearText(allocator, calendar.year);
    defer allocator.free(year_text);
    const date_text = try std.fmt.allocPrint(allocator, "{s}-{d:0>2}-{d:0>2}", .{ year_text, calendar.month, calendar.day });
    defer allocator.free(date_text);
    const fraction_full = try std.fmt.allocPrint(allocator, "{d:0>3}", .{time.millisecond});
    defer allocator.free(fraction_full);
    const time_text = try std.fmt.allocPrint(allocator, "{d:0>2}:{d:0>2}:{d:0>2}.{s}", .{ time.hour, time.minute, time.second, fraction_full });
    defer allocator.free(time_text);
    return try std.fmt.allocPrint(allocator, "{s}T{s}Z", .{ date_text, time_text });
}

fn formatWeekValue(allocator: std.mem.Allocator, value: f64) errors.Result([]u8) {
    const calendar = calendarFromMilliseconds(value);
    const week = isoWeekInfoFromDays(calendar.days);
    const year_text = try formatYearText(allocator, week.week_year);
    defer allocator.free(year_text);
    return try std.fmt.allocPrint(allocator, "{s}-W{d:0>2}", .{ year_text, week.week });
}

fn formatTimeParts(
    allocator: std.mem.Allocator,
    hour: u8,
    minute: u8,
    second: u8,
    millisecond: u16,
) errors.Result([]u8) {
    if (second == 0 and millisecond == 0) {
        return try std.fmt.allocPrint(allocator, "{d:0>2}:{d:0>2}", .{ hour, minute });
    }
    if (millisecond == 0) {
        return try std.fmt.allocPrint(allocator, "{d:0>2}:{d:0>2}:{d:0>2}", .{ hour, minute, second });
    }

    const fraction_full = try std.fmt.allocPrint(allocator, "{d:0>3}", .{millisecond});
    defer allocator.free(fraction_full);
    var fraction_len = fraction_full.len;
    while (fraction_len > 1 and fraction_full[fraction_len - 1] == '0') {
        fraction_len -= 1;
    }
    return try std.fmt.allocPrint(allocator, "{d:0>2}:{d:0>2}:{d:0>2}.{s}", .{ hour, minute, second, fraction_full[0..fraction_len] });
}

fn formatYearText(allocator: std.mem.Allocator, year: i64) errors.Result([]u8) {
    const magnitude: u64 = if (year < 0)
        @as(u64, @intCast(-year))
    else
        @as(u64, @intCast(year));
    const digits = try std.fmt.allocPrint(allocator, "{d:0>4}", .{magnitude});
    if (year >= 0) return digits;

    defer allocator.free(digits);
    return try std.fmt.allocPrint(allocator, "-{s}", .{digits});
}

const CalendarDate = struct {
    year: i64,
    month: u8,
    day: u8,
    weekday: u8,
    days: i64,
};

const TimeParts = struct {
    hour: u8,
    minute: u8,
    second: u8,
    millisecond: u16,
};

fn millisPerDay() i64 {
    return 86_400_000;
}

fn isLeapYearNumber(year: i64) bool {
    if (@mod(year, 4) != 0) return false;
    if (@mod(year, 100) != 0) return true;
    return @mod(year, 400) == 0;
}

fn daysInMonth(year: i64, month: u8) u8 {
    return switch (month) {
        1, 3, 5, 7, 8, 10, 12 => 31,
        4, 6, 9, 11 => 30,
        2 => if (isLeapYearNumber(year)) 29 else 28,
        else => 30,
    };
}

fn daysFromCivil(year: i64, month: u8, day: u8) i64 {
    var y = year;
    const m: i64 = month;
    const d: i64 = day;
    y -= if (m <= 2) 1 else 0;
    const era = @divFloor(y, 400);
    const yoe = y - era * 400;
    const mp: i64 = m + @as(i64, if (m > 2) -3 else 9);
    const doy = @divFloor(153 * mp + 2, 5) + d - 1;
    const doe = yoe * 365 + @divFloor(yoe, 4) - @divFloor(yoe, 100) + doy;
    return era * 146097 + doe - 719468;
}

fn civilFromDays(days: i64) CalendarDate {
    const z = days + 719468;
    const era = @divFloor(z, 146097);
    const doe = z - era * 146097;
    const yoe = @divFloor(doe - @divFloor(doe, 1460) + @divFloor(doe, 36524) - @divFloor(doe, 146096), 365);
    var y = yoe + era * 400;
    const doy = doe - (yoe * 365 + @divFloor(yoe, 4) - @divFloor(yoe, 100));
    const mp = @divFloor(5 * doy + 2, 153);
    const day = doy - @divFloor(153 * mp + 2, 5) + 1;
    const month: i64 = mp + @as(i64, if (mp < 10) 3 else -9);
    y += if (month <= 2) 1 else 0;
    return .{
        .year = y,
        .month = @as(u8, @intCast(month)),
        .day = @as(u8, @intCast(day)),
        .weekday = @as(u8, @intCast(@mod(days + 3, 7) + 1)),
        .days = days,
    };
}

fn timeFromMilliseconds(value: f64) TimeParts {
    const rounded = @as(i64, @intFromFloat(std.math.floor(value)));
    const millis = @mod(rounded, millisPerDay());
    const hour = @as(u8, @intCast(@divTrunc(millis, 3600000)));
    const minute = @as(u8, @intCast(@divTrunc(@mod(millis, 3600000), 60000)));
    const second = @as(u8, @intCast(@divTrunc(@mod(millis, 60000), 1000)));
    const millisecond = @as(u16, @intCast(@mod(millis, 1000)));
    return .{
        .hour = hour,
        .minute = minute,
        .second = second,
        .millisecond = millisecond,
    };
}

fn calendarFromMilliseconds(value: f64) CalendarDate {
    const days = @as(i64, @intFromFloat(std.math.floor(value / @as(f64, @floatFromInt(millisPerDay())))));
    return civilFromDays(days);
}

fn weeksInIsoWeekYear(year: i64) u8 {
    const jan1 = civilFromDays(daysFromCivil(year, 1, 1));
    return if (jan1.weekday == 4 or (isLeapYearNumber(year) and jan1.weekday == 3)) 53 else 52;
}

fn isoWeekStartDays(week_year: i64, week: u8) i64 {
    const jan4 = daysFromCivil(week_year, 1, 4);
    const jan4_weekday = @as(i64, @intCast(civilFromDays(jan4).weekday));
    const week1_monday = jan4 - (jan4_weekday - 1);
    return week1_monday + @as(i64, @intCast(week - 1)) * 7;
}

fn isoWeekInfoFromDays(days: i64) struct { week_year: i64, week: u8 } {
    const weekday = @as(i64, @intCast(civilFromDays(days).weekday));
    const thursday_days = days + (4 - weekday);
    const week_year = civilFromDays(thursday_days).year;
    const week1_monday = isoWeekStartDays(week_year, 1);
    const week = @as(u8, @intCast(@divFloor(thursday_days - week1_monday, 7) + 1));
    return .{ .week_year = week_year, .week = week };
}

fn isValidEmailAddress(value: []const u8) bool {
    if (value.len == 0) return false;
    if (std.mem.indexOfAny(u8, value, " \t\r\n\x0c") != null) return false;

    const at_index = std.mem.indexOfScalar(u8, value, '@') orelse return false;
    if (at_index == 0 or at_index + 1 >= value.len) return false;
    if (std.mem.indexOfScalarPos(u8, value, at_index + 1, '@') != null) return false;

    const local_part = value[0..at_index];
    const domain_part = value[at_index + 1 ..];
    if (local_part[0] == '.' or local_part[local_part.len - 1] == '.') return false;
    if (domain_part[0] == '.' or domain_part[domain_part.len - 1] == '.') return false;

    if (std.mem.indexOfScalar(u8, domain_part, '.') != null) return true;
    return domain_part.len >= 2 and domain_part[0] == '[' and domain_part[domain_part.len - 1] == ']';
}

fn isValidEmailValue(value: []const u8, multiple: bool) bool {
    if (!multiple) {
        return isValidEmailAddress(value);
    }

    var parts = std.mem.splitScalar(u8, value, ',');
    var saw_value = false;
    while (parts.next()) |part| {
        const trimmed = std.mem.trim(u8, part, " \t\r\n\x0c");
        if (trimmed.len == 0) return false;
        saw_value = true;
        if (!isValidEmailAddress(trimmed)) return false;
    }
    return saw_value;
}

fn isValidUrlValue(allocator: std.mem.Allocator, value: []const u8) bool {
    if (value.len == 0) return false;
    if (std.mem.indexOfAny(u8, value, " \t\r\n\x0c") != null) return false;

    if (std.Uri.parse(value)) |_| {
        return true;
    } else |_| {}

    const base_uri = std.Uri.parse("https://example.invalid/") catch return false;
    const buffer = allocator.alloc(u8, value.len + 64) catch return false;
    defer allocator.free(buffer);
    @memcpy(buffer[0..value.len], value);
    var aux_buf = buffer;
    _ = std.Uri.resolveInPlace(base_uri, value.len, &aux_buf) catch return false;
    return true;
}

const PatternQuantifier = struct {
    min: usize,
    max: ?usize,
};

fn patternMatches(allocator: std.mem.Allocator, pattern: []const u8, value: []const u8) errors.Result(bool) {
    const matched = try patternMatchRange(allocator, pattern, 0, pattern.len, value, 0);
    if (matched == null) return false;
    return matched.? == value.len;
}

fn patternMatchRange(
    allocator: std.mem.Allocator,
    pattern: []const u8,
    start: usize,
    end: usize,
    value: []const u8,
    value_index: usize,
) errors.Result(?usize) {
    var segment_start = start;
    var index = start;
    var depth: usize = 0;
    var in_class = false;

    while (index < end) : (index += 1) {
        const byte = pattern[index];
        if (in_class) {
            if (byte == '\\') {
                index += 1;
                continue;
            }
            if (byte == ']') {
                in_class = false;
            }
            continue;
        }

        if (byte == '\\') {
            index += 1;
            continue;
        }
        if (byte == '[') {
            in_class = true;
            continue;
        }
        if (byte == '(') {
            depth += 1;
            continue;
        }
        if (byte == ')') {
            if (depth == 0) return null;
            depth -= 1;
            continue;
        }
        if (byte == '|' and depth == 0) {
            if (try patternMatchSequence(allocator, pattern, segment_start, index, value, value_index)) |matched| {
                return matched;
            }
            segment_start = index + 1;
        }
    }

    if (depth != 0 or in_class) return null;
    return try patternMatchSequence(allocator, pattern, segment_start, end, value, value_index);
}

fn patternMatchSequence(
    allocator: std.mem.Allocator,
    pattern: []const u8,
    start: usize,
    end: usize,
    value: []const u8,
    value_index: usize,
) errors.Result(?usize) {
    var p = start;
    const current_index = value_index;

    while (p < end) {
        const byte = pattern[p];
        if (byte == '^' or byte == '$') {
            p += 1;
            continue;
        }
        if (byte == '|' or byte == ')') return null;

        const atom_end = patternAtomEnd(pattern, p, end) orelse return null;
        const quantifier_start = atom_end;
        const has_quantifier = quantifier_start < end and patternQuantifierStart(pattern[quantifier_start]);
        const quant: PatternQuantifier = if (has_quantifier)
            patternQuantifier(pattern, atom_end, end) orelse return null
        else
            .{ .min = 1, .max = 1 };
        const next_pattern_index = if (has_quantifier) patternQuantifierEnd(pattern, atom_end, end) orelse return null else atom_end;

        if (has_quantifier and (pattern[p] == '^' or pattern[p] == '$')) return null;

        var positions: std.ArrayList(usize) = .empty;
        errdefer positions.deinit(allocator);

        var repeat_index = current_index;
        while (true) {
            const next_index = (try patternMatchAtomOnce(allocator, pattern, p, atom_end, value, repeat_index)) orelse break;
            if (next_index == repeat_index) break;
            try positions.append(allocator, next_index);
            repeat_index = next_index;
            if (quant.max) |max| {
                if (positions.items.len >= max) break;
            }
        }

        if (positions.items.len < quant.min) {
            positions.deinit(allocator);
            return null;
        }

        var repeat_count = if (quant.max) |max| @min(max, positions.items.len) else positions.items.len;
        while (repeat_count >= quant.min) {
            const next_index = if (repeat_count == 0) current_index else positions.items[repeat_count - 1];
            if (try patternMatchSequence(allocator, pattern, next_pattern_index, end, value, next_index)) |result| {
                positions.deinit(allocator);
                return result;
            }
            if (repeat_count == 0) break;
            repeat_count -= 1;
        }

        positions.deinit(allocator);
        return null;
    }

    return current_index;
}

fn patternQuantifierStart(byte: u8) bool {
    return byte == '*' or byte == '+' or byte == '?' or byte == '{';
}

fn patternQuantifierEnd(pattern: []const u8, start: usize, end: usize) ?usize {
    if (start >= end) return start;
    const byte = pattern[start];
    if (byte == '*' or byte == '+' or byte == '?') {
        return start + 1;
    }
    if (byte != '{') return start;

    var index = start + 1;
    var saw_digit = false;
    while (index < end and std.ascii.isDigit(pattern[index])) : (index += 1) {
        saw_digit = true;
    }
    if (!saw_digit) return null;
    if (index < end and pattern[index] == '}') return index + 1;
    if (index >= end or pattern[index] != ',') return null;
    index += 1;
    while (index < end and std.ascii.isDigit(pattern[index])) : (index += 1) {}
    if (index >= end or pattern[index] != '}') return null;
    return index + 1;
}

fn patternQuantifier(pattern: []const u8, start: usize, end: usize) ?PatternQuantifier {
    if (start >= end) return .{ .min = 1, .max = 1 };
    const byte = pattern[start];
    if (byte == '*') return .{ .min = 0, .max = null };
    if (byte == '+') return .{ .min = 1, .max = null };
    if (byte == '?') return .{ .min = 0, .max = 1 };
    if (byte != '{') return .{ .min = 1, .max = 1 };

    var index = start + 1;
    var min_value: usize = 0;
    var saw_digit = false;
    while (index < end and std.ascii.isDigit(pattern[index])) : (index += 1) {
        saw_digit = true;
        min_value = min_value * 10 + @as(usize, pattern[index] - '0');
    }
    if (!saw_digit) return null;

    if (index < end and pattern[index] == '}') {
        return .{ .min = min_value, .max = min_value };
    }

    if (index >= end or pattern[index] != ',') return null;
    index += 1;

    var max_value: ?usize = null;
    var has_max = false;
    var parsed_max: usize = 0;
    while (index < end and std.ascii.isDigit(pattern[index])) : (index += 1) {
        has_max = true;
        parsed_max = parsed_max * 10 + @as(usize, pattern[index] - '0');
    }
    if (has_max) max_value = parsed_max;

    if (index >= end or pattern[index] != '}') return null;
    if (max_value) |max| {
        if (max < min_value) return null;
    }
    return .{ .min = min_value, .max = max_value };
}

fn patternAtomEnd(pattern: []const u8, start: usize, end: usize) ?usize {
    if (start >= end) return null;
    switch (pattern[start]) {
        '\\' => {
            if (start + 1 >= end) return null;
            return start + 2;
        },
        '[' => return patternClassEnd(pattern, start + 1, end),
        '(' => return patternGroupEnd(pattern, start + 1, end),
        else => return start + 1,
    }
}

fn patternClassEnd(pattern: []const u8, start: usize, end: usize) ?usize {
    var index = start;
    var first_content = true;
    while (index < end) : (index += 1) {
        const byte = pattern[index];
        if (byte == '\\') {
            index += 1;
            first_content = false;
            continue;
        }
        if (byte == ']' and !first_content) {
            return index + 1;
        }
        first_content = false;
    }
    return null;
}

fn patternGroupEnd(pattern: []const u8, start: usize, end: usize) ?usize {
    var index = start;
    var depth: usize = 1;
    var in_class = false;
    while (index < end) : (index += 1) {
        const byte = pattern[index];
        if (in_class) {
            if (byte == '\\') {
                index += 1;
                continue;
            }
            if (byte == ']') in_class = false;
            continue;
        }
        if (byte == '\\') {
            index += 1;
            continue;
        }
        if (byte == '[') {
            in_class = true;
            continue;
        }
        if (byte == '(') {
            depth += 1;
            continue;
        }
        if (byte == ')') {
            depth -= 1;
            if (depth == 0) return index + 1;
        }
    }
    return null;
}

fn patternMatchAtomOnce(
    allocator: std.mem.Allocator,
    pattern: []const u8,
    start: usize,
    end: usize,
    value: []const u8,
    value_index: usize,
) errors.Result(?usize) {
    if (start >= end) return null;
    switch (pattern[start]) {
        '^', '$' => return value_index,
        '.' => {
            if (value_index >= value.len) return null;
            const byte = value[value_index];
            if (byte == '\n' or byte == '\r') return null;
            return value_index + 1;
        },
        '[' => {
            if (value_index >= value.len) return null;
            return if (patternClassMatches(pattern, start + 1, end - 1, value[value_index])) value_index + 1 else null;
        },
        '(' => {
            const close = patternGroupEnd(pattern, start + 1, end) orelse return null;
            const matched = try patternMatchRange(allocator, pattern, start + 1, close - 1, value, value_index);
            if (matched == null) return null;
            return matched;
        },
        '\\' => {
            if (start + 1 >= end) return null;
            if (value_index >= value.len) return null;
            return if (patternEscapeMatches(pattern[start + 1], value[value_index])) value_index + 1 else null;
        },
        else => {
            if (value_index >= value.len) return null;
            return if (pattern[start] == value[value_index]) value_index + 1 else null;
        },
    }
}

fn patternEscapeMatches(escaped: u8, byte: u8) bool {
    return switch (escaped) {
        'd' => std.ascii.isDigit(byte),
        'D' => !std.ascii.isDigit(byte),
        's' => isHtmlWhitespace(byte),
        'S' => !isHtmlWhitespace(byte),
        'w' => std.ascii.isAlphanumeric(byte) or byte == '_',
        'W' => !(std.ascii.isAlphanumeric(byte) or byte == '_'),
        't' => byte == '\t',
        'r' => byte == '\r',
        'n' => byte == '\n',
        'f' => byte == 0x0c,
        else => escaped == byte,
    };
}

fn patternClassMatches(pattern: []const u8, start: usize, end: usize, byte: u8) bool {
    if (start > end) return false;
    var index = start;
    var negated = false;
    if (index < end and pattern[index] == '^') {
        negated = true;
        index += 1;
    }

    var matched = false;
    while (index < end) : (index += 1) {
        const current = pattern[index];
        if (current == '\\' and index + 1 < end) {
            index += 1;
            if (patternEscapeMatches(pattern[index], byte)) matched = true;
            continue;
        }
        if (index + 2 < end and pattern[index + 1] == '-' and pattern[index + 2] != ']') {
            const range_start = current;
            const range_end = pattern[index + 2];
            if (range_start <= byte and byte <= range_end) matched = true;
            index += 2;
            continue;
        }
        if (current == byte) matched = true;
    }

    return if (negated) !matched else matched;
}

fn inputTypeMismatch(self: *const DomStore, allocator: std.mem.Allocator, node_id: NodeId, input_type: ?[]const u8, value: []const u8) bool {
    const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
    if (trimmed.len == 0) return false;

    const input_kind = input_type orelse "text";
    if (std.mem.eql(u8, input_kind, "email")) {
        const node = self.nodeAt(node_id) orelse return false;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return false,
        };
        const multiple = elementAttributeValue(element, "multiple") != null;
        return !isValidEmailValue(trimmed, multiple);
    }
    if (std.mem.eql(u8, input_kind, "url")) {
        return !isValidUrlValue(allocator, trimmed);
    }
    return false;
}

fn inputPatternMismatch(self: *const DomStore, allocator: std.mem.Allocator, node_id: NodeId, input_type: ?[]const u8, value: []const u8) errors.Result(bool) {
    if (!isPatternInputType(input_type)) return false;
    if (value.len == 0) return false;

    const node = self.nodeAt(node_id) orelse return false;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return false,
    };

    const pattern = elementAttributeValue(element, "pattern") orelse return false;
    const matched = try patternMatches(allocator, pattern, value);
    return !matched;
}

fn textLengthLimits(self: *const DomStore, node_id: NodeId) ?TextLengthLimits {
    const node = self.nodeAt(node_id) orelse return null;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return null,
    };

    if (std.mem.eql(u8, element.tag_name, "textarea")) {
        const min = if (elementAttributeValue(element, "minlength")) |value| blk: {
            const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
            if (trimmed.len == 0) break :blk null;
            break :blk std.fmt.parseInt(usize, trimmed, 10) catch null;
        } else null;

        const max = if (elementAttributeValue(element, "maxlength")) |value| blk: {
            const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
            if (trimmed.len == 0) break :blk null;
            break :blk std.fmt.parseInt(usize, trimmed, 10) catch null;
        } else null;

        return .{ .min = min, .max = max };
    }

    if (!std.mem.eql(u8, element.tag_name, "input")) {
        return null;
    }

    const input_type = elementAttributeValue(element, "type");
    if (!isLengthInputType(input_type)) return null;

    const min = if (elementAttributeValue(element, "minlength")) |value| blk: {
        const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
        if (trimmed.len == 0) break :blk null;
        break :blk std.fmt.parseInt(usize, trimmed, 10) catch null;
    } else null;

    const max = if (elementAttributeValue(element, "maxlength")) |value| blk: {
        const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
        if (trimmed.len == 0) break :blk null;
        break :blk std.fmt.parseInt(usize, trimmed, 10) catch null;
    } else null;

    return .{ .min = min, .max = max };
}

fn isLangPseudoClass(self: *const DomStore, node_id: NodeId, langs: []const []const u8) bool {
    const node = self.nodeAt(node_id) orelse return false;
    switch (node.kind) {
        .element => {},
        else => return false,
    }

    var current = node_id;
    while (true) {
        const current_node = self.nodeAt(current) orelse return false;
        switch (current_node.kind) {
            .element => |element| {
                if (elementAttributeValue(element, "lang")) |value| {
                    const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
                    if (trimmed.len > 0) {
                        for (langs) |lang| {
                            if (langMatchesRange(trimmed, lang)) return true;
                        }
                        return false;
                    }
                } else if (elementAttributeValue(element, "xml:lang")) |value| {
                    const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
                    if (trimmed.len > 0) {
                        for (langs) |lang| {
                            if (langMatchesRange(trimmed, lang)) return true;
                        }
                        return false;
                    }
                }

                if (parentOf(self, current)) |parent_id| {
                    current = parent_id;
                    continue;
                }
                break;
            },
            .document => break,
            else => return false,
        }
    }

    return false;
}

fn isDirPseudoClass(self: *const DomStore, node_id: NodeId, dir: SelectorDirValue) bool {
    const node = self.nodeAt(node_id) orelse return false;
    switch (node.kind) {
        .element => {},
        else => return false,
    }

    return inheritedDirectionality(self, node_id) == dir;
}

fn inheritedDirectionality(self: *const DomStore, node_id: NodeId) SelectorDirValue {
    var current = node_id;
    while (true) {
        const current_node = self.nodeAt(current) orelse break;
        switch (current_node.kind) {
            .element => |element| {
                if (elementAttributeValue(element, "dir")) |value| {
                    const trimmed = std.mem.trim(u8, value, " \t\r\n\x0c");
                    if (trimmed.len == 0) {
                        if (parentOf(self, current)) |parent_id| {
                            current = parent_id;
                            continue;
                        }
                        break;
                    }

                    if (asciiEqualIgnoreCase(trimmed, "ltr")) return .ltr;
                    if (asciiEqualIgnoreCase(trimmed, "rtl")) return .rtl;
                }

                if (parentOf(self, current)) |parent_id| {
                    current = parent_id;
                    continue;
                }
                break;
            },
            .document => break,
            else => break,
        }
    }

    return .ltr;
}

fn isFormStatePseudoCandidate(tag_name: []const u8) bool {
    return std.mem.eql(u8, tag_name, "input") or std.mem.eql(u8, tag_name, "button") or std.mem.eql(u8, tag_name, "select") or std.mem.eql(u8, tag_name, "textarea") or std.mem.eql(u8, tag_name, "option") or std.mem.eql(u8, tag_name, "optgroup") or std.mem.eql(u8, tag_name, "fieldset");
}

fn isRequiredPseudoCandidate(tag_name: []const u8) bool {
    return std.mem.eql(u8, tag_name, "input") or std.mem.eql(u8, tag_name, "select") or std.mem.eql(u8, tag_name, "textarea");
}

fn sameNodeId(left: NodeId, right: NodeId) bool {
    return left.index == right.index and left.generation == right.generation;
}

fn elementDataEqual(left: ElementData, right: ElementData) bool {
    if (!std.mem.eql(u8, left.tag_name, right.tag_name)) return false;
    if (left.attributes.items.len != right.attributes.items.len) return false;

    for (left.attributes.items) |left_attribute| {
        var matched = false;
        for (right.attributes.items) |right_attribute| {
            if (!asciiEqualIgnoreCase(left_attribute.name, right_attribute.name)) continue;
            if (!std.mem.eql(u8, left_attribute.value, right_attribute.value)) continue;
            matched = true;
            break;
        }
        if (!matched) return false;
    }

    return true;
}

fn nodeChildrenEqual(self: *const DomStore, left: []const NodeId, right: []const NodeId) bool {
    if (left.len != right.len) return false;
    for (left, right) |left_child, right_child| {
        if (!nodeIsEqualNode(self, left_child, right_child)) return false;
    }
    return true;
}

fn serializationNamespaceForChild(
    parent: *const NodeRecord,
    parent_namespace: SerializationNamespace,
    child_tag: []const u8,
) SerializationNamespace {
    switch (parent.kind) {
        .document => return namespaceForHtmlChild(child_tag),
        .element => |parent_element| switch (parent_namespace) {
            .html => return namespaceForHtmlChild(child_tag),
            .svg => {
                if (std.mem.eql(u8, parent_element.tag_name, "foreignobject")) {
                    return namespaceForHtmlChild(child_tag);
                }
                return namespaceForSvgChild(child_tag);
            },
            .mathml => {
                if (std.mem.eql(u8, parent_element.tag_name, "annotation-xml") and isMathMlHtmlIntegrationPoint(parent_element)) {
                    return namespaceForHtmlChild(child_tag);
                }
                return namespaceForMathMlChild(child_tag);
            },
        },
        else => return namespaceForHtmlChild(child_tag),
    }
}

fn namespaceForHtmlChild(child_tag: []const u8) SerializationNamespace {
    if (std.mem.eql(u8, child_tag, "svg")) return .svg;
    if (std.mem.eql(u8, child_tag, "math")) return .mathml;
    return .html;
}

fn namespaceForSvgChild(child_tag: []const u8) SerializationNamespace {
    if (std.mem.eql(u8, child_tag, "svg")) return .svg;
    if (std.mem.eql(u8, child_tag, "math")) return .mathml;
    return .svg;
}

fn namespaceForMathMlChild(child_tag: []const u8) SerializationNamespace {
    if (std.mem.eql(u8, child_tag, "svg")) return .svg;
    if (std.mem.eql(u8, child_tag, "math")) return .mathml;
    return .mathml;
}

fn isMathMlHtmlIntegrationPoint(element: ElementData) bool {
    if (!std.mem.eql(u8, element.tag_name, "annotation-xml")) {
        return false;
    }

    const encoding = elementAttributeValue(element, "encoding") orelse return false;
    return std.mem.eql(u8, encoding, "text/html") or std.mem.eql(u8, encoding, "application/xhtml+xml");
}

fn serializedElementName(namespace: SerializationNamespace, name: []const u8) []const u8 {
    return switch (namespace) {
        .html => name,
        .svg => adjustSvgElementName(name),
        .mathml => name,
    };
}

fn serializedAttributeName(namespace: SerializationNamespace, element: *const ElementData, name: []const u8) []const u8 {
    _ = element;
    return switch (namespace) {
        .html => name,
        .svg => adjustSvgAttributeName(name),
        .mathml => if (std.mem.eql(u8, name, "definitionurl")) "definitionURL" else name,
    };
}

fn adjustSvgElementName(name: []const u8) []const u8 {
    const mappings = [_]struct { from: []const u8, to: []const u8 }{
        .{ .from = "altglyph", .to = "altGlyph" },
        .{ .from = "altglyphdef", .to = "altGlyphDef" },
        .{ .from = "altglyphitem", .to = "altGlyphItem" },
        .{ .from = "animatecolor", .to = "animateColor" },
        .{ .from = "animatemotion", .to = "animateMotion" },
        .{ .from = "animatetransform", .to = "animateTransform" },
        .{ .from = "clippath", .to = "clipPath" },
        .{ .from = "feblend", .to = "feBlend" },
        .{ .from = "fecolormatrix", .to = "feColorMatrix" },
        .{ .from = "fecomponenttransfer", .to = "feComponentTransfer" },
        .{ .from = "fecomposite", .to = "feComposite" },
        .{ .from = "feconvolvematrix", .to = "feConvolveMatrix" },
        .{ .from = "fediffuselighting", .to = "feDiffuseLighting" },
        .{ .from = "fedisplacementmap", .to = "feDisplacementMap" },
        .{ .from = "fedistantlight", .to = "feDistantLight" },
        .{ .from = "fedropshadow", .to = "feDropShadow" },
        .{ .from = "feflood", .to = "feFlood" },
        .{ .from = "fefunca", .to = "feFuncA" },
        .{ .from = "fefuncb", .to = "feFuncB" },
        .{ .from = "fefuncg", .to = "feFuncG" },
        .{ .from = "fefuncr", .to = "feFuncR" },
        .{ .from = "fegaussianblur", .to = "feGaussianBlur" },
        .{ .from = "feimage", .to = "feImage" },
        .{ .from = "femerge", .to = "feMerge" },
        .{ .from = "femergenode", .to = "feMergeNode" },
        .{ .from = "femorphology", .to = "feMorphology" },
        .{ .from = "feoffset", .to = "feOffset" },
        .{ .from = "fepointlight", .to = "fePointLight" },
        .{ .from = "fespecularlighting", .to = "feSpecularLighting" },
        .{ .from = "fespotlight", .to = "feSpotLight" },
        .{ .from = "fetile", .to = "feTile" },
        .{ .from = "feturbulence", .to = "feTurbulence" },
        .{ .from = "foreignobject", .to = "foreignObject" },
        .{ .from = "glyphref", .to = "glyphRef" },
        .{ .from = "lineargradient", .to = "linearGradient" },
        .{ .from = "radialgradient", .to = "radialGradient" },
        .{ .from = "textpath", .to = "textPath" },
    };

    for (mappings) |mapping| {
        if (std.mem.eql(u8, name, mapping.from)) return mapping.to;
    }
    return name;
}

fn adjustSvgAttributeName(name: []const u8) []const u8 {
    const mappings = [_]struct { from: []const u8, to: []const u8 }{
        .{ .from = "attributename", .to = "attributeName" },
        .{ .from = "attributetype", .to = "attributeType" },
        .{ .from = "basefrequency", .to = "baseFrequency" },
        .{ .from = "baseprofile", .to = "baseProfile" },
        .{ .from = "calcmode", .to = "calcMode" },
        .{ .from = "clippathunits", .to = "clipPathUnits" },
        .{ .from = "diffuseconstant", .to = "diffuseConstant" },
        .{ .from = "edgemode", .to = "edgeMode" },
        .{ .from = "filterunits", .to = "filterUnits" },
        .{ .from = "glyphref", .to = "glyphRef" },
        .{ .from = "gradienttransform", .to = "gradientTransform" },
        .{ .from = "gradientunits", .to = "gradientUnits" },
        .{ .from = "kernelmatrix", .to = "kernelMatrix" },
        .{ .from = "kernelunitlength", .to = "kernelUnitLength" },
        .{ .from = "keypoints", .to = "keyPoints" },
        .{ .from = "keysplines", .to = "keySplines" },
        .{ .from = "keytimes", .to = "keyTimes" },
        .{ .from = "lengthadjust", .to = "lengthAdjust" },
        .{ .from = "limitingconeangle", .to = "limitingConeAngle" },
        .{ .from = "markerheight", .to = "markerHeight" },
        .{ .from = "markerunits", .to = "markerUnits" },
        .{ .from = "markerwidth", .to = "markerWidth" },
        .{ .from = "maskcontentunits", .to = "maskContentUnits" },
        .{ .from = "maskunits", .to = "maskUnits" },
        .{ .from = "numoctaves", .to = "numOctaves" },
        .{ .from = "pathlength", .to = "pathLength" },
        .{ .from = "patterncontentunits", .to = "patternContentUnits" },
        .{ .from = "patterntransform", .to = "patternTransform" },
        .{ .from = "patternunits", .to = "patternUnits" },
        .{ .from = "pointsatx", .to = "pointsAtX" },
        .{ .from = "pointsaty", .to = "pointsAtY" },
        .{ .from = "pointsatz", .to = "pointsAtZ" },
        .{ .from = "preservealpha", .to = "preserveAlpha" },
        .{ .from = "preserveaspectratio", .to = "preserveAspectRatio" },
        .{ .from = "primitiveunits", .to = "primitiveUnits" },
        .{ .from = "refx", .to = "refX" },
        .{ .from = "refy", .to = "refY" },
        .{ .from = "repeatcount", .to = "repeatCount" },
        .{ .from = "repeatdur", .to = "repeatDur" },
        .{ .from = "requiredextensions", .to = "requiredExtensions" },
        .{ .from = "requiredfeatures", .to = "requiredFeatures" },
        .{ .from = "specularconstant", .to = "specularConstant" },
        .{ .from = "specularexponent", .to = "specularExponent" },
        .{ .from = "spreadmethod", .to = "spreadMethod" },
        .{ .from = "startoffset", .to = "startOffset" },
        .{ .from = "stddeviation", .to = "stdDeviation" },
        .{ .from = "stitchtiles", .to = "stitchTiles" },
        .{ .from = "surfacescale", .to = "surfaceScale" },
        .{ .from = "systemlanguage", .to = "systemLanguage" },
        .{ .from = "tablevalues", .to = "tableValues" },
        .{ .from = "targetx", .to = "targetX" },
        .{ .from = "targety", .to = "targetY" },
        .{ .from = "textlength", .to = "textLength" },
        .{ .from = "viewbox", .to = "viewBox" },
        .{ .from = "viewtarget", .to = "viewTarget" },
        .{ .from = "xchannelselector", .to = "xChannelSelector" },
        .{ .from = "ychannelselector", .to = "yChannelSelector" },
        .{ .from = "zoomandpan", .to = "zoomAndPan" },
    };

    for (mappings) |mapping| {
        if (std.mem.eql(u8, name, mapping.from)) return mapping.to;
    }
    return name;
}

fn sliceContainsNodeId(nodes: []const NodeId, needle: NodeId) bool {
    for (nodes) |candidate| {
        if (sameNodeId(candidate, needle)) return true;
    }
    return false;
}

fn collectSelectorMatches(
    self: *const DomStore,
    node_id: NodeId,
    chains: []const SelectorChain,
    scope_root: ?NodeId,
    results: *std.ArrayList(NodeId),
    allocator: std.mem.Allocator,
) errors.Result(void) {
    const node = self.nodeAt(node_id) orelse return error.HtmlParse;
    switch (node.kind) {
        .element => {
            if (nodeMatchesAnyChain(self, node_id, chains, scope_root)) {
                try results.append(allocator, node_id);
            }
        },
        else => {},
    }

    for (node.children.items) |child_id| {
        try collectSelectorMatches(self, child_id, chains, scope_root, results, allocator);
    }
}

fn collectSelectorMatchesWithin(
    self: *const DomStore,
    node_id: NodeId,
    chains: []const SelectorChain,
    scope_root: ?NodeId,
    results: *std.ArrayList(NodeId),
    allocator: std.mem.Allocator,
) errors.Result(void) {
    const node = self.nodeAt(node_id) orelse return error.HtmlParse;
    for (node.children.items) |child_id| {
        const child = self.nodeAt(child_id) orelse return error.HtmlParse;
        switch (child.kind) {
            .element => {
                if (nodeMatchesAnyChain(self, child_id, chains, scope_root)) {
                    try results.append(allocator, child_id);
                }
            },
            else => {},
        }

        try collectSelectorMatchesWithin(self, child_id, chains, scope_root, results, allocator);
    }
}

fn nodeMatchesAnyChain(self: *const DomStore, node_id: NodeId, chains: []const SelectorChain, scope_root: ?NodeId) bool {
    for (chains) |chain| {
        if (nodeMatchesSelectorChain(self, node_id, &chain, scope_root)) return true;
    }
    return false;
}

fn nodeMatchesSelectorChain(self: *const DomStore, node_id: NodeId, chain: *const SelectorChain, scope_root: ?NodeId) bool {
    if (chain.parts.items.len == 0) return false;
    const last_index = chain.parts.items.len - 1;
    return nodeMatchesSelectorChainPart(self, node_id, chain.parts.items, chain.relations.items, last_index, scope_root);
}

fn nodeMatchesSelectorChainPart(
    self: *const DomStore,
    node_id: NodeId,
    parts: []const SelectorQuery,
    relations: []const SelectorCombinator,
    index: usize,
    scope_root: ?NodeId,
) bool {
    if (!nodeMatchesSelectorQuery(self, node_id, parts[index], scope_root)) return false;
    if (index == 0) return true;

    return switch (relations[index - 1]) {
        .child => blk: {
            const parent_id = parentOf(self, node_id) orelse break :blk false;
            break :blk nodeMatchesSelectorChainPart(self, parent_id, parts, relations, index - 1, scope_root);
        },
        .descendant => blk: {
            var ancestor = parentOf(self, node_id);
            while (ancestor) |ancestor_id| {
                if (nodeMatchesSelectorChainPart(self, ancestor_id, parts, relations, index - 1, scope_root)) {
                    break :blk true;
                }
                ancestor = parentOf(self, ancestor_id);
            }
            break :blk false;
        },
        .adjacent_sibling => blk: {
            const sibling_id = previousElementSibling(self, node_id) orelse break :blk false;
            break :blk nodeMatchesSelectorChainPart(self, sibling_id, parts, relations, index - 1, scope_root);
        },
        .general_sibling => blk: {
            var sibling = previousElementSibling(self, node_id);
            while (sibling) |sibling_id| {
                if (nodeMatchesSelectorChainPart(self, sibling_id, parts, relations, index - 1, scope_root)) {
                    break :blk true;
                }
                sibling = previousElementSibling(self, sibling_id);
            }
            break :blk false;
        },
    };
}

fn nodeMatchesSelectorQuery(self: *const DomStore, node_id: NodeId, query: SelectorQuery, scope_root: ?NodeId) bool {
    const node = self.nodeAt(node_id) orelse return false;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return false,
    };

    if (query.tag) |tag| {
        if (!asciiEqualIgnoreCase(element.tag_name, tag)) return false;
    }

    if (query.id) |id| {
        const actual_id = elementAttributeValue(element, "id") orelse return false;
        if (!std.mem.eql(u8, actual_id, id)) return false;
    }

    for (query.classes.items) |class_name| {
        if (!elementHasClass(element, class_name)) return false;
    }

    for (query.attributes.items) |attribute| {
        if (!elementMatchesAttribute(element, attribute)) return false;
    }

    for (query.pseudos.items) |pseudo| {
        if (!nodeMatchesPseudoClass(self, node_id, pseudo, scope_root)) return false;
    }

    return true;
}

fn isTargetPseudoClass(self: *const DomStore, node_id: NodeId) bool {
    const fragment = self.targetFragment() orelse return false;
    const node = self.nodeAt(node_id) orelse return false;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return false,
    };

    if (elementAttributeValue(element, "id")) |id| {
        if (std.mem.eql(u8, id, fragment)) return true;
    }
    if (elementAttributeValue(element, "name")) |name| {
        if (std.mem.eql(u8, name, fragment)) return true;
    }

    if (self.targetNodeForFragment(fragment)) |target| {
        return sameNodeId(target, node_id);
    }

    return false;
}

fn isFocusPseudoClass(self: *const DomStore, node_id: NodeId) bool {
    return if (self.focusedNode()) |focused| sameNodeId(focused, node_id) else false;
}

fn isFocusWithinPseudoClass(self: *const DomStore, node_id: NodeId) bool {
    var current = self.focusedNode();
    while (current) |current_id| {
        if (sameNodeId(current_id, node_id)) return true;
        current = parentOf(self, current_id);
    }

    return false;
}

fn isPopoverOpenPseudoClass(self: *const DomStore, node_id: NodeId) bool {
    return self.popoverVisibleForNode(node_id) orelse false;
}

fn isNthChild(self: *const DomStore, node_id: NodeId, pattern: SelectorNthChildPattern) bool {
    const filters = if (pattern.of_selectors) |selectors| selectors.items else null;
    const position = elementChildPositionFiltered(self, node_id, filters) orelse return false;
    return matchesNthPattern(self, @intCast(position), pattern);
}

fn isNthLastChild(self: *const DomStore, node_id: NodeId, pattern: SelectorNthChildPattern) bool {
    const filters = if (pattern.of_selectors) |selectors| selectors.items else null;
    const position = elementChildPositionFromEndFiltered(self, node_id, filters) orelse return false;
    return matchesNthPattern(self, @intCast(position), pattern);
}

fn isNthOfType(self: *const DomStore, node_id: NodeId, pattern: SelectorNthChildPattern) bool {
    const filters = if (pattern.of_selectors) |selectors| selectors.items else null;
    const position = elementSiblingPositionOfTypeFiltered(self, node_id, filters) orelse return false;
    return matchesNthPattern(self, @intCast(position), pattern);
}

fn isNthLastOfType(self: *const DomStore, node_id: NodeId, pattern: SelectorNthChildPattern) bool {
    const filters = if (pattern.of_selectors) |selectors| selectors.items else null;
    const position = elementSiblingPositionFromEndOfTypeFiltered(self, node_id, filters) orelse return false;
    return matchesNthPattern(self, @intCast(position), pattern);
}

fn elementChildPositionFiltered(
    self: *const DomStore,
    node_id: NodeId,
    of_selectors: ?[]const SelectorChain,
) ?usize {
    const parent_id = parentOf(self, node_id) orelse return null;
    const parent = self.nodeAt(parent_id) orelse return null;
    var position: usize = 0;

    for (parent.children.items) |child_id| {
        const child = self.nodeAt(child_id) orelse return null;
        switch (child.kind) {
            .element => {
                if (!matchesNthChildOfFilters(self, child_id, of_selectors)) {
                    if (sameNodeId(child_id, node_id)) return null;
                    continue;
                }
                position += 1;
                if (sameNodeId(child_id, node_id)) return position;
            },
            else => if (sameNodeId(child_id, node_id)) return null,
        }
    }

    return null;
}

fn elementChildPositionFromEndFiltered(
    self: *const DomStore,
    node_id: NodeId,
    of_selectors: ?[]const SelectorChain,
) ?usize {
    const parent_id = parentOf(self, node_id) orelse return null;
    const parent = self.nodeAt(parent_id) orelse return null;
    var position: usize = 0;

    var index = parent.children.items.len;
    while (index > 0) {
        index -= 1;
        const child_id = parent.children.items[index];
        const child = self.nodeAt(child_id) orelse return null;
        switch (child.kind) {
            .element => {
                if (!matchesNthChildOfFilters(self, child_id, of_selectors)) {
                    if (sameNodeId(child_id, node_id)) return null;
                    continue;
                }
                position += 1;
                if (sameNodeId(child_id, node_id)) return position;
            },
            else => if (sameNodeId(child_id, node_id)) return null,
        }
    }

    return null;
}

fn matchesNthChildOfFilters(
    self: *const DomStore,
    node_id: NodeId,
    of_selectors: ?[]const SelectorChain,
) bool {
    const selectors = of_selectors orelse return true;
    for (selectors) |chain| {
        if (nodeMatchesSelectorChain(self, node_id, &chain, null)) return true;
    }
    return false;
}

fn elementSiblingPositionOfTypeFiltered(
    self: *const DomStore,
    node_id: NodeId,
    of_selectors: ?[]const SelectorChain,
) ?usize {
    const node = self.nodeAt(node_id) orelse return null;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return null,
    };

    const parent_id = node.parent orelse return null;
    const parent = self.nodeAt(parent_id) orelse return null;
    var matching_sibling_count: usize = 0;

    for (parent.children.items) |child_id| {
        const child = self.nodeAt(child_id) orelse return null;
        const child_element = switch (child.kind) {
            .element => |child_element| child_element,
            else => continue,
        };

        if (std.mem.eql(u8, child_element.tag_name, element.tag_name) and matchesNthOfTypeFilters(self, child_id, of_selectors)) {
            matching_sibling_count += 1;
            if (sameNodeId(child_id, node_id)) return matching_sibling_count;
        } else if (sameNodeId(child_id, node_id)) {
            return null;
        }
    }

    return null;
}

fn elementSiblingPositionFromEndOfTypeFiltered(
    self: *const DomStore,
    node_id: NodeId,
    of_selectors: ?[]const SelectorChain,
) ?usize {
    const node = self.nodeAt(node_id) orelse return null;
    const element = switch (node.kind) {
        .element => |element| element,
        else => return null,
    };

    const parent_id = node.parent orelse return null;
    const parent = self.nodeAt(parent_id) orelse return null;
    var matching_sibling_count: usize = 0;

    var index = parent.children.items.len;
    while (index > 0) {
        index -= 1;
        const child_id = parent.children.items[index];
        const child = self.nodeAt(child_id) orelse return null;
        const child_element = switch (child.kind) {
            .element => |child_element| child_element,
            else => continue,
        };

        if (std.mem.eql(u8, child_element.tag_name, element.tag_name) and matchesNthOfTypeFilters(self, child_id, of_selectors)) {
            matching_sibling_count += 1;
            if (sameNodeId(child_id, node_id)) return matching_sibling_count;
        } else if (sameNodeId(child_id, node_id)) {
            return null;
        }
    }

    return null;
}

fn matchesNthOfTypeFilters(
    self: *const DomStore,
    node_id: NodeId,
    of_selectors: ?[]const SelectorChain,
) bool {
    return matchesNthChildOfFilters(self, node_id, of_selectors);
}

fn matchesNthPattern(self: *const DomStore, position: isize, pattern: SelectorNthChildPattern) bool {
    _ = self;
    return switch (pattern.step) {
        0 => position == pattern.offset and position > 0,
        else => blk: {
            if (pattern.step > 0) {
                const diff = std.math.sub(isize, position, pattern.offset) catch return false;
                if (diff < 0) break :blk false;
                break :blk @mod(diff, pattern.step) == 0;
            }

            const step = if (pattern.step < 0) -pattern.step else pattern.step;
            const diff = std.math.sub(isize, pattern.offset, position) catch return false;
            if (diff < 0) break :blk false;
            break :blk @mod(diff, step) == 0;
        },
    };
}

fn nodeMatchesPseudoClass(self: *const DomStore, node_id: NodeId, pseudo: SelectorPseudoClass, scope_root: ?NodeId) bool {
    return switch (pseudo) {
        .scope => if (scope_root) |scope_id| sameNodeId(node_id, scope_id) else false,
        .root => sameNodeId(node_id, documentElementId(self) orelse return false),
        .empty => isEmptyElement(self, node_id),
        .first_child => isFirstElementChild(self, node_id),
        .last_child => isLastElementChild(self, node_id),
        .only_child => isOnlyElementChild(self, node_id),
        .first_of_type => isFirstElementOfType(self, node_id),
        .last_of_type => isLastElementOfType(self, node_id),
        .only_of_type => isOnlyElementOfType(self, node_id),
        .checked => isCheckedPseudo(self, node_id),
        .disabled => isDisabledPseudo(self, node_id),
        .enabled => isEnabledPseudo(self, node_id),
        .required => isRequiredPseudo(self, node_id),
        .optional => isOptionalPseudo(self, node_id),
        .link => isLinkPseudo(self, node_id),
        .any_link => isLinkPseudo(self, node_id),
        .defined => isDefinedPseudoClass(self, node_id),
        .placeholder_shown => isPlaceholderShownPseudo(self, node_id),
        .blank => isBlankPseudoClass(self, node_id),
        .indeterminate => isIndeterminatePseudoClass(self, node_id),
        .default => isDefaultPseudoClass(self, node_id),
        .valid => isValidPseudoClass(self, node_id),
        .invalid => isInvalidPseudoClass(self, node_id),
        .in_range => isInRangePseudoClass(self, node_id),
        .out_of_range => isOutOfRangePseudoClass(self, node_id),
        .read_only => isReadOnlyPseudoClass(self, node_id),
        .read_write => isReadWritePseudoClass(self, node_id),
        .target => isTargetPseudoClass(self, node_id),
        .focus => isFocusPseudoClass(self, node_id),
        .focus_visible => isFocusPseudoClass(self, node_id),
        .focus_within => isFocusWithinPseudoClass(self, node_id),
        .popover_open => isPopoverOpenPseudoClass(self, node_id),
        .lang => |langs| isLangPseudoClass(self, node_id, langs.items),
        .dir => |dir| isDirPseudoClass(self, node_id, dir),
        .not => |selectors| blk: {
            for (selectors.items) |chain| {
                if (nodeMatchesSelectorChain(self, node_id, &chain, scope_root)) break :blk false;
            }
            break :blk true;
        },
        .is => |selectors| blk: {
            for (selectors.items) |chain| {
                if (nodeMatchesSelectorChain(self, node_id, &chain, scope_root)) break :blk true;
            }
            break :blk false;
        },
        .where => |selectors| blk: {
            for (selectors.items) |chain| {
                if (nodeMatchesSelectorChain(self, node_id, &chain, scope_root)) break :blk true;
            }
            break :blk false;
        },
        .has => |relative_selectors| blk: {
            for (relative_selectors.items) |relative_selector| {
                if (matchesSelectorRelativeSelector(self, node_id, relative_selector, scope_root)) {
                    break :blk true;
                }
            }
            break :blk false;
        },
        .nth_child => |pattern| isNthChild(self, node_id, pattern),
        .nth_last_child => |pattern| isNthLastChild(self, node_id, pattern),
        .nth_of_type => |pattern| isNthOfType(self, node_id, pattern),
        .nth_last_of_type => |pattern| isNthLastOfType(self, node_id, pattern),
    };
}

fn matchesSelectorRelativeSelector(
    self: *const DomStore,
    node_id: NodeId,
    relative_selector: SelectorRelativeSelector,
    scope_root: ?NodeId,
) bool {
    const subject_scope = scope_root orelse node_id;

    if (relative_selector.combinator) |combinator| {
        return switch (combinator) {
            .child => hasChildMatchingChain(self, node_id, &relative_selector.chain, subject_scope),
            .adjacent_sibling => hasAdjacentSiblingMatchingChain(self, node_id, &relative_selector.chain, subject_scope),
            .general_sibling => hasGeneralSiblingMatchingChain(self, node_id, &relative_selector.chain, subject_scope),
            .descendant => hasDescendantMatchingChain(self, node_id, &relative_selector.chain, subject_scope),
        };
    }

    if (relative_selector.chain.parts.items.len == 1 and chainStartsWithScope(&relative_selector.chain)) {
        return nodeMatchesSelectorChain(self, node_id, &relative_selector.chain, subject_scope);
    }

    return hasDescendantMatchingChain(self, node_id, &relative_selector.chain, subject_scope);
}

fn chainStartsWithScope(chain: *const SelectorChain) bool {
    if (chain.parts.items.len == 0) return false;
    const first = chain.parts.items[0];
    for (first.pseudos.items) |pseudo| {
        switch (pseudo) {
            .scope => return true,
            else => {},
        }
    }
    return false;
}

fn hasDescendantMatchingChain(
    self: *const DomStore,
    node_id: NodeId,
    chain: *const SelectorChain,
    scope_root: ?NodeId,
) bool {
    const node = self.nodeAt(node_id) orelse return false;
    for (node.children.items) |child_id| {
        if (nodeMatchesSelectorChain(self, child_id, chain, scope_root)) {
            return true;
        }
        if (hasDescendantMatchingChain(self, child_id, chain, scope_root)) {
            return true;
        }
    }
    return false;
}

fn hasChildMatchingChain(
    self: *const DomStore,
    node_id: NodeId,
    chain: *const SelectorChain,
    scope_root: ?NodeId,
) bool {
    const node = self.nodeAt(node_id) orelse return false;
    for (node.children.items) |child_id| {
        if (nodeMatchesSelectorChain(self, child_id, chain, scope_root)) {
            return true;
        }
    }
    return false;
}

fn hasAdjacentSiblingMatchingChain(
    self: *const DomStore,
    node_id: NodeId,
    chain: *const SelectorChain,
    scope_root: ?NodeId,
) bool {
    const sibling_id = nextElementSibling(self, node_id) orelse return false;
    return nodeMatchesSelectorChain(self, sibling_id, chain, scope_root);
}

fn hasGeneralSiblingMatchingChain(
    self: *const DomStore,
    node_id: NodeId,
    chain: *const SelectorChain,
    scope_root: ?NodeId,
) bool {
    var sibling = nextElementSibling(self, node_id);
    while (sibling) |sibling_id| {
        if (nodeMatchesSelectorChain(self, sibling_id, chain, scope_root)) {
            return true;
        }
        sibling = nextElementSibling(self, sibling_id);
    }

    return false;
}

fn elementMatchesAttribute(element: ElementData, condition: SelectorAttribute) bool {
    const actual = elementAttributeValue(element, condition.name) orelse return false;
    const expected = condition.value orelse return condition.operator == .exists;

    return switch (condition.operator) {
        .exists => true,
        .equals => textMatchesByOperator(actual, expected, condition.case_sensitivity, .equals),
        .prefix => textMatchesByOperator(actual, expected, condition.case_sensitivity, .prefix),
        .suffix => textMatchesByOperator(actual, expected, condition.case_sensitivity, .suffix),
        .contains => textMatchesByOperator(actual, expected, condition.case_sensitivity, .contains),
        .contains_word => textMatchesByOperator(
            actual,
            expected,
            condition.case_sensitivity,
            .contains_word,
        ),
        .hyphen_prefix => textMatchesByOperator(
            actual,
            expected,
            condition.case_sensitivity,
            .hyphen_prefix,
        ),
    };
}

fn elementAttributeValue(element: ElementData, name: []const u8) ?[]const u8 {
    for (element.attributes.items) |attribute| {
        if (asciiEqualIgnoreCase(attribute.name, name)) {
            return attribute.value;
        }
    }
    return null;
}

fn elementHasClass(element: ElementData, class_name: []const u8) bool {
    const classes = elementAttributeValue(element, "class") orelse return false;
    var iter = std.mem.tokenizeAny(u8, classes, " \t\r\n\x0c");
    while (iter.next()) |candidate| {
        if (std.mem.eql(u8, candidate, class_name)) return true;
    }
    return false;
}

fn textMatchesByOperator(
    actual: []const u8,
    expected: []const u8,
    case_sensitivity: SelectorAttributeCaseSensitivity,
    operator: SelectorAttributeOperator,
) bool {
    return switch (operator) {
        .exists => true,
        .equals => textEquals(actual, expected, case_sensitivity),
        .prefix => textStartsWith(actual, expected, case_sensitivity),
        .suffix => textEndsWith(actual, expected, case_sensitivity),
        .contains => textContains(actual, expected, case_sensitivity),
        .contains_word => textContainsWord(actual, expected, case_sensitivity),
        .hyphen_prefix => textHyphenPrefix(actual, expected, case_sensitivity),
    };
}

fn textEquals(
    actual: []const u8,
    expected: []const u8,
    case_sensitivity: SelectorAttributeCaseSensitivity,
) bool {
    return switch (case_sensitivity) {
        .case_sensitive => std.mem.eql(u8, actual, expected),
        .case_insensitive => asciiEqualIgnoreCase(actual, expected),
    };
}

fn textStartsWith(
    actual: []const u8,
    expected: []const u8,
    case_sensitivity: SelectorAttributeCaseSensitivity,
) bool {
    if (actual.len < expected.len) return false;
    const prefix = actual[0..expected.len];
    return textEquals(prefix, expected, case_sensitivity);
}

fn textEndsWith(
    actual: []const u8,
    expected: []const u8,
    case_sensitivity: SelectorAttributeCaseSensitivity,
) bool {
    if (actual.len < expected.len) return false;
    const suffix = actual[actual.len - expected.len ..];
    return textEquals(suffix, expected, case_sensitivity);
}

fn textContains(
    actual: []const u8,
    expected: []const u8,
    case_sensitivity: SelectorAttributeCaseSensitivity,
) bool {
    if (expected.len == 0) return true;
    if (actual.len < expected.len) return false;

    var offset: usize = 0;
    while (offset + expected.len <= actual.len) : (offset += 1) {
        if (textEquals(actual[offset .. offset + expected.len], expected, case_sensitivity)) {
            return true;
        }
    }
    return false;
}

fn nodeIdEquals(left: NodeId, right: NodeId) bool {
    return left.index == right.index and left.generation == right.generation;
}

fn expectNodeIdSliceEquals(expected: []const NodeId, actual: []const NodeId) !void {
    try std.testing.expectEqual(expected.len, actual.len);
    for (expected, 0..) |expected_id, index| {
        try std.testing.expect(nodeIdEquals(expected_id, actual[index]));
    }
}

const SelectorFixtureBuilder = struct {
    allocator: std.mem.Allocator,
    html: std.ArrayList(u8) = .empty,
    next_node_index: u32 = 1,

    fn init(allocator: std.mem.Allocator) SelectorFixtureBuilder {
        return .{
            .allocator = allocator,
        };
    }

    fn deinit(self: *SelectorFixtureBuilder) void {
        self.html.deinit(self.allocator);
    }

    fn source(self: *const SelectorFixtureBuilder) []const u8 {
        return self.html.items;
    }

    fn startTag(
        self: *SelectorFixtureBuilder,
        tag: []const u8,
        id_value: ?[]const u8,
        class_value: ?[]const u8,
    ) errors.Result(NodeId) {
        try self.html.appendSlice(self.allocator, "<");
        try self.html.appendSlice(self.allocator, tag);
        if (id_value) |value| {
            try self.html.appendSlice(self.allocator, " id='");
            try self.html.appendSlice(self.allocator, value);
            try self.html.appendSlice(self.allocator, "'");
        }
        if (class_value) |value| {
            try self.html.appendSlice(self.allocator, " class='");
            try self.html.appendSlice(self.allocator, value);
            try self.html.appendSlice(self.allocator, "'");
        }
        try self.html.appendSlice(self.allocator, ">");

        const node_id = NodeId.new(self.next_node_index, 0);
        self.next_node_index += 1;
        return node_id;
    }

    fn endTag(self: *SelectorFixtureBuilder, tag: []const u8) errors.Result(void) {
        try self.html.appendSlice(self.allocator, "</");
        try self.html.appendSlice(self.allocator, tag);
        try self.html.appendSlice(self.allocator, ">");
    }

    fn openClose(
        self: *SelectorFixtureBuilder,
        tag: []const u8,
        id_value: ?[]const u8,
        class_value: ?[]const u8,
    ) errors.Result(NodeId) {
        const node_id = try self.startTag(tag, id_value, class_value);
        try self.endTag(tag);
        return node_id;
    }
};

fn textContainsWord(
    actual: []const u8,
    expected: []const u8,
    case_sensitivity: SelectorAttributeCaseSensitivity,
) bool {
    var index: usize = 0;
    while (index < actual.len) {
        while (index < actual.len and isHtmlWhitespace(actual[index])) {
            index += 1;
        }
        const start = index;
        while (index < actual.len and !isHtmlWhitespace(actual[index])) {
            index += 1;
        }
        if (index > start and textEquals(actual[start..index], expected, case_sensitivity)) {
            return true;
        }
    }
    return false;
}

fn textHyphenPrefix(
    actual: []const u8,
    expected: []const u8,
    case_sensitivity: SelectorAttributeCaseSensitivity,
) bool {
    if (textEquals(actual, expected, case_sensitivity)) return true;
    if (actual.len <= expected.len) return false;
    if (actual[expected.len] != '-') return false;
    return textStartsWith(actual, expected, case_sensitivity);
}

fn asciiEqualIgnoreCase(left: []const u8, right: []const u8) bool {
    if (left.len != right.len) return false;
    for (left, 0..) |byte, index| {
        if (std.ascii.toLower(byte) != std.ascii.toLower(right[index])) return false;
    }
    return true;
}

fn langMatchesRange(lang: []const u8, range: []const u8) bool {
    if (asciiEqualIgnoreCase(lang, range)) return true;
    if (lang.len <= range.len) return false;
    if (lang[range.len] != '-') return false;
    return asciiEqualIgnoreCase(lang[0..range.len], range);
}

test "phase one: bootstrapHtml builds a nested tree" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='app'><span>Hello</span><input disabled></main>");

    try std.testing.expectEqual(@as(usize, 5), store.nodeCount());
    const dumped = try store.dumpDom(allocator);
    defer allocator.free(dumped);
    try std.testing.expectEqualStrings(
        "#document\n  <main id=\"app\">\n    <span>\n      \"Hello\"\n    </span>\n    <input disabled />\n  </main>\n",
        dumped,
    );
}

test "phase one: malformed html is rejected explicitly" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try std.testing.expectError(error.HtmlParse, store.bootstrapHtml("<main><span></main>"));
}

test "phase one: selector subset matches ids, tags, and attributes" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='app' data-state='Ready'><span data-tags='Ready NOW'>Hello</span><input disabled></main>");

    const by_id = try store.select(allocator, "#app");
    defer allocator.free(by_id);
    try std.testing.expectEqual(@as(usize, 1), by_id.len);
    try std.testing.expectEqual(NodeId.new(1, 0), by_id[0]);

    const by_tag = try store.select(allocator, "main");
    defer allocator.free(by_tag);
    try std.testing.expectEqual(@as(usize, 1), by_tag.len);
    try std.testing.expectEqual(NodeId.new(1, 0), by_tag[0]);

    const by_universal = try store.select(allocator, "*");
    defer allocator.free(by_universal);
    try std.testing.expectEqual(@as(usize, 3), by_universal.len);
    try std.testing.expectEqual(NodeId.new(1, 0), by_universal[0]);
    try std.testing.expectEqual(NodeId.new(2, 0), by_universal[1]);
    try std.testing.expectEqual(NodeId.new(4, 0), by_universal[2]);

    const by_attr_exists = try store.select(allocator, "[disabled]");
    defer allocator.free(by_attr_exists);
    try std.testing.expectEqual(@as(usize, 1), by_attr_exists.len);
    try std.testing.expectEqual(NodeId.new(4, 0), by_attr_exists[0]);

    const by_attr_equals = try store.select(allocator, "[data-state=ready i]");
    defer allocator.free(by_attr_equals);
    try std.testing.expectEqual(@as(usize, 1), by_attr_equals.len);
    try std.testing.expectEqual(NodeId.new(1, 0), by_attr_equals[0]);

    const by_attr_word = try store.select(allocator, "[data-tags~=ready i]");
    defer allocator.free(by_attr_word);
    try std.testing.expectEqual(@as(usize, 1), by_attr_word.len);
    try std.testing.expectEqual(NodeId.new(2, 0), by_attr_word[0]);

    const by_list = try store.select(allocator, "input, main");
    defer allocator.free(by_list);
    try std.testing.expectEqual(@as(usize, 2), by_list.len);
    try std.testing.expectEqual(NodeId.new(1, 0), by_list[0]);
    try std.testing.expectEqual(NodeId.new(4, 0), by_list[1]);
}

test "property: selector combinators keep document-order subsets stable" {
    const allocator = std.testing.allocator;

    for (0..12) |case_index| {
        var fixture = SelectorFixtureBuilder.init(allocator);
        defer fixture.deinit();

        var expected_global: std.ArrayList(NodeId) = .empty;
        defer expected_global.deinit(allocator);
        var expected_main_descendant: std.ArrayList(NodeId) = .empty;
        defer expected_main_descendant.deinit(allocator);
        var expected_section_children: std.ArrayList(NodeId) = .empty;
        defer expected_section_children.deinit(allocator);

        const leading_noise_count = 1 + (case_index % 2);
        for (0..leading_noise_count) |noise_index| {
            const class_value = if ((case_index + noise_index) % 2 == 0) "action" else "secondary";
            const node_id = try fixture.openClose("button", null, class_value);
            if (std.mem.eql(u8, class_value, "action")) {
                try expected_global.append(allocator, node_id);
            }
        }

        _ = try fixture.startTag("main", "app", null);

        const section_count = 2 + (case_index % 3);
        for (0..section_count) |section_index| {
            _ = try fixture.startTag("section", null, "panel");
            if (((case_index + section_index) % 2) == 0) {
                const direct_id = try fixture.openClose("button", null, "action");
                try expected_global.append(allocator, direct_id);
                try expected_main_descendant.append(allocator, direct_id);
                try expected_section_children.append(allocator, direct_id);
            } else {
                _ = try fixture.startTag("div", null, null);
                const wrapped_id = try fixture.openClose("button", null, "action");
                try fixture.endTag("div");
                try expected_global.append(allocator, wrapped_id);
                try expected_main_descendant.append(allocator, wrapped_id);
            }
            try fixture.endTag("section");
        }

        try fixture.endTag("main");

        const trailing_noise_count = 1 + ((case_index + 1) % 3);
        for (0..trailing_noise_count) |noise_index| {
            const class_value = if ((case_index + noise_index) % 3 == 0) "action" else "secondary";
            const node_id = try fixture.openClose("button", null, class_value);
            if (std.mem.eql(u8, class_value, "action")) {
                try expected_global.append(allocator, node_id);
            }
        }

        var store = try DomStore.init(allocator);
        defer store.deinit();
        try store.bootstrapHtml(fixture.source());

        const action_nodes = try store.select(allocator, ".action");
        defer allocator.free(action_nodes);
        try expectNodeIdSliceEquals(expected_global.items, action_nodes);

        const main_nodes = try store.select(allocator, "main .action");
        defer allocator.free(main_nodes);
        try expectNodeIdSliceEquals(expected_main_descendant.items, main_nodes);

        const direct_nodes = try store.select(allocator, "main > section.panel > button.action");
        defer allocator.free(direct_nodes);
        try expectNodeIdSliceEquals(expected_section_children.items, direct_nodes);
    }
}

test "property: selector lists deduplicate overlapping matches" {
    const allocator = std.testing.allocator;

    for (0..12) |case_index| {
        var fixture = SelectorFixtureBuilder.init(allocator);
        defer fixture.deinit();

        var expected_action: std.ArrayList(NodeId) = .empty;
        defer expected_action.deinit(allocator);
        var expected_primary: std.ArrayList(NodeId) = .empty;
        defer expected_primary.deinit(allocator);
        var expected_union: std.ArrayList(NodeId) = .empty;
        defer expected_union.deinit(allocator);

        const leading_primary = try fixture.openClose("div", null, "primary");
        try expected_primary.append(allocator, leading_primary);
        try expected_union.append(allocator, leading_primary);

        _ = try fixture.startTag("main", "root", null);

        const slot_count = 4 + (case_index % 3);
        for (0..slot_count) |slot_index| {
            switch ((case_index + slot_index) % 4) {
                0 => {
                    const node_id = try fixture.openClose("button", null, "action");
                    try expected_action.append(allocator, node_id);
                    try expected_union.append(allocator, node_id);
                },
                1 => {
                    const node_id = try fixture.openClose("div", null, "primary");
                    try expected_primary.append(allocator, node_id);
                    try expected_union.append(allocator, node_id);
                },
                2 => {
                    const node_id = try fixture.openClose("button", null, "action primary");
                    try expected_action.append(allocator, node_id);
                    try expected_primary.append(allocator, node_id);
                    try expected_union.append(allocator, node_id);
                },
                3 => {
                    _ = try fixture.openClose("span", null, "secondary");
                },
                else => unreachable,
            }
        }

        try fixture.endTag("main");

        const trailing_both = try fixture.openClose("button", null, "action primary");
        try expected_action.append(allocator, trailing_both);
        try expected_primary.append(allocator, trailing_both);
        try expected_union.append(allocator, trailing_both);

        var store = try DomStore.init(allocator);
        defer store.deinit();
        try store.bootstrapHtml(fixture.source());

        const action_nodes = try store.select(allocator, "button.action");
        defer allocator.free(action_nodes);
        try expectNodeIdSliceEquals(expected_action.items, action_nodes);

        const primary_nodes = try store.select(allocator, ".primary");
        defer allocator.free(primary_nodes);
        try expectNodeIdSliceEquals(expected_primary.items, primary_nodes);

        const union_nodes = try store.select(allocator, "button.action, .primary");
        defer allocator.free(union_nodes);
        try expectNodeIdSliceEquals(expected_union.items, union_nodes);
    }
}

test "phase six: selector expansion matches classes and combinators" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml(
        "<main id='app' class='shell'><section class='panel'><button id='save' class='primary action'>Save</button></section><section class='panel'><button id='cancel' class='secondary action'>Cancel</button></section></main>",
    );

    const by_class = try store.select(allocator, ".primary");
    defer allocator.free(by_class);
    try std.testing.expectEqual(@as(usize, 1), by_class.len);
    try std.testing.expectEqual(NodeId.new(3, 0), by_class[0]);

    const by_tag_class = try store.select(allocator, "button.action");
    defer allocator.free(by_tag_class);
    try std.testing.expectEqual(@as(usize, 2), by_tag_class.len);
    try std.testing.expectEqual(NodeId.new(3, 0), by_tag_class[0]);
    try std.testing.expectEqual(NodeId.new(6, 0), by_tag_class[1]);

    const by_compound = try store.select(allocator, "#cancel.secondary.action");
    defer allocator.free(by_compound);
    try std.testing.expectEqual(@as(usize, 1), by_compound.len);
    try std.testing.expectEqual(NodeId.new(6, 0), by_compound[0]);

    const by_descendant = try store.select(allocator, "main .action");
    defer allocator.free(by_descendant);
    try std.testing.expectEqual(@as(usize, 2), by_descendant.len);
    try std.testing.expectEqual(NodeId.new(3, 0), by_descendant[0]);
    try std.testing.expectEqual(NodeId.new(6, 0), by_descendant[1]);

    const by_child = try store.select(allocator, "main > section.panel > button.action");
    defer allocator.free(by_child);
    try std.testing.expectEqual(@as(usize, 2), by_child.len);
    try std.testing.expectEqual(NodeId.new(3, 0), by_child[0]);
    try std.testing.expectEqual(NodeId.new(6, 0), by_child[1]);
}

test "phase six: selector expansion matches sibling combinators" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='root'><button id='first'>First</button><span id='gap'>Gap</span><button id='second'>Second</button><button id='third'>Third</button></main>");

    const adjacent = try store.select(allocator, "#first + span");
    defer allocator.free(adjacent);
    try std.testing.expectEqual(@as(usize, 1), adjacent.len);
    try std.testing.expectEqual(NodeId.new(4, 0), adjacent[0]);

    const adjacent_miss = try store.select(allocator, "#first + button");
    defer allocator.free(adjacent_miss);
    try std.testing.expectEqual(@as(usize, 0), adjacent_miss.len);

    const general = try store.select(allocator, "#first ~ button");
    defer allocator.free(general);
    try std.testing.expectEqual(@as(usize, 2), general.len);
    try std.testing.expectEqual(NodeId.new(6, 0), general[0]);
    try std.testing.expectEqual(NodeId.new(8, 0), general[1]);
}

test "phase six: selector expansion matches :has pseudo-class" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml(
        "<main id='root'><section id='first' class='child'>First</section><section id='child' class='child'><div id='grandchild' class='grandchild'>Grand</div></section></main>",
    );

    const by_descendant = try store.select(allocator, "main:has(section .grandchild)");
    defer allocator.free(by_descendant);
    try std.testing.expectEqual(@as(usize, 1), by_descendant.len);
    try std.testing.expectEqual(NodeId.new(1, 0), by_descendant[0]);

    const by_child = try store.select(allocator, "main:has(> .child)");
    defer allocator.free(by_child);
    try std.testing.expectEqual(@as(usize, 1), by_child.len);
    try std.testing.expectEqual(NodeId.new(1, 0), by_child[0]);

    const by_list = try store.select(allocator, "main:has(#missing, #child)");
    defer allocator.free(by_list);
    try std.testing.expectEqual(@as(usize, 1), by_list.len);
    try std.testing.expectEqual(NodeId.new(1, 0), by_list[0]);

    const section = store.findElementById("child").?;
    try std.testing.expect(try store.matchesSelector(allocator, section, ":has(.grandchild)"));
    try std.testing.expect(!try store.matchesSelector(allocator, section, ":has(.missing)"));
}

test "phase six: selector expansion matches :lang and :dir pseudo-classes" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml(
        "<main id='root' lang='en-US' dir='rtl'><section id='section'><span id='leaf'>Leaf</span></section><article id='article' xml:lang='fr'><span id='french'>Bonjour</span></article></main>",
    );

    const by_lang_list = try store.select(allocator, "section:lang(fr, en)");
    defer allocator.free(by_lang_list);
    try std.testing.expectEqual(@as(usize, 1), by_lang_list.len);
    try std.testing.expectEqual(NodeId.new(2, 0), by_lang_list[0]);

    const by_xml_lang = try store.select(allocator, "span:lang(fr)");
    defer allocator.free(by_xml_lang);
    try std.testing.expectEqual(@as(usize, 1), by_xml_lang.len);
    try std.testing.expectEqual(NodeId.new(6, 0), by_xml_lang[0]);

    const by_dir = try store.select(allocator, "section:dir(rtl)");
    defer allocator.free(by_dir);
    try std.testing.expectEqual(@as(usize, 1), by_dir.len);
    try std.testing.expectEqual(NodeId.new(2, 0), by_dir[0]);

    const not_dir = try store.select(allocator, "section:dir(ltr)");
    defer allocator.free(not_dir);
    try std.testing.expectEqual(@as(usize, 0), not_dir.len);

    const leaf = store.findElementById("leaf").?;
    try std.testing.expect(try store.matchesSelector(allocator, leaf, ":lang(en)"));
}

test "phase six: selector expansion matches structural and state pseudo-classes" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml(
        "<main id='root'><section id='buttons'><button id='first' class='primary' disabled>First</button><button id='second' class='secondary'>Second</button></section><div id='empty'></div><div id='not-empty'>x</div><input id='agree' type='checkbox' checked><input id='field' required><input id='optional'><input id='blank-input' value='   '><textarea id='bio' placeholder='Bio'></textarea><textarea id='blank-textarea'>   </textarea><div id='blank-editable' contenteditable='true'>   </div><a id='docs' href='/docs'>Docs</a><section id='types'><span id='only-span'>Span</span><button id='other'>Other</button></section></main>",
    );

    const root = try store.select(allocator, ":root");
    defer allocator.free(root);
    try std.testing.expectEqual(@as(usize, 1), root.len);
    try std.testing.expectEqualStrings("root", (try store.getAttribute(root[0], "id")).?);

    const first_child = try store.select(allocator, "#buttons > button:first-child");
    defer allocator.free(first_child);
    try std.testing.expectEqual(@as(usize, 1), first_child.len);
    try std.testing.expectEqualStrings("first", (try store.getAttribute(first_child[0], "id")).?);

    const last_child = try store.select(allocator, "#buttons > button:last-child");
    defer allocator.free(last_child);
    try std.testing.expectEqual(@as(usize, 1), last_child.len);
    try std.testing.expectEqualStrings("second", (try store.getAttribute(last_child[0], "id")).?);

    const first_of_type = try store.select(allocator, "#buttons > button:first-of-type");
    defer allocator.free(first_of_type);
    try std.testing.expectEqual(@as(usize, 1), first_of_type.len);
    try std.testing.expectEqualStrings("first", (try store.getAttribute(first_of_type[0], "id")).?);

    const last_of_type = try store.select(allocator, "#buttons > button:last-of-type");
    defer allocator.free(last_of_type);
    try std.testing.expectEqual(@as(usize, 1), last_of_type.len);
    try std.testing.expectEqualStrings("second", (try store.getAttribute(last_of_type[0], "id")).?);

    const only_of_type = try store.select(allocator, "#types > span:only-of-type");
    defer allocator.free(only_of_type);
    try std.testing.expectEqual(@as(usize, 1), only_of_type.len);
    try std.testing.expectEqualStrings("only-span", (try store.getAttribute(only_of_type[0], "id")).?);

    const only_child = try store.select(allocator, "#buttons > button:only-child");
    defer allocator.free(only_child);
    try std.testing.expectEqual(@as(usize, 0), only_child.len);

    const empty = try store.select(allocator, "#empty:empty");
    defer allocator.free(empty);
    try std.testing.expectEqual(@as(usize, 1), empty.len);
    try std.testing.expectEqualStrings("empty", (try store.getAttribute(empty[0], "id")).?);

    const not_empty = try store.select(allocator, "#not-empty:empty");
    defer allocator.free(not_empty);
    try std.testing.expectEqual(@as(usize, 0), not_empty.len);

    const checked = try store.select(allocator, "#agree:checked");
    defer allocator.free(checked);
    try std.testing.expectEqual(@as(usize, 1), checked.len);
    try std.testing.expectEqualStrings("agree", (try store.getAttribute(checked[0], "id")).?);

    const disabled = try store.select(allocator, "#first:disabled");
    defer allocator.free(disabled);
    try std.testing.expectEqual(@as(usize, 1), disabled.len);
    try std.testing.expectEqualStrings("first", (try store.getAttribute(disabled[0], "id")).?);

    const enabled = try store.select(allocator, "#second:enabled");
    defer allocator.free(enabled);
    try std.testing.expectEqual(@as(usize, 1), enabled.len);
    try std.testing.expectEqualStrings("second", (try store.getAttribute(enabled[0], "id")).?);

    const required = try store.select(allocator, "#field:required");
    defer allocator.free(required);
    try std.testing.expectEqual(@as(usize, 1), required.len);
    try std.testing.expectEqualStrings("field", (try store.getAttribute(required[0], "id")).?);

    const optional = try store.select(allocator, "#optional:optional");
    defer allocator.free(optional);
    try std.testing.expectEqual(@as(usize, 1), optional.len);
    try std.testing.expectEqualStrings("optional", (try store.getAttribute(optional[0], "id")).?);

    const link = try store.select(allocator, "#docs:link");
    defer allocator.free(link);
    try std.testing.expectEqual(@as(usize, 1), link.len);
    try std.testing.expectEqualStrings("docs", (try store.getAttribute(link[0], "id")).?);

    const any_link = try store.select(allocator, "#docs:any-link");
    defer allocator.free(any_link);
    try std.testing.expectEqual(@as(usize, 1), any_link.len);
    try std.testing.expectEqualStrings("docs", (try store.getAttribute(any_link[0], "id")).?);

    const placeholder_shown = try store.select(allocator, "#bio:placeholder-shown");
    defer allocator.free(placeholder_shown);
    try std.testing.expectEqual(@as(usize, 1), placeholder_shown.len);
    try std.testing.expectEqualStrings("bio", (try store.getAttribute(placeholder_shown[0], "id")).?);

    const blank_input = try store.select(allocator, "#blank-input:blank");
    defer allocator.free(blank_input);
    try std.testing.expectEqual(@as(usize, 1), blank_input.len);
    try std.testing.expectEqualStrings("blank-input", (try store.getAttribute(blank_input[0], "id")).?);

    const blank_textarea = try store.select(allocator, "#blank-textarea:blank");
    defer allocator.free(blank_textarea);
    try std.testing.expectEqual(@as(usize, 1), blank_textarea.len);
    try std.testing.expectEqualStrings("blank-textarea", (try store.getAttribute(blank_textarea[0], "id")).?);

    const blank_editable = try store.select(allocator, "#blank-editable:blank");
    defer allocator.free(blank_editable);
    try std.testing.expectEqual(@as(usize, 1), blank_editable.len);
    try std.testing.expectEqualStrings("blank-editable", (try store.getAttribute(blank_editable[0], "id")).?);

    const filled_blank = try store.select(allocator, "#filled:blank");
    defer allocator.free(filled_blank);
    try std.testing.expectEqual(@as(usize, 0), filled_blank.len);

    const is_buttons = try store.select(allocator, "#buttons > button:is(.primary, .secondary)");
    defer allocator.free(is_buttons);
    try std.testing.expectEqual(@as(usize, 2), is_buttons.len);
    try std.testing.expectEqualStrings("first", (try store.getAttribute(is_buttons[0], "id")).?);
    try std.testing.expectEqualStrings("second", (try store.getAttribute(is_buttons[1], "id")).?);

    const where_buttons = try store.select(allocator, "#buttons > button:where(.primary, .secondary)");
    defer allocator.free(where_buttons);
    try std.testing.expectEqual(@as(usize, 2), where_buttons.len);
    try std.testing.expectEqualStrings("first", (try store.getAttribute(where_buttons[0], "id")).?);
    try std.testing.expectEqualStrings("second", (try store.getAttribute(where_buttons[1], "id")).?);

    const not_buttons = try store.select(allocator, "#buttons > button:not(.missing, .secondary)");
    defer allocator.free(not_buttons);
    try std.testing.expectEqual(@as(usize, 1), not_buttons.len);
    try std.testing.expectEqualStrings("first", (try store.getAttribute(not_buttons[0], "id")).?);
}

test "phase six: selector expansion matches defined pseudo-class" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml(
        "<main id='root'><x-widget id='widget'></x-widget><svg id='svg'><text id='svg-text'>Hi</text></svg></main>",
    );

    const defined = try store.select(allocator, ":defined");
    defer allocator.free(defined);
    try std.testing.expectEqual(@as(usize, 3), defined.len);
    try std.testing.expectEqualStrings("root", (try store.getAttribute(defined[0], "id")).?);
    try std.testing.expectEqualStrings("svg", (try store.getAttribute(defined[1], "id")).?);
    try std.testing.expectEqualStrings("svg-text", (try store.getAttribute(defined[2], "id")).?);

    const widget_defined = try store.select(allocator, "#widget:defined");
    defer allocator.free(widget_defined);
    try std.testing.expectEqual(@as(usize, 0), widget_defined.len);

    const svg_defined = try store.select(allocator, "#svg:defined");
    defer allocator.free(svg_defined);
    try std.testing.expectEqual(@as(usize, 1), svg_defined.len);
    try std.testing.expectEqualStrings("svg", (try store.getAttribute(svg_defined[0], "id")).?);
}

test "phase six: selector expansion matches default and indeterminate pseudo-classes" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml(
        "<main id='root'><progress id='loading'></progress><form id='signup'><input type='radio' name='mode' id='mode-a'><input type='radio' name='mode' id='mode-b'></form><form id='chosen'><input type='radio' name='picked' id='picked-a' checked><input type='radio' name='picked' id='picked-b'></form><form id='form'><input id='submit' type='submit'><input id='agree' type='checkbox' checked><input id='mode-c' type='radio' name='mode2'><input id='mode-d' type='radio' name='mode2' checked><select id='select'><option id='first' value='a'>A</option><option id='selected' value='b' selected>B</option></select></form></main>",
    );

    const defaults = try store.select(allocator, ":default");
    defer allocator.free(defaults);
    try std.testing.expectEqual(@as(usize, 5), defaults.len);
    try std.testing.expectEqualStrings("picked-a", (try store.getAttribute(defaults[0], "id")).?);
    try std.testing.expectEqualStrings("submit", (try store.getAttribute(defaults[1], "id")).?);
    try std.testing.expectEqualStrings("agree", (try store.getAttribute(defaults[2], "id")).?);
    try std.testing.expectEqualStrings("mode-d", (try store.getAttribute(defaults[3], "id")).?);
    try std.testing.expectEqualStrings("selected", (try store.getAttribute(defaults[4], "id")).?);

    const indeterminate = try store.select(allocator, ":indeterminate");
    defer allocator.free(indeterminate);
    try std.testing.expectEqual(@as(usize, 3), indeterminate.len);
    try std.testing.expectEqualStrings("loading", (try store.getAttribute(indeterminate[0], "id")).?);
    try std.testing.expectEqualStrings("mode-a", (try store.getAttribute(indeterminate[1], "id")).?);
    try std.testing.expectEqualStrings("mode-b", (try store.getAttribute(indeterminate[2], "id")).?);
}

test "phase six: selector expansion matches read-only and read-write pseudo-classes" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml(
        "<main id='root'><input id='name' value='Ada'><input id='readonly' value='Bee' readonly><textarea id='bio'>Hello</textarea><div id='editable' contenteditable='true'>Edit</div><select id='mode'><option id='option' value='a'>A</option></select><button id='button'>Button</button></main>",
    );

    const read_write = try store.select(allocator, ":read-write");
    defer allocator.free(read_write);
    try std.testing.expectEqual(@as(usize, 3), read_write.len);
    try std.testing.expectEqualStrings("name", (try store.getAttribute(read_write[0], "id")).?);
    try std.testing.expectEqualStrings("bio", (try store.getAttribute(read_write[1], "id")).?);
    try std.testing.expectEqualStrings("editable", (try store.getAttribute(read_write[2], "id")).?);

    const read_only = try store.select(allocator, ":read-only");
    defer allocator.free(read_only);
    try std.testing.expectEqual(@as(usize, 5), read_only.len);
    try std.testing.expectEqualStrings("root", (try store.getAttribute(read_only[0], "id")).?);
    try std.testing.expectEqualStrings("readonly", (try store.getAttribute(read_only[1], "id")).?);
    try std.testing.expectEqualStrings("mode", (try store.getAttribute(read_only[2], "id")).?);
    try std.testing.expectEqualStrings("option", (try store.getAttribute(read_only[3], "id")).?);
    try std.testing.expectEqualStrings("button", (try store.getAttribute(read_only[4], "id")).?);
}

test "phase six: selector expansion matches validation and range pseudo-classes" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml(
        "<main id='root'><input id='filled' type='text' required value='Ada'><input id='empty' type='text' required><input id='check' type='checkbox' required><input id='check-ok' type='checkbox' required checked><input id='low' type='number' min='2' max='6' value='1'><input id='high' type='number' min='2' max='6' value='7'><input id='in-range' type='number' min='2' max='6' value='4'><textarea id='bio' required></textarea><select id='mode' required><option value='a' selected>A</option><option value='b'>B</option></select><button id='button'>Button</button></main>",
    );

    const valid = try store.select(allocator, ":valid");
    defer allocator.free(valid);
    try std.testing.expectEqual(@as(usize, 4), valid.len);
    try std.testing.expectEqualStrings("filled", (try store.getAttribute(valid[0], "id")).?);
    try std.testing.expectEqualStrings("check-ok", (try store.getAttribute(valid[1], "id")).?);
    try std.testing.expectEqualStrings("in-range", (try store.getAttribute(valid[2], "id")).?);
    try std.testing.expectEqualStrings("mode", (try store.getAttribute(valid[3], "id")).?);

    const invalid = try store.select(allocator, ":invalid");
    defer allocator.free(invalid);
    try std.testing.expectEqual(@as(usize, 5), invalid.len);
    try std.testing.expectEqualStrings("empty", (try store.getAttribute(invalid[0], "id")).?);
    try std.testing.expectEqualStrings("check", (try store.getAttribute(invalid[1], "id")).?);
    try std.testing.expectEqualStrings("low", (try store.getAttribute(invalid[2], "id")).?);
    try std.testing.expectEqualStrings("high", (try store.getAttribute(invalid[3], "id")).?);
    try std.testing.expectEqualStrings("bio", (try store.getAttribute(invalid[4], "id")).?);

    const in_range = try store.select(allocator, ":in-range");
    defer allocator.free(in_range);
    try std.testing.expectEqual(@as(usize, 1), in_range.len);
    try std.testing.expectEqualStrings("in-range", (try store.getAttribute(in_range[0], "id")).?);

    const out_of_range = try store.select(allocator, ":out-of-range");
    defer allocator.free(out_of_range);
    try std.testing.expectEqual(@as(usize, 2), out_of_range.len);
    try std.testing.expectEqualStrings("low", (try store.getAttribute(out_of_range[0], "id")).?);
    try std.testing.expectEqualStrings("high", (try store.getAttribute(out_of_range[1], "id")).?);
}

test "phase six: selector expansion matches scope pseudo-class" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='root'><section id='section'><div id='child'>Child</div></section></main>");

    const doc_scope = try store.querySelector(allocator, ":scope");
    try std.testing.expect(doc_scope != null);
    try std.testing.expectEqualStrings("root", (try store.getAttribute(doc_scope.?, "id")).?);

    const root = store.findElementById("root").?;
    const section = try store.querySelectorWithin(allocator, root, ":scope > section");
    try std.testing.expect(section != null);
    try std.testing.expectEqualStrings("section", (try store.getAttribute(section.?, "id")).?);

    const missing = try store.querySelectorWithin(allocator, root, ":scope");
    try std.testing.expect(missing == null);

    try std.testing.expect(try store.matchesSelector(allocator, root, ":scope"));

    const child = store.findElementById("child").?;
    const closest = try store.closestSelector(allocator, child, ":scope");
    try std.testing.expect(closest != null);
    try std.testing.expectEqualStrings("child", (try store.getAttribute(closest.?, "id")).?);
}

test "phase six: selector expansion matches focus target and nth pseudo-classes" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='root'><section id='panel'><input id='field'></section><section id='list'><span id='a' class='match'>A</span><div id='b'>B</div><span id='c'>C</span><div id='d' class='match'>D</div></section><a id='fallback' name='named'>Target</a></main>");

    const field = store.findElementById("field").?;
    const panel = store.findElementById("panel").?;
    const root = store.findElementById("root").?;
    const fallback = store.findElementById("fallback").?;
    store.setFocusedNode(field);
    try store.setTargetFragment("named");

    const focus = try store.select(allocator, ":focus");
    defer allocator.free(focus);
    try std.testing.expectEqual(@as(usize, 1), focus.len);
    try std.testing.expectEqual(field, focus[0]);

    const focus_visible = try store.select(allocator, ":focus-visible");
    defer allocator.free(focus_visible);
    try std.testing.expectEqual(@as(usize, 1), focus_visible.len);
    try std.testing.expectEqual(field, focus_visible[0]);

    const focus_within = try store.select(allocator, "#panel:focus-within");
    defer allocator.free(focus_within);
    try std.testing.expectEqual(@as(usize, 1), focus_within.len);
    try std.testing.expectEqual(panel, focus_within[0]);

    const root_focus_within = try store.select(allocator, "#root:focus-within");
    defer allocator.free(root_focus_within);
    try std.testing.expectEqual(@as(usize, 1), root_focus_within.len);
    try std.testing.expectEqual(root, root_focus_within[0]);

    const target = try store.select(allocator, ":target");
    defer allocator.free(target);
    try std.testing.expectEqual(@as(usize, 1), target.len);
    try std.testing.expectEqual(fallback, target[0]);

    const nth_child = try store.select(allocator, "#list > span:nth-child(1)");
    defer allocator.free(nth_child);
    try std.testing.expectEqual(@as(usize, 1), nth_child.len);
    try std.testing.expectEqualStrings("a", (try store.getAttribute(nth_child[0], "id")).?);

    const nth_last_child = try store.select(allocator, "#list > span:nth-last-child(2)");
    defer allocator.free(nth_last_child);
    try std.testing.expectEqual(@as(usize, 1), nth_last_child.len);
    try std.testing.expectEqualStrings("c", (try store.getAttribute(nth_last_child[0], "id")).?);

    const nth_of_type = try store.select(allocator, "#list > div:nth-of-type(2)");
    defer allocator.free(nth_of_type);
    try std.testing.expectEqual(@as(usize, 1), nth_of_type.len);
    try std.testing.expectEqualStrings("d", (try store.getAttribute(nth_of_type[0], "id")).?);

    const nth_of_type_filtered = try store.select(allocator, "#list > span:nth-of-type(1 of .match)");
    defer allocator.free(nth_of_type_filtered);
    try std.testing.expectEqual(@as(usize, 1), nth_of_type_filtered.len);
    try std.testing.expectEqualStrings("a", (try store.getAttribute(nth_of_type_filtered[0], "id")).?);

    const nth_child_filtered = try store.select(allocator, "#list > .match:nth-child(1 of .match)");
    defer allocator.free(nth_child_filtered);
    try std.testing.expectEqual(@as(usize, 1), nth_child_filtered.len);
    try std.testing.expectEqualStrings("a", (try store.getAttribute(nth_child_filtered[0], "id")).?);

    const nth_last_child_filtered = try store.select(allocator, "#list > .match:nth-last-child(1 of .match)");
    defer allocator.free(nth_last_child_filtered);
    try std.testing.expectEqual(@as(usize, 1), nth_last_child_filtered.len);
    try std.testing.expectEqualStrings("d", (try store.getAttribute(nth_last_child_filtered[0], "id")).?);

    const nth_last_of_type_filtered = try store.select(allocator, "#list > div:nth-last-of-type(1 of .match)");
    defer allocator.free(nth_last_of_type_filtered);
    try std.testing.expectEqual(@as(usize, 1), nth_last_of_type_filtered.len);
    try std.testing.expectEqualStrings("d", (try store.getAttribute(nth_last_of_type_filtered[0], "id")).?);
}

test "phase seven: selector single-node helpers match and climb ancestors" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='root'><section id='panel' class='panel'><span id='marker'>panel</span><button id='second' class='secondary'>Second</button></section></main>");

    const second = store.findElementById("second").?;
    try std.testing.expect(try store.matchesSelector(allocator, second, "button.secondary"));
    try std.testing.expect(!(try store.matchesSelector(allocator, second, "button.primary")));

    const closest = try store.closestSelector(allocator, second, "section.panel");
    try std.testing.expectEqual(store.findElementById("panel").?, closest.?);
}

test "phase seven: selector collection helpers stay within the subtree" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='root'><button id='first' class='action'>First</button><section><button id='second' class='action'>Second</button></section></main>");

    const root = store.findElementById("root").?;
    const within = try store.selectWithin(allocator, root, "main, button.action");
    defer allocator.free(within);

    try std.testing.expectEqual(@as(usize, 2), within.len);
    try std.testing.expectEqual(NodeId.new(2, 0), within[0]);
    try std.testing.expectEqual(NodeId.new(5, 0), within[1]);
}

test "phase eight: attribute reflection updates selectors and form state" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='root'><button id='button'>First</button><input id='name'><input id='agree' type='checkbox'><select id='mode'><option value='a'>A</option><option id='selected' value='b'>B</option></select></main>");

    const button = store.findElementById("button").?;
    try std.testing.expectEqual(@as(?[]const u8, null), try store.getAttribute(button, "data-label"));
    try store.setAttribute(button, "class", "primary");
    try std.testing.expect(try store.hasAttribute(button, "class"));
    try std.testing.expect(try store.toggleAttribute(button, "data-flag", null));

    const class_nodes = try store.select(allocator, ".primary");
    defer allocator.free(class_nodes);
    try std.testing.expectEqual(@as(usize, 1), class_nodes.len);

    const flag_nodes = try store.select(allocator, "[data-flag]");
    defer allocator.free(flag_nodes);
    try std.testing.expectEqual(@as(usize, 1), flag_nodes.len);

    try std.testing.expect(!(try store.toggleAttribute(button, "data-flag", false)));
    try std.testing.expect(!(try store.hasAttribute(button, "data-flag")));

    try store.setAttribute(button, "data-label", "Hello");
    try std.testing.expectEqualStrings("Hello", (try store.getAttribute(button, "data-label")).?);
    try store.removeAttribute(button, "data-label");
    try std.testing.expectEqual(@as(?[]const u8, null), try store.getAttribute(button, "data-label"));

    const name = store.findElementById("name").?;
    try store.setAttribute(name, "value", "Alice");
    const name_value = try store.valueForNode(allocator, name);
    defer allocator.free(name_value);
    try std.testing.expectEqualStrings("Alice", name_value);

    const agree = store.findElementById("agree").?;
    try store.setAttribute(agree, "checked", "");
    try std.testing.expectEqual(@as(?bool, true), store.checkedForNode(agree));

    const selected = store.findElementById("selected").?;
    try store.setAttribute(selected, "selected", "");
    const mode = store.findElementById("mode").?;
    const mode_value = try store.valueForNode(allocator, mode);
    defer allocator.free(mode_value);
    try std.testing.expectEqualStrings("b", mode_value);
}

test "phase eight: tree mutation primitives preserve order in DomStore" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='root'><section id='target'><span id='placeholder'>Placeholder</span></section><button id='first'>First</button><button id='second'>Second</button><button id='third'>Third</button></main>");

    const target = store.findElementById("target").?;
    const first = store.findElementById("first").?;
    const second = store.findElementById("second").?;
    const third = store.findElementById("third").?;
    const placeholder = store.findElementById("placeholder").?;

    try store.appendChildren(target, &.{ first, second });
    try store.prependChildren(target, &.{third});
    try store.replaceChildren(target, &.{ first, placeholder, second });

    const target_text = try store.textContent(allocator, target);
    defer allocator.free(target_text);
    try std.testing.expectEqualStrings("FirstPlaceholderSecond", target_text);

    const target_buttons = try store.select(allocator, "#target > button");
    defer allocator.free(target_buttons);
    try std.testing.expectEqual(@as(usize, 2), target_buttons.len);
    try std.testing.expectEqual(first, target_buttons[0]);
    try std.testing.expectEqual(second, target_buttons[1]);

    const placeholder_nodes = try store.select(allocator, "#target > #placeholder");
    defer allocator.free(placeholder_nodes);
    try std.testing.expectEqual(@as(usize, 1), placeholder_nodes.len);
}

test "phase eight: HTML serialization surfaces round-trip fragments in DomStore" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section><div id='out'></div><script id='script'>const raw = \"<span id='first'>One</span><span id='second'>Two</span>\";</script></main>");

    const target = store.findElementById("target").?;
    const script = store.findElementById("script").?;

    const before = try store.innerHtml(allocator, target);
    defer allocator.free(before);
    try std.testing.expectEqualStrings("<button class=\"primary\" id=\"old\">Old</button>", before);

    const script_html = try store.innerHtml(allocator, script);
    defer allocator.free(script_html);
    try std.testing.expectEqualStrings("const raw = \"<span id='first'>One</span><span id='second'>Two</span>\";", script_html);

    try store.setInnerHtml(target, "<span id=\"first\">One</span><span id=\"second\">Two</span>");

    const after = try store.innerHtml(allocator, target);
    defer allocator.free(after);
    try std.testing.expectEqualStrings("<span id=\"first\">One</span><span id=\"second\">Two</span>", after);

    const outer = try store.outerHtml(allocator, target);
    defer allocator.free(outer);
    try std.testing.expectEqualStrings("<section id=\"target\"><span id=\"first\">One</span><span id=\"second\">Two</span></section>", outer);

    const old_nodes = try store.select(allocator, "#old");
    defer allocator.free(old_nodes);
    try std.testing.expectEqual(@as(usize, 0), old_nodes.len);

    try store.setOuterHtml(target, "<article id=\"replacement\"><em id=\"inner\">Inner</em></article>");

    const replacement = store.findElementById("replacement").?;
    const replacement_html = try store.outerHtml(allocator, replacement);
    defer allocator.free(replacement_html);
    try std.testing.expectEqualStrings("<article id=\"replacement\"><em id=\"inner\">Inner</em></article>", replacement_html);

    const replacement_nodes = try store.select(allocator, "#replacement");
    defer allocator.free(replacement_nodes);
    try std.testing.expectEqual(@as(usize, 1), replacement_nodes.len);

    const inner_nodes = try store.select(allocator, "#inner");
    defer allocator.free(inner_nodes);
    try std.testing.expectEqual(@as(usize, 1), inner_nodes.len);
}

test "phase eight: HTML serialization surfaces support insertAdjacentHTML positions in DomStore" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section></main>");

    const target = store.findElementById("target").?;
    try store.insertAdjacentHtml(target, "beforebegin", "<aside id=\"before\">Before</aside>");
    try store.insertAdjacentHtml(target, "afterbegin", "<span id=\"first\">First</span>");
    try store.insertAdjacentHtml(target, "beforeend", "<span id=\"last\">Last</span>");
    try store.insertAdjacentHtml(target, "afterend", "<aside id=\"after\">After</aside>");

    const root = store.findElementById("root").?;
    const root_html = try store.innerHtml(allocator, root);
    defer allocator.free(root_html);
    try std.testing.expectEqualStrings(
        "<aside id=\"before\">Before</aside><section id=\"target\"><span id=\"first\">First</span><button class=\"primary\" id=\"old\">Old</button><span id=\"last\">Last</span></section><aside id=\"after\">After</aside>",
        root_html,
    );

    const target_html = try store.innerHtml(allocator, target);
    defer allocator.free(target_html);
    try std.testing.expectEqualStrings(
        "<span id=\"first\">First</span><button class=\"primary\" id=\"old\">Old</button><span id=\"last\">Last</span>",
        target_html,
    );

    const before_nodes = try store.select(allocator, "#before");
    defer allocator.free(before_nodes);
    try std.testing.expectEqual(@as(usize, 1), before_nodes.len);

    const after_nodes = try store.select(allocator, "#after");
    defer allocator.free(after_nodes);
    try std.testing.expectEqual(@as(usize, 1), after_nodes.len);
}

test "phase eight: HTML serialization surfaces use namespace-aware names in DomStore" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='root'><svg id='icon' viewbox='0 0 10 10'><foreignobject id='foreign'><div id='html'>Text</div></foreignobject></svg><math id='formula' definitionurl='https://example.com'><mi id='symbol'>x</mi></math></main>");

    const icon = store.findElementById("icon").?;
    const icon_html = try store.outerHtml(allocator, icon);
    defer allocator.free(icon_html);
    try std.testing.expectEqualStrings(
        "<svg id=\"icon\" viewBox=\"0 0 10 10\"><foreignObject id=\"foreign\"><div id=\"html\">Text</div></foreignObject></svg>",
        icon_html,
    );

    const formula = store.findElementById("formula").?;
    const formula_html = try store.outerHtml(allocator, formula);
    defer allocator.free(formula_html);
    try std.testing.expectEqualStrings(
        "<math definitionURL=\"https://example.com\" id=\"formula\"><mi id=\"symbol\">x</mi></math>",
        formula_html,
    );
}

test "failure: HTML serialization surfaces reject malformed fragments in DomStore" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='root'><section id='target'></section></main>");

    const target = store.findElementById("target").?;
    try std.testing.expectError(error.HtmlParse, store.setInnerHtml(target, "<span></main>"));
}

test "failure: HTML serialization surfaces reject insertAdjacentHTML positions in DomStore" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='root'><section id='target'></section></main>");

    const target = store.findElementById("target").?;
    try std.testing.expectError(error.DomError, store.insertAdjacentHtml(target, "middle", "<span id='bad'>Bad</span>"));
}

test "failure: HTML serialization surfaces reject insertAdjacentHTML on void elements in DomStore" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='root'><img id='image'><section id='target'></section></main>");

    const image = store.findElementById("image").?;
    try std.testing.expectError(error.DomError, store.insertAdjacentHtml(image, "beforeend", "<span id='bad'>Bad</span>"));
}

test "failure: HTML serialization surfaces reject detached insertAdjacentHTML in DomStore" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='root'><section id='target'><span id='old'>Old</span></section></main>");

    const target = store.findElementById("target").?;
    try store.setOuterHtml(target, "<section id=\"replacement\"></section>");
    try std.testing.expectError(error.DomError, store.insertAdjacentHtml(target, "beforebegin", "<aside id='before'>Before</aside>"));
}

test "contract: HTML serialization surfaces escape mixed-quote attribute values in DomStore" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='root'><div id='target'></div></main>");

    const target = store.findElementById("target").?;
    try store.setAttribute(target, "data-label", "a'b\"c&d<e>");
    const html = try store.outerHtml(allocator, target);
    defer allocator.free(html);
    try std.testing.expectEqualStrings(
        "<div data-label=\"a'b&quot;c&amp;d&lt;e&gt;\" id=\"target\"></div>",
        html,
    );
}

test "failure: tree mutation rejects ancestor cycles in DomStore" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='root'><section id='child'><span id='grandchild'>x</span></section></main>");

    const root = store.findElementById("root").?;
    const child = store.findElementById("child").?;
    try std.testing.expectError(error.DomError, store.appendChild(child, root));
}

test "phase eight: empty attribute names are rejected explicitly" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='root'><button id='button'>First</button></main>");

    const button = store.findElementById("button").?;
    try std.testing.expectError(error.DomError, store.setAttribute(button, "   ", "x"));
    try std.testing.expectError(error.DomError, store.getAttribute(button, ""));
    try std.testing.expectError(error.DomError, store.removeAttribute(button, ""));
    try std.testing.expectError(error.DomError, store.toggleAttribute(button, "", null));
}

test "phase one: unsupported selector syntax is rejected explicitly" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='app'><span>Hello</span></main>");

    try std.testing.expectError(error.HtmlParse, store.select(allocator, "main::before"));
    try std.testing.expectError(error.HtmlParse, store.select(allocator, "[data-state"));
    try std.testing.expectError(error.HtmlParse, store.select(allocator, ""));
}
