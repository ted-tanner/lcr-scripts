use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MarginSet {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Dimensions {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OrgOrdering {
    pub name: String,
    pub begins_new_column: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub diagram_start_x: i32,
    pub diagram_start_y: i32,
    pub calling_bubbles_per_row: i32,
    pub org_bubble_width: i32,
    pub org_bubble_title_height: i32,
    pub org_bubble_margins: MarginSet,
    pub sub_org_bubble_horzontal_margins: i32,
    pub sub_org_bubble_vertical_margins: i32,
    pub calling_bubble_vertical_margins: i32,
    pub calling_bubble_dimensions: Dimensions,
    pub calling_bubble_min_horizontal_margin: i32,
    pub org_ordering: Vec<OrgOrdering>,
}
