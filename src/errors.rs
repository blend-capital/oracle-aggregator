use soroban_sdk::contracterror;
#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum OracleAggregatorErrors {
    NotImplemented = 100,
    InvalidAssetOracle = 101,
    MaxAssetsExceeded = 102,
    AssetExists = 103,
    AssetNotFound = 104,
    InvalidMaxAge = 105,
}
