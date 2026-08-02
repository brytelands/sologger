//!# sologger-idl-decoder
//!
//!**Overview**
//!
//!Decodes what Anchor programs emit — `Program data:` events and `custom program error`
//!codes — into structured form, driven purely by the program's IDL JSON. Supports both
//!the legacy (pre-0.30) and the 0.30+ IDL spec, and depends on serde rather than
//!anchor-lang so it stays small and WASM-friendly.
//!
//!**Example Usage**
//!
//!```rust
//!    let mut registry = IdlRegistry::new();
//!    registry.insert_json("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C", idl_json)?;
//!
//!    // log_contexts from LogContext::parse_logs / the transformer crates
//!    registry.enrich_all(&mut log_contexts);
//!
//!    // Each matching context now carries:
//!    //   decoded_events: [r#"{"name":"SwapEvent","data":{...}}"#]
//!    //   error_name:     Some("NotApproved")   // when error_code matched the IDL
//!```
//!
//!Standalone decoding without the registry:
//!
//!```rust
//!    let idl = Idl::from_json(idl_json)?;
//!    let event = decode_event(&idl, base64_payload)?; // Option<DecodedEvent>
//!    let error = idl.lookup_error(6001);              // Option<&IdlErrorCode>
//!```

pub mod decoder;
pub mod idl;
pub mod registry;

pub use decoder::{decode_event, decode_events, event_discriminator, DecodeError, DecodedEvent};
pub use idl::{Idl, IdlErrorCode, IdlEvent};
pub use registry::IdlRegistry;

#[cfg(test)]
mod tests {
    use base64::engine::general_purpose::STANDARD as BASE64;
    use base64::Engine;
    use serde_json::json;
    use sologger_log_context::programs_selector::ProgramsSelector;
    use sologger_log_context::sologger_log_context::LogContext;

    use crate::decoder::{decode_event, decode_events, event_discriminator};
    use crate::idl::{Idl, IdlType};
    use crate::registry::IdlRegistry;

    /// Real 0.30+ spec IDL (Raydium CP-AMM), with explicit event discriminators.
    const RAYDIUM_IDL: &str = include_str!("../tests/fixtures/raydium_cp_swap_idl.json");
    /// Crafted legacy (pre-0.30) spec IDL: inline event fields, no discriminators,
    /// "publicKey" spelling, {"defined": "Name"} type references.
    const LEGACY_IDL: &str = include_str!("../tests/fixtures/legacy_anchor_idl.json");

    const RAYDIUM_PROGRAM_ID: &str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";

    fn push_pubkey(buf: &mut Vec<u8>, byte: u8) -> String {
        let key = [byte; 32];
        buf.extend_from_slice(&key);
        bs58::encode(key).into_string()
    }

