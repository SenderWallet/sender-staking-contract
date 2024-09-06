use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::{
    FungibleToken, FungibleTokenCore, FungibleTokenResolver,
};
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::borsh::BorshSerialize;
use near_sdk::json_types::U128;
use near_sdk::{
    env, log, near, require, AccountId, BorshStorageKey, NearToken, PanicOnDefault, PromiseOrValue,
};

#[derive(PanicOnDefault)]
#[near(contract_state)]
pub struct Contract {
    token: FungibleToken,
    name: String,
    symbol: String,
    icon: Option<String>,
    decimals: u8,
}


#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
enum StorageKey {
    FungibleToken,
}

#[near]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(name: String, symbol: String, decimals: u8) -> Self {
        require!(!env::state_exists(), "Already initialized");
        Self {
            token: FungibleToken::new(StorageKey::FungibleToken),
            name,
            symbol,
            icon: None,
            decimals,
        }
    }

    pub fn mint(&mut self, account_id: AccountId, amount: U128) {
        if self.token.storage_balance_of(account_id.clone()).is_none() {
            self.token.internal_register_account(&account_id);
        }
        self.token.internal_deposit(&account_id, amount.into());
    }

    
    pub fn burn(&mut self, account_id: AccountId, amount: U128) {
        self.token.internal_withdraw(&account_id, amount.into());
    }

    #[private]
    pub fn set_token_name(&mut self, name: String, symbol: String) {
        self.name = name;
        self.symbol = symbol;
    }

    #[private]
    pub fn set_icon(&mut self, icon: String) {
        self.icon = Some(icon);
    }

    #[private]
    pub fn set_decimals(&mut self, dec: u8) {
        self.decimals = dec;
    }
}

#[near]
impl FungibleTokenCore for Contract {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        self.token.ft_transfer(receiver_id, amount, memo)
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.token.ft_transfer_call(receiver_id, amount, memo, msg)
    }

    fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.token.ft_balance_of(account_id)
    }
}

#[near]
impl FungibleTokenResolver for Contract {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        let (used_amount, burned_amount) =
            self.token.internal_ft_resolve_transfer(&sender_id, receiver_id, amount);
        if burned_amount > 0 {
            log!("Account @{} burned {}", sender_id, burned_amount);
        }
        used_amount.into()
    }
}

#[near]
impl StorageManagement for Contract {
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        self.token.storage_deposit(account_id, registration_only)
    }

    #[payable]
    fn storage_withdraw(&mut self, amount: Option<NearToken>) -> StorageBalance {
        self.token.storage_withdraw(amount)
    }

    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        #[allow(unused_variables)]
        if let Some((account_id, balance)) = self.token.internal_storage_unregister(force) {
            log!("Closed @{} with {}", account_id, balance);
            true
        } else {
            false
        }
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        self.token.storage_balance_bounds()
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.token.storage_balance_of(account_id)
    }
}

#[near]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            icon: None,
            reference: None,
            reference_hash: None,
            decimals: self.decimals,
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_contract_standards::fungible_token::Balance;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

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
    fn test_basics() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.build());
        let mut contract = Contract::new(String::from("TBD"), String::from("TBD"), 24);
        testing_env!(context
            .attached_deposit(env::storage_byte_cost().checked_mul(125).unwrap())
            .build());
        contract.mint(accounts(0), 1_000_000.into());
        assert_eq!(contract.ft_balance_of(accounts(0)), 1_000_000.into());

        testing_env!(context
            .attached_deposit(env::storage_byte_cost().checked_mul(125).unwrap())
            .build());
        contract.storage_deposit(Some(accounts(1)), None);
        testing_env!(context
            .attached_deposit(NearToken::from_yoctonear(1))
            .predecessor_account_id(accounts(0))
            .build());
        contract.ft_transfer(accounts(1), 1_000.into(), None);
        assert_eq!(contract.ft_balance_of(accounts(1)), 1_000.into());

        contract.burn(accounts(1), 500.into());
        assert_eq!(contract.ft_balance_of(accounts(1)), 500.into());
    }
}