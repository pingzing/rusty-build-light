mod coinbase_response;
use coinbase_response::CoinbaseResponse;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

extern crate serde;
extern crate serde_json;
extern crate reqwest;

use std::time::Duration;
use std::thread;
use reqwest::{Client, Url};

const SLEEP_DURATION: u64 = 5000;

lazy_static!{
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}


//https://api.coinbase.com/v2/prices/{coinType}-USD/spot"
fn main() {
    let ltc_handle = thread::spawn(|| {
        loop {
            print_coin_values("ltc");                        
            thread::sleep(Duration::from_millis(SLEEP_DURATION));
        }
    });

    let btc_handle = thread::spawn(|| {
        loop {            
            print_coin_values("btc");
            thread::sleep(Duration::from_millis(SLEEP_DURATION));
        }        
    });

    ltc_handle.join().expect("Unable to join the LTC thread.");
    btc_handle.join().expect("Unable to join the BTC thread.");
}

fn print_coin_values(currency_type: &str) {
    let uri_string = format!("https://api.coinbase.com/v2/prices/{}-USD/spot", currency_type);
    if let Ok(url) = Url::parse(uri_string.as_str()) {
        let response = HTTP_CLIENT.get(url).send();
        match response {
            Ok(mut result) => {
                let body = result.text().unwrap_or("".to_string());
                if let Ok(result) = serde_json::from_str::<CoinbaseResponse>(body.as_str()) {
                    println!("Got {} response: Currency: {}, Amount: {}", result.data.currency, 
                                                                          result.data.currency, 
                                                                          result.data.amount);
                } 
                else {
                    println!("Unable to deserialize the response to a CoinbaseResponse.");
                }
            }
            Err(err) => {
                println!("Error getting BTC: {}", err);
            }
        }    
    }
    else {
        println!("Failed to parse '{}' into a valid URL.", uri_string);
    }
}