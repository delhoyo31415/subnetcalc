use std::{env, error::Error};

use subnetcalc::{IpAddressBlock, NetworkHosts};

const HELP: &str = "usage: subnetcalc IPADDR_BLOCK OPTION ARGS

* IPADDR_BLOCK: Address block which is going to be divided
* OPTION: Strategy to follow to divide the address block
    --vlsm | -v: Uses the Variable Length Subnet Mask (VLSM) strategy. In this case, ARGS is a 
    space separated set of numbers which represent the number of host each network is going to have
    --flsm | -f: Uses the Fixed Length Subnet Mask (FLSM) strategy. In this case ARGS is the number 
    of subnets you want";

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

        let ipaddr_block = match args.next() {
            Some(text) if text == "--help" || text == "-h" => return Ok(CliOption::Help),
            Some(ipaddr_block) => ipaddr_block.parse()?,
            None => Err("missing first option")?,
        };

        let option = match args.next() {
            Some(opt) if opt == "--vlsm" || opt == "-v" => {
                let results = args
                    .map(|arg| arg.parse::<NetworkHosts>())
                    .collect::<Result<Vec<_>, _>>()?;
                RunOptions::VLSM(results)
            }
            Some(opt) if opt == "--flsm" || opt == "-f" => {
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

        Ok(CliOption::Run(Config {
            ipaddr_block,
            option,
        }))
    }
}

#[derive(Debug)]
enum RunOptions {
    FLSM(usize),
    VLSM(Vec<NetworkHosts>),
}

#[derive(Debug)]
struct Config {
    ipaddr_block: IpAddressBlock,
    option: RunOptions,
}

fn show_subnets(config: Config) -> Result<(), Box<dyn Error + 'static>> {
    match config.option {
        RunOptions::FLSM(target_subnets) => match config.ipaddr_block.subnet_flsm(target_subnets) {
            Some(result) => {
                for (idx, subnet) in result.iter().enumerate() {
                    println!("{}) {}", idx + 1, subnet);
                }
            }
            None => println!(
                "It is not possible to divide {} in {} subnetworks using FLSM",
                config.ipaddr_block, target_subnets
            ),
        },
        RunOptions::VLSM(nets) => match config.ipaddr_block.subnet_vlsm(nets) {
            Some(result) => {
                for (idx, (net_hosts, nets)) in result.iter().enumerate() {
                    println!("{}) {} - {}", idx + 1, net_hosts.hosts(), nets);
                }
            }
            None => println!(
                "It is not possible to subnet {} using VLSM with those requirements",
                config.ipaddr_block
            ),
        },
    }
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
