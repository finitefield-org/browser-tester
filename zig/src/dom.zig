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
    attributes: std.ArrayListUnmanaged(SelectorAttribute) = .{},

    fn deinit(self: *SelectorQuery, allocator: std.mem.Allocator) void {
        self.attributes.deinit(allocator);
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

    pub fn childIds(self: *const DomStore, node_id: NodeId) []const NodeId {
        if (self.nodeAt(node_id)) |node| {
            return node.children.items;
        }
        return &.{};
    }

    pub fn tagNameForNode(self: *const DomStore, node_id: NodeId) ?[]const u8 {
        const node = self.nodeAt(node_id) orelse return null;
        return switch (node.kind) {
            .element => |element| element.tag_name,
            else => null,
        };
    }

    pub fn select(self: *const DomStore, allocator: std.mem.Allocator, selector: []const u8) errors.Result([]NodeId) {
        const trimmed = std.mem.trim(u8, selector, " \t\r\n");
        if (trimmed.len == 0) return error.HtmlParse;

        var queries: std.ArrayList(SelectorQuery) = .empty;
        errdefer {
            for (queries.items) |*query| {
                query.deinit(allocator);
            }
            queries.deinit(allocator);
        }

        try appendSelectorQueries(allocator, trimmed, &queries);
        if (queries.items.len == 0) return error.HtmlParse;

        var results: std.ArrayList(NodeId) = .empty;
        errdefer results.deinit(allocator);

        for (self.nodes.items) |node| {
            if (!nodeMatchesAnyQuery(node, queries.items)) continue;
            try results.append(allocator, node.id);
        }

        for (queries.items) |*query| {
            query.deinit(allocator);
        }
        queries.deinit(allocator);

        const owned = try allocator.dupe(NodeId, results.items);
        results.deinit(allocator);
        return owned;
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
        try self.appendChild(parent, node_id);
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
        try self.appendChild(parent, node_id);
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
        try self.appendChild(parent, node_id);
        return node_id;
    }

    fn appendChild(self: *DomStore, parent: NodeId, child: NodeId) errors.Result(void) {
        const parent_index: usize = @intCast(parent.index);
        if (parent_index >= self.nodes.items.len) return error.HtmlParse;
        const arena_alloc = self.arena.allocator();
        try self.nodes.items[parent_index].children.append(arena_alloc, child);
    }

    fn setSourceHtml(self: *DomStore, html: []const u8) errors.Result(void) {
        const arena_alloc = self.arena.allocator();
        self.source_html = try arena_alloc.dupe(u8, html);
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

        while (self.pos < self.bytes.len) {
            const current_parent = stack.items[stack.items.len - 1];
            if (store.tagNameForNode(current_parent)) |tag_name| {
                if (isRawTextElement(tag_name)) {
                    const rest = self.input[self.pos..];
                    const closing_tag = try std.fmt.allocPrint(arena_alloc, "</{s}>", .{tag_name});
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
                    try self.parseClosingTag(store, &stack);
                    continue;
                }

                if (self.startsWith("<!")) {
                    try self.parseDeclaration();
                    continue;
                }

                try self.parseStartTag(store, &stack);
                continue;
            }

            try self.parseText(store, current_parent);
        }

        if (stack.items.len != 1) {
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

    fn parseClosingTag(self: *HtmlParser, store: *DomStore, stack: *std.ArrayListUnmanaged(NodeId)) errors.Result(void) {
        self.pos += 2;
        self.skipWhitespace();
        if (self.pos >= self.bytes.len or !isTagNameByte(self.currentByte().?)) {
            return error.HtmlParse;
        }

        const closing_name = try self.parseNameToken(store);
        self.skipWhitespace();
        if (self.currentByte() != '>') return error.HtmlParse;
        self.pos += 1;

        if (stack.items.len <= 1) {
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

fn isHtmlWhitespace(byte: u8) bool {
    return switch (byte) {
        ' ', '\n', '\r', '\t', 0x0c => true,
        else => false,
    };
}

fn isTagNameByte(byte: u8) bool {
    return std.ascii.isAlphanumeric(byte) or byte == '-' or byte == '_' or byte == ':';
}

fn isSelectorTokenByte(byte: u8) bool {
    return std.ascii.isAlphanumeric(byte) or byte == '-' or byte == '_';
}

fn isVoidElement(tag_name: []const u8) bool {
    return std.mem.eql(u8, tag_name, "area")
        or std.mem.eql(u8, tag_name, "base")
        or std.mem.eql(u8, tag_name, "br")
        or std.mem.eql(u8, tag_name, "col")
        or std.mem.eql(u8, tag_name, "embed")
        or std.mem.eql(u8, tag_name, "hr")
        or std.mem.eql(u8, tag_name, "img")
        or std.mem.eql(u8, tag_name, "input")
        or std.mem.eql(u8, tag_name, "link")
        or std.mem.eql(u8, tag_name, "meta")
        or std.mem.eql(u8, tag_name, "param")
        or std.mem.eql(u8, tag_name, "source")
        or std.mem.eql(u8, tag_name, "track")
        or std.mem.eql(u8, tag_name, "wbr");
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

fn appendSelectorQueries(
    allocator: std.mem.Allocator,
    selector: []const u8,
    queries: *std.ArrayList(SelectorQuery),
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
            '"' , '\'' => quote = byte,
            '[' => bracket_depth += 1,
            ']' => {
                if (bracket_depth == 0) return error.HtmlParse;
                bracket_depth -= 1;
            },
            ',' => {
                if (bracket_depth != 0) continue;
                const item = std.mem.trim(u8, selector[start..index], " \t\r\n");
                if (item.len == 0) return error.HtmlParse;
                try appendSelectorQuery(allocator, queries, item);
                start = index + 1;
            },
            else => {},
        }
    }

    if (quote != null or bracket_depth != 0) return error.HtmlParse;

    const item = std.mem.trim(u8, selector[start..], " \t\r\n");
    if (item.len == 0) return error.HtmlParse;
    try appendSelectorQuery(allocator, queries, item);
}

fn appendSelectorQuery(
    allocator: std.mem.Allocator,
    queries: *std.ArrayList(SelectorQuery),
    selector: []const u8,
) errors.Result(void) {
    var query = try parseSelectorQuery(allocator, selector);
    errdefer query.deinit(allocator);
    try queries.append(allocator, query);
}

fn parseSelectorQuery(
    allocator: std.mem.Allocator,
    selector: []const u8,
) errors.Result(SelectorQuery) {
    var query: SelectorQuery = .{};
    errdefer query.deinit(allocator);

    var pos: usize = 0;
    var saw_token = false;

    while (pos < selector.len) {
        const byte = selector[pos];
        if (isHtmlWhitespace(byte)) return error.HtmlParse;

        switch (byte) {
            '#' => {
                pos += 1;
                const token = try parseSelectorToken(selector, &pos);
                if (query.id != null) return error.HtmlParse;
                query.id = token;
                saw_token = true;
            },
            '[' => {
                try parseSelectorAttribute(allocator, selector, &pos, &query);
                saw_token = true;
            },
            else => {
                if (!isSelectorTokenByte(byte)) return error.HtmlParse;
                const token = try parseSelectorToken(selector, &pos);
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
    skipSelectorWhitespace(selector, pos);

    const name = try parseSelectorToken(selector, pos);
    skipSelectorWhitespace(selector, pos);

    var operator: SelectorAttributeOperator = .exists;
    var value: ?[]const u8 = null;
    var case_sensitivity: SelectorAttributeCaseSensitivity = .case_sensitive;

    if (pos.* < selector.len and selector[pos.*] != ']') {
        operator = try parseSelectorAttributeOperator(selector, pos);
        skipSelectorWhitespace(selector, pos);
        value = try parseSelectorAttributeValue(selector, pos);
        skipSelectorWhitespace(selector, pos);

        if (pos.* < selector.len and (selector[pos.*] == 'i' or selector[pos.*] == 's')) {
            case_sensitivity = if (selector[pos.*] == 'i')
                .case_insensitive
            else
                .case_sensitive;
            pos.* += 1;
            skipSelectorWhitespace(selector, pos);
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

fn skipSelectorWhitespace(selector: []const u8, pos: *usize) void {
    while (pos.* < selector.len and isHtmlWhitespace(selector[pos.*])) {
        pos.* += 1;
    }
}

fn nodeMatchesAnyQuery(node: NodeRecord, queries: []const SelectorQuery) bool {
    for (queries) |query| {
        if (nodeMatchesQuery(node, query)) return true;
    }
    return false;
}

fn nodeMatchesQuery(node: NodeRecord, query: SelectorQuery) bool {
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

test "phase one: unsupported selector syntax is rejected explicitly" {
    const allocator = std.testing.allocator;
    var store = try DomStore.init(allocator);
    defer store.deinit();

    try store.bootstrapHtml("<main id='app'><span>Hello</span></main>");

    try std.testing.expectError(error.HtmlParse, store.select(allocator, "main > span"));
    try std.testing.expectError(error.HtmlParse, store.select(allocator, "[data-state"));
    try std.testing.expectError(error.HtmlParse, store.select(allocator, ""));
}
