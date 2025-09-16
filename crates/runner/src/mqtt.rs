use std::{
    io,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::Duration,
};

use embedded_graphics::{
    mono_font::{iso_8859_16::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::RgbColor,
};
use log::debug;
use logic::screens::{EnvironmentScreen, Screen, TextScreen};
use rpi_led_panel::Canvas;
use rumqttc::{Client, Event, Incoming, MqttOptions, Publish, QoS, SubscribeFilter};

pub struct MQTTListener {
    mqtt_options: MqttOptions,
    screen_channel: mpsc::Sender<Box<dyn Screen<Canvas>>>,
    screen_del_channel: mpsc::Sender<String>,
    next_colour: Rgb888,

    last_co2: u32,
    last_temp: f32,

    sleep: Arc<AtomicBool>,
}

const TEXT_COLOUR_TOPIC: &str = "display/g1/windowled/colour";
const TEXT_TOPIC: &str = "display/g1/windowled/text";

const TEMP_TOPIC: &str = "sensor/g1/temperature";
const CO2_TOPIC: &str = "environment/g1/elsys/co2";

const GLOBAL_PRESENCE_TOPIC: &str = "sensor/global/presence";

impl MQTTListener {
    pub fn new(
        conn_string: &str,
        screen_channel: mpsc::Sender<Box<dyn Screen<Canvas>>>,
        screen_del_channel: mpsc::Sender<String>,
        sleep: Arc<AtomicBool>,
    ) -> Result<Self, io::Error> {
        let mut mqtt_options = MqttOptions::new("rpiledmatrix", conn_string, 1883);
        mqtt_options.set_keep_alive(Duration::from_secs(30));

        Ok(Self {
            mqtt_options,
            screen_channel,
            screen_del_channel,
            next_colour: Rgb888::MAGENTA,
            last_co2: 0,
            last_temp: 0.0,
            sleep,
        })
    }

    pub fn main_loop(mut self) -> ! {
        let (client, mut connection) = Client::new(self.mqtt_options.clone(), 10);

        client
            .subscribe_many([
                SubscribeFilter::new(TEXT_TOPIC.to_string(), QoS::ExactlyOnce),
                SubscribeFilter::new(TEXT_COLOUR_TOPIC.to_string(), QoS::ExactlyOnce),
                SubscribeFilter::new(TEMP_TOPIC.to_string(), QoS::ExactlyOnce),
                SubscribeFilter::new(CO2_TOPIC.to_string(), QoS::ExactlyOnce),
                SubscribeFilter::new(GLOBAL_PRESENCE_TOPIC.to_string(), QoS::ExactlyOnce),
            ])
            .unwrap();
        loop {
            for notification in connection.iter() {
                let notification = notification.unwrap();
                let Event::Incoming(Incoming::Publish(msg)) = notification else {
                    continue;
                };

                let _ = self.attempt_handle_message(msg);
            }
        }
    }

    fn attempt_handle_message(&mut self, msg: Publish) -> Option<()> {
        let payload = String::from_utf8(msg.payload.to_vec()).ok()?;
        if msg.topic.as_str() == GLOBAL_PRESENCE_TOPIC {
            self.sleep.store(payload == "empty", Ordering::Relaxed);
            debug!("new sleep state: {}", payload == "empty");
        }

        if self.sleep.load(Ordering::Relaxed) {
            return None;
        }

        match msg.topic.as_str() {
            TEXT_COLOUR_TOPIC => {
                let mut iter = payload.split(",");
                let (r, g, b) = (iter.next()?, iter.next()?, iter.next()?);
                let (r, g, b) = (
                    u8::from_str(r).ok()?,
                    u8::from_str(g).ok()?,
                    u8::from_str(b).ok()?,
                );

                self.next_colour = Rgb888::new(r, g, b);

                Some(())
            }
            TEXT_TOPIC => {
                self.screen_channel
                    .send(Box::new(TextScreen::new(
                        payload,
                        MonoTextStyle::new(&FONT_10X20, self.next_colour),
                        None,
                    )))
                    .unwrap();

                Some(())
            }
            TEMP_TOPIC => {
                self.last_temp = payload.parse().ok()?;

                self.refresh_environment_screen();

                Some(())
            }
            CO2_TOPIC => {
                self.last_co2 = payload.parse().ok()?;

                self.refresh_environment_screen();

                Some(())
            }
            _ => None,
        }
    }

    fn refresh_environment_screen(&mut self) {
        self.screen_del_channel
            .send("environment".to_string())
            .unwrap();

        self.screen_channel
            .send(Box::new(EnvironmentScreen::new(
                self.last_temp,
                self.last_co2,
            )))
            .unwrap();
    }
}
