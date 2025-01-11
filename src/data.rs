#![allow(dead_code)]

use std::fmt::Display;

#[derive(Copy, Clone, Debug)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[derive(Debug)]
pub struct MemberWithCalling<'a> {
    pub given_names: &'a str,
    pub last_name: &'a str,
    pub email: Option<&'a str>,
    pub phone: Option<&'a str>,

    pub held_calling_since: Date,
    pub set_apart: bool,
}

#[derive(Debug)]
pub struct Calling<'a> {
    pub name: &'a str,
    pub member: Option<MemberWithCalling<'a>>,
}

#[derive(Debug)]
pub struct Organization<'a> {
    pub name: &'a str,
    pub children: Vec<Organization<'a>>,
    pub callings: Vec<Calling<'a>>,
}

#[derive(Debug)]
pub struct CsvRecord<'a> {
    pub member_last_name: &'a str,
    pub member_given_names: &'a str,
    pub calling: &'a str,
    pub sub_sub_organization: &'a str,
    pub sub_organization: &'a str,
    pub organization: &'a str,
    pub held_calling_since: Date,
    pub set_apart: bool,
    pub member_email: Option<&'a str>,
    pub member_phone: Option<&'a str>,
}

impl Display for CsvRecord<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",{},{},\"{}\",\"{}\"",
            self.member_last_name,
            self.member_given_names,
            self.calling,
            self.sub_sub_organization,
            self.sub_organization,
            self.organization,
            self.held_calling_since,
            self.set_apart,
            self.member_email.unwrap_or(""),
            self.member_phone.unwrap_or("")
        )
    }
}
