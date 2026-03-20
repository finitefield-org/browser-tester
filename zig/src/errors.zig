const std = @import("std");

pub const Error = error{
    InvalidSelector,
    AssertionFailed,
    DomError,
    EventError,
    HtmlParse,
    ScriptParse,
    ScriptRuntime,
    InvalidUrl,
    OutOfMemory,
};

pub fn Result(comptime T: type) type {
    return Error!T;
}
