const std = @import("std");
const Config = @import("config.zig").Config;
const data = @import("data.zig");

const DiagramGenerationError = error{
    OrgBubbleWidthTooSmall,
    UnrecognizedOrgName,
} || std.mem.Allocator.Error || std.fmt.AllocPrintError || data.DateFormatError;

pub fn diagram_file_contents(allocator: std.mem.Allocator, orgs: std.StringArrayHashMap(data.Organization), conf: Config) ![]const u8 {
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

    // TODO
    const min_org_bubble_height: i32 = conf.org_bubble_title_height + conf.calling_bubble_dimensions.height + 2 * conf.calling_bubble_vertical_margins;

    const calling_bubble_row_width_no_margins: i32 = conf.calling_bubble_dimensions.width * conf.calling_bubbles_per_row;
    const calling_bubble_left_margin_in_org_bubble: i32 = @divTrunc((conf.org_bubble_width - calling_bubble_row_width_no_margins), (conf.calling_bubbles_per_row + 1));

    const sub_org_bubble_width: i32 = conf.org_bubble_width - 2 * conf.sub_org_bubble_horzontal_margins;
    const calling_bubble_left_margin_in_sub_org_bubble: i32 = @divTrunc((sub_org_bubble_width - calling_bubble_row_width_no_margins), (conf.calling_bubbles_per_row + 1));

    if (calling_bubble_left_margin_in_sub_org_bubble < conf.calling_bubble_min_horizontal_margin) {
        return DiagramGenerationError.OrgBubbleWidthTooSmall;
    }

    var org_bubble_cursor_x: i32 = conf.diagram_start_x;
    var org_bubble_cursor_y: i32 = conf.diagram_start_y;

    var calling_num: i32 = 0;

    for (conf.org_ordering) |ordering| {
        const org_name = try sanitize(allocator, ordering.name);
        const start_new_column = ordering.begins_new_column;

        const org = orgs.getPtr(org_name) orelse return DiagramGenerationError.UnrecognizedOrgName;

        var org_bubble_id = try allocator.alloc(u8, org_name.len);
        _ = std.mem.replace(u8, org_name, " ", "-", org_bubble_id);
        org_bubble_id = try std.fmt.allocPrint(allocator, "{s}-{}", .{ org_bubble_id, rand_tag(&rng) });

        if (start_new_column) {
            org_bubble_cursor_x += conf.org_bubble_width + conf.org_bubble_margins.right + conf.org_bubble_margins.left;
            org_bubble_cursor_y = conf.diagram_start_y;
        }

        org_bubble_cursor_y += conf.org_bubble_margins.top;
        const org_bubble_x: i32 = org_bubble_cursor_x;
        const org_bubble_y: i32 = org_bubble_cursor_y;

        var org_calling_bubble_elems = std.ArrayList([]const u8).init(allocator);

        var calling_bubble_cursor_x: i32 = 0;
        var calling_bubble_cursor_y: i32 = 0;

        calling_bubble_cursor_y += conf.org_bubble_title_height + conf.calling_bubble_vertical_margins;

        var i: i32 = 0;
        for (org.callings.items) |calling| {
            const member = calling.member orelse continue;

            if (i != 0) {
                if (@mod(i, conf.calling_bubbles_per_row) == 0) {
                    calling_bubble_cursor_x = 0;

                    const calling_bubble_height_with_margin = conf.calling_bubble_dimensions.height + conf.calling_bubble_vertical_margins;
                    calling_bubble_cursor_y += calling_bubble_height_with_margin;
                } else {
                    calling_bubble_cursor_x += conf.calling_bubble_dimensions.width;
                }
            }

            calling_bubble_cursor_x += calling_bubble_left_margin_in_org_bubble;

            const calling_bubble_elem = try std.fmt.allocPrint(allocator,
                \\          <mxCell id="calling-{}" value="&lt;div&gt;&lt;b&gt;&lt;font style=&quot;font-size: 18px;&quot;&gt;{s}&lt;/font&gt;&lt;/b&gt;&lt;/div&gt;&lt;div&gt;&lt;br&gt;&lt;/div&gt;&lt;div&gt;{s}, {s}&lt;/div&gt;&lt;div&gt;Since: {s}&lt;/div&gt;" style="rounded=1;whiteSpace=wrap;html=1;align=left;spacingLeft=0;spacingTop=0;spacing=10;fontSize=16;" vertex="1" parent="{s}">
                \\            <mxGeometry x="{}" y="{}" width="{}" height="{}" as="geometry" />
                \\          </mxCell>
            , .{
                calling_num,
                try sanitize(allocator, calling.name),
                member.last_name,
                member.given_names,
                try member.held_calling_since.allocPrint(allocator),
                org_bubble_id,
                calling_bubble_cursor_x,
                calling_bubble_cursor_y,
                conf.calling_bubble_dimensions.width,
                conf.calling_bubble_dimensions.height,
            });
            calling_num += 1;

            try org_calling_bubble_elems.append(calling_bubble_elem);

            i += 1;
        }

        org_bubble_cursor_x = org_bubble_x;

        // Assumption: If an org has sub-orgs, there are no callings in the org that aren't part of
        //             a sub-org
        var sub_org_bubble_cursor_y: i32 = conf.org_bubble_title_height;
        var all_sub_orgs_in_org_bubble_elems = std.ArrayList(std.ArrayList([]const u8)).init(allocator);

        for (org.children.items) |child| {
            // If there are grandchildren, put their callings the sub org
            var callings = std.ArrayList(data.Calling).init(allocator);

            for (child.callings.items) |c| {
                if (c.member == null) {
                    continue;
                }
                try callings.append(c);
            }

            for (child.children.items) |grandchild| {
                for (grandchild.callings.items) |c| {
                    if (c.member == null) {
                        continue;
                    }
                    try callings.append(c);
                }
            }

            if (callings.items.len == 0) {
                continue;
            }

            const child_name = try sanitize(allocator, child.name);

            var sub_org_bubble_id = try allocator.alloc(u8, child_name.len);
            _ = std.mem.replace(u8, child_name, " ", "-", sub_org_bubble_id);
            sub_org_bubble_id = try std.fmt.allocPrint(allocator, "{s}-{}", .{ sub_org_bubble_id, rand_tag(&rng) });

            var sub_org_bubble_elems = std.ArrayList([]const u8).init(allocator);
            _ = try sub_org_bubble_elems.addOne(); // Placeholder for sub-org bubble itself

            calling_bubble_cursor_x = 0;
            calling_bubble_cursor_y = conf.org_bubble_title_height + conf.calling_bubble_vertical_margins;

            i = 0;
            for (callings.items) |calling| {
                const member = calling.member orelse continue;

                if (i != 0) {
                    if (@mod(i, conf.calling_bubbles_per_row) == 0) {
                        calling_bubble_cursor_x = 0;

                        const calling_bubble_height_with_margin = conf.calling_bubble_dimensions.height + conf.calling_bubble_vertical_margins;
                        calling_bubble_cursor_y += calling_bubble_height_with_margin;
                    } else {
                        calling_bubble_cursor_x += conf.calling_bubble_dimensions.width;
                    }
                }

                calling_bubble_cursor_x += calling_bubble_left_margin_in_sub_org_bubble;

                const calling_bubble_elem = try std.fmt.allocPrint(allocator,
                    \\            <mxCell id="calling-{}" value="&lt;div&gt;&lt;b&gt;&lt;font style=&quot;font-size: 18px;&quot;&gt;{s}&lt;/font&gt;&lt;/b&gt;&lt;/div&gt;&lt;div&gt;&lt;br&gt;&lt;/div&gt;&lt;div&gt;{s}, {s}&lt;/div&gt;&lt;div&gt;Since: {s}&lt;/div&gt;" style="rounded=1;whiteSpace=wrap;html=1;align=left;spacingLeft=0;spacingTop=0;spacing=10;fontSize=16;" vertex="1" parent="{s}">
                    \\              <mxGeometry x="{}" y="{}" width="{}" height="{}" as="geometry" />
                    \\            </mxCell>
                , .{
                    calling_num,
                    try sanitize(allocator, calling.name),
                    member.last_name,
                    member.given_names,
                    try member.held_calling_since.allocPrint(allocator),
                    sub_org_bubble_id,
                    calling_bubble_cursor_x,
                    calling_bubble_cursor_y,
                    conf.calling_bubble_dimensions.width,
                    conf.calling_bubble_dimensions.height,
                });
                calling_num += 1;

                try sub_org_bubble_elems.append(calling_bubble_elem);

                i += 1;
            }

            // The first element in sub_org_calling_bubble_elems is a placeholder for the sub-org bubble itself,
            // so subtract one from the length of the sub_org_bubble_elems list to do this calculation
            const sub_org_filled_callings_count: i32 = if (sub_org_bubble_elems.items.len - 1 > std.math.maxInt(i32)) std.math.maxInt(i32) else @intCast(sub_org_bubble_elems.items.len - 1);
            var sub_org_bubble_height: i32 = (@divTrunc(sub_org_filled_callings_count - 1, conf.calling_bubbles_per_row) + 1) * (conf.calling_bubble_dimensions.height + conf.calling_bubble_vertical_margins) + conf.calling_bubble_vertical_margins + conf.org_bubble_title_height;
            if (sub_org_bubble_height < min_org_bubble_height) {
                sub_org_bubble_height = min_org_bubble_height;
            }

            sub_org_bubble_cursor_y += conf.sub_org_bubble_vertical_margins;

            const sub_org_bubble_elem = try std.fmt.allocPrint(allocator,
                \\          <mxCell id="{s}" value="&lt;font style=&quot;font-size: 22px;&quot;&gt;{s}&lt;/font&gt;" style="swimlane;whiteSpace=wrap;html=1;rounded=1;strokeWidth=4;startSize=40;strokeColor=#9E9E9E;fontColor=#6B6B6B;" vertex="1" parent="{s}">
                \\            <mxGeometry x="{}" y="{}" width="{}" height="{}" as="geometry" />
                \\          </mxCell>
            , .{
                sub_org_bubble_id,
                child_name,
                org_bubble_id,
                conf.sub_org_bubble_horzontal_margins,
                sub_org_bubble_cursor_y,
                sub_org_bubble_width,
                sub_org_bubble_height,
            });

            sub_org_bubble_elems.items[0] = sub_org_bubble_elem;

            sub_org_bubble_cursor_y += sub_org_bubble_height;

            try all_sub_orgs_in_org_bubble_elems.append(sub_org_bubble_elems);
        }

        const org_filled_callings_count: i32 = if (org_calling_bubble_elems.items.len > std.math.maxInt(i32)) std.math.maxInt(i32) else @intCast(org_calling_bubble_elems.items.len);
        var org_bubble_height: i32 = 0;

        if (org.children.items.len == 0) {
            org_bubble_height = (@divTrunc(org_filled_callings_count - 1, conf.calling_bubbles_per_row) + 1) * (conf.calling_bubble_dimensions.height + conf.calling_bubble_vertical_margins) + conf.calling_bubble_vertical_margins + conf.org_bubble_title_height;
        } else {
            org_bubble_height = sub_org_bubble_cursor_y + conf.sub_org_bubble_vertical_margins;
        }

        if (org_bubble_height < min_org_bubble_height) {
            org_bubble_height = min_org_bubble_height;
        }

        org_bubble_cursor_y += org_bubble_height;

        const org_bubble_elem = try std.fmt.allocPrint(allocator,
            \\        <mxCell id="{s}" value="&lt;font style=&quot;font-size: 22px;&quot;&gt;{s}&lt;/font&gt;" style="swimlane;whiteSpace=wrap;html=1;rounded=1;strokeWidth=4;startSize=40;" vertex="1" parent="1">
            \\          <mxGeometry x="{}" y="{}" width="{}" height="{}" as="geometry" />
            \\        </mxCell>
        , .{
            org_bubble_id,
            org_name,
            org_bubble_x,
            org_bubble_y,
            conf.org_bubble_width,
            org_bubble_height,
        });

        try file_contents.appendSlice(org_bubble_elem);

        for (org_calling_bubble_elems.items) |elem| {
            try file_contents.appendSlice(elem);
        }

        for (all_sub_orgs_in_org_bubble_elems.items) |sub_org_bubble_elems| {
            for (sub_org_bubble_elems.items) |elem| {
                try file_contents.appendSlice(elem);
            }
        }

        org_bubble_cursor_y += conf.org_bubble_margins.bottom;
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

fn sanitize(allocator: std.mem.Allocator, str: []const u8) std.mem.Allocator.Error![]u8 {
    const sanitized = try allocator.alloc(u8, std.mem.replacementSize(u8, str, "&", "and"));
    _ = std.mem.replace(u8, str, "&", "and", sanitized);
    return sanitized;
}

fn rand_tag(rng: *std.rand.DefaultPrng) u32 {
    return rng.random().int(u32) % 900000 + 100000;
}
