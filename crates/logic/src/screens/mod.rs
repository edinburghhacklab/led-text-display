use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
};

mod text;
pub use text::*;

pub trait Screen<D: DrawTarget<Color = Rgb888>> {
    /// Draw a frame of the screen to the given display
    fn draw(&mut self, display: &mut D) -> Result<(), D::Error>;

    /// Return true if the screen wants to keep drawing things
    fn wants_redraw(&self) -> bool {
        true
    }

    /// Called when screen is no longer active on display
    fn paused(&mut self) {}
}

pub struct TestScreen;
impl<D: DrawTarget<Color = Rgb888>> Screen<D> for TestScreen {
    fn draw(&mut self, display: &mut D) -> Result<(), D::Error> {
        const COLOURS: &[Rgb888] = &[
            Rgb888::RED,
            Rgb888::CSS_ORANGE,
            Rgb888::YELLOW,
            Rgb888::GREEN,
            Rgb888::BLUE,
            Rgb888::CSS_VIOLET,
        ];
        let square_size = (display.bounding_box().columns().count() / COLOURS.len()) as u32;

        for (start_col, colour) in display
            .bounding_box()
            .columns()
            .step_by(square_size as usize)
            .zip(COLOURS.iter())
        {
            Rectangle::new(
                Point::new(start_col, display.bounding_box().rows().start),
                Size::new(square_size, display.bounding_box().rows().count() as u32),
            )
            .into_styled(PrimitiveStyleBuilder::new().fill_color(*colour).build())
            .draw(display)?;
        }

        Ok(())
    }
}
