import createApi from "./api.js";
import { generateSeed } from "./rand.js";
import { throwError } from "./validate_error.js";

function getLibExt() {
  switch (process.platform) {
    case "darwin":
      return "dylib";
    case "win32":
      return "dll";
    case "linux":
    case "freebsd":
    case "openbsd":
    case "android":
    case "sunos":
      return "so";
  }
}

function getLibLocation() {
  const name = `secp256k1-${process.arch}-${process.platform}.${getLibExt()}`;
  return new URL(name, import.meta.url).pathname;
}

function loadAddon() {
  try {
    const module = { exports: { throwError, generateSeed } };
    process.dlopen(module, getLibLocation());
    return createApi(module.exports);
  } catch (_error) {
    return null;
  }
}

export default loadAddon();
