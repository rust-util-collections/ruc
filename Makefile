all: fmt lint

build:
	cargo build

lint:
	cargo clippy --features="full"
	cargo clippy --features="full" --tests
	cargo clippy --no-default-features --features="no_std"

release:
	cargo build --release

test:
	cargo test --release -- --nocapture --test-threads=1
	cargo test --release --no-default-features -- --nocapture --test-threads=1
	cargo test --release --features="full,compact" -- --nocapture --test-threads=1

update:
	rustup update stable
	cargo update

fmt:
	cargo +nightly fmt

fmtall:
	bash tools/fmt.sh

doc:
	cargo doc --features="full" --open

clean:
	cargo clean

cleanall: clean
	git stash
	git clean -fdx
