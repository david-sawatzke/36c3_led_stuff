use cortex_m_semihosting::dbg;
use embedded_hal::digital::v2::OutputPin;
use stm32g0xx_hal as hal;

use hal::gpio::gpiob::*;
use hal::gpio::{Output, PushPull};
use hal::prelude::*;
use hal::stm32::TIM1;

// Has to be higher than 128, so all the bits can be represented
const TIMER_PERIOD: u16 = 129;
pub struct Hub75Dma<A, B, C, D, LATCH> {
    row_pins: (A, B, C, D),
    latch: LATCH,
    _oe_pulse: hal::timer::pwm::PwmPin<TIM1, hal::timer::Channel3>,
    output_port: *mut u8,
    //                     bits
    data: *mut [[[u8; 128]; 8]; 16],
    o_count: u8,
}

unsafe impl<A, B, C, D, LATCH> Send for Hub75Dma<A, B, C, D, LATCH> {}

impl<A: OutputPin, B: OutputPin, C: OutputPin, D: OutputPin, LATCH: OutputPin>
    Hub75Dma<A, B, C, D, LATCH>
{
    // In the pointer: from bit 0 to 6: R1, G1, B1, R2, G2, B2, clk
    // The 7th bit is undefined
    pub unsafe fn new(
        pins: (
            PB0<Output<PushPull>>,
            PB1<Output<PushPull>>,
            PB2<Output<PushPull>>,
            PB3<Output<PushPull>>,
            PB4<Output<PushPull>>,
            PB5<Output<PushPull>>,
            A,
            B,
            C,
            D,
            PB6<Output<PushPull>>,
            LATCH,
        ),
        data: *mut [[[u8; 128]; 8]; 16],
        mut oe_pulse: hal::timer::pwm::PwmPin<TIM1, hal::timer::Channel3>,
    ) -> Self {
        // Get pointer
        let output_port = core::mem::transmute(&((*hal::stm32::GPIOB::ptr()).odr) as *const _);
        assert_eq!(output_port as usize, 0x5000_0414);
        oe_pulse.enable();
        // Lets hack the timer so that it does what we want
        let tim1: &mut hal::stm32::tim1::RegisterBlock = &mut *(TIM1::ptr() as *mut _);
        // Stop the timer & set opm mode
        tim1.cr1.write(|w| w.opm().set_bit());
        tim1.cr2.write(|w| w);
        // Enable update interrupt
        tim1.dier.write(|w| w.cc1ie().set_bit());
        // Normal pwm mode
        tim1.ccmr2_output_mut().write(|w| w.oc3m().bits(6));
        // Set the prescaler so that the timer is done when a row is shifted out
        tim1.psc.write(|w| w.psc().bits(8));
        // Need this so ARR is reached in the first iteration
        tim1.cnt.write(|w| w.cnt().bits(0));
        // We adjust the low period via ccr3, since the output is low between ccr & arr
        tim1.arr.write(|w| w.arr().bits(TIMER_PERIOD));
        // Generate a timer interrupt after this time
        // Experimentally determined, so that the output can still be active
        // while shifting new data
        tim1.ccr1.write(|w| w.ccr1().bits(60));
        let mut tmp = Self {
            row_pins: (pins.6, pins.7, pins.8, pins.9),
            latch: pins.11,
            output_port,
            data,
            _oe_pulse: oe_pulse,
            o_count: 0,
        };
        // To generate a clear image
        tmp.clear();
        tmp
    }

    pub fn output(&mut self) {
        let row = (self.o_count / 8 % 16) as usize;
        let bit = (self.o_count % 8) as usize;
        self.o_count = self.o_count.wrapping_add(1);
        // Shift the data out
        for port_data in unsafe { (*self.data)[row][bit].iter() } {
            unsafe { *self.output_port = *port_data };
        }
        let tim1: &mut hal::stm32::tim1::RegisterBlock = unsafe { &mut *(TIM1::ptr() as *mut _) };
        // Check that the timer isn't still running
        assert!(tim1.cr1.read().cen().bit() == false);
        // Select the row
        // Doing it now, since oe is guaranteed to be disabled now
        if bit == 0 {
            Self::select_row(row as u8, &mut self.row_pins);
        }
        // Latch the data
        self.latch.set_high().ok();
        self.latch.set_low().ok();
        // Generate pulse
        let compare: u16 = TIMER_PERIOD - (1 << (bit as u16));
        // Pin is low between CCR3 & ARR
        tim1.ccr3.write(|w| unsafe { w.ccr3().bits(compare) });
        tim1.cr1.modify(|_, w| w.opm().set_bit().cen().set_bit());
    }

    fn select_row(row: u8, row_pins: &mut (A, B, C, D)) {
        // Select row
        if row & 1 != 0 {
            row_pins.0.set_high().ok();
        } else {
            row_pins.0.set_low().ok();
        }
        if row & 2 != 0 {
            row_pins.1.set_high().ok();
        } else {
            row_pins.1.set_low().ok();
        }
        if row & 4 != 0 {
            row_pins.2.set_high().ok();
        } else {
            row_pins.2.set_low().ok();
        }
        if row & 8 != 0 {
            row_pins.3.set_high().ok();
        } else {
            row_pins.3.set_low().ok();
        }
    }

    pub fn clear(&mut self) {
        for row in unsafe { &mut *self.data }.iter_mut() {
            for bit in row.iter_mut() {
                for (byte, byte_data) in bit.iter_mut().enumerate() {
                    *byte_data = if byte % 2 == 0 {
                        0b0000_0000
                    } else {
                        0b0100_0000
                    };
                }
            }
        }
    }
}

