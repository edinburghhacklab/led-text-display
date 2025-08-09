use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::{DrawTarget, RgbColor},
};
use log::debug;
use screens::Screen;

pub mod screens;

/// Handles the main logic for displaying things to the LED.
/// Primarily, multiplexing between different [`screen::Screen`]s
pub struct DisplayLogic<D: DrawTarget<Color = Rgb888>> {
    curr_screens: VecDeque<Box<dyn Screen<D>>>,
    last_screen_change: Option<Instant>,
}

impl<D: DrawTarget<Color = Rgb888>> Default for DisplayLogic<D> {
    fn default() -> Self {
        Self {
            curr_screens: VecDeque::new(),
            last_screen_change: Default::default(),
        }
    }
}

impl<D: DrawTarget<Color = Rgb888>> DisplayLogic<D> {
    /// Add the given [`DisplayedScreen`] to the rotation
    pub fn add(&mut self, sd: Box<dyn Screen<D>>) {
        self.curr_screens.push_back(sd);
    }

    /// Draw a frame to the given display.
    pub fn draw(&mut self, display: &mut D) -> Result<(), D::Error> {
        let Some(screen) = self.curr_screens.front_mut() else {
            // When no screens are displayed, just clear it.
            return display.clear(Rgb888::BLACK);
        };

        // See if we need to move on to the next screen, and/or remove this screen.
        let last_screen_change = self.last_screen_change.get_or_insert(Instant::now());
        let displayed_for = Instant::now() - *last_screen_change;
        let single_display_duration = screen.single_display_duration(display);
        if displayed_for >= single_display_duration {
            debug!(
                "screen displayed for {:?} out of {:?}",
                displayed_for, single_display_duration
            );

            // Move to the next screen, possibly removing this one.
            screen.paused(displayed_for);
            if screen.should_remove() {
                debug!("removing current screen");
                self.curr_screens.pop_front();
            } else {
                debug!("going to next screen");
                self.curr_screens.rotate_left(1);
            }

            self.last_screen_change = Some(Instant::now());
            display.clear(Rgb888::BLACK)?;
        }

        // May have just removed the last screen, so check again.
        let Some(screen) = self.curr_screens.front_mut() else {
            // When no screens are displayed, just clear it.
            return display.clear(Rgb888::BLACK);
        };

        // Finally, draw the current screen.
        screen.draw(display)
    }
}
