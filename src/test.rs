#![cfg(test)]

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events},
    vec, Address, Env, IntoVal,
};

use crate::{RevoraRevenueShare, RevoraRevenueShareClient};

// ─── Event-to-flow mapping ───────────────────────────────────────────────────
//
//  Flow: Offering Registration  (register_offering)
//    topic[0] = Symbol("offer_reg")
//    topic[1] = Address  (issuer)
//    data     = (Address (token), u32 (revenue_share_bps))
//
//  Flow: Revenue Report  (report_revenue)
//    topic[0] = Symbol("rev_rep")
//    topic[1] = Address  (issuer)
//    topic[2] = Address  (token)
//    data     = (i128 (amount), u64 (period_id))
//
// ─────────────────────────────────────────────────────────────────────────────

// ── Single-event structure tests ─────────────────────────────────────────────

#[test]
fn register_offering_emits_exact_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RevoraRevenueShare);
    let client = RevoraRevenueShareClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let token = Address::generate(&env);
    let bps: u32 = 1_500;

    client.register_offering(&issuer, &token, &bps);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                (symbol_short!("offer_reg"), issuer.clone()).into_val(&env),
                (token.clone(), bps).into_val(&env),
            ),
        ]
    );
}

#[test]
fn report_revenue_emits_exact_event() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RevoraRevenueShare);
    let client = RevoraRevenueShareClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let token = Address::generate(&env);
    let amount: i128 = 5_000_000;
    let period_id: u64 = 42;

    client.report_revenue(&issuer, &token, &amount, &period_id);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                (symbol_short!("rev_rep"), issuer.clone(), token.clone()).into_val(&env),
                (amount, period_id).into_val(&env),
            ),
        ]
    );
}

// ── Ordering tests ───────────────────────────────────────────────────────────

#[test]
fn combined_flow_preserves_event_order() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RevoraRevenueShare);
    let client = RevoraRevenueShareClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let token = Address::generate(&env);
    let bps: u32 = 1_000;
    let amount: i128 = 1_000_000;
    let period_id: u64 = 1;

    client.register_offering(&issuer, &token, &bps);
    client.report_revenue(&issuer, &token, &amount, &period_id);

    let events = env.events().all();
    assert_eq!(events.len(), 2);

    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id.clone(),
                (symbol_short!("offer_reg"), issuer.clone()).into_val(&env),
                (token.clone(), bps).into_val(&env),
            ),
            (
                contract_id.clone(),
                (symbol_short!("rev_rep"), issuer.clone(), token.clone()).into_val(&env),
                (amount, period_id).into_val(&env),
            ),
        ]
    );
}

#[test]
fn complex_mixed_flow_events_in_order() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RevoraRevenueShare);
    let client = RevoraRevenueShareClient::new(&env, &contract_id);

    let issuer_a = Address::generate(&env);
    let issuer_b = Address::generate(&env);
    let token_x = Address::generate(&env);
    let token_y = Address::generate(&env);

    // Interleave: register A, register B, report A, report B
    client.register_offering(&issuer_a, &token_x, &500);
    client.register_offering(&issuer_b, &token_y, &750);
    client.report_revenue(&issuer_a, &token_x, &100_000, &1);
    client.report_revenue(&issuer_b, &token_y, &200_000, &1);

    let events = env.events().all();
    assert_eq!(events.len(), 4);

    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id.clone(),
                (symbol_short!("offer_reg"), issuer_a.clone()).into_val(&env),
                (token_x.clone(), 500u32).into_val(&env),
            ),
            (
                contract_id.clone(),
                (symbol_short!("offer_reg"), issuer_b.clone()).into_val(&env),
                (token_y.clone(), 750u32).into_val(&env),
            ),
            (
                contract_id.clone(),
                (symbol_short!("rev_rep"), issuer_a.clone(), token_x.clone()).into_val(&env),
                (100_000i128, 1u64).into_val(&env),
            ),
            (
                contract_id.clone(),
                (symbol_short!("rev_rep"), issuer_b.clone(), token_y.clone()).into_val(&env),
                (200_000i128, 1u64).into_val(&env),
            ),
        ]
    );
}

