use std::time::Duration;
use std::thread;
use std::sync::{mpsc, Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use wiringpi;
use wiringpi::*;

lazy_static! {
    static ref PI: WiringPi<pin::Gpio> = wiringpi::setup_gpio();
}

pub struct RgbLedLight {
    red_pin: wiringpi::pin::SoftPwmPin<wiringpi::pin::Gpio>,
    green_pin: wiringpi::pin::SoftPwmPin<wiringpi::pin::Gpio>,
    blue_pin: wiringpi::pin::SoftPwmPin<wiringpi::pin::Gpio>,
    is_blinking: Arc<Mutex<bool>>,
    stop_blinking_transmitter: Option<Sender<bool>>,
}

impl RgbLedLight {
    pub const RED: (i32, i32, i32) = (100, 0, 0);
    pub const GREEN: (i32, i32, i32) = (0, 100, 0);
    pub const BLUE: (i32, i32, i32) = (0, 0, 100);
    pub const TEAL: (i32, i32, i32) = (0, 100, 100);
    pub const YELLOW: (i32, i32, i32) = (100, 100, 0);
    pub const WHITE: (i32, i32, i32) = (100, 100, 00);
    pub const PURPLE: (i32, i32, i32) = (100, 0, 100);

    pub fn new(red: u16, green: u16, blue: u16) -> RgbLedLight {
        RgbLedLight {
            red_pin: PI.soft_pwm_pin(red),
            green_pin: PI.soft_pwm_pin(green),
            blue_pin: PI.soft_pwm_pin(blue),
            is_blinking: Arc::new(Mutex::new(false)),
            stop_blinking_transmitter: None,
        }
    }

    pub fn turn_led_on(&mut self) {
        self.stop_blinking();
        self.turn_led_on_internal();
    }

    pub fn turn_led_off(&mut self) {
        self.stop_blinking();
        self.turn_led_off_internal();
    }

    pub fn set_led_rgb_values(&mut self, rgb: (i32, i32, i32)) {
        self.stop_blinking();
        let (r, g, b) = rgb;
        self.set_led_rgb_values_internal(r, g, b);
    }

    pub fn blink_led(&mut self, rgb: (i32, i32, i32)) {
        if self.is_blinking() {
            self.stop_blinking();
        }

        let mut led_clone = RgbLedLight {
            red_pin: PI.soft_pwm_pin(self.red_pin.number() as u16),
            green_pin: PI.soft_pwm_pin(self.green_pin.number() as u16),
            blue_pin: PI.soft_pwm_pin(self.blue_pin.number() as u16),
            is_blinking: Arc::new(Mutex::new(false)),
            stop_blinking_transmitter: None,
        };

        let (r, g, b) = rgb; //destructure the tuple, so we can refer to individual values

        self.start_blinking();
        let (tx, rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();
        self.stop_blinking_transmitter = Some(tx);
        // reference to self.is_blinking, so the thread can safely watch it for value changes
        let is_blinking = self.is_blinking.clone();
        thread::spawn(move || loop {
            if rx.try_recv().is_ok() {
                return;
            }
            led_clone.set_led_rgb_values_internal(r, g, b);
            thread::sleep(Duration::from_millis(750));

            if rx.try_recv().is_ok() {
                return;
            }
            led_clone.turn_led_off_internal();
            thread::sleep(Duration::from_millis(750));
        });
    }

    pub fn glow_led(&mut self, rgb: (i32, i32, i32)) {
        if self.is_blinking() {
            self.stop_blinking();
        }

        let mut led_clone = RgbLedLight {
            red_pin: PI.soft_pwm_pin(self.red_pin.number() as u16),
            green_pin: PI.soft_pwm_pin(self.green_pin.number() as u16),
            blue_pin: PI.soft_pwm_pin(self.blue_pin.number() as u16),
            is_blinking: Arc::new(Mutex::new(false)),
            stop_blinking_transmitter: None,
        };

        let (r, g, b) = rgb; //destructure the tuple, so we can refer to individual values

        self.start_blinking();
        let (tx, rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();
        self.stop_blinking_transmitter = Some(tx);
        thread::spawn(move || loop {
            if rx.try_recv().is_ok() {
                return;
            }
            for i in 0..101 {
                if rx.try_recv().is_ok() {
                    return;
                }
                let partial_red = ((i as f32 / 100f32) * r as f32) as i32;
                let partial_green = ((i as f32 / 100f32) * g as f32) as i32;
                let partial_blue = ((i as f32 / 100f32) * b as f32) as i32;
                led_clone.set_led_rgb_values_internal(partial_red, partial_green, partial_blue);
                thread::sleep(Duration::from_millis(7));
            }

            for i in (0..101).rev() {
                if rx.try_recv().is_ok() {
                    return;
                }

                let partial_red = ((i as f32 / 100f32) * r as f32) as i32;
                let partial_green = ((i as f32 / 100f32) * g as f32) as i32;
                let partial_blue = ((i as f32 / 100f32) * b as f32) as i32;
                led_clone.set_led_rgb_values_internal(partial_red, partial_green, partial_blue);
                thread::sleep(Duration::from_millis(7));
            }
        });
    }

    fn turn_led_on_internal(&mut self) {
        self.red_pin.pwm_write(100);
        self.green_pin.pwm_write(100);
        self.blue_pin.pwm_write(100);
    }

    fn turn_led_off_internal(&mut self) {
        self.red_pin.pwm_write(0);
        self.green_pin.pwm_write(0);
        self.blue_pin.pwm_write(0);
    }

    fn set_led_rgb_values_internal(&mut self, r: i32, g: i32, b: i32) {
        self.red_pin.pwm_write(r);
        self.green_pin.pwm_write(g);
        self.blue_pin.pwm_write(b);
    }

    fn start_blinking(&mut self) {
        let mut is_blinking = self.is_blinking.lock().unwrap();
        *is_blinking = true;
    }

    fn stop_blinking(&mut self) {
        if let Some(ref tx) = self.stop_blinking_transmitter {
            tx.send(true);
        }
        let mut is_blinking = self.is_blinking.lock().unwrap();
        *is_blinking = false;
    }

    fn is_blinking(&mut self) -> bool {
        let is_blinking = self.is_blinking.lock().unwrap();
        return *is_blinking;
    }
}
