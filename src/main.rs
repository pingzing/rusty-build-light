mod errors;
mod headers;
mod network;

mod integrations;
use integrations::jenkins_integration::JenkinsIntegration;
use integrations::remote_integration::RemoteIntegration;
use integrations::unity_cloud_integration::UnityCloudIntegration;

mod remote_status;
use remote_status::RemoteStatus;

mod config_file;
use config_file::*;

mod pin;
use pin::RgbLedLight;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate failure;

#[macro_use]
extern crate log;
extern crate log4rs;

#[macro_use]
extern crate hyper;

extern crate chrono;
extern crate ctrlc;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate toml;
extern crate wiringpi;

use std::fs::File;
use std::io::prelude::*;
use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const SLEEP_DURATION: u64 = 10000;

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}

fn main() {
    let is_running_flag = Arc::new(AtomicBool::new(true));
    let r = is_running_flag.clone();
    ctrlc::set_handler(move || {
        info!("Ctrl-C received, signaling child threads to stop...");
        r.store(false, Ordering::SeqCst); // signal that main should stop.
    }).unwrap_or_else(|_| {
        error!("Error setting Ctrl-C handler.");
        panic!("Aborting...");
    });

    let failure_count = Arc::new(Mutex::new(0u32));
    match std::env::current_exe() {
        Ok(path) => {
            // Init logging
            let mut log_config_file_path = std::path::PathBuf::from(path.parent().unwrap());
            log_config_file_path.push("log4rs.yml");
            println!("Looking for log config file at: {:?}", log_config_file_path);
            log4rs::init_file(log_config_file_path, Default::default()).unwrap();

            // Init config file
            let mut config_file_path = std::path::PathBuf::from(path.parent().unwrap());
            config_file_path.push("config.toml");
            info!("Looking for config file at: {:?}", config_file_path);
            let mut config_file = File::open(config_file_path).unwrap_or_else(|err| {
                error!("No config.toml found in /src directory. Error: {}", err);
                panic!("Aborting...");
            });
            let mut config_text = String::new();
            config_file
                .read_to_string(&mut config_text)
                .unwrap_or_else(|err| {
                    error!("Failed to read config file. Error: {}", err);
                    panic!("Aborting...");
                });

            let config_values: Config =
                toml::from_str(config_text.as_str()).unwrap_or_else(|err| {
                    error!("Failed to deserialize config file. Error: {}", err);
                    panic!("Aborting...");
                });
            let jenkins_username = config_values.jenkins_username;
            let jenkins_password = config_values.jenkins_password;
            let jenkins_base_url = config_values.jenkins_base_url;
            let jenkins_running_flag = is_running_flag.clone();
            let (jenkins_r, jenkins_g, jenkins_b) = (
                config_values.jenkins_led_pins[0],
                config_values.jenkins_led_pins[1],
                config_values.jenkins_led_pins[2],
            );

            let unity_api_token = config_values.unity_cloud_api_token;
            let unity_base_url = config_values.unity_base_url;
            let unity_running_flag = is_running_flag.clone();
            let (unity_r, unity_g, unity_b) = (
                config_values.unity_led_pins[0],
                config_values.unity_led_pins[1],
                config_values.unity_led_pins[2],
            );

            let allowed_total_failures = config_values.allowed_failures;

            // Init main threads
            let jenkins_counter = Arc::clone(&failure_count);
            let jenkins_handle = thread::spawn(move || {
                run_and_recover(
                    "Jenkins",
                    allowed_total_failures,
                    jenkins_counter,
                    jenkins_running_flag.clone(),
                    || {
                        let jenkins_integration = JenkinsIntegration::new(
                            jenkins_r,
                            jenkins_g,
                            jenkins_b,
                            &jenkins_username,
                            &jenkins_password,
                            &jenkins_base_url,
                        );
                        start_thread(jenkins_integration, jenkins_running_flag.clone())
                    },
                )
            });

            let unity_cloud_counter = Arc::clone(&failure_count);
            let unity_cloud_handle = thread::spawn(move || {
                run_and_recover(
                    "Unity Cloud",
                    allowed_total_failures,
                    unity_cloud_counter,
                    unity_running_flag.clone(),
                    || {
                        let unity_cloud_integration = UnityCloudIntegration::new(
                            unity_r,
                            unity_g,
                            unity_b,
                            &unity_api_token,
                            &unity_base_url,
                        );
                        start_thread(unity_cloud_integration, unity_running_flag.clone())
                    },
                )
            });

            // Wait for all main threads to finish.
            jenkins_handle
                .join()
                .expect("The Jenkins thread terminated abnormally.");
            unity_cloud_handle
                .join()
                .expect("The Unity Cloud build thread terminated abnormally.");

            info!("All threads terminated. Terminating program...");
        }
        Err(e) => {
            error!(
                "Failed to obtain current executable directory. Details: {}. Exiting...",
                e
            );
        }
    }
}

