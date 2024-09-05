//-----------------------------
//contract main state migration
//-----------------------------

use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
//---------------------------------------------------
//  PREVIOUS Main Contract State for state migrations
//---------------------------------------------------
#[derive(BorshDeserialize, BorshSerialize)]
struct OldState {
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

    // current
    pub current_withdraw_delay: u32,
    pub acc_current_staked_amount: Balance,
    pub total_current_staked_amount: Balance,

    pub total_current_unstaked_amount: Balance,
    pub total_current_unstaked_interest: Balance,

    // fixed
    pub acc_fixed_staked_amount: Balance,
    pub total_fixed_staked_amount: Balance,
    pub total_fixed_unstaked_amount: Balance,
    pub total_fixed_unstaked_interest: Balance,
}

#[near_bindgen]
impl Contract {
    //-----------------
    //-- migration called after code upgrade
    ///  For next version upgrades, change this function.
    //-- executed after upgrade to NEW CODE
    //-----------------
    /// This fn WILL be called by this contract from `pub fn upgrade` (started from DAO)
    /// Originally a **NOOP implementation. KEEP IT if you haven't changed contract state.**
    /// If you have changed state, you need to implement migration from old state (keep the old struct with different name to deserialize it first).
    ///
    #[init(ignore_state)] //do not auto-load state before this function
    #[private]
    pub fn migrate() -> Self {
        // read state with OLD struct
        // uncomment when state migration is required on upgrade
        let old: OldState = env::state_read().expect("Old state doesn't exist");

        // can only be called by this same contract (it's called from fn upgrade())
        assert_eq!(
            &env::predecessor_account_id(),
            &env::current_account_id(),
            "Can only be called by this contract"
        );

        // uncomment when state migration is required on upgrade
        // Create the new contract state using the data from the old contract state.
        // returns this struct that gets stored as contract state
        return Self {
            // owner
            owner_id: old.owner_id,
            // token account id
            token_account_id: old.token_account_id,
            // users
            users: old.users,

            current_switch: old.current_switch,
            current_term_apr: old.current_term_apr,
            fixed_switch: old.fixed_switch,
            fixed_term_apr: old.fixed_term_apr,
            
            current_withdraw_delay: old.current_withdraw_delay,
            acc_current_staked_amount: old.acc_current_staked_amount,
            total_current_staked_amount: old.total_current_staked_amount,
            total_current_unstaked_amount: old.total_current_unstaked_amount,
            total_current_unstaked_interest: 0,

            acc_fixed_staked_amount: old.acc_fixed_staked_amount,
            total_fixed_staked_amount: old.total_fixed_staked_amount,
            total_fixed_unstaked_amount: old.total_fixed_unstaked_amount,
            total_fixed_unstaked_interest: 0,
        };
    }
}