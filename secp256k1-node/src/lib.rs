#[macro_use]
extern crate napi_derive;

use napi::{CallContext, JsBoolean, JsObject, JsString, Result as NapiResult};

#[module_exports]
fn init(mut exports: JsObject) -> NapiResult<()> {
    exports.create_named_method("isPoint", is_point)?;

    Ok(())
}

#[js_function(1)]
fn is_point(ctx: CallContext) -> NapiResult<JsBoolean> {
    // unsafe { pubkey_parse(PUBLIC_KEY_INPUT.as_ptr(), inputlen).map_or_else(|_error| 0, |_pk| 1) }
    ctx.env.get_boolean(false)
}
