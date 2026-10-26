//! Browser bindings for `rhythm-tree`.

use serde::Serialize;
use wasm_bindgen::prelude::*;

mod view;

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
    Ok(value.serialize(&serde_wasm_bindgen::Serializer::json_compatible())?)
}

/// Decodes a link or URL: `{ issues: { level, msg }[] }`.
#[wasm_bindgen(js_name = openLink)]
pub fn open_link(input: &str) -> Result<JsValue, JsError> {
    to_js(&view::open_of(input))
}

/// The groove a link holds, as the client draws it.
#[wasm_bindgen]
pub fn view(link: &str) -> Result<JsValue, JsError> {
    to_js(&view::view_of(link))
}
