mod config_file;
use config_file::*;

mod jenkins_response;
use jenkins_response::*;

mod unity_cloud_response;
use unity_cloud_response::*;

mod team_city_response;
use team_city_response::*;

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
            let unity_api_token = config_values.unity_cloud_api_token;
            let team_city_username = config_values.team_city_username;
            let team_city_password = config_values.team_city_password;

            // Init threads that check build statuses
            let jenkins_handle = thread::spawn(move || {        
                loop {
                    print_jenkins_status(jenkins_username.as_str(), jenkins_password.as_str());                        
                    thread::sleep(Duration::from_millis(SLEEP_DURATION));
                }
            });

            let unity_cloud_handle = thread::spawn(move || {
                loop {            
                    print_unity_cloud_status(unity_api_token.as_str());
                    // todo: Add a check for what our allowed requests per minute, and adjust sleep duration as necessary.
                    thread::sleep(Duration::from_millis(SLEEP_DURATION));
                }        
            });

            let team_city_handle = thread::spawn(move || {
                loop {
                    print_team_city_status(team_city_username.as_str(), team_city_password.as_str());
                    thread::sleep(Duration::from_millis(SLEEP_DURATION));
                }
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

fn print_jenkins_status(username: &str, password: &str) {        
    let url_string = "http://52.58.239.149:8080/api/json";
    let mut auth_headers = Headers::new();
    auth_headers.set(Authorization(get_basic_credentials(username, Some(password.to_string()))));

    let all_jobs_response: Result<JenkinsJobResponse, Error> = get_url_reponse(&url_string, auth_headers.clone());   

    match all_jobs_response {
        Ok(result) => {               
            for job in result.jobs {
                let job_url_string = format!("http://52.58.239.149:8080/job/{}/lastBuild/api/json", job.name);
                let job_response: Result<JenkinsBuildResult, Error> = get_url_reponse(&job_url_string, auth_headers.clone());

                match job_response {
                    Ok(job_result) => {                           
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

fn print_team_city_status(username: &str, password: &str) {
    let url = "http://52.58.239.149:100/app/rest/builds/count:1";

    let mut headers = Headers::new();
    let auth_header = get_basic_credentials(username, Some(password.to_string()));
    // todo: check to see if we have a TCSESSION cookie, and use it instead of auth
    headers.set(Authorization(auth_header));
    headers.set(Accept(vec![qitem(mime::APPLICATION_JSON)]));

    let team_city_response: Result<TeamCityResponse, Error> = get_url_reponse(url, headers);
    match team_city_response {
        Ok(result) => {
            // TODO: Get and return cookie for faster auth in the future
            info!("Team City build status: {:?}", result.status);
        }
        Err(team_city_network_err) => {
            warn!("Failure getting Team City build status: {}", team_city_network_err);
        }
    }
}

fn print_unity_cloud_status(api_token: &str) {    
    let mut headers = Headers::new();
    let auth_header = get_basic_credentials(api_token, None);    
    headers.set(Authorization(auth_header));
    headers.set(ContentType::json());

    let ios_build_status = get_unity_ios_status(&headers);
    let android_build_response = get_unity_android_status(&headers);
    if ios_build_status == UnityBuildStatus::Success 
        && android_build_response == UnityBuildStatus::Success {
            info!("Unity Cloud Build: SUCCESS")
    }
    else {
        info!("Unity Cloud Build: FAILING");
    }
}

fn get_unity_ios_status(headers: &Headers) -> UnityBuildStatus {
    let ios_url_string = "https://build-api.cloud.unity3d.com/api/v1/orgs/futurice/projects/finavia-helsinki-airport/buildtargets/ios-development/builds?per_page=1";
    let ios_build_response: Result<Vec<UnityBuild>, Error> = get_url_reponse(&ios_url_string, headers.clone());
    match ios_build_response {
        Ok(mut ios_http_result) => {
            if ios_http_result.len() != 0 {
                return ios_http_result.remove(0).build_status;
            }
            else {
                warn!("No iOS builds retrieved from Unity Cloud. Aborting...");
                return UnityBuildStatus::Unknown;
            }
        },
        Err(ios_http_err) => {
            warn!("Failure getting Unity Cloud build iOS status: {}", ios_http_err);
            return UnityBuildStatus::Unknown;
        }
    }
}

fn get_unity_android_status(headers: &Headers) -> UnityBuildStatus {
    let android_url_string = "https://build-api.cloud.unity3d.com/api/v1/orgs/futurice/projects/finavia-helsinki-airport/buildtargets/android-development/builds?per_page=1";
    let android_build_response: Result<Vec<UnityBuild>, Error> = get_url_reponse(&android_url_string, headers.clone());
    match android_build_response {
        Ok(mut android_http_result) => {
            if android_http_result.len() != 0 {
                return android_http_result.remove(0).build_status;
            }
            else { 
                warn!("No Android builds retrieved from Unity Cloud. Aborting...");
                return UnityBuildStatus::Unknown;
            }
        }
        Err(android_http_err) => {
            warn!("Failure getting Unity Cloud build Android status: {}", android_http_err);
            return UnityBuildStatus::Unknown;
        }
    }
}

fn get_basic_credentials(username: &str, password: Option<String>) -> Basic {
    Basic {
        username: username.to_string(),
        password: password
    }
}

fn get_url_reponse<T>(url_string: &str, headers: Headers) -> Result<T, Error> 
    where T: serde::de::DeserializeOwned {
    if let Ok(url) = Url::parse(&url_string) {
        let mut response = HTTP_CLIENT.get(url)
            .headers(headers)
            .send()?;

        match response.status() {
            StatusCode::Ok => {
                let body_string = response.text()?;                
                let deser = serde_json::from_str::<T>(body_string.as_str())?;
                Ok(deser)
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