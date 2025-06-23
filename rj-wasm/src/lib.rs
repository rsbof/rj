use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn format(input: &str) -> Result<String, JsError> {
    let formatted = rj::format(input)?;
    Ok(formatted)
}

#[wasm_bindgen]
pub fn parse(input: &str) -> Result<String, JsError> {
    let parsed = rj::parse(input)?;
    Ok(format!("{:#?}", parsed))
}
