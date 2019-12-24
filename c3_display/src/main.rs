#![no_main]
#![no_std]

use panic_semihosting as _;

use stm32g0xx_hal as hal;

use hal::delay::Delay;
use hal::gpio::*;
use hal::gpio::{gpioa::*, gpiob::*};
use hal::prelude::*;
use hal::rcc::{self, PllConfig};

use c3_display::brightness::BrightnessAdjustment;
use c3_display::hub75dma::{Hub75Dma, Hub75DmaWrite};
use cortex_m::peripheral::SYST;
use cortex_m_semihosting::dbg;
use embedded_graphics::prelude::*;

// We use a global variable to ensure it's only allowated once, since ram
// is quite tight
static mut BUFFER: [[[u8; 128]; 8]; 16] = [[[0; 128]; 8]; 16];
#[rtfm::app(device = stm32g0xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        display: Hub75Dma<
            PA0<Output<PushPull>>,
            PA1<Output<PushPull>>,
            PA4<Output<PushPull>>,
            PB11<Output<PushPull>>,
            PB12<Output<PushPull>>,
        >,
        display_write: Hub75DmaWrite,
        delay: Delay<SYST>,
        serial: hal::serial::Serial<hal::stm32::USART2>,
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
        let (tx, rx, r1, g1, b1, r2, g2, b2, a, b, c, d, clk, lat, oe) = (
            // Serial pins
            gpioa.pa2.into_floating_input(),
            gpioa.pa3.into_floating_input(),
            // Shift pins
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
            // latch
            gpiob.pb12.into_push_pull_output().set_speed(VeryHigh),
            // oe (TODO kinda hacky)
            gpioa
                .pa10
                .into_push_pull_output()
                .set_speed(VeryHigh)
                .into_floating_input(),
        );
        let serial_config = hal::serial::Config::default().baudrate(9600.bps());
        let serial = p.USART2.usart(tx, rx, serial_config, &mut rcc).unwrap();
        // Get pulse output
        let pwm = p.TIM1.pwm(10.khz(), &mut rcc);
        let oe_pulse = pwm.bind_pin(oe);

        let (display, display_write) = unsafe {
            Hub75Dma::new(
                (r1, g1, b1, r2, g2, b2, a, b, c, d, clk, lat),
                &mut BUFFER,
                oe_pulse,
            )
        };
        init::LateResources {
            delay,
            display,
            display_write,
            serial,
        }
    }

    #[idle(resources = [delay, display_write, serial])]
    #[allow(unused_imports)]
    fn idle(c: idle::Context) -> ! {
        use embedded_graphics::fonts::{Font12x16, Font6x8};
        use embedded_graphics::geometry::Point;
        use embedded_graphics::image::ImageTga;
        use embedded_graphics::pixelcolor::Rgb565;
        use embedded_graphics::primitives::{Circle, Rectangle};

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
        // let image = {
        //     // ImageBmp::new(include_bytes!("../../../visuals/ewg_small.bmp")).unwrap()
        //     ImageTga::new(include_bytes!("../../../visuals/midnight_font_preset.tga")).unwrap()
        //     // ImageBmp::new(include_bytes!("../../../visuals/ferris-flat-happy-small.bmp")).unwrap()
        // };
        let images = [
            ImageTga::new(include_bytes!(
                "../../resources/ferris-flat-happy-small.tga"
            ))
            .unwrap(),
            ImageTga::new(include_bytes!("../../resources/ewg_small.tga")).unwrap(),
            ImageTga::new(include_bytes!("../../resources/36c3_white_small.tga")).unwrap(),
            ImageTga::new(include_bytes!("../../resources/rust_2am.tga")).unwrap(),
            ImageTga::new(include_bytes!("../../resources/c3_led_stuff.tga")).unwrap(),
        ];
        // c.resources.display.draw(image.into_iter());

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
        // c.resources.display.draw(&imagetmp);
        // c.resources.display.clear();
        loop {
            if let Ok(image_num) = c.resources.serial.read() {
                match image_num {
                    b'3' => c.resources.display_write.clear(),

                    b'6' => {
                        for i in 0..4 {
                            let mut dimm_disp = BrightnessAdjustment {
                                display: c.resources.display_write,
                                brightness: i * 64,
                            };
                            images[0].draw(&mut dimm_disp);
                        }
                        for i in 0..16 {
                            let mut dimm_disp = BrightnessAdjustment {
                                display: c.resources.display_write,
                                brightness: 255 - i * 16,
                            };
                            images[0].draw(&mut dimm_disp);
                            c.resources.delay.delay_ms(50u8);
                        }
                    }
                    i => {
                        (if i < images.len() as u8 {
                            images[i as usize].draw(c.resources.display_write);
                        })
                    }
                }
            }
            // image_ewg.draw(c.resources.display_write);
            // c.resources.delay.delay_ms(10000u16);

            // c.resources.delay.delay_ms(10000u16);
            // image.draw(c.resources.display_write);
            // c.resources.delay.delay_ms(10000u16);
            // image_36c3.draw(c.resources.display_write);
            // c.resources.delay.delay_ms(10000u16);
            // c.resources.display_write.clear();
        }
    }
    #[task(binds = TIM1_CC, priority = 1, resources = [display])]
    fn tim1(c: tim1::Context) {
        // Clear the interrupt
        let tim1: &hal::stm32::tim1::RegisterBlock = unsafe { &*(hal::stm32::TIM1::ptr()) };
        tim1.sr
            .write(|w| unsafe { w.bits(0xFFFFFFFF).cc1if().clear_bit() });
        c.resources.display.output();
    }
};
