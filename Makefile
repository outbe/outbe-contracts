clippy:
	cargo clippy --all-targets -- -D warnings

udeps:
	cargo +nightly udeps --target wasm32-unknown-unknown

build:
	docker run --rm -v "$(shell pwd)":/code \
	--mount type=volume,source="$(shell basename "$(shell pwd)")_cache",target=/target \
	--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
	cosmwasm/optimizer:0.16.0
