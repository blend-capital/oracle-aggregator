use sep_40_oracle::Asset;
use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone)]
pub struct AssetConfig {
    /// The asset to be used when fetching the price
    pub asset: Asset,
    /// The address of the source oracle
    pub oracle_id: Address,
    /// The decimals of the source oracle
    pub decimals: u32,
    /// The resolution of the source oracle (in seconds)
    pub resolution: u32,
}
