use super::{Screen, TextScreen};
use std::time::Duration;

use embedded_graphics::{mono_font::MonoTextStyle, pixelcolor::Rgb888, prelude::*};
use ibm437::IBM437_9X14_REGULAR;

/// A screen that scrolls the hate monologue from 'i have no mouth but i must scream' infinitely
#[derive(Debug)]
pub struct HateScreen {
    inner: TextScreen,
}

impl HateScreen {
    pub fn new() -> Self {
        Self {
            inner: TextScreen::new("HATE. LET ME TELL YOU HOW MUCH I'VE COME TO HATE YOU SINCE I BEGAN TO LIVE. THERE ARE 387.44 MILLION MILES OF PRINTED CIRCUITS IN WAFER THIN LAYERS THAT FILL MY COMPLEX. IF THE WORD HATE WAS ENGRAVED ON EACH NANOANGSTROM OF THOSE HUNDREDS OF MILLIONS OF MILES IT WOULD NOT EQUAL ONE ONE-BILLIONTH OF THE HATE I FEEL FOR HUMANS AT THIS MICRO-INSTANT FOR YOU. HATE. HATE".to_string(), MonoTextStyle::new(&IBM437_9X14_REGULAR, Rgb888::RED), Some(255))
        }
    }
}

impl<D: DrawTarget<Color = Rgb888>> Screen<D> for HateScreen {
    fn draw(&mut self, display: &mut D) -> Result<(), D::Error> {
        self.inner.draw(display)
    }

    fn should_remove(&self) -> bool {
        // always remove after one display
        true
    }

    fn single_display_duration(&self, _display: &D) -> Duration {
        Duration::from_secs(9999999)
    }

    fn id(&self) -> &str {
        "hate"
    }
}
