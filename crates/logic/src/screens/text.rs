use std::time::{Duration, Instant};

use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};

use super::Screen;

/// A screen that just displays a line of text.
pub struct TextScreen {
    text: String,
    style: MonoTextStyle<'static, Rgb888>,
    offset: i32,
    offset_last_incremented: Option<Instant>,
    offset_inc_interval: Duration,
}

impl TextScreen {
    /// Show the given text in a particular style.
    pub fn new(text: String, style: MonoTextStyle<'static, Rgb888>) -> Self {
        Self {
            text: text.replace("\n", ""),
            style,
            offset: 0,
            offset_last_incremented: None,
            offset_inc_interval: Duration::from_millis(15),
        }
    }

    /// Show the given text with a white font.
    pub fn with_text(text: String) -> Self {
        Self::new(text, MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE))
    }
}

impl<D: DrawTarget<Color = Rgb888>> Screen<D> for TextScreen {
    fn draw(&mut self, display: &mut D) -> Result<(), D::Error> {
        let text_total_width = self.style.font.character_size.width * self.text.len() as u32;

        let (position, text_style) = if text_total_width <= display.bounding_box().size.width {
            // no need for scrolling
            (
                display.bounding_box().center(),
                TextStyleBuilder::new()
                    .baseline(Baseline::Middle)
                    .alignment(Alignment::Center)
                    .build(),
            )
        } else {
            if let Some(last_inc) = self.offset_last_incremented {
                let since_last_inc = Instant::now() - last_inc;
                if since_last_inc >= self.offset_inc_interval {
                    self.offset = (self.offset
                        + (since_last_inc.div_duration_f32(self.offset_inc_interval)) as i32)
                        % (display.bounding_box().size.width + text_total_width + 10) as i32;
                    self.offset_last_incremented = Some(Instant::now());
                }
            } else {
                self.offset_last_incremented = Some(Instant::now());
            }
            (
                Point::new(
                    display.bounding_box().bottom_right().unwrap().x - self.offset,
                    display.bounding_box().center().y,
                ),
                TextStyleBuilder::new().baseline(Baseline::Middle).build(),
            )
        };

        display.clear(Rgb888::BLACK)?;
        Text::with_text_style(&self.text, position, self.style, text_style).draw(display)?;

        Ok(())
    }

    fn paused(&mut self) {
        self.offset_last_incremented = None;
    }
}