use embedded_graphics::pixelcolor::RgbColor;
use embedded_graphics::{drawable::Pixel, pixelcolor::Rgb888, Drawing};

impl<A: OutputPin, B: OutputPin, C: OutputPin, D: OutputPin, LATCH: OutputPin> Drawing<Rgb888>
    for Hub75Dma<A, B, C, D, LATCH>
{
    fn draw<T>(&mut self, item_pixels: T)
    where
        T: IntoIterator<Item = Pixel<Rgb888>>,
    {
        // This table remaps linear input values
        // (the numbers we’d like to use; e.g. 127 = half brightness)
        // to nonlinear gamma-corrected output values
        // (numbers producing the desired effect on the LED;
        // e.g. 36 = half brightness).
        const GAMMA8: [u8; 256] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 4,
            4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 11, 11,
            12, 12, 13, 13, 13, 14, 14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22,
            22, 23, 24, 24, 25, 25, 26, 27, 27, 28, 29, 29, 30, 31, 32, 32, 33, 34, 35, 35, 36, 37,
            38, 39, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 50, 51, 52, 54, 55, 56, 57, 58,
            59, 60, 61, 62, 63, 64, 66, 67, 68, 69, 70, 72, 73, 74, 75, 77, 78, 79, 81, 82, 83, 85,
            86, 87, 89, 90, 92, 93, 95, 96, 98, 99, 101, 102, 104, 105, 107, 109, 110, 112, 114,
            115, 117, 119, 120, 122, 124, 126, 127, 129, 131, 133, 135, 137, 138, 140, 142, 144,
            146, 148, 150, 152, 154, 156, 158, 160, 162, 164, 167, 169, 171, 173, 175, 177, 180,
            182, 184, 186, 189, 191, 193, 196, 198, 200, 203, 205, 208, 210, 213, 215, 218, 220,
            223, 225, 228, 231, 233, 236, 239, 241, 244, 247, 249, 252, 255,
        ];
        for Pixel(coord, color) in item_pixels {
            let row = (coord[1] % 16) as usize;
            let collumn = coord[0] as usize;
            let mut pixel_data = [0; 8];
            let r = GAMMA8[color.r() as usize];
            let g = GAMMA8[color.g() as usize];
            let b = GAMMA8[color.b() as usize];
            for i in 0..8 {
                pixel_data[i] = ((r & (1 << i)) != 0) as u8
                    | ((((g & (1 << i)) != 0) as u8) << 1)
                    | ((((b & (1 << i)) != 0) as u8) << 2);
            }
            let (bitmask, bitshift) = if coord[1] < 16 {
                (0b111000, 0)
            } else {
                (0b111, 3)
            };
            for i in 0..8 {
                unsafe {
                    let mut byte = (*self.data)[row][i][collumn * 2];
                    // Preserve upper bits
                    byte = (byte & bitmask) | pixel_data[i] << bitshift;
                    (*self.data)[row][i][collumn * 2] = byte;
                    (*self.data)[row][i][(collumn * 2) + 1] = byte | 0b100_0000;
                }
            }
        }
    }
}
