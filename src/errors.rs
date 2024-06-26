use soroban_sdk::contracterror;
#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum OracleAggregatorErrors {
    AlreadyInitialized = 3,

    NotImplemented = 100,
    InvalidOracleConfig = 101,
    InvalidAssets = 102,
    OracleNotFound = 103,
    CircuitBreakerTripped = 104,
    AssetNotFound = 105,
    InvalidTimestamp = 106,
    AssetBlocked = 107,
}
