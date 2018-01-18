mod config_file;
use config_file::*;

mod jenkins_response;
use jenkins_response::*;

mod unity_cloud_response;
use unity_cloud_response::*;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate failure;

extern crate serde;
extern crate serde_json;
extern crate reqwest;
extern crate toml;

use std::fs::File;
use std::io::prelude::*;
use std::time::Duration;
use std::thread;
use reqwest::{Client, Url, StatusCode};
use reqwest::header::{Authorization, Basic, ContentType, Header, Headers};
use failure::Error;

const SLEEP_DURATION: u64 = 5000;

lazy_static!{
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}

fn main() {    
    match std::env::current_exe() {
        Ok(path) => {
            let mut config_file_path = std::path::PathBuf::from(path.parent().unwrap());
            config_file_path.push("config.toml");
            println!("Looking for config file at: {:?}", config_file_path);
            let mut config_file = File::open(config_file_path).expect("No config.toml found in /src directory. Aborting...");
            let mut config_text = String::new();
            config_file.read_to_string(&mut config_text).expect("Failed to read config file");

            let config_values: Config = toml::from_str(config_text.as_str()).expect("Failed to deserialize config file.");
            let jenkins_username = config_values.jenkins_username;
            let jenkins_password = config_values.jenkins_password;
            let unity_api_token = config_values.unity_cloud_api_token;

            let jenkins_handle = thread::spawn(move || {        
                loop {
                    print_jenkins_status(jenkins_username.as_str(), jenkins_password.as_str());                        
                    thread::sleep(Duration::from_millis(SLEEP_DURATION));
                }
            });

            let btc_handle = thread::spawn(move || {
                loop {            
                    print_unity_cloud_status(unity_api_token.as_str());
                    thread::sleep(Duration::from_millis(SLEEP_DURATION));
                }        
            });

            jenkins_handle.join().expect("Unable to join the Jenkins status thread.");
            btc_handle.join().expect("Unable to join the Unity Cloud build status thread.");
        }
        Err(e) => {
            println!("Failed to obtain current executable directory. Details: {}. Exiting...", e);
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
                            println!("Job {} result: {}", job.name, job_result.build_result);
                    }                        
                    Err(job_err) => {
                        println!("HTTP failure when attempting to get job result for job: {}. Error: {}", &job_url_string, job_err);
                    }
                }
            }
        }
        Err(err) => {
            println!("Error getting all jobs: {}", err);
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
            println!("Unity Cloud Build: SUCCESS")
    }
    else {
        println!("Unity Cloud Build: FAILING");
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
                println!("No iOS builds retrieved from Unity Cloud. Aborting...");
                return UnityBuildStatus::Unknown;
            }
        },
        Err(ios_http_err) => {
            println!("Failure getting Unity Cloud build iOS status: {}", ios_http_err);
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
                println!("No Android builds retrieved from Unity Cloud. Aborting...");
                return UnityBuildStatus::Unknown;
            }
        }
        Err(android_http_err) => {
            println!("Failure getting Unity Cloud build Android status: {}", android_http_err);
            return UnityBuildStatus::Unknown;
        }
    }
}