import { readFileSync } from "fs";
import { URL } from "url";
import * as validate_wasm from "./validate_wasm.js";

const binary = readFileSync(new URL("secp256k1.wasm", import.meta.url));
const imports = {
  "./validate_wasm.js": validate_wasm,
};

const mod = new WebAssembly.Module(binary);
const instance = new WebAssembly.Instance(mod, imports);

export default instance.exports;
