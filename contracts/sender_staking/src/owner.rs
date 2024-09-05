use crate::*;

impl Contract {
    pub fn assert_owner(&self) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            "ERR_NOT_ALLOWED"
        );
    }

}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn set_owner(&mut self, owner_id: AccountId) {
        assert_one_yocto();
        self.assert_owner();
        self.owner_id = owner_id;
    }

    pub fn set_current_switch(&mut self, switch: bool) {
        self.assert_owner();
        self.current_switch = switch;
    }

    pub fn set_fixed_switch(&mut self, switch: bool) {
        self.assert_owner();
        self.fixed_switch = switch;
    }

    pub fn set_current_apr(&mut self, apr: u32) {
        self.assert_owner();
        require!(apr > 0, "apr must be positive");
        self.current_term_apr = apr;
    }

    pub fn set_current_withdraw_delay(&mut self, delay_in_days: u32) {
        self.assert_owner();
        require!(delay_in_days > 0, "delay_in_days must be positive");
        self.current_withdraw_delay = delay_in_days;
    }

}


#[cfg(target_arch = "wasm32")]
mod upgrade {
    use near_sdk::Gas;
    //use near_gas::*;
    use near_sys as sys;

    use super::*;

    /// Gas for calling migration call.
    //pub const GAS_FOR_MIGRATE_CALL: Gas = Gas(5_000_000_000_000);
    pub const GAS_FOR_MIGRATE_CALL: NearGas = NearGas::from_tgas(5);

    /// Self upgrade and call migrate, optimizes gas by not loading into memory the code.
    /// Takes as input non serialized set of bytes of the code.
    #[no_mangle]
    pub fn upgrade() {
        env::setup_panic_hook();
        let contract: Contract = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
        contract.assert_owner();
        let current_id = env::current_account_id().as_bytes().to_vec();
        let method_name = "migrate".as_bytes().to_vec();
        unsafe {
            // Load input (wasm code) into register 0.
            sys::input(0);
            // Create batch action promise for the current contract ID
            let promise_id =
                sys::promise_batch_create(current_id.len() as _, current_id.as_ptr() as _);
            // 1st action in the Tx: "deploy contract" (code is taken from register 0)
            sys::promise_batch_action_deploy_contract(promise_id, u64::MAX as _, 0);
            // 2nd action in the Tx: call this_contract.migrate() with remaining gas
            let attached_gas = env::prepaid_gas().as_gas() - env::used_gas().as_gas() - GAS_FOR_MIGRATE_CALL.as_gas();
            sys::promise_batch_action_function_call(
                promise_id,
                method_name.len() as _,
                method_name.as_ptr() as _,
                0 as _,
                0 as _,
                0 as _,
                attached_gas,
            );
        }
    }
}