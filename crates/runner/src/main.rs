use display::Display;
use logic::{
    screens::{TestScreen, TextScreen},
    DisplayLogic,
};
use rpi_led_panel::{HardwareMapping, NamedPixelMapperType, RGBMatrix, RGBMatrixConfig};

mod display;

fn main() {
    let config: RGBMatrixConfig = {
        let mut c = RGBMatrixConfig::default();
        c.hardware_mapping = HardwareMapping::adafruit_hat_pwm();
        c.rows = 32;
        c.cols = 192;
        c.refresh_rate = 120;
        c.pwm_bits = 11;
        c.pwm_lsb_nanoseconds = 130;
        c.dither_bits = 0;
        c.led_brightness = 50;
        c.slowdown = Some(1);
        c.pixelmapper = vec![NamedPixelMapperType::Rotate(180)];

        c
    };

    let mut display_logic = DisplayLogic::default();
    display_logic.add(Box::new(TextScreen::with_text("Hello, World!".to_string())));
    display_logic.add(Box::new(TextScreen::with_text(
        "some much longer text that goes off the screen".to_string(),
    )));
    display_logic.add(Box::new(TestScreen));

    let (matrix, canvas) = RGBMatrix::new(config, 0).expect("Matrix initialization failed");
    let display = Display::new(matrix, canvas, display_logic);
    display.main_loop().unwrap();
}
