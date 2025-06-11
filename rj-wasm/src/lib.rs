use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn format(input: &str) -> String {
    rj::format(input)
}

#[wasm_bindgen]
pub fn parse(input: &str) -> String {
    let parsed = rj::parse(input);
    format!("{:#?}", parsed)
}
