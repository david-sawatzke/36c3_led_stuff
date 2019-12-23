use rand::prelude::*;
use serialport::open;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;

/// The host side part of `c3_led_stuff`
#[derive(StructOpt, Debug)]
#[structopt(name = "c3_host")]
struct Opt {
    /// The serial port
    #[structopt(short, long)]
    tty: PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    let mut serial = open(&opt.tty).expect("Open serial port");
    let mut rng = rand::thread_rng();
    loop {
        let image = rng.gen_range(0, 5);
        let delay = rng.gen_range(10, 20);
        serial.write(&[image]).unwrap();
        thread::sleep(Duration::from_millis(50 * delay));
    }
}
