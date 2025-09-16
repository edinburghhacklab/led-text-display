use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay, Window};
use logic::DisplayLogic;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread::sleep,
    time::{Duration, Instant},
};

const TARGET_FRAMERATE: u64 = 120;

mod local;

fn main() -> Result<(), core::convert::Infallible> {
    // Boring stuff: Logging, deal with ctrl+c
    env_logger::init();
    let keep_going = Arc::new(AtomicBool::new(true));
    ctrlc::set_handler({
        let keep_going2 = keep_going.clone();
        move || {
            keep_going2.store(false, Ordering::Relaxed);
        }
    })
    .expect("Error setting Ctrl-C handler");

    // Setup output window
    let mut display = SimulatorDisplay::<Rgb888>::new(Size::new(192, 32));
    let output_settings = OutputSettingsBuilder::new()
        .pixel_spacing(2)
        .scale(4)
        .build();
    let mut window = Window::new("LED display simulator", &output_settings);

    // Wire up display logic
    let (_, recv) = mpsc::channel();
    let (_, recv_del) = mpsc::channel();
    let _sleep = Arc::new(AtomicBool::new(false));
    let mut display_logic = DisplayLogic::new(recv, recv_del, _sleep);

    // Copy local.rs.tmpl to local.rs
    local::add_screens(&mut display_logic);

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
