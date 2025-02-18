#![cfg(test)]
use crate::{
    testutils::{
        assert_assets_equal, create_oracle_aggregator, setup_default_aggregator, EnvTestUtils,
    },
    types::OracleConfig,
};
use sep_40_oracle::Asset;
use soroban_sdk::{testutils::Address as _, vec, Address, Env, Symbol};

#[test]
fn test_initalize() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    e.cost_estimate().budget().reset_unlimited();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let asset_0 = Asset::Stellar(Address::generate(&e));
    let asset_1 = Asset::Stellar(Address::generate(&e));
    let asset_2 = Asset::Other(Symbol::new(&e, "wETH"));

    let (aggregator, oracle_aggregator_client) = create_oracle_aggregator(&e);
    let (oracle_0_1, oracle_2) =
        setup_default_aggregator(&e, &aggregator, &admin, &base, &asset_0, &asset_1, &asset_2);

    assert!(assert_assets_equal(oracle_aggregator_client.base(), base));

    assert_eq!(oracle_aggregator_client.decimals(), 7);

    let assets = oracle_aggregator_client.assets();
    assert_eq!(oracle_aggregator_client.assets().len(), 3);
    assert_assets_equal(assets.get(0).unwrap(), asset_0.clone());
    assert_assets_equal(assets.get(1).unwrap(), asset_1.clone());
    assert_assets_equal(assets.get(2).unwrap(), asset_2.clone());
    let config_0 = oracle_aggregator_client.config(&asset_0);
    assert_eq!(config_0.decimals, oracle_0_1.decimals());
    assert_eq!(config_0.resolution, oracle_0_1.resolution());
    assert_eq!(config_0.oracle_id, oracle_0_1.address);
    let config_1 = oracle_aggregator_client.config(&asset_1);
    assert_eq!(config_1.decimals, oracle_0_1.decimals());
    assert_eq!(config_1.resolution, oracle_0_1.resolution());
    assert_eq!(config_1.oracle_id, oracle_0_1.address);
    let config_2 = oracle_aggregator_client.config(&asset_2);
    assert_eq!(config_2.decimals, oracle_2.decimals());
    assert_eq!(config_2.resolution, oracle_2.resolution());
    assert_eq!(config_2.oracle_id, oracle_2.address);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_already_initialized() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let asset_0 = Asset::Stellar(Address::generate(&e));
    let asset_1 = Asset::Stellar(Address::generate(&e));
    let asset_2 = Asset::Other(Symbol::new(&e, "wETH"));

    let (aggregator, oracle_aggregator_client) = create_oracle_aggregator(&e);
    setup_default_aggregator(&e, &aggregator, &admin, &base, &asset_0, &asset_1, &asset_2);

    oracle_aggregator_client.initialize(
        &admin,
        &base,
        &vec![&e, asset_0],
        &vec![
            &e,
            OracleConfig {
                oracle_id: Address::generate(&e),
                decimals: 7,
                resolution: 234,
                asset: base.clone(),
            },
        ],
        &7,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #102)")]
fn test_initalize_no_assets() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));

    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e);

    oracle_aggregator_client.initialize(&admin, &base, &vec![&e], &vec![&e], &7);
}

#[test]
#[should_panic(expected = "Error(Contract, #101)")]
fn test_initalize_missing_configs() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let asset_0 = Asset::Stellar(Address::generate(&e));
    let asset_1 = Asset::Stellar(Address::generate(&e));

    let (_, oracle_aggregator_client) = create_oracle_aggregator(&e);

    oracle_aggregator_client.initialize(
        &admin,
        &base,
        &vec![&e, asset_0, asset_1],
        &vec![
            &e,
            OracleConfig {
                oracle_id: Address::generate(&e),
                decimals: 7,
                resolution: 234,
                asset: base.clone(),
            },
        ],
        &7,
    );
}
