all: build

build:
	cargo build

lint:
	cargo clippy

release:
	cargo build --release

test:
	cargo test -- --nocapture --test-threads=1

fmt:
	@ cargo fmt

doc:
	cargo doc --open

clean:
	@ cargo clean
