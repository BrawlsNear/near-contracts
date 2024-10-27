use near_sdk::near;
use near_workspaces::types::{AccountId, Gas, NearToken};
use serde_json::json;

#[near(serializers = [json])]
#[derive(Clone)]
pub struct Bid {
    pub bidder: AccountId,
    pub bid: NearToken,
}

const TEN_NEAR: NearToken = NearToken::from_near(10);

#[tokio::test]
async fn test_contract_is_operational() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;

    let root = sandbox.root_account()?;

    // Create accounts
    let alice = create_subaccount(&root, "alice").await?;
    let bob = create_subaccount(&root, "bob").await?;
    let dave = create_subaccount(&root, "dave").await?;

    let brawl_creator = create_subaccount(&root, "brawl_creator").await?;

    // Deploy and initialize contract
    let contract_wasm = near_workspaces::compile_project("./").await?;
    let contract = brawl_creator.deploy(&contract_wasm).await?.unwrap();

    let init = contract
        .call("init")
        .args_json(json!({"owner":brawl_creator.id(), "title":"Test Brawl".to_string(), "id":"test_brawl".to_string(), "options":vec!["option-1".to_string(), "option-2".to_string()]}))
        .transact()
        .await?;

    assert!(init.is_success());

    // Check owner
    let owner = contract.view("get_owner").await?;
    let owner_id: AccountId = owner.json::<AccountId>()?;
    assert_eq!(owner_id, brawl_creator.id().clone());

    // Check options
    let options = contract.view("get_options").await?;
    let options_vec: Vec<String> = options.json::<Vec<String>>()?;
    assert_eq!(
        options_vec,
        vec!["option-1".to_string(), "option-2".to_string()]
    );

    // Check finalized
    let finalized = contract.view("get_is_finalized").await?;
    let is_finalized: bool = finalized.json::<bool>()?;
    assert!(!is_finalized);

    // Alice deposits
    let deposit = alice
        .call(contract.id(), "deposit")
        .args_json(json!({"option":"option-1"}))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;

    assert!(deposit.is_success());

    // Check Alice deposit
    let deposits = contract.view("get_total_deposits").await?;
    let alice_deposit: NearToken = deposits.json::<NearToken>()?;
    assert_eq!(alice_deposit, NearToken::from_near(1));

    // Bob deposits
    let deposit = bob
        .call(contract.id(), "deposit")
        .args_json(json!({"option":"option-2"}))
        .deposit(NearToken::from_near(3))
        .transact()
        .await?;

    assert!(deposit.is_success());

    // Check total deposit
    let deposits = contract.view("get_total_deposits").await?;
    let total_deposit: NearToken = deposits.json::<NearToken>()?;
    assert_eq!(total_deposit, NearToken::from_near(4));

    // Dave deposits
    let deposit = dave
        .call(contract.id(), "deposit")
        .args_json(json!({"option":"option-2"}))
        .deposit(NearToken::from_near(3))
        .transact()
        .await?;

    assert!(deposit.is_success());

    // Check total deposit
    let deposits = contract.view("get_total_deposits").await?;
    let total_deposit: NearToken = deposits.json::<NearToken>()?;
    assert_eq!(total_deposit, NearToken::from_near(7));

    // Claim Fail
    let claim = alice.call(contract.id(), "claim").transact().await?;
    assert!(!claim.is_success());

    // Finalize Not Owner
    let finalize = bob
        .call(contract.id(), "update_correct_option")
        .args_json(json!({"option":"option-1"}))
        .transact()
        .await?;
    assert!(!finalize.is_success());

    // Finalize Owner
    let finalize = brawl_creator
        .call(contract.id(), "update_correct_option")
        .args_json(json!({"option":"option-2"}))
        .transact()
        .await?;
    assert!(finalize.is_success());

    // Check correct option
    let correct_option = contract.view("get_correct_option").await?;
    let correct_option_str: String = correct_option.json::<String>()?;
    assert_eq!(correct_option_str, "option-2");

    // Check finalized
    let finalized = contract.view("get_is_finalized").await?;
    let is_finalized: bool = finalized.json::<bool>()?;
    assert!(is_finalized);

    // Claim Alice Lost
    let claim = alice.call(contract.id(), "claim").transact().await?;
    assert!(!claim.is_success());

    // Claim Bob Won
    let bob_balance = bob.view_account().await?.balance;
    let claim = bob.call(contract.id(), "claim").transact().await?;
    assert!(claim.is_success());

    // Check Bob balance
    let bob_new_balance = bob.view_account().await?.balance;
    assert!(bob_new_balance >= bob_balance.saturating_add(NearToken::from_millinear(3490)));

   // Claim Dave Won
   let dave_balance = dave.view_account().await?.balance;
   let claim = dave.call(contract.id(), "claim").transact().await?;
   assert!(claim.is_success());

   // Check Dave balance
   let dave_new_balance = dave.view_account().await?.balance;
   assert!(dave_new_balance >= dave_balance.saturating_add(NearToken::from_millinear(3490)));

    // Claim Dave Again
    let claim = dave.call(contract.id(), "claim").transact().await?;
    assert!(!claim.is_success());

    Ok(())
}

async fn create_subaccount(
    root: &near_workspaces::Account,
    name: &str,
) -> Result<near_workspaces::Account, Box<dyn std::error::Error>> {
    let subaccount = root
        .create_subaccount(name)
        .initial_balance(TEN_NEAR)
        .transact()
        .await?
        .unwrap();

    Ok(subaccount)
}
