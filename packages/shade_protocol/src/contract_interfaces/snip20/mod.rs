pub mod manager; //TODO: fragment this so it doesnt import useless stuff in interface
pub mod batch;
pub mod transaction_history;
pub mod errors;
pub mod helpers;

use cosmwasm_std::MessageInfo;
use crate::c_std::{Binary, Env, Addr, StdError, StdResult, Storage};
use crate::query_authentication::permit::Permit;

use secret_toolkit::crypto::sha_256;
use crate::utils::{HandleCallback, InitCallback, Query};
use cosmwasm_schema::{cw_serde};
use crate::c_std::Uint128;
use crate::contract_interfaces::snip20::errors::{invalid_decimals, invalid_name_format, invalid_symbol_format};
use crate::contract_interfaces::snip20::manager::{Admin, Balance, CoinInfo, Config, ContractStatusLevel, Minters, RandSeed, TotalSupply};
use crate::contract_interfaces::snip20::transaction_history::{RichTx, Tx};
#[cfg(feature = "snip20-impl")]
use crate::contract_interfaces::snip20::transaction_history::store_mint;
use crate::utils::generic_response::ResponseStatus;
#[cfg(feature = "snip20-impl")]
use crate::utils::storage::plus::ItemStorage;
#[cfg(feature = "snip20-impl")]
use secret_storage_plus::Item;

pub const VERSION: &str = "SNIP24";

#[cw_serde]
pub struct InitialBalance {
    pub address: Addr,
    pub amount: Uint128,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
    pub admin: Option<Addr>,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Option<Vec<InitialBalance>>,
    pub prng_seed: Binary,
    pub config: Option<InitConfig>,
}

fn is_valid_name(name: &str) -> bool {
    let len = name.len();
    (3..=30).contains(&len)
}

fn is_valid_symbol(symbol: &str) -> bool {
    let len = symbol.len();
    let len_is_valid = (3..=6).contains(&len);

    len_is_valid && symbol.bytes().all(|byte| (b'A'..=b'Z').contains(&byte))
}

#[cfg(feature = "snip20-impl")]
impl InstantiateMsg {
    pub fn save(&self, storage: &mut dyn Storage, env: Env, info: MessageInfo) -> StdResult<()> {
        if !is_valid_name(&self.name) {
            return Err(invalid_name_format(&self.name));
        }

        if !is_valid_symbol(&self.symbol) {
            return Err(invalid_symbol_format(&self.symbol));
        }

        if self.decimals > 18 {
            return Err(invalid_decimals());
        }

        let config = self.config.clone().unwrap_or_default();
        config.save(storage)?;

        CoinInfo {
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            decimals: self.decimals
        }.save(storage)?;

        let admin = self.admin.clone().unwrap_or(info.sender);
        Admin(admin.clone()).save(storage)?;
        RandSeed(sha_256(&self.prng_seed.0).to_vec()).save(storage)?;

        let mut total_supply = Uint128::zero();

        if let Some(initial_balances) = &self.initial_balances{
            for balance in initial_balances.iter() {
                Balance::set(storage, balance.amount.clone(), &balance.address)?;
                total_supply = total_supply.checked_add(balance.amount)?;

                store_mint(
                    storage,
                    &admin,
                    &balance.address,
                    balance.amount,
                    self.symbol.clone(),
                    Some("Initial Balance".to_string()),
                    &env.block
                )?;
            }
        }

        TotalSupply::set(storage, total_supply)?;

        ContractStatusLevel::NormalRun.save(storage)?;

        Minters(vec![]).save(storage)?;

        Ok(())
    }
}

#[cw_serde]
pub struct InitConfig {
    /// Indicates whether the total supply is public or should be kept secret.
    /// default: False
    pub public_total_supply: Option<bool>,
    /// Indicates whether deposit functionality should be enabled
    /// default: False
    pub enable_deposit: Option<bool>,
    /// Indicates whether redeem functionality should be enabled
    /// default: False
    pub enable_redeem: Option<bool>,
    /// Indicates whether mint functionality should be enabled
    /// default: False
    pub enable_mint: Option<bool>,
    /// Indicates whether burn functionality should be enabled
    /// default: False
    pub enable_burn: Option<bool>,
    /// Indicates whether transferring tokens should be enables
    /// default: True
    pub enable_transfer: Option<bool>,
}

impl Default for InitConfig {
    fn default() -> Self {
        Self {
            public_total_supply: None,
            enable_deposit: None,
            enable_redeem: None,
            enable_mint: None,
            enable_burn: None,
            enable_transfer: None
        }
    }
}

