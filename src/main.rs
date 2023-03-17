use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

mod config;
mod detector;

use crate::{config::Args, detector::Detector};

use clap::Parser;
use sysinfo::{RefreshKind, System, SystemExt};

const SWAP_DELTA_THRESHOLD: u64 = 10_000_000; // 10MB
const FREE_MEM_THRESHOLD: u64 = 50_000_000; // 50MB

fn main() -> Result<(), Box<dyn Error>> {
    let conf: Args = Args::parse();

    let mut size: usize = conf.memory_to_occupy;
    let verbose: bool = conf.verbose;
    let check_delay: u64 = conf.delay_between_checks;

    let sleep_duration: Duration = Duration::from_millis(check_delay);

    let rk = RefreshKind::new().with_memory();
    let mut sys_info = System::new_with_specifics(rk);
    let previous_swap_usage = sys_info.used_swap();
    let mut increment;
    let mut total_size= size;

    if verbose {
        println!("\n------------ Runtime settings ------------");
        if size == 0 {
            println!("Using all available RAM as detector");
            // Calculate 1/2 of the available memory
            // Evaluate how much is left after attempting to use all the memory. Check if any swap has been used
            // If swap has been used, decrement by 1/2 of the original amount
            // If swap has not been used, increase by 1/2 of the previous amount until the amount is less than 10MB increments

            let mut init_detectors = vec![];
            // Start at 1/2 of available memory
            size = (sys_info.available_memory() / 2) as usize;
            total_size = size;
            increment = size;
            print_detector_stats(&sys_info, size);
            let mut detector = Detector::new(0, size);
            detector.write(42);
            init_detectors.insert(0, detector);
            loop {
                sys_info.refresh_specifics(rk);
                increment = increment / 2;
                if sys_info.total_swap() > 0 {
                    // If there is swap
                    if sys_info.used_swap() - previous_swap_usage > SWAP_DELTA_THRESHOLD {
                        // Swap increased, decrease amount of memory used
                        // Remove previous detector
                        init_detectors.remove(0);
                        total_size -= size;
                    }
                    else {
                        if FREE_MEM_THRESHOLD > increment as u64 {
                            break;
                        }
                        // Swap usage did not increase, increase amount of memory to use
                    }

                    size = size - increment;
                    total_size += size;
                }
                else {
                    // No swap
                    if 0 > (sys_info.available_memory() as i64 - FREE_MEM_THRESHOLD as i64) as i64 {
                        // Passed free memory threshold, reduce memory consumption
                        // Remove previous detector
                        init_detectors.remove(0);
                        total_size -= size;
                    }
                    else {
                        // Only increase until there is 50MB spare
                        if FREE_MEM_THRESHOLD > increment as u64 {
                            break;
                        }
                    }

                    size = size - increment;
                    total_size += size;
                }

                print_detector_stats(&sys_info, size);

                let mut detector = Detector::new(0, size);
                detector.write(42);
                init_detectors.insert(0, detector);
            }

            size = total_size;
        }
        println!("Using {} bits ({}) of RAM as detector", size, mem_size(size as u64));

        if check_delay == 0 {
            println!("Will do continuous integrity checks");
        } else {
            println!("Waiting {:?} between integrity checks", sleep_duration);
        }
        println!("Checking memory integrity in parallel");
        println!("------------------------------------------\n");

        print!("Allocating detector memory...");
        stdout().flush()?;
    }



    // Instead of building a detector out of scintillators and photo multiplier tubes,
    // we just allocate some memory on this here computer.
    let mut detector = Detector::new(0, size);
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
        // TODO have a thread watching to see if the free memory buffer begins to decrease (in which case, shrink the detector) instead of relying on swap.

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

fn print_detector_stats(sys_info: &System, size: usize) {
    println!("Total: {} Free: {} Available: {} Used: {} Total-Used: {}", sys_info.total_memory(), sys_info.free_memory(), sys_info.available_memory(), sys_info.used_memory(), sys_info.total_memory() - sys_info.used_memory());
    println!("Total: {} Free: {} Available: {} Used: {} Total-Used: {}", mem_size(sys_info.total_memory()), mem_size(sys_info.free_memory()), mem_size(sys_info.available_memory()), mem_size(sys_info.used_memory()), mem_size(sys_info.total_memory() - sys_info.used_memory()));
    println!("Creating next detector of size {} ({})", size, mem_size(size as u64));
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
