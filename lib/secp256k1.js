const { readFileSync } = require("fs");
const { join } = require("path");
const validate_wasm = require("./validate_wasm");

const binary = readFileSync(join(__dirname, "secp256k1.wasm"));
const imports = {
  "./validate_wasm.js": validate_wasm,
  // TODO: temporary, fix WASM and remove `env`.
  env: {
    free() {},
    malloc() {},
    secp256k1_default_illegal_callback_fn() {},
    secp256k1_default_error_callback_fn() {},
  },
};

const mod = new WebAssembly.Module(binary);
const instance = new WebAssembly.Instance(mod, imports);

module.exports = instance.exports;
