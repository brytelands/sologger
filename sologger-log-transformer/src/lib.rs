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

pub mod log_context_transformer;

//TODO provide error mapping
