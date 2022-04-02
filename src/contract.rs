#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use anchor_token::staking::{ ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, StakerInfoResponse, StateResponse,};
use crate::{
//     querier::query_anc_minter,
    state::{
        read_config, read_staker_info, remove_staker_info, store_config,
        store_staker_info,  Config, StakerInfo,
    },
};

use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use std::collections::BTreeMap;

// use cw2::set_contract_version;

// use crate::error::ContractError;
// use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
// use crate::state::{State, STATE};

// // version info for migration info
// const CONTRACT_NAME: &str = "crates.io:{{project-name}}";
// const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    store_config(
        deps.storage,
        &Config {
            staking_token: deps.api.addr_canonicalize(&msg.staking_token)?,
        },
    )?;
    
    //start here
//     set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
//     STATE.save(deps.storage, &state)?;
    

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("end", msg.end.to_string()))
}


pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Bond {}) => {
            // only staking token contract can execute this message
            if config.staking_token != deps.api.addr_canonicalize(info.sender.as_str())? {
                return Err(StdError::generic_err("unauthorized"));
            }

            let cw20_sender = deps.api.addr_validate(&cw20_msg.sender)?;
            bond(deps, env, cw20_sender, cw20_msg.amount)
        }
        Err(_) => Err(StdError::generic_err("data should be given")),
    }
}

pub fn bond(deps: DepsMut, env: Env, sender_addr: Addr, amount: Uint128) -> StdResult<Response> {
    let sender_addr_raw: CanonicalAddr = deps.api.addr_canonicalize(sender_addr.as_str())?;

    let config: Config = read_config(deps.storage)?;
//     let mut state: State = read_state(deps.storage)?;
    let mut staker_info: StakerInfo = read_staker_info(deps.storage, &sender_addr_raw)?;

    // Compute global reward & staker reward
//     compute_reward(&config, &mut state, env.block.time.seconds());
//     compute_staker_reward(&state, &mut staker_info)?;

    // Increase bond_amount
    increase_bond_amount(&mut staker_info, amount);

    // Store updated state with staker's staker_info
    store_staker_info(deps.storage, &sender_addr_raw, &staker_info)?;
//     store_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "bond"),
        ("owner", sender_addr.as_str()),
        ("amount", amount.to_string().as_str()),
    ]))
}


fn increase_bond_amount( staker_info: &mut StakerInfo, amount: Uint128) {
//     state.total_bond_amount += amount;
    staker_info.bond_amount += amount;
}

// withdraw rewards to executor
pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let sender_addr_raw = deps.api.addr_canonicalize(info.sender.as_str())?;

    let config: Config = read_config(deps.storage)?;
//     let mut state: State = read_state(deps.storage)?;
    let mut staker_info = read_staker_info(deps.storage, &sender_addr_raw)?;

    // Compute global reward & staker reward
//     compute_reward(&config, &mut state, env.block.time.seconds());
//     compute_staker_reward(&state, &mut staker_info)?;

    let amount = staker_info.bond_amount;
    staker_info.bond_amount = staker_info.pending_reward - amount;

    // Store or remove updated rewards info
    // depends on the left pending reward and bond amount
    if staker_info.bond_amount.is_zero() {
        remove_staker_info(deps.storage, &sender_addr_raw);
    } else {
        store_staker_info(deps.storage, &sender_addr_raw, &staker_info)?;
    }

    // Store updated state
//     store_state(deps.storage, &state)?;

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.staking_token)?.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount,
            })?,
            funds: vec![],
        })])
        .add_attributes(vec![
            ("action", "withdraw"),
            ("owner", info.sender.as_str()),
            ("amount", amount.to_string().as_str()),
        ]))
}
