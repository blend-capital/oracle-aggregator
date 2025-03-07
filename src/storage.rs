use crate::types::AssetConfig;
use sep_40_oracle::Asset;
use soroban_sdk::{contracttype, unwrap::UnwrapOptimized, Address, Env, Symbol, Vec};

const ADMIN_KEY: &str = "Admin";
const ASSETS_KEY: &str = "Assets";
const BASE_KEY: &str = "Base";
const DECIMALS_KEY: &str = "Decimals";
const MAX_AGE_KEY: &str = "MaxAge";

const ONE_DAY_LEDGERS: u32 = 17280; // assumes 5 seconds per ledger on average
const LEDGER_THRESHOLD: u32 = 30 * ONE_DAY_LEDGERS;
const LEDGER_BUMP: u32 = 31 * ONE_DAY_LEDGERS;

#[derive(Clone)]
#[contracttype]
pub enum AggregatorDataKey {
    Asset(Asset),
}

//********** Storage Utils **********//

/// Bump the instance lifetime by the defined amount
pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(LEDGER_THRESHOLD, LEDGER_BUMP);
}

/********** Instance **********/

/// Set the admin address
pub fn set_admin(e: &Env, admin: &Address) {
    e.storage()
        .instance()
        .set::<Symbol, Address>(&Symbol::new(e, ADMIN_KEY), &admin);
}

/// Get the admin address
pub fn get_admin(e: &Env) -> Address {
    e.storage()
        .instance()
        .get::<Symbol, Address>(&Symbol::new(e, ADMIN_KEY))
        .unwrap_optimized()
}

/// Set the max age of a price, in seconds
pub fn set_max_age(e: &Env, max_age: &u64) {
    e.storage()
        .instance()
        .set::<Symbol, u64>(&Symbol::new(e, MAX_AGE_KEY), max_age);
}

/// Set the max age of a price, in seconds
pub fn get_max_age(e: &Env) -> u64 {
    e.storage()
        .instance()
        .get::<Symbol, u64>(&Symbol::new(e, MAX_AGE_KEY))
        .unwrap_optimized()
}

/// Set the base asset for the oracle aggregator
pub fn set_base(e: &Env, base: &Asset) {
    e.storage()
        .instance()
        .set::<Symbol, Asset>(&Symbol::new(e, BASE_KEY), base);
}

/// Get the base asset for the oracle aggregator
pub fn get_base(e: &Env) -> Asset {
    e.storage()
        .instance()
        .get::<Symbol, Asset>(&Symbol::new(e, BASE_KEY))
        .unwrap()
}

/// Set the number of decimals the oracle will report prices in
pub fn set_decimals(e: &Env, decimals: &u32) {
    e.storage()
        .instance()
        .set::<Symbol, u32>(&Symbol::new(e, DECIMALS_KEY), decimals);
}

/// Get the number of decimals the oracle will report prices in
pub fn get_decimals(e: &Env) -> u32 {
    e.storage()
        .instance()
        .get::<Symbol, u32>(&Symbol::new(e, DECIMALS_KEY))
        .unwrap()
}

/// Set a list of assets
pub fn set_assets(e: &Env, assets: &Vec<Asset>) {
    e.storage()
        .instance()
        .set::<Symbol, Vec<Asset>>(&Symbol::new(e, ASSETS_KEY), assets);
}

/// Get a list of assets
pub fn get_assets(e: &Env) -> Vec<Asset> {
    e.storage()
        .instance()
        .get::<Symbol, Vec<Asset>>(&Symbol::new(e, ASSETS_KEY))
        .unwrap()
}

/********** Persistent **********/

/// Set an asset configuration
pub fn set_asset_config(e: &Env, asset: &Asset, config: &AssetConfig) {
    let key = AggregatorDataKey::Asset(asset.clone());
    e.storage()
        .persistent()
        .set::<AggregatorDataKey, AssetConfig>(&key, config);
    e.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
}

/// Get an asset configuration
pub fn get_asset_config(e: &Env, asset: &Asset) -> Option<AssetConfig> {
    let key = AggregatorDataKey::Asset(asset.clone());
    if let Some(result) = e.storage().persistent().get(&key) {
        e.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_THRESHOLD, LEDGER_BUMP);
        result
    } else {
        None
    }
}
