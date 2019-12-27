#![no_main]
#![no_std]

#[allow(unused)]
use panic_semihosting;

use xmc1100_hal as hal;

use hal::delay::Delay;
use hal::prelude::*;
use hal::scu::Scu;
use hal::serial::Serial;
use hal::time::Bps;
use hal::usic;
use nb::block;

use hal::gpio::{port2::*, *};
#[rtfm::app(device = xmc1100_hal::xmc1100, peripherals = true)]
const APP: () = {
    struct Resources {
        delay: Delay,
        serial: Serial<
            hal::xmc1100::USIC0_CH0,
            P2_0<Alternate<AF6>>,
            usic::Dx0Dx3Pin<P2_2<Input<Floating>>, hal::xmc1100::USIC0_CH0>,
        >,
    }

    #[init]
    fn init(context: init::Context) -> init::LateResources {
        // NOTE(unsafe): Safe, since rtfm guarantees that no interrupts run in init
        let cs = unsafe { cortex_m::interrupt::CriticalSection::new() };

        let p = context.device;
        let port2 = p.PORT2.split();
        let mut usic = p.USIC0_CH0;

        let mut scu = Scu::new(p.SCU_GENERAL, p.SCU_CLK).freeze();

        let delay = Delay::new(context.core.SYST, &scu);

        let (rx, tx) = (
            usic::dx3pin_to_dx0pin(port2.p2_2.into_floating_input(&cs), &mut usic),
            port2.p2_0.into_alternate_af6(&cs),
        );

        // Get the rx serial instance
        let serial = Serial::usic0_ch0(usic, (tx, rx), Bps(9600), &mut scu);

        init::LateResources { delay, serial }
    }

    #[idle(resources = [delay, serial])]
    fn idle(c: idle::Context) -> ! {
        // Chosen by fair dice roll
        let mut rand = oorandom::Rand32::new(0);
        let mut image = 0;
        // Choose something out of range
        let mut prev_image = 5;
        loop {
            let delay = rand.rand_range(10..20);
            // "Do-while" loop
            // TODO Abuse while loops less
            while {
                image = rand.rand_range(0..5) as u8;
                image == prev_image
            } {}
            prev_image = image;
            block!(c.resources.serial.write(image)).unwrap();
            c.resources.delay.delay_ms(200u32 * delay);
        }
    }
};
