build-wasm-cp = mkdir -p lib && cp -f target/wasm32-unknown-unknown/$(1)/secp256k1_wasm.wasm $(build-wasm-location)
build-wasm-location = lib/secp256k1.wasm

.PHONY: build-node-%
build-node-%: export PAIR = $(subst +, ,$(subst build-node-,,$@))
build-node-%:
	cargo build --package secp256k1-node --target $(firstword $(PAIR)) --release
	cp target/$(firstword $(PAIR))/release/libsecp256k1_node.so lib/secp256k1-$(lastword $(PAIR)).so

.PHONY: build-node-debug-%
build-node-debug-%: export PAIR = $(subst +, ,$(subst build-node-debug-,,$@))
build-node-debug-%:
	cargo build --package secp256k1-node --target $(firstword $(PAIR))
	cp target/$(firstword $(PAIR))/release/libsecp256k1_node.so lib/secp256k1-$(lastword $(PAIR)).so

.PHONY: build-wasm
build-wasm:
	cargo build --package secp256k1-wasm --target wasm32-unknown-unknown --release
	$(call build-wasm-cp,release)
	wasm-opt --strip-debug --strip-producers --output $(build-wasm-location) $(build-wasm-location)
	node util/wasm-strip.js $(build-wasm-location)
	wasm-opt -O4 --output $(build-wasm-location) $(build-wasm-location)

.PHONY: build-node-debug
build-wasm-debug:
	cargo build --package secp256k1-wasm --target wasm32-unknown-unknown
	$(call build-wasm-cp,debug)

.PHONY: clean
clean:
	rm -rf lib/secp256k1* target node_modules tests/browser

.PHONY: format
format:
	cargo-fmt
	npx prettier -w . 

.PHONY: lint
lint:
	cargo fmt -- --check
	cargo clippy --target wasm32-unknown-unknown
	npx prettier -c .

.PHONY: test
test: test-browser test-node

.PHONY: test-browser-build
test-browser-build:
	npx webpack build -c tests/browser.webpack.js

.PHONY: test-browser
test-browser: build-wasm-debug test-browser-build
	cat tests/browser/index.js | npx browser-run --static tests/browser | npx tap-difflet -p

.PHONY: test-node
test-node: build-node-debug
	node --experimental-json-modules tests/index.js | npx tap-difflet -p
