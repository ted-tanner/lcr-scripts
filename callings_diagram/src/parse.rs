use crate::data::{Calling, DataError, Date, MemberWithCalling, Organization};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    JsonError(serde_json::Error),
    DataError(DataError),
    InvalidFormat(String),
}

impl std::error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::JsonError(e) => write!(f, "JSON error: {}", e),
            ParseError::DataError(e) => write!(f, "Data error: {}", e),
            ParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::JsonError(err)
    }
}

impl From<DataError> for ParseError {
    fn from(err: DataError) -> Self {
        ParseError::DataError(err)
    }
}

pub fn orgs_from_lcr_data(lcr_data: &str) -> Result<HashMap<String, Organization>, ParseError> {
    let parsed_contents: Value = serde_json::from_str(lcr_data)?;

    let mut orgs = HashMap::new();

    let parsed_orgs = parsed_contents
        .as_array()
        .ok_or_else(|| ParseError::InvalidFormat("Expected array at root level".to_string()))?;

    for parsed_org in parsed_orgs {
        let obj = parsed_org.as_object().ok_or_else(|| {
            ParseError::InvalidFormat("Expected object for organization".to_string())
        })?;

        let name = obj.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            ParseError::InvalidFormat("Missing or invalid 'name' field".to_string())
        })?;

        let children = obj
            .get("children")
            .ok_or_else(|| ParseError::InvalidFormat("Missing 'children' field".to_string()))?;

        let callings = obj
            .get("callings")
            .ok_or_else(|| ParseError::InvalidFormat("Missing 'callings' field".to_string()))?;

        let org = Organization {
            name: name.to_string(),
            children: process_child_orgs(children)?,
            callings: process_callings(callings)?,
        };

        orgs.insert(org.name.clone(), org);
    }

    Ok(orgs)
}

fn process_child_orgs(parsed_children: &Value) -> Result<Vec<Organization>, ParseError> {
    let mut child_orgs = Vec::new();

    let children_array = parsed_children
        .as_array()
        .ok_or_else(|| ParseError::InvalidFormat("Expected array for children".to_string()))?;

    for parsed_child in children_array {
        let obj = parsed_child.as_object().ok_or_else(|| {
            ParseError::InvalidFormat("Expected object for child organization".to_string())
        })?;

        let child_name = obj.get("name").and_then(|v| v.as_str()).ok_or_else(|| {
            ParseError::InvalidFormat("Missing or invalid 'name' field in child".to_string())
        })?;

        let children = obj.get("children").ok_or_else(|| {
            ParseError::InvalidFormat("Missing 'children' field in child".to_string())
        })?;

        let callings = obj.get("callings").ok_or_else(|| {
            ParseError::InvalidFormat("Missing 'callings' field in child".to_string())
        })?;

        let child_org = Organization {
            name: child_name.to_string(),
            children: process_child_orgs(children)?,
            callings: process_callings(callings)?,
        };

        child_orgs.push(child_org);
    }

    Ok(child_orgs)
}

fn process_callings(parsed_callings: &Value) -> Result<Vec<Calling>, ParseError> {
    let mut callings = Vec::new();

    let callings_array = parsed_callings
        .as_array()
        .ok_or_else(|| ParseError::InvalidFormat("Expected array for callings".to_string()))?;

    for parsed_calling in callings_array {
        let obj = parsed_calling
            .as_object()
            .ok_or_else(|| ParseError::InvalidFormat("Expected object for calling".to_string()))?;

        let calling_name = obj
            .get("position")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ParseError::InvalidFormat("Missing or invalid 'position' field".to_string())
            })?;

        let member_name_value = obj
            .get("memberName")
            .ok_or_else(|| ParseError::InvalidFormat("Missing 'memberName' field".to_string()))?;

        let calling = if member_name_value.is_null() {
            Calling {
                name: calling_name.to_string(),
                member: None,
            }
        } else {
            let member_name = member_name_value.as_str().ok_or_else(|| {
                ParseError::InvalidFormat("Invalid 'memberName' field".to_string())
            })?;

            let held_calling_since_str = obj
                .get("activeDate")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ParseError::InvalidFormat("Missing or invalid 'activeDate' field".to_string())
                })?;

            if held_calling_since_str.len() != 8 {
                return Err(ParseError::InvalidFormat(format!(
                    "Invalid date format: {}",
                    held_calling_since_str
                )));
            }

            let year = held_calling_since_str[0..4].parse::<u16>().map_err(|_| {
                ParseError::InvalidFormat(format!(
                    "Invalid year: {}",
                    &held_calling_since_str[0..4]
                ))
            })?;
            let month = held_calling_since_str[4..6].parse::<u8>().map_err(|_| {
                ParseError::InvalidFormat(format!(
                    "Invalid month: {}",
                    &held_calling_since_str[4..6]
                ))
            })?;
            let day = held_calling_since_str[6..8].parse::<u8>().map_err(|_| {
                ParseError::InvalidFormat(format!("Invalid day: {}", &held_calling_since_str[6..8]))
            })?;

            let held_calling_since = Date { year, month, day };

            let set_apart = obj
                .get("setApart")
                .and_then(|v| v.as_bool())
                .ok_or_else(|| {
                    ParseError::InvalidFormat("Missing or invalid 'setApart' field".to_string())
                })?;

            let comma_pos = member_name.find(',').ok_or_else(|| {
                ParseError::InvalidFormat(format!("Invalid member name format: {}", member_name))
            })?;

            if comma_pos + 2 >= member_name.len() {
                return Err(ParseError::InvalidFormat(format!(
                    "Invalid member name format: {}",
                    member_name
                )));
            }

            let email = obj
                .get("memberEmail")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let phone = obj
                .get("memberPhone")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let member = MemberWithCalling {
                given_names: member_name[comma_pos + 2..].to_string(),
                last_name: member_name[0..comma_pos].to_string(),
                email,
                phone,
                held_calling_since,
                set_apart,
            };

            Calling {
                name: calling_name.to_string(),
                member: Some(member),
            }
        };

        callings.push(calling);
    }

    Ok(callings)
}
