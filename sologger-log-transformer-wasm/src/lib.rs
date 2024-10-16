//!# sologger-log-transformer
//!
//!**Overview**
//!
//!This library provides utility to extract logs from various Solana API structs, such as blocks, transactions and responses.
//!
//!**Example Usage**
//!
//!```javascript
//!    const logs = ["Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 invoke [1]",
//!         "Program log: Instruction: Initialize",
//!         "Program 11111111111111111111111111111111 invoke [2]",
//!         "Program 11111111111111111111111111111111 success",
//!         "Program log: Initialized new event. Current value",
//!         "Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 consumed 59783 of 200000 compute units",
//!         "Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 success",
//!         "Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL invoke [1]",
//!         "Program log: Create",
//!         "Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL consumed 5475 of 200000 compute units",
//!         "Program failed to complete: Invoked an instruction with data that is too large (12178014311288245306 > 10240)",
//!         "Program AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL failed: Program failed to complete"
//!     ];
//! 
//!     const transformer = new WasmLogContextTransformer(["9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7", "AbcdefGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"]);
//! 
//!     // Example usage of from_rpc_logs_response
//!     const rpcLogsResponse = {
//!         signature: "test_signature",
//!         err: null,
//!         logs: logs
//!     };
//!     const slot = 123456789;
//! 
//!     try {
//!         const result = transformer.from_rpc_logs_response(rpcLogsResponse, BigInt(slot));
//!         console.log(JSON.parse(JSON.stringify(result)));
//!     } catch (error) {
//!         console.error("Error:", error);
//!     }
//! 
//!     // Example usage of from_rpc_response
//!     const rpcResponse = {
//!         context: {
//!             slot: 12324,
//!             api_version: null
//!         },
//!         value: rpcLogsResponse
//!     };
//! 
//!     try {
//!         const result = transformer.from_rpc_response(rpcResponse);
//!         console.log(JSON.parse(JSON.stringify(result)));
//!     } catch (error) {
//!         console.error("Error:", error);
//!     }
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