#[cfg(feature = "snip20-impl")]
impl InitConfig {
    pub fn save(self, storage: &mut dyn Storage) -> StdResult<()> {
        Config {
            public_total_supply: self.public_total_supply(),
            enable_deposit: self.deposit_enabled(),
            enable_redeem: self.redeem_enabled(),
            enable_mint: self.mint_enabled(),
            enable_burn: self.burn_enabled(),
            enable_transfer: self.transfer_enabled()
        }.save(storage)?;
        Ok(())
    }
    pub fn public_total_supply(&self) -> bool {
        self.public_total_supply.unwrap_or(false)
    }
    pub fn deposit_enabled(&self) -> bool {
        self.enable_deposit.unwrap_or(false)
    }
    pub fn redeem_enabled(&self) -> bool {
        self.enable_redeem.unwrap_or(false)
    }
    pub fn mint_enabled(&self) -> bool {
        self.enable_mint.unwrap_or(false)
    }
    pub fn burn_enabled(&self) -> bool {
        self.enable_burn.unwrap_or(false)
    }
    pub fn transfer_enabled(&self) -> bool {
        self.enable_transfer.unwrap_or(true)
    }
}

impl InitCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    // Native coin interactions
    Redeem {
        amount: Uint128,
        denom: Option<String>,
        padding: Option<String>,
    },
    Deposit {
        padding: Option<String>,
    },

    // Base ERC-20 stuff
    Transfer {
        recipient: Addr,
        amount: Uint128,
        memo: Option<String>,
        padding: Option<String>,
    },
    Send {
        recipient: Addr,
        recipient_code_hash: Option<String>,
        amount: Uint128,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>,
    },
    BatchTransfer {
        actions: Vec<batch::TransferAction>,
        padding: Option<String>,
    },
    BatchSend {
        actions: Vec<batch::SendAction>,
        padding: Option<String>,
    },
    Burn {
        amount: Uint128,
        memo: Option<String>,
        padding: Option<String>,
    },
    RegisterReceive {
        code_hash: String,
        padding: Option<String>,
    },
    CreateViewingKey {
        entropy: String,
        padding: Option<String>,
    },
    SetViewingKey {
        key: String,
        padding: Option<String>,
    },

    // Allowance
    IncreaseAllowance {
        spender: Addr,
        amount: Uint128,
        expiration: Option<u64>,
        padding: Option<String>,
    },
    DecreaseAllowance {
        spender: Addr,
        amount: Uint128,
        expiration: Option<u64>,
        padding: Option<String>,
    },
    TransferFrom {
        owner: Addr,
        recipient: Addr,
        amount: Uint128,
        memo: Option<String>,
        padding: Option<String>,
    },
    SendFrom {
        owner: Addr,
        recipient: Addr,
        recipient_code_hash: Option<String>,
        amount: Uint128,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>,
    },
    BatchTransferFrom {
        actions: Vec<batch::TransferFromAction>,
        padding: Option<String>,
    },
    BatchSendFrom {
        actions: Vec<batch::SendFromAction>,
        padding: Option<String>,
    },
    BurnFrom {
        owner: Addr,
        amount: Uint128,
        memo: Option<String>,
        padding: Option<String>,
    },
    BatchBurnFrom {
        actions: Vec<batch::BurnFromAction>,
        padding: Option<String>,
    },

    // Mint
    Mint {
        recipient: Addr,
        amount: Uint128,
        memo: Option<String>,
        padding: Option<String>,
    },
    BatchMint {
        actions: Vec<batch::MintAction>,
        padding: Option<String>,
    },
    AddMinters {
        minters: Vec<Addr>,
        padding: Option<String>,
    },
    RemoveMinters {
        minters: Vec<Addr>,
        padding: Option<String>,
    },
    SetMinters {
        minters: Vec<Addr>,
        padding: Option<String>,
    },

    // Admin
    ChangeAdmin {
        address: Addr,
        padding: Option<String>,
    },
    SetContractStatus {
        level: ContractStatusLevel,
        padding: Option<String>,
    },

    // Permit
    RevokePermit {
        permit_name: String,
        padding: Option<String>,
    },
}

impl HandleCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub struct Snip20ReceiveMsg {
    pub sender: Addr,
    pub from: Addr,
    pub amount: Uint128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    pub msg: Option<Binary>,
}

#[cw_serde]
pub enum ReceiverHandleMsg {
    Receive(Snip20ReceiveMsg),
}

impl ReceiverHandleMsg {
    pub fn new(
        sender: Addr,
        from: Addr,
        amount: Uint128,
        memo: Option<String>,
        msg: Option<Binary>,
    ) -> Self {
        Self::Receive(Snip20ReceiveMsg{
            sender,
            from,
            amount,
            memo,
            msg
        })
    }
}

impl HandleCallback for ReceiverHandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum HandleAnswer {
    // Native
    Deposit {
        status: ResponseStatus,
    },
    Redeem {
        status: ResponseStatus,
    },

