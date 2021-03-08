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

    // field/scalar implementations (32bit/64bit)
    match env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap().as_str() {
        "32" => {
            base_config
                .define("USE_FIELD_10X26", "1")
                .define("USE_SCALAR_8X32", "1");
        }
        "64" => {
            base_config
                .define("HAVE___INT128", "1")
                .define("USE_ASM_X86_64", "1")
                .define("USE_FIELD_5X52", "1")
                .define("USE_SCALAR_4X64", "1");
        }
        width @ _ => panic!("Unknown pointer width: {}", width),
    }

    // Header files. Only for WASM.
    if env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "wasm32" {
        base_config.include("wasm-sysroot");
    }

    // secp256k1
    base_config
        .file("secp256k1/src/secp256k1.c")
        .compile("libsecp256k1.a");
}
