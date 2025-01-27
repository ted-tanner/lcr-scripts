// Get the JSON file from https://lcr.churchofjesuschrist.org/api/orgs/sub-orgs-with-callings?ip=true&lang=eng
// after sigining into LCR

const std = @import("std");
const data = @import("data.zig");
const gen = @import("generate.zig");
const parse = @import("parse.zig");

const max_input_file_size = 2 * 1024 * 1024 * 1024; // 2 GB

pub fn main() !void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const allocator = arena.allocator();

    const args = try std.process.argsAlloc(allocator);
    if (args.len != 3) {
        std.log.err("invalid args\nusage: callings-diagram <input file> <output file>\n", .{});
        std.process.exit(1);
    }

    const input_file_contents = try std.fs.cwd().readFileAlloc(allocator, args[1], max_input_file_size);
    const orgs = try parse.orgs_from_lcr_data(allocator, input_file_contents);

    for (orgs.values()) |org| {
        std.debug.print("{s}\n", .{org.name});
        for (org.callings.items) |calling| {
            if (calling.member != null) {
                std.debug.print("\t{s}: {s}, {s} (Since {s})\n", .{ calling.name, calling.member.?.last_name, calling.member.?.given_names, try calling.member.?.held_calling_since.allocPrint(allocator) });
            }
        }

        for (org.children.items) |child| {
            std.debug.print("\t{s}\n", .{child.name});
            for (child.callings.items) |calling| {
                if (calling.member != null) {
                    std.debug.print("\t\t{s}: {s}, {s} (Since {s})\n", .{ calling.name, calling.member.?.last_name, calling.member.?.given_names, try calling.member.?.held_calling_since.allocPrint(allocator) });
                }
            }

            for (child.children.items) |grandchild| {
                std.debug.print("\t\t{s}\n", .{grandchild.name});
                for (grandchild.callings.items) |calling| {
                    if (calling.member != null) {
                        std.debug.print("\t\t\t{s}: {s}, {s} (Since {s})\n", .{ calling.name, calling.member.?.last_name, calling.member.?.given_names, try calling.member.?.held_calling_since.allocPrint(allocator) });
                    }
                }
            }
        }
    }

    const output_file_contents = try gen.generate_diagram_file_contents(allocator, orgs);
    const output_file = try std.fs.cwd().createFile(args[2], .{ .truncate = true });
    try output_file.writeAll(output_file_contents);
}
