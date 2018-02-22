use sysfs_gpio::{Direction, Pin};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::time::Duration;
use std::thread;

#[derive(Debug, Clone, Copy)]
pub struct RgbLedLight {
    pub red_pin: u64,
    pub green_pin: u64,
    pub blue_pin: u64
}

impl IntoIterator for RgbLedLight {
    type Item = u64;
    type IntoIter = RgbLedLightIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        RgbLedLightIntoIterator {rgb_led_light: self, index: 0} 
    }
}

pub struct RgbLedLightIntoIterator {
    rgb_led_light: RgbLedLight,
    index: usize
}

impl Iterator for RgbLedLightIntoIterator {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        let result = match self.index {
            0 => self.rgb_led_light.red_pin,
            1 => self.rgb_led_light.green_pin,
            2 => self.rgb_led_light.blue_pin,
            _ => return None
        };
        self.index += 1;
        Some(result)
    }
}

pub fn turn_led_on(led: RgbLedLight) {
    for pin in led.into_iter().map(|x| Pin::new(x)) {
        if !pin.is_exported() {
            pin.export();
        }

        pin.set_direction(Direction::High);
        pin.set_value(1);
    }
}

pub fn turn_led_off(led: RgbLedLight) {
    for pin in led.into_iter().map(|x| Pin::new(x)) {
        if !pin.is_exported() {
            pin.export();
        }

        pin.set_direction(Direction::Low);
        pin.set_value(0);
    }
}

pub fn set_led_rgb_values(led: RgbLedLight, r: u8, g: u8, b: u8) {
    let pin_list: Vec<Pin> = led.into_iter().map(|x| Pin::new(x)).collect();
    for pin in &pin_list {
        if !pin.is_exported() {
            pin.export();
        }

        pin.set_direction(Direction::Out);
    }

    pin_list[0].set_value(r);
    pin_list[1].set_value(g);
    pin_list[2].set_value(b);    
}

pub fn blink_led(led: RgbLedLight) -> Sender<bool> {
    let (tx, rx) = channel();
    thread::spawn(move || {
         loop {
            if let Ok(received_value) = rx.try_recv() {
                break;
            } else {
                turn_led_on(led.clone());
                thread::sleep(Duration::from_millis(750));
                turn_led_off(led.clone());
                thread::sleep(Duration::from_millis(750));
            }
        }
    });   
    tx 
}

// todo: read this from config file or something
pub fn get_led_1() -> RgbLedLight {
    RgbLedLight { red_pin: 2, green_pin: 3, blue_pin: 4 }
}