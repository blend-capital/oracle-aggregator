#![cfg(test)]

use crate::testutils::{setup_default_aggregator, EnvTestUtils};
use sep_40_oracle::Asset;
use soroban_sdk::{testutils::Address as _, vec, Address, Env, Symbol, Vec};

#[test]
fn test_lastprice() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let asset_0 = Asset::Stellar(Address::generate(&e));
    let asset_1 = Asset::Stellar(Address::generate(&e));
    let asset_2 = Asset::Other(Symbol::new(&e, "wETH"));

    let (oracle_aggregator_client, oracle_1, oracle_2) =
        setup_default_aggregator(&e, &admin, &base, &asset_0, &asset_1, &asset_2);

    oracle_1.set_price(
        &Vec::from_array(&e, [0_110000000, 1_000000000]),
        &e.ledger().timestamp(),
    );

    oracle_2.set_price(
        &Vec::from_array(&e, [1010_000000]),
        &(e.ledger().timestamp() - 600),
    );

    let price_0 = oracle_aggregator_client.lastprice(&asset_0).unwrap();
    assert_eq!(price_0.price, 0_1100000);
    assert_eq!(price_0.timestamp, e.ledger().timestamp());

    let price_1 = oracle_aggregator_client.lastprice(&asset_1).unwrap();
    assert_eq!(price_1.price, 1_0000000);
    assert_eq!(price_1.timestamp, e.ledger().timestamp());

    let price_2 = oracle_aggregator_client.lastprice(&asset_2).unwrap();
    assert_eq!(price_2.price, 1010_0000000);
    assert_eq!(price_2.timestamp, e.ledger().timestamp() - 600);
}

#[test]
#[should_panic(expected = "Error(Contract, #104)")]
fn test_lastprice_asset_not_found() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let asset_0 = Asset::Stellar(Address::generate(&e));
    let asset_1 = Asset::Stellar(Address::generate(&e));
    let asset_2 = Asset::Other(Symbol::new(&e, "wETH"));

    let (oracle_aggregator_client, _, _) =
        setup_default_aggregator(&e, &admin, &base, &asset_0, &asset_1, &asset_2);

    oracle_aggregator_client.lastprice(&Asset::Other(Symbol::new(&e, "NOT_FOUND")));
}

#[test]
fn test_lastprice_exceeds_max_timestamp() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let asset_0 = Asset::Stellar(Address::generate(&e));
    let asset_1 = Asset::Stellar(Address::generate(&e));
    let asset_2 = Asset::Other(Symbol::new(&e, "wETH"));

    let (oracle_aggregator_client, oracle_1, _) =
        setup_default_aggregator(&e, &admin, &base, &asset_0, &asset_1, &asset_2);

    let recent_norm_time = e.ledger().timestamp() / 300 * 300;
    oracle_1.set_price(
        &Vec::from_array(&e, [0_120000000, 1_010000000]),
        &(recent_norm_time - 1200),
    );
    oracle_1.set_price(
        &Vec::from_array(&e, [0_120000000, 1_010000000]),
        &(recent_norm_time - 900),
    );

    // jump 1 block to ensure the most recent price is > 900 seconds old
    e.jump(1);

    let price_0 = oracle_aggregator_client.lastprice(&asset_0);
    assert!(price_0.is_none());
}

#[test]
fn test_lastprice_retries_with_timestamp() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let asset_0 = Asset::Stellar(Address::generate(&e));
    let asset_1 = Asset::Stellar(Address::generate(&e));
    let asset_2 = Asset::Other(Symbol::new(&e, "wETH"));

    let (oracle_aggregator_client, oracle_1, oracle_2) =
        setup_default_aggregator(&e, &admin, &base, &asset_0, &asset_1, &asset_2);

    let recent_norm_time_1 = e.ledger().timestamp() / 300 * 300;
    oracle_1.set_price(
        &Vec::from_array(&e, [0_120000000, 1_010000000]),
        &(recent_norm_time_1 - 600),
    );
    oracle_1.set_price(&vec![&e], &(recent_norm_time_1 - 300));
    oracle_1.set_price(&vec![&e], &recent_norm_time_1);

    let recent_norm_time_2 = e.ledger().timestamp() / 600 * 600;
    oracle_2.set_price(
        &Vec::from_array(&e, [100_123_456]),
        &(recent_norm_time_2 - 600),
    );
    oracle_2.set_price(&vec![&e], &recent_norm_time_2);

    e.jump(10);

    let price_0 = oracle_aggregator_client.lastprice(&asset_0).unwrap();
    assert_eq!(price_0.price, 0_1200000);
    assert_eq!(price_0.timestamp, recent_norm_time_1 - 600);

    let price_1 = oracle_aggregator_client.lastprice(&asset_2).unwrap();
    assert_eq!(price_1.price, 100_123_456_0);
    assert_eq!(price_1.timestamp, recent_norm_time_2 - 600);
}

