const std = @import("std");
const json = std.json;
const data = @import("data.zig");

const max_input_file_size = 2 * 1024 * 1024 * 1024; // 2 GB
const invalid_fmt_message = "Input file is missing essential data fields or is in the wrong format";

pub fn main() !void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const allocator = arena.allocator();

    const args = try std.process.argsAlloc(allocator);
    if (args.len != 3) {
        err_and_exit("invalid args\nusage: callings-diagram <input file> <output file>\n", .{});
    }

    const input_file_contents = std.fs.cwd().readFileAlloc(allocator, args[1], max_input_file_size) catch |err| {
        err_and_exit("Could not read file '{s}': {s}", .{ args[1], @errorName(err) });
    };

    const parsed_contents = json.parseFromSlice(json.Value, allocator, input_file_contents, .{}) catch |err| {
        err_and_exit("Failed to parse input file '{s}' as JSON: {s}", .{ args[1], @errorName(err) });
    };
    defer parsed_contents.deinit();

    var orgs = std.ArrayList(data.Organization).init(allocator);
    defer orgs.deinit();

    const parsed_orgs = switch (parsed_contents.value) {
        .array => |v| v,
        else => err_and_exit("Input file JSON must be an array at the top level", .{}),
    };

    for (parsed_orgs.items) |parsed_org| {
        const obj = switch (parsed_org) {
            .object => |v| v,
            else => err_and_exit("{s}", .{invalid_fmt_message}),
        };

        const name = obj.get("name") orelse err_and_exit("{s}", .{invalid_fmt_message});
        const children = obj.get("children") orelse err_and_exit("{s}", .{invalid_fmt_message});

        const org = data.Organization{
            .name = switch (name) {
                .string => |v| v,
                else => err_and_exit("{s}", .{invalid_fmt_message}),
            },
            .children = try process_child_orgs(allocator, &children),
            .callings = std.ArrayList(data.Calling).init(allocator), // TODO
        };

        try orgs.append(org);
    }

    // TODO: Remove this
    for (orgs.items) |org| {
        std.debug.print("{s}\n", .{org.name});
        for (org.children.items) |child| {
            std.debug.print("\t{s}\n", .{child.name});
            for (child.children.items) |grandchild| {
                std.debug.print("\t\t{s}\n", .{grandchild.name});
            }
        }
    }
}

fn process_child_orgs(allocator: std.mem.Allocator, parsed_children: *const json.Value) !std.ArrayList(data.Organization) {
    var child_orgs = std.ArrayList(data.Organization).init(allocator);

    switch (parsed_children.*) {
        .array => |v| {
            for (v.items) |parsed_child| {
                switch (parsed_child) {
                    .object => |obj| {
                        const child_name = obj.get("name") orelse err_and_exit("{s}", .{invalid_fmt_message});
                        const children = obj.get("children") orelse err_and_exit("{s}", .{invalid_fmt_message});
                        const callings = obj.get("callings") orelse err_and_exit("{s}", .{invalid_fmt_message});

                        const child_org = data.Organization{
                            .name = switch (child_name) {
                                .string => |n| n,
                                else => err_and_exit("{s}", .{invalid_fmt_message}),
                            },
                            .children = try process_child_orgs(allocator, &children),
                            .callings = try process_callings(allocator, &callings),
                        };

                        try child_orgs.append(child_org);
                    },
                    else => err_and_exit("{s}", .{invalid_fmt_message}),
                }
            }
        },
        else => err_and_exit("{s}", .{invalid_fmt_message}),
    }

    return child_orgs;
}

fn process_callings(allocator: std.mem.Allocator, parsed_callings: *const json.Value) !std.ArrayList(data.Calling) {
    const callings = std.ArrayList(data.Calling).init(allocator);

    _ = parsed_callings;

    return callings;
}

fn err_and_exit(comptime fmt: []const u8, args: anytype) noreturn {
    std.log.err(fmt, args);
    std.process.exit(1);
}
