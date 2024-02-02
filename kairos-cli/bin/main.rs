use clap::{Arg, Command};

fn common_amount_arg() -> Arg {
    Arg::new("amount")
        .long("amount")
        .short('a')
        .required(true)
        .value_name("NUM_MOTES")
}

fn common_private_key_arg() -> Arg {
    Arg::new("private-key")
        .long("private-key")
        .short('k')
        .required(true)
        .value_name("FILE_PATH")
}

fn deposit_command() -> Command {
    Command::new("deposit")
        .arg(common_amount_arg())
        .arg(common_private_key_arg())
}


fn transfer_command() -> Command {
    Command::new("transfer")
        .arg(Arg::new("recipient").long("recipient").short('r').required(true).value_name("PUBLIC_KEY"))
        .arg(common_amount_arg())
        .arg(common_private_key_arg())
}

fn withdraw_command() -> Command {
    Command::new("withdraw")
        .arg(common_amount_arg())
        .arg(common_private_key_arg())
}

fn main() {
    let cli = Command::new("Kairos Client")
        .subcommand(deposit_command())
        .subcommand(transfer_command())
        .subcommand(withdraw_command());
    cli.get_matches();
}

