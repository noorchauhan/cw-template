use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub end: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInfo {
    pub bond-amount: Uint128,
}

pub fn store_staker_info(
    storage: &mut dyn Storage,
    owner: &CanonicalAddr,
    staker_info: &StakerInfo,
) -> StdResult<()> {
    Bucket::new(storage, PREFIX_REWARD).save(owner.as_slice(), staker_info)
}

pub fn remove_staker_info(storage: &mut dyn Storage, owner: &CanonicalAddr) {
    Bucket::<StakerInfo>::new(storage, PREFIX_REWARD).remove(owner.as_slice())
}


pub fn read_staker_info(storage: &dyn Storage, owner: &CanonicalAddr) -> StdResult<StakerInfo> {
    match ReadonlyBucket::new(storage, PREFIX_REWARD).may_load(owner.as_slice())? {
        Some(staker_info) => Ok(staker_info),
        None => Ok(StakerInfo {
            bond_amount: Uint128::zero(),
           
        }),
    }
}


