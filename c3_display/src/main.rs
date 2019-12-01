#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f1xx_hal as hal;

use crate::hal::delay::Delay;
use crate::hal::gpio::*;
use crate::hal::gpio::{gpioa::*, gpiob::*};
use crate::hal::prelude::*;
use crate::hal::time::Hertz;

use embedded_hal::digital::v2::OutputPin;

#[rtfm::app(device = stm32f1xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        display_pins: (
            PB3<Output<PushPull>>,
            PB4<Output<PushPull>>,
            PB5<Output<PushPull>>,
            PB6<Output<PushPull>>,
            PB7<Output<PushPull>>,
            PB8<Output<PushPull>>,
            PA15<Output<PushPull>>,
            PA10<Output<PushPull>>,
            PA9<Output<PushPull>>,
            PA8<Output<PushPull>>,
            PB15<Output<PushPull>>,
            PB14<Output<PushPull>>,
            PB13<Output<PushPull>>,
        ),
        delay: hal::delay::Delay,
    }

    #[init]
    fn init(context: init::Context) -> init::LateResources {
        let p = context.device;
        let mut flash = p.FLASH.constrain();
        let mut rcc = p.RCC.constrain();
        let mut afio = p.AFIO.constrain(&mut rcc.apb2);
        let nvic = context.core.NVIC;
        let clocks = rcc
            .cfgr
            .sysclk(Hertz(64_000_000))
            .pclk1(Hertz(32_000_000))
            .freeze(&mut flash.acr);

        let delay = Delay::new(context.core.SYST, clocks);
        let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = p.GPIOB.split(&mut rcc.apb2);
        let (pa15, pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

        let display_pins = (
            pb3.into_push_pull_output_with_state(&mut gpiob.crl, State::Low),
            pb4.into_push_pull_output_with_state(&mut gpiob.crl, State::Low),
            gpiob
                .pb5
                .into_push_pull_output_with_state(&mut gpiob.crl, State::Low),
            gpiob
                .pb6
                .into_push_pull_output_with_state(&mut gpiob.crl, State::Low),
            gpiob
                .pb7
                .into_push_pull_output_with_state(&mut gpiob.crl, State::Low),
            gpiob
                .pb8
                .into_push_pull_output_with_state(&mut gpiob.crh, State::Low),
            pa15.into_push_pull_output_with_state(&mut gpioa.crh, State::Low),
            gpioa
                .pa10
                .into_push_pull_output_with_state(&mut gpioa.crh, State::Low),
            gpioa
                .pa9
                .into_push_pull_output_with_state(&mut gpioa.crh, State::Low),
            gpioa
                .pa8
                .into_push_pull_output_with_state(&mut gpioa.crh, State::Low),
            gpiob
                .pb15
                .into_push_pull_output_with_state(&mut gpiob.crh, State::Low),
            gpiob
                .pb14
                .into_push_pull_output_with_state(&mut gpiob.crh, State::Low),
            gpiob
                .pb13
                .into_push_pull_output_with_state(&mut gpiob.crh, State::Low),
        );

        init::LateResources {
            delay,
            display_pins,
        }
    }

    #[idle(resources = [delay, display_pins])]
    fn idle(c: idle::Context) -> ! {
        //let (r1, g1, b1, r2, g2, b2, a, b, c, d, clk, lat, oe) = c.resources.display_pins;
        loop {
            for row in 0..16 {
                shift(
                    c.resources.display_pins,
                    c.resources.delay,
                    row | (1 << (row * 3 + 4)),
                    row,
                );
            }
        }
    }
};

fn shift<
    R1: OutputPin,
    G1: OutputPin,
    B1: OutputPin,
    R2: OutputPin,
    G2: OutputPin,
    B2: OutputPin,
    A: OutputPin,
    B: OutputPin,
    C: OutputPin,
    D: OutputPin,
    CLK: OutputPin,
    LAT: OutputPin,
    OE: OutputPin,
    DELAY: embedded_hal::blocking::delay::DelayUs<u16>,
>(
    pins: &mut (R1, G1, B1, R2, G2, B2, A, B, C, D, CLK, LAT, OE),
    delay: &mut DELAY,
    data: u64,
    row: u64,
) {
    let (r1, g1, b1, r2, g2, b2, a, b, c, d, clk, lat, oe) = pins;
    for i in 0..64 {
        if ((1 << i) & data) != 0 {
            r1.set_high().ok();
            g1.set_high().ok();
            g2.set_high().ok();
        } else {
            r1.set_low().ok();
            g1.set_low().ok();
            g2.set_low().ok();
        }
        if i == 62 {
            r1.set_high().ok();
            g1.set_high().ok();
            b1.set_high().ok();
            b2.set_high().ok();
        } else {
            b1.set_low().ok();
            b2.set_low().ok();
        }
        clk.set_high().ok();
        clk.set_low().ok();
    }
    oe.set_high().ok();
    delay.delay_us(20);
    lat.set_low().ok();
    lat.set_high().ok();
    if row & 1 != 0 {
        a.set_high().ok();
    } else {
        a.set_low().ok();
    }
    if row & 2 != 0 {
        b.set_high().ok();
    } else {
        b.set_low().ok();
    }
    if row & 4 != 0 {
        c.set_high().ok();
    } else {
        c.set_low().ok();
    }
    if row & 8 != 0 {
        d.set_high().ok();
    } else {
        d.set_low().ok();
    }
    delay.delay_us(20);
    oe.set_low().ok();
}
