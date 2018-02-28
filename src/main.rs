mod config_file;
use config_file::*;

mod jenkins_response;
use jenkins_response::*;

mod unity_cloud_response;
use unity_cloud_response::*;

mod team_city_response;
use team_city_response::*;

mod pin;
use pin::RgbLedLight;

mod errors;
use errors::UnityRetrievalError;

mod headers;

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
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate toml;
extern crate wiringpi;

use std::fs::File;
use std::io::prelude::*;
use std::time::Duration;
use std::thread;

use reqwest::{Client, Url, StatusCode};
use reqwest::header::{Accept, Authorization, Basic, ContentType, Headers, qitem};

use reqwest::mime;

use failure::Error;

use chrono::prelude::*;

const SLEEP_DURATION: u64 = 5000;
const UNITY_SLEEP_DURATION: u64 = 1000 * 60;

lazy_static!{
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();    
}

fn main() {
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
            let mut config_file = File::open(config_file_path)
                .unwrap_or_else(|err| {
                    error!("No config.toml found in /src directory. Error: {}", err);
                    panic!("Aborting...");
                });
            let mut config_text = String::new();
            config_file.read_to_string(&mut config_text)
                .unwrap_or_else(|err| {
                    error!("Failed to read config file. Error: {}", err);
                    panic!("Aborting...");
                });                

            let config_values: Config = toml::from_str(config_text.as_str())
                .unwrap_or_else(|err|{
                    error!("Failed to deserialize config file. Error: {}", err);
                    panic!("Aborting...");
                });
            let jenkins_username = config_values.jenkins_username;
            let jenkins_password = config_values.jenkins_password;
            let jenkins_base_url = config_values.jenkins_base_url;
            let (jenkins_r, jenkins_g, jenkins_b) = (config_values.jenkins_led_pins[0], config_values.jenkins_led_pins[1], config_values.jenkins_led_pins[2]);

            let unity_api_token = config_values.unity_cloud_api_token;
            let unity_base_url = config_values.unity_base_url;
            let (unity_r, unity_g, unity_b) = (config_values.unity_led_pins[0], config_values.unity_led_pins[1], config_values.unity_led_pins[2]);

            let team_city_username = config_values.team_city_username;
            let team_city_password = config_values.team_city_password;
            let team_city_base_url = config_values.team_city_base_url;
            let (team_city_r, team_city_g, team_city_b) = (config_values.team_city_led_pins[0], config_values.team_city_led_pins[1], config_values.team_city_led_pins[2]);

            // Init various check-status loops
            let jenkins_handle = thread::spawn(move || {
                let jenkins_led = RgbLedLight::new(jenkins_r, jenkins_g, jenkins_b);
                run_jenkins_loop(jenkins_led, jenkins_username.as_str(), jenkins_password.as_str(), jenkins_base_url.as_str())
            });

            let unity_cloud_handle = thread::spawn(move || {
                let unity_led = RgbLedLight::new(unity_r, unity_g, unity_b);
                run_unity_loop(unity_led, unity_api_token.as_str(), unity_base_url.as_str());
            });

            let team_city_handle = thread::spawn(move || {
                let team_city_led = RgbLedLight::new(team_city_r, team_city_g, team_city_b);
                run_team_city_loop(team_city_led, team_city_username.as_str(), team_city_password.as_str(), team_city_base_url.as_str());
            });
                        
            jenkins_handle.join().expect("Unable to join the Jenkins status thread.");
            unity_cloud_handle.join().expect("Unable to join the Unity Cloud build status thread.");
            team_city_handle.join().expect("Unable to join Team City build status thread.");
        }
        Err(e) => {
            error!("Failed to obtain current executable directory. Details: {}. Exiting...", e);
        }
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

    test_led.glow_led(RgbLedLight::TEAL);    
}

