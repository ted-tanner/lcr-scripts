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
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            m => panic!("Invalid month: {}", m),
        };

        let ordinal_suffix = match self.day {
            1 | 21 | 31 => "st",
            2 | 22 => "nd",
            3 | 23 => "rd",
            _ => "th",
        };

        write!(f, "{}, {} {}{}", self.year, month, self.day, ordinal_suffix)
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
