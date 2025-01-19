const std = @import("std");

pub const Date = struct {
    year: u16,
    month: u8,
    day: u8,
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
