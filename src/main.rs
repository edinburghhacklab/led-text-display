use display::Display;
use rpi_led_panel::{HardwareMapping, NamedPixelMapperType, RGBMatrix, RGBMatrixConfig};

mod display;

fn main() {
    let config: RGBMatrixConfig = {
        let mut c = RGBMatrixConfig::default();
        c.hardware_mapping = HardwareMapping::adafruit_hat_pwm();
        c.rows = 32;
        c.cols = 192;
        c.pwm_bits = 1;
        // c.slowdown = Some(2);
        c.pixelmapper = vec![NamedPixelMapperType::Rotate(180)];

        c
    };

    let (matrix, canvas) = RGBMatrix::new(config, 0).expect("Matrix initialization failed");
    let display = Display::new(matrix, canvas);
    display.main_loop().unwrap();
}
