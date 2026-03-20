const std = @import("std");

pub const Error = error{
    InvalidSelector,
    AssertionFailed,
    DomError,
    EventError,
    HtmlParse,
    MockError,
    ScriptParse,
    ScriptRuntime,
    InvalidUrl,
    TimerError,
    OutOfMemory,
};

pub fn Result(comptime T: type) type {
    return Error!T;
}
