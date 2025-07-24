use std::fmt;

#[derive(Debug)]
pub enum DataError {
    InvalidMonth(u8),
    InvalidDay { day: u8, month: u8 },
}

impl std::error::Error for DataError {}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::InvalidMonth(month) => write!(f, "Invalid month: {}", month),
            DataError::InvalidDay { day, month } => {
                write!(f, "Invalid day: {} for month {}", day, month)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl Date {
    pub fn to_string(&self) -> Result<String, DataError> {
        let (month_str, days_in_month) = match self.month {
            1 => ("January", 31),
            2 => {
                let is_leap =
                    (self.year % 4 == 0) && (self.year % 100 != 0 || self.year % 400 == 0);
                ("February", if is_leap { 29 } else { 28 })
            }
            3 => ("March", 31),
            4 => ("April", 30),
            5 => ("May", 31),
            6 => ("June", 30),
            7 => ("July", 31),
            8 => ("August", 31),
            9 => ("September", 30),
            10 => ("October", 31),
            11 => ("November", 30),
            12 => ("December", 31),
            _ => return Err(DataError::InvalidMonth(self.month)),
        };

        if self.day == 0 || self.day > days_in_month {
            return Err(DataError::InvalidDay {
                day: self.day,
                month: self.month,
            });
        }

        let ordinal_suffix = match self.day {
            1 | 21 | 31 => "st",
            2 | 22 => "nd",
            3 | 23 => "rd",
            _ => "th",
        };

        Ok(format!(
            "{}, {} {}{}",
            self.year, month_str, self.day, ordinal_suffix
        ))
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MemberWithCalling {
    pub given_names: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub held_calling_since: Date,
    pub set_apart: bool,
}

#[derive(Debug, Clone)]
pub struct Calling {
    pub name: String,
    pub member: Option<MemberWithCalling>,
}

#[derive(Debug, Clone)]
pub struct Organization {
    pub name: String,
    pub children: Vec<Organization>,
    pub callings: Vec<Calling>,
}
