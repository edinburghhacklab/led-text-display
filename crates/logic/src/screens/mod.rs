use std::{fmt::Debug, time::Duration};

use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
};

mod text;
pub use text::*;

pub trait Screen<D: DrawTarget<Color = Rgb888>>: Send + Debug {
    /// Draw a frame of the screen to the given display
    fn draw(&mut self, display: &mut D) -> Result<(), D::Error>;

    /// Returns the desired duration for a single continuous display of this screen
    fn single_display_duration(&self, _display: &D) -> Duration {
        Duration::from_secs(5)
    }

    /// Called after the screen stops being actively displayed, with the duration it was displayed for.
    /// [`Self::should_remove`] will be called after this.
    fn paused(&mut self, _for_dur: Duration) {}

    /// Returns true if the screen wants to be removed from the active rotation
    fn should_remove(&self) -> bool {
        false
    }
}

#[derive(Debug)]
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

    // always remove after one display
    fn should_remove(&self) -> bool {
        true
    }
}
