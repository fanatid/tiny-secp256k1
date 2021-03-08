#![no_std]
#![feature(core_intrinsics)]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::intrinsics::abort()
}

#[cfg(not(target_arch = "wasm32"))]
compile_error!("Only `wasm32` target_arch is supported.");

use secp256k1_sys::{
    c_void, secp256k1_context_no_precomp, secp256k1_ec_pubkey_combine, secp256k1_ec_pubkey_create,
    secp256k1_ec_pubkey_parse, secp256k1_ec_pubkey_serialize, secp256k1_ec_pubkey_tweak_add,
    secp256k1_ec_pubkey_tweak_mul, secp256k1_ec_seckey_negate, secp256k1_ec_seckey_tweak_add,
    secp256k1_ecdsa_sign, secp256k1_ecdsa_signature_normalize,
    secp256k1_ecdsa_signature_parse_compact, secp256k1_ecdsa_signature_serialize_compact,
    secp256k1_ecdsa_verify, secp256k1_nonce_function_rfc6979, Context, PublicKey, Signature,
    SECP256K1_SER_COMPRESSED, SECP256K1_SER_UNCOMPRESSED,
};

#[link(wasm_import_module = "./validate_wasm.js")]
extern "C" {
    #[link_name = "generateInt32"]
    fn generate_int32() -> i32;

    #[link_name = "throwError"]
    fn throw_error(errcode: usize);
}

const PRIVATE_KEY_SIZE: usize = 32;
const PUBLIC_KEY_COMPRESSED_SIZE: usize = 33;
const PUBLIC_KEY_UNCOMPRESSED_SIZE: usize = 65;
const TWEAK_SIZE: usize = 32;
const HASH_SIZE: usize = 32;
const EXTRA_DATA_SIZE: usize = 32;
const SIGNATURE_SIZE: usize = 64;

// const ERROR_BAD_PRIVATE: usize = 0;
const ERROR_BAD_POINT: usize = 1;
// const ERROR_BAD_TWEAK: usize = 2;
// const ERROR_BAD_HASH: usize = 3;
const ERROR_BAD_SIGNATURE: usize = 4;
// const ERROR_BAD_EXTRA_DATA: usize = 5;

static mut CONTEXT_BUFFER: [u8; 1114320] = [0; 1114320];
static mut CONTEXT_SEED: [u8; 32] = [0; 32];

#[no_mangle]
pub static mut PRIVATE_INPUT: [u8; PRIVATE_KEY_SIZE] = [0; PRIVATE_KEY_SIZE];
#[no_mangle]
pub static mut PUBLIC_KEY_INPUT: [u8; PUBLIC_KEY_UNCOMPRESSED_SIZE] =
    [0; PUBLIC_KEY_UNCOMPRESSED_SIZE];
#[no_mangle]
pub static PUBLIC_KEY_INPUT2: [u8; PUBLIC_KEY_UNCOMPRESSED_SIZE] =
    [0; PUBLIC_KEY_UNCOMPRESSED_SIZE];
#[no_mangle]
pub static mut TWEAK_INPUT: [u8; TWEAK_SIZE] = [0; TWEAK_SIZE];
#[no_mangle]
pub static HASH_INPUT: [u8; HASH_SIZE] = [0; HASH_SIZE];
#[no_mangle]
pub static EXTRA_DATA_INPUT: [u8; EXTRA_DATA_SIZE] = [0; EXTRA_DATA_SIZE];
#[no_mangle]
pub static mut SIGNATURE_INPUT: [u8; SIGNATURE_SIZE] = [0; SIGNATURE_SIZE];

type InvalidInputResult<T> = Result<T, usize>;

macro_rules! jstry {
    ($value:expr) => {
        jstry!($value, ())
    };
    ($value:expr, $ret:expr) => {
        match $value {
            Ok(value) => value,
            Err(code) => {
                throw_error(code);
                return $ret;
            }
        }
    };
}

