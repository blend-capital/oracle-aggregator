use crate::{errors::OracleAggregatorErrors, price_data::get_price, storage, types::AssetConfig};
use sep_40_oracle::{Asset, PriceData, PriceFeedClient, PriceFeedTrait};
use soroban_sdk::{contract, contractimpl, panic_with_error, vec, Address, Env, Vec};

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
        let config = storage::get_asset_config(&e, &asset);
        if let Some(config) = config {
            crate::price_data::get_price(&e, &config)
        } else {
            panic_with_error!(&e, OracleAggregatorErrors::AssetNotFound);
        }
    }
}

#[contractimpl]
impl OracleAggregator {
    /// Initialize the oracle aggregator contract.
    ///
    /// ### Arguments
    /// * `admin` - The address of the admin
    /// * `base` - The address of the base asset the oracle will report in
    /// * `decimals` - The decimals the oracle will report in
    /// * `max_age` - The maximum time the oracle will look back for a price (in seconds)
    ///
    /// ### Errors
    /// * `InvalidMaxAge` - The max age is not between 360 (6m) and 3600 (60m)
    pub fn __constructor(e: Env, admin: Address, base: Asset, decimals: u32, max_age: u64) {
        storage::extend_instance(&e);
        storage::set_admin(&e, &admin);
        storage::set_base(&e, &base);
        storage::set_decimals(&e, &decimals);
        if max_age < 360 || max_age > 3600 {
            panic_with_error!(&e, OracleAggregatorErrors::InvalidMaxAge);
        }
        storage::set_max_age(&e, &max_age);
        storage::set_assets(&e, &vec![&e]);
    }

    /// Fetch the configuration of an asset
    pub fn config(e: Env, asset: Asset) -> Option<AssetConfig> {
        storage::get_asset_config(&e, &asset)
    }

    /// Fetch the max age of a price
    pub fn max_age(e: Env) -> u64 {
        storage::get_max_age(&e)
    }

    /// (Admin Only) Add an asset to the oracle aggregator
    ///
    /// ### Arguments
    /// * `asset` - The asset to add
    /// * `oracle_id` - The address of the oracle
    /// * `oracle_asset` - The asset used to fetch the oracle price
    ///
    /// ### Errors
    /// * `InvalidAssetOracle` - Unable to fetch a price for the oracle asset
    /// * `AssetExists` - The asset already exists
    ///
    /// ### Returns
    /// The price data of the asset as would be returned by `self.lastprice`. This is useful
    /// for simulation to verify the asset was added correctly.
    pub fn add_asset(e: Env, asset: Asset, oracle_id: Address, oracle_asset: Asset) -> PriceData {
        storage::get_admin(&e).require_auth();

        // verify asset list is not full and the asset has not already been added
        let mut asset_list = storage::get_assets(&e);
        if asset_list.contains(&asset) {
            panic_with_error!(&e, OracleAggregatorErrors::AssetExists);
        } else if asset_list.len() >= 50 {
            panic_with_error!(&e, OracleAggregatorErrors::MaxAssetsExceeded);
        }

        let oracle_client = PriceFeedClient::new(&e, &oracle_id);
        let oracle_decimals = oracle_client.decimals();
        let oracle_resolution = oracle_client.resolution();
        let config = AssetConfig {
            asset: oracle_asset,
            oracle_id,
            decimals: oracle_decimals,
            resolution: oracle_resolution,
        };
        let price = get_price(&e, &config);
        if let Some(price) = price {
            // able to fetch a price for the asset, add asset and return price
            storage::set_asset_config(&e, &asset, &config);
            asset_list.push_back(asset);
            storage::set_assets(&e, &asset_list);
            return price;
        } else {
            panic_with_error!(&e, OracleAggregatorErrors::InvalidAssetOracle);
        }
    }
}
