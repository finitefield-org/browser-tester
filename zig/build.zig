const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    _ = b.addModule("browser_tester_zig", .{
        .root_source_file = b.path("src/root.zig"),
        .target = target,
    });

    const test_root = b.createModule(.{
        .root_source_file = b.path("src/root.zig"),
        .target = target,
        .optimize = optimize,
    });

    const tests = b.addTest(.{
        .root_module = test_root,
    });

    const run_tests = b.addRunArtifact(tests);
    const test_step = b.step("test", "Run the browser_tester_zig tests");
    test_step.dependOn(&run_tests.step);

    const check_step = b.step("check", "Alias for test");
    check_step.dependOn(test_step);
}
