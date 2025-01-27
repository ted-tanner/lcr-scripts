// Get the JSON file from https://lcr.churchofjesuschrist.org/api/orgs/sub-orgs-with-callings?ip=true&lang=eng
// after sigining into LCR

mod data;

use data::{Calling, Date, MemberWithCalling, Organization};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Write},
};
use uuid::Uuid;

struct MarginSet {
    top: i32,
    right: i32,
    bottom: i32,
    left: i32,
}

struct Dimensions {
    width: i32,
    height: i32,
}

const ORG_BUBBLE_MARGINS: MarginSet = MarginSet {
    top: 0,
    right: 40,
    bottom: 0,
    left: 40,
};
const SUB_ORG_BUBBLE_MARGINS: MarginSet = MarginSet {
    top: 10,
    right: 10,
    bottom: 10,
    left: 10,
};
const CALLING_BUBBLE_MARGINS: MarginSet = MarginSet {
    top: 10,
    right: 15,
    bottom: 10,
    left: 15,
};

const CALLING_BUBBLE_DIMENSIONS: Dimensions = Dimensions {
    width: 340,
    height: 110,
};

const CALLING_BUBBLES_PER_ROW: i32 = 3;
const ORG_BUBBLE_TITLE_HEIGHT: i32 = 45;

const DIAGRAM_START_X: i32 = 0;
const DIAGRAM_START_Y: i32 = 0;

// Calulated values (not for configuration)
const CALLING_BUBBLE_TOTAL_HEIGHT: i32 =
    CALLING_BUBBLE_DIMENSIONS.height + CALLING_BUBBLE_MARGINS.top + CALLING_BUBBLE_MARGINS.bottom;
const ORG_BUBBLE_WIDTH: i32 =
    (CALLING_BUBBLE_DIMENSIONS.width + CALLING_BUBBLE_MARGINS.left + CALLING_BUBBLE_MARGINS.right)
        * CALLING_BUBBLES_PER_ROW
        + ORG_BUBBLE_MARGINS.left
        + ORG_BUBBLE_MARGINS.right;

