.PHONY: all run-v42 run-v43

all: run-v42 run-v43

run-v42:
	cargo run --manifest-path v42/Cargo.toml --target-dir target/v42

run-v43:
	cargo run --manifest-path v43/Cargo.toml --target-dir target/v43
