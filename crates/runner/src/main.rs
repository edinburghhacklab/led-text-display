use std::{env, sync::mpsc, thread};

use display::Display;
use logic::{screens::TestScreen, DisplayLogic};
use mqtt::MQTTListener;
use rpi_led_panel::{HardwareMapping, NamedPixelMapperType, RGBMatrix, RGBMatrixConfig};

mod display;
mod mqtt;

fn main() {
    env_logger::init();

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

    let (send, recv) = mpsc::channel();
    let mqtt = MQTTListener::new(
        env::var("MQTT_HOST")
            .unwrap_or_else(|_| "mqtt.hacklab".to_string())
            .as_str(),
        send,
    )
    .unwrap();

    let mut display_logic = DisplayLogic::new(recv);
    display_logic.add(Box::new(TestScreen));

    let (matrix, canvas) = RGBMatrix::new(config, 0).expect("Matrix initialization failed");
    let display = Display::new(matrix, canvas, display_logic);
    thread::scope(move |scope| {
        scope.spawn(move || {
            mqtt.main_loop();
        });
        scope.spawn(move || {
            display.main_loop().unwrap();
        });
    });
}
