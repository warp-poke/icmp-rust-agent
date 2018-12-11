#![feature(await_macro, async_await, futures_api)]
#![feature(pin)]
#![feature(arbitrary_self_types)]

use structopt::StructOpt;
use log::debug;

mod configuration;
mod kafka;

use crate::configuration::Settings;
use crate::kafka::run_async_handler;

#[derive(StructOpt, Debug)]
#[structopt(name = "icmp-rust-agent")]
struct Opt {
    /// config file
    #[structopt(short = "c", long = "config")]
    config: String,
}

fn main() {
    env_logger::init();

    let opt = Opt::from_args();

    let setting = Settings::from(opt.config).expect("invalid configuration file");
    debug!("Starting icmp-rust-agent with {:?}", setting);

    run_async_handler(setting);
}
