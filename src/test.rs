#![cfg(test)]

use super::*;
use soroban_sdk::{Env, Address};
use soroban_sdk::testutils::Address as _;

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubSor);
    let client = SubSorClient::new(&env, &contract_id);

    client.initialize();
    
    // Verify initialization
    let counter: u64 = env
        .storage()
        .instance()
        .get(&DataKey::SubscriptionCounter)
        .unwrap_or(1);
    assert_eq!(counter, 0);
}

#[test]
fn test_create_subscription() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubSor);
    let client = SubSorClient::new(&env, &contract_id);

    client.initialize();

    let owner = Address::generate(&env);
    let subscriber = Address::generate(&env);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    let sub_id = client.create_subscription(
        &owner,
        &subscriber,
        &1000000i128, // 10 XLM (assuming 7 decimals)
        &30u32,
        &recipient,
        &1500u32, // 15%
    );

    assert_eq!(sub_id, 1);

    let subscription = client.get_subscription(&sub_id);
    assert_eq!(subscription.id, 1);
    assert_eq!(subscription.owner, owner);
    assert_eq!(subscription.subscriber, subscriber);
    assert_eq!(subscription.amount, 1000000);
    assert_eq!(subscription.period_days, 30);
    assert_eq!(subscription.recipient, recipient);
    assert_eq!(subscription.split_percentage, 1500);
    assert!(subscription.is_active);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_create_subscription_invalid_amount() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubSor);
    let client = SubSorClient::new(&env, &contract_id);

    client.initialize();

    let owner = Address::generate(&env);
    let subscriber = Address::generate(&env);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    client.create_subscription(&owner, &subscriber, &0i128, &30u32, &recipient, &1500u32);
}

#[test]
#[should_panic(expected = "Split percentage cannot exceed 100%")]
fn test_create_subscription_invalid_split() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubSor);
    let client = SubSorClient::new(&env, &contract_id);

    client.initialize();

    let owner = Address::generate(&env);
    let subscriber = Address::generate(&env);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    client.create_subscription(
        &owner,
        &subscriber,
        &1000000i128,
        &30u32,
        &recipient,
        &10001u32, // Exceeds 100%
    );
}

#[test]
fn test_cancel_subscription() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubSor);
    let client = SubSorClient::new(&env, &contract_id);

    client.initialize();

    let owner = Address::generate(&env);
    let subscriber = Address::generate(&env);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    let sub_id = client.create_subscription(
        &owner,
        &subscriber,
        &1000000i128,
        &30u32,
        &recipient,
        &1500u32,
    );

    client.cancel_subscription(&sub_id);

    let subscription = client.get_subscription(&sub_id);
    assert!(!subscription.is_active);
}

#[test]
fn test_get_balance() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubSor);
    let client = SubSorClient::new(&env, &contract_id);

    client.initialize();

    let owner = Address::generate(&env);
    let subscriber = Address::generate(&env);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    let _sub_id = client.create_subscription(
        &owner,
        &subscriber,
        &1000000i128,
        &30u32,
        &recipient,
        &1500u32, // 15% = 150000
    );

    // Initially balance should be 0
    let balance = client.get_balance(&recipient);
    assert_eq!(balance, 0);

    // After renewal, balance should accumulate
    // Note: In real implementation, token transfers would happen here
}

#[test]
fn test_list_subscriptions() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SubSor);
    let client = SubSorClient::new(&env, &contract_id);

    client.initialize();

    let owner = Address::generate(&env);
    let subscriber1 = Address::generate(&env);
    let subscriber2 = Address::generate(&env);
    let recipient = Address::generate(&env);

    env.mock_all_auths();

    let _sub_id1 = client.create_subscription(
        &owner,
        &subscriber1,
        &1000000i128,
        &30u32,
        &recipient,
        &1500u32,
    );

    let _sub_id2 = client.create_subscription(
        &owner,
        &subscriber2,
        &2000000i128,
        &30u32,
        &recipient,
        &2000u32,
    );

    let subscriptions = client.get_all_subscriptions(&owner);
    assert_eq!(subscriptions.len(), 2);
}
