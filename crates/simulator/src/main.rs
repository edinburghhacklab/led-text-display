use embedded_graphics::{
    mono_font::{ascii::FONT_6X9, MonoTextStyle},
    pixelcolor::{BinaryColor, Rgb888},
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Rectangle},
    text::Text,
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use logic::{
    screens::{TestScreen, TextScreen},
    DisplayLogic, DisplayedScreen,
};
use std::{
    cell::Cell,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::sleep,
    time::{Duration, Instant},
};

const TARGET_FRAMERATE: u64 = 60;

fn main() -> Result<(), core::convert::Infallible> {
    let mut display = SimulatorDisplay::<Rgb888>::new(Size::new(192, 32));

    let keep_going = Arc::new(AtomicBool::new(true));
    ctrlc::set_handler({
        let keep_going2 = keep_going.clone();
        move || {
            keep_going2.store(false, Ordering::Relaxed);
        }
    })
    .expect("Error setting Ctrl-C handler");

    let output_settings = OutputSettingsBuilder::new()
        .pixel_spacing(2)
        .scale(8)
        .build();
    let mut window = Window::new("LED display simulator", &output_settings);

    let mut display_logic = DisplayLogic::default();

    // display_logic.add(DisplayedScreen::new(Box::new(TextScreen::with_text(
    //     "Hello, World!".to_string(),
    // ))));
    display_logic.add(DisplayedScreen::new(Box::new(TextScreen::with_text(
        "some much longer text that goes off the screen".to_string(),
    ))));
    // display_logic.add(DisplayedScreen::new(Box::new(TestScreen)));

    loop {
        if !keep_going.load(Ordering::Relaxed) {
            break;
        }

        let frame_start = Instant::now();
        display_logic.draw(&mut display)?;
        window.update(&display);
        sleep(frame_start + Duration::from_millis(1000 / TARGET_FRAMERATE) - Instant::now());
    }

    Ok(())
}
