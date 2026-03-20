const std = @import("std");

const ReservedState = struct {};

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
    reserved: ReservedState = .{},

    pub fn deinit(self: *Session) void {
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
};
