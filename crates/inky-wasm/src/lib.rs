use wasm_bindgen::prelude::*;
use inky_core::{Inky, Config};

#[wasm_bindgen]
pub fn transform(html: &str) -> String {
    Inky::new().transform(html)
}

#[wasm_bindgen]
pub fn transform_with_config(html: &str, column_count: u32) -> String {
    let config = Config {
        column_count,
        ..Default::default()
    };
    Inky::with_config(config).transform(html)
}
