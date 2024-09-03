use near_sdk::{
    AccountId, log,
    serde::{Serialize},
    serde_json::{json},
    json_types::U128,
};

const EVENT_STANDARD: &str = "sender_staking";
const EVENT_STANDARD_VERSION: &str = "1.0.0";

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Event<'a> {
    Stake{
        user_id: &'a AccountId, 
        stake_type: &'a String,
        amount: &'a U128,
        duration: u32,
        time: u64
    },
    Unstake{
        user_id: &'a AccountId, 
        unstake_type: &'a String,
        amount: &'a U128,
        time: u64
    },
    Withdraw{
        user_id: &'a AccountId,
        amount: &'a U128,
        time: u64
    },
}

impl Event<'_> {
    pub fn emit(&self) {
        emit_event(&self);
    }
}

// Emit event that follows NEP-297 standard: https://nomicon.io/Standards/EventsFormat
// Arguments
// * `standard`: name of standard, e.g. nep171
// * `version`: e.g. 1.0.0
// * `event`: type of the event, e.g. nft_mint
// * `data`: associate event data. Strictly typed for each set {standard, version, event} inside corresponding NEP
pub (crate) fn emit_event<T: ?Sized + Serialize>(data: &T) {
    let result = json!(data);
    let event_json = json!({
        "standard": EVENT_STANDARD,
        "version": EVENT_STANDARD_VERSION,
        "event": result["event"],
        "data": [result["data"]]
    })
    .to_string();
    log!("{}", format!("EVENT_JSON:{}", event_json));
}


#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::{test_utils, AccountId};

    fn alice() -> AccountId {
        AccountId::new_unvalidated("alice".to_string())
    }

    #[test]
    fn stake() {
        let caller_id = &alice();
        let token_id = &"1".to_string();
        let stake_type = &"current_deposit".to_string();
        let amount = &U128(10000000000);
        let duration = 30;
        let time = 100;


        Event::Stake { 
            user_id: caller_id, 
            stake_type: stake_type,
            amount: amount,
            duration: duration,
            time: time
        }.emit();
        assert_eq!(
            test_utils::get_logs()[0],
            r#"EVENT_JSON:{"data":[{"amount":"10000000000","duration":30,"stake_type":"current_deposit","time":100,"user_id":"alice"}],"event":"stake","standard":"sender_staking","version":"1.0.0"}"#
        );
    }
}