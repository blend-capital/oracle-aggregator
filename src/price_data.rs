use sep_40_oracle::{PriceData, PriceFeedClient};
use soroban_sdk::Env;

use crate::{storage, types::AssetConfig};

/// Fetch a price based on the asset config
pub fn get_price(e: &Env, config: &AssetConfig) -> Option<PriceData> {
    let oracle = PriceFeedClient::new(e, &config.oracle_id);
    let mut price: Option<PriceData> = oracle.lastprice(&config.asset);
    let decimals = storage::get_decimals(e);
    let oldest_timestamp = e.ledger().timestamp() - storage::get_max_age(e);
    if price.is_none() {
        let mut next_timestamp = e.ledger().timestamp() - config.resolution as u64;
        // attempt to use the `price` method to get an older price if price is None
        while price.is_none() && next_timestamp >= oldest_timestamp {
            price = oracle.price(&config.asset, &next_timestamp);
            next_timestamp -= config.resolution as u64;
        }
    }
    // if we found a price, normalize it and verify it is not too old
    // otherwise, return None
    if let Some(price) = price {
        let normalized_price = normalize_price(price, &decimals, &config.decimals);
        if normalized_price.timestamp >= oldest_timestamp {
            return Some(normalized_price);
        }
    }
    return None;
}

/// Normalize the price data to the correct number of decimals
fn normalize_price(mut price_data: PriceData, decimals: &u32, oracle_decimals: &u32) -> PriceData {
    if oracle_decimals > decimals {
        let diff = oracle_decimals - decimals;
        price_data.price = price_data.price / 10_i128.pow(diff);
        return price_data;
    } else if oracle_decimals < decimals {
        let diff = decimals - oracle_decimals;
        price_data.price = price_data.price * 10_i128.pow(diff);
        return price_data;
    } else {
        return price_data;
    }
}

// @dev: `get_price` tested in intergration tests in `test_lastprice.rs`
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_normalize_price() {
        let price = PriceData {
            price: 1_2345678,
            timestamp: 100,
        };
        let normalized_price = normalize_price(price.clone(), &6, &7);
        assert_eq!(normalized_price.price, 1234567);

        let normalized_price = normalize_price(price.clone(), &8, &7);
        assert_eq!(normalized_price.price, 123456780);

        let normalized_price = normalize_price(price.clone(), &18, &7);
        assert_eq!(normalized_price.price, 1234567800000000000);

        let normalized_price = normalize_price(price, &2, &7);
        assert_eq!(normalized_price.price, 123);
    }
}