fn run_and_recover<F: Fn() -> R + panic::UnwindSafe + panic::RefUnwindSafe, R>(
    thread_name: &str,
    allowed_total_failures: u32,
    failure_counter: Arc<Mutex<u32>>,
    running_flag: Arc<AtomicBool>,
    func: F,
) -> thread::Result<R>
where
    R: std::fmt::Debug,
{
    loop {
        if let Ok(counter) = failure_counter.lock() {
            if *counter > allowed_total_failures {
                running_flag.store(false, Ordering::SeqCst); // Force a global stop
                return Result::Err(Box::new(format!(
                    "Failure count for {} exceeded, forcing stop.",
                    thread_name
                )));
            }
        }
        let thread_result = panic::catch_unwind(|| func());
        if thread_result.is_ok() {
            info!("Thread {} terminated gracefully. Ending...", thread_name);
            return thread_result;
        } else {
            error!(
                "Thread {} terminated abnormally. Details: {:?}. Restarting...",
                thread_name, thread_result
            );
            if let Ok(mut counter) = failure_counter.lock() {
                *counter += 1;
            } else {
                error!("Attempted to increment failure count for thread {}, but failed to acquire a lock on the counter.", thread_name);
            }
        }
    }
}

fn start_thread<T: RemoteIntegration>(mut remote: T, running_flag: Arc<AtomicBool>) {
    let mut led = RgbLedLight::new(
        remote.get_red_id(),
        remote.get_green_id(),
        remote.get_blue_id(),
    );
    run_power_on_test(&mut led);
    loop {
        match remote.get_status() {
            RemoteStatus::Unknown => led.glow_led(RgbLedLight::PURPLE),
            RemoteStatus::InProgress => led.glow_led_period(RgbLedLight::GREEN, 700),
            RemoteStatus::Passing => led.set_led_rgb_values(RgbLedLight::GREEN),
            RemoteStatus::Failing => led.blink_led(RgbLedLight::RED),
        }

        if !running_flag.load(Ordering::SeqCst) {
            led.glow_led(RgbLedLight::WHITE);
            thread::sleep(Duration::from_millis(1400)); // Should be long enough for a single "glow on -> glow off" cycle
            led.turn_led_off();
            return;
        }

        thread::sleep(Duration::from_millis(SLEEP_DURATION));
    }
}

fn run_power_on_test(test_led: &mut pin::RgbLedLight) {
    test_led.turn_led_off();
    thread::sleep(Duration::from_millis(1000));
    test_led.set_led_rgb_values(RgbLedLight::RED);
    thread::sleep(Duration::from_millis(250));
    test_led.set_led_rgb_values(RgbLedLight::GREEN);
    thread::sleep(Duration::from_millis(250));
    test_led.set_led_rgb_values(RgbLedLight::BLUE);
    thread::sleep(Duration::from_millis(250));
    test_led.turn_led_off();
    thread::sleep(Duration::from_millis(250));
    test_led.set_led_rgb_values(RgbLedLight::WHITE);
    thread::sleep(Duration::from_millis(250));
    test_led.turn_led_off();

    test_led.glow_led(RgbLedLight::PURPLE);
}
