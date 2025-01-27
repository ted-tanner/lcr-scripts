const std = @import("std");

const DateFormatError = error{
    InvalidMonth,
    InvalidDay,
};

pub const Date = struct {
    year: u16,
    month: u8,
    day: u8,

    pub fn allocPrint(self: Date, allocator: std.mem.Allocator) ![]u8 {
        const month_str_and_day_count: struct { []const u8, u8 } = switch (self.month) {
            1 => .{ "January", 31 },
            2 => .{ "February", if (self.year % 4 == 0 and (self.year % 100 != 0 or self.year % 400 == 0)) 29 else 28 },
            3 => .{ "March", 31 },
            4 => .{ "April", 30 },
            5 => .{ "May", 31 },
            6 => .{ "June", 30 },
            7 => .{ "July", 31 },
            8 => .{ "August", 31 },
            9 => .{ "September", 30 },
            10 => .{ "October", 31 },
            11 => .{ "November", 30 },
            12 => .{ "December", 31 },
            else => return DateFormatError.InvalidMonth,
        };

        const month_str = month_str_and_day_count[0];
        const days_in_month = month_str_and_day_count[1];

        if (self.day == 0 or self.day > days_in_month) {
            return DateFormatError.InvalidDay;
        }

        const ordinal_suffix = switch (self.day) {
            1 | 21 | 31 => "st",
            2 | 22 => "nd",
            3 | 23 => "rd",
            else => "th",
        };

        return std.fmt.allocPrint(allocator, "{}, {s} {}{s}", .{ self.year, month_str, self.day, ordinal_suffix });
    }
};

pub const MemberWithCalling = struct {
    given_names: []const u8,
    last_name: []const u8,
    email: ?[]const u8,
    phone: ?[]const u8,

    held_calling_since: Date,
    set_apart: bool,
};

pub const Calling = struct {
    name: []const u8,
    member: ?MemberWithCalling,
};

pub const Organization = struct {
    name: []const u8,
    children: std.ArrayList(Organization),
    callings: std.ArrayList(Calling),
};
