use std::time::Duration;

use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::{Baseline, Text, TextStyleBuilder},
};
use embedded_layout::{layout::linear::LinearLayout, prelude::*};

use crate::recolour_image::RecolouredImageRaw;

use super::Screen;

const ICON_HEIGHT: u32 = 28;

const CO2_YELLOW_THRESHOLD: u32 = 1000;
const CO2_RED_THRESHOLD: u32 = 1200;

#[derive(Debug)]
/// A screen that just displays a line of environment.
pub struct EnvironmentScreen {
    temp: f32,
    co2: u32,
}

impl EnvironmentScreen {
    /// Show the given environment in a particular style.
    pub fn new(temp: f32, co2: u32) -> Self {
        Self { temp, co2 }
    }
}

impl<D: DrawTarget<Color = Rgb888>> Screen<D> for EnvironmentScreen {
    fn draw(&mut self, display: &mut D) -> Result<(), D::Error> {
        display.clear(Rgb888::BLACK)?;

        let co2_colour = match self.co2 {
            ..CO2_YELLOW_THRESHOLD => Rgb888::GREEN,
            CO2_YELLOW_THRESHOLD..CO2_RED_THRESHOLD => Rgb888::YELLOW,
            _ => Rgb888::RED,
        };

        LinearLayout::horizontal(
            Chain::new(Image::new(
                &RecolouredImageRaw::<Rgb888>::new(
                    include_bytes!("../../../../assets/co2.raw"),
                    ICON_HEIGHT,
                    (Rgb888::BLACK, co2_colour),
                ),
                Point::zero(),
            ))
            .append(Text::with_text_style(
                &format!("{}", self.co2),
                Point::zero(),
                MonoTextStyle::new(&FONT_10X20, co2_colour),
                TextStyleBuilder::new().baseline(Baseline::Middle).build(),
            ))
            .append(
                Rectangle::new(Point::zero(), Size::new(5, 1))
                    .into_styled(PrimitiveStyleBuilder::new().build()),
            )
            .append(Image::new(
                &ImageRaw::<Rgb888>::new(
                    include_bytes!("../../../../assets/temp.raw"),
                    ICON_HEIGHT,
                ),
                Point::zero(),
            ))
            .append(Text::with_text_style(
                &format!("{}", self.temp),
                Point::zero(),
                MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE),
                TextStyleBuilder::new().baseline(Baseline::Middle).build(),
            )),
        )
        .with_alignment(vertical::Center)
        .arrange()
        .align_to(
            &display.bounding_box(),
            horizontal::Center,
            vertical::Center,
        )
        .draw(display)?;

        // Image::new(
        //     &RecolouredImageRaw::<Rgb888>::new(
        //         include_bytes!("../../../../assets/temp.raw"),
        //         ICON_HEIGHT,
        //         (Rgb888::BLACK, Rgb888::CSS_WHITE),
        //     ),
        //     Point::new(
        //         1,
        //         display.bounding_box().center().y - (ICON_HEIGHT as i32 / 2),
        //     ),
        // )
        // .draw(display)?;

        // Text::with_text_style(
        //     &format!("{}", self.co2),
        //     Point::new(
        //         (PADDING + ICON_HEIGHT + PADDING + ) as i32,
        //         display.bounding_box().center().y,
        //     ),
        //     MonoTextStyle::new(&FONT_10X20, co2_colour),
        //     TextStyleBuilder::new().baseline(Baseline::Middle).build(),
        // )
        // .draw(display)?;
        Ok(())
    }

    fn single_display_duration(&self, _display: &D) -> Duration {
        Duration::from_secs(5)
    }

    fn paused(&mut self, _for_dur: Duration) {}

    fn should_remove(&self) -> bool {
        false
    }

    fn id(&self) -> &str {
        "environment"
    }
}
