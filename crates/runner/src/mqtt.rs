use std::{
    io,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::{Duration, SystemTime},
};

use embedded_graphics::{
    mono_font::{iso_8859_16::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::RgbColor,
};
use log::{debug, warn};
use logic::screens::{EnvironmentScreen, HateScreen, Screen, TextScreen};
use rpi_led_panel::Canvas;
use rumqttc::{Client, Event, Incoming, MqttOptions, Outgoing, Publish, QoS, SubscribeFilter};

/// Deals with listening on the MQTT bus, and sending messages to the logic based off of that.
pub struct MQTTListener {
    /// Client stuff
    mqtt_options: MqttOptions,

    /// Communicating with the logic
    screen_channel: mpsc::Sender<Box<dyn Screen<Canvas>>>,
    screen_del_channel: mpsc::Sender<String>,

    /// For text screen
    next_colour: Rgb888,

    /// For environment screen
    last_co2: Option<(u32, SystemTime)>,
    last_temp: Option<(f32, SystemTime)>,

    sleep: Arc<AtomicBool>,
}

// Topics for text screen
const TEXT_COLOUR_TOPIC: &str = "display/g1/windowled/colour";
const TEXT_TOPIC: &str = "display/g1/windowled/text";

// Topics for environment screen
const TEMP_TOPIC: &str = "environment/g1/elsys/temperature";
const CO2_TOPIC: &str = "environment/g1/elsys/co2";

const GLOBAL_PRESENCE_TOPIC: &str = "sensor/global/presence";
const CHECK_TIME_EXPIRY_TOPIC: &str = "timesignal/300";

const CATASTROPHE_LEVER_TOPIC: &str = "catastrophe/state/lever";

const ENVIRONMENT_DATA_TIMEOUT: Duration = Duration::from_mins(2);

impl MQTTListener {
    /// Create a new listener for the given MQTT server, communicating with the logic loop via the given channels.
    pub fn new(
        conn_string: &str,
        screen_channel: mpsc::Sender<Box<dyn Screen<Canvas>>>,
        screen_del_channel: mpsc::Sender<String>,
        sleep: Arc<AtomicBool>,
    ) -> Result<Self, io::Error> {
        let mut mqtt_options = MqttOptions::new("rpiledmatrix", conn_string, 1883);
        mqtt_options.set_keep_alive(Duration::from_secs(120));

        Ok(Self {
            mqtt_options,
            screen_channel,
            screen_del_channel,
            next_colour: Rgb888::MAGENTA,
            last_co2: None,
            last_temp: None,
            sleep,
        })
    }

    /// Run the main MQTT loop, processing events and sending the results to the logic loop.
    pub fn main_loop(mut self) -> ! {
        // Setup
        let (mut client, mut connection) = Client::new(self.mqtt_options.clone(), 10);
        client
            .subscribe_many([
                SubscribeFilter::new(TEXT_TOPIC.to_string(), QoS::ExactlyOnce),
                SubscribeFilter::new(TEXT_COLOUR_TOPIC.to_string(), QoS::ExactlyOnce),
                SubscribeFilter::new(TEMP_TOPIC.to_string(), QoS::ExactlyOnce),
                SubscribeFilter::new(CO2_TOPIC.to_string(), QoS::ExactlyOnce),
                SubscribeFilter::new(GLOBAL_PRESENCE_TOPIC.to_string(), QoS::ExactlyOnce),
                SubscribeFilter::new(CHECK_TIME_EXPIRY_TOPIC.to_string(), QoS::AtMostOnce),
                SubscribeFilter::new(CATASTROPHE_LEVER_TOPIC.to_string(), QoS::ExactlyOnce),
            ])
            .unwrap();

        self.refresh_environment_screen();

        loop {
            // Process messages
            for notification in connection.iter() {
                let notification = notification.unwrap();
                match notification {
                    Event::Incoming(Incoming::Publish(msg)) => {
                        let _ = self.attempt_handle_message(msg, &mut client);
                    }
                    Event::Incoming(Incoming::Disconnect)
                    | Event::Outgoing(Outgoing::Disconnect) => panic!("disconnect"),
                    _ => continue,
                }
            }
        }
    }

    /// Attempt to handle a single message.
    fn attempt_handle_message(&mut self, msg: Publish, client: &mut Client) -> Option<()> {
        let payload = String::from_utf8(msg.payload.to_vec()).ok()?;

        // If asleep, try not to process so much
        if msg.topic.as_str() == GLOBAL_PRESENCE_TOPIC {
            self.sleep.store(payload == "empty", Ordering::Relaxed);
            debug!("new sleep state: {}", payload == "empty");
        }
        if self.sleep.load(Ordering::Relaxed) {
            return None;
        }

        match msg.topic.as_str() {
            // Text display
            TEXT_COLOUR_TOPIC => {
                // Store text colour, for future message on the text topic to use
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
                // Show some text
                self.screen_channel
                    .send(Box::new(TextScreen::new(
                        payload,
                        MonoTextStyle::new(&FONT_10X20, self.next_colour),
                        None,
                    )))
                    .unwrap();

                Some(())
            }

            // Environment display
            TEMP_TOPIC => {
                let val: f32 = payload.parse().ok()?;
                debug!("received new temperature reading: {val}");
                if val < 0.001 {
                    debug!("rejecting faulty temperature reading");
                    return Some(());
                }

                self.last_temp = Some((val, SystemTime::now()));

                self.refresh_environment_screen();

                Some(())
            }
            CO2_TOPIC => {
                let val: u32 = payload.parse().ok()?;
                debug!("received new co2 reading: {val}");
                if val == 0 {
                    debug!("rejecting faulty co2 reading");
                    return Some(());
                }
                self.last_co2 = Some((val, SystemTime::now()));

                self.refresh_environment_screen();

                Some(())
            }

            CHECK_TIME_EXPIRY_TOPIC => {
                debug!("checking expiry of temp/co2 data");

                let mut refresh = false;
                if self.last_temp.is_some_and(|x| {
                    SystemTime::now().duration_since(x.1).unwrap() > ENVIRONMENT_DATA_TIMEOUT
                }) {
                    debug!("temp expired");
                    self.last_temp = None;
                    refresh = true;
                }

                if self.last_co2.is_some_and(|x| {
                    SystemTime::now().duration_since(x.1).unwrap() > ENVIRONMENT_DATA_TIMEOUT
                }) {
                    debug!("co2 expired");
                    self.last_co2 = None;
                    refresh = true;
                }

                if refresh {
                    warn!("one or more data points expired");
                    self.refresh_environment_screen();
                }

                Some(())
            }

            CATASTROPHE_LEVER_TOPIC => {
                self.screen_del_channel.send("hate".to_string()).unwrap();
                if payload == "on" {
                    self.screen_channel.send(Box::new(HateScreen::new())).ok()?;
                    client
                        .publish("display/g1/leds", QoS::AtLeastOnce, false, "red")
                        .ok()?;
                } else {
                    client
                        .publish("display/g1/leds", QoS::AtLeastOnce, false, "rainbow")
                        .ok()?;
                }

                Some(())
            }

            _ => None,
        }
    }

    /// Delete and replace the environment screen, based on updated info in `self`
    fn refresh_environment_screen(&mut self) {
        self.screen_del_channel
            .send("environment".to_string())
            .unwrap();

        self.screen_channel
            .send(Box::new(EnvironmentScreen::new(
                self.last_temp.map(|x| x.0),
                self.last_co2.map(|x| x.0),
            )))
            .unwrap();
    }
}
