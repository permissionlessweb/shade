use crate::{handle, query, state::config_w};
use shade_protocol::c_std::{
    debug_print,
    to_binary,
    Api,
    Binary,
    Env,
    DepsMut,
    Response,
    Querier,
    StdResult,
    Storage,
};
use shade_protocol::contract_interfaces::oracles::oracle::{
    HandleMsg,
    InitMsg,
    OracleConfig,
    QueryMsg,
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> StdResult<Response> {
    let state = OracleConfig {
        admin: match msg.admin {
            None => info.sender.clone(),
            Some(admin) => admin,
        },
        band: msg.band,
        sscrt: msg.sscrt,
    };

    config_w(deps.storage).save(&state)?;

    debug_print!("Contract was initialized by {}", info.sender);

    Ok(Response::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    msg: HandleMsg,
) -> StdResult<Response> {
    match msg {
        HandleMsg::UpdateConfig { admin, band } => {
            handle::try_update_config(deps, env, admin, band)
        }
        HandleMsg::RegisterPair { pair } => handle::register_pair(deps, env, pair),
        HandleMsg::UnregisterPair { symbol, pair } => {
            handle::unregister_pair(deps, env, symbol, pair)
        }
        HandleMsg::RegisterIndex { symbol, basket } => {
            handle::register_index(deps, env, symbol, basket)
        }
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: Deps,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Price { symbol } => to_binary(&query::price(deps, symbol)?),
        QueryMsg::Prices { symbols } => to_binary(&query::prices(deps, symbols)?),
    }
}
