use embedded_graphics::{
    drawable::Pixel, geometry::Size, pixelcolor::Rgb888, pixelcolor::RgbColor, DrawTarget,
};
pub struct BrightnessAdjustment<'a, DISPLAY: DrawTarget<Rgb888>> {
    pub display: &'a mut DISPLAY,
    pub brightness: u8,
}

impl<'a, DISPLAY: DrawTarget<Rgb888>> DrawTarget<Rgb888> for BrightnessAdjustment<'a, DISPLAY> {
    fn draw_pixel(&mut self, item: Pixel<Rgb888>) {
        let Pixel(coord, color) = item;
        let color = Rgb888::new(
            (color.r() as u16 * self.brightness as u16 / 256) as u8,
            (color.g() as u16 * self.brightness as u16 / 256) as u8,
            (color.b() as u16 * self.brightness as u16 / 256) as u8,
        );
        let item = Pixel(coord, color);
        self.display.draw_pixel(item);
    }
    fn size(&self) -> Size {
        self.display.size()
    }
}
