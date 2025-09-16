use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::Instant,
};

use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::{DrawTarget, RgbColor},
};
use log::debug;
use screens::Screen;

pub mod screens;

mod recolour_image;

/// Handles the main logic for displaying things to the LED.
/// Primarily, multiplexing between different [`screen::Screen`]s
pub struct DisplayLogic<D: DrawTarget<Color = Rgb888>> {
    curr_screens: VecDeque<Box<dyn Screen<D>>>,
    last_screen_change: Option<Instant>,
    recv_screen: mpsc::Receiver<Box<dyn Screen<D>>>,
    recv_del_screen: mpsc::Receiver<String>,
    sleep: Arc<AtomicBool>,
}

impl<D: DrawTarget<Color = Rgb888> + 'static> DisplayLogic<D> {
    pub fn new(
        recv_screen: mpsc::Receiver<Box<dyn Screen<D>>>,
        recv_del_screen: mpsc::Receiver<String>,
        sleep: Arc<AtomicBool>,
    ) -> Self {
        Self {
            recv_screen,
            recv_del_screen,
            curr_screens: VecDeque::new(),
            last_screen_change: Default::default(),
            sleep,
        }
    }
    /// Add the given [`DisplayedScreen`] to the rotation
    pub fn add(&mut self, sd: Box<dyn Screen<D>>) {
        self.curr_screens.push_back(sd);
    }

    /// Draw a frame to the given display.
    pub fn draw(&mut self, display: &mut D) -> Result<(), D::Error> {
        if self.sleep.load(Ordering::Relaxed) {
            display.clear(Rgb888::BLACK)?;
            return Ok(());
        }

        // Add/delete screens now if needed
        let mut iter = self.recv_del_screen.try_iter();
        while let Some(del_screen) = iter.next() {
            debug!("deleting screens with id {:?}", del_screen);
            self.curr_screens.retain(|s| s.id() != del_screen);
        }

        let mut iter = self.recv_screen.try_iter();
        while let Some(new_screen) = iter.next() {
            debug!("got new screen: {:?}", new_screen);
            self.curr_screens.push_back(new_screen);
        }

        let Some(screen) = self.curr_screens.front_mut() else {
            // When no screens are displayed, just clear it.
            self.last_screen_change = None;
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
                debug!("going to next screen (len = {})", self.curr_screens.len());
                self.curr_screens.rotate_left(1);
            }

            self.last_screen_change = Some(Instant::now());
            display.clear(Rgb888::BLACK)?;

            debug!("new screen: {:?}", self.curr_screens.front());
        }

        // May have just removed the last screen, so check again.
        let Some(screen) = self.curr_screens.front_mut() else {
            // When no screens are displayed, just clear it.
            self.last_screen_change = None;
            return display.clear(Rgb888::BLACK);
        };

        // Finally, draw the current screen.
        screen.draw(display)
    }
}
