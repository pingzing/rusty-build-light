use sysfs_gpio::{Direction, Pin, Error};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread;
use std::time::Duration;


pub fn turn_pin_on(pin_number: u64) -> Result<(), Error> {
    let pin = Pin::new(pin_number);
    pin.with_exported(|| {
        info!("Turning pin {} on.", pin_number);
        pin.set_direction(Direction::High)?;
        pin.set_value(1)?;
        Ok(())
    })
}

pub fn turn_pin_off(pin_number: u64) -> Result<(), Error> {
    let pin = Pin::new(pin_number);
    pin.with_exported(|| {
        info!("Turning pin {} off.", pin_number);
        pin.set_direction(Direction::Low)?;
        pin.set_value(0)?;
        Ok(())
    })
}

pub fn blink_pin(pin_number: u64) -> Result<Sender<bool>, Error> {
    let (tx, rx) = channel();
    let pin = Pin::new(pin_number);
    let pin_result = pin.with_exported(move || {
        pin.set_direction(Direction::Low)?;
        loop {
            if let Ok(received_value) = rx.try_recv() {
                break;
            } else {
                turn_pin_on(pin_number)?;
                thread::sleep(Duration::from_millis(750));
                turn_pin_off(pin_number)?;
                thread::sleep(Duration::from_millis(750));
            }
        }
        Ok(())
    });
    match pin_result {
        Ok(_) => Ok(tx),
        Err(e) => Err(e)
    }
}