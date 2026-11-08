//! WASM bindings for the Estragon layout core.
//!
//! JSON in, JSON out: keeps the boundary trivial while the schema is in flux.

use wasm_bindgen::prelude::*;

/// Lay out a `.ui.json` document. Returns a JSON array of
/// `{ id, depth, x, y, width, height }` in document coordinates.
#[wasm_bindgen(js_name = layoutDocument)]
pub fn layout_document(document_json: &str) -> Result<String, JsError> {
    let doc: estragon_layout::schema::Document = serde_json::from_str(document_json)?;
    let placed = estragon_layout::layout_document(&doc).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(serde_json::to_string(&placed)?)
}
