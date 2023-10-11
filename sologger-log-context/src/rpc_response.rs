use serde_derive::{Deserialize, Serialize};

/// RPC response from the solana RPC endpoint
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub method: String,
    pub params: Params,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Params {
    pub result: Result,
    pub subscription: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Result {
    pub context: Context,
    pub value: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Context {
    pub slot: u64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Value {
    pub signature: String,
    #[serde(default)]
    pub err: Option<serde_json::Value>,
    pub logs: Vec<String>,
}
