use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::{DrawTarget, RgbColor},
};
use screens::Screen;

pub mod screens;

/// An instance of a screen being displayed, along with information about how long it should be displayed for.
pub struct DisplayedScreen<D: DrawTarget<Color = Rgb888>> {
    inner: Box<dyn Screen<D>>,
}

impl<D: DrawTarget<Color = Rgb888>> DisplayedScreen<D> {
    /// Display the given screen for the default duration/time.
    pub fn new(inner: Box<dyn Screen<D>>) -> Self {
        Self { inner }
    }

    /// Draw a frame of the screen to the given display
    fn draw(&mut self, display: &mut D) -> Result<(), D::Error> {
        self.inner.draw(display)
    }

    /// Return true if the screen can keep drawing things
    fn wants_redraw(&self) -> bool {
        self.inner.wants_redraw()
    }

    /// Returns the desired duration for a single continuous display of this screen
    fn single_display_duration(&self) -> Duration {
        // todo!()
        Duration::from_secs(5)
    }

    /// Update the inner state to reflect the screen was displayed for the given duration
    fn displayed(&mut self, for_dur: Duration) {
        // todo!()
    }

    /// Returns true if the screen wants to be removed from the active rotation
    fn should_remove(&self) -> bool {
        false
        // todo!()
    }

    fn paused(&mut self) {
        self.inner.paused()
    }
}

/// Handles the main logic for displaying things to the LED.
/// Primarily, multiplexing between different [`screen::Screen`]s
pub struct DisplayLogic<D: DrawTarget<Color = Rgb888>> {
    curr_screens: VecDeque<DisplayedScreen<D>>,
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
    pub fn add(&mut self, sd: DisplayedScreen<D>) {
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
        if !screen.wants_redraw() || displayed_for >= screen.single_display_duration() {
            // Move to the next screen, possibly removing this one.
            screen.displayed(displayed_for);
            if screen.should_remove() {
                self.curr_screens.pop_front();
            } else {
                screen.paused();
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