fn initialize_context_seed() {
    unsafe {
        for offset in (0..8).map(|v| v * 4) {
            let value = generate_int32();
            let bytes: [u8; 4] = core::mem::transmute(value);
            CONTEXT_SEED[offset..offset + 4].copy_from_slice(&bytes);
        }
    }
}

fn get_context() -> *const Context {
    static mut CONTEXT: *const Context = core::ptr::null();
    unsafe {
        if CONTEXT_SEED[0] == 0 {
            initialize_context_seed();
            CONTEXT = Context::create(
                CONTEXT_BUFFER.as_mut_ptr(),
                CONTEXT_BUFFER.len(),
                CONTEXT_SEED.as_ptr(),
            );
            CONTEXT_SEED[0] = 1;
            CONTEXT_SEED[1..].fill(0);
        }
        CONTEXT
    }
}

unsafe fn pubkey_parse(input: *const u8, inputlen: usize) -> InvalidInputResult<PublicKey> {
    let mut pk = PublicKey::new();
    if secp256k1_ec_pubkey_parse(secp256k1_context_no_precomp, &mut pk, input, inputlen) == 1 {
        Ok(pk)
    } else {
        Err(ERROR_BAD_POINT)
    }
}

unsafe fn pubkey_serialize(pk: &PublicKey, output: *mut u8, mut outputlen: usize) {
    let flags = if outputlen == PUBLIC_KEY_COMPRESSED_SIZE {
        SECP256K1_SER_COMPRESSED
    } else {
        SECP256K1_SER_UNCOMPRESSED
    };
    assert_eq!(
        secp256k1_ec_pubkey_serialize(
            secp256k1_context_no_precomp,
            output,
            &mut outputlen,
            pk.as_ptr() as *const PublicKey,
            flags,
        ),
        1
    );
}

#[no_mangle]
#[export_name = "initializeContext"]
pub extern "C" fn initialize_context() {
    get_context();
}

#[no_mangle]
#[export_name = "isPoint"]
pub extern "C" fn is_point(inputlen: usize) -> usize {
    unsafe { pubkey_parse(PUBLIC_KEY_INPUT.as_ptr(), inputlen).map_or_else(|_error| 0, |_pk| 1) }
}

#[no_mangle]
#[export_name = "pointAdd"]
pub extern "C" fn point_add(inputlen: usize, inputlen2: usize, outputlen: usize) -> i32 {
    unsafe {
        let pk1 = jstry!(pubkey_parse(PUBLIC_KEY_INPUT.as_ptr(), inputlen), 0);
        let pk2 = jstry!(pubkey_parse(PUBLIC_KEY_INPUT2.as_ptr(), inputlen2), 0);
        let mut pk = PublicKey::new();
        let ptrs = [pk1.as_ptr(), pk2.as_ptr()];
        if secp256k1_ec_pubkey_combine(
            secp256k1_context_no_precomp,
            &mut pk,
            ptrs.as_ptr() as *const *const PublicKey,
            ptrs.len() as i32,
        ) == 1
        {
            pubkey_serialize(&pk, PUBLIC_KEY_INPUT.as_mut_ptr(), outputlen);
            1
        } else {
            0
        }
    }
}

#[no_mangle]
#[export_name = "pointAddScalar"]
pub extern "C" fn point_add_scalar(inputlen: usize, outputlen: usize) -> i32 {
    unsafe {
        let mut pk = jstry!(pubkey_parse(PUBLIC_KEY_INPUT.as_ptr(), inputlen), 0);
        if secp256k1_ec_pubkey_tweak_add(
            get_context(),
            pk.as_mut_ptr() as *mut PublicKey,
            TWEAK_INPUT.as_ptr(),
        ) == 1
        {
            pubkey_serialize(&pk, PUBLIC_KEY_INPUT.as_mut_ptr(), outputlen);
            1
        } else {
            0
        }
    }
}