fn main() {
    let input_file_path = std::env::args()
        .nth(1)
        .expect("usage: callings-diagram <input_file> <output_file>");
    let output_file_path = std::env::args()
        .nth(2)
        .expect("usage: callings-diagram <input_file> <output_file>");

    let input_file = File::open(&input_file_path).expect("Could not open input file");
    let json: serde_json::Value =
        serde_json::from_reader(BufReader::new(input_file)).expect("Could not parse JSON file");

    let mut orgs = HashMap::new();
    let mut ordered_org_names = Vec::new();

    let json_orgs = json.as_array().expect("Could not read organizations array");
    for json_org in json_orgs {
        let org_name = json_org["name"]
            .as_str()
            .expect("Could not read organization name");

        ordered_org_names.push(org_name);

        orgs.entry(org_name).or_insert_with(|| Organization {
            name: org_name,
            children: process_child_orgs(&json_org["children"]),
            callings: process_callings(&json_org["callings"]),
        });
    }

    // for org in orgs.values() {
    //     println!("{}", org.name);
    //     for calling in &org.callings {
    //         let Some(member) = &calling.member else {
    //             continue;
    //         };
    //         println!(
    //             "\t{}: {}, {} (Since {})",
    //             calling.name, member.last_name, member.given_names, member.held_calling_since
    //         );
    //     }

    //     for child in &org.children {
    //         println!("\t{}", child.name);

    //         for calling in &child.callings {
    //             let Some(member) = &calling.member else {
    //                 continue;
    //             };
    //             println!(
    //                 "\t\t{}: {}, {} (Since {})",
    //                 calling.name, member.last_name, member.given_names, member.held_calling_since
    //             );
    //         }

    //         for grandchild in &child.children {
    //             println!("\t\t{}", grandchild.name);

    //             for calling in &grandchild.callings {
    //                 let Some(member) = &calling.member else {
    //                     continue;
    //                 };
    //                 println!(
    //                     "\t\t\t{}: {}, {} (Since {})",
    //                     calling.name,
    //                     member.last_name,
    //                     member.given_names,
    //                     member.held_calling_since
    //                 );
    //             }
    //         }
    //     }
    // }

    //     let mut bubbles = Vec::new();
    //     let mut org_cursor_x = DIAGRAM_START_X;
    //     let mut calling_iter = 0;

    //     for org_name in ordered_org_names {
    //         let org = orgs.get(org_name).expect("Could not find organization");

    //         org_cursor_x += ORG_BUBBLE_MARGINS.left;

    //         let org_id = format!("ORG-{}", org_name.replace(" ", "-"));
    //         let mut org_bubble_height = ORG_BUBBLE_TITLE_HEIGHT + CALLING_BUBBLE_TOTAL_HEIGHT;

    //         let mut other_bubbles = Vec::new();

    //         if org.children.is_empty() {
    //             // Place callings directly into org bubble
    //             let mut calling_cursor_x = org_cursor_x;
    //             let mut calling_cursor_y =
    //                 DIAGRAM_START_Y + ORG_BUBBLE_TITLE_HEIGHT + CALLING_BUBBLE_MARGINS.top;

    //             let mut calling_bubbles = Vec::new();

    //             for calling in &org.callings {
    //                 let Some(ref member) = calling.member else {
    //                     continue;
    //                 };

    //                 calling_cursor_x += CALLING_BUBBLE_MARGINS.left;

    //                 let calling_bubble = format!(
    //                     r##"
    //                             <mxCell id="calling-{}" value="&lt;div&gt;&lt;b&gt;&lt;font style=&quot;font-size: 18px;&quot;&gt;{}&lt;/font&gt;&lt;/b&gt;&lt;/div&gt;&lt;div&gt;&lt;br&gt;&lt;/div&gt;&lt;div&gt;{}, {}&lt;/div&gt;&lt;div&gt;Since: {}&lt;/div&gt;" style="rounded=1;whiteSpace=wrap;html=1;align=left;spacingLeft=0;spacingTop=0;spacing=10;fontSize=16;" vertex="1" parent="{}">
    //                               <mxGeometry x="{}" y="{}" width="{}" height="{}" as="geometry" />
    //                             </mxCell>
    //                     "##,
    //                     calling_iter,
    //                     calling.name,
    //                     member.last_name,
    //                     member.given_names,
    //                     member.held_calling_since,
    //                     org_id,
    //                     calling_cursor_x,
    //                     calling_cursor_y,
    //                     CALLING_BUBBLE_DIMENSIONS.width,
    //                     CALLING_BUBBLE_DIMENSIONS.height,
    //                 );

    //                 calling_iter += 1;
    //                 calling_bubbles.push(calling_bubble);

    //                 calling_cursor_x += CALLING_BUBBLE_DIMENSIONS.width + CALLING_BUBBLE_MARGINS.right;

    //                 if calling_cursor_x
    //                     + CALLING_BUBBLE_MARGINS.left
    //                     + CALLING_BUBBLE_DIMENSIONS.width
    //                     + CALLING_BUBBLE_MARGINS.right
    //                     > ORG_BUBBLE_WIDTH
    //                 {
    //                     calling_cursor_x = org_cursor_x;

    //                     calling_cursor_y += CALLING_BUBBLE_TOTAL_HEIGHT;
    //                     org_bubble_height += CALLING_BUBBLE_TOTAL_HEIGHT;
    //                 }
    //             }

    //             other_bubbles.extend(calling_bubbles);
    //         } else {
    //             // let mut sub_org_bubbles = Vec::new();

    //             // for child_org in &org.children {
    //             //     if child_org.children.is_empty() {
    //             //         // Use child org as sub-org bubble

    //             //         continue;
    //             //     }

    //             //     // Use grandchild org as sub-org bubble
    //             //     for grandchild_org in &child_org.children {

    //             //     }

    //             //     // Place child org callings into "Other" bubble
    //             // }
    //         }

    //         org_bubble_height += CALLING_BUBBLE_MARGINS.bottom;

    //         let org_bubble = format!(
    //             r##"
    //                     <mxCell id="{}" value="&lt;font style=&quot;font-size: 22px;&quot;&gt;{}&lt;/font&gt;" style="swimlane;whiteSpace=wrap;html=1;rounded=1;strokeWidth=4;startSize=40;" vertex="1" parent="1">
    //                       <mxGeometry x="{}" y="{}" width="{}" height="{}" as="geometry" />
    //                     </mxCell>
    //             "##,
    //             org_id, org_name, org_cursor_x, DIAGRAM_START_Y, ORG_BUBBLE_WIDTH, org_bubble_height,
    //         );

    //         bubbles.push(org_bubble);
    //         bubbles.extend(other_bubbles);

    //         org_cursor_x += ORG_BUBBLE_WIDTH + ORG_BUBBLE_MARGINS.right;
    //     }

    //     let diagram_header = format!(
    //         r##"<mxfile host="app.diagrams.net" agent="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.2 Safari/605.1.15" version="26.0.5">
    //       <diagram id="{}" name="Page-1">
    //         <mxGraphModel grid="1" page="0" gridSize="10" guides="1" tooltips="1" connect="1" arrows="1" fold="1" pageScale="1" pageWidth="827" pageHeight="1169" math="0" shadow="0">
    //           <root>
    //             <mxCell id="0" />
    //             <mxCell id="1" parent="0" />"##,
    //         Uuid::new_v4(),
    //     );

    //     let diagram_footer = r##"      </root>
    //     </mxGraphModel>
    //   </diagram>
    // </mxfile>"##;

    //     let mut output_file =
    //         std::fs::File::create(&output_file_path).expect("Could not open output file");

    //     output_file
    //         .write_all(diagram_header.as_bytes())
    //         .expect("Could not write diagram header");
    //     for bubble in bubbles {
    //         output_file
    //             .write_all(bubble.as_bytes())
    //             .expect("Could not write bubble");
    //     }
    //     output_file
    //         .write_all(diagram_footer.as_bytes())
    //         .expect("Could not write diagram header");

    //     println!("Generated diagram at '{}'", output_file_path);
}

