#![no_main]
#![no_std]

#[allow(unused)]
use panic_semihosting;

use microbit::hal;
use ws2812_timer_delay as ws2812;

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
        ws: ws2812::Ws2812<
            CountDownTimer<hal::nrf51::TIMER0>,
            hal::gpio::gpio::PIN21<hal::gpio::Output<hal::gpio::PushPull>>,
        >,
        timer: CountDownTimer<hal::nrf51::TIMER1>,
        serial: Rx<hal::nrf51::UART0>,
    }

    #[init]
    fn init(context: init::Context) -> init::LateResources {
        let p = context.device;

        let gpio = p.GPIO.split();

        let (rx, tx, ws) = (
            // 24 & 25 are internal serial pins
            gpio.pin0.into_floating_input().into(),
            gpio.pin20.into_push_pull_output().into(),
            // gpio.pin25.into_floating_input().into(),
            // gpio.pin24.into_push_pull_output().into(),
            // Spi
            gpio.pin21.into_push_pull_output(),
        );

        let mut timer = CountDownTimer::new(p.TIMER0, TimerFrequency::Freq16MHz);
        // Runs at 16 MHz, so we divide it by 5 for ~3 MHz
        timer.start(Hfticks(1));

        let ws = ws2812::Ws2812::new(timer, ws);

        let mut timer = CountDownTimer::new(p.TIMER1, TimerFrequency::Freq31250Hz);
        // 20 Hertz
        timer.start(Hfticks::from_ms(50));
        // Get the rx serial instance
        let serial = Serial::uart0(p.UART0, tx, rx, hal::serial::BAUD9600)
            .split()
            .1;

        init::LateResources { timer, serial, ws }
    }

    #[idle(resources = [serial, timer, ws])]
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
