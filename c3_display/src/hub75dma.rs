use cortex_m_semihosting::dbg;
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::OutputPin;
use hub75::Outputs;
pub struct Hub75Dma<PINS> {
    pins: PINS,
    output_port: *mut u8,
    //                     bits
    data: *mut [[[u8; 128]; 8]; 16],
}

impl<PINS: Outputs> Hub75Dma<PINS> {
    // In the pointer: from bit 0 to 6: R1, G1, B1, R2, G2, B2, clk
    // The 7th bit is undefined
    pub unsafe fn new(pins: PINS, output_port: *mut u8, data: *mut [[[u8; 128]; 8]; 16]) -> Self {
        let mut tmp = Self {
            pins,
            output_port,
            data,
        };
        // To get a clear image
        tmp.clear();
        tmp
    }

    pub fn output<DELAY: DelayUs<u16>>(&mut self, delay: &mut DELAY) {
        // Row
        for (row, row_data) in unsafe { *self.data }.iter().enumerate() {
            // Select row
            if row & 1 != 0 {
                self.pins.a().set_high().ok();
            } else {
                self.pins.a().set_low().ok();
            }
            if row & 2 != 0 {
                self.pins.b().set_high().ok();
            } else {
                self.pins.b().set_low().ok();
            }
            if row & 4 != 0 {
                self.pins.c().set_high().ok();
            } else {
                self.pins.c().set_low().ok();
            }
            if row & 8 != 0 {
                self.pins.d().set_high().ok();
            } else {
                self.pins.d().set_low().ok();
            }
            // bit
            for (bit, bit_data) in row_data.iter().skip(2).enumerate() {
                // Shift the data out
                for port_data in bit_data.iter() {
                    unsafe { *self.output_port = *port_data };
                }
                // latch
                self.pins.lat().set_high().ok();
                self.pins.lat().set_low().ok();
                self.pins.oe().set_low().ok();
                for _ in 0..(1 << bit) {
                    delay.delay_us(1);
                }
                self.pins.oe().set_high().ok();
            }
            // Prevent ghosting
            delay.delay_us(100);
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

use embedded_graphics::{
    drawable::{Dimensions, Pixel},
    pixelcolor::Rgb565,
    Drawing, SizedDrawing,
};
impl<PINS: Outputs> Drawing<Rgb565> for Hub75Dma<PINS> {
    fn draw<T>(&mut self, item_pixels: T)
    where
        T: IntoIterator<Item = Pixel<Rgb565>>,
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