fn run_jenkins_loop(mut jenkins_led: RgbLedLight, jenkins_username: &str, jenkins_password: &str, jenkins_base_url: &str) {    
    run_power_on_test(&mut jenkins_led);
    loop {
        match get_jenkins_status(
            jenkins_username,
            jenkins_password,
            jenkins_base_url) 
        {
            Ok(results) => {
                let (retrieved, not_retrieved): (
                    Vec<Result<JenkinsBuildStatus, Error>>,
                    Vec<Result<JenkinsBuildStatus, Error>>,
                ) = results.into_iter().partition(|x| x.is_ok());

                let retrieved: Vec<JenkinsBuildStatus> = retrieved.into_iter().map(|x| x.unwrap()).collect();
                let retrieved_count = retrieved.len();
                let not_retrieved_count = not_retrieved.len();
                let build_failures = *(&retrieved.iter().filter(|x| **x != JenkinsBuildStatus::Success).count());
                let build_successes = *(&retrieved.iter().filter(|x| **x == JenkinsBuildStatus::Success).count());

                // Failure states: NONE of the builds succeeded.
                if build_successes <= 0 {
                    if not_retrieved_count > build_failures || build_failures == 0 {
                        // Glow blue if we fail to retrieve the majority, or if we have no success AND no failures
                        jenkins_led.glow_led(RgbLedLight::BLUE);
                    }
                    else {
                        jenkins_led.blink_led(RgbLedLight::RED);
                    }
                }

                // Success, or partial success states: at least SOME builds succeeded.
                else {
                    if build_failures == 0 {
                        jenkins_led.set_led_rgb_values(RgbLedLight::GREEN);
                    }
                    else if build_successes > build_failures {
                        jenkins_led.glow_led(RgbLedLight::GREEN);
                    }
                    else {
                        jenkins_led.blink_led(RgbLedLight::RED);
                    }
                }

                info!("--Jenkins--: Retrieved {} jobs, failed to retrieve {} jobs. Of those, {} succeeded, and {} failed.", retrieved_count, not_retrieved_count, build_successes, build_failures);
            },
            Err(e) => {
                    jenkins_led.glow_led(RgbLedLight::BLUE);
                warn!("--Jenkins--: Failed to retrieve any jobs from Jenkins. Details: {}", e);
            }
        }                      
        thread::sleep(Duration::from_millis(SLEEP_DURATION));
    }
}

fn get_jenkins_status(username: &str, password: &str, base_url: &str) -> Result<Vec<Result<JenkinsBuildStatus, Error>>, Error> {        
    let url_string = format!("{base}/api/json", base=base_url);
    let mut auth_headers = Headers::new();
    auth_headers.set(Authorization(get_basic_credentials(username, Some(password.to_string()))));

    let all_jobs_response: Result<(JenkinsJobResponse, Headers), Error> = get_url_response(&url_string, auth_headers.clone());   

    match all_jobs_response {
        Ok((result, _)) => {               
            let results = result.jobs.iter().map(|job| {
                let job_url_string = format!("{base}/job/{job}/lastBuild/api/json", base=base_url, job=job.name);
                let job_response: Result<(JenkinsBuildResult, Headers), Error> = get_url_response(&job_url_string, auth_headers.clone());                

                match job_response {
                    Ok((job_result, _)) => {
                        Ok(job_result.build_result)
                    }                        
                    Err(job_err) => {
                        warn!("HTTP failure when attempting to get job result for job: {}. Error: {}", &job_url_string, job_err);
                        Err(job_err)
                    }
                }
            }).collect();
            Ok(results)
        }
        Err(err) => {
            warn!("Error getting all jobs: {}", err);
            Err(err)
        }
    }
}

fn run_team_city_loop(mut team_city_led: RgbLedLight, team_city_username: &str, team_city_password: &str, team_city_base_url: &str) {    
    run_power_on_test(&mut team_city_led);
    loop {
        let team_city_status = get_team_city_status(team_city_username, 
                                                    team_city_password, 
                                                    team_city_base_url);
        match team_city_status {
            Some(status) => {
                match status {
                    TeamCityBuildStatus::Success => team_city_led.set_led_rgb_values(RgbLedLight::GREEN),
                    TeamCityBuildStatus::Failure => team_city_led.blink_led(RgbLedLight::RED),
                    TeamCityBuildStatus::Error => team_city_led.set_led_rgb_values(RgbLedLight::BLUE)
                }
            }
            None => {
                team_city_led.set_led_rgb_values(RgbLedLight::BLUE);
            }
        }

        thread::sleep(Duration::from_millis(SLEEP_DURATION));
    }
}

