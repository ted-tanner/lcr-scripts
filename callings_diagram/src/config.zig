const std = @import("std");
const json = std.json;

const MarginSet = struct {
    top: i32,
    right: i32,
    bottom: i32,
    left: i32,
};

const Dimensions = struct {
    width: i32,
    height: i32,
};

const OrgOrdering = struct {
    name: []const u8,
    begins_new_column: bool,
};

pub const Config = struct {
    diagram_start_x: i32,
    diagram_start_y: i32,
    calling_bubbles_per_row: i32,

    org_bubble_width: i32,
    org_bubble_title_height: i32,
    org_bubble_margins: MarginSet,

    sub_org_bubble_horzontal_margins: i32,
    sub_org_bubble_vertical_margins: i32,

    calling_bubble_vertical_margins: i32,
    calling_bubble_dimensions: Dimensions,
    calling_bubble_min_horizontal_margin: i32,

    org_ordering: []OrgOrdering,
};

pub fn parse(allocator: std.mem.Allocator, config_file_contents: []u8) !Config {
    const conf = try json.parseFromSlice(Config, allocator, config_file_contents, .{});
    return conf.value;
}
