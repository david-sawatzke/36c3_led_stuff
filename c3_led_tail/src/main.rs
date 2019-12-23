#![no_main]
#![no_std]

#[allow(unused)]
use panic_semihosting;

use stm32f0xx_hal as hal;
use ws2812_spi as ws2812;

use crate::hal::delay::Delay;
use crate::hal::prelude::*;
use crate::hal::spi::Spi;
use heapless::consts::*;
use heapless::spsc::{Iter, Queue};
use smart_leds::SmartLedsWrite;
use smart_leds::RGB8;

use core::iter::{Peekable, Rev};

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
    }

    #[init]
    fn init(context: init::Context) -> init::LateResources {
        // NOTE(unsafe): Safe, since rtfm guarantees that no interrupts run in init
        let cs = unsafe { cortex_m::interrupt::CriticalSection::new() };

        let p = context.device;
        let mut flash = p.FLASH;
        let mut rcc = p.RCC.configure().sysclk(48.mhz()).freeze(&mut flash);
        let gpioa = p.GPIOA.split(&mut rcc);
        let (sck, miso, mosi) = (
            gpioa.pa5.into_alternate_af0(&cs),
            gpioa.pa6.into_alternate_af0(&cs),
            gpioa.pa7.into_alternate_af0(&cs),
        );
        // let timer = Timer::tim14(p.TIM14, MegaHertz(3), &mut rcc);
        let delay = Delay::new(context.core.SYST, &mut rcc);

        let spi = Spi::spi1(
            p.SPI1,
            (sck, miso, mosi),
            ws2812::MODE,
            3_000_000.hz(),
            &mut rcc,
        );

        //let mut syscon = p.SYS.configure().freeze();
        let ws = ws2812::Ws2812::new_sk6812w(spi);
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
            // 36c3 green
            RGB8 {
                r: 0,
                g: 187,
                b: 49,
            },
        ];
        let mut elements = Elements::new(60, 15);
        // Chosen by fair dice roll
        let mut rand = oorandom::Rand32::new(0);
        // On average add a new color every 15 steps
        let mut steps = rand.rand_range(10..20);
        loop {
            steps -= 1;
            if steps == 0 {
                steps = rand.rand_range(10..20);
                elements
                    .add(colors[rand.rand_range(0..colors.len() as u32) as usize])
                    .unwrap();
            }
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
            c.resources.delay.delay_ms(50 as u16);
        }
    }
};

struct QueueElement {
    color: RGB8,
    position: u8,
}

struct Elements {
    queue: Queue<QueueElement, U16, u8, heapless::spsc::SingleCore>,
    length: u8,
    trail_length: u8,
}

impl Elements {
    fn new(length: u8, trail_length: u8) -> Self {
        let queue = unsafe { Queue::u8_sc() };

        Self {
            queue,
            length,
            trail_length,
        }
    }

    fn step(&mut self) {
        for x in self.queue.iter_mut() {
            x.position += 1;
        }
        self.cull();
    }

    fn add(&mut self, color: RGB8) -> Result<(), ()> {
        let element = QueueElement { color, position: 0 };
        if self
            .queue
            .iter_mut()
            .next_back()
            .map(|x| x.position != 0)
            .unwrap_or(true)
        {
            self.queue.enqueue(element).map_err(|_| ())
        } else {
            // Too many elements, skip this one
            Ok(())
        }
    }

    // Drop elements that aren't visible anymore
    fn cull(&mut self) {
        while self
            .queue
            .peek()
            .map(|x| x.position > (self.length + self.trail_length))
            .unwrap_or(false)
        {
            self.queue.dequeue();
        }
    }

    fn iter<'a>(&'a mut self) -> ElementIter<'a> {
        ElementIter {
            iter: self.queue.iter().rev().peekable(),
            pos: 0,
            step: 255 / self.trail_length,
            trail_length: self.trail_length,
            length: self.length,
        }
    }
}

struct ElementIter<'a> {
    iter: Peekable<Rev<Iter<'a, QueueElement, U16, u8, heapless::spsc::SingleCore>>>,
    pos: u8,
    trail_length: u8,
    step: u8,
    length: u8,
}

impl<'a> Iterator for ElementIter<'a> {
    type Item = RGB8;
    fn next(&mut self) -> Option<RGB8> {
        let pos = self.pos;
        self.pos += 1;
        // Check if we exceeded the length
        if pos >= self.length {
            return None;
        }
        // Check if it's time for the next element
        // We don't return now to get results for the full length of the chain
        if self.iter.peek().map(|x| x.position < pos).unwrap_or(false) {
            self.iter.next();
        }
        if let Some(x) = self.iter.peek() {
            let distance = x.position - pos;
            let multiplier = self.trail_length.saturating_sub(distance) as u16 * self.step as u16;
            Some(brightness(x.color, multiplier))
        } else {
            // Return dark pixel
            Some(RGB8 { r: 0, g: 0, b: 0 })
        }
    }
}

fn brightness(color: RGB8, multiplier: u16) -> RGB8 {
    RGB8 {
        r: (color.r as u16 * multiplier / 256) as u8,
        g: (color.g as u16 * multiplier / 256) as u8,
        b: (color.b as u16 * multiplier / 256) as u8,
    }
}
