const std = @import("std");
const data = @import("data.zig");

const DiagramGenerationError = error{
    OrgBubbleWidthTooSmall,
    UnrecognizedOrgName,
};

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

const calling_bubbles_per_row = 3;
const diagram_start_x = 0;
const diagram_start_y = 0;

const org_bubble_width = 1250;
const org_bubble_title_height = 40;
const org_bubble_margins = MarginSet{
    .top = 0,
    .right = 80,
    .bottom = 80,
    .left = 0,
};

const sub_org_bubble_margins = MarginSet{
    .top = 0,
    .right = 10,
    .bottom = 40,
    .left = 10,
};

const calling_bubble_vertical_margin = 25;
const min_horizontal_margin_between_calling_bubbles = 20;
const calling_bubble_dimensions = Dimensions{
    .width = 340,
    .height = 110,
};

const min_org_bubble_height = org_bubble_title_height + calling_bubble_dimensions.height + 2 * calling_bubble_vertical_margin;

// Tuple of org name and whether org starts new column
const org_ordering = [_]OrgOrdering{
    OrgOrdering{ .name = "Primary", .begins_new_column = false },
    OrgOrdering{ .name = "Relief Society", .begins_new_column = true },
    OrgOrdering{ .name = "Young Women", .begins_new_column = false },
    OrgOrdering{ .name = "Bishopric", .begins_new_column = true },
    OrgOrdering{ .name = "Sunday School", .begins_new_column = false },
    OrgOrdering{ .name = "Elders Quorum", .begins_new_column = true },
    OrgOrdering{ .name = "Aaronic Priesthood Quorums", .begins_new_column = false },
    OrgOrdering{ .name = "Ward Missionaries", .begins_new_column = true },
    OrgOrdering{ .name = "Full-Time Missionaries", .begins_new_column = false },
    OrgOrdering{ .name = "Temple and Family History", .begins_new_column = false },
    OrgOrdering{ .name = "Other Callings", .begins_new_column = false },
};

