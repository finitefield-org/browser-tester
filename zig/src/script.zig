const std = @import("std");

const dom = @import("dom.zig");
const errors = @import("errors.zig");

pub const ListenerTarget = union(enum) {
    document,
    window,
    element: dom.NodeId,
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
            const source = try store.textContent(allocator, script_id);
            defer allocator.free(source);
            try self.evalScriptSourceWithBindings(allocator, host, source, "inline-script", &.{});
        }

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
    collection_iterator: *CollectionIteratorState,
    iterator_result: *IteratorResult,
    document_scripts,
    document_anchors,
    document_style_sheets,
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
        .element => |element| {
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
        .identifier => |name| evalIdentifier(bindings, name),
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

fn evalIdentifier(bindings: []const Binding, name: []const u8) errors.Result(Value) {
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
                break :blk Value{ .html_collection = .{ .root = host.domStore().documentId(), .kind = .document_plugins } };
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
        .window => if (std.mem.eql(u8, member.property, "document"))
            Value{ .document = {} }
        else
            error.ScriptRuntime,
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
        .style_sheet => error.ScriptRuntime,
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
        .document => if (std.mem.eql(u8, method, "getElementById")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const id_value = try evalExpr(allocator, host, bindings, args[0]);
            const id = try asString(allocator, id_value);
            if (host.domStore().findElementById(id)) |element_id| {
                break :blk Value{ .element = element_id };
            }
            break :blk Value{ .null_value = {} };
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
        } else error.ScriptRuntime,
        .element => |element| if (std.mem.eql(u8, method, "textContent")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            const text = try host.domStore().textContent(allocator, element);
            break :blk Value{ .string = text };
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
        } else if (std.mem.eql(u8, method, "removeAttribute")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const name_value = try evalExpr(allocator, host, bindings, args[0]);
            const name = try asString(allocator, name_value);
            host.domStoreMut().removeAttribute(element, name) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .undefined_value = {} };
        } else if (std.mem.eql(u8, method, "hasAttribute")) blk: {
            if (args.len != 1) return error.ScriptRuntime;
            const name_value = try evalExpr(allocator, host, bindings, args[0]);
            const name = try asString(allocator, name_value);
            const present = host.domStore().hasAttribute(element, name) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .boolean = present };
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
        } else error.ScriptRuntime,
        .template_content => |_| error.ScriptRuntime,
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
        .style_sheet => error.ScriptRuntime,
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
        } else if (std.mem.eql(u8, method, "forEach")) blk: {
            break :blk try htmlCollectionForEach(allocator, host, bindings, collection, args);
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
        } else error.ScriptRuntime,
        .collection_iterator => |iterator| if (std.mem.eql(u8, method, "next")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            break :blk try collectionIteratorNext(allocator, iterator);
        } else error.ScriptRuntime,
        .node => |node_id| if (std.mem.eql(u8, method, "remove")) blk: {
            if (args.len != 0) return error.ScriptRuntime;
            host.domStoreMut().removeNode(node_id) catch |err| switch (err) {
                error.OutOfMemory => return error.OutOfMemory,
                else => return error.ScriptRuntime,
            };
            break :blk Value{ .undefined_value = {} };
        } else error.ScriptRuntime,
        .iterator_result => error.ScriptRuntime,
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
    if (std.mem.eql(u8, property, "nodeName")) {
        const name = host.domStore().nodeNameForNode(node_id) orelse return error.ScriptRuntime;
        return Value{ .string = name };
    }

    if (std.mem.eql(u8, property, "nodeType")) {
        const node_type = host.domStore().nodeTypeForNode(node_id) orelse return error.ScriptRuntime;
        return Value{ .number = @floatFromInt(node_type) };
    }

    if (std.mem.eql(u8, property, "textContent")) {
        const node = host.domStore().nodeAt(node_id) orelse return error.ScriptRuntime;
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
        const node = host.domStore().nodeAt(node_id) orelse return error.ScriptRuntime;
        switch (node.kind) {
            .document, .element => {
                return Value{ .html_collection = .{ .root = node_id } };
            },
            else => return error.ScriptRuntime,
        }
    }

    return null;
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
            return try host.domStore().valueForNode(allocator, item);
        }
    }

    return "";
}

fn functionBindings(
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

    const child = try evalElementHandle(allocator, host, bindings, args[0]);
    const appended = host.domStoreMut().appendChild(element, child) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return Value{ .element = appended };
}

fn elementInsertBefore(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    element: dom.NodeId,
    args: []*Expr,
) errors.Result(Value) {
    if (args.len != 2) return error.ScriptRuntime;

    const child = try evalElementHandle(allocator, host, bindings, args[0]);
    const reference = try evalOptionalElementHandle(allocator, host, bindings, args[1]);
    const inserted = host.domStoreMut().insertBefore(element, child, reference) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return Value{ .element = inserted };
}

fn elementReplaceChild(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    element: dom.NodeId,
    args: []*Expr,
) errors.Result(Value) {
    if (args.len != 2) return error.ScriptRuntime;

    const new_child = try evalElementHandle(allocator, host, bindings, args[0]);
    const old_child = try evalElementHandle(allocator, host, bindings, args[1]);
    const replaced = host.domStoreMut().replaceChild(element, new_child, old_child) catch |err| switch (err) {
        error.OutOfMemory => return error.OutOfMemory,
        else => return error.ScriptRuntime,
    };
    return Value{ .element = replaced };
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

fn evalOptionalElementHandle(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    expr: *Expr,
) errors.Result(?dom.NodeId) {
    const value = try evalExpr(allocator, host, bindings, expr);
    return switch (value) {
        .element => |element| element,
        .null_value, .undefined_value => null,
        else => error.ScriptRuntime,
    };
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
        .element => "[object Element]",
        .node => "[object Node]",
        .template_content => "[object DocumentFragment]",
        .class_list => "[object DOMTokenList]",
        .dataset => "[object DOMStringMap]",
        .node_list => "[object NodeList]",
        .collection_iterator => "[object Object]",
        .iterator_result => "[object Object]",
        .html_collection => "[object HTMLCollection]",
        .document_scripts => "[object HTMLCollection]",
        .document_anchors => "[object HTMLCollection]",
        .document_style_sheets => "[object StyleSheetList]",
        .style_sheet => "[object CSSStyleSheet]",
        .radio_node_list => "[object RadioNodeList]",
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
        .element, .node, .template_content, .class_list, .dataset, .node_list, .collection_iterator, .iterator_result, .html_collection, .document_scripts, .document_anchors, .document_style_sheets, .style_sheet, .radio_node_list, .event, .document, .window, .function => true,
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
