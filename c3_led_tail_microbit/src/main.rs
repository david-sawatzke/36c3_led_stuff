#![no_main]
#![no_std]

#[allow(unused)]
use panic_semihosting;

use microbit::hal;
use ws2812_spi as ws2812;

use hal::delay::Delay;
use hal::hi_res_timer::TimerFrequency;
use hal::prelude::*;
use hal::serial::Serial;
use hal::time::Hfticks;
use hal::timer::CountDownTimer;
use nb::block;
use smart_leds::SmartLedsWrite;

use c3_led_tail::Elements;

#[rtfm::app(device = microbit, peripherals = true)]
const APP: () = {
    struct Resources {
        // dotstar: apa102::Apa102<
        //     bitbang_hal::spi::SPI<
        //         Pa10<Input<Floating>>,
        //         Pa0<Output<PushPull>>,
        //         Pa1<Output<PushPull>>,
        //         TimerCounter<hal::pac::TC3>,
        //     >,
        // >,
        delay: Delay,
        timer: CountDownTimer<hal::nrf51::TIMER1>,
        serial: Serial<hal::nrf51::UART0>,
    }

    #[init]
    fn init(context: init::Context) -> init::LateResources {
        let p = context.device;

        let gpio = p.GPIO.split();

        let delay = Delay::new(p.TIMER0);

        let (rx, tx) = (
            gpio.pin24.into_floating_input().into(),
            gpio.pin25.into_push_pull_output().into(),
        );

        // let spi = bitbang_hal::spi::SPI::new(apa102_spi::MODE, nc, odi, oci, timer_dotstar);

        // let ws2812 = ws2812::Ws2812::new(spi);

        let mut timer = CountDownTimer::new(p.TIMER1, TimerFrequency::Freq31250Hz);
        // 20 Hertz
        timer.start(Hfticks::from_ms(50));
        let serial = Serial::uart0(p.UART0, tx, rx, hal::serial::BAUD9600);

        init::LateResources {
            delay,
            timer,
            serial,
        }
    }

    #[idle(resources = [delay, serial, timer])]
    fn idle(c: idle::Context) -> ! {
        loop {}
        // // Matching resources in c3_display
        // let mut elements = Elements::new(400, 15);
        // // Chosen by fair dice roll
        // let mut rand = oorandom::Rand32::new(0);
        // // On average add a new color every 15 steps
        // let mut steps = rand.rand_range(10..20);
        // // Do something when host isn't active yet
        // // Drops first byte
        // while c.resources.serial.read().is_err() {
        //     steps -= 1;
        //     if steps == 0 {
        //         steps = rand.rand_range(10..20);
        //         elements
        //             .add_predefined(rand.rand_range(0..c3_led_tail::COLORS.len() as u32) as usize)
        //             .unwrap();
        //     }
        //     block!(c.resources.timer.wait()).unwrap();
        // }
        // // Host driven mode
        // loop {
        //     if let Ok(byte) = c.resources.serial.read().map(|x| x as usize) {
        //         if byte < c3_led_tail::COLORS.len() {
        //             elements.add_predefined(byte).unwrap();
        //         }
        //     }
        //     if c.resources.timer.wait().is_ok() {
        //         elements.step();
        //         c.resources
        //             .dotstar
        //             // Only the onboard led
        //             .write(smart_leds::gamma(elements.iter()).take(1))
        //             .expect("Write");
        //     }
        // }
    }
};
