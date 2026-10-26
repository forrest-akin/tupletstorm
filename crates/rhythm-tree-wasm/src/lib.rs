//! Browser bindings for `rhythm-tree`.

use serde::Serialize;
use wasm_bindgen::prelude::*;

mod view;

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
    Ok(value.serialize(&serde_wasm_bindgen::Serializer::json_compatible())?)
}

/// Decodes a link or URL: `{ link: string, lanes: string[], issues: { level, msg }[] }`.
#[wasm_bindgen(js_name = openLink)]
pub fn open_link(input: &str) -> Result<JsValue, JsError> {
    to_js(&view::open_of(input))
}

/// The groove a link holds, with the given lanes, as the client draws it.
#[wasm_bindgen]
pub fn view(link: &str, lanes: Vec<String>) -> Result<JsValue, JsError> {
    let lanes: Vec<&str> = lanes.iter().map(String::as_str).collect();
    to_js(&view::view_of(link, &lanes))
}
