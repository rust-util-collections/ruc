all: fmt lint test

build:
	cargo build

lint:
	cargo clippy --features="full"
	cargo clippy --features="full" --tests

release:
	cargo build --release

test:
	cargo test --release -- --test-threads=1
	cargo test --release --no-default-features -- --test-threads=1
	cargo test --release --features="full,compact" -- --test-threads=1

update:
	cargo update --verbose

fmt:
	cargo fmt

fmtall:
	bash tools/fmt.sh

doc:
	cargo doc --features="full" --open

clean:
	cargo clean

cleanall: clean
	git stash
	git clean -fdx
