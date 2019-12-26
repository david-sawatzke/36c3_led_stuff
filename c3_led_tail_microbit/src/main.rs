#![no_main]
#![no_std]

#[allow(unused)]
use panic_semihosting;

use microbit::hal;
use ws2812_spi as ws2812;

use hal::delay::Delay;
use hal::hi_res_timer::TimerFrequency;
use hal::prelude::*;
use hal::serial::{Rx, Serial};
use hal::spi::{self, Spi};
use hal::time::Hfticks;
use hal::timer::CountDownTimer;
use nb::block;
use smart_leds::SmartLedsWrite;

use c3_led_tail::Elements;

#[rtfm::app(device = microbit, peripherals = true)]
const APP: () = {
    struct Resources {
        ws: ws2812::Ws2812<Spi<hal::nrf51::SPI1>>,
        delay: Delay,
        timer: CountDownTimer<hal::nrf51::TIMER1>,
        serial: Rx<hal::nrf51::UART0>,
    }

    #[init]
    fn init(context: init::Context) -> init::LateResources {
        let p = context.device;

        let gpio = p.GPIO.split();

        let delay = Delay::new(p.TIMER0);

        let (rx, tx, sck, mosi, miso) = (
            // 24 & 25 are internal serial pins
            gpio.pin19.into_floating_input().into(),
            gpio.pin20.into_push_pull_output().into(),
            // Spi
            gpio.pin13.into_push_pull_output().into(),
            gpio.pin15.into_push_pull_output().into(),
            gpio.pin14.into_floating_input().into(),
        );

        let spi_pins = spi::Pins { sck, mosi, miso };
        // Luckily has a frequency of 4 MHz
        let spi = Spi::new(p.SPI1, spi_pins);

        let ws = ws2812::Ws2812::new(spi);

        let mut timer = CountDownTimer::new(p.TIMER1, TimerFrequency::Freq31250Hz);
        // 20 Hertz
        timer.start(Hfticks::from_ms(50));
        // Get the rx serial instance
        let serial = Serial::uart0(p.UART0, tx, rx, hal::serial::BAUD9600)
            .split()
            .1;

        init::LateResources {
            delay,
            timer,
            serial,
            ws,
        }
    }

    #[idle(resources = [delay, serial, timer, ws])]
    fn idle(c: idle::Context) -> ! {
        // Matching resources in c3_display
        let mut elements = Elements::new(400, 15);
        // Chosen by fair dice roll
        let mut rand = oorandom::Rand32::new(0);
        // On average add a new color every 15 steps
        let mut steps = rand.rand_range(10..20);
        // Do something when host isn't active yet
        // Drops first byte
        while c.resources.serial.read().is_err() {
            steps -= 1;
            if steps == 0 {
                steps = rand.rand_range(10..20);
                elements
                    .add_predefined(rand.rand_range(0..c3_led_tail::COLORS.len() as u32) as usize)
                    .unwrap();
            }
            block!(c.resources.timer.wait()).unwrap();
        }
        // Host driven mode
        loop {
            if let Ok(byte) = c.resources.serial.read().map(|x| x as usize) {
                if byte < c3_led_tail::COLORS.len() {
                    elements.add_predefined(byte).unwrap();
                }
            }
            if c.resources.timer.wait().is_ok() {
                elements.step();
                c.resources
                    .ws
                    .write(smart_leds::gamma(elements.iter()))
                    .expect("Write");
            }
        }
    }
};
