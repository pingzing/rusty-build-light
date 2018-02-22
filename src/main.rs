mod config_file;
use config_file::*;

mod jenkins_response;
use jenkins_response::*;

mod unity_cloud_response;
use unity_cloud_response::*;

mod team_city_response;
use team_city_response::*;

mod pin;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate failure;

#[macro_use]
extern crate log;
extern crate log4rs;

extern crate serde;
extern crate serde_json;
extern crate reqwest;
extern crate toml;
extern crate sysfs_gpio;

use std::fs::File;
use std::io::prelude::*;
use std::time::Duration;
use std::thread;
use reqwest::{Client, Url, StatusCode};
use reqwest::header::{Accept, Authorization, Basic, ContentType, Headers, qitem};
use reqwest::mime;
use failure::Error;

const SLEEP_DURATION: u64 = 5000;

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

            let unity_api_token = config_values.unity_cloud_api_token;
            let unity_base_url = config_values.unity_base_url;

            let team_city_username = config_values.team_city_username;
            let team_city_password = config_values.team_city_password;
            let team_city_base_url = config_values.team_city_base_url;  

            let led_test_thread = thread::spawn(move || {
                loop {
                    let pin_numbers = vec!(2, 3, 4);
                    info!("Turning pins 2 3 and 4 on!");
                    pin::turn_led_on(&pin_numbers);
                    thread::sleep(Duration::from_millis(2000));
                                        
                    info!("Turning pins 2 3 and 4 off!");
                    pin::turn_led_off(&pin_numbers);
                    thread::sleep(Duration::from_millis(2000));
                }
            });

            // Init threads that check build statuses
            let jenkins_handle = thread::spawn(move || {        
                loop {
                    print_jenkins_status(jenkins_username.as_str(), jenkins_password.as_str(), jenkins_base_url.as_str());                        
                    thread::sleep(Duration::from_millis(SLEEP_DURATION));
                }
            });

            let unity_cloud_handle = thread::spawn(move || {
                loop {            
                    print_unity_cloud_status(unity_api_token.as_str(), unity_base_url.as_str());
                    // todo: Add a check for what our allowed requests per minute, and adjust sleep duration as necessary.
                    thread::sleep(Duration::from_millis(SLEEP_DURATION));
                }        
            });

            let team_city_handle = thread::spawn(move || {
                loop {
                    print_team_city_status(team_city_username.as_str(), team_city_password.as_str(), team_city_base_url.as_str());
                    thread::sleep(Duration::from_millis(SLEEP_DURATION));
                }
            });
            
            led_test_thread.join().expect("Unable to join the LED test thread.");
            jenkins_handle.join().expect("Unable to join the Jenkins status thread.");
            unity_cloud_handle.join().expect("Unable to join the Unity Cloud build status thread.");
            team_city_handle.join().expect("Unable to join Team City build status thread.");
        }
        Err(e) => {
            error!("Failed to obtain current executable directory. Details: {}. Exiting...", e);
        }
    }    
}

fn print_jenkins_status(username: &str, password: &str, base_url: &str) {        
    let url_string = format!("{base}/api/json", base=base_url);
    let mut auth_headers = Headers::new();
    auth_headers.set(Authorization(get_basic_credentials(username, Some(password.to_string()))));

    let all_jobs_response: Result<(JenkinsJobResponse, Headers), Error> = get_url_response(&url_string, auth_headers.clone());   

    match all_jobs_response {
        Ok((result, all_jobs_response_headers)) => {               
            for job in result.jobs {
                let job_url_string = format!("{base}/job/{job}/lastBuild/api/json", base=base_url, job=job.name);
                let job_response: Result<(JenkinsBuildResult, Headers), Error> = get_url_response(&job_url_string, auth_headers.clone());

                match job_response {
                    Ok((job_result, single_job_reponse_headers)) => {                           
                            info!("Job {} result: {}", job.name, job_result.build_result);
                    }                        
                    Err(job_err) => {
                        warn!("HTTP failure when attempting to get job result for job: {}. Error: {}", &job_url_string, job_err);
                    }
                }
            }
        }
        Err(err) => {
            warn!("Error getting all jobs: {}", err);
        }
    }    
}

fn print_team_city_status(username: &str, password: &str, base_url: &str) {
    let url = format!("{base}/app/rest/builds/count:1", base=base_url);

    let mut headers = Headers::new();
    let auth_header = get_basic_credentials(username, Some(password.to_string()));
    // todo: check to see if we have a TCSESSION cookie, and use it instead of auth
    headers.set(Authorization(auth_header));
    headers.set(Accept(vec![qitem(mime::APPLICATION_JSON)]));

    let team_city_response: Result<(TeamCityResponse, Headers), Error> = get_url_response(url.as_str(), headers);
    match team_city_response {
        Ok((result, response_headers)) => {
            // TODO: Get and return cookie for faster auth in the future
            info!("Team City build status: {:?}", result.status);
        }
        Err(team_city_network_err) => {
            warn!("Failure getting Team City build status: {}", team_city_network_err);
        }
    }
}

fn print_unity_cloud_status(api_token: &str, base_url: &str) {    
    let mut headers = Headers::new();
    let auth_header = get_basic_credentials(api_token, None);    
    headers.set(Authorization(auth_header));
    headers.set(ContentType::json());

    let ios_url = format!("{base}/buildtargets/ios-development/builds?per_page=1", base=base_url);    
    let ios_build_response = get_unity_status(&headers, ios_url.as_str());

    let android_url = format!("{base}/buildtargets/android-development/builds?per_page=1", base=base_url);
    let android_build_response = get_unity_status(&headers, android_url.as_str());
    if ios_build_response == UnityBuildStatus::Success 
        && android_build_response == UnityBuildStatus::Success {
            info!("Unity Cloud Build: SUCCESS")
    }
    else {
        info!("Unity Cloud Build: FAILING");
    }
}

fn get_unity_status(headers: &Headers, url: &str) -> UnityBuildStatus {    
    let unity_build_response: Result<(Vec<UnityBuild>, Headers), Error> = get_url_response(&url, headers.clone());
    match unity_build_response {
        Ok((mut unity_http_result, response_headers)) => {
            if unity_http_result.len() != 0 {
                return unity_http_result.remove(0).build_status;
            }
            else {
                warn!("No builds retrieved from Unity Cloud for URL {}. Aborting...", url);
                UnityBuildStatus::Unknown
            }
        },
        Err(unity_http_err) => {
            warn!("Failure getting Unity Cloud build status for url: {}. Error: {}", url, unity_http_err);
            UnityBuildStatus::Unknown
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