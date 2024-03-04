use std::i64;

use crate::deployments::{
    call_create_purse, call_incr_counter, get_counter, get_deposit_event, put_deposit_session,
    put_withdrawal_session,
};
use casper_types::{URef, U512};
use clap::{Parser, Subcommand};
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

// I want to allow CAPS here,
// because these enum variants are constant methods.
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Subcommand)]
enum Commands {
    CREATE_PURSE {
        node_address: String,
        rpc_port: String,
        secret_key_path: String,
        chain_name: String,
        contract_addr: String,
    },
    INCR_COUNTER {
        node_address: String,
        rpc_port: String,
        secret_key_path: String,
        chain_name: String,
        contract_addr: String,
    },
    DEPOSIT {
        node_address: String,
        rpc_port: String,
        secret_key_path: String,
        chain_name: String,
        wasm_path: String,
        contract_addr: String,
        amount: i64,
    },
    WITHDRAWAL {
        node_address: String,
        rpc_port: String,
        secret_key_path: String,
        chain_name: String,
        wasm_path: String,
        contract_addr: String,
        amount: i64,
        destination: String,
    },
    GET_DEPOSIT {
        node_address: String,
        rpc_port: String,
        dict_uref: String,
        key: String,
    },
    GET_COUNTER {
        node_address: String,
        rpc_port: String,
        counter_uref: String,
    },
}

pub async fn commander() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::CREATE_PURSE {
            node_address,
            rpc_port,
            secret_key_path,
            chain_name,
            contract_addr,
        }) => {
            let deploy_hash = call_create_purse::call(
                node_address.to_owned(),
                rpc_port.to_owned(),
                secret_key_path.to_owned(),
                chain_name,
                contract_addr,
            )
            .await;
            println!("Deploy Hash: {:?}", &deploy_hash);
        }
        Some(Commands::INCR_COUNTER {
            node_address,
            rpc_port,
            secret_key_path,
            chain_name,
            contract_addr,
        }) => {
            let deploy_hash = call_incr_counter::call(
                node_address.to_owned(),
                rpc_port.to_owned(),
                secret_key_path.to_owned(),
                chain_name,
                contract_addr,
            )
            .await;
            println!("Deploy Hash: {:?}", &deploy_hash);
        }
        Some(Commands::DEPOSIT {
            node_address,
            rpc_port,
            secret_key_path,
            chain_name,
            wasm_path,
            contract_addr,
            amount,
        }) => {
            let deploy_hash = put_deposit_session::put(
                node_address.to_owned(),
                rpc_port.to_owned(),
                secret_key_path.to_owned(),
                chain_name,
                wasm_path,
                contract_addr,
                U512::from(*amount),
            )
            .await;
            println!("Deploy Hash: {:?}", &deploy_hash);
        }
        Some(Commands::WITHDRAWAL {
            node_address,
            rpc_port,
            secret_key_path,
            chain_name,
            wasm_path,
            contract_addr,
            amount,
            destination,
        }) => {
            let withdrawal_hash = put_withdrawal_session::put(
                node_address.to_owned(),
                rpc_port.to_owned(),
                secret_key_path.to_owned(),
                chain_name,
                wasm_path,
                contract_addr,
                U512::from(*amount),
                URef::from_formatted_str(destination).unwrap(),
            )
            .await;
            println!("Deploy Hash: {:?}", &withdrawal_hash);
        }
        Some(Commands::GET_DEPOSIT {
            node_address,
            rpc_port,
            dict_uref,
            key,
        }) => {
            let deposit = get_deposit_event::get(
                node_address,
                rpc_port.to_owned(),
                URef::from_formatted_str(dict_uref).unwrap(),
                key.to_owned(),
            )
            .await;
            println!("Found Deposit Event: {:?}", &deposit);
        }
        Some(Commands::GET_COUNTER {
            node_address,
            rpc_port,
            counter_uref,
        }) => {
            let value = get_counter::get(
                node_address,
                rpc_port.to_owned(),
                URef::from_formatted_str(counter_uref).unwrap(),
            )
            .await;
            println!("Counter value: {:?}", &value);
        }
        None => {
            println!(
                "
                Possible commands:
                
                create_purse, get_counter, get_deposit, incr_counter, deposit, withdrawal
            "
            );
        }
    }
}
