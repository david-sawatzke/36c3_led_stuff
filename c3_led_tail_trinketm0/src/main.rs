#![no_main]
#![no_std]

#[allow(unused)]
use panic_semihosting;

use apa102_spi as apa102;
use trinket_m0 as hal;

use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::gpio::*;
use hal::prelude::*;
use hal::sercom::*;
use hal::time::Hertz;
use hal::timer::TimerCounter;
use instant_timer::InstantTimer;
use nb::block;
use smart_leds::SmartLedsWrite;

use c3_led_tail::Elements;

#[rtfm::app(device = trinket_m0::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        dotstar: apa102::Apa102<
            bitbang_hal::spi::SPI<
                Pa10<Input<Floating>>,
                Pa0<Output<PushPull>>,
                Pa1<Output<PushPull>>,
                TimerCounter<hal::pac::TC3>,
            >,
        >,
        external: apa102::Apa102<
            bitbang_hal::spi::SPI<
                Pa9<Input<Floating>>,
                Pa8<Output<PushPull>>,
                Pa2<Output<PushPull>>,
                // TimerCounter<hal::pac::TC5>,
                InstantTimer,
            >,
        >,
        delay: hal::delay::Delay,
        timer: TimerCounter<hal::pac::TC4>,
        serial: UART0<Sercom0Pad3<Pa7<PfD>>, Sercom0Pad2<Pa6<PfD>>, (), ()>,
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
        let delay = Delay::new(context.core.SYST, &mut clocks);

        let (odi, oci, nc, edi, eci, enc, rx, tx) = (
            // Onboard apa102
            pins.dotstar_di.into_push_pull_output(&mut pins.port),
            pins.dotstar_ci.into_push_pull_output(&mut pins.port),
            pins.d13.into_floating_input(&mut pins.port),
            // Extrenal
            pins.d0.into_push_pull_output(&mut pins.port),
            pins.d1.into_push_pull_output(&mut pins.port),
            pins.d2.into_floating_input(&mut pins.port),
            pins.d3.into_floating_input(&mut pins.port),
            pins.d4.into_floating_input(&mut pins.port),
        );

        let gclk0 = clocks.gclk0();
        let timer_clock = clocks.tcc2_tc3(&gclk0).unwrap();
        let mut timer_dotstar = TimerCounter::tc3_(&timer_clock, p.TC3, &mut p.PM);
        timer_dotstar.start(5.khz());

        let spi = bitbang_hal::spi::SPI::new(apa102_spi::MODE, nc, odi, oci, timer_dotstar);

        let dotstar = apa102_spi::Apa102::new(spi);

        let timer_clock = clocks.tc4_tc5(&gclk0).unwrap();
        // TODO Timer doesn't seem to work at higher frequencies
        // let mut timer_external = TimerCounter::tc5_(&timer_clock, p.TC5, &mut p.PM);
        // timer_external.start(5.khz());
        let timer_external = InstantTimer {};
        let spi = bitbang_hal::spi::SPI::new(apa102_spi::MODE, enc, edi, eci, timer_external);
        let external = apa102_spi::Apa102::new(spi);
        let mut timer = TimerCounter::tc4_(&timer_clock, p.TC4, &mut p.PM);

        // Has half as much leds per m as the other ones, so half the frueqency
        timer.start(Hertz(10));
        let serial = hal::uart(
            &mut clocks,
            Hertz(9600),
            p.SERCOM0,
            &mut p.PM,
            rx,
            tx,
            &mut pins.port,
        );

        init::LateResources {
            delay,
            dotstar,
            timer,
            serial,
            external,
        }
    }

    #[idle(resources = [delay, dotstar, serial, timer, external])]
    fn idle(c: idle::Context) -> ! {
        // Matching resources in c3_display
        // Half the tail length, since half the leds per m
        let mut elements = Elements::new(80, 8);
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
                    .dotstar
                    // Only the onboard led
                    .write(smart_leds::gamma(elements.iter()).take(1))
                    .expect("Write");
                c.resources
                    .external
                    // Only the onboard led
                    .write(smart_leds::gamma(elements.iter()))
                    .expect("Write");
            }
        }
    }
};
