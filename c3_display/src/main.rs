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
use crate::hal::timer::Timer;

use embedded_graphics::prelude::*;
use embedded_hal::digital::v2::OutputPin;

#[rtfm::app(device = stm32f1xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        display: hub75::Hub75<(
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
            PwmOutputPin,
        )>,
        delay: Delay,
    }

    #[init]
    fn init(context: init::Context) -> init::LateResources {
        let p = context.device;
        let mut flash = p.FLASH.constrain();
        let mut rcc = p.RCC.constrain();
        let mut afio = p.AFIO.constrain(&mut rcc.apb2);
        let _nvic = context.core.NVIC;
        let clocks = rcc
            .cfgr
            .sysclk(Hertz(64_000_000))
            .pclk1(Hertz(32_000_000))
            .freeze(&mut flash.acr);

        let delay = Delay::new(context.core.SYST, clocks);
        // pwm setup
        let mut gpioa = p.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = p.GPIOB.split(&mut rcc.apb2);
        let (pa15, pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

        let (r1, g1, b1, r2, g2, b2, a, b, c, d, clk, lat, _oe, mut c4) = (
            pb3.into_push_pull_output(&mut gpiob.crl),
            pb4.into_push_pull_output(&mut gpiob.crl),
            gpiob.pb5.into_push_pull_output(&mut gpiob.crl),
            gpiob.pb6.into_push_pull_output(&mut gpiob.crl),
            gpiob.pb7.into_push_pull_output(&mut gpiob.crl),
            gpiob.pb8.into_push_pull_output(&mut gpiob.crh),
            pa15.into_push_pull_output(&mut gpioa.crh),
            gpioa.pa10.into_push_pull_output(&mut gpioa.crh),
            gpioa.pa9.into_push_pull_output(&mut gpioa.crh),
            gpioa.pa8.into_push_pull_output(&mut gpioa.crh),
            gpiob.pb15.into_push_pull_output(&mut gpiob.crh),
            gpiob.pb14.into_push_pull_output(&mut gpiob.crh),
            gpiob.pb13.into_push_pull_output(&mut gpiob.crh),
            gpiob.pb9.into_push_pull_output(&mut gpiob.crh),
        );
        c4.set_high().ok();
        let c4 = c4.into_alternate_push_pull(&mut gpiob.crh);

        let pwm_pins = c4;
        let mut pwm = Timer::tim4(p.TIM4, &clocks, &mut rcc.apb1).pwm(
            pwm_pins,
            &mut afio.mapr,
            Hertz(600_000),
        );
        // unsafe { (*hal::stm32::TIM4::ptr()).psc.write(|w| w.psc().bits(1)) };
        pwm.enable();
        // assert_eq!(pwm.get_max_duty(), 128);
        let mut oe_pwm = PwmOutputPin::new(pwm);
        oe_pwm.set_value(127);
        let display = hub75::Hub75::new((r1, g1, b1, r2, g2, b2, a, b, c, d, clk, lat, oe_pwm), 3);
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
        let image =
            // ImageBmp::new(include_bytes!("../../../visuals/ewg_small.bmp")).unwrap();
        // ImageBmp::new(include_bytes!("../../../visuals/36c3_white_small.bmp")).unwrap();
        ImageBmp::new(include_bytes!("../../../visuals/midnight_font_preset.bmp")).unwrap();
        // ImageBmp::new(include_bytes!("../../../visuals/ferris-flat-happy-small.bmp")).unwrap();
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
pub struct PwmOutputPin {
    pin: hal::pwm::Pwm<hal::stm32::TIM4, hal::pwm::C4>,
}

impl PwmOutputPin {
    fn new(pin: hal::pwm::Pwm<hal::stm32::TIM4, hal::pwm::C4>) -> Self {
        Self { pin }
    }

    fn set_value(&mut self, value: u8) {
        self.pin.set_duty((127 - value) as u16);
    }
}
impl OutputPin for PwmOutputPin {
    type Error = ();
    fn set_high(&mut self) -> Result<(), ()> {
        use hal::stm32::GPIOB;
        unsafe { (*GPIOB::ptr()).crh.modify(|_, w| w.cnf9().push_pull()) };
        Ok(())
    }
    fn set_low(&mut self) -> Result<(), ()> {
        use hal::stm32::GPIOB;
        unsafe { (*GPIOB::ptr()).crh.modify(|_, w| w.cnf9().alt_push_pull()) };
        Ok(())
    }
}
