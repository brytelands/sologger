//! 5.3 Historical backfill: replays past transactions of the selected programs through
//! the normal pipeline (getSignaturesForAddress → getTransaction → parse → enrich →
//! export), turning sologger into a post-mortem tool rather than only a live tail.
//! Driven by the optional `backfill` block in sologger-config.json.

use anyhow::Result;
use log::{info, warn};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_rpc_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_rpc_client_api::config::{CommitmentConfig, RpcTransactionConfig};
use solana_rpc_client_api::response::RpcConfirmedTransactionStatusWithSignature;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_transaction_status::UiTransactionEncoding;
use sologger_log_context::programs_selector::ProgramsSelector;
use sologger_log_transformer::log_context_transformer::from_encoded_confirmed_transaction;
use std::str::FromStr;
use std::time::Duration;

use crate::log_subscriber::LogPipeline;
use crate::sologger_config::BackfillConfig;

/// getSignaturesForAddress returns at most 1000 entries per call.
const SIGNATURE_PAGE_LIMIT: usize = 1000;

/// Replays historical transactions for every selected program, oldest first. Requires
/// explicit programs in the selector — the RPC has no "all programs" history API.
pub(crate) async fn run(
    backfill_config: &BackfillConfig,
    program_selector: &ProgramsSelector,
    pipeline: &LogPipeline,
) -> Result<()> {
    let Some(rpc_client) = &pipeline.rpc_client else {
        warn!("[backfill] no HTTP RPC client available; skipping backfill");
        return Ok(());
    };
    if program_selector.select_all_programs || program_selector.programs.is_empty() {
        warn!(
            "[backfill] historical backfill requires explicit programs in programsSelector; skipping"
        );
        return Ok(());
    }

    for program_bytes in &program_selector.programs {
        let program_id = bs58::encode(program_bytes).into_string();
        let Ok(address) = Pubkey::from_str(&program_id) else {
            warn!("[backfill] {} is not a valid address; skipping", program_id);
            continue;
        };

        let signatures =
            collect_signatures(rpc_client, &address, backfill_config).await?;
        info!(
            "[backfill] {}: replaying {} transactions",
            program_id,
            signatures.len()
        );

        // Signatures arrive newest-first; replay in chronological order
        let mut replayed = 0usize;
        for status in signatures.into_iter().rev() {
            tokio::time::sleep(Duration::from_millis(backfill_config.throttle_ms)).await;
            let Ok(signature) = Signature::from_str(&status.signature) else {
                continue;
            };
            let config = RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Json),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            };
            match rpc_client.get_transaction_with_config(&signature, config).await {
                Ok(transaction) => {
                    let slot = transaction.slot;
                    match from_encoded_confirmed_transaction(
                        &transaction,
                        slot,
                        program_selector,
                    ) {
                        Ok(log_contexts) => {
                            replayed += 1;
                            pipeline.process(log_contexts).await;
                        }
                        Err(err) => {
                            warn!("[backfill] failed to parse {}: {}", status.signature, err)
                        }
                    }
                }
                Err(err) => warn!(
                    "[backfill] getTransaction {} failed: {}",
                    status.signature, err
                ),
            }
        }
        info!("[backfill] {}: replayed {} transactions", program_id, replayed);
    }
    Ok(())
}

/// Pages through getSignaturesForAddress (newest first) until the configured limit or
/// slot floor is reached, filtering by the optional slot range.
async fn collect_signatures(
    rpc_client: &RpcClient,
    address: &Pubkey,
    backfill_config: &BackfillConfig,
) -> Result<Vec<RpcConfirmedTransactionStatusWithSignature>> {
    let mut collected: Vec<RpcConfirmedTransactionStatusWithSignature> = Vec::new();
    let mut before: Option<Signature> = None;

    'paging: while collected.len() < backfill_config.limit {
        let page_limit = SIGNATURE_PAGE_LIMIT.min(backfill_config.limit - collected.len());
        let config = GetConfirmedSignaturesForAddress2Config {
            before,
            until: None,
            limit: Some(page_limit),
            commitment: Some(CommitmentConfig::confirmed()),
        };
        let page = rpc_client
            .get_signatures_for_address_with_config(address, config)
            .await?;
        if page.is_empty() {
            break;
        }
        let page_len = page.len();
        before = page
            .last()
            .and_then(|status| Signature::from_str(&status.signature).ok());

        for status in page {
            // Newest-first: once below the slot floor, everything further back is too old
            if let Some(from_slot) = backfill_config.from_slot {
                if status.slot < from_slot {
                    break 'paging;
                }
            }
            if let Some(until_slot) = backfill_config.until_slot {
                if status.slot > until_slot {
                    continue;
                }
            }
            collected.push(status);
            if collected.len() >= backfill_config.limit {
                break 'paging;
            }
        }
        if page_len < page_limit {
            break;
        }
    }
    Ok(collected)
}
