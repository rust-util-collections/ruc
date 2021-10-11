all: build

build:
	cargo build

lint:
	cargo clippy --features="rich"
	cargo clippy --features="rich" --tests

release:
	cargo build --release

test:
	cargo test --release -- --nocapture --test-threads=1
	cargo test --release --no-default-features -- --nocapture --test-threads=1
	cargo test --release --features="rich" -- --nocapture --test-threads=1

fmt:
	@ cargo fmt

doc:
	cargo doc --open

clean:
	@ cargo clean
