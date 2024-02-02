use clap::Arg;

pub mod amount {
    use super::*;

    pub fn arg() -> Arg {
        Arg::new("amount")
            .long("amount")
            .short('a')
            .required(true)
            .value_name("NUM_MOTES")
    }
}

pub mod private_key {
    use super::*;

    pub fn arg() -> Arg {
        Arg::new("private-key")
            .long("private-key")
            .short('k')
            .required(true)
            .value_name("FILE_PATH")
    }
}