fn process_child_orgs(children: &serde_json::Value) -> Vec<Organization<'_>> {
    let mut processed_child_orgs = Vec::new();

    if let Some(children) = children.as_array() {
        for json_child in children {
            let child_name = json_child["name"]
                .as_str()
                .expect("Could not read org name");

            let child_org = Organization {
                name: child_name,
                children: process_child_orgs(&json_child["children"]),
                callings: process_callings(&json_child["callings"]),
            };

            processed_child_orgs.push(child_org);
        }
    }

    processed_child_orgs
}

fn process_callings(callings: &serde_json::Value) -> Vec<Calling<'_>> {
    let mut processed_callings = Vec::new();

    for json_calling in callings.as_array().expect("Could not read callings array") {
        let calling_name = json_calling["position"]
            .as_str()
            .expect("Could not read calling name");

        let mut calling = Calling {
            name: calling_name,
            member: None,
        };

        match json_calling["memberName"].as_str() {
            Some(member_name) => {
                let held_calling_since_str = json_calling["activeDate"]
                    .as_str()
                    .expect("Could not read calling active date");
                let set_apart = json_calling["setApart"].as_bool().unwrap_or(false);

                if held_calling_since_str.len() != 8 {
                    panic!("Invalid date format '{}'", held_calling_since_str);
                }

                let held_calling_since = Date {
                    year: held_calling_since_str[0..4]
                        .parse()
                        .expect("Could not parse year"),
                    month: held_calling_since_str[4..6]
                        .parse()
                        .expect("Could not parse month"),
                    day: held_calling_since_str[6..8]
                        .parse()
                        .expect("Could not parse day"),
                };

                let name_parts = member_name.split(",").collect::<Vec<&str>>();

                let member = MemberWithCalling {
                    given_names: if name_parts.len() > 1 {
                        name_parts[1].trim()
                    } else {
                        name_parts[0]
                    },
                    last_name: if name_parts.len() > 1 {
                        name_parts[0]
                    } else {
                        ""
                    },
                    email: json_calling["memberEmail"].as_str(),
                    phone: json_calling["memberPhone"].as_str(),
                    held_calling_since,
                    set_apart,
                };

                calling.member = Some(member);
            }
            None => continue, // No member holds the calling
        };

        processed_callings.push(calling);
    }

    processed_callings
}
