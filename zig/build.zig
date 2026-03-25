const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const browser_tester_module = b.addModule("browser_tester_zig", .{
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

    const contract_test_root = b.createModule(.{
        .root_source_file = b.path("tests/contract_harness_core.zig"),
        .target = target,
        .optimize = optimize,
        .imports = &.{
            .{
                .name = "browser_tester_zig",
                .module = browser_tester_module,
            },
        },
    });
    const contract_tests = b.addTest(.{
        .root_module = contract_test_root,
    });
    const run_contract_tests = b.addRunArtifact(contract_tests);
    test_step.dependOn(&run_contract_tests.step);

    const issue_test_root = b.createModule(.{
        .root_source_file = b.path("tests/issue_regressions.zig"),
        .target = target,
        .optimize = optimize,
        .imports = &.{
            .{
                .name = "browser_tester_zig",
                .module = browser_tester_module,
            },
        },
    });
    const issue_tests = b.addTest(.{
        .root_module = issue_test_root,
    });
    const run_issue_tests = b.addRunArtifact(issue_tests);
    test_step.dependOn(&run_issue_tests.step);

    const runtime_test_root = b.createModule(.{
        .root_source_file = b.path("tests/runtime_and_real_world.zig"),
        .target = target,
        .optimize = optimize,
        .imports = &.{
            .{
                .name = "browser_tester_zig",
                .module = browser_tester_module,
            },
        },
    });
    const runtime_tests = b.addTest(.{
        .root_module = runtime_test_root,
    });
    const run_runtime_tests = b.addRunArtifact(runtime_tests);
    test_step.dependOn(&run_runtime_tests.step);

    const check_step = b.step("check", "Alias for test");
    check_step.dependOn(test_step);
}
