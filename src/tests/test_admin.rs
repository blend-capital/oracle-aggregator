#![cfg(test)]
use crate::{
    storage,
    testutils::{assert_assets_equal, create_oracle_aggregator, EnvTestUtils},
};
use sep_40_oracle::{
    testutils::{Asset as MockAsset, MockPriceOracleClient, MockPriceOracleWASM},
    Asset,
};
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Env, Error, IntoVal, Symbol, Vec,
};

#[test]
fn test_admin_actions() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    e.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let decimals = 7;
    let max_age = 900;

    let asset_0 = Address::generate(&e);
    let asset_1 = Address::generate(&e);
    let asset_2 = Address::generate(&e);
    let symbol_2 = Symbol::new(&e, "wETH");

    // deploy two mock oracles
    // - setup oracle with all asset prices
    let oracle_0_1_id = Address::generate(&e);
    e.register_at(&oracle_0_1_id, MockPriceOracleWASM, ());
    let oracle_0_1 = MockPriceOracleClient::new(&e, &oracle_0_1_id);
    oracle_0_1.set_data(
        &Address::generate(&e),
        &MockAsset::Other(Symbol::new(&e, "BASE")),
        &Vec::from_array(
            &e,
            [
                MockAsset::Stellar(asset_0.clone()),
                MockAsset::Stellar(asset_1.clone()),
                // @dev - this is added just to test edge cases but is not used
                MockAsset::Stellar(asset_2.clone()),
            ],
        ),
        &9,
        &300,
    );
    let oracle_0_1_norm_timestamp = e.ledger().timestamp() / 300 * 300;
    oracle_0_1.set_price(
        &vec![&e, 1_300_000_055, 2223_421_213_231, 0_900_120_123],
        &(e.ledger().timestamp() - 300),
    );
    oracle_0_1.set_price(
        &vec![&e, 1_200_000_055, 2123_421_213_231, 0_924_120_123],
        &e.ledger().timestamp(),
    );

    // - setup oracle with ASSET_2 price
    let oracle_2_id = Address::generate(&e);
    e.register_at(&oracle_2_id, MockPriceOracleWASM, ());
    let oracle_2 = MockPriceOracleClient::new(&e, &oracle_2_id);
    oracle_2.set_data(
        &Address::generate(&e),
        &MockAsset::Other(Symbol::new(&e, "BASE")),
        &Vec::from_array(&e, [MockAsset::Other(symbol_2.clone())]),
        &6,
        &600,
    );
    let oracle_2_norm_timestamp = e.ledger().timestamp() / 600 * 600;
    oracle_2.set_price(&vec![&e, 0_994_120], &(e.ledger().timestamp() - 600));
    oracle_2.set_price(&vec![&e, 1_024_025], &e.ledger().timestamp());

    // deploy oracle aggregator
    let (oracle_aggregator_id, oracle_aggregator_client) =
        create_oracle_aggregator(&e, &admin, &base, &decimals, &max_age);
    assert!(assert_assets_equal(oracle_aggregator_client.base(), base));
    assert_eq!(oracle_aggregator_client.decimals(), 7);
    assert_eq!(oracle_aggregator_client.max_age(), 900);

    e.jump(10);

    // add asset_0
    let result_0 = oracle_aggregator_client.add_asset(
        &Asset::Stellar(asset_0.clone()),
        &oracle_0_1_id,
        &Asset::Stellar(asset_0.clone()),
    );
    assert_eq!(
        e.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    oracle_aggregator_id.clone(),
                    Symbol::new(&e, "add_asset"),
                    vec![
                        &e,
                        Asset::Stellar(asset_0.clone()).into_val(&e),
                        oracle_0_1_id.to_val(),
                        Asset::Stellar(asset_0.clone()).into_val(&e),
                    ]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
    assert_eq!(result_0.price, 1_200_000_0);
    assert_eq!(result_0.timestamp, oracle_0_1_norm_timestamp);
    let assets = oracle_aggregator_client.assets();
    assert_eq!(assets.len(), 1);
    assert_assets_equal(assets.get_unchecked(0), Asset::Stellar(asset_0.clone()));
    let asset_0_config = oracle_aggregator_client
        .config(&Asset::Stellar(asset_0.clone()))
        .unwrap();
    assert_eq!(asset_0_config.oracle_id, oracle_0_1_id);
    assert_eq!(asset_0_config.decimals, 9);
    assert_eq!(asset_0_config.resolution, 300);
    assert_assets_equal(asset_0_config.asset, Asset::Stellar(asset_0.clone()));

    // add asset_1
    let result_1 = oracle_aggregator_client.add_asset(
        &Asset::Stellar(asset_1.clone()),
        &oracle_0_1_id,
        &Asset::Stellar(asset_1.clone()),
    );
    assert_eq!(result_1.price, 2123_421_213_2);
    assert_eq!(result_1.timestamp, oracle_0_1_norm_timestamp);
    let assets = oracle_aggregator_client.assets();
    assert_eq!(assets.len(), 2);
    assert_assets_equal(assets.get_unchecked(0), Asset::Stellar(asset_0.clone()));
    assert_assets_equal(assets.get_unchecked(1), Asset::Stellar(asset_1.clone()));
    let asset_1_config = oracle_aggregator_client
        .config(&Asset::Stellar(asset_1.clone()))
        .unwrap();
    assert_eq!(asset_1_config.oracle_id, oracle_0_1_id);
    assert_eq!(asset_1_config.decimals, 9);
    assert_eq!(asset_1_config.resolution, 300);
    assert_assets_equal(asset_1_config.asset, Asset::Stellar(asset_1.clone()));

    // try and add an incorrect config for asset_2
    let result_bad_config = oracle_aggregator_client.try_add_asset(
        &Asset::Stellar(asset_2.clone()),
        &oracle_2_id,
        &Asset::Stellar(asset_2.clone()),
    );
    // - mock oracle throws an error during last price fetch
    assert_eq!(
        result_bad_config.err(),
        Some(Ok(Error::from_contract_error(2)))
    );

    // try and add asset_2 when the list is full
    // - manually create a really long asset list to test the max asset limit
    e.as_contract(&oracle_aggregator_id, || {
        let mut assets = Vec::new(&e);
        for _ in 0..50 {
            assets.push_back(Asset::Stellar(Address::generate(&e)));
        }
        storage::set_assets(&e, &assets);
    });
    // - do test
    let result_full = oracle_aggregator_client.try_add_asset(
        &Asset::Stellar(asset_2.clone()),
        &oracle_2_id,
        &Asset::Other(symbol_2.clone()),
    );
    assert_eq!(result_full.err(), Some(Ok(Error::from_contract_error(102))));
    // - revert asset list change
    e.as_contract(&oracle_aggregator_id, || {
        storage::set_assets(&e, &assets);
    });

    // add asset_2
    let result_2 = oracle_aggregator_client.add_asset(
        &Asset::Stellar(asset_2.clone()),
        &oracle_2_id,
        &Asset::Other(symbol_2.clone()),
    );
    assert_eq!(result_2.price, 1_024_0250);
    assert_eq!(result_2.timestamp, oracle_2_norm_timestamp);
    let assets = oracle_aggregator_client.assets();
    assert_eq!(assets.len(), 3);
    assert_assets_equal(assets.get_unchecked(0), Asset::Stellar(asset_0.clone()));
    assert_assets_equal(assets.get_unchecked(1), Asset::Stellar(asset_1.clone()));
    assert_assets_equal(assets.get_unchecked(2), Asset::Stellar(asset_2.clone()));
    let asset_2_config = oracle_aggregator_client
        .config(&Asset::Stellar(asset_2.clone()))
        .unwrap();
    assert_eq!(asset_2_config.oracle_id, oracle_2_id);
    assert_eq!(asset_2_config.decimals, 6);
    assert_eq!(asset_2_config.resolution, 600);
    assert_assets_equal(asset_2_config.asset, Asset::Other(symbol_2.clone()));

    // try and add a duplicate asset
    let result_dupe = oracle_aggregator_client.try_add_asset(
        &Asset::Stellar(asset_2.clone()),
        &oracle_0_1_id,
        &Asset::Stellar(asset_2.clone()),
    );
    assert_eq!(result_dupe.err(), Some(Ok(Error::from_contract_error(103))));
}

#[test]
#[should_panic(expected = "Error(Contract, #105)")]
fn test_init_max_age_too_small() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    e.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let decimals = 7;
    let max_age = 359;

    create_oracle_aggregator(&e, &admin, &base, &decimals, &max_age);
}

#[test]
#[should_panic(expected = "Error(Contract, #105)")]
fn test_init_max_age_too_large() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    e.cost_estimate().budget().reset_unlimited();

    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let decimals = 7;
    let max_age = 359;

    create_oracle_aggregator(&e, &admin, &base, &decimals, &max_age);
}
