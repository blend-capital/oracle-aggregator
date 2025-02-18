#![cfg(test)]

use crate::{
    storage,
    testutils::{create_oracle_aggregator, setup_default_aggregator, EnvTestUtils},
};
use sep_40_oracle::{Asset, PriceData};
use soroban_sdk::{testutils::Address as _, Address, Env, Error, Symbol, Vec};

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

    let (aggregator, oracle_aggregator_client) = create_oracle_aggregator(&e);
    let (oracle_1, oracle_2) =
        setup_default_aggregator(&e, &aggregator, &admin, &base, &asset_0, &asset_1, &asset_2);
    oracle_1.set_price(
        &Vec::from_array(&e, [0_110000000, 1_000000000]),
        &e.ledger().timestamp(),
    );
    oracle_2.set_price(
        &Vec::from_array(&e, [1010_000000]),
        &(e.ledger().timestamp() - 600),
    );

    let price_0 = oracle_aggregator_client.lastprice(&asset_0);
    match price_0 {
        Some(price) => {
            assert_eq!(price.price, 0_1100000);
            assert_eq!(price.timestamp, e.ledger().timestamp());
        }
        None => {
            assert!(false)
        }
    }

    let price_1 = oracle_aggregator_client.lastprice(&asset_1);
    match price_1 {
        Some(price) => {
            assert_eq!(price.price, 1_0000000);
            assert_eq!(price.timestamp, e.ledger().timestamp());
        }
        None => {
            assert!(false)
        }
    }

    let price_2 = oracle_aggregator_client.lastprice(&asset_2);
    match price_2 {
        Some(price) => {
            assert_eq!(price.price, 1010_0000000);
            assert_eq!(price.timestamp, e.ledger().timestamp() - 600);
        }
        None => {
            assert!(false)
        }
    }
}

#[test]
#[should_panic(expected = "Error(Contract, #105)")]
fn test_lastprice_asset_not_found() {
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

    oracle_aggregator_client.lastprice(&Asset::Other(Symbol::new(&e, "NOT_FOUND")));
}

#[test]
fn test_lastprice_asset_blocked() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let asset_0 = Asset::Stellar(Address::generate(&e));
    let asset_1 = Asset::Stellar(Address::generate(&e));
    let asset_2 = Asset::Other(Symbol::new(&e, "wETH"));

    let (aggregator, oracle_aggregator_client) = create_oracle_aggregator(&e);
    let (oracle_1, oracle_2) =
        setup_default_aggregator(&e, &aggregator, &admin, &base, &asset_0, &asset_1, &asset_2);
    oracle_1.set_price(
        &Vec::from_array(&e, [0_110000000, 1_000000000]),
        &e.ledger().timestamp(),
    );
    oracle_2.set_price(
        &Vec::from_array(&e, [1010_000000]),
        &(e.ledger().timestamp() - 600),
    );
    oracle_aggregator_client.block(&asset_1);
    let price_0 = oracle_aggregator_client.lastprice(&asset_0);
    match price_0 {
        Some(price) => {
            assert_eq!(price.price, 0_1100000);
            assert_eq!(price.timestamp, e.ledger().timestamp());
        }
        None => {
            assert!(false)
        }
    }
    let result = oracle_aggregator_client.try_lastprice(&asset_1);
    assert_eq!(result.err(), Some(Ok(Error::from_contract_error(107))));
}

#[test]
fn test_lastprice_use_last_fetched() {
    let e = Env::default();
    e.set_default_info();
    e.mock_all_auths();
    let admin = Address::generate(&e);
    let base = Asset::Other(Symbol::new(&e, "BASE"));
    let asset_0 = Asset::Stellar(Address::generate(&e));
    let asset_1 = Asset::Stellar(Address::generate(&e));
    let asset_2 = Asset::Other(Symbol::new(&e, "wETH"));

    let (aggregator, oracle_aggregator_client) = create_oracle_aggregator(&e);

    let asset_0_last_fetched_price = PriceData {
        price: 0_1100000,
        timestamp: e.ledger().timestamp() - (5 * 60),
    };
    let asset_1_last_fetched_price = PriceData {
        price: 1_0000000,
        timestamp: e.ledger().timestamp() - (5 * 60 + 1),
    };

    e.as_contract(&aggregator, || {
        storage::set_last_fetched_price(&e, &asset_0, &asset_0_last_fetched_price);
        storage::set_last_fetched_price(&e, &asset_1, &asset_1_last_fetched_price);
    });
    setup_default_aggregator(&e, &aggregator, &admin, &base, &asset_0, &asset_1, &asset_2);
    let price_0 = oracle_aggregator_client.lastprice(&asset_0);
    match price_0 {
        Some(price) => {
            assert_eq!(price.price, 0_1100000);
            assert_eq!(price.timestamp, asset_0_last_fetched_price.timestamp);
        }
        None => {
            assert!(false)
        }
    }

    // last fetched price is older than 5 minutes
    let price_1 = oracle_aggregator_client.lastprice(&asset_1);
    match price_1 {
        Some(_) => {
            assert!(false)
        }
        None => {
            assert!(true)
        }
    }

    // last fetched price is not set
    let price_2 = oracle_aggregator_client.lastprice(&asset_2);
    match price_2 {
        Some(_) => {
            assert!(false)
        }
        None => {
            assert!(true)
        }
    }
}
