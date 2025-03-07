# Oracle Aggregator

Example Oracle Aggregator that can be used with Blend pools. This contract allows one Blend pool to access multiple oracle prices sources from one location via `lastprice` method.

### Supported Oracles

This Oracle Aggregator contract makes a few assumptions about the oracles it can support:

* Oracle must have the same base asset as the aggregator (e.g. USD)
* Oracle must support SEP-40 `lastprice(asset: Address)`, and it should return the most recently reported price by the oracle
* If `lastprice(asset: Address)` can return `None` intermittently, like in the event of the most recent round being missed, the oracle must support SEP-40 `price(asset: Address, timestamp: u64)`, and it should return the most recently reported price on or before the timestamp given

### Config

The oracle aggregator uses some global configuration defined through the constructor:

* admin `Address` - The admin has the ability to add additional assets to the oracle aggregator. This should be done cautiosly, as they can never be removed or edited.
* base `Asset` - The base asset the oracle aggregator will report prices in
* decimals `u32` - The decimals the oracle aggregator will report prices in
* max_age `u64` - The maximum age (in seconds) of a fetched price the oracle aggregator will return from the current ledger timestamp. This must be between 360s (6m) and 3600s (60m).

Each supported asset is defined via an AssetConfig:

**Asset Config**
* asset `Asset` - The asset to be used when fetching the price
* oracle_id `Address` - The address of the source oracle
* decimals `u32` - The decimals of the source oracle
* resolution `u32` - The resolution of the source oracle (in seconds)

Up to 50 additional assets can be supported.

### Last Price Method

The aggregator will attempt to fetch the assets price via `lastprice` first. Some oracles opt to return `None` if the latest round did not reach consensus, or there was an issue. In this case, the aggregator will attempt to call `price` for each `resolution` period since the current timestamp, up to the `max_age` of a price. If no price can be resolved that is at most `max_age` old, the aggregator will return `None`.

## Safety

Oracle Aggregator has not had an audit conducted. If an audit is conducted, it will appear here.

Oracle Aggregator is made available under the MIT License, which disclaims all warranties in relation to the project and which limits the liability of those that contribute and maintain the project, including Script3. You acknowledge that you are solely responsible for any use of Oracle Aggregator and you assume all risks associated with any such use.
