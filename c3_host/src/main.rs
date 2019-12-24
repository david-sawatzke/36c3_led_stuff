use rand::prelude::*;
use serialport::open_with_settings;
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
    let mut settings: serialport::SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(10);
    settings.baud_rate = 9600;
    let mut serial = open_with_settings(&opt.tty, &settings).expect("Open serial port");
    let mut rng = rand::thread_rng();
    loop {
        let mut image = rng.gen_range(0, 5);
        let delay = rng.gen_range(10, 20);
        serial.write(&[image]).expect("Writing to serial port");
        println!("{}", image);
        thread::sleep(Duration::from_millis(50 * delay));
    }
}
