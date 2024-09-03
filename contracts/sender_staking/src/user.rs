use crate::*;


#[derive(BorshSerialize, BorshDeserialize, Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CurrentDepositTerm {
    /// A copy of an user ID. 
    #[serde(with = "u128_dec_format")]
    pub amount: Balance, // Current deposit amount
    pub last_stake_time: u64,
    pub last_unstake_time: u64,
    #[serde(with = "u128_dec_format")]
    pub accrued_interest: Balance,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct FixedDepositTerm {
    /// A copy of an user ID.
    #[serde(with = "u128_dec_format")] 
    pub amount: Balance,
    pub start_time: u64,
    pub duration: u64, // Deposit term in seconds
    #[serde(with = "u128_dec_format")]
    pub accrued_interest: Balance,
}

/*
 * User structure 
 * suppport multiple current deposit and fixed deposit
 */
#[derive(BorshSerialize, BorshDeserialize, Serialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct User {
    /// A copy of an user ID. 
    #[serde(with = "u128_dec_format")]
    pub withdrawable_amount: Balance, //
    pub current_deposit: CurrentDepositTerm,
    pub fixed_deposits: Vec<FixedDepositTerm>, // Support multiple fixed deposits.
}

impl User {
    pub fn new( ) -> Self {
        Self {
            withdrawable_amount: 0,
            current_deposit: CurrentDepositTerm {
                amount: 0,
                last_stake_time: 0,
                last_unstake_time: 0,
                accrued_interest: 0,
            },
            fixed_deposits: Vec::new(),
        }
    }
}

impl Contract {
   pub fn internal_get_user(&self, user_id: &AccountId) -> Option<User> {
       self.users.get(user_id).map(|o| o.into())
   }

   pub fn internal_unwrap_user_or_default(&self, user_id: &AccountId) -> User {
       self.users.get(&user_id).unwrap_or( User::new() )
   }

   pub fn internal_set_user(&mut self, user_id: &AccountId, user: User) {
       self.users.insert(user_id, &user.into());
   }
}