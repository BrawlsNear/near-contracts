use near_sdk::{json_types::U128, NearToken};
use near_sdk::{near, AccountId, Gas};
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::{Account, Contract};
use serde_json::json;

const TEN_NEAR: NearToken = NearToken::from_near(10);

#[tokio::test]

async fn test_contract_is_operational() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;

    let root: near_workspaces::Account = sandbox.root_account()?;

    // Create accounts
    let alice = create_subaccount(&root, "alice").await?;
    let bob = create_subaccount(&root, "bob").await?;
    let contract_account = create_subaccount(&root, "contract").await?;

    // Deploy factory contract
    let contract_wasm = near_workspaces::compile_project("./").await?;
    let contract = contract_account.deploy(&contract_wasm).await?.unwrap();

    let deploy_new_brawl: ExecutionFinalResult = alice
        .call(contract.id(), "deploy_new_brawl")
        .args_json(
            json!({"title":"Test Brawl".to_string(), "id":"test_brawl".to_string(), "options":vec!["option-1".to_string(), "option-2".to_string()]}),
        )
        .max_gas()
        .deposit(NearToken::from_millinear(1600))
        .transact()
        .await?;

    assert!(deploy_new_brawl.is_success());

    let new_brawl_account_id: AccountId = format!("test_brawl.{}", contract.id()).parse().unwrap();

    let owner = alice
        .view(&new_brawl_account_id, "get_owner")
        .args_json({})
        .await?
        .json::<AccountId>()?;

    let alice_id = alice.id().to_string();
    assert_eq!(owner.to_string(), alice_id);

    // Finalize Not Owner
    let finalize = bob
        .call(&new_brawl_account_id, "update_correct_option")
        .args_json(json!({"option":"option-1"}))
        .transact()
        .await?;
    assert!(!finalize.is_success());

    // Finalize Owner
    let finalize = alice
        .call(&new_brawl_account_id, "update_correct_option")
        .args_json(json!({"option":"option-2"}))
        .transact()
        .await?;
    assert!(finalize.is_success());

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
