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
        _ = self;

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
};

const Assignment = struct {
    target: PropertyTarget,
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

const Value = union(enum) {
    undefined_value,
    null_value,
    boolean: bool,
    number: f64,
    string: []const u8,
    element: dom.NodeId,
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
    for (program.statements) |statement| {
        try evalStatement(allocator, host, bindings, statement);
    }
}

fn evalStatement(
    allocator: std.mem.Allocator,
    host: anytype,
    bindings: []const Binding,
    statement: Statement,
) errors.Result(void) {
    switch (statement) {
        .expression => |expr| {
            _ = try evalExpr(allocator, host, bindings, expr);
        },
        .assignment => |assignment| {
            const value = try evalExpr(allocator, host, bindings, assignment.value);
            try evalAssignment(allocator, host, bindings, assignment.target, value);
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

            return error.ScriptRuntime;
        },
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
    for (bindings) |binding| {
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
        .document => if (std.mem.eql(u8, member.property, "defaultView"))
            Value{ .window = {} }
        else
            error.ScriptRuntime,
        .window => if (std.mem.eql(u8, member.property, "document"))
            Value{ .document = {} }
        else
            error.ScriptRuntime,
        .element => |element| blk: {
            if (std.mem.eql(u8, member.property, "textContent")) {
                const text = try host.domStore().textContent(allocator, element);
                break :blk Value{ .string = text };
            }
            if (std.mem.eql(u8, member.property, "value")) {
                const value = try host.domStore().valueForNode(allocator, element);
                break :blk Value{ .string = value };
            }
            if (std.mem.eql(u8, member.property, "checked")) {
                break :blk Value{ .boolean = host.domStore().checkedForNode(element) orelse false };
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
        } else if (std.mem.eql(u8, method, "addEventListener")) blk: {
            break :blk try registerListener(allocator, host, bindings, .{ .element = element }, args);
        } else error.ScriptRuntime,
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
        .element, .event, .document, .window, .function => true,
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
