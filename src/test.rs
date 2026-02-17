#![cfg(test)]
use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{RevoraRevenueShare, RevoraRevenueShareClient};

#[test]
fn it_emits_events_on_register_and_report() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RevoraRevenueShare);
    let client = RevoraRevenueShareClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let token = Address::generate(&env);

    client.register_offering(&issuer, &token, &1_000); // 10% in bps
    client.report_revenue(&issuer, &token, &1_000_000, &1);

    // In a real test, inspect events / state here.
    assert!(env.events().all().len() >= 2);
}

