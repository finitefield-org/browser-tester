const std = @import("std");

pub const Error = error{
    InvalidUrl,
    OutOfMemory,
};

pub fn Result(comptime T: type) type {
    return Error!T;
}