#[no_mangle]
#[export_name = "pointCompress"]
pub extern "C" fn point_compress(inputlen: usize, outputlen: usize) {
    unsafe {
        let pk = jstry!(pubkey_parse(PUBLIC_KEY_INPUT.as_ptr(), inputlen));
        pubkey_serialize(&pk, PUBLIC_KEY_INPUT.as_mut_ptr(), outputlen);
    }
}

#[no_mangle]
#[export_name = "pointFromScalar"]
pub extern "C" fn point_from_scalar(outputlen: usize) -> i32 {
    unsafe {
        let mut pk = PublicKey::new();
        if secp256k1_ec_pubkey_create(get_context(), &mut pk, PRIVATE_INPUT.as_ptr()) == 1 {
            pubkey_serialize(&pk, PUBLIC_KEY_INPUT.as_mut_ptr(), outputlen);
            1
        } else {
            0
        }
    }
}

#[no_mangle]
#[export_name = "pointMultiply"]
pub extern "C" fn point_multiply(inputlen: usize, outputlen: usize) -> i32 {
    unsafe {
        let mut pk = jstry!(pubkey_parse(PUBLIC_KEY_INPUT.as_ptr(), inputlen), 0);
        if secp256k1_ec_pubkey_tweak_mul(get_context(), &mut pk, TWEAK_INPUT.as_ptr()) == 1 {
            pubkey_serialize(&pk, PUBLIC_KEY_INPUT.as_mut_ptr(), outputlen);
            1
        } else {
            0
        }
    }
}

#[no_mangle]
#[export_name = "privateAdd"]
pub extern "C" fn private_add() -> i32 {
    unsafe {
        if secp256k1_ec_seckey_tweak_add(
            secp256k1_context_no_precomp,
            PRIVATE_INPUT.as_mut_ptr(),
            TWEAK_INPUT.as_ptr(),
        ) == 1
        {
            1
        } else {
            0
        }
    }
}

#[no_mangle]
#[export_name = "privateSub"]
pub extern "C" fn private_sub() -> i32 {
    unsafe {
        assert_eq!(
            secp256k1_ec_seckey_negate(secp256k1_context_no_precomp, TWEAK_INPUT.as_mut_ptr()),
            1
        );
        if secp256k1_ec_seckey_tweak_add(
            secp256k1_context_no_precomp,
            PRIVATE_INPUT.as_mut_ptr(),
            TWEAK_INPUT.as_ptr(),
        ) == 1
        {
            1
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn sign(extra_data: i32) {
    unsafe {
        let mut sig = Signature::new();
        let noncedata = if extra_data == 0 {
            core::ptr::null()
        } else {
            EXTRA_DATA_INPUT.as_ptr()
        } as *const c_void;

        assert_eq!(
            secp256k1_ecdsa_sign(
                get_context(),
                &mut sig,
                HASH_INPUT.as_ptr(),
                PRIVATE_INPUT.as_ptr(),
                secp256k1_nonce_function_rfc6979,
                noncedata
            ),
            1
        );

        assert_eq!(
            secp256k1_ecdsa_signature_serialize_compact(
                secp256k1_context_no_precomp,
                SIGNATURE_INPUT.as_mut_ptr(),
                &sig,
            ),
            1
        );
    }
}

#[no_mangle]
pub extern "C" fn verify(inputlen: usize, strict: i32) -> i32 {
    unsafe {
        let pk = jstry!(pubkey_parse(PUBLIC_KEY_INPUT.as_ptr(), inputlen), 0);

        let mut signature = Signature::new();
        if secp256k1_ecdsa_signature_parse_compact(
            secp256k1_context_no_precomp,
            &mut signature,
            SIGNATURE_INPUT.as_ptr(),
        ) == 0
        {
            throw_error(ERROR_BAD_SIGNATURE);
            return 0;
        }

        if strict == 0 {
            secp256k1_ecdsa_signature_normalize(
                secp256k1_context_no_precomp,
                &mut signature,
                &signature,
            );
        }

        if secp256k1_ecdsa_verify(get_context(), &signature, HASH_INPUT.as_ptr(), &pk) == 1 {
            1
        } else {
            0
        }
    }
}
