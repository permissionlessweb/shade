use cosmwasm_std::{
    debug_print, to_binary, Api, Binary,
    Env, Extern, HandleResponse, InitResponse, 
    Querier, StdResult, Storage, 
};

use shade_protocol::{
    staking_pool::{
        Config,
        InitMsg, 
        HandleMsg,
        QueryMsg,
    },
};

use crate::{
    state::{
        viewing_key_w,
        config_w,
        self_address_w,
    },
    handle, query,
};


pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    let state = Config {
        owner: match msg.admin {
            None => { env.message.sender.clone() }
            Some(admin) => { admin }
        },
        sscrt: msg.sscrt,
        token: msg.token,
    };

    config_w(&mut deps.storage).save(&state)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse {
        messages: vec![],
        log: vec![]
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Receive {
            sender,
            from,
            amount,
            msg,
            ..
        } => handle::receive(deps, env, sender, from, amount, msg),
        HandleMsg::UpdateConfig {
            owner,
        } => handle::try_update_config(deps, env, owner),
        HandleMsg::ClaimRewards {
        } => handle::claim_rewards(deps, env),
        HandleMsg::RefreshStake {
        } => handle::refresh_stake(deps, env),
        // Begin unbonding of a certain amount of scrt
        // Handle unbonding in receive
        /*
        HandleMsg::Unbond {
        } => handle::unbond(deps, env),
        */
        // Collect a completed unbonding
        HandleMsg::Collect {
        } => handle::refresh_stake(deps, env),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query::config(deps)?),
        // All delegations
        QueryMsg::Delegations { },
        QueryMsg::Rewards { },
    }
}
