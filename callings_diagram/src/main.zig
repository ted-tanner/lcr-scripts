// Get the JSON file from https://lcr.churchofjesuschrist.org/api/orgs/sub-orgs-with-callings?ip=true&lang=eng
// after sigining into LCR

const std = @import("std");
const config = @import("config.zig");
const data = @import("data.zig");
const gen = @import("generate.zig");
const parse = @import("parse.zig");

pub const log_level: std.log.Level = .info;

const max_input_file_size = 2 * 1024 * 1024 * 1024; // 2 GB

pub fn main() !void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const allocator = arena.allocator();

    const args = try std.process.argsAlloc(allocator);
    if (args.len != 3) {
        std.log.err("invalid args\nusage: callings-diagram <input file> <output file>", .{});
        std.process.exit(1);
    }

    const config_file_contents = std.fs.cwd().readFileAlloc(allocator, "diagram-config.json", max_input_file_size) catch |err| {
        std.log.err("Expected 'diagram-config.json' file in current directory: {any}", .{err});
        std.process.exit(1);
    };

    const conf = config.parse(allocator, config_file_contents) catch |err| {
        std.log.err("Failed to parse config file 'diagram-config.json': {any}", .{err});
        std.process.exit(1);
    };

    const input_file_contents = std.fs.cwd().readFileAlloc(allocator, args[1], max_input_file_size) catch |err| {
        std.log.err("Failed to read input file '{s}': {any}", .{ args[1], err });
        std.process.exit(1);
    };

    const orgs = parse.orgs_from_lcr_data(allocator, input_file_contents) catch |err| {
        std.log.err("Failed to parse input file '{s}': {any}", .{ args[1], err });
        std.process.exit(1);
    };

    const output_file_contents = gen.diagram_file_contents(allocator, orgs, conf) catch |err| {
        std.log.err("Failed to generate diagram file contents: {any}", .{err});
        std.process.exit(1);
    };

    const output_file = std.fs.cwd().createFile(args[2], .{ .truncate = true }) catch |err| {
        std.log.err("Failed to create or open output file '{s}': {any}", .{ args[2], err });
        std.process.exit(1);
    };

    output_file.writeAll(output_file_contents) catch |err| {
        std.log.err("Failed to write to output file '{s}: {any}", .{ args[2], err });
        std.process.exit(1);
    };

    std.log.info("Successfully wrote diagram to {s}", .{args[2]});
}
