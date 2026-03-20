const std = @import("std");

pub const Error = error{
    HtmlParse,
    InvalidUrl,
    OutOfMemory,
};

pub fn Result(comptime T: type) type {
    return Error!T;
}
