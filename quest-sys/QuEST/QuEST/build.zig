
const std = @import("std");

pub fn build(b: *std.build.Builder) void {
    const target = b.standardTargetOptions(.{});
    const mode = b.standardReleaseOptions();

    const QuEST = b.addStaticLibrary("QuEST", null);
    QuEST.setTarget(target);
    QuEST.setBuildMode(mode);
    QuEST.addIncludePath("include/");
    QuEST.addIncludePath("src/");
    QuEST.linkLibC();
    QuEST.force_pic = true;
    QuEST.addCSourceFiles(&.{
        "src/QuEST.c",
        "src/QuEST_common.c",
        "src/QuEST_qasm.c",
        "src/QuEST_validation.c",
        "src/mt19937ar.c",
        "src/CPU/QuEST_cpu.c",
        "src/CPU/QuEST_cpu_local.c",
    }, &.{
        "-std=c11",
        "-Wall",
        "-W",
        "-Wstrict-prototypes",
        "-Wwrite-strings",
        "-Wno-missing-field-initializers",
    });
    QuEST.install();
}