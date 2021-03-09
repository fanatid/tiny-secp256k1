use std::env;

fn main() {
    let mut base_config = cc::Build::new();
    base_config
        .include("secp256k1/")
        .include("secp256k1/include")
        .include("secp256k1/src")
        .flag_if_supported("-Wno-unused-function")
        .flag_if_supported("-Wno-nonnull-compare")
        .define("USE_EXTERNAL_DEFAULT_CALLBACKS", "1");

    // Default in configuration file
    base_config
        .define("SECP256K1_BUILD", "1")
        .define("ECMULT_GEN_PREC_BITS", "4")
        .define("ECMULT_WINDOW_SIZE", "15");

    // `--with-bignum=no`
    base_config
        .define("USE_NUM_NONE", "1")
        .define("USE_FIELD_INV_BUILTIN", "1")
        .define("USE_SCALAR_INV_BUILTIN", "1");

    // TODO: benchmark first
    // .define("USE_ENDOMORPHISM", "1");

    // Header files. Only for WASM.
    if env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "wasm32" {
        base_config.include("wasm-sysroot");
    }

    // secp256k1
    base_config
        .file("secp256k1/src/secp256k1.c")
        .compile("libsecp256k1.a");
}
