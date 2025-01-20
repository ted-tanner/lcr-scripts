const std = @import("std");
const json = std.json;
const data = @import("data.zig");

const max_input_file_size = 2 * 1024 * 1024 * 1024; // 2 GB
const invalid_fmt_message = "Input file is missing essential data fields or is in the wrong format";

const JsonResponseFormatError = error{
    WrongType,
    MissingField,
    BadFormatting,
};

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

    const parsed_contents = try json.parseFromSlice(json.Value, allocator, input_file_contents, .{});
    defer parsed_contents.deinit();

    var orgs = std.ArrayList(data.Organization).init(allocator);
    defer orgs.deinit();

    const parsed_orgs = switch (parsed_contents.value) {
        .array => parsed_contents.value.array,
        else => return JsonResponseFormatError.WrongType,
    };

    for (parsed_orgs.items) |parsed_org| {
        const obj = switch (parsed_org) {
            .object => parsed_org.object,
            else => return JsonResponseFormatError.WrongType,
        };

        const name = obj.getPtr("name") orelse return JsonResponseFormatError.MissingField;
        const children = obj.getPtr("children") orelse return JsonResponseFormatError.MissingField;
        const callings = obj.getPtr("callings") orelse return JsonResponseFormatError.MissingField;

        const org = data.Organization{
            .name = switch (name.*) {
                .string => name.string,
                else => return JsonResponseFormatError.WrongType,
            },
            .children = try process_child_orgs(allocator, children),
            .callings = try process_callings(allocator, callings),
        };

        try orgs.append(org);
    }

    // TODO: Remove this
    for (orgs.items) |org| {
        std.debug.print("{s}\n", .{org.name});
        for (org.callings.items) |calling| {
            if (calling.member != null) {
                std.debug.print("\t{s}: {s}, {s}\n", .{ calling.name, calling.member.?.last_name, calling.member.?.given_names });
            }
        }

        for (org.children.items) |child| {
            std.debug.print("\t{s}\n", .{child.name});
            for (child.callings.items) |calling| {
                if (calling.member != null) {
                    std.debug.print("\t\t{s}: {s}, {s}\n", .{ calling.name, calling.member.?.last_name, calling.member.?.given_names });
                }
            }

            for (child.children.items) |grandchild| {
                std.debug.print("\t\t{s}\n", .{grandchild.name});
                for (grandchild.callings.items) |calling| {
                    if (calling.member != null) {
                        std.debug.print("\t\t\t{s}: {s}, {s}\n", .{ calling.name, calling.member.?.last_name, calling.member.?.given_names });
                    }
                }
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
                        const child_name = obj.getPtr("name") orelse return JsonResponseFormatError.MissingField;
                        const children = obj.getPtr("children") orelse return JsonResponseFormatError.MissingField;
                        const callings = obj.getPtr("callings") orelse return JsonResponseFormatError.MissingField;

                        const child_org = data.Organization{
                            .name = switch (child_name.*) {
                                .string => child_name.string,
                                else => return JsonResponseFormatError.WrongType,
                            },
                            .children = try process_child_orgs(allocator, children),
                            .callings = try process_callings(allocator, callings),
                        };

                        try child_orgs.append(child_org);
                    },
                    else => return JsonResponseFormatError.WrongType,
                }
            }
        },
        else => return JsonResponseFormatError.WrongType,
    }

    return child_orgs;
}

fn process_callings(allocator: std.mem.Allocator, parsed_callings: *const json.Value) !std.ArrayList(data.Calling) {
    var callings = std.ArrayList(data.Calling).init(allocator);

    switch (parsed_callings.*) {
        .array => |v| {
            for (v.items) |parsed_calling| {
                switch (parsed_calling) {
                    .object => |obj| {
                        const calling_name_value = obj.getPtr("position") orelse return JsonResponseFormatError.MissingField;
                        const calling_name = switch (calling_name_value.*) {
                            .string => calling_name_value.string,
                            else => return JsonResponseFormatError.WrongType,
                        };

                        const member_name_value = obj.getPtr("memberName") orelse return JsonResponseFormatError.MissingField;
                        const calling = switch (member_name_value.*) {
                            .null => data.Calling{ .name = calling_name, .member = null },
                            .string => str_blk: {
                                const member_name = member_name_value.string;

                                const held_calling_since_value = obj.getPtr("activeDate") orelse return JsonResponseFormatError.MissingField;
                                const held_calling_since_str = switch (held_calling_since_value.*) {
                                    .string => held_calling_since_value.string,
                                    else => return JsonResponseFormatError.WrongType,
                                };

                                if (held_calling_since_str.len != 8) {
                                    std.debug.print("hcs: {s}", .{held_calling_since_str});
                                    return JsonResponseFormatError.BadFormatting;
                                }

                                const held_calling_since = data.Date{
                                    .year = try std.fmt.parseUnsigned(u16, held_calling_since_str[0..4], 10),
                                    .month = try std.fmt.parseUnsigned(u8, held_calling_since_str[4..6], 10),
                                    .day = try std.fmt.parseUnsigned(u8, held_calling_since_str[6..8], 10),
                                };

                                const set_apart_value = obj.getPtr("setApart") orelse return JsonResponseFormatError.MissingField;
                                const set_apart = switch (set_apart_value.*) {
                                    .bool => set_apart_value.bool,
                                    else => return JsonResponseFormatError.WrongType,
                                };

                                var comma_pos: usize = 0;
                                for (0.., member_name) |i, c| {
                                    if (c == ',') {
                                        comma_pos = i;
                                    }
                                }

                                if (comma_pos + 2 == member_name.len) {
                                    return JsonResponseFormatError.BadFormatting;
                                }

                                const email_value = obj.getPtr("memberEmail") orelse return JsonResponseFormatError.MissingField;
                                const email = switch (email_value.*) {
                                    .null => null,
                                    .string => email_value.string,
                                    else => return JsonResponseFormatError.WrongType,
                                };

                                const phone_value = obj.getPtr("memberPhone") orelse return JsonResponseFormatError.MissingField;
                                const phone = switch (phone_value.*) {
                                    .null => null,
                                    .string => phone_value.string,
                                    else => return JsonResponseFormatError.WrongType,
                                };

                                const member = data.MemberWithCalling{
                                    .given_names = member_name[comma_pos + 2 ..],
                                    .last_name = member_name[0..comma_pos],
                                    .email = email,
                                    .phone = phone,

                                    .held_calling_since = held_calling_since,
                                    .set_apart = set_apart,
                                };

                                break :str_blk data.Calling{ .name = calling_name, .member = member };
                            },
                            else => return JsonResponseFormatError.WrongType,
                        };

                        try callings.append(calling);
                    },
                    else => return JsonResponseFormatError.WrongType,
                }
            }
        },
        else => return JsonResponseFormatError.WrongType,
    }

    return callings;
}
