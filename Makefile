all: build

build:
	cargo build

lint:
	cargo clippy --features="full"
	cargo clippy --features="full" --tests

release:
	cargo build --release

test:
	cargo test --release -- --nocapture --test-threads=1
	cargo test --release --no-default-features -- --nocapture --test-threads=1
	cargo test --release --features="full" -- --nocapture --test-threads=1

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
