#![no_std]
#![allow(non_camel_case_types)]

pub use core::ffi::c_void;
use core::{mem::MaybeUninit, slice, str};

pub type c_int = i32;
pub type c_uchar = u8;
pub type c_uint = u32;
pub type size_t = usize;
type c_char = i8;

/// Flag for context to enable verification precomputation
pub const SECP256K1_START_VERIFY: c_uint = 1 | (1 << 8);
/// Flag for context to enable signing precomputation
pub const SECP256K1_START_SIGN: c_uint = 1 | (1 << 9);

/// Flag for keys to indicate uncompressed serialization format
#[allow(unused_parens)]
pub const SECP256K1_SER_UNCOMPRESSED: c_uint = (1 << 1);
/// Flag for keys to indicate compressed serialization format
pub const SECP256K1_SER_COMPRESSED: c_uint = (1 << 1) | (1 << 8);

pub type NonceFn = Option<
    unsafe extern "C" fn(
        nonce32: *mut c_uchar,
        msg32: *const c_uchar,
        key32: *const c_uchar,
        algo16: *const c_uchar,
        data: *mut c_void,
        attempt: c_uint,
    ) -> c_int,
>;

#[repr(C)]
pub struct Context(c_int);

impl Context {
    pub fn create(buf: *mut u8, buflen: usize, seed: *const u8) -> *const Self {
        unsafe {
            let size =
                secp256k1_context_preallocated_size(SECP256K1_START_SIGN | SECP256K1_START_VERIFY);
            assert_eq!(size, buflen);
            let ctx = secp256k1_context_preallocated_create(
                buf as *mut c_void,
                SECP256K1_START_SIGN | SECP256K1_START_VERIFY,
            );
            assert_eq!(secp256k1_context_randomize(ctx, seed), 1);
            ctx as *const Self
        }
    }
}

#[repr(C)]
pub struct PublicKey(MaybeUninit<[c_uchar; 64]>);

impl PublicKey {
    pub fn new() -> Self {
        Self(MaybeUninit::uninit())
    }

    pub fn as_ptr(&self) -> *const [c_uchar; 64] {
        self.0.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut [c_uchar; 64] {
        self.0.as_mut_ptr()
    }
}

#[repr(C)]
pub struct Signature(MaybeUninit<[c_uchar; 64]>);

impl Signature {
    pub fn new() -> Self {
        Self(MaybeUninit::uninit())
    }
}

extern "C" {
    // Nonce function
    pub static secp256k1_nonce_function_rfc6979: NonceFn;

    // Context
    pub static secp256k1_context_no_precomp: *const Context;

    pub fn secp256k1_context_preallocated_size(flags: c_uint) -> size_t;

    pub fn secp256k1_context_preallocated_create(
        prealloc: *mut c_void,
        flags: c_uint,
    ) -> *mut Context;

    pub fn secp256k1_context_randomize(cx: *mut Context, seed32: *const c_uchar) -> c_int;

    // Public Key
    pub fn secp256k1_ec_pubkey_parse(
        cx: *const Context,
        pk: *mut PublicKey,
        input: *const c_uchar,
        in_len: size_t,
    ) -> c_int;

    pub fn secp256k1_ec_pubkey_serialize(
        cx: *const Context,
        output: *mut c_uchar,
        out_len: *mut size_t,
        pk: *const PublicKey,
        compressed: c_uint,
    ) -> c_int;

    pub fn secp256k1_ec_pubkey_combine(
        cx: *const Context,
        out: *mut PublicKey,
        ins: *const *const PublicKey,
        n: c_int,
    ) -> c_int;

    pub fn secp256k1_ec_pubkey_create(
        cx: *const Context,
        pk: *mut PublicKey,
        sk: *const c_uchar,
    ) -> c_int;

    pub fn secp256k1_ec_pubkey_tweak_add(
        cx: *const Context,
        pk: *mut PublicKey,
        tweak: *const c_uchar,
    ) -> c_int;

    pub fn secp256k1_ec_pubkey_tweak_mul(
        cx: *const Context,
        pk: *mut PublicKey,
        tweak: *const c_uchar,
    ) -> c_int;

    // Private Key
    pub fn secp256k1_ec_seckey_negate(cx: *const Context, sk: *mut c_uchar) -> c_int;

    pub fn secp256k1_ec_seckey_tweak_add(
        cx: *const Context,
        sk: *mut c_uchar,
        tweak: *const c_uchar,
    ) -> c_int;

    // Signature
    pub fn secp256k1_ecdsa_signature_parse_compact(
        cx: *const Context,
        sig: *mut Signature,
        input64: *const c_uchar,
    ) -> c_int;

    pub fn secp256k1_ecdsa_signature_serialize_compact(
        cx: *const Context,
        output64: *mut c_uchar,
        sig: *const Signature,
    ) -> c_int;

    pub fn secp256k1_ecdsa_signature_normalize(
        cx: *const Context,
        out_sig: *mut Signature,
        in_sig: *const Signature,
    ) -> c_int;

    // ECDSA
    pub fn secp256k1_ecdsa_sign(
        cx: *const Context,
        sig: *mut Signature,
        msg32: *const c_uchar,
        sk: *const c_uchar,
        noncefn: NonceFn,
        noncedata: *const c_void,
    ) -> c_int;

    pub fn secp256k1_ecdsa_verify(
        cx: *const Context,
        sig: *const Signature,
        msg32: *const c_uchar,
        pk: *const PublicKey,
    ) -> c_int;
}

unsafe fn strlen(mut str_ptr: *const c_char) -> usize {
    let mut ctr = 0;
    while *str_ptr != '\0' as c_char {
        ctr += 1;
        str_ptr = str_ptr.offset(1);
    }
    ctr
}

unsafe fn handle_callback_fn(prefix: &str, message: *const c_char) {
    let msg_slice = slice::from_raw_parts(message as *const u8, strlen(message));
    let msg = str::from_utf8_unchecked(msg_slice);
    panic!("[libsecp256k1] {}: {}\n", prefix, msg);
}

#[no_mangle]
pub unsafe extern "C" fn secp256k1_default_illegal_callback_fn(
    message: *const c_char,
    _data: *mut c_void,
) {
    handle_callback_fn("illegal argument", message)
}

#[no_mangle]
pub unsafe extern "C" fn secp256k1_default_error_callback_fn(
    message: *const c_char,
    _data: *mut c_void,
) {
    handle_callback_fn("internal consistency check failed", message)
}

#[no_mangle]
pub unsafe extern "C" fn malloc(_size: size_t) -> size_t {
    panic!("malloc should not used")
}

#[no_mangle]
pub unsafe extern "C" fn free(_size: size_t) -> size_t {
    panic!("free should not used")
}
