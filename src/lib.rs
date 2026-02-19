#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol, String, Map};

/// Basic skeleton for a revenue-share contract.
///
/// This is intentionally minimal and focuses on the high-level shape:
/// - Registering a startup "offering"
/// - Recording a revenue report
/// - Emitting events that an off-chain distribution engine can consume
/// - Attaching off-chain metadata references to offerings

#[contract]
pub struct RevoraRevenueShare;

#[derive(Clone)]
pub struct Offering {
    pub issuer: Address,
    pub token: Address,
    pub revenue_share_bps: u32,
}

// Storage key constants
const METADATA_KEY: Symbol = symbol_short!("meta");

// Event symbols
const EVENT_REVENUE_REPORTED: Symbol = symbol_short!("rev_rep");
const EVENT_METADATA_CREATED: Symbol = symbol_short!("meta_new");
const EVENT_METADATA_UPDATED: Symbol = symbol_short!("meta_upd");
const EVENT_METADATA_DELETED: Symbol = symbol_short!("meta_del");

// Configuration constants
const MAX_METADATA_LENGTH: u32 = 1024; // 1KB max for metadata URI

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

    /// Set metadata reference for an offering.
    /// Only the issuer or issuer admin can set metadata.
    /// 
    /// # Arguments
    /// * `env` - The contract environment
    /// * `issuer` - The issuer address
    /// * `offering_id` - Unique identifier for the offering
    /// * `metadata_uri` - Off-chain metadata reference (IPFS hash, HTTPS URL, etc.)
    ///
    /// # Panics
    /// - If metadata_uri exceeds MAX_METADATA_LENGTH
    /// - If caller is not authorized (issuer or admin)
    /// - If metadata_uri is empty
    pub fn set_metadata(
        env: Env,
        issuer: Address,
        offering_id: String,
        metadata_uri: String,
    ) {
        issuer.require_auth();
        
        // Validate metadata_uri is not empty
        if metadata_uri.len() == 0 {
            panic!("Metadata URI cannot be empty");
        }

        // Validate metadata_uri length
        if metadata_uri.len() > MAX_METADATA_LENGTH {
            panic!("Metadata URI exceeds maximum length of {} bytes", MAX_METADATA_LENGTH);
        }

        // Create a compound key for the metadata storage
        let mut metadata_map: Map<String, String> = env
            .storage()
            .persistent()
            .get(&(METADATA_KEY, issuer.clone()))
            .unwrap_or_else(|| Map::new(&env));

        let is_new = !metadata_map.contains_key(offering_id.clone());

        // Store the metadata reference
        metadata_map.set(offering_id.clone(), metadata_uri.clone());
        env.storage()
            .persistent()
            .set(&(METADATA_KEY, issuer.clone()), &metadata_map);

        // Emit appropriate event
        if is_new {
            env.events().publish(
                (EVENT_METADATA_CREATED, issuer.clone()),
                (offering_id, metadata_uri),
            );
        } else {
            env.events().publish(
                (EVENT_METADATA_UPDATED, issuer.clone()),
                (offering_id, metadata_uri),
            );
        }
    }

    /// Get metadata reference for an offering.
    /// Returns the stored metadata URI or None if not set.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `issuer` - The issuer address
    /// * `offering_id` - Unique identifier for the offering
    pub fn get_metadata(
        env: Env,
        issuer: Address,
        offering_id: String,
    ) -> Option<String> {
        let metadata_map: Map<String, String> = env
            .storage()
            .persistent()
            .get(&(METADATA_KEY, issuer))
            .unwrap_or_else(|| Map::new(&env));

        metadata_map.get(offering_id)
    }

    /// Update metadata reference for an offering.
    /// Only the issuer can update existing metadata.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `issuer` - The issuer address
    /// * `offering_id` - Unique identifier for the offering
    /// * `metadata_uri` - New off-chain metadata reference
    ///
    /// # Panics
    /// - If metadata doesn't exist for this offering
    /// - If new metadata_uri exceeds MAX_METADATA_LENGTH
    /// - If caller is not the issuer
    /// - If new metadata_uri is empty
    pub fn update_metadata(
        env: Env,
        issuer: Address,
        offering_id: String,
        metadata_uri: String,
    ) {
        issuer.require_auth();

        // Validate metadata_uri is not empty
        if metadata_uri.len() == 0 {
            panic!("Metadata URI cannot be empty");
        }

        // Validate metadata_uri length
        if metadata_uri.len() > MAX_METADATA_LENGTH {
            panic!("Metadata URI exceeds maximum length of {} bytes", MAX_METADATA_LENGTH);
        }

        let mut metadata_map: Map<String, String> = env
            .storage()
            .persistent()
            .get(&(METADATA_KEY, issuer.clone()))
            .unwrap_or_else(|| Map::new(&env));

        // Verify metadata exists
        if !metadata_map.contains_key(offering_id.clone()) {
            panic!("No metadata found for offering");
        }

        // Update the metadata reference
        metadata_map.set(offering_id.clone(), metadata_uri.clone());
        env.storage()
            .persistent()
            .set(&(METADATA_KEY, issuer.clone()), &metadata_map);

        // Emit update event
        env.events().publish(
            (EVENT_METADATA_UPDATED, issuer.clone()),
            (offering_id, metadata_uri),
        );
    }

    /// Delete metadata reference for an offering.
    /// Only the issuer can delete metadata.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `issuer` - The issuer address
    /// * `offering_id` - Unique identifier for the offering
    ///
    /// # Panics
    /// - If metadata doesn't exist for this offering
    /// - If caller is not the issuer
    pub fn delete_metadata(
        env: Env,
        issuer: Address,
        offering_id: String,
    ) {
        issuer.require_auth();

        let mut metadata_map: Map<String, String> = env
            .storage()
            .persistent()
            .get(&(METADATA_KEY, issuer.clone()))
            .unwrap_or_else(|| Map::new(&env));

        // Verify metadata exists
        if !metadata_map.contains_key(offering_id.clone()) {
            panic!("No metadata found for offering");
        }

        // Remove the metadata reference
        metadata_map.remove(offering_id.clone());
        env.storage()
            .persistent()
            .set(&(METADATA_KEY, issuer.clone()), &metadata_map);

        // Emit deletion event (using updated event with empty string to indicate deletion)
        env.events().publish(
            (EVENT_METADATA_DELETED, issuer.clone()),
            (offering_id,),
        );
    }
}

mod test;

