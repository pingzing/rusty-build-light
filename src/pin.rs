use sysfs_gpio::{Direction, Pin};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::time::Duration;
use std::thread;

pub fn turn_led_on(pin_numbers: &Vec<u64> ) {
    for pin in pin_numbers.iter().map(|x| Pin::new(*x)) {
        pin.with_exported(|| {            
            pin.set_direction(Direction::High)?;
            pin.set_value(1)?;
            Ok(())
        });
    }
}

pub fn turn_led_off(pin_numbers: &Vec<u64> ) {
    for pin in pin_numbers.iter().map(|x| Pin::new(*x)) {
        pin.with_exported(|| {            
            pin.set_direction(Direction::Low)?;
            pin.set_value(0)?;
            Ok(())
        });
    }
}

pub fn blink_led(pin_numbers: &Vec<u64>) -> Sender<bool> {
    let (tx, rx) = channel();
    loop {
        if let Ok(received_value) = rx.try_recv() {
            break;
        } else {
            turn_led_on(pin_numbers);
            thread::sleep(Duration::from_millis(750));
            turn_led_on(pin_numbers);
            thread::sleep(Duration::from_millis(750));
        }
    }
    tx 
}