use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet, LookupSet};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_one_yocto, env, ext_contract, log, near_bindgen, require, serde_json, AccountId, NearToken, Gas,
    BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue, PromiseResult
};
use near_contract_standards::fungible_token::Balance;
use near_contract_standards::fungible_token::core::ext_ft_core;
use std::collections::HashMap;
use serde_json::json;

use near_gas::NearGas;

mod errors;
mod events;
mod owner;
mod user;
mod utils;
mod view;
mod ft_token_receiver;

pub use crate::errors::*;
pub use crate::events::*;
pub use crate::user::*;
pub use crate::utils::*;
pub use crate::ft_token_receiver::*;

pub const STORAGE_DEPOSIT_AMOUNT: Balance = 1250000000000000000000;
pub const ONE_YOCTO_NEAR: Balance = 1;

pub const TGAS: u64 = 1_000_000_000_000;
pub const GAS_FOR_DEPOSIT: NearGas = NearGas::from_tgas(15);
pub const GAS_FOR_TRANSFER: NearGas = NearGas::from_tgas(30);
pub const GAS_FOR_TRANSFER_ON_CALL: NearGas = NearGas::from_tgas(45);

pub const YOCTO8: u128 = 100_000_000;
pub const YOCTO18: u128 = 1_000_000_000_000_000_000;
pub const YOCTO24: u128 = 1_000_000_000_000_000_000_000_000;

pub const ONE_DAY_IN_SECS: u64 = 24 * 60 * 60; // NOTICE: RESTORE 
pub const DEFAULT_DURATION_SEC: u64 = 60 * 60;
pub const MAX_DURATION_SEC: u64 = 72 * 60 * 60;

pub const TERM_APR_DEMONINATOR: u32 = 10000;
pub const DEFAULT_FIXED_TERM_APR: u32 = 6000; 
pub const DEFAULT_CURRENT_TERM_APR: u32 = 3200;
pub const DEFAULT_WITHDRAW_DAYS: u32 = 21;


pub type TimeStampSec = u64;


// external contract interface for callback
#[ext_contract(ext_self)]
pub trait MyContract {
    fn on_transfer_complete(&mut self, receiver_id: AccountId, amount: u128, time: u64);
}

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    User,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    // owner
    pub owner_id: AccountId,
    // token account id
    pub token_account_id: AccountId,
    // users
    pub users: UnorderedMap<AccountId, User>,

    pub current_switch: bool,
    pub current_term_apr: u32,
    pub fixed_switch: bool,
    pub fixed_term_apr: u32,

    pub total_current_staked_amount: Balance,
    pub total_fixed_staked_amount: Balance,
    pub current_withdraw_delay: u32,
}


