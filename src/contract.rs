use crate::{
    errors::OracleAggregatorErrors, price_data::normalize_price, storage, types::OracleConfig,
};
use sep_40_oracle::{Asset, PriceData, PriceFeedClient, PriceFeedTrait};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, unwrap::UnwrapOptimized, Address, Env, Vec,
};

#[contract]
pub struct OracleAggregator;

#[contractimpl]
impl PriceFeedTrait for OracleAggregator {
    fn resolution(e: Env) -> u32 {
        panic_with_error!(e, OracleAggregatorErrors::NotImplemented);
    }

    fn price(e: Env, _asset: Asset, _timestamp: u64) -> Option<PriceData> {
        panic_with_error!(e, OracleAggregatorErrors::NotImplemented);
    }

    fn prices(e: Env, _asset: Asset, _records: u32) -> Option<Vec<PriceData>> {
        panic_with_error!(e, OracleAggregatorErrors::NotImplemented);
    }

    fn base(e: Env) -> Asset {
        storage::get_base(&e)
    }

    fn decimals(e: Env) -> u32 {
        storage::get_decimals(&e)
    }

    fn assets(e: Env) -> Vec<Asset> {
        storage::get_assets(&e)
    }

    fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        if !storage::has_asset_config(&e, &asset) {
            panic_with_error!(&e, OracleAggregatorErrors::AssetNotFound);
        }
        if storage::get_blocked_status(&e, &asset) {
            panic_with_error!(&e, OracleAggregatorErrors::AssetBlocked);
        }

        let config = storage::get_asset_config(&e, &asset);
        let oracle = PriceFeedClient::new(&e, &config.oracle_id);
        let price: Option<PriceData> = oracle.lastprice(&config.asset);
        if let Some(price) = price {
            let decimals = storage::get_decimals(&e);
            let normalized_price = normalize_price(price.clone(), &decimals, &config.decimals);
            storage::set_last_fetched_price(&e, &asset, &normalized_price);
            return Some(normalized_price);
        } else {
            if let Some(price) = storage::get_last_fetched_price(&e, &asset) {
                if price.timestamp + 5 * 60 >= e.ledger().timestamp() {
                    return Some(price);
                }
            }
            return None;
        }
    }
}

#[contractimpl]
impl OracleAggregator {
    /// Initialize the contract with the admin and the oracle configurations
    ///
    /// ### Arguments
    /// * `admin` - The address of the admin
    /// * `base` - The base asset
    /// * `assets` - The list of supported assets
    /// * `asset_configs` - The list of oracle configurations for each asset
    ///
    /// ### Errors
    /// * `AlreadyInitialized` - The contract has already been initialized
    /// * `InvalidAssets` - The asset array is invalid
    /// * `InvalidOracleConfig` - The oracle config array is invalid
    pub fn initialize(
        e: Env,
        admin: Address,
        base: Asset,
        assets: Vec<Asset>,
        asset_configs: Vec<OracleConfig>,
        decimals: u32,
    ) {
        if storage::get_is_init(&e) {
            panic_with_error!(&e, OracleAggregatorErrors::AlreadyInitialized);
        }

        let assets_count = assets.len();
        if assets_count <= 0 {
            panic_with_error!(&e, OracleAggregatorErrors::InvalidAssets);
        }

        if assets_count != asset_configs.len() {
            panic_with_error!(&e, OracleAggregatorErrors::InvalidOracleConfig);
        }

        for index in 0..assets_count {
            let asset = assets.get(index).unwrap_optimized();
            let config = asset_configs.get(index).unwrap_optimized();
            if storage::has_asset_config(&e, &asset) {
                panic_with_error!(&e, OracleAggregatorErrors::InvalidAssets);
            }
            storage::set_asset_config(&e, &asset, &config);
        }

        storage::extend_instance(&e);
        storage::set_is_init(&e);
        storage::set_admin(&e, &admin);
        storage::set_base(&e, &base);
        storage::set_decimals(&e, &decimals);
        storage::set_assets(&e, &assets);
    }

    /// Fetch the configuration of an asset
    ///
    /// ### Arguments
    /// * `asset` - The asset to fetch the configuration
    ///
    /// ### Returns
    /// * The configuration of the asset
    ///
    /// ### Errors
    /// * `AssetNotFound` - The asset is not found
    pub fn config(e: Env, asset: Asset) -> OracleConfig {
        if storage::has_asset_config(&e, &asset) {
            return storage::get_asset_config(&e, &asset);
        } else {
            panic_with_error!(&e, OracleAggregatorErrors::AssetNotFound);
        }
    }

    /// (Admin only) Block an asset
    ///
    /// ### Arguments
    /// * `asset` - The asset to block
    ///
    /// ### Errors
    /// * `AssetNotFound` - The asset is not found
    pub fn block(e: Env, asset: Asset) {
        let admin = storage::get_admin(&e);
        admin.require_auth();

        if !storage::has_asset_config(&e, &asset) {
            panic_with_error!(&e, OracleAggregatorErrors::AssetNotFound);
        }
        storage::set_blocked_status(&e, &asset, &true);
    }

    /// (Admin only) Unblock an asset
    ///
    /// ### Arguments
    /// * `asset` - The asset to unblock
    ///
    /// ### Errors
    /// * `AssetNotFound` - The asset is not found
    pub fn unblock(e: Env, asset: Asset) {
        let admin = storage::get_admin(&e);
        admin.require_auth();

        if !storage::has_asset_config(&e, &asset) {
            panic_with_error!(&e, OracleAggregatorErrors::AssetNotFound);
        }
        storage::set_blocked_status(&e, &asset, &false);
    }
}
