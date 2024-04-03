#![cfg(test)]
use crate::{
    types::{Asset, OracleConfig, PriceData, SettingsConfig},
    OracleAggregator, OracleAggregatorClient,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    Address, Env, Symbol, Vec,
};
mod oracle_aggregator_wasm {
    soroban_sdk::contractimport!(
        file = "./target/wasm32-unknown-unknown/optimized/oracle_aggregator.wasm"
    );
}
use oracle_aggregator_wasm::WASM as OracleAggregatorWasm;

use crate::mock_oracle::{MockOracle, MockOracleClient};

const ONE_DAY_LEDGERS: u32 = 24 * 60 * 60 / 5;

pub trait EnvTestUtils {
    /// Jump the env by the given amount of ledgers. Assumes 5 seconds per ledger.
    fn jump(&self, ledgers: u32);

    /// Set the ledger to the default LedgerInfo
    ///
    /// Time -> 1441065600 (Sept 1st, 2015 12:00:00 AM UTC)
    /// Sequence -> 100
    fn set_default_info(&self);
}

impl EnvTestUtils for Env {
    fn jump(&self, ledgers: u32) {
        self.ledger().set(LedgerInfo {
            timestamp: self.ledger().timestamp().saturating_add(ledgers as u64 * 5),
            protocol_version: 20,
            sequence_number: self.ledger().sequence().saturating_add(ledgers),
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 50 * ONE_DAY_LEDGERS,
            min_persistent_entry_ttl: 50 * ONE_DAY_LEDGERS,
            max_entry_ttl: 365 * ONE_DAY_LEDGERS,
        });
    }

    fn set_default_info(&self) {
        self.ledger().set(LedgerInfo {
            timestamp: 1441065600, // Sept 1st, 2015 12:00:00 AM UTC
            protocol_version: 20,
            sequence_number: 100,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 50 * ONE_DAY_LEDGERS,
            min_persistent_entry_ttl: 50 * ONE_DAY_LEDGERS,
            max_entry_ttl: 365 * ONE_DAY_LEDGERS,
        });
    }
}

pub fn create_oracle_aggregator<'a>(
    e: &Env,
    admin: &Address,
    config: &SettingsConfig,
) -> (Address, OracleAggregatorClient<'a>) {
    let oracle_aggregator_address = e.register_contract(None, OracleAggregator {});
    let oracle_aggregator_client: OracleAggregatorClient<'a> =
        OracleAggregatorClient::new(&e, &oracle_aggregator_address);
    oracle_aggregator_client.initialize(admin, config);
    return (oracle_aggregator_address, oracle_aggregator_client);
}

pub fn create_oracle_aggregator_wasm<'a>(
    e: &Env,
    admin: &Address,
    config: &SettingsConfig,
) -> (Address, OracleAggregatorClient<'a>) {
    let oracle_aggregator_address = e.register_contract_wasm(None, OracleAggregatorWasm);
    let oracle_aggregator_client: OracleAggregatorClient<'a> =
        OracleAggregatorClient::new(&e, &oracle_aggregator_address);
    oracle_aggregator_client.initialize(admin, config);
    return (oracle_aggregator_address, oracle_aggregator_client);
}

pub fn default_aggregator_settings(
    e: &Env,
) -> (SettingsConfig, MockOracleClient, MockOracleClient) {
    let xlm = Asset::Stellar(Address::generate(&e));
    let usdc = Asset::Other(Symbol::new(&e, "USDC"));
    let weth = Asset::Other(Symbol::new(&e, "wETH"));

    let assets = Vec::from_array(&e, [xlm.clone(), usdc.clone(), weth.clone()]);

    let xlm_usdc_oracle_id = e.register_contract(None, MockOracle {});
    let xlm_usdc_oracle: MockOracleClient = MockOracleClient::new(&e, &xlm_usdc_oracle_id);
    xlm_usdc_oracle.set_prices(
        &xlm,
        &Vec::from_array(
            &e,
            [
                PriceData {
                    price: 0_110000000,
                    timestamp: e.ledger().timestamp(),
                },
                PriceData {
                    price: 0_100000000,
                    timestamp: e.ledger().timestamp() - 300,
                },
                PriceData {
                    price: 0_120000000,
                    timestamp: e.ledger().timestamp() - 300 * 2,
                },
            ],
        ),
    );

    xlm_usdc_oracle.set_prices(
        &usdc,
        &Vec::from_array(
            &e,
            [
                PriceData {
                    price: 1_000000000,
                    timestamp: e.ledger().timestamp(),
                },
                PriceData {
                    price: 0_990000000,
                    timestamp: e.ledger().timestamp() - 300,
                },
                PriceData {
                    price: 1_010000000,
                    timestamp: e.ledger().timestamp() - 300 * 2,
                },
            ],
        ),
    );

    let weth_oracle_id = e.register_contract(None, MockOracle {});
    let weth_oracle: MockOracleClient = MockOracleClient::new(&e, &weth_oracle_id);
    weth_oracle.set_prices(
        &weth,
        &Vec::from_array(
            &e,
            [
                PriceData {
                    price: 1010_000000,
                    timestamp: e.ledger().timestamp(),
                },
                PriceData {
                    price: 1010_000000,
                    timestamp: e.ledger().timestamp() - 600,
                },
                PriceData {
                    price: 999_000000,
                    timestamp: e.ledger().timestamp() - 600 * 2,
                },
            ],
        ),
    );

    let asset_configs = Vec::from_array(
        &e,
        [
            OracleConfig {
                oracle_id: xlm_usdc_oracle_id.clone(),
                decimals: 9,
                resolution: 300,
            },
            OracleConfig {
                oracle_id: xlm_usdc_oracle_id,
                decimals: 9,
                resolution: 300,
            },
            OracleConfig {
                oracle_id: weth_oracle_id,
                decimals: 6,
                resolution: 600,
            },
        ],
    );

    return (
        SettingsConfig {
            assets,
            asset_configs,
            decimals: 7,
            base: usdc,
            enable_circuit_breaker: true,
            // 20% deviation from the previous price in 5 minutes
            circuit_breaker_threshold: 100000,
            // 2 hours
            circuit_breaker_timeout: 7200,
        },
        xlm_usdc_oracle,
        weth_oracle,
    );
}