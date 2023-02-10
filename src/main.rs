use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use clap::Parser;

mod config;
mod detector;

use crate::{config::Args, detector::Detector};

fn main() -> Result<(), Box<dyn Error>> {
    let conf: Args = Args::parse();

    let size: usize = conf.memory_to_occupy.get();
    let verbose: bool = conf.verbose;
    let parallel: bool = conf.parallel;
    let check_delay: u64 = conf.delay_between_checks;

    let sleep_duration: Duration = Duration::from_millis(check_delay);

    if verbose {
        println!("\n------------ Runtime settings ------------");
        println!("Using {} bits ({}) of RAM as detector", 8 * size, mem_size((8 * size) as u64));

        if check_delay == 0 {
            println!("Will do continuous integrity checks");
        } else {
            println!("Waiting {:?} between integrity checks", sleep_duration);
        }
        if parallel {
            println!("Checking memory integrity in parallel");
        }
        println!("------------------------------------------\n");

        print!("Allocating detector memory...");
        stdout().flush()?;
    }

    // Instead of building a detector out of scintillators and photo multiplier tubes,
    // we just allocate some memory on this here computer.
    let mut detector = Detector::new(parallel, 0, size);
    // Less exciting, much less accurate and sensitive, but much cheaper

    // Avoid the pitfalls of virtual memory by writing nonzero values to the allocated memory.
    detector.write(42);

    if verbose {
        println!("done");
        println!("Adding start entry to log file");
    }

    let mut file: File;
    match OpenOptions::new()
        .write(true)
        .append(true)
        .open(conf.file_path) {
        Ok(open_file) => file = open_file,
        Err(err) => return Err(Box::new(err))
    };

    let start = SystemTime::now();
    let unix_timestamp = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let start_entry_str = format!("{},{},,,{},{}\n", unix_timestamp.as_millis(), conf.delay_between_checks, conf.latitude, conf.longitude);
    file.write(start_entry_str.as_bytes()).expect("An error with opening the file occurred");
    file.flush()?;
    file.sync_data()?;

    if verbose {
        println!("\nBeginning detection loop");
    }

    let mut total_checks: u64 = 1;
    let mut checks_since_last_bitflip: u64 = 1;
    let mut everything_is_fine: bool;
    let start: Instant = Instant::now();
    loop {
        // Reset detector!
        if verbose {
            print!("Zeroing detector memory... ");
            stdout().flush()?;
        }
        detector.reset();
        everything_is_fine = true;

        // Some feedback for the user that the program is still running
        if verbose {
            println!("done");
            print!("Waiting for first check");
            stdout().flush()?;
        }

        while everything_is_fine {
            // We're not gonna miss any events by being too slow
            sleep(sleep_duration);
            // Check if all the bytes are still zero
            everything_is_fine = detector.is_intact();
            if verbose {
                print!("\rIntegrity checks passed: {}", total_checks);
                stdout().flush()?;
            }
            total_checks += 1;
            checks_since_last_bitflip += 1;
        }

        let end_check_time = SystemTime::now();
        let end_check_time_unix_timestamp = end_check_time
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        println!(
            "\nDetected a bitflip after {:?} on integrity check number {}",
            start.elapsed(),
            total_checks
        );

        let log_entry_str: String;
        match detector.find_index_of_changed_element() {
            Some(index) => {
                println!(
                    "Bitflip in byte at index {}, it became {}",
                    index,
                    // unwrap() is okay since we already found the index of the value in the detector earlier.
                    detector.get(index).unwrap(),
                );
                log_entry_str = format!("{},{},{},{},{},{},{}\n", unix_timestamp.as_millis(), conf.delay_between_checks, checks_since_last_bitflip, 0, end_check_time_unix_timestamp.as_millis(), conf.latitude, conf.longitude);
            },
            None => {
                println!(
                    "The same bit flipped back before we could find which one it was! Incredible!"
                );
                log_entry_str = format!("{},{},{},{},{},{},{}\n", unix_timestamp.as_millis(), conf.delay_between_checks, checks_since_last_bitflip, 1, end_check_time_unix_timestamp.as_millis(), conf.latitude, conf.longitude);
            },
        }

        file.write(log_entry_str.as_bytes()).expect("An error with opening the file occurred");
        file.flush()?;
        file.sync_data()?;

        checks_since_last_bitflip = 0;
    }
}

/// Get human readable byte sizes
fn mem_size(mem_size: u64) -> String {
    let mut mem_units: Vec<&str> = vec![" TiB", " GiB", " MiB", " KiB", " B"];
    let mut mem_size: f32 = mem_size as f32;
    let mut unit: String = mem_units.pop().unwrap().parse().unwrap();
    while mem_size > 1024 as f32 {
        mem_size = mem_size / 1024.0;
        unit = mem_units.pop().unwrap().parse().unwrap();
    }
    return mem_size.to_string() + unit.as_str();
}
