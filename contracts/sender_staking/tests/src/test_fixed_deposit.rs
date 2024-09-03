use std::collections::HashMap;
use std::convert::TryInto;

use near_gas::NearGas;
use near_sdk::json_types::U128;
use near_units::parse_near;
use near_workspaces::network::Sandbox;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::types::NearToken;
use near_workspaces::{Account, AccountId, Contract, Worker};
use near_workspaces::{BlockHeight, DevNetwork};
use serde::{Deserialize, Serialize};
use near_contract_standards::fungible_token::Balance;
//

use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CurrentDepositTerm {
    /// A copy of an user ID. 
    pub amount: U128, // Current deposit amount
    pub last_stake_time: u64,
    pub last_unstake_time: u64,
    pub accrued_interest: U128,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct FixedDepositTerm {
    /// A copy of an user ID.
    pub amount: U128,
    pub start_time: u64,
    pub duration: u64, // deposit term in secs
    pub accrued_interest: U128,
}

/*
 * User structure 
 * suppport multiple current deposit and fixed deposit
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct User {
    /// A copy of an user ID. 
    pub withdrawable_amount: U128, //
    pub current_deposit: CurrentDepositTerm,
    pub fixed_deposits: Vec<FixedDepositTerm>, // support multiple fixed deposits
}

const FT_CONTRACT_FILEPATH: &str = "/home/zonquan/NearProject/sender-staking-contract/res/mock_ft.wasm";
const STAKING_CONTRACT_FILEPATH: &str = "/home/zonquan/NearProject/sender-staking-contract/res/sender_staking.wasm";


/// BlockId referencing back to a specific time just in case the contract has
/// changed or has been updated at a later time.
const BLOCK_HEIGHT: BlockHeight = 50_000_000;

pub fn get_total_gas(r: &ExecutionFinalResult) -> u64 {
    r.outcomes().iter().map(|x| x.gas_burnt.as_gas()).sum()
}

pub fn get_total_near_burnt(r: &ExecutionFinalResult) -> u128 {
    r.outcomes()
        .iter()
        .map(|x| x.tokens_burnt.as_yoctonear())
        .sum()
}


/// Create our own custom Fungible Token contract and setup the initial state.
async fn create_custom_ft(
    owner: &Account,
    worker: &Worker<impl DevNetwork>,
) -> anyhow::Result<Contract> {
    println!("Creating custom fungible token...");
    let ft: Contract = worker
        .dev_deploy(&std::fs::read(FT_CONTRACT_FILEPATH)?)
        .await?;

    ft.call("new")
        .args_json(json!({
            "name": "mytoken".to_string(),
            "symbol": "NEAR".to_string(),
            "decimals":24
        }))
        .transact()
        .await?
        .into_result()?;

    Ok(ft)
}

/// Create our own custom contract and setup the initial state.
async fn create_sender_staking(
    owner: &Account,
    worker: &Worker<impl DevNetwork>,
    ft: &Contract,
) -> anyhow::Result<Contract> {
    let contract: Contract = worker
        .dev_deploy(&std::fs::read(STAKING_CONTRACT_FILEPATH)?)
        .await?;

        contract.call("new")
        .args_json(json!({
            "owner_id": owner.id(),
            "token_account_id":ft.id()
        }))
        .transact()
        .await?
        .into_result()?;

    Ok(contract)
}

async fn mint_ft(owner: &Account, user: &Account, contract: &Contract) -> anyhow::Result<()> {
    let mint_amount = U128::from(parse_near!("1,000 N"));

    // register user and mint ft
    let r = user
        .call(contract.id(), "mint")
        .args_json(serde_json::json!({
            "account_id": user.id(),
            "amount":mint_amount
        }))
        .transact()
        .await?
        .into_result()?;

    //println!("result = {:#?}", r);

    let user_balance: U128 = owner
        .call(contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": user.id()
        }))
        .transact()
        .await?
        .json()?;

    assert_eq!(user_balance, mint_amount);
    println!("{:?} balance = {:#?}", user.id(), user_balance);

    println!("      Passed ✅ mint_ft");
    Ok(())
}

/*
async fn test_simple_transfer(
    owner: &Account,
    user: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let transfer_amount = U128::from(parse_near!("1,000 N"));

    // register user
    user.call(contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": user.id()
        }))
        .deposit(NearToken::from_yoctonear(parse_near!("0.008 N")))
        .transact()
        .await?
        .into_result()?;

    // transfer ft
    owner.call(contract.id(), "ft_transfer")
        .args_json(serde_json::json!({
            "receiver_id": user.id(),
            "amount": transfer_amount
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?
        .into_result()?;

    let root_balance: U128 = owner
        .call(contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": owner.id()
        }))
        .transact()
        .await?
        .json()?;

    let alice_balance: U128 = owner
        .call(contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": user.id()
        }))
        .transact()
        .await?
        .json()?;

    //assert_eq!(root_balance, U128::from(parse_near!("999,999,000 N")));
    assert_eq!(alice_balance, transfer_amount);

    println!("      Passed ✅ test_simple_transfer");
    Ok(())
}
*/
async fn turn_on_switch(
    owner: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    // owner call set_switch
    owner
        .call(contract.id(), "set_current_switch")
        .args_json(serde_json::json!({
            "switch": true
        }))
        .transact()
        .await?
        .into_result()?;

    let metadata: Metadata = worker
        .view(contract.id(), "get_metadata")
        .args_json(json!({}))
        .await?
        .json()?;
    println!("metadata: {:#?}", metadata);
    assert_eq!(metadata.current_switch, true);

    println!("      Passed ✅ turn_on_switch\n");
    Ok(())
}

async fn turn_off_switch(
    owner: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    // owner call set_switch
    owner
        .call(contract.id(), "set_current_switch")
        .args_json(serde_json::json!({
            "switch": false
        }))
        .transact()
        .await?
        .into_result()?;

    let metadata: Metadata = worker
        .view(contract.id(), "get_metadata")
        .args_json(json!({}))
        .await?
        .json()?;
    //println!("metadata: {:#?}", metadata);
    assert_eq!(metadata.current_switch, false);

    println!("      Passed ✅ turn_off_switch\n");
    Ok(())
}

async fn test_fixed_deposit(
    owner: &Account,
    user: &Account,
    contract: &Contract,
    ft_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    println!("test_fixed_deposit");
    // owner call set_switch
    owner
        .call(contract.id(), "set_current_switch")
        .args_json(serde_json::json!({
            "switch": false
        }))
        .transact()
        .await?
        .into_result()?;

    let metadata: Metadata = worker
        .view(contract.id(), "get_metadata")
        .args_json(json!({}))
        .await?
        .json()?;
    //println!("metadata: {:#?}", metadata);
    assert_eq!(metadata.current_switch, false);

    // user call ft_transfer_call
    let msg: String = String::from("{\"staking_type\": \"current_deposit\"}");
    let result: ExecutionFinalResult = user
        .call(ft_contract.id(), "ft_transfer_call")
        .args_json(serde_json::json!({
            "receiver_id":contract.id(),
            "amount":"10000000000000000000",
            "msg":msg.clone()
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(50))
        .transact()
        .await?;
    //println!("result = {:#?}", result);

    // will get refund when switch is off
    let metadata: Metadata = worker
        .view(contract.id(), "get_metadata")
        .args_json(json!({}))
        .await?
        .json()?;
    assert_eq!(metadata.total_current_staked_amount, U128(0));

    assert_eq!(true, result.is_success());
    assert_eq!(false, result.is_failure());

    // owner call set_switch
    owner
        .call(contract.id(), "set_current_switch")
        .args_json(serde_json::json!({
            "switch": true
        }))
        .transact()
        .await?
        .into_result()?;

    let metadata: Metadata = worker
        .view(contract.id(), "get_metadata")
        .args_json(json!({}))
        .await?
        .json()?;
    //println!("metadata: {:#?}", metadata);
    assert_eq!(metadata.current_switch, true);

    // user call ft_transfer_call
    let result: ExecutionFinalResult = user
        .call(ft_contract.id(), "ft_transfer_call")
        .args_json(serde_json::json!({
            "receiver_id":contract.id(),
            "amount":"10000000000000000000",
            "msg":msg.clone()
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(50))
        .transact()
        .await?;
    //println!("result = {:#?}", result);

    let metadata: Metadata = worker
        .view(contract.id(), "get_metadata")
        .args_json(json!({}))
        .await?
        .json()?;
    assert_eq!(metadata.total_current_staked_amount, U128(10000000000000000000));

    let user_info: User = worker
        .view(contract.id(), "get_user")
        .args_json(json!({"user_id":user.id()}))
        .await?
        .json()?;
    assert_eq!(user_info.current_deposit.amount, U128(10000000000000000000));

    assert_eq!(true, result.is_success());
    assert_eq!(false, result.is_failure());

    // test unstake
    // move forward 100
    let blocks_to_advance = 100;
    println!("--------Fast forward = {} blocks--------", blocks_to_advance);
    worker.fast_forward(blocks_to_advance).await?;
    println!("--------Fast forward = {} blocks--------done\n", blocks_to_advance);

    let user_info: User = worker
        .view(contract.id(), "get_user")
        .args_json(json!({"user_id":user.id()}))
        .await?
        .json()?;
    println!("before unstake, user = {:#?}", user_info);

    // user call unstake_current
    let result: ExecutionFinalResult = user
        .call(contract.id(), "unstake_current")
        .args_json(serde_json::json!({}))
        .transact()
        .await?;
    //println!("result = {:#?}", result);
    let user_info: User = worker
        .view(contract.id(), "get_user")
        .args_json(json!({"user_id":user.id()}))
        .await?
        .json()?;
    println!("after unstake, user = {:#?}", user_info);
    
    // withdraw. To test withdraw, need to comment out the following line
    // require!(timestamp > (user.current_deposit.last_unstake_time + DEFAULT_WITHDRAW_DAYS * ONE_DAY_IN_SECS),msg);
    // in lib.rs
    // user call withdraw
    let result: ExecutionFinalResult = user
        .call(contract.id(), "withdraw")
        .args_json(serde_json::json!({}))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(60))
        .transact()
        .await?;
    println!("result = {:#?}", result);
    assert_eq!(true, result.is_success());
    assert_eq!(false, result.is_failure());


    let user_info: User = worker
        .view(contract.id(), "get_user")
        .args_json(json!({"user_id":user.id()}))
        .await?
        .json()?;
    println!("after withdraw, user = {:#?}", user_info);

    println!("check ft_balance of");
    let user_balance: U128 = user
        .call(ft_contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": user.id()
        }))
        .transact()
        .await?
        .json()?;

    println!("{:?} balance = {:#?}", user.id(), user_balance);
    
    println!("      Passed ✅ test_fixed_deposit\n");
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let owner = worker.root_account()?;
    println!("Using account {}", owner.id());

    // create accounts
    let owner = worker.root_account().unwrap();
    let alice = owner
        .create_subaccount("alice")
        .initial_balance(NearToken::from_yoctonear(parse_near!("30 N")))
        .transact()
        .await?
        .into_result()?;
    let bob = owner
        .create_subaccount("bob")
        .initial_balance(NearToken::from_yoctonear(parse_near!("30 N")))
        .transact()
        .await?
        .into_result()?;

    let bal: anyhow::Result<u128> = worker
        .view_account(&alice.id())
        .await
        .map(|d| d.balance.as_yoctonear())
        .map_err(Into::into);
    println!("alice bal = {:#?}", bal);

    let bal: anyhow::Result<u128> = worker
        .view_account(&bob.id())
        .await
        .map(|d| d.balance.as_yoctonear())
        .map_err(Into::into);
    println!("bob bal = {:#?}", bal);

    // create contracts
    let ft_contract = create_custom_ft(&owner, &worker).await?;
    let staking_contract = create_sender_staking(&owner, &worker, &ft_contract).await?;
    mint_ft(&owner, &alice, &ft_contract).await?;
    mint_ft(&owner, &bob, &ft_contract).await?;
    mint_ft(&owner, &staking_contract.as_account(), &ft_contract).await?;

    // Get metadata
    let metadata: Metadata = worker
        .view(staking_contract.id(), "get_metadata")
        .args_json(json!({}))
        .await?
        .json()?;
    println!("metadata: {:#?}", metadata);

    // start to do test
    turn_off_switch(&owner, &staking_contract, &worker).await?;
    turn_on_switch(&owner, &staking_contract, &worker).await?;
    test_fixed_deposit(&owner, &alice, &staking_contract, &ft_contract, &worker).await?;

    Ok(())
}
