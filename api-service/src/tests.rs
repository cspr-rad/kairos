use std::io::Read;

use reqwest::blocking::Client;
use kairos_risc0_types::{Deposit, constants::FORMATTED_DEFAULT_ACCOUNT_STR};
use kairos_risc0_types::{Key, U512};
#[test]
fn transfer(){
    let client =  Client::new();
    let deposit = Deposit{
        account: Key::from_formatted_str(FORMATTED_DEFAULT_ACCOUNT_STR).unwrap(),
        amount: U512::from(1),
        timestamp: None,
        processed: false

    };
    let url = "http://127.0.0.1:3000/transfer";
    let response = client.post(url).body(serde_json::to_vec(&deposit).expect("Failed to serialize!")).send().expect("Failed to send!");

    let echo: Deposit = serde_json::from_slice(&response.bytes().unwrap()).unwrap();
    println!("Response: {:?}", &echo);
}