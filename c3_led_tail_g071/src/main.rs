#![no_main]
#![no_std]

#[allow(unused)]
use panic_semihosting;

use stm32f0xx_hal as hal;
use ws2812_spi as ws2812;

use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use crate::hal::serial::Serial;
use crate::hal::spi::Spi;
use crate::hal::time::Hertz;
use crate::hal::timers::Timer;
use nb::block;
use smart_leds::SmartLedsWrite;

use c3_led_tail::Elements;

use hal::gpio::gpioa::*;
use hal::gpio::*;

#[rtfm::app(device = stm32f0xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        ws: ws2812::Ws2812<
            Spi<hal::stm32::SPI1, PA5<Alternate<AF0>>, PA6<Alternate<AF0>>, PA7<Alternate<AF0>>>,
            ws2812::devices::Sk6812w,
        >,
        delay: hal::delay::Delay,
        timer: Timer<hal::stm32::TIM1>,
        serial: Serial<hal::stm32::USART2, PA2<Alternate<AF1>>, PA3<Alternate<AF1>>>,
    }

    #[init]
    fn init(context: init::Context) -> init::LateResources {
        // NOTE(unsafe): Safe, since rtfm guarantees that no interrupts run in init
        let cs = unsafe { cortex_m::interrupt::CriticalSection::new() };

        let p = context.device;
        let mut flash = p.FLASH;
        let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut flash);
        let gpioa = p.GPIOA.split(&mut rcc);
        let (sck, miso, mosi, tx, rx) = (
            // SPI
            gpioa.pa5.into_alternate_af0(&cs),
            gpioa.pa6.into_alternate_af0(&cs),
            gpioa.pa7.into_alternate_af0(&cs),
            // Serial
            gpioa.pa2.into_alternate_af1(&cs),
            gpioa.pa3.into_alternate_af1(&cs),
        );

        let delay = Delay::new(context.core.SYST, &mut rcc);

        let spi = Spi::spi1(
            p.SPI1,
            (sck, miso, mosi),
            ws2812::MODE,
            3_000_000.hz(),
            &mut rcc,
        );

        let timer = Timer::tim1(p.TIM1, Hertz(20), &mut rcc);
        let serial = Serial::usart2(p.USART2, (tx, rx), 9600.bps(), &mut rcc);

        let ws = ws2812::Ws2812::new_sk6812w(spi);
        init::LateResources {
            ws,
            delay,
            timer,
            serial,
        }
    }

    #[idle(resources = [ws, delay, timer, serial])]
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
                    .write(
                        smart_leds::gamma(elements.iter()).map(|e| smart_leds::RGBW {
                            r: e.r,
                            g: e.g,
                            b: e.b,
                            a: smart_leds::White(0),
                        }),
                    )
                    .expect("Write");
            }
        }
    }
};
