RFLAGS="-C link-arg=-s"

build: sender_staking mock_ft

release:
	$(call docker_build)
ifeq ($(OS), Windows_NT)
	mkdir res
	copy target\wasm32-unknown-unknown\release\sender_staking.wasm res\sender_staking_release.wasm
else
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/sender_staking.wasm res/sender_staking_release.wasm
endif

unittest: build
ifdef TC
	cargo test $(TC) -p sender_staking --lib -- --nocapture
else
	cargo test -p sender_staking --lib -- --nocapture
endif

test: build
ifdef TF
	cargo test -p sender_staking --test $(TF) -- --nocapture
else
	cargo test -p sender_staking --tests -- --nocapture
endif

sender_staking: contracts/sender_staking
	rustup target add wasm32-unknown-unknown
	cargo build -p sender_staking --target wasm32-unknown-unknown --release
ifeq ($(OS), Windows_NT)
	copy target\wasm32-unknown-unknown\release\sender_staking.wasm  res
else
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/sender_staking.wasm ./res/sender_staking.wasm
endif

mock_ft: contracts/mock_ft
	rustup target add wasm32-unknown-unknown
	cargo build -p mock_ft --target wasm32-unknown-unknown --release
ifeq ($(OS), Windows_NT)
	copy target\wasm32-unknown-unknown\release\mock_ft.wasm res
else
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/mock_ft.wasm ./res/mock_ft.wasm
endif


clean:
	cargo clean
	rm -rf res/

define docker_build
	docker build -t my-contract-builder .
	docker run \
		--mount type=bind,source=${PWD},target=/host \
		--cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
		-w /host \
		-e RUSTFLAGS=$(RFLAGS) \
		-i -t my-contract-builder \
		cargo build -p sender_staking --target wasm32-unknown-unknown --release
endef