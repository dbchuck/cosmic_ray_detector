use std::fs::File;
use std::num::ParseIntError;
use clap::Parser;
use std::usize;

const DELAY_DEFAULT: u64 = 30000;

/// Monitors memory for bit-flips (won't work on ECC memory).
/// The chance of detection scales with the physical size of your DRAM modules
/// and the percentage of them you allocate to this program.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, required = false, value_parser(parse_size_string), default_value_t = 0)]
    /// The size of the memory to monitor for bitflips, understands e.g. 200, 5kB, 2GB and 3Mb. If this is specified or set to a non-zero value, the program will not automatically fill all available memory
    pub memory_to_occupy: usize,

    #[arg(short, required = false, default_value_t = DELAY_DEFAULT)]
    /// An optional delay in between each integrity check (in milliseconds)
    pub delay_between_checks: u64,

    #[arg(long, required = true)]
    /// The longitude of where the computer is that is running the program
    pub longitude: String,

    #[arg(long, required = true)]
    /// The latitude of where the computer is that is running the program
    pub latitude: String,

    #[arg(long, required = true, value_parser(parse_logging_file_path))]
    /// The file path to save bitflip results
    pub file_path: String,

    #[arg(short, required = false, long, default_value_t = true)]
    /// Whether to print extra information
    pub verbose: bool,
}

/// Parses a string describing a number of bytes into an integer.
/// The string can use common SI prefixes as well, like '4GB' or '30kB'.
pub fn parse_size_string(size_string: &str) -> Result<usize, String> {
    println!("test");
    let chars: Vec<char> = size_string.chars().collect();
    let len: usize = chars.len();
    let last: char = match chars.last() {
        Some(l) => *l,
        None => return Err("memory_to_occupy was empty".into()),
    };

    if last == '0' {
        // Use up all available memory
        return Ok(0);
    }

    if (last != 'B' && last != 'b') || len < 2 {
        return Err("Unable to parse memory_to_occupy".into());
    }

    let next_to_last: char = chars[len - 2];

    let si_prefix_factor: f64 = if next_to_last == 'k' {
        1e3
    } else if next_to_last == 'M' {
        1e6
    } else if next_to_last == 'G' {
        1e9
    } else if next_to_last == 'T' {
        //Future proofing...
        1e12
    } else if next_to_last == 'P' {
        //HOW?!
        1e15
    } else if !next_to_last.is_ascii_digit() {
        return Err("Unsupported memory size".into());
    } else {
        return Err("Could not parse memory size".into());
    };

    let bit_size: f64 = if last == 'B' { 1.0 } else { 1.0 / 8.0 };

    let factor: usize = (si_prefix_factor * bit_size) as usize;

    let digits: String = chars[..len - 2].iter().collect();
    let number: usize = digits.parse().map_err(|e: ParseIntError| e.to_string())?;

    Ok(number * factor)
}

pub fn parse_logging_file_path(file_path: &str) -> Result<String, String> {
    match File::open(file_path) {
        Ok(_open_file) => println!("Found existing file {}", file_path),
        Err(_open_err) => {
            println!("File {} does not exist, trying to create it.", file_path);
            match File::create(file_path) {
                Ok(_create_file) => println!("Created file {}", file_path),
                Err(create_err) => {
                    // Unable to create file
                    return Err(format!("Unable to create file: {}", create_err));
                }
            }
        }
    }

    println!("Logging bitflips to {}", file_path);
    return Ok(file_path.to_string());
}