    /// Borsh-encodes a Raydium SwapEvent and returns (base64 payload, expected pubkeys).
    fn encode_swap_event() -> (String, Vec<String>) {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&[64, 198, 205, 232, 38, 8, 113, 226]); // SwapEvent discriminator
        let pool_id = push_pubkey(&mut buf, 3);
        buf.extend_from_slice(&500u64.to_le_bytes()); // input_vault_before
        buf.extend_from_slice(&600u64.to_le_bytes()); // output_vault_before
        buf.extend_from_slice(&1_000_000u64.to_le_bytes()); // input_amount
        buf.extend_from_slice(&990_000u64.to_le_bytes()); // output_amount
        buf.extend_from_slice(&10u64.to_le_bytes()); // input_transfer_fee
        buf.extend_from_slice(&12u64.to_le_bytes()); // output_transfer_fee
        buf.push(1); // base_input = true
        let input_mint = push_pubkey(&mut buf, 4);
        let output_mint = push_pubkey(&mut buf, 5);
        buf.extend_from_slice(&2500u64.to_le_bytes()); // trade_fee
        buf.extend_from_slice(&100u64.to_le_bytes()); // creator_fee
        buf.push(0); // creator_fee_on_input = false
        (BASE64.encode(&buf), vec![pool_id, input_mint, output_mint])
    }

    #[test]
    fn event_discriminator_matches_anchor() {
        // Cross-checked against the explicit discriminators in the real Raydium IDL
        assert_eq!(
            event_discriminator("SwapEvent"),
            [64, 198, 205, 232, 38, 8, 113, 226]
        );
        assert_eq!(
            event_discriminator("LpChangeEvent"),
            [121, 163, 205, 201, 57, 218, 117, 60]
        );
        // sha256("event:TradeEvent")[..8], computed independently
        assert_eq!(
            event_discriminator("TradeEvent"),
            [189, 219, 127, 211, 78, 230, 97, 238]
        );
    }

    #[test]
    fn parses_both_idl_specs() {
        let raydium = Idl::from_json(RAYDIUM_IDL).unwrap();
        assert_eq!(raydium.program_name(), "raydium_cp_swap");
        assert_eq!(raydium.events.len(), 2);
        assert_eq!(raydium.lookup_error(6000).unwrap().name, "NotApproved");

        let legacy = Idl::from_json(LEGACY_IDL).unwrap();
        assert_eq!(legacy.program_name(), "legacy_demo");
        assert_eq!(legacy.events.len(), 1);
        assert!(legacy.events[0].discriminator.is_none());
        assert!(legacy.events[0].fields.is_some());
        assert_eq!(legacy.lookup_error(6001).unwrap().name, "MarketClosed");
        assert_eq!(
            legacy.lookup_error(6000).unwrap().msg.as_deref(),
            Some("The trade is invalid")
        );
        assert!(legacy.lookup_error(42).is_none());
    }

    #[test]
    fn legacy_type_spellings_parse() {
        let legacy = Idl::from_json(LEGACY_IDL).unwrap();
        let fields = legacy.events[0].fields.as_ref().unwrap();
        assert_eq!(fields[0].ty, IdlType::Pubkey); // "publicKey"
        assert_eq!(
            fields[5].ty,
            IdlType::Vec(Box::new(IdlType::Defined("Leg".to_string())))
        );
    }

    #[test]
    fn decodes_new_spec_event() {
        let idl = Idl::from_json(RAYDIUM_IDL).unwrap();
        let (payload, pubkeys) = encode_swap_event();

        let event = decode_event(&idl, &payload).unwrap().unwrap();
        assert_eq!(event.name, "SwapEvent");
        assert_eq!(event.data["pool_id"], json!(pubkeys[0]));
        assert_eq!(event.data["input_amount"], json!(1_000_000u64));
        assert_eq!(event.data["output_amount"], json!(990_000u64));
        assert_eq!(event.data["base_input"], json!(true));
        assert_eq!(event.data["input_mint"], json!(pubkeys[1]));
        assert_eq!(event.data["output_mint"], json!(pubkeys[2]));
        assert_eq!(event.data["creator_fee_on_input"], json!(false));

        // Round-trips through the JSON string form stored on LogContext
        let as_json: serde_json::Value = serde_json::from_str(&event.to_json()).unwrap();
        assert_eq!(as_json["name"], json!("SwapEvent"));
        assert_eq!(as_json["data"]["trade_fee"], json!(2500u64));
    }

    /// Borsh-encodes the legacy fixture's TradeEvent, exercising string, u128, option,
    /// vec-of-struct, enum and fixed array decoding.
    fn encode_trade_event() -> (String, String) {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&event_discriminator("TradeEvent"));
        let trader = push_pubkey(&mut buf, 7);
        buf.extend_from_slice(&5000u64.to_le_bytes()); // amount
        buf.extend_from_slice(&12345678901234567890123u128.to_le_bytes()); // price
        buf.extend_from_slice(&2u32.to_le_bytes()); // memo length
        buf.extend_from_slice(b"gm"); // memo
        buf.push(0); // referrer: None
        buf.extend_from_slice(&2u32.to_le_bytes()); // legs length
        buf.extend_from_slice(&7u16.to_le_bytes()); // legs[0].market
        buf.extend_from_slice(&9u32.to_le_bytes()); // legs[0].qty
        buf.extend_from_slice(&8u16.to_le_bytes()); // legs[1].market
        buf.extend_from_slice(&10u32.to_le_bytes()); // legs[1].qty
        buf.push(1); // side: Sell
        buf.extend_from_slice(&[1, 2, 3, 4]); // tags [u8; 4]
        (BASE64.encode(&buf), trader)
    }

    #[test]
    fn decodes_legacy_spec_event() {
        let idl = Idl::from_json(LEGACY_IDL).unwrap();
        let (payload, trader) = encode_trade_event();

        let event = decode_event(&idl, &payload).unwrap().unwrap();
        assert_eq!(event.name, "TradeEvent");
        assert_eq!(event.data["trader"], json!(trader));
        assert_eq!(event.data["amount"], json!(5000u64));
        assert_eq!(event.data["price"], json!("12345678901234567890123"));
        assert_eq!(event.data["memo"], json!("gm"));
        assert_eq!(event.data["referrer"], serde_json::Value::Null);
        assert_eq!(
            event.data["legs"],
            json!([{"market": 7, "qty": 9}, {"market": 8, "qty": 10}])
        );
        assert_eq!(event.data["side"], json!("Sell"));
        assert_eq!(event.data["tags"], json!([1, 2, 3, 4]));
    }

    #[test]
    fn unknown_discriminator_returns_none() {
        let idl = Idl::from_json(RAYDIUM_IDL).unwrap();
        let payload = BASE64.encode([0u8; 24]);
        assert!(decode_event(&idl, &payload).unwrap().is_none());
        // Shorter than a discriminator: also None, not an error
        assert!(decode_event(&idl, &BASE64.encode([1u8, 2]))
            .unwrap()
            .is_none());
    }

    #[test]
    fn malformed_payloads_error() {
        let idl = Idl::from_json(RAYDIUM_IDL).unwrap();
        // Not base64 at all
        assert!(decode_event(&idl, "not-base64!!!").is_err());
        // Valid discriminator but truncated payload
        let truncated = BASE64.encode([64, 198, 205, 232, 38, 8, 113, 226, 1, 2, 3]);
        assert!(decode_event(&idl, &truncated).is_err());
        // decode_events skips both instead of failing
        let decoded = decode_events(
            &idl,
            &[
                "not-base64!!!".to_string(),
                truncated,
                encode_swap_event().0,
            ],
        );
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].name, "SwapEvent");
    }

    #[test]
    fn length_prefix_beyond_payload_errors() {
        let idl = Idl::from_json(LEGACY_IDL).unwrap();
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&event_discriminator("TradeEvent"));
        buf.extend_from_slice(&[7u8; 32]); // trader
        buf.extend_from_slice(&5000u64.to_le_bytes()); // amount
        buf.extend_from_slice(&1u128.to_le_bytes()); // price
        buf.extend_from_slice(&u32::MAX.to_le_bytes()); // memo length: absurd
        let result = decode_event(&idl, &BASE64.encode(&buf));
        assert!(result.is_err());
    }

    #[test]
    fn registry_enriches_log_contexts() {
        let mut registry = IdlRegistry::new();
        registry
            .insert_json(RAYDIUM_PROGRAM_ID, RAYDIUM_IDL)
            .unwrap();
        assert_eq!(registry.len(), 1);

        let (payload, _) = encode_swap_event();
        let logs: Vec<String> = vec![
            format!("Program {} invoke [1]", RAYDIUM_PROGRAM_ID),
            "Program log: Instruction: SwapBaseInput".to_string(),
            format!("Program data: {}", payload),
            format!(
                "Program {} failed: custom program error: 0x1770",
                RAYDIUM_PROGRAM_ID
            ),
        ];
        let mut log_contexts = LogContext::parse_logs(
            &logs,
            "".to_string(),
            &ProgramsSelector::new_all_programs(),
            1,
            "sig".to_string(),
        );

        registry.enrich_all(&mut log_contexts);

        assert_eq!(log_contexts.len(), 1);
        assert_eq!(log_contexts[0].decoded_events.len(), 1);
        let event: serde_json::Value =
            serde_json::from_str(&log_contexts[0].decoded_events[0]).unwrap();
        assert_eq!(event["name"], json!("SwapEvent"));
        assert_eq!(event["data"]["input_amount"], json!(1_000_000u64));
        assert_eq!(log_contexts[0].error_code, Some(6000));
        assert_eq!(log_contexts[0].error_name.as_deref(), Some("NotApproved"));

        // The enriched context serializes with the new fields present
        let json = log_contexts[0].to_json();
        assert!(json.contains("\"decoded_events\""));
        assert!(json.contains("\"error_name\":\"NotApproved\""));
    }

    #[test]
    fn registry_skips_unregistered_programs() {
        let mut registry = IdlRegistry::new();
        registry
            .insert_json(RAYDIUM_PROGRAM_ID, RAYDIUM_IDL)
            .unwrap();

        let (payload, _) = encode_swap_event();
        let logs: Vec<String> = vec![
            "Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 invoke [1]".to_string(),
            format!("Program data: {}", payload),
            "Program 9RX7oz3WN5VRTqekBBHBvEJFVMNRnrCmVy7S6B6S5oU7 success".to_string(),
        ];
        let mut log_contexts = LogContext::parse_logs(
            &logs,
            "".to_string(),
            &ProgramsSelector::new_all_programs(),
            1,
            "sig".to_string(),
        );

        registry.enrich_all(&mut log_contexts);
        assert!(log_contexts[0].decoded_events.is_empty());
        assert!(log_contexts[0].error_name.is_none());
    }

    #[test]
    fn registry_rejects_invalid_idl_json() {
        let mut registry = IdlRegistry::new();
        assert!(registry.insert_json("SomeProgram", "{ not json").is_err());
        assert!(registry.is_empty());
    }
}
