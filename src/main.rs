extern crate structopt;
#[macro_use]
extern crate log;
extern crate config;
extern crate env_logger;
extern crate futures;
extern crate futures_cpupool;
extern crate rdkafka;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio;
extern crate tokio_ping;
extern crate trust_dns_resolver;

mod configuration;
mod kafka;

use crate::configuration::Settings;
use crate::kafka::run_async_handler;
use std::error::Error;
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

    let setting = Settings::from(opt.config)?;
    debug!("Starting icmp-rust-agent with {:?}", setting);

    run_async_handler(&setting)?;

    Ok(())
}