fn get_team_city_status(username: &str, password: &str, base_url: &str) -> Option<TeamCityBuildStatus> {
    let url = format!("{base}/app/rest/builds/count:1", base=base_url);

    let mut headers = Headers::new();
    let auth_header = get_basic_credentials(username, Some(password.to_string()));
    // todo: check to see if we have a TCSESSION cookie, and use it instead of auth
    headers.set(Authorization(auth_header));
    headers.set(Accept(vec![qitem(mime::APPLICATION_JSON)]));

    let team_city_response: Result<(TeamCityResponse, Headers), Error> = get_url_response(url.as_str(), headers);
    match team_city_response {
        Ok((result, _)) => {
            // TODO: Get and return cookie for faster auth in the future
            info!("--Team City--: Build status: {:?}", result.status);
            Some(result.status)
        }
        Err(team_city_network_err) => {
            warn!("--Team City--: Failed to get build status: {}", team_city_network_err);
            None
        }
    }
}

fn run_unity_loop(mut unity_led: RgbLedLight, unity_api_token: &str, unity_base_url: &str) {    
    let mut sleep_duration = UNITY_SLEEP_DURATION;
    run_power_on_test(&mut unity_led);
    loop {
        let unity_results = get_unity_cloud_status(unity_api_token, unity_base_url);
        let (retrieved, not_retrieved): (
            Vec<Result<(UnityBuildStatus, Headers), UnityRetrievalError>>,
            Vec<Result<(UnityBuildStatus, Headers), UnityRetrievalError>>,
        ) = unity_results.into_iter().partition(|x| x.is_ok());        

        let retrieved_results: Vec<(UnityBuildStatus, Headers)> = retrieved.into_iter().map(|x| x.unwrap()).collect();
        let not_retrieved_results: Vec<UnityRetrievalError> = not_retrieved.into_iter().map(|x| x.unwrap_err()).collect();                    

        if not_retrieved_results.len() > 0 {
            info!("--Unity--: At least one result no retrieved.");
            unity_led.glow_led(RgbLedLight::BLUE);
        } else {
            let passing_builds = *(&retrieved_results.iter().filter(|x| x.0 == UnityBuildStatus::Success).count());
            let failing_builds = *(&retrieved_results.iter().filter(|x| x.0 == UnityBuildStatus::Failure).count());
            let other_status_builds = *(&retrieved_results.iter().filter(|x| x.0 != UnityBuildStatus::Success && x.0 != UnityBuildStatus::Failure).count());

            // More misc statuses than knowns
            if other_status_builds > passing_builds + failing_builds {
                info!("--Unity--: More otherstatuses than passing AND failing.");
                unity_led.glow_led(RgbLedLight::BLUE);
            }
            // All passing or misc
            else if passing_builds > 0 && failing_builds == 0 {
                info!("--Unity--: All passing or misc.");
                unity_led.set_led_rgb_values(RgbLedLight::GREEN);
            } 
            // All failing or misc
            else if passing_builds == 0 && failing_builds > 0 {                            
                info!("--Unity--: All failing or misc.");
                unity_led.blink_led(RgbLedLight::RED);
            }
            // Both failing and passing
            else if passing_builds > 0 && failing_builds > 0 {
                info!("--Unity--: At least one failing AND passing.");
                unity_led.glow_led(RgbLedLight::GREEN);
            }
            // ?????
            else {
                info!("--Unity--: Default case. Glowing white.");
                unity_led.glow_led(RgbLedLight::WHITE);
            }

            info!("--Unity--: {} passing builds, {} failing builds, {} builds with misc statuses.", passing_builds, failing_builds, other_status_builds);
        }                    

        // Adjust our timeout based on current rate limiting (if possible)
        if retrieved_results.len() > 0 {
            // Grab any of the headers at random
            let response_headers = &retrieved_results[0].1;
            if let Some(limit_remaining) = response_headers.get::<headers::XRateLimitRemaining>() {
                let limit_remaining = limit_remaining.0;
                if let Some(reset_timestamp_utc) = response_headers.get::<headers::XRateLimitReset>() {
                    let reset_timestamp_utc = reset_timestamp_utc.0 as f32 / 1000f32; // Convert from milliseconds to seconds
                    let now_unix_seconds = Utc::now().timestamp() as u64;
                    let max_requests_per_second = limit_remaining as f32 / ((reset_timestamp_utc - now_unix_seconds as f32) as f32).max(1f32);
                    let seconds_per_request = (1f32 / max_requests_per_second).max(UNITY_SLEEP_DURATION as f32);
                    sleep_duration = seconds_per_request as u64;

                    let human_date: DateTime<Utc> = DateTime::from_utc(NaiveDateTime::from_timestamp(reset_timestamp_utc as i64, 0), Utc);
                    info!("--Unity--: Readjusting sleep duration per iteration to {}. RateLimit-Remaining was: {}. Reset-Timestamp was: {}. Will reset at: {}", 
                        sleep_duration, 
                        limit_remaining,
                        reset_timestamp_utc,
                        human_date);
                }
            }
        }                    

        // todo: Add a check for what our allowed requests per minute, and adjust sleep duration as necessary.
        thread::sleep(Duration::from_millis(sleep_duration));
    }        
}