// ── Multi-entity tests ───────────────────────────────────────────────────────

#[test]
fn multiple_offerings_emit_distinct_events() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RevoraRevenueShare);
    let client = RevoraRevenueShareClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let token_a = Address::generate(&env);
    let token_b = Address::generate(&env);
    let token_c = Address::generate(&env);

    client.register_offering(&issuer, &token_a, &100);
    client.register_offering(&issuer, &token_b, &200);
    client.register_offering(&issuer, &token_c, &300);

    let events = env.events().all();
    assert_eq!(events.len(), 3);

    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id.clone(),
                (symbol_short!("offer_reg"), issuer.clone()).into_val(&env),
                (token_a.clone(), 100u32).into_val(&env),
            ),
            (
                contract_id.clone(),
                (symbol_short!("offer_reg"), issuer.clone()).into_val(&env),
                (token_b.clone(), 200u32).into_val(&env),
            ),
            (
                contract_id.clone(),
                (symbol_short!("offer_reg"), issuer.clone()).into_val(&env),
                (token_c.clone(), 300u32).into_val(&env),
            ),
        ]
    );
}

#[test]
fn multiple_revenue_reports_same_offering() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RevoraRevenueShare);
    let client = RevoraRevenueShareClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let token = Address::generate(&env);

    client.report_revenue(&issuer, &token, &10_000, &1);
    client.report_revenue(&issuer, &token, &20_000, &2);
    client.report_revenue(&issuer, &token, &30_000, &3);

    let events = env.events().all();
    assert_eq!(events.len(), 3);

    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id.clone(),
                (symbol_short!("rev_rep"), issuer.clone(), token.clone()).into_val(&env),
                (10_000i128, 1u64).into_val(&env),
            ),
            (
                contract_id.clone(),
                (symbol_short!("rev_rep"), issuer.clone(), token.clone()).into_val(&env),
                (20_000i128, 2u64).into_val(&env),
            ),
            (
                contract_id.clone(),
                (symbol_short!("rev_rep"), issuer.clone(), token.clone()).into_val(&env),
                (30_000i128, 3u64).into_val(&env),
            ),
        ]
    );
}

#[test]
fn same_issuer_different_tokens() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RevoraRevenueShare);
    let client = RevoraRevenueShareClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let token_x = Address::generate(&env);
    let token_y = Address::generate(&env);

    client.register_offering(&issuer, &token_x, &1_000);
    client.register_offering(&issuer, &token_y, &2_000);
    client.report_revenue(&issuer, &token_x, &500_000, &1);
    client.report_revenue(&issuer, &token_y, &750_000, &1);

    let events = env.events().all();
    assert_eq!(events.len(), 4);

    assert_eq!(
        events,
        vec![
            &env,
            // Registrations: same issuer topic, different token in data
            (
                contract_id.clone(),
                (symbol_short!("offer_reg"), issuer.clone()).into_val(&env),
                (token_x.clone(), 1_000u32).into_val(&env),
            ),
            (
                contract_id.clone(),
                (symbol_short!("offer_reg"), issuer.clone()).into_val(&env),
                (token_y.clone(), 2_000u32).into_val(&env),
            ),
            // Revenue reports: token appears in topics, distinguishing them
            (
                contract_id.clone(),
                (symbol_short!("rev_rep"), issuer.clone(), token_x.clone()).into_val(&env),
                (500_000i128, 1u64).into_val(&env),
            ),
            (
                contract_id.clone(),
                (symbol_short!("rev_rep"), issuer.clone(), token_y.clone()).into_val(&env),
                (750_000i128, 1u64).into_val(&env),
            ),
        ]
    );
}

// ── Topic / symbol inspection tests ──────────────────────────────────────────

