all: build

build:
	cargo build

lint:
	cargo clippy

release:
	cargo build --release

fmt:
	@ cargo fmt

clean:
	@ cargo clean
