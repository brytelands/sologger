//!# sologger-log-transformer
//!
//!**Overview**
//!
//!This library provides utility to extract logs from various Solana API structs, such as blocks, transactions and responses.
//!
//!**Example Usage**
//!
//!```rust
//!    //Extract logs from a Response<RpcLogsResponse> struct for all program IDs
//!    let logs_contexts = from_rpc_response(&response, &ProgramsSelector::new_all_programs()).unwrap();
//!```
//!
//!Please see the sologger-log-context crate for more information regarding LogContext.

pub mod log_context_transformer_wasm;

use wasm_bindgen::prelude::*;
use sologger_log_context::programs_selector::ProgramsSelector;
use crate::log_context_transformer_wasm::{from_rpc_logs_response, from_rpc_response, Response, RpcLogsResponse};

#[wasm_bindgen]
pub struct WasmLogContextTransformer {
    program_selector: ProgramsSelector,
}

#[wasm_bindgen]
impl WasmLogContextTransformer {
    #[wasm_bindgen(constructor)]
    pub fn new(program_ids: Vec<String>) -> Self {
        console_error_panic_hook::set_once();
        Self {
            program_selector: ProgramsSelector::new(&program_ids),
        }
    }

    #[wasm_bindgen]
    pub fn from_rpc_response(&self, response: JsValue) -> Result<JsValue, JsValue> {
        let response: Response<RpcLogsResponse> = serde_wasm_bindgen::from_value(response)?;
        let result = from_rpc_response(&response, &self.program_selector)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(serde_wasm_bindgen::to_value(&result)?)
    }

    #[wasm_bindgen]
    pub fn from_rpc_logs_response(&self, rpc_logs_response: JsValue, slot: u64) -> Result<JsValue, JsValue> {
        let rpc_logs_response: RpcLogsResponse = serde_wasm_bindgen::from_value(rpc_logs_response)?;
        let result = from_rpc_logs_response(&rpc_logs_response, slot, &self.program_selector)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(serde_wasm_bindgen::to_value(&result)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    use serde_json::json;

    // wasm_bindgen_test_configure!(run_in_browser);

    // #[wasm_bindgen_test]
    // fn test_wasm_log_context_transformer_new() {
    //     let transformer = WasmLogContextTransformer::new(vec!["11111111111111111111111111111111".to_string()]);
    //     assert!(transformer.program_selector.is_program_selected("11111111111111111111111111111111".as_ref()));
    // }

    // #[wasm_bindgen_test]
    // async fn test_from_rpc_logs_response() {
    //     let transformer = WasmLogContextTransformer::new(vec!["*".to_string()]);
    //     let rpc_logs_response = json!({
    //         "signature": "test_signature",
    //         "err": null,
    //         "logs": [
    //             "Program 11111111111111111111111111111111 invoke [1]",
    //             "Program 11111111111111111111111111111111 success"
    //         ]
    //     });
    // 
    //     let result = transformer.from_rpc_logs_response(
    //         serde_wasm_bindgen::to_value(&rpc_logs_response).unwrap(),
    //         123456789
    //     ).unwrap();
    // 
    //     let log_contexts: Vec<serde_json::Value> = serde_wasm_bindgen::from_value(result).unwrap();
    //     assert_eq!(log_contexts.len(), 1);
    //     assert_eq!(log_contexts[0]["program_id"], "11111111111111111111111111111111");
    //     assert_eq!(log_contexts[0]["slot"], 123456789);
    // }
    // 
    // #[wasm_bindgen_test]
    // async fn test_from_rpc_response() {
    //     let transformer = WasmLogContextTransformer::new(vec!["*".to_string()]);
    //     let rpc_response = json!({
    //         "context": {
    //             "slot": 12324,
    //             "api_version": null
    //         },
    //         "value": {
    //             "signature": "test_signature",
    //             "err": null,
    //             "logs": [
    //                 "Program 11111111111111111111111111111111 invoke [1]",
    //                 "Program 11111111111111111111111111111111 success"
    //             ]
    //         }
    //     });
    // 
    //     let result = transformer.from_rpc_response(
    //         serde_wasm_bindgen::to_value(&rpc_response).unwrap()
    //     ).unwrap();
    // 
    //     let log_contexts: Vec<serde_json::Value> = serde_wasm_bindgen::from_value(result).unwrap();
    //     assert_eq!(log_contexts.len(), 1);
    //     assert_eq!(log_contexts[0]["program_id"], "11111111111111111111111111111111");
    //     assert_eq!(log_contexts[0]["slot"], 12324);
    // }
    // 
    #[wasm_bindgen_test]
    async fn test_error_handling() {
        let transformer = WasmLogContextTransformer::new(vec!["*".to_string()]);
        let invalid_response = json!({
            "invalid": "data"
        });
    
        let result = transformer.from_rpc_response(
            serde_wasm_bindgen::to_value(&invalid_response).unwrap()
        );
    
        assert!(result.is_err());
    }
}

