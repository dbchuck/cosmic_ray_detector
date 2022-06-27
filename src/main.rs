use clap::{Arg, Command};
use rayon::prelude::*;
use std::io::{self, Write};
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() -> Result<(), String> {
    let conf: Config = match Config::new() {
        Ok(c) => c,
        Err(e) => return Err(e),
    };

    let size: usize = match parse_size_string(conf.memory_to_occupy) {
        Ok(s) => s,
        Err(e) => return Err(e),
    };

    let verbose: bool = conf.verbose;

    if verbose {
        println!("Using {} bits of RAM as detector", 8 * size);
        io::stdout().flush().unwrap();
    }

    //Instead of building a detector out of scintillators and photo multiplier tubes,
    //we just allocate some zeros on this here computer
    let mut detector_mass: Vec<u8> = vec![0; size];
    //less exciting, but much cheaper

    let start: Instant = Instant::now();
    let sleep_duration: Duration = Duration::from_millis(conf.check_delay);

    if verbose {
        if conf.check_delay == 0 {
            println!("Will do continuous integrity checks");
        } else {
            println!("Waiting {:?} between integrity checks", sleep_duration);
        }
        if conf.parallel {
            println!("Checking memory integrity in parallel");
        }
    }

    let mut checks: u64 = 1;
    let mut everything_is_fine = true;
    //Some feedback for the user that the program is still running
    if verbose {
        print!("Waiting for first check");
        io::stdout().flush().unwrap();
    }
    loop {
        while everything_is_fine {
            //We're not gonna miss any events by being too slow
            sleep(sleep_duration);
            //Check if all the bytes are still zero
            everything_is_fine = if conf.parallel {
                detector_mass.par_iter().all(|i| *i == 0)
            } else {
                detector_mass.iter().all(|i| *i == 0)
            };
            if verbose {
                print!("\rIntegrity checks passed: {}", checks);
                io::stdout().flush().unwrap();
            }
            checks += 1;
        }

        println!();

        println!(
            "Detected a bitflip after {:?} on integrity check number {}",
            start.elapsed(),
            checks
        );
        let index = detector_mass.iter().position(|&r| r != 0).unwrap();
        println!(
            "Bit flip in byte {}, it became {}",
            index,
            detector_mass[index]
        );

        //Reset detector!
        detector_mass[index] = 0;
        everything_is_fine = true;
    }
}

fn parse_size_string(size_string: String) -> Result<usize, String> {
    match size_string.parse() {
        Ok(t) => Ok(t),
        Err(_) => {
            let chars: Vec<char> = size_string.chars().collect();
            let len: usize = chars.len();
            let last: char = match chars.last() {
                Some(l) => *l,
                None => return Err("memory_to_occupy was empty".to_owned()),
            };

            if (last != 'B' && last != 'b') || len < 2 {
                return Err("memory_to_occupy was incorrectly formatted".to_owned());
            }

            let next_to_last: char = chars[len - 2];

            let si_prefix_factor: f64 = if next_to_last == 'k' {
                1e3
            } else if next_to_last == 'M' {
                1e6
            } else if next_to_last == 'G' {
                1e9
            } else if next_to_last == 'T' {
                1e12
            } else if !next_to_last.is_digit(10) {
                return Err("unsupported memory size".to_owned());
            } else {
                return Err("could not parse memory size".to_owned());
            };

            let bit_size: f64 = if last == 'B' { 1.0 } else { 1.0 / 8.0 };

            let factor: usize = (si_prefix_factor * bit_size) as usize;

            let digits: String = chars[..len - 2].iter().collect();
            let number: usize = match digits.parse() {
                Ok(n) => n,
                Err(e) => return Err(e.to_string()),
            };

            Ok(number * factor)
        }
    }
}

struct Config {
    memory_to_occupy: String,
    check_delay: u64,
    parallel: bool,
    verbose: bool,
}

impl Config {
    fn new() -> Result<Self, String> {
        let memory_default = "1GB";
        let delay_default = "30000";

        let args = Command::new("cosmic ray detector")
            .about("monitors memory for bit-flips (won't work on ECC memory)")
            .version("v0.1.0")
            .author("Johanna Sörngård (jsorngard@gmail.com)")
            .arg(
                Arg::with_name("memory_size")
                    .help(&*format!("the size of the memory to monitor for bit flips, understands e.g. 200, 5kB, 2GB and 3Mb (default: {})", memory_default))
                    .short('m')
                    .takes_value(true)
                    .required(false)
                    .default_value(memory_default),
            )
            .arg(
                Arg::with_name("check_delay")
                    .help(&*format!("an optional delay in between each integrity check (in milliseconds) (default: {})", delay_default))
                    .short('d')
                    .takes_value(true)
                    .required(false)
                    .default_value(delay_default),
            )
            .arg(
                Arg::with_name("parallel")
                    .help("whether to run the integrity check in parallel to speed it up")
                    .long("parallel")
                    .takes_value(false)
                    .required(false),
            )
            .arg(
                Arg::with_name("quiet")
                    .help("whether to only print information about eventual detections")
                    .long("quiet")
                    .takes_value(false)
                    .required(false),
            )
            .get_matches();

        let parallel = args.is_present("parallel");

        let verbose = !args.is_present("quiet");

        let memory_to_occupy = args.value_of("memory_size").unwrap_or("1GB").to_owned();

        let check_delay: u64 = match args.value_of("check_delay") {
            Some(s) => match s.parse() {
                Ok(t) => t,
                Err(e) => return Err(e.to_string()),
            },
            None => 0,
        };

        Ok(Config {
            memory_to_occupy,
            check_delay,
            parallel,
            verbose,
        })
    }
}
