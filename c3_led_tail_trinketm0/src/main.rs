#![no_main]
#![no_std]

#[allow(unused)]
use panic_semihosting;

use apa102_spi as apa102;
use trinket_m0 as hal;

use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;
use hal::timer::TimerCounter;
use nb::block;
use smart_leds::SmartLedsWrite;

use c3_led_tail::Elements;

#[rtfm::app(device = trinket_m0::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        // ws: ws2812::Ws2812<
        //     Spi<hal::stm32::SPI1, PA5<Alternate<AF0>>, PA6<Alternate<AF0>>, PA7<Alternate<AF0>>>,
        //     ws2812::devices::Sk6812w,
        // >,
        delay: hal::delay::Delay,
        // timer: Timer<hal::stm32::TIM1>,
        // serial: Serial<hal::stm32::USART2, PA2<Alternate<AF1>>, PA3<Alternate<AF1>>>,
    }

    #[init]
    fn init(context: init::Context) -> init::LateResources {
        let mut p = context.device;

        let mut clocks = GenericClockController::with_internal_32kosc(
            p.GCLK,
            &mut p.PM,
            &mut p.SYSCTRL,
            &mut p.NVMCTRL,
        );

        let mut pins = crate::hal::Pins::new(p.PORT);
        let mut delay = Delay::new(context.core.SYST, &mut clocks);

        let (odi, oci, nc) = (
            // Onboard apa102
            pins.dotstar_di.into_push_pull_output(&mut pins.port),
            pins.dotstar_ci.into_push_pull_output(&mut pins.port),
            pins.d13.into_floating_input(&mut pins.port),
        );

        let gclk0 = clocks.gclk0();
        let timer_clock = clocks.tcc2_tc3(&gclk0).unwrap();
        let mut timer = TimerCounter::tc3_(&timer_clock, p.TC3, &mut p.PM);
        timer.start(5.khz());

        let spi = bitbang_hal::spi::SPI::new(apa102_spi::MODE, nc, odi, oci, timer);

        let mut dotstar = apa102_spi::Apa102::new(spi);

        // let timer = Timer::tim1(p.TIM1, Hertz(20), &mut rcc);
        // let serial = Serial::usart2(p.USART2, (tx, rx), 9600.bps(), &mut rcc);

        // let ws = ws2812::Ws2812::new_sk6812w(spi);
        init::LateResources { delay }
    }

    #[idle(resources = [delay])]
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
        //             .ws
        //             .write(
        //                 smart_leds::gamma(elements.iter()).map(|e| smart_leds::RGBW {
        //                     r: e.r,
        //                     g: e.g,
        //                     b: e.b,
        //                     a: smart_leds::White(0),
        //                 }),
        //             )
        //             .expect("Write");
        //     }
        // }
    }
};