#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, token_account_id: AccountId) -> Self {
        require!(!env::state_exists(), "Already initialized");
        Contract {
            owner_id,
            token_account_id,
            users: UnorderedMap::new(StorageKey::User),
            current_switch: true,
            current_term_apr: DEFAULT_CURRENT_TERM_APR,
            fixed_switch: true,
            fixed_term_apr: DEFAULT_FIXED_TERM_APR,
            total_current_staked_amount: 0,
            total_fixed_staked_amount: 0,
            current_withdraw_delay: DEFAULT_WITHDRAW_DAYS,
        }
    }

    /* ========== CORE FUNCTION ========== */
    pub fn unstake_current(&mut self) {
        let predecessor_id = env::predecessor_account_id();
        let mut user: User = self.internal_unwrap_user_or_default(&predecessor_id);
        require!(user.current_deposit.amount > 0, "No current deposit to unstake" );
        let timestamp = nano_to_sec(env::block_timestamp());
        let delta_time = timestamp - user.current_deposit.last_stake_time;
        let interest = (user.current_deposit.amount*delta_time as u128 *(self.current_term_apr as u128)/(TERM_APR_DEMONINATOR as u128))/(365*ONE_DAY_IN_SECS) as u128;

        let unstake_amount = user.current_deposit.amount;
        user.withdrawable_amount += user.current_deposit.amount;
        user.withdrawable_amount += user.current_deposit.accrued_interest + interest;

        user.current_deposit.amount = 0;
        user.current_deposit.accrued_interest = 0;
        user.current_deposit.last_unstake_time = timestamp;

        require!(self.total_current_staked_amount >= unstake_amount,"Unstake amount is greater than total_current_staked_amount" );
        self.total_current_staked_amount -= unstake_amount;
        self.internal_set_user(&predecessor_id,user);

        Event::Unstake { 
            user_id: &predecessor_id.clone(), 
            unstake_type: &"current_deposit".to_string(),
            amount: &U128(unstake_amount),
            time: timestamp
        }.emit();
    }

    pub fn unstake_fixed(&mut self) {
        let predecessor_id = env::predecessor_account_id();
        let mut user: User = self.internal_unwrap_user_or_default(&predecessor_id);
        require!(user.current_deposit.amount > 0, "No current deposit to unstake" );
        let timestamp = nano_to_sec(env::block_timestamp());
        let delta_time = timestamp - user.current_deposit.last_stake_time;

    }

    #[payable]
    pub fn withdraw(&mut self) -> Promise {
        let predecessor_id = env::predecessor_account_id();
        let user: User = self.internal_unwrap_user_or_default(&predecessor_id);
        let timestamp = nano_to_sec(env::block_timestamp());
        
        require!(user.withdrawable_amount > 0, "The withdrawable amount is zero" );
        require!(user.current_deposit.last_unstake_time > 0, "need to unstake" );
        let msg = format!("need to wait for {} days", self.current_withdraw_delay);
        require!(timestamp > (user.current_deposit.last_unstake_time + ONE_DAY_IN_SECS * self.current_withdraw_delay as u64), msg);

        let transfer_promise = Promise::new(self.token_account_id.clone()).function_call(
            "ft_transfer".to_string(),
            json!({
                "receiver_id": predecessor_id.clone(),
                "amount": U128(user.withdrawable_amount),
            }).to_string().into_bytes(),
            NearToken::from_yoctonear(ONE_YOCTO_NEAR),
            Gas::from_gas(GAS_FOR_TRANSFER.as_gas())
        );

        return transfer_promise.then(
            Self::ext(env::current_account_id())
            .with_static_gas(Gas::from_tgas(20))
            .on_transfer_complete(predecessor_id.clone(), U128(user.withdrawable_amount), timestamp)
        );

    }  

    #[private]
    pub fn on_transfer_complete(&mut self, receiver_id: AccountId, amount: U128, timestamp: u64) {
        // check the result of promise
        match env::promise_result(0) {
            PromiseResult::Failed => env::panic_str("Transfer failed."),
            PromiseResult::Successful(_result) => {
                let mut user: User = self.internal_unwrap_user_or_default(&receiver_id);
                user.withdrawable_amount = 0;
                user.current_deposit.last_unstake_time = 0; // reset last_unstake_time to 0 after withdraw
                self.internal_set_user(&receiver_id, user);
        
                Event::Withdraw { 
                    user_id: &receiver_id.clone(), 
                    amount: &amount,
                    time: timestamp
                }.emit(); 
            }
        }
    }
}

impl Contract{
    pub fn stake_current(&mut self, sender_id: AccountId, amount: Balance) {
        let mut user: User = self.internal_unwrap_user_or_default(&sender_id);

        let timestamp = nano_to_sec(env::block_timestamp());
        //log!("timestamp = {:#?}", timestamp);
        let delta_time = timestamp - user.current_deposit.last_stake_time;
        let interest = (user.current_deposit.amount*delta_time as u128 *(self.current_term_apr as u128)/(TERM_APR_DEMONINATOR as u128))/(365*ONE_DAY_IN_SECS) as u128;
        // update accrued interest
        user.current_deposit.accrued_interest += interest;
        // update last_stake_time
        user.current_deposit.last_stake_time = timestamp;
        // update stake amount
        user.current_deposit.amount += amount;
        // update total_current_staked_amount
        self.total_current_staked_amount += amount;

        self.internal_set_user(&sender_id,user);
        Event::Stake { 
            user_id: &sender_id.clone(), 
            stake_type: &"current_deposit".to_string(),
            amount: &U128(amount),
            duration: 0,
            time: timestamp
        }.emit(); 
    }

    pub fn stake_fixed(&mut self, sender_id: AccountId, amount: Balance, duration: u32) {
        let mut user: User = self.internal_unwrap_user_or_default(&sender_id);

        let timestamp = nano_to_sec(env::block_timestamp());
        //log!("timestamp = {:#?}", timestamp);

        Event::Stake { 
            user_id: &sender_id.clone(), 
            stake_type: &"fixed_deposit".to_string(),
            amount: &U128(amount),
            duration: duration,
            time: timestamp
        }.emit(); 
 
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env};
    use near_contract_standards::fungible_token::Balance;

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

}
