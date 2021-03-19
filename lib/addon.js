import { join } from "path";
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
        default:
            return "¯_(ツ)_/¯";
    }
}
function getPrebuildLibLocation() {
    const name = `secp256k1-${process.arch}-${process.platform}.${getLibExt()}`;
    return new URL(name, import.meta.url).pathname;
}
function getLocalBuildLibLocation(mode) {
    const path = join("..", "target", mode, "libsecp256k1_node.so");
    return new URL(path, import.meta.url).pathname;
}
function dlopen(location) {
    // Suppress: error TS2339: Property 'dlopen' does not exist on type 'Process'.
    const module = { exports: { throwError, generateSeed } };
    // @ts-ignore
    process.dlopen(module, location);
    // @ts-ignore
    return module.exports;
}
export function loadAddon(location) {
    try {
        return createApi(dlopen(location));
    }
    catch (_error) {
        return null;
    }
}
export default loadAddon(getLocalBuildLibLocation("debug")) ||
    loadAddon(getLocalBuildLibLocation("release")) ||
    loadAddon(getPrebuildLibLocation());
