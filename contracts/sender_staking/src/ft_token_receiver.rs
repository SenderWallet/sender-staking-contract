use crate::{
    env, log, near_bindgen, serde_json, AccountId, Contract, ContractExt,  
    PromiseOrValue, U128,
};

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
pub enum TransferCallInfo {
   StakeInfo{ staking_type: String, duration: Option<u32> },
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> PromiseOrValue<U128> {
        assert_eq!(env::predecessor_account_id(), self.token_account_id, "unsupported token");
        let info: TransferCallInfo = serde_json::from_str::<TransferCallInfo>(&msg).expect("invalid msg");
        let mut refund = 0;
        match info {
            TransferCallInfo::StakeInfo{staking_type, duration:_} => {
                if staking_type == "current_deposit".to_string() && self.current_switch == true {
                    self.stake_current(sender_id, amount.0);
                }
                else{
                    refund = amount.0;
                    log!("unsupported staking type");
                }
            },

        }

        PromiseOrValue::Value(refund.into())
    }
}