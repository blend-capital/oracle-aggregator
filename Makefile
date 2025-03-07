default: build

test: build
	cargo test --all --tests

build:
	mkdir -p target/wasm32-unknown-unknown/optimized

	cargo rustc --manifest-path=Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release 
	stellar contract optimize \
		--wasm target/wasm32-unknown-unknown/release/oracle_aggregator.wasm \
		--wasm-out target/wasm32-unknown-unknown/optimized/oracle_aggregator.wasm

	cd target/wasm32-unknown-unknown/optimized/ && \
		for i in *.wasm ; do \
			ls -l "$$i"; \
		done

fmt:
	cargo fmt --all

clean:
	cargo clean

generate-js:
	stellar contract bindings typescript --overwrite \
		--contract-id CBWH54OKUK6U2J2A4J2REJEYB625NEFCHISWXLOPR2D2D6FTN63TJTWN \
		--wasm ./target/wasm32-unknown-unknown/optimized/oracle_aggregator.wasm --output-dir ./js/aggregator/ \
		--rpc-url http://localhost:8000 --network-passphrase "Standalone Network ; February 2017" --network Standalone

