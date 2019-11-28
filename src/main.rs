#![no_main]
#![no_std]

#[allow(unused)]
use panic_halt;

use stm32f0xx_hal as hal;
use ws2812_nop_stm32f0 as ws2812;

use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use smart_leds::SmartLedsWrite;
use smart_leds::RGB8;

#[rtfm::app(device = stm32f0xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        ws: ws2812::Ws2812<hal::gpio::gpiob::PB5<hal::gpio::Output<hal::gpio::PushPull>>>,
        delay: hal::delay::Delay,
    }

    #[init]
    fn init(context: init::Context) -> init::LateResources {
        // NOTE(unsafe): Safe, since rtfm guarantees that no interrupts run in init
        let cs = unsafe { cortex_m::interrupt::CriticalSection::new() };

        let p = context.device;
        let mut flash = p.FLASH;
        let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut flash);
        let gpiob = p.GPIOB.split(&mut rcc);
        // let timer = Timer::tim14(p.TIM14, MegaHertz(3), &mut rcc);
        let delay = Delay::new(context.core.SYST, &mut rcc);

        //let mut syscon = p.SYS.configure().freeze();
        let ws_pin = gpiob.pb5.into_push_pull_output_hs(&cs);
        let ws = ws2812::Ws2812::new(ws_pin);
        init::LateResources { ws, delay }
    }

    #[idle(resources = [ws, delay])]
    fn idle(c: idle::Context) -> ! {
        let colors = [
            // Embedded wg bright
            RGB8 {
                r: 99,
                g: 107,
                b: 201,
            },
            // Embedded wg
            // RGB8 {
            //     r: 47,
            //     g: 51,
            //     b: 96,
            // },
            // 36c3 orange
            RGB8 {
                r: 254,
                g: 80,
                b: 0,
            },
            // 36c3 gree
            RGB8 {
                r: 0,
                g: 187,
                b: 49,
            },
        ];
        loop {
            for i in 0..(15 * 3) {
                // let trail = smart_leds::gamma(
                //     core::slice::from_ref(&colors[i])
                //         .into_iter()
                //         .cycle()
                //         .take(20)
                //         .cloned(),
                // );
                let trail = smart_leds::gamma(
                    LedTrail {
                        color: colors[0],
                        multiplier: 255,
                        step: 16,
                        remaining: 19,
                    }
                    .chain(LedTrail {
                        color: colors[1],
                        multiplier: 255,
                        step: 16,
                        remaining: 19,
                    })
                    .chain(LedTrail {
                        color: colors[2],
                        multiplier: 255,
                        step: 16,
                        remaining: 19,
                    })
                    .cycle()
                    .skip(15 * 3 - 1 - i)
                    .take(60),
                );
                c.resources.ws.write(trail).unwrap();
                c.resources.delay.delay_ms(50 as u16);
            }
        }
    }
};

#[derive(Clone)]
struct LedTrail {
    color: RGB8,
    multiplier: u8,
    step: u8,
    remaining: u8,
}

impl Iterator for LedTrail {
    type Item = RGB8;
    fn next(&mut self) -> Option<RGB8> {
        if self.remaining > 0 {
            let color = RGB8 {
                r: (self.color.r as u16 * (self.multiplier as u16 + 1) / 256) as u8,
                g: (self.color.g as u16 * (self.multiplier as u16 + 1) / 256) as u8,
                b: (self.color.b as u16 * (self.multiplier as u16 + 1) / 256) as u8,
            };
            self.multiplier = self.multiplier.checked_sub(self.step)?;
            Some(color)
        } else {
            None
        }
    }
}