pub fn generate_diagram_file_contents(allocator: std.mem.Allocator, orgs: std.StringArrayHashMap(data.Organization)) ![]const u8 {
    var file_contents = std.ArrayList(u8).init(allocator);

    var rng = std.rand.DefaultPrng.init(@as(u64, @bitCast(std.time.microTimestamp())));
    const diagram_header = try std.fmt.allocPrint(allocator,
        \\<mxfile host="app.diagrams.net" agent="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.2 Safari/605.1.15" version="26.0.5">
        \\  <diagram id="{}" name="Page-1">
        \\    <mxGraphModel grid="1" page="0" gridSize="10" guides="1" tooltips="1" connect="1" arrows="1" fold="1" pageScale="1" pageWidth="827" pageHeight="1169" math="0" shadow="0">
        \\      <root>
        \\        <mxCell id="0" />
        \\        <mxCell id="1" parent="0" />
    , .{rng.random().int(u128)});

    try file_contents.appendSlice(diagram_header);

    const calling_bubble_row_width = calling_bubble_dimensions.width * calling_bubbles_per_row;
    const calling_bubble_left_margin_in_org_bubble = (org_bubble_width - calling_bubble_row_width) / (calling_bubbles_per_row + 1);

    // const sub_org_bubble_width = org_bubble_width - sub_org_bubble_margins.left - sub_org_bubble_margins.right;
    // const calling_bubble_left_margin_in_sub_org_bubble = (sub_org_bubble_width - calling_bubble_min_row_width) / (calling_bubbles_per_row + 1);

    // TODO: Validate bubbles will fit with min_horizontal_margin_between_calling_bubbles or throw

    var org_bubble_cursor_x: i32 = diagram_start_x;
    var org_bubble_cursor_y: i32 = diagram_start_y;

    var calling_num: i32 = 0;

    for (org_ordering) |ordering| {
        const org_name = ordering.name;
        const start_new_column = ordering.begins_new_column;

        const org = orgs.getPtr(org_name) orelse return DiagramGenerationError.UnrecognizedOrgName;
        const org_bubble_id = try allocator.alloc(u8, org_name.len);
        _ = std.mem.replace(u8, org_name, " ", "-", org_bubble_id);

        if (start_new_column) {
            org_bubble_cursor_x += org_bubble_width + org_bubble_margins.right + org_bubble_margins.left;
            org_bubble_cursor_y = diagram_start_y;
        }

        org_bubble_cursor_y += org_bubble_margins.top;
        const org_bubble_x = org_bubble_cursor_x;
        const org_bubble_y = org_bubble_cursor_y;

        var org_calling_bubble_elems = std.ArrayList([]u8).init(allocator);
        // var org_sub_org_bubble_elems = std.ArrayList(std.ArrayList([]u8)).init(allocator);

        var calling_bubble_cursor_x: i32 = 0;
        var calling_bubble_cursor_y: i32 = 0;

        calling_bubble_cursor_y += org_bubble_title_height + calling_bubble_vertical_margin;

        var i: u32 = 0;
        for (org.callings.items) |calling| {
            const member = calling.member orelse continue;

            if (i != 0) {
                if (i % calling_bubbles_per_row == 0) {
                    calling_bubble_cursor_x = 0;
                    calling_bubble_cursor_y += calling_bubble_dimensions.height + calling_bubble_vertical_margin;
                    org_bubble_cursor_y += calling_bubble_cursor_y;
                } else {
                    calling_bubble_cursor_x += calling_bubble_dimensions.width;
                }
            }

            calling_bubble_cursor_x += calling_bubble_left_margin_in_org_bubble;

            const calling_bubble_elem = try std.fmt.allocPrint(allocator,
                \\          <mxCell id="calling-{}" value="&lt;div&gt;&lt;b&gt;&lt;font style=&quot;font-size: 18px;&quot;&gt;{s}&lt;/font&gt;&lt;/b&gt;&lt;/div&gt;&lt;div&gt;&lt;br&gt;&lt;/div&gt;&lt;div&gt;{s}, {s}&lt;/div&gt;&lt;div&gt;Since: {s}&lt;/div&gt;" style="rounded=1;whiteSpace=wrap;html=1;align=left;spacingLeft=0;spacingTop=0;spacing=10;fontSize=16;" vertex="1" parent="{s}">
                \\            <mxGeometry x="{}" y="{}" width="{}" height="{}" as="geometry" />
                \\          </mxCell>
            , .{
                calling_num,
                calling.name,
                member.last_name,
                member.given_names,
                try member.held_calling_since.allocPrint(allocator),
                org_bubble_id,
                calling_bubble_cursor_x,
                calling_bubble_cursor_y,
                calling_bubble_dimensions.width,
                calling_bubble_dimensions.height,
            });
            calling_num += 1;

            try org_calling_bubble_elems.append(calling_bubble_elem);

            i += 1;
        }

        if (org.callings.items.len > 0) {
            org_bubble_cursor_y += calling_bubble_vertical_margin;
        }

        org_bubble_cursor_x = org_bubble_x;

        // TODO
        // for (org.children.items) |child| {}

        var org_bubble_height = org_bubble_cursor_y - org_bubble_y;
        if (org_bubble_height < min_org_bubble_height) {
            org_bubble_cursor_y += min_org_bubble_height - org_bubble_height;
            org_bubble_height = min_org_bubble_height;
        }

        const org_bubble_elem = try std.fmt.allocPrint(allocator,
            \\        <mxCell id="{s}" value="&lt;font style=&quot;font-size: 22px;&quot;&gt;{s}&lt;/font&gt;" style="swimlane;whiteSpace=wrap;html=1;rounded=1;strokeWidth=4;startSize=40;" vertex="1" parent="1">
            \\          <mxGeometry x="{}" y="{}" width="{}" height="{}" as="geometry" />
            \\        </mxCell>
        , .{
            org_bubble_id,
            org_name,
            org_bubble_x,
            org_bubble_y,
            org_bubble_width,
            org_bubble_height,
        });

        try file_contents.appendSlice(org_bubble_elem);
        // TODO: Append sub-org bubbles
        for (org_calling_bubble_elems.items) |elem| {
            try file_contents.appendSlice(elem);
        }

        org_bubble_cursor_y += org_bubble_margins.bottom;
    }

    const diagram_footer = try std.fmt.allocPrint(allocator,
        \\      </root>
        \\    </mxGraphModel>
        \\  </diagram>
        \\</mxfile>
    , .{});

    try file_contents.appendSlice(diagram_footer);

    return file_contents.items;
}
