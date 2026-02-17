#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

/// Basic skeleton for a revenue-share contract.
///
/// This is intentionally minimal and focuses on the high-level shape:
/// - Registering a startup "offering"
/// - Recording a revenue report
/// - Emitting events that an off-chain distribution engine can consume

#[contract]
pub struct RevoraRevenueShare;

#[derive(Clone)]
pub struct Offering {
    pub issuer: Address,
    pub token: Address,
    pub revenue_share_bps: u32,
}

const EVENT_REVENUE_REPORTED: Symbol = symbol_short!("rev_rep");

#[contractimpl]
impl RevoraRevenueShare {
    /// Register a new revenue-share offering.
    /// In a production contract this would handle access control, supply caps,
    /// and issuance hooks. Here we only emit an event.
    pub fn register_offering(env: Env, issuer: Address, token: Address, revenue_share_bps: u32) {
        issuer.require_auth();

        env.events().publish(
            (symbol_short!("offer_reg"), issuer.clone()),
            (token, revenue_share_bps),
        );
    }

    /// Record a revenue report for an offering.
    /// The actual payout calculation and distribution can be performed either
    /// fully on-chain or in a hybrid model where this event is the trigger.
    pub fn report_revenue(env: Env, issuer: Address, token: Address, amount: i128, period_id: u64) {
        issuer.require_auth();

        env.events().publish(
            (EVENT_REVENUE_REPORTED, issuer.clone(), token.clone()),
            (amount, period_id),
        );
    }
}

mod test;