#[test]
fn topic_symbols_are_distinct() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RevoraRevenueShare);
    let client = RevoraRevenueShareClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let token = Address::generate(&env);

    client.register_offering(&issuer, &token, &1_000);
    client.report_revenue(&issuer, &token, &1_000_000, &1);

    // The two event types carry different topic[0] symbols ("offer_reg" vs "rev_rep")
    // and different topic arities (2 vs 3). Verify via exact structural match.
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                (symbol_short!("offer_reg"), issuer.clone()).into_val(&env),
                (token.clone(), 1_000u32).into_val(&env),
            ),
            (
                contract_id.clone(),
                (symbol_short!("rev_rep"), issuer.clone(), token.clone()).into_val(&env),
                (1_000_000i128, 1u64).into_val(&env),
            ),
        ]
    );
}

#[test]
fn rev_rep_topics_include_token_address() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RevoraRevenueShare);
    let client = RevoraRevenueShareClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let token = Address::generate(&env);

    client.report_revenue(&issuer, &token, &999, &7);

    // Verify topic tuple is (Symbol("rev_rep"), issuer, token) — token in topics
    // allows off-chain indexers to filter by token without decoding data.
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                (symbol_short!("rev_rep"), issuer.clone(), token.clone()).into_val(&env),
                (999i128, 7u64).into_val(&env),
            ),
        ]
    );
}

// ── Boundary / edge-case tests ───────────────────────────────────────────────

#[test]
fn zero_bps_offering() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RevoraRevenueShare);
    let client = RevoraRevenueShareClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let token = Address::generate(&env);

    client.register_offering(&issuer, &token, &0);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                (symbol_short!("offer_reg"), issuer.clone()).into_val(&env),
                (token.clone(), 0u32).into_val(&env),
            ),
        ]
    );
}

#[test]
fn max_bps_offering() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RevoraRevenueShare);
    let client = RevoraRevenueShareClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let token = Address::generate(&env);

    // 10_000 bps == 100%
    client.register_offering(&issuer, &token, &10_000);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                (symbol_short!("offer_reg"), issuer.clone()).into_val(&env),
                (token.clone(), 10_000u32).into_val(&env),
            ),
        ]
    );
}

#[test]
fn zero_amount_revenue_report() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RevoraRevenueShare);
    let client = RevoraRevenueShareClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let token = Address::generate(&env);

    client.report_revenue(&issuer, &token, &0, &1);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                (symbol_short!("rev_rep"), issuer.clone(), token.clone()).into_val(&env),
                (0i128, 1u64).into_val(&env),
            ),
        ]
    );
}

#[test]
fn large_revenue_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RevoraRevenueShare);
    let client = RevoraRevenueShareClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let token = Address::generate(&env);

    let large_amount: i128 = i128::MAX;
    client.report_revenue(&issuer, &token, &large_amount, &u64::MAX);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                (symbol_short!("rev_rep"), issuer.clone(), token.clone()).into_val(&env),
                (large_amount, u64::MAX).into_val(&env),
            ),
        ]
    );
}

#[test]
fn negative_revenue_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RevoraRevenueShare);
    let client = RevoraRevenueShareClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let token = Address::generate(&env);

    // Negative revenue (e.g. clawback / adjustment)
    let negative: i128 = -500_000;
    client.report_revenue(&issuer, &token, &negative, &99);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                (symbol_short!("rev_rep"), issuer.clone(), token.clone()).into_val(&env),
                (negative, 99u64).into_val(&env),
            ),
        ]
    );
}

// ── Original smoke test (kept for backwards compatibility) ───────────────────

#[test]
fn it_emits_events_on_register_and_report() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, RevoraRevenueShare);
    let client = RevoraRevenueShareClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let token = Address::generate(&env);

    client.register_offering(&issuer, &token, &1_000);
    client.report_revenue(&issuer, &token, &1_000_000, &1);

    assert!(env.events().all().len() >= 2);
}
