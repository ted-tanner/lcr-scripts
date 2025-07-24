use std::env;
use std::fs;
use std::process;

mod config;
mod data;
mod generate;
mod parse;

use generate::diagram_file_contents;
use parse::orgs_from_lcr_data;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("invalid args\nusage: callings-diagram <input file> <output file>");
        process::exit(1);
    }

    let config_file_contents = match fs::read_to_string("diagram-config.json") {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!(
                "Expected 'diagram-config.json' file in current directory: {}",
                err
            );
            process::exit(1);
        }
    };

    let conf = match config::parse(&config_file_contents) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Failed to parse config file 'diagram-config.json': {}", err);
            process::exit(1);
        }
    };

    let input_file_contents = match fs::read_to_string(&args[1]) {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!("Failed to read input file '{}': {}", args[1], err);
            process::exit(1);
        }
    };

    let orgs = match orgs_from_lcr_data(&input_file_contents) {
        Ok(orgs) => orgs,
        Err(err) => {
            eprintln!("Failed to parse input file '{}': {}", args[1], err);
            process::exit(1);
        }
    };

    let output_file_contents = match diagram_file_contents(&orgs, &conf) {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!("Failed to generate diagram file contents: {}", err);
            process::exit(1);
        }
    };

    if let Err(err) = fs::write(&args[2], output_file_contents) {
        eprintln!("Failed to write to output file '{}': {}", args[2], err);
        process::exit(1);
    }

    println!("Successfully wrote diagram to {}", args[2]);
}
