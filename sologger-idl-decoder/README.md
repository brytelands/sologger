# sologger-idl-decoder

Decodes what Anchor programs emit — `Program data:` events and `custom program error`
codes — into structured form, driven purely by the program's IDL JSON.

- Supports both the legacy (pre-0.30) and the 0.30+ Anchor IDL spec, including the
  differing discriminator layouts (explicit `discriminator` arrays vs
  `sha256("event:<Name>")[..8]`).
- Parses the IDL with serde instead of depending on `anchor-lang`, keeping the
  dependency tree small and WASM-friendly.
- Enriches `LogContext` records from `sologger-log-context` in place: `data_logs` become
  `decoded_events`, and `error_code` resolves to `error_name` via the IDL's `errors`
  array.

```rust
let mut registry = IdlRegistry::new();
registry.insert_json("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C", idl_json)?;

// log_contexts from LogContext::parse_logs / the transformer crates
registry.enrich_all(&mut log_contexts);
```

Used by the `sologger` binary (via the `idls` map in `sologger-config.json`) and by
`sologger-log-transformer-wasm` for the browser UI.
