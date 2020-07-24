all: build

build:
	cargo clippy

release:
	cargo build --release

fmt:
	@ cargo fmt

clean:
	@ cargo clean
