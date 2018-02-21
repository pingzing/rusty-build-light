use wiringpi;
use wiringpi::pin::Value::{High, Low};
use std::time::Duration;
use std::thread;

lazy_static!{
    static ref PI: wiringpi::WiringPi<wiringpi::pin::WiringPi> = wiringpi::setup();
}

pub fn turn_led_on(pin_numbers: Vec<u32> ) {
    for num in pin_numbers {
        // do stuff
    }
}

pub fn turn_led_off(pin_numbers: Vec<u32> ) {
    for num in pin_numbers {
        // do stuff
    }
}