fn get_unity_cloud_status(api_token: &str, base_url: &str) -> Vec<Result<(UnityBuildStatus, Headers), UnityRetrievalError>> {    
    let mut headers = Headers::new();
    let auth_header = get_basic_credentials(api_token, None);    
    headers.set(Authorization(auth_header));
    headers.set(ContentType::json());

    let ios_url = format!("{base}/buildtargets/ios-development/builds?per_page=1", base=base_url);    
    let ios_build_response = get_unity_status(&headers, ios_url.as_str());

    let android_url = format!("{base}/buildtargets/android-development/builds?per_page=1", base=base_url);
    let android_build_response = get_unity_status(&headers, android_url.as_str());
    vec!(ios_build_response, android_build_response)
}

fn get_unity_status(headers: &Headers, url: &str) -> Result<(UnityBuildStatus, Headers), UnityRetrievalError> {    
    let unity_build_response: Result<(Vec<UnityBuild>, Headers), Error> = get_url_response(&url, headers.clone());
    match unity_build_response {
        Ok((mut unity_http_result, response_headers)) => {
            if unity_http_result.len() != 0 {
                Ok((unity_http_result.remove(0).build_status, response_headers))
            }
            else {
                warn!("No builds retrieved from Unity Cloud for URL {}. Aborting...", url);
                Err(UnityRetrievalError::NoBuildsReturned)
            }
        },
        Err(unity_http_err) => {
            warn!("Failure getting Unity Cloud build status for url: {}. Error: {}", url, unity_http_err);
            Err(UnityRetrievalError::HttpError{ http_error_message: unity_http_err.to_string() })
        }
    }
}

fn get_basic_credentials(username: &str, password: Option<String>) -> Basic {
    Basic {
        username: username.to_string(),
        password: password
    }
}

fn get_url_response<T>(url_string: &str, headers: Headers) -> Result<(T, Headers), Error> 
    where T: serde::de::DeserializeOwned {
    if let Ok(url) = Url::parse(&url_string) {
        let mut response = HTTP_CLIENT.get(url)
            .headers(headers)
            .send()?;

        match response.status() {
            StatusCode::Ok => {
                let body_string = response.text()?;                
                let deser = serde_json::from_str::<T>(body_string.as_str())?;
                //todo: Do we have to clone this?
                Ok((deser, response.headers().clone()))
            }
            other_code => {
                Err(format_err!("HTTP call to {} failed with code: {}", &url_string, other_code))
            }
        }
    }

    else {
        Err(format_err!("Unable to parse url: {}", url_string))
    }
}