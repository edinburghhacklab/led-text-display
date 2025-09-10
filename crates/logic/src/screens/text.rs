use std::time::{Duration, Instant};

use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    text::{Alignment, Baseline, Text, TextStyleBuilder},
};

use super::Screen;

#[derive(Debug)]
/// A screen that just displays a line of text.
pub struct TextScreen {
    text: String,
    style: MonoTextStyle<'static, Rgb888>,

    offset: i32,
    offset_last_incremented: Option<Instant>,
    offset_inc_interval: Duration,

    show_count: u8,
}

impl TextScreen {
    /// Show the given text in a particular style.
    pub fn new(
        text: String,
        style: MonoTextStyle<'static, Rgb888>,
        show_count: Option<u8>,
    ) -> Self {
        Self {
            text: text.replace("\n", ""),
            style,
            offset: 0,
            offset_last_incremented: None,
            offset_inc_interval: Duration::from_millis(8),
            show_count: show_count.unwrap_or(3) + 1,
        }
    }

    /// Show the given text with a white font.
    pub fn with_text(text: String, show_count: Option<u8>) -> Self {
        Self::new(
            text,
            MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE),
            show_count,
        )
    }

    fn text_total_width(&self) -> u32 {
        self.style.font.character_size.width * self.text.len() as u32
    }

    fn max_offset_for<D: DrawTarget<Color = Rgb888>>(&self, display: &D) -> Option<u32> {
        let display_width = display.bounding_box().size.width;
        // no max offset when we're not scrolling
        if self.text_total_width() <= display_width {
            return None;
        }

        Some(
            display_width
                + self.text_total_width()
                + (Duration::from_millis(250).div_duration_f32(self.offset_inc_interval) as u32),
        )
    }
}

impl<D: DrawTarget<Color = Rgb888>> Screen<D> for TextScreen {
    fn draw(&mut self, display: &mut D) -> Result<(), D::Error> {
        let (position, text_style) = if self.text_total_width() <= display.bounding_box().size.width
        {
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
                    let num_elapsed =
                        since_last_inc.div_duration_f32(self.offset_inc_interval) as i32;
                    self.offset =
                        (self.offset + num_elapsed) % self.max_offset_for(display).unwrap() as i32;
                    self.offset_last_incremented = Some(
                        Instant::now()
                            - (since_last_inc - (self.offset_inc_interval * num_elapsed as u32)),
                    );
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

    fn single_display_duration(&self, display: &D) -> Duration {
        match self.max_offset_for(display) {
            Some(o) => o * self.offset_inc_interval,
            None => Duration::from_secs(5),
        }
    }

    fn paused(&mut self, _for_dur: Duration) {
        self.offset_last_incremented = None;
        self.show_count = self.show_count.saturating_sub(1);
    }

    fn should_remove(&self) -> bool {
        self.show_count == 0
    }

    fn id(&self) -> &str {
        "text"
    }
}
