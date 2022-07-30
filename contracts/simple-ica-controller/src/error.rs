use thiserror::Error;

use cosmwasm_std::{Coin, StdError};

use cw1_whitelist::ContractError as whitelist_error;
use simple_ica::SimpleIcaError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    SimpleIca(#[from] SimpleIcaError),

    #[error("No account for channel {0}")]
    UnregisteredChannel(String),

    #[error("remote account changed from {old} to {addr}")]
    RemoteAccountChanged { addr: String, old: String },

    #[error("you must send the coins you wish to ibc transfer")]
    EmptyFund {},

    #[error("you can only ibc transfer one coin")]
    TooManyCoins { coins: Vec<Coin> },

    #[error("Invalid remote address for this channel")]
    InvalidRemoteAddr { addr: String, channel: String },

    #[error("Unaothorized")]
    Whitelist(#[from] whitelist_error),
}
