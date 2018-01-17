mod config_file;
use config_file::*;

mod jenkins_response;
use jenkins_response::*;

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
use reqwest::header::{Authorization, Basic, Header};
use failure::Error;

const SLEEP_DURATION: u64 = 5000;

lazy_static!{
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}

fn main() {
    let mut config_file = File::open("./config.toml").expect("No config.toml found in /src directory. Aborting...");
    let mut config_text = String::new();
    config_file.read_to_string(&mut config_text).expect("Failed to read config file");

    let config_values: Config = toml::from_str(config_text.as_str()).expect("Failed to deserialize config file.");    

    let jenkins_handle = thread::spawn(move || {        
        loop {
            print_jenkins_status(config_values.jenkins_username.clone().as_str(), config_values.jenkins_password.clone().as_str());                        
            thread::sleep(Duration::from_millis(SLEEP_DURATION));
        }
    });

    let btc_handle = thread::spawn(|| {
        loop {            
            //print_jenkins_status();
            //thread::sleep(Duration::from_millis(SLEEP_DURATION));
        }        
    });

    jenkins_handle.join().expect("Unable to join the Jenkins status thread.");
    btc_handle.join().expect("Unable to join the BTC thread.");
}

fn get_basic_credentials(username: &str, password: &str) -> Basic {
    Basic {
        username: username.to_string(),
        password: Some(password.to_string())
    }
}

fn get_url_reponse<T, H: Header>(url_string: &str, auth_header: H) -> Result<T, Error> 
    where T: serde::de::DeserializeOwned {
    if let Ok(url) = Url::parse(&url_string) {
        let mut response = HTTP_CLIENT.get(url)
            .header(auth_header)
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
    let all_jobs_response: Result<JenkinsJobResponse, Error> = get_url_reponse(&url_string, 
        Authorization(get_basic_credentials(username, password)));   

    match all_jobs_response {
        Ok(result) => {               
            for job in result.jobs {
                let job_url_string = format!("http://52.58.239.149:8080/job/{}/lastBuild/api/json", job.name);
                let job_response: Result<JenkinsBuildResult, Error> = get_url_reponse(&job_url_string, 
                    Authorization(get_basic_credentials(username, password)));

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