use crate::config::Config;
use crate::data::Organization;
use rand::Rng;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub enum GenerateError {
    DataError(crate::data::DataError),
    InvalidLayout(String),
}

impl std::error::Error for GenerateError {}

impl fmt::Display for GenerateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenerateError::DataError(e) => write!(f, "Data error: {}", e),
            GenerateError::InvalidLayout(msg) => write!(f, "Invalid layout: {}", msg),
        }
    }
}

impl From<crate::data::DataError> for GenerateError {
    fn from(err: crate::data::DataError) -> Self {
        GenerateError::DataError(err)
    }
}

pub fn diagram_file_contents(
    orgs: &HashMap<String, Organization>,
    conf: &Config,
) -> Result<String, GenerateError> {
    let mut file_contents = String::new();

    let mut rng = rand::thread_rng();
    let diagram_id = rng.gen_range(0..u128::MAX);

    let diagram_header = format!(
        r#"<mxfile host="app.diagrams.net" agent="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.2 Safari/605.1.15" version="26.0.5">
  <diagram id="{}" name="Page-1">
    <mxGraphModel grid="1" page="0" gridSize="10" guides="1" tooltips="1" connect="1" arrows="1" fold="1" pageScale="1" pageWidth="827" pageHeight="1169" math="0" shadow="0">
      <root>
        <mxCell id="0" />
        <mxCell id="1" parent="0" />"#,
        diagram_id
    );

    file_contents.push_str(&diagram_header);

    let min_org_bubble_height = conf.org_bubble_title_height
        + conf.calling_bubble_dimensions.height
        + 2 * conf.calling_bubble_vertical_margins;

    let calling_bubble_row_width_no_margins =
        conf.calling_bubble_dimensions.width * conf.calling_bubbles_per_row;
    let calling_bubble_left_margin_in_org_bubble = (conf.org_bubble_width
        - calling_bubble_row_width_no_margins)
        / (conf.calling_bubbles_per_row + 1);

    let sub_org_bubble_width = conf.org_bubble_width - 2 * conf.sub_org_bubble_horzontal_margins;
    let calling_bubble_left_margin_in_sub_org_bubble = (sub_org_bubble_width
        - calling_bubble_row_width_no_margins)
        / (conf.calling_bubbles_per_row + 1);

    if calling_bubble_left_margin_in_sub_org_bubble < conf.calling_bubble_min_horizontal_margin {
        return Err(GenerateError::InvalidLayout(
            "Org bubble width too small".to_string(),
        ));
    }

    let mut org_bubble_cursor_x = conf.diagram_start_x;
    let mut org_bubble_cursor_y = conf.diagram_start_y;

    let mut calling_num = 0;

    for ordering in &conf.org_ordering {
        let org_name = sanitize(&ordering.name);
        let start_new_column = ordering.begins_new_column;

        let org = orgs.get(&ordering.name).ok_or_else(|| {
            GenerateError::InvalidLayout(format!("Unrecognized org name: {}", ordering.name))
        })?;

        let mut org_bubble_id = org_name.replace(" ", "-");
        org_bubble_id = format!("{}-{}", org_bubble_id, rand_tag(&mut rng));

        if start_new_column {
            org_bubble_cursor_x += conf.org_bubble_width
                + conf.org_bubble_margins.right
                + conf.org_bubble_margins.left;
            org_bubble_cursor_y = conf.diagram_start_y;
        }

        org_bubble_cursor_y += conf.org_bubble_margins.top;
        let org_bubble_x = org_bubble_cursor_x;
        let org_bubble_y = org_bubble_cursor_y;

        let mut org_calling_bubble_elems = Vec::new();

        let mut calling_bubble_cursor_x = 0;
        let mut calling_bubble_cursor_y = 0;

        calling_bubble_cursor_y +=
            conf.org_bubble_title_height + conf.calling_bubble_vertical_margins;

        let mut i = 0;
        for calling in &org.callings {
            let member = match &calling.member {
                Some(m) => m,
                None => continue,
            };

            if i != 0 {
                if i % conf.calling_bubbles_per_row == 0 {
                    calling_bubble_cursor_x = 0;
                    let calling_bubble_height_with_margin = conf.calling_bubble_dimensions.height
                        + conf.calling_bubble_vertical_margins;
                    calling_bubble_cursor_y += calling_bubble_height_with_margin;
                } else {
                    calling_bubble_cursor_x += conf.calling_bubble_dimensions.width;
                }
            }

            calling_bubble_cursor_x += calling_bubble_left_margin_in_org_bubble;

            let calling_bubble_elem = format!(
                r#"          <mxCell id="calling-{}" value="&lt;div&gt;&lt;b&gt;&lt;font style=&quot;font-size: 18px;&quot;&gt;{}&lt;/font&gt;&lt;/b&gt;&lt;/div&gt;&lt;div&gt;&lt;br&gt;&lt;/div&gt;&lt;div&gt;{}, {}&lt;/div&gt;&lt;div&gt;Since: {}&lt;/div&gt;" style="rounded=1;whiteSpace=wrap;html=1;align=left;spacingLeft=0;spacingTop=0;spacing=10;fontSize=16;" vertex="1" parent="{}">
            <mxGeometry x="{}" y="{}" width="{}" height="{}" as="geometry" />
          </mxCell>"#,
                calling_num,
                sanitize(&calling.name),
                member.last_name,
                member.given_names,
                member.held_calling_since.to_string()?,
                org_bubble_id,
                calling_bubble_cursor_x,
                calling_bubble_cursor_y,
                conf.calling_bubble_dimensions.width,
                conf.calling_bubble_dimensions.height,
            );
            calling_num += 1;

            org_calling_bubble_elems.push(calling_bubble_elem);

            i += 1;
        }

        org_bubble_cursor_x = org_bubble_x;

        // Assumption: If an org has sub-orgs, there are no callings in the org that aren't part of
        //             a sub-org
        let mut sub_org_bubble_cursor_y = conf.org_bubble_title_height;
        let mut all_sub_orgs_in_org_bubble_elems = Vec::new();

        for child in &org.children {
            // If there are grandchildren, put their callings the sub org
            let mut callings = Vec::new();

            for c in &child.callings {
                if c.member.is_some() {
                    callings.push(c.clone());
                }
            }

            for grandchild in &child.children {
                for c in &grandchild.callings {
                    if c.member.is_some() {
                        callings.push(c.clone());
                    }
                }
            }

            if callings.is_empty() {
                continue;
            }

            let child_name = sanitize(&child.name);

            let mut sub_org_bubble_id = child_name.replace(" ", "-");
            sub_org_bubble_id = format!("{}-{}", sub_org_bubble_id, rand_tag(&mut rng));

            let mut sub_org_bubble_elems = Vec::new();
            sub_org_bubble_elems.push(String::new()); // Placeholder for sub-org bubble itself

            calling_bubble_cursor_x = 0;
            calling_bubble_cursor_y =
                conf.org_bubble_title_height + conf.calling_bubble_vertical_margins;

            i = 0;
            for calling in &callings {
                let member = match &calling.member {
                    Some(m) => m,
                    None => continue,
                };

                if i != 0 {
                    if i % conf.calling_bubbles_per_row == 0 {
                        calling_bubble_cursor_x = 0;
                        let calling_bubble_height_with_margin =
                            conf.calling_bubble_dimensions.height
                                + conf.calling_bubble_vertical_margins;
                        calling_bubble_cursor_y += calling_bubble_height_with_margin;
                    } else {
                        calling_bubble_cursor_x += conf.calling_bubble_dimensions.width;
                    }
                }

                calling_bubble_cursor_x += calling_bubble_left_margin_in_sub_org_bubble;

                let calling_bubble_elem = format!(
                    r#"            <mxCell id="calling-{}" value="&lt;div&gt;&lt;b&gt;&lt;font style=&quot;font-size: 18px;&quot;&gt;{}&lt;/font&gt;&lt;/b&gt;&lt;/div&gt;&lt;div&gt;&lt;br&gt;&lt;/div&gt;&lt;div&gt;{}, {}&lt;/div&gt;&lt;div&gt;Since: {}&lt;/div&gt;" style="rounded=1;whiteSpace=wrap;html=1;align=left;spacingLeft=0;spacingTop=0;spacing=10;fontSize=16;" vertex="1" parent="{}">
              <mxGeometry x="{}" y="{}" width="{}" height="{}" as="geometry" />
            </mxCell>"#,
                    calling_num,
                    sanitize(&calling.name),
                    member.last_name,
                    member.given_names,
                    member.held_calling_since.to_string()?,
                    sub_org_bubble_id,
                    calling_bubble_cursor_x,
                    calling_bubble_cursor_y,
                    conf.calling_bubble_dimensions.width,
                    conf.calling_bubble_dimensions.height,
                );
                calling_num += 1;

                sub_org_bubble_elems.push(calling_bubble_elem);

                i += 1;
            }

            // The first element in sub_org_calling_bubble_elems is a placeholder for the sub-org bubble itself,
            // so subtract one from the length of the sub_org_bubble_elems list to do this calculation
            let sub_org_filled_callings_count = sub_org_bubble_elems.len() - 1;
            let mut sub_org_bubble_height = ((sub_org_filled_callings_count - 1)
                / conf.calling_bubbles_per_row as usize
                + 1) as i32
                * (conf.calling_bubble_dimensions.height + conf.calling_bubble_vertical_margins)
                + conf.calling_bubble_vertical_margins
                + conf.org_bubble_title_height;
            if sub_org_bubble_height < min_org_bubble_height {
                sub_org_bubble_height = min_org_bubble_height;
            }

            sub_org_bubble_cursor_y += conf.sub_org_bubble_vertical_margins;

            let sub_org_bubble_elem = format!(
                r#"          <mxCell id="{}" value="&lt;font style=&quot;font-size: 22px;&quot;&gt;{}&lt;/font&gt;" style="swimlane;whiteSpace=wrap;html=1;rounded=1;strokeWidth=4;startSize=40;strokeColor=#9E9E9E;fontColor=#6B6B6B;" vertex="1" parent="{}">
            <mxGeometry x="{}" y="{}" width="{}" height="{}" as="geometry" />
          </mxCell>"#,
                sub_org_bubble_id,
                child_name,
                org_bubble_id,
                conf.sub_org_bubble_horzontal_margins,
                sub_org_bubble_cursor_y,
                sub_org_bubble_width,
                sub_org_bubble_height,
            );

            sub_org_bubble_elems[0] = sub_org_bubble_elem;

            sub_org_bubble_cursor_y += sub_org_bubble_height;

            all_sub_orgs_in_org_bubble_elems.push(sub_org_bubble_elems);
        }

        let org_filled_callings_count = org_calling_bubble_elems.len();
        let mut org_bubble_height = if org.children.is_empty() {
            ((org_filled_callings_count - 1) / conf.calling_bubbles_per_row as usize + 1) as i32
                * (conf.calling_bubble_dimensions.height + conf.calling_bubble_vertical_margins)
                + conf.calling_bubble_vertical_margins
                + conf.org_bubble_title_height
        } else {
            sub_org_bubble_cursor_y + conf.sub_org_bubble_vertical_margins
        };

        if org_bubble_height < min_org_bubble_height {
            org_bubble_height = min_org_bubble_height;
        }

        org_bubble_cursor_y += org_bubble_height;

        let org_bubble_elem = format!(
            r#"        <mxCell id="{}" value="&lt;font style=&quot;font-size: 22px;&quot;&gt;{}&lt;/font&gt;" style="swimlane;whiteSpace=wrap;html=1;rounded=1;strokeWidth=4;startSize=40;" vertex="1" parent="1">
          <mxGeometry x="{}" y="{}" width="{}" height="{}" as="geometry" />
        </mxCell>"#,
            org_bubble_id,
            org_name,
            org_bubble_x,
            org_bubble_y,
            conf.org_bubble_width,
            org_bubble_height,
        );

        file_contents.push_str(&org_bubble_elem);

        for elem in org_calling_bubble_elems {
            file_contents.push_str(&elem);
        }

        for sub_org_bubble_elems in all_sub_orgs_in_org_bubble_elems {
            for elem in sub_org_bubble_elems {
                file_contents.push_str(&elem);
            }
        }

        org_bubble_cursor_y += conf.org_bubble_margins.bottom;
    }

    let diagram_footer = r#"      </root>
    </mxGraphModel>
  </diagram>
</mxfile>"#;

    file_contents.push_str(diagram_footer);

    Ok(file_contents)
}

fn sanitize(str: &str) -> String {
    str.replace("&", "and")
}

fn rand_tag(rng: &mut impl Rng) -> u32 {
    rng.gen_range(100000..1000000)
}
