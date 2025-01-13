use std::{fs::File, io::{Read, Write}};

use regex::Regex;

fn main() {
    let input_file_path = std::env::args()
        .nth(1)
        .expect("usage: date-transform <input_file> <output_file>");
    let output_file_path = std::env::args()
        .nth(2)
        .expect("usage: date-transform <input_file> <output_file>");

    let mut input_file = File::open(&input_file_path).expect("Could not open input file");

    let mut file_contents = String::new();
    input_file.read_to_string(&mut file_contents).expect("Could not read file");

    let date_regex = Regex::new(r"(?<year>20\d{2})-(?<month>[0-1]?\d)-(?<day>[0-3]?\d)").expect("Invalid regex");

    let new_contents = date_regex.replace_all(&file_contents, |captures: &regex::Captures| {
        let year = captures.name("year").expect("Could not read year").as_str();
        let month = captures.name("month").expect("Could not read month").as_str();
        let day = captures.name("day").expect("Could not read day").as_str();

        let month = match month {
            "01" | "1" => "Jan",
            "02" | "2" => "Feb",
            "03" | "3" => "Mar",
            "04" | "4" => "Apr",
            "05" | "5" => "May",
            "06" | "6" => "Jun",
            "07" | "7" => "Jul",
            "08" | "8" => "Aug",
            "09" | "9" => "Sep",
            "10" => "Oct",
            "11" => "Nov",
            "12" => "Dec",
            m => panic!("Invalid month: {}", m),
        };

        let day = day.trim_start_matches('0');

        format!("{}, {} {}", year, month, day)
    });

    let mut output_file = File::create(&output_file_path).expect("Could not create output file");
    output_file.write_all(new_contents.as_bytes()).expect("Could not write to output file");
}
