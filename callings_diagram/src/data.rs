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
        let month = match self.month {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            m => panic!("Invalid month: {}", m),
        };

        write!(f, "{}, {} {}", self.year, month, self.day)
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
