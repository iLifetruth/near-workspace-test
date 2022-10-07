use serde_json::json;
use near_units::parse_near;
// use workspaces::prelude::*; 
use workspaces::{network::Sandbox, Account, Contract, Worker};

const WASM_FILEPATH: &str = "../../res/status_message.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;

    // Load Contracts
    let wasm_0 = std::fs::read(WASM_FILEPATH)?;

    // create accounts
    let owner = worker.root_account();

    // Contract Deploy
    let contract_0 = owner
        .create_subaccount(&worker, "contract_0")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;

    let contract_1 = owner
        .create_subaccount(&worker, "contract_1")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;

    let contract_a = contract_0.deploy(&worker, &wasm_0).await?.unwrap();
    let contract_b = contract_1.deploy(&worker, &wasm_0).await?.unwrap();

    // AccountId("alice.test.near")
    // AccountId("bob.test.near")
    println!("{:?} {:?}",contract_0.id(),contract_a.id());
    println!("{:?} {:?}",contract_1.id(),contract_b.id());

    let user = owner
        .create_subaccount(&worker, "user")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;

    // begin tests  
    test_set_message(&owner, &user, &contract_a, &worker).await?;
    test_null_messages(&owner, &user, &contract_b, &worker).await?;
    test_differing_statuses(&owner, &user, &contract_a, &worker).await?;
    Ok(())
}   

async fn test_set_message(
    owner: &Account,
    user: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    user
        .call(&worker, contract.id(), "set_status")
        .args_json(json!({ "message": "hello" }))?
        .transact()
        .await?;

    let alice_status: String = owner
        .call(&worker, contract.id(), "get_status")
        .args_json(json!({ "account_id": user.id() }))?
        .transact()
        .await?
        .json()?;

    assert_eq!(alice_status, "hello");
    println!("      Passed ✅ set get message");
    Ok(())
}

async fn test_null_messages(
    owner: &Account,
    user: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let owner_status: Option<String> = user
        .call(&worker, contract.id(), "get_status")
        .args_json(json!({ "account_id": owner.id() }))?
        .transact()
        .await?
        .json()?;

    assert_eq!(owner_status, None);
    println!("      Passed ✅ get nonexistent message");
    Ok(())
}

async fn test_differing_statuses(
    owner: &Account,
    user: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    owner
        .call(&worker, contract.id(), "set_status")
        .args_json(json!({ "message": "world" }))?
        .transact()
        .await?;

    let alice_status: String = owner
        .call(&worker, contract.id(), "get_status")
        .args_json(json!({ "account_id": user.id() }))?
        .transact()
        .await?
        .json()?;

    assert_eq!(alice_status, "hello");

    let owner_status: String = owner
        .call(&worker, contract.id(), "get_status")
        .args_json(json!({ "account_id": owner.id() }))?
        .transact()
        .await?
        .json()?;

    assert_eq!(owner_status, "world");
    println!("      Passed ✅ root and alice have different statuses");
    Ok(())
}