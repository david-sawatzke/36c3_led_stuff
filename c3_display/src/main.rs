#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32g0xx_hal as hal;

use hal::delay::Delay;
use hal::gpio::*;
use hal::gpio::{gpioa::*, gpiob::*};
use hal::prelude::*;
use hal::rcc::{self, PllConfig};
use hal::time::Hertz;

use cortex_m::peripheral::SYST;
use embedded_graphics::prelude::*;
use embedded_hal::digital::v2::OutputPin;

#[rtfm::app(device = stm32g0xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        display: hub75::Hub75<(
            PB0<Output<PushPull>>,
            PB1<Output<PushPull>>,
            PB2<Output<PushPull>>,
            PB3<Output<PushPull>>,
            PB4<Output<PushPull>>,
            PB5<Output<PushPull>>,
            PA0<Output<PushPull>>,
            PA1<Output<PushPull>>,
            PA4<Output<PushPull>>,
            PB11<Output<PushPull>>,
            PB6<Output<PushPull>>,
            PB12<Output<PushPull>>,
            PA10<Output<PushPull>>,
        )>,
        delay: Delay<SYST>,
    }

    #[init]
    fn init(context: init::Context) -> init::LateResources {
        let p = context.device;
        let pll_cfg = PllConfig::with_hsi(4, 48, 2);
        let rcc_cfg = rcc::Config::pll().pll_cfg(pll_cfg);
        let mut rcc = p.RCC.freeze(rcc_cfg);

        let delay: Delay<SYST> = Delay::syst(context.core.SYST, &rcc);

        let gpioa = p.GPIOA.split(&mut rcc);
        let gpiob = p.GPIOB.split(&mut rcc);

        use hal::gpio::Speed::VeryHigh;
        let (r1, g1, b1, r2, g2, b2, a, b, c, d, clk, lat, oe) = (
            gpiob.pb0.into_push_pull_output().set_speed(VeryHigh),
            gpiob.pb1.into_push_pull_output().set_speed(VeryHigh),
            gpiob.pb2.into_push_pull_output().set_speed(VeryHigh),
            gpiob.pb3.into_push_pull_output().set_speed(VeryHigh),
            gpiob.pb4.into_push_pull_output().set_speed(VeryHigh),
            gpiob.pb5.into_push_pull_output().set_speed(VeryHigh),
            // Now the multiplexing
            gpioa.pa0.into_push_pull_output().set_speed(VeryHigh),
            gpioa.pa1.into_push_pull_output().set_speed(VeryHigh),
            gpioa.pa4.into_push_pull_output().set_speed(VeryHigh),
            gpiob.pb11.into_push_pull_output().set_speed(VeryHigh),
            // Shift register control
            // CLK
            gpiob.pb6.into_push_pull_output().set_speed(VeryHigh),
            gpiob.pb12.into_push_pull_output().set_speed(VeryHigh),
            gpioa.pa10.into_push_pull_output().set_speed(VeryHigh),
        );
        let display = hub75::Hub75::new((r1, g1, b1, r2, g2, b2, a, b, c, d, clk, lat, oe), 3);
        init::LateResources { delay, display }
    }

    #[idle(resources = [delay, display])]
    #[allow(unused_imports)]
    fn idle(c: idle::Context) -> ! {
        use embedded_graphics::fonts::{Font12x16, Font6x8};
        use embedded_graphics::image::ImageBmp;
        use embedded_graphics::pixelcolor::Rgb565;
        use embedded_graphics::primitives::{Circle, Rectangle};
        use embedded_graphics::{egrectangle, icoord};

        // let mut buffer = [0u8; 10];
        // c.resources.display.draw(
        //     Font6x8::render_str("Hello")
        //         .stroke(Some(Rgb565(0x000Fu16)))
        //         .fill(Some(Rgb565(0x0000u16)))
        //         .translate(icoord!(i, i)),
        // );
        // c.resources.display.draw(
        //     Font6x8::render_str("World")
        //         .stroke(Some(Rgb565(0xF00Fu16)))
        //         .fill(Some(Rgb565(0x0000u16)))
        //         .translate(icoord!(i, 8 + i)),
        // );
        // let mut counter = 0;
        let image = //ImageBmp::new(include_bytes!("../../../visuals/ewg_small.bmp")).unwrap();
        // ImageBmp::new(include_bytes!("../../../visuals/36c3_white_small.bmp")).unwrap();
        // ImageBmp::new(include_bytes!("../../../visuals/midnight_font_preset.bmp")).unwrap();
        ImageBmp::new(include_bytes!("../../../visuals/ferris-flat-happy-small.bmp")).unwrap();
        //c.resources.display.draw(image.into_iter());

        // let circle = Circle::new(Coord::new(40, 15), 8).fill(Some(Rgb565(0xF000u16)));
        // c.resources.display.draw(circle);
        // c.resources.display.draw(
        //     Font12x16::render_str(
        //         core::str::from_utf8(counter.numtoa(10, &mut buffer)).unwrap(),
        //     )
        //     .stroke(Some(Rgb565(0x18)))
        //     .fill(Some(Rgb565(0))),
        // );
        // c.resources.display.draw(
        //     Font6x8::render_str("iterations")
        //         .stroke(Some(Rgb565(0xFFFFu16)))
        //         .translate(icoord!(0, 16)),
        // );
        // counter += 1;
        c.resources.display.draw(&image);
        loop {
            c.resources.display.output(c.resources.delay);
            c.resources.delay.delay_us(1000 as u32);
            // c.resources.display.clear();
        }
    }
};
