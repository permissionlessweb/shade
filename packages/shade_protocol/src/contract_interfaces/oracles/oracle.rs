use crate::math_compat::Uint128;
use crate::c_std::HumanAddr;
use crate::schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use crate::serde::{Deserialize, Serialize};

use crate::{
    contract_interfaces::{
        dex::dex::TradingPair,
    },
    utils::{asset::Contract, generic_response::ResponseStatus},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct IndexElement {
    pub symbol: String,
    pub weight: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct OracleConfig {
    pub admin: HumanAddr,
    pub band: Contract,
    pub sscrt: Contract,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub band: Contract,
    pub sscrt: Contract,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        admin: Option<HumanAddr>,
        band: Option<Contract>,
    },
    // Register Secret Swap or Sienna Pair (should be */sSCRT or sSCRT/*)
    RegisterPair {
        pair: Contract,
    },
    // Unregister Secret Swap Pair (opposite action to RegisterSswapPair)
    UnregisterPair {
        symbol: String,
        pair: Contract,
    },

    RegisterIndex {
        symbol: String,
        basket: Vec<IndexElement>,
    },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    UpdateConfig {
        status: ResponseStatus,
    },

    RegisterPair {
        status: ResponseStatus,
        symbol: String,
        pair: TradingPair,
    },
    UnregisterPair {
        status: ResponseStatus,
    },
    RegisterIndex {
        status: ResponseStatus,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Price { symbol: String },
    Prices { symbols: Vec<String> },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: OracleConfig },
}
