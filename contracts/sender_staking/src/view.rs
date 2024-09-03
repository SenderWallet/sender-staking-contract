use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(feature = "test", derive(Clone))]
pub struct Metadata {
    pub version: String,
    pub owner_id: AccountId,
    pub token_account_id: AccountId,
    pub current_switch: bool,
    pub current_term_apr: u32,
    pub fixed_switch: bool,
    pub fixed_term_apr: u32,

    pub total_current_staked_amount: U128,
    pub total_fixed_staked_amount: U128,
}

#[near_bindgen]
impl Contract {
    //******** Contract Concern */
    pub fn get_metadata(&self) -> Metadata {
        Metadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            owner_id: self.owner_id.clone(),
            token_account_id: self.token_account_id.clone(),
            current_switch: self.current_switch,
            current_term_apr: self.current_term_apr,
            fixed_switch: self.fixed_switch,
            fixed_term_apr: self.fixed_term_apr,
            total_current_staked_amount: U128(self.total_current_staked_amount),
            total_fixed_staked_amount: U128(self.total_fixed_staked_amount),
        }
    }

    /* ========== VIEW FUNCTION ========== */
    pub fn get_total_user_num(&self) -> u32 {
        let keys = self.users.keys_as_vector();
        keys.len() as u32
    }

    pub fn get_user(&self, user_id: AccountId) -> User {
        let mut user = self.internal_unwrap_user_or_default(&user_id);
        let timestamp = nano_to_sec(env::block_timestamp());
        let delta_time = timestamp - user.current_deposit.last_stake_time;
        let interest = (user.current_deposit.amount*delta_time as u128 *(self.current_term_apr as u128)/(TERM_APR_DEMONINATOR as u128))/(365*ONE_DAY_IN_SECS) as u128;

        user.current_deposit.accrued_interest += interest;
        user
    }

    pub fn get_user_current_deposit(&self, user_id: AccountId) -> U128 {
        let user = self.internal_unwrap_user_or_default(&user_id);
        user.current_deposit.amount.into()
    }    
    pub fn get_user_current_accrued_interest(&self, user_id: AccountId) -> U128 {
        let user = self.internal_unwrap_user_or_default(&user_id);
        let timestamp = nano_to_sec(env::block_timestamp());
        let delta_time = timestamp - user.current_deposit.last_stake_time;
        let interest = (user.current_deposit.amount*delta_time as u128 *(self.current_term_apr as u128)/(TERM_APR_DEMONINATOR as u128))/(365*ONE_DAY_IN_SECS) as u128;
        U128(user.current_deposit.accrued_interest + interest)
    }

    pub fn get_user_withdrawable_time(&self, user_id: AccountId) -> u64 {
        let user = self.internal_unwrap_user_or_default(&user_id);
        if user.current_deposit.last_unstake_time == 0 {
            0
        } else {
            user.current_deposit.last_unstake_time + ONE_DAY_IN_SECS * self.current_withdraw_delay as u64
        }

    }

}
