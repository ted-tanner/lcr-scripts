// Get the JSON file from https://lcr.churchofjesuschrist.org/api/orgs/sub-orgs-with-callings?ip=true&lang=eng
// after sigining into LCR

mod data;

use data::{Calling, CsvRecord, Date, MemberWithCalling, Organization};
use std::{
    collections::HashMap,
    io::{BufReader, Write},
};

fn main() {
    let input_file_path = std::env::args()
        .nth(1)
        .expect("usage: callings-spreadsheet <input_file> <output_file>");
    let output_file_path = std::env::args()
        .nth(2)
        .expect("usage: callings-spreadsheet <input_file> <output_file>");

    let input_file = std::fs::File::open(&input_file_path).expect("Could not open input file");
    let json: serde_json::Value =
        serde_json::from_reader(BufReader::new(input_file)).expect("Could not parse JSON file");

    let mut orgs = HashMap::new();

    let json_orgs = json.as_array().expect("Could not read organizations array");
    for json_org in json_orgs {
        let org_name = json_org["name"]
            .as_str()
            .expect("Could not read organization name");

        orgs.entry(org_name).or_insert_with(|| Organization {
            name: org_name,
            children: process_child_orgs(&json_org["children"]),
            callings: process_callings(&json_org["callings"]),
        });
    }

    let mut csv_records = Vec::new();

    for org in orgs.values() {
        let org_name = org.name;
        let mut sub_org_name = "";
        let mut sub_sub_org_name = "";
        for calling in &org.callings {
            if let Some(member) = &calling.member {
                let record = CsvRecord {
                    member_last_name: member.last_name,
                    member_given_names: member.given_names,
                    calling: calling.name,
                    sub_sub_organization: &sub_sub_org_name,
                    sub_organization: &sub_org_name,
                    organization: org_name,
                    held_calling_since: member.held_calling_since,
                    set_apart: member.set_apart,
                    member_email: member.email,
                    member_phone: member.phone,
                };
                csv_records.push(record);
            }
        }

        for child in &org.children {
            sub_org_name = child.name;
            for calling in &child.callings {
                if let Some(member) = &calling.member {
                    let record = CsvRecord {
                        member_last_name: member.last_name,
                        member_given_names: member.given_names,
                        calling: calling.name,
                        sub_sub_organization: &sub_sub_org_name,
                        sub_organization: &sub_org_name,
                        organization: org_name,
                        held_calling_since: member.held_calling_since,
                        set_apart: member.set_apart,
                        member_email: member.email,
                        member_phone: member.phone,
                    };
                    csv_records.push(record);
                }
            }

            for child in &child.children {
                if !child.children.is_empty() {
                    panic!("Organizations are too deeply layered!");
                }

                sub_sub_org_name = child.name;
                for calling in &child.callings {

                    if let Some(member) = &calling.member {
                        let record = CsvRecord {
                            member_last_name: member.last_name,
                            member_given_names: member.given_names,
                            calling: calling.name,
                            sub_sub_organization: &sub_sub_org_name,
                            sub_organization: &sub_org_name,
                            organization: org_name,
                            held_calling_since: member.held_calling_since,
                            set_apart: member.set_apart,
                            member_email: member.email,
                            member_phone: member.phone,
                        };
                        csv_records.push(record);
                    }
                }
            }
        }
    }

    let csv_headings = "Member Last Name, Member Given Names,Calling,Sub-sub-organization,Sub-organization,Organization,Held Calling Since,Set Apart,Member Email,Member Phone\r\n";
    let mut output_file =
        std::fs::File::create(&output_file_path).expect("Could not open output file");

    output_file.write(csv_headings.as_bytes()).expect("Could not write to file");
    for record in csv_records {
        output_file.write(record.to_string().as_bytes()).expect("Could not write to file");
        output_file.write("\r\n".as_bytes()).expect("Could not write to file");
    }

    println!("Generated spreadsheet at '{}'", output_file_path);
}

fn process_child_orgs<'a>(children: &'a serde_json::Value) -> Vec<Organization<'a>> {
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

fn process_callings<'a>(callings: &'a serde_json::Value) -> Vec<Calling<'a>> {
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
                    given_names: if name_parts.len() > 1 { &name_parts[1] } else { &name_parts[0] },
                    last_name: if name_parts.len() > 1 { &name_parts[0] } else { "" },
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