    // Base
    Transfer {
        status: ResponseStatus,
    },
    Send {
        status: ResponseStatus,
    },
    BatchTransfer {
        status: ResponseStatus,
    },
    BatchSend {
        status: ResponseStatus,
    },
    Burn {
        status: ResponseStatus,
    },
    RegisterReceive {
        status: ResponseStatus,
    },
    CreateViewingKey {
        key: String,
    },
    SetViewingKey {
        status: ResponseStatus,
    },

    // Allowance
    IncreaseAllowance {
        spender: Addr,
        owner: Addr,
        allowance: Uint128,
    },
    DecreaseAllowance {
        spender: Addr,
        owner: Addr,
        allowance: Uint128,
    },
    TransferFrom {
        status: ResponseStatus,
    },
    SendFrom {
        status: ResponseStatus,
    },
    BatchTransferFrom {
        status: ResponseStatus,
    },
    BatchSendFrom {
        status: ResponseStatus,
    },
    BurnFrom {
        status: ResponseStatus,
    },
    BatchBurnFrom {
        status: ResponseStatus,
    },

    // Mint
    Mint {
        status: ResponseStatus,
    },
    BatchMint {
        status: ResponseStatus,
    },
    AddMinters {
        status: ResponseStatus,
    },
    RemoveMinters {
        status: ResponseStatus,
    },
    SetMinters {
        status: ResponseStatus,
    },

    // Other
    ChangeAdmin {
        status: ResponseStatus,
    },
    SetContractStatus {
        status: ResponseStatus,
    },

    // Permit
    RevokePermit {
        status: ResponseStatus,
    },
}

pub type QueryPermit = Permit<PermitParams>;

#[cw_serde]
pub struct PermitParams {
    pub allowed_tokens: Vec<Addr>,
    pub permit_name: String,
    pub permissions: Vec<Permission>,
}

impl PermitParams {
    pub fn contains(&self, perm: Permission) -> bool {
        self.permissions.contains(&perm)
    }
}

#[cw_serde]
pub enum Permission {
    /// Allowance for SNIP-20 - Permission to query allowance of the owner & spender
    Allowance,
    /// Balance for SNIP-20 - Permission to query balance
    Balance,
    /// History for SNIP-20 - Permission to query transfer_history & transaction_hisotry
    History,
    /// Owner permission indicates that the bearer of this permit should be granted all
    /// the access of the creator/signer of the permit.  SNIP-721 uses this to grant
    /// viewing access to all data that the permit creator owns and is whitelisted for.
    /// For SNIP-721 use, a permit with Owner permission should NEVER be given to
    /// anyone else.  If someone wants to share private data, they should whitelist
    /// the address they want to share with via a SetWhitelistedApproval tx, and that
    /// address will view the data by creating their own permit with Owner permission
    Owner,
}

#[cw_serde]
pub enum QueryMsg {
    TokenInfo {},
    TokenConfig {},
    ContractStatus {},
    ExchangeRate {},
    Allowance {
        owner: Addr,
        spender: Addr,
        key: String,
    },
    Balance {
        address: Addr,
        key: String,
    },
    TransferHistory {
        address: Addr,
        key: String,
        page: Option<u32>,
        page_size: u32,
    },
    TransactionHistory {
        address: Addr,
        key: String,
        page: Option<u32>,
        page_size: u32,
    },
    Minters {},
    WithPermit {
        permit: QueryPermit,
        query: QueryWithPermit,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryWithPermit {
    Allowance {
        owner: Addr,
        spender: Addr,
    },
    Balance {},
    TransferHistory {
        page: Option<u32>,
        page_size: u32,
    },
    TransactionHistory {
        page: Option<u32>,
        page_size: u32,
    },
}

#[cw_serde]
pub enum QueryAnswer {
    TokenInfo {
        name: String,
        symbol: String,
        decimals: u8,
        total_supply: Option<Uint128>,
    },
    TokenConfig {
        // TODO: add other config items as optionals so they can be ignored in other snip20s
        public_total_supply: bool,
        deposit_enabled: bool,
        redeem_enabled: bool,
        mint_enabled: bool,
        burn_enabled: bool,
        transfer_enabled: bool,
    },
    ContractStatus {
        status: ContractStatusLevel,
    },
    ExchangeRate {
        rate: Uint128,
        denom: String,
    },
    Allowance {
        spender: Addr,
        owner: Addr,
        allowance: Uint128,
        expiration: Option<u64>,
    },
    Balance {
        amount: Uint128,
    },
    TransferHistory {
        txs: Vec<Tx>,
        total: Option<u64>,
    },
    TransactionHistory {
        txs: Vec<RichTx>,
        total: Option<u64>,
    },
    ViewingKeyError {
        msg: String,
    },
    Minters {
        minters: Vec<Addr>,
    },
}