extern crate structopt;

#[macro_use]
extern crate log;
extern crate env_logger;
use std::error::Error;
extern crate icmp;
use icmp::configuration;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "icmp-rust-agent")]
struct Opt {
    /// config file
    #[structopt(short = "c", long = "config")]
    config: String,
}

fn main() -> Result<(), Box<Error>> {
    env_logger::init();

    let opt = Opt::from_args();

    let setting = configuration::Settings::from(opt.config)?;
    debug!("Starting icmp-rust-agent with {:?}", setting);
    
    Ok(())
}
