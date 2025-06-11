use embedded_graphics::{
    image::{Image, ImageRawBE},
    pixelcolor::{Gray8, Rgb888},
    prelude::*,
};
use rpi_led_panel::{Canvas, RGBMatrix};

const LOGO_IMAGE: &[u8] = include_bytes!("../assets/hacklab_logo.raw");
const LOGO_SIZE: usize = 32;

pub struct Display {
    matrix: RGBMatrix,
    canvas: Box<Canvas>,
}

impl Display {
    pub fn new(matrix: RGBMatrix, canvas: Box<Canvas>) -> Self {
        Self { matrix, canvas }
    }

    pub fn main_loop(mut self) -> Result<(), &'static str> {
        let image_data = ImageRawBE::<Rgb888>::new(LOGO_IMAGE, LOGO_SIZE as u32);
        let image = Image::new(&image_data, Point::new(0, 0));

        loop {
            self.canvas.fill(0, 0, 0);
            image.draw(self.canvas.as_mut()).unwrap();
            self.canvas = self.matrix.update_on_vsync(self.canvas);

            // if step % 120 == 0 {
            //     print!("\r{:>100}\rFramerate: {}", "", self.matrix.get_framerate());
            //     std::io::stdout().flush().unwrap();
            // }
        }
    }
}
