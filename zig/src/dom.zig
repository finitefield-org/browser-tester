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
    attributes: std.ArrayListUnmanaged(Attribute) = .{},
};

pub const NodeKind = union(enum) {
    document,
    element: ElementData,
    text: []const u8,
    comment: []const u8,
};

const SerializationNamespace = enum {
    html,
    svg,
    mathml,
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
    tag: ?[]const u8 = null,
    id: ?[]const u8 = null,
    classes: std.ArrayListUnmanaged([]const u8) = .{},
    attributes: std.ArrayListUnmanaged(SelectorAttribute) = .{},

    fn deinit(self: *SelectorQuery, allocator: std.mem.Allocator) void {
        self.classes.deinit(allocator);
        self.attributes.deinit(allocator);
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

pub const DomStore = struct {
    allocator: std.mem.Allocator,
    arena: std.heap.ArenaAllocator,
    nodes: std.ArrayListUnmanaged(NodeRecord) = .{},
    source_html: ?[]const u8 = null,

    pub fn init(allocator: std.mem.Allocator) errors.Result(DomStore) {
        var arena = std.heap.ArenaAllocator.init(allocator);
        errdefer arena.deinit();

        var store = DomStore{
            .allocator = allocator,
            .arena = arena,
            .nodes = .{},
            .source_html = null,
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
        self.arena.deinit();
    }

    pub fn documentId(self: *const DomStore) NodeId {
        _ = self;
        return NodeId.new(0, 0);
    }

    pub fn sourceHtml(self: *const DomStore) ?[]const u8 {
        return self.source_html;
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

    pub fn tagNameForNode(self: *const DomStore, node_id: NodeId) ?[]const u8 {
        const node = self.nodeAt(node_id) orelse return null;
        return switch (node.kind) {
            .element => |element| element.tag_name,
            else => null,
        };
    }

    pub fn namespaceUriForNode(self: *const DomStore, node_id: NodeId) ?[]const u8 {
        if (self.nodeAt(node_id) == null) return null;

        return switch (self.serializationNamespaceForNode(node_id)) {
            .html => "http://www.w3.org/1999/xhtml",
            .svg => "http://www.w3.org/2000/svg",
            .mathml => "http://www.w3.org/1998/Math/MathML",
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

        try collectSelectorMatchesWithin(self, root_id, chains.items, &results, allocator);

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

        return self.findFirstMatchingDescendant(root_id, chains.items);
    }

    pub fn matchesSelector(
        self: *const DomStore,
        allocator: std.mem.Allocator,
        node_id: NodeId,
        selector: []const u8,
    ) errors.Result(bool) {
        var chains = try self.parseSelectorChains(allocator, selector);
        defer self.deinitSelectorChains(allocator, &chains);

        return nodeMatchesAnyChain(self, node_id, chains.items);
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
            if (nodeMatchesAnyChain(self, current, chains.items)) {
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

        try self.cloneFragmentChildrenInto(&fragment_store, fragment_children, node_id, 0);
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
        try self.cloneFragmentChildrenInto(&fragment_store, fragment_children, parent_id, insertion_index);
        return;
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
            try self.cloneFragmentChildrenInto(&fragment_store, fragment_children, parent_id, insertion_index);
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
            try self.cloneFragmentChildrenInto(&fragment_store, fragment_children, node_id, 0);
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
            try self.cloneFragmentChildrenInto(&fragment_store, fragment_children, node_id, insertion_index);
            return;
        }

        if (std.mem.eql(u8, position, "afterend")) {
            const parent_id = parentOf(self, node_id) orelse return error.DomError;
            const insertion_index = try self.childIndex(parent_id, node_id) + 1;

            var fragment_store = try DomStore.init(self.allocator);
            defer fragment_store.deinit();

            const fragment_root = try self.parseHtmlFragmentIntoStore(&fragment_store, parent_id, html);
            const fragment_children = fragment_store.childIds(fragment_root);
            try self.cloneFragmentChildrenInto(&fragment_store, fragment_children, parent_id, insertion_index);
            return;
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
            },
            else => return error.HtmlParse,
        }
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
            element.attributes.items[index].value = value_copy;
            return;
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
                    _ = element.attributes.orderedRemove(found);
                }
                return false;
            }

            if (index == null) {
                try element.attributes.append(arena_alloc, .{
                    .name = try duplicateLowercase(self, trimmed),
                    .value = try duplicateString(self, ""),
                });
            }
            return true;
        }

        if (index) |found| {
            _ = element.attributes.orderedRemove(found);
            return false;
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

        if (std.mem.eql(u8, element.tag_name, "input") and isTextInputType(elementAttributeValue(element.*, "type"))) {
            const arena_alloc = self.arena.allocator();
            try upsertAttribute(arena_alloc, &element.attributes, "value", try duplicateString(self, value));
            return;
        }

        if (std.mem.eql(u8, element.tag_name, "input") and isCheckableInputType(elementAttributeValue(element.*, "type"))) {
            return error.DomError;
        }

        if (std.mem.eql(u8, element.tag_name, "select")) {
            return error.DomError;
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
        return;
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

        const record = self.nodeAtMut(node_id) orelse return error.DomError;
        record.parent = null;
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

    fn parseHtmlFragmentIntoStore(
        self: *const DomStore,
        fragment_store: *DomStore,
        context_parent: NodeId,
        html: []const u8,
    ) errors.Result(NodeId) {
        const context_tag = try self.fragmentContextTagName(context_parent);
        const context_tag_copy = try duplicateString(fragment_store, context_tag);
        const fragment_root = try fragment_store.addElement(fragment_store.documentId(), context_tag_copy, .{});

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
    ) errors.Result(void) {
        var next_index = insertion_index;
        for (fragment_children) |child_id| {
            _ = try self.cloneSubtreeAt(fragment_store, child_id, parent, next_index);
            next_index += 1;
        }
    }

    fn cloneSubtreeAt(
        self: *DomStore,
        source: *const DomStore,
        source_node_id: NodeId,
        parent: NodeId,
        insertion_index: usize,
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
                const node_id = try self.addElement(parent, try duplicateString(self, element.tag_name), attributes);
                break :blk node_id;
            },
        };

        try self.moveAppendedChildToIndex(parent, created, insertion_index);

        if (source_node.kind == .element) {
            const source_children = source_node.children.items;
            var child_index: usize = 0;
            while (child_index < source_children.len) : (child_index += 1) {
                _ = try self.cloneSubtreeAt(source, source_children[child_index], created, child_index);
            }
        }

        return created;
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

            const quote: u8 = if (std.mem.indexOfScalar(u8, attribute.value, '"') == null) '"' else if (std.mem.indexOfScalar(u8, attribute.value, '\'') == null) '\'' else return error.DomError;
            try output.append(allocator, '=');
            try output.append(allocator, quote);
            try output.appendSlice(allocator, attribute.value);
            try output.append(allocator, quote);
        }
    }

    fn inputValueForNode(self: *const DomStore, allocator: std.mem.Allocator, node_id: NodeId) errors.Result([]u8) {
        const node = self.nodeAt(node_id) orelse return allocator.dupe(u8, "");
        const element = switch (node.kind) {
            .element => |element| element,
            else => return allocator.dupe(u8, ""),
        };
        return if (elementAttributeValue(element, "value")) |value|
            allocator.dupe(u8, value)
        else
            allocator.dupe(u8, "");
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
            if (self.isOptionSelected(descendant_id)) {
                return allocator.dupe(u8, option_value);
            }
        }

        if (first_option) |value| {
            first_option = null;
            return value;
        }

        return allocator.dupe(u8, "");
    }

    fn isOptionSelected(self: *const DomStore, node_id: NodeId) bool {
        const node = self.nodeAt(node_id) orelse return false;
        const element = switch (node.kind) {
            .element => |element| element,
            else => return false,
        };
        return std.mem.eql(u8, element.tag_name, "option") and elementAttributeValue(element, "selected") != null;
    }

    fn isOptionNode(self: *const DomStore, node_id: NodeId) bool {
        const tag_name = self.tagNameForNode(node_id) orelse return false;
        return std.mem.eql(u8, tag_name, "option");
    }

    fn setOptionSelected(self: *DomStore, node_id: NodeId, selected: bool) errors.Result(void) {
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

    fn serializationNamespaceForNode(self: *const DomStore, node_id: NodeId) SerializationNamespace {
        const node = self.nodeAt(node_id) orelse return .html;
        const parent_id = node.parent orelse return .html;
        const parent = self.nodeAt(parent_id) orelse return .html;
        const parent_namespace = self.serializationNamespaceForNode(parent_id);

        return switch (node.kind) {
            .element => |element| serializationNamespaceForChild(parent, parent_namespace, element.tag_name),
            else => parent_namespace,
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

        return;
    }

    fn addElement(
        self: *DomStore,
        parent: NodeId,
        tag_name: []const u8,
        attributes: std.ArrayListUnmanaged(Attribute),
    ) errors.Result(NodeId) {
        const arena_alloc = self.arena.allocator();
        const node_id = NodeId.new(@intCast(self.nodes.items.len), 0);
        try self.nodes.append(arena_alloc, .{
            .id = node_id,
            .parent = parent,
            .children = .{},
            .kind = .{ .element = .{
                .tag_name = tag_name,
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
    ) errors.Result(?NodeId) {
        const node = self.nodeAt(node_id) orelse return error.HtmlParse;
        for (node.children.items) |child_id| {
            const child = self.nodeAt(child_id) orelse return error.HtmlParse;
            switch (child.kind) {
                .element => {
                    if (nodeMatchesAnyChain(self, child_id, chains)) {
                        return child_id;
                    }
                },
                else => {},
            }

            if (try self.findFirstMatchingDescendant(child_id, chains)) |found| {
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
        const text = rest[0..next_tag];
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
                _ = try store.addElement(stack.items[stack.items.len - 1], tag_name, attributes);
                return;
            }

            if (self.currentByte() == '>') {
                self.pos += 1;
                const node_id = try store.addElement(stack.items[stack.items.len - 1], tag_name, attributes);
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
            const value = rest[0..end];
            self.pos += end + 1;
            return duplicateString(store, value);
        }

        const start = self.pos;
        while (self.currentByte()) |byte| {
            if (isHtmlWhitespace(byte) or byte == '>') break;
            self.pos += 1;
        }
        if (self.pos == start) return error.HtmlParse;
        return duplicateString(store, self.input[start..self.pos]);
    }
};

fn duplicateString(store: *DomStore, value: []const u8) errors.Result([]const u8) {
    return try store.arena.allocator().dupe(u8, value);
}

fn duplicateLowercase(store: *DomStore, value: []const u8) errors.Result([]const u8) {
    const out = try store.arena.allocator().alloc(u8, value.len);
    for (value, 0..) |byte, i| {
        out[i] = std.ascii.toLower(byte);
    }
    return out;
}

fn trimAttributeName(name: []const u8) ?[]const u8 {
    const trimmed = std.mem.trim(u8, name, " \t\r\n");
    return if (trimmed.len == 0) null else trimmed;
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

fn isTagNameByte(byte: u8) bool {
    return std.ascii.isAlphanumeric(byte) or byte == '-' or byte == '_' or byte == ':';
}

fn isTextInputType(input_type: ?[]const u8) bool {
    const value = input_type orelse "text";
    return std.mem.eql(u8, value, "text") or std.mem.eql(u8, value, "search") or std.mem.eql(u8, value, "url") or std.mem.eql(u8, value, "tel") or std.mem.eql(u8, value, "email") or std.mem.eql(u8, value, "password") or std.mem.eql(u8, value, "number") or std.mem.eql(u8, value, "date") or std.mem.eql(u8, value, "datetime-local") or std.mem.eql(u8, value, "month") or std.mem.eql(u8, value, "week") or std.mem.eql(u8, value, "time") or std.mem.eql(u8, value, "color");
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
    for (value) |byte| {
        switch (byte) {
            '&' => try output.appendSlice(allocator, "&amp;"),
            '<' => try output.appendSlice(allocator, "&lt;"),
            '>' => try output.appendSlice(allocator, "&gt;"),
            '"' => try output.appendSlice(allocator, "&quot;"),
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
            ',' => {
                if (bracket_depth != 0) continue;
                const item = std.mem.trim(u8, selector[start..index], " \t\r\n");
                if (item.len == 0) return error.HtmlParse;
                try appendSelectorChain(allocator, chains, item);
                start = index + 1;
            },
            else => {},
        }
    }

    if (quote != null or bracket_depth != 0) return error.HtmlParse;

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
            '[' => {
                try parseSelectorAttribute(allocator, selector, pos, &query);
                saw_token = true;
            },
            else => {
                if (!isSelectorTokenByte(byte)) return error.HtmlParse;
                const token = try parseSelectorToken(selector, pos);
                if (query.tag != null) return error.HtmlParse;
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

fn previousElementSibling(self: *const DomStore, node_id: NodeId) ?NodeId {
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

fn sameNodeId(left: NodeId, right: NodeId) bool {
    return left.index == right.index and left.generation == right.generation;
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
    results: *std.ArrayList(NodeId),
    allocator: std.mem.Allocator,
) errors.Result(void) {
    const node = self.nodeAt(node_id) orelse return error.HtmlParse;
    switch (node.kind) {
        .element => {
            if (nodeMatchesAnyChain(self, node_id, chains)) {
                try results.append(allocator, node_id);
            }
        },
        else => {},
    }

    for (node.children.items) |child_id| {
        try collectSelectorMatches(self, child_id, chains, results, allocator);
    }
}

fn collectSelectorMatchesWithin(
    self: *const DomStore,
    node_id: NodeId,
    chains: []const SelectorChain,
    results: *std.ArrayList(NodeId),
    allocator: std.mem.Allocator,
) errors.Result(void) {
    const node = self.nodeAt(node_id) orelse return error.HtmlParse;
    for (node.children.items) |child_id| {
        const child = self.nodeAt(child_id) orelse return error.HtmlParse;
        switch (child.kind) {
            .element => {
                if (nodeMatchesAnyChain(self, child_id, chains)) {
                    try results.append(allocator, child_id);
                }
            },
            else => {},
        }

        try collectSelectorMatchesWithin(self, child_id, chains, results, allocator);
    }
}

fn nodeMatchesAnyChain(self: *const DomStore, node_id: NodeId, chains: []const SelectorChain) bool {
    for (chains) |chain| {
        if (nodeMatchesSelectorChain(self, node_id, &chain)) return true;
    }
    return false;
}

fn nodeMatchesSelectorChain(self: *const DomStore, node_id: NodeId, chain: *const SelectorChain) bool {
    if (chain.parts.items.len == 0) return false;
    const last_index = chain.parts.items.len - 1;
    return nodeMatchesSelectorChainPart(self, node_id, chain.parts.items, chain.relations.items, last_index);
}

fn nodeMatchesSelectorChainPart(
    self: *const DomStore,
    node_id: NodeId,
    parts: []const SelectorQuery,
    relations: []const SelectorCombinator,
    index: usize,
) bool {
    if (!nodeMatchesSelectorQuery(self, node_id, parts[index])) return false;
    if (index == 0) return true;

    return switch (relations[index - 1]) {
        .child => blk: {
            const parent_id = parentOf(self, node_id) orelse break :blk false;
            break :blk nodeMatchesSelectorChainPart(self, parent_id, parts, relations, index - 1);
        },
        .descendant => blk: {
            var ancestor = parentOf(self, node_id);
            while (ancestor) |ancestor_id| {
                if (nodeMatchesSelectorChainPart(self, ancestor_id, parts, relations, index - 1)) {
                    break :blk true;
                }
                ancestor = parentOf(self, ancestor_id);
            }
            break :blk false;
        },
        .adjacent_sibling => blk: {
            const sibling_id = previousElementSibling(self, node_id) orelse break :blk false;
            break :blk nodeMatchesSelectorChainPart(self, sibling_id, parts, relations, index - 1);
        },
        .general_sibling => blk: {
            var sibling = previousElementSibling(self, node_id);
            while (sibling) |sibling_id| {
                if (nodeMatchesSelectorChainPart(self, sibling_id, parts, relations, index - 1)) {
                    break :blk true;
                }
                sibling = previousElementSibling(self, sibling_id);
            }
            break :blk false;
        },
    };
}

fn nodeMatchesSelectorQuery(self: *const DomStore, node_id: NodeId, query: SelectorQuery) bool {
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

    return true;
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

test "failure: HTML serialization surfaces reject lossy attribute serialization in DomStore" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='root'><div id='target'></div></main>");

    const target = store.findElementById("target").?;
    try store.setAttribute(target, "data-label", "a'b\"c");
    try std.testing.expectError(error.DomError, store.outerHtml(allocator, target));
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
