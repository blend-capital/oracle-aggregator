#![cfg(test)]

use crate::testutils::{create_oracle_aggregator, setup_default_aggregator, EnvTestUtils};
use sep_40_oracle::Asset;
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    vec, Address, Env, Error, IntoVal, Symbol, Vec,
};

#[test]
fn test_block_unblock() {
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

    let price = oracle_aggregator_client.lastprice(&asset_0);
    assert!(price.is_some());

    oracle_aggregator_client.block(&asset_0);
    assert_eq!(
        e.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    oracle_aggregator_client.address.clone(),
                    Symbol::new(&e, "block"),
                    vec![&e, asset_0.into_val(&e)]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
    let result = oracle_aggregator_client.try_lastprice(&asset_0).err();
    assert_eq!(result, Some(Ok(Error::from_contract_error(107))));

    oracle_aggregator_client.unblock(&asset_0);
    assert_eq!(
        e.auths()[0],
        (
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    oracle_aggregator_client.address.clone(),
                    Symbol::new(&e, "unblock"),
                    vec![&e, asset_0.into_val(&e)]
                )),
                sub_invocations: std::vec![]
            }
        )
    );
    let price = oracle_aggregator_client.lastprice(&asset_0);
    assert!(price.is_some());
}

#[test]
#[should_panic(expected = "Error(Contract, #105)")]
fn test_block_asset_not_found() {
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
    let asset = Asset::Other(Symbol::new(&e, "NOT_FOUND"));

    oracle_aggregator_client.block(&asset);
}

#[test]
#[should_panic(expected = "Error(Contract, #105)")]
fn test_unblock_asset_not_found() {
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

    let asset = Asset::Other(Symbol::new(&e, "NOT_FOUND"));
    oracle_aggregator_client.unblock(&asset);
}