#[test]
fn test_lastprice_retry_exceeds_max_timestamp() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let asset_0 = Asset::Stellar(Address::generate(&e));
    let asset_1 = Asset::Stellar(Address::generate(&e));
    let asset_2 = Asset::Other(Symbol::new(&e, "wETH"));

    let (oracle_aggregator_client, oracle_1, _) =
        setup_default_aggregator(&e, &admin, &base, &asset_0, &asset_1, &asset_2);

    let recent_norm_time = e.ledger().timestamp() / 300 * 300;
    oracle_1.set_price(
        &Vec::from_array(&e, [0_120000000, 1_010000000]),
        &(recent_norm_time - 1200),
    );
    oracle_1.set_price(
        &Vec::from_array(&e, [0_120000000, 1_010000000]),
        &(recent_norm_time - 900),
    );
    oracle_1.set_price(&Vec::from_array(&e, []), &e.ledger().timestamp());

    // jump 1 block to ensure the most recent price is > 900 seconds old
    e.jump(1);

    let price_0 = oracle_aggregator_client.lastprice(&asset_0);
    assert!(price_0.is_none());
}

#[test]
fn test_lastprice_retry_stops_if_over_max_age() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let asset_0 = Asset::Stellar(Address::generate(&e));
    let asset_1 = Asset::Stellar(Address::generate(&e));
    let asset_2 = Asset::Other(Symbol::new(&e, "wETH"));

    let (oracle_aggregator_client, oracle_1, oracle_2) =
        setup_default_aggregator(&e, &admin, &base, &asset_0, &asset_1, &asset_2);

    let recent_norm_time = e.ledger().timestamp() / 300 * 300;
    oracle_1.set_price(
        &Vec::from_array(&e, [0_120000000, 1_010000000]),
        &(recent_norm_time - 900),
    );
    oracle_1.set_price(&Vec::from_array(&e, []), &(recent_norm_time - 600));
    oracle_1.set_price(&Vec::from_array(&e, []), &(recent_norm_time - 300));
    oracle_1.set_price(&Vec::from_array(&e, []), &recent_norm_time);

    let recent_norm_time_2 = e.ledger().timestamp() / 600 * 600;
    oracle_2.set_price(
        &Vec::from_array(&e, [1_000_000]),
        &(recent_norm_time - 1800),
    );
    oracle_2.set_price(&Vec::from_array(&e, []), &(recent_norm_time_2 - 1200));
    oracle_2.set_price(&Vec::from_array(&e, []), &(recent_norm_time_2 - 600));
    oracle_2.set_price(&Vec::from_array(&e, []), &recent_norm_time_2);

    // jump 1 block to ensure the most recent price is > 900 seconds old
    e.jump(1);

    // validate both prices are not found, and oracle 2 (longer resolution) checks less
    // entries than oracle 1. This is a bit hacky but shows that the retry mechanism
    // stops based on the resolution of the oracle.
    let price_0 = oracle_aggregator_client.lastprice(&asset_0);
    let read_entries_0 = e.cost_estimate().resources().read_entries;
    assert!(price_0.is_none());

    let price_2 = oracle_aggregator_client.lastprice(&asset_2);
    let read_entries_2 = e.cost_estimate().resources().read_entries;
    assert!(price_2.is_none());

    assert!(read_entries_2 < read_entries_0);
}
