use claim_core::{
    evaluation,
    graph::{Document, ReviewRequest},
};
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
pub fn analyze(input: &str) -> Result<String, JsValue> {
    let d = Document::parse(input).map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_json::to_string(&d.graph().map_err(|e| JsValue::from_str(&e.to_string()))?)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
#[wasm_bindgen]
pub fn review(input: &str, request: &str) -> Result<String, JsValue> {
    if request.len() > 10000 {
        return Err(JsValue::from_str("review request too large"));
    }
    let d = Document::parse(input).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let r: ReviewRequest =
        claim_core::strict::parse(request).map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_json::to_string(&d.review(r).map_err(|e| JsValue::from_str(&e.to_string()))?)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
#[wasm_bindgen]
pub fn evaluate(input: &str, gold: &str) -> Result<String, JsValue> {
    if gold.len() > 500000 {
        return Err(JsValue::from_str("gold too large"));
    }
    let d = Document::parse(input).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let pairs: Vec<evaluation::GoldPair> =
        claim_core::strict::parse(gold).map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_json::to_string(
        &evaluation::evaluate(&d, &pairs).map_err(|e| JsValue::from_str(&e.to_string()))?,
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))
}
#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;
    #[wasm_bindgen_test]
    fn corpus_in_wasm() {
        let b = claim_core::Bundle::parse(include_str!("../../../examples/corpus.json")).unwrap();
        let doc = serde_json::to_string(&Document::new(b)).unwrap();
        let result: serde_json::Value = serde_json::from_str(
            &evaluate(&doc, include_str!("../../../examples/gold.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(result["correct"], 30);
        assert!(analyze(&doc).is_ok());
    }
}
