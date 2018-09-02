extern crate serde;
extern crate serde_json;

use failure::Error;
use reqwest::header::{Basic, Headers};
use reqwest::{StatusCode, Url};
use HTTP_CLIENT;

pub fn get_basic_credentials(username: &str, password: Option<String>) -> Basic {
    Basic {
        username: username.to_string(),
        password: password,
    }
}

pub fn get_url_response<T>(url_string: &str, headers: Headers) -> Result<(T, Headers), Error>
where
    T: serde::de::DeserializeOwned,
{
    if let Ok(url) = Url::parse(&url_string) {
        let mut response = HTTP_CLIENT.get(url).headers(headers).send()?;

        match response.status() {
            StatusCode::Ok => {
                let body_string = response.text()?;
                let deser = serde_json::from_str::<T>(body_string.as_str())?;
                //todo: Do we have to clone this?
                Ok((deser, response.headers().clone()))
            }
            other_code => Err(format_err!(
                "HTTP call to {} failed with code: {}",
                &url_string,
                other_code
            )),
        }
    } else {
        Err(format_err!("Unable to parse url: {}", url_string))
    }
}
