use std::{env, error::Error};

use subnetcalc::{NetworkHosts, IpAddressBlock};

const HELP: &str = "
TODO";

#[derive(Debug)]
enum CliOption {
    Run(Config),
    Help,
}

impl CliOption {
    fn parse(mut args: env::Args) -> Result<Self, Box<dyn Error + 'static>> {
        // FIXME: change error type

        // The first element is the executable name, so we ignore it
        args.next();

        let ipaddr = match args.next() {
            Some(text) if text == "--help" => return Ok(CliOption::Help),
            Some(ipaddr) => ipaddr.parse()?,
            None => Err("missing first option")?,
        };

        let option = match args.next() {
            Some(opt) if opt == "--vlsm" => {
                let results = args
                    .map(|arg| arg.parse::<NetworkHosts>())
                    .collect::<Result<Vec<_>, _>>()?;
                RunOptions::VLSM(results)
            }
            Some(opt) if opt == "--flsm" => {
                let num = args
                    .next()
                    .ok_or("missing option")?
                    .parse::<usize>()
                    .map_err(|_| "invalid option")?;
                RunOptions::FLSM(num)
            }
            Some(_) => Err("invalid option")?,
            None => Err("missing option")?,
        };

        Ok(CliOption::Run(Config { ipaddr, option }))
    }
}

#[derive(Debug)]
enum RunOptions {
    FLSM(usize),
    VLSM(Vec<NetworkHosts>),
}

#[derive(Debug)]
struct Config {
    ipaddr: IpAddressBlock,
    option: RunOptions,
}

fn show_subnets(config: Config) -> Result<(), Box<dyn Error + 'static>> {
    dbg!(config);
    Ok(())
}

fn run() -> Result<(), Box<dyn Error + 'static>> {
    match CliOption::parse(env::args())? {
        CliOption::Run(config) => show_subnets(config),
        CliOption::Help => {
            println!("{}", HELP);
            Ok(())
        }
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}
