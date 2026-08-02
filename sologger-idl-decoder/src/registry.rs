use std::collections::HashMap;

use sologger_log_context::sologger_log_context::LogContext;

use crate::decoder::{decode_event, DecodeError};
use crate::idl::Idl;

/// IDLs keyed by program ID. The enrichment entry point for both the sologger binary
/// (loaded from the `idls` map in sologger-config.json) and the WASM transformer
/// (loaded via `add_idl` from the browser).
#[derive(Default, Clone, Debug)]
pub struct IdlRegistry {
    idls: HashMap<String, Idl>,
}

impl IdlRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.idls.is_empty()
    }

    pub fn len(&self) -> usize {
        self.idls.len()
    }

    /// Registers a parsed IDL for a program ID, replacing any previous one.
    pub fn insert(&mut self, program_id: impl Into<String>, idl: Idl) {
        self.idls.insert(program_id.into(), idl);
    }

    /// Parses IDL JSON (either spec version) and registers it for a program ID.
    pub fn insert_json(&mut self, program_id: &str, idl_json: &str) -> Result<(), DecodeError> {
        let idl = Idl::from_json(idl_json)?;
        self.insert(program_id, idl);
        Ok(())
    }

    pub fn get(&self, program_id: &str) -> Option<&Idl> {
        self.idls.get(program_id)
    }

    /// Enriches a LogContext in place when an IDL is registered for its program:
    /// decodes `data_logs` into `decoded_events`, and resolves `error_code` into
    /// `error_name`. A LogContext for an unregistered program is left untouched.
    pub fn enrich(&self, log_context: &mut LogContext) {
        let Some(idl) = self.idls.get(&log_context.program_id) else {
            return;
        };

        for data_log in &log_context.data_logs {
            match decode_event(idl, data_log) {
                Ok(Some(event)) => log_context.decoded_events.push(event.to_json()),
                Ok(None) => {}
                Err(err) => log::debug!(
                    "failed to decode data log for program {}: {}",
                    log_context.program_id,
                    err
                ),
            }
        }

        if log_context.error_name.is_none() {
            if let Some(code) = log_context.error_code {
                if let Some(idl_error) = idl.lookup_error(code) {
                    log_context.error_name = Some(idl_error.name.clone());
                }
            }
        }
    }

    /// Enriches every LogContext in the slice. Cheap no-op when the registry is empty.
    pub fn enrich_all(&self, log_contexts: &mut [LogContext]) {
        if self.idls.is_empty() {
            return;
        }
        for log_context in log_contexts {
            self.enrich(log_context);
        }
    }
}
