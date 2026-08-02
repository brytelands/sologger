//! Live ingestion: supervised WebSocket subscriptions (logsSubscribe or blockSubscribe)
//! that auto-reconnect with exponential backoff, detect slot gaps across reconnects,
//! and run every parsed batch through the shared [`LogPipeline`] — truncation backfill,
//! IDL enrichment, telemetry/webhook export, and the configured log transport.

use crate::log_processor::log_contexts_from_logs;
use crate::sologger_config::{LogSource, SologgerConfig};
use anyhow::Result;
use futures_util::StreamExt;
use log::{info, trace, warn};
use solana_pubsub_client::nonblocking::pubsub_client::PubsubClient;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_rpc_client_api::config::{
    CommitmentConfig, CommitmentLevel, RpcBlockSubscribeConfig, RpcBlockSubscribeFilter,
    RpcTransactionConfig, RpcTransactionLogsConfig, RpcTransactionLogsFilter,
};
use solana_sdk::signature::Signature;
use solana_transaction_status::{TransactionDetails, UiTransactionEncoding};
use sologger_idl_decoder::IdlRegistry;
use sologger_log_context::programs_selector::ProgramsSelector;
use sologger_log_context::sologger_log_context::LogContext;
use sologger_log_transformer::log_context_transformer::{
    from_encoded_confirmed_transaction, from_rpc_response, from_ui_confirmed_block,
};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;

const INITIAL_BACKOFF: Duration = Duration::from_secs(1);
const MAX_BACKOFF: Duration = Duration::from_secs(30);

/// Everything needed to take a freshly parsed batch to the transports. Shared by the
/// live subscription tasks and the historical backfill.
pub(crate) struct LogPipeline {
    pub program_selector: ProgramsSelector,
    pub idl_registry: IdlRegistry,
    pub rpc_client: Option<RpcClient>,
    pub backfill_truncated: bool,
}

impl LogPipeline {
    /// Runs one parsed batch through truncation backfill, IDL enrichment, telemetry and
    /// webhook export, and the log transport.
    pub(crate) async fn process(&self, log_contexts: Vec<LogContext>) {
        if log_contexts.is_empty() {
            return;
        }
        let mut log_contexts = if self.backfill_truncated {
            self.refetch_truncated(log_contexts).await
        } else {
            log_contexts
        };

        // Decode Anchor events / resolve error names for programs with a configured IDL
        self.idl_registry.enrich_all(&mut log_contexts);

        // Export transaction traces / metrics when enabled in the OTel config
        #[cfg(feature = "enable_otel")]
        crate::telemetry::export(&log_contexts);

        // POST matching records to the configured webhook, off the hot path
        #[cfg(feature = "enable_webhook")]
        crate::webhook_sender::dispatch(&log_contexts);

        if let Err(err) = log_contexts_from_logs(&log_contexts).await {
            warn!("failed to ship log contexts: {}", err);
        }
    }

    /// 5.2 Truncation backfill: when a transaction's logs arrived truncated, fetch the
    /// stored transaction over HTTP and re-parse it. Falls back to the truncated
    /// contexts on any failure.
    async fn refetch_truncated(&self, log_contexts: Vec<LogContext>) -> Vec<LogContext> {
        let Some(rpc_client) = &self.rpc_client else {
            return log_contexts;
        };
        if !log_contexts
            .iter()
            .any(|context| context.invoke_result == "Log truncated")
        {
            return log_contexts;
        }

        // Batches from blockSubscribe can span transactions: resolve per signature group
        let mut resolved = Vec::with_capacity(log_contexts.len());
        let mut group: Vec<LogContext> = Vec::new();
        for context in log_contexts {
            if group
                .last()
                .is_some_and(|last| last.signature != context.signature)
            {
                let finished = std::mem::take(&mut group);
                resolved.extend(self.resolve_group(rpc_client, finished).await);
            }
            group.push(context);
        }
        resolved.extend(self.resolve_group(rpc_client, group).await);
        resolved
    }

    async fn resolve_group(
        &self,
        rpc_client: &RpcClient,
        group: Vec<LogContext>,
    ) -> Vec<LogContext> {
        if !group
            .iter()
            .any(|context| context.invoke_result == "Log truncated")
        {
            return group;
        }
        let signature_str = group[0].signature.clone();
        let Ok(signature) = Signature::from_str(&signature_str) else {
            return group;
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
                    &self.program_selector,
                ) {
                    Ok(full) if !full.is_empty() => {
                        info!("backfilled truncated logs for {}", signature_str);
                        full
                    }
                    Ok(_) => group,
                    Err(err) => {
                        warn!("re-parse of {} failed: {}; keeping truncated logs", signature_str, err);
                        group
                    }
                }
            }
            Err(err) => {
                warn!(
                    "getTransaction for truncated {} failed: {}; keeping truncated logs",
                    signature_str, err
                );
                group
            }
        }
    }
}

/// What one supervised task subscribes to.
#[derive(Clone, Debug)]
enum SubscriptionKind {
    Logs(RpcTransactionLogsFilter),
    Block(RpcBlockSubscribeFilter),
}

#[cfg(feature = "solana_client_subscriber")]
pub async fn start_client(
    sologger_config: &SologgerConfig,
    program_selector: &ProgramsSelector,
    idl_registry: &IdlRegistry,
) -> Result<()> {
    trace!("{:?}", &program_selector);

    let commitment_config = match &sologger_config.commitment_level {
        Some(level) => {
            let commitment_level = CommitmentLevel::from_str(level).unwrap();
            Some(CommitmentConfig {
                commitment: commitment_level,
            })
        }
        None => None,
    };
    trace!("commitment_config: {:?}", commitment_config);

    // The HTTP client is only needed for the backfill paths
    let rpc_client = (sologger_config.backfill_truncated || sologger_config.backfill.is_some())
        .then(|| RpcClient::new(sologger_config.http_url()));

    let pipeline = Arc::new(LogPipeline {
        program_selector: program_selector.clone(),
        idl_registry: idl_registry.clone(),
        rpc_client,
        backfill_truncated: sologger_config.backfill_truncated,
    });

    // 5.3 Historical backfill, before the live tail starts
    if let Some(backfill_config) = &sologger_config.backfill {
        crate::backfill::run(backfill_config, program_selector, &pipeline).await?;
        if backfill_config.exit_after {
            info!("backfill finished; exitAfter is set, shutting down");
            return Ok(());
        }
    }

    #[cfg(feature = "enable_tokio_rt_metrics")]
    enable_tokio_rt_metrics();

    let subscriptions = build_subscriptions(sologger_config, program_selector);
    trace!("subscriptions: {:?}", subscriptions);

    let mut join_handles = Vec::with_capacity(subscriptions.len());
    for (key, kind) in subscriptions {
        join_handles.push(tokio::spawn(supervise_subscription(
            sologger_config.rpc_url.clone(),
            key,
            kind,
            commitment_config,
            Arc::clone(&pipeline),
        )));
    }

    // Wait for input or some application-specific shutdown condition.
    tokio::io::stdin().read_u8().await?;

    // The supervisors loop forever; aborting them drops the clients, which unsubscribes.
    for handle in join_handles {
        handle.abort();
    }

    Ok(())
}

/// One subscription per selected program (or a single "all" subscription), for the
/// configured source.
fn build_subscriptions(
    sologger_config: &SologgerConfig,
    program_selector: &ProgramsSelector,
) -> Vec<(String, SubscriptionKind)> {
    let mut subscriptions = Vec::new();
    match sologger_config.source {
        LogSource::LogsSubscribe => {
            if program_selector.select_all_programs {
                let filter = if sologger_config.all_with_votes {
                    RpcTransactionLogsFilter::AllWithVotes
                } else {
                    RpcTransactionLogsFilter::All
                };
                subscriptions.push(("all".to_string(), SubscriptionKind::Logs(filter)));
            } else {
                for program_id in &program_selector.programs {
                    let program_key = bs58::encode(program_id).into_string();
                    subscriptions.push((
                        program_key.clone(),
                        SubscriptionKind::Logs(RpcTransactionLogsFilter::Mentions(vec![
                            program_key,
                        ])),
                    ));
                }
            }
        }
        LogSource::BlockSubscribe => {
            if program_selector.select_all_programs {
                subscriptions.push((
                    "all".to_string(),
                    SubscriptionKind::Block(RpcBlockSubscribeFilter::All),
                ));
            } else {
                for program_id in &program_selector.programs {
                    let program_key = bs58::encode(program_id).into_string();
                    subscriptions.push((
                        program_key.clone(),
                        SubscriptionKind::Block(RpcBlockSubscribeFilter::MentionsAccountOrProgram(
                            program_key,
                        )),
                    ));
                }
            }
        }
    }
    subscriptions
}

/// 5.1 Reconnect supervisor: owns one subscription, reconnecting forever with
/// exponential backoff. The backoff resets once a connection delivers messages.
async fn supervise_subscription(
    url: String,
    key: String,
    kind: SubscriptionKind,
    commitment: Option<CommitmentConfig>,
    pipeline: Arc<LogPipeline>,
) {
    let mut backoff = INITIAL_BACKOFF;
    let mut last_seen_slot: Option<u64> = None;
    let mut is_reconnect = false;

    loop {
        if is_reconnect {
            #[cfg(feature = "enable_otel")]
            crate::telemetry::record_reconnect();
        }
        match connect_and_stream(
            &url,
            &key,
            &kind,
            commitment,
            &pipeline,
            &mut last_seen_slot,
            is_reconnect,
        )
        .await
        {
            Ok(messages) => {
                info!("[{}] subscription stream ended after {} messages", key, messages);
                if messages > 0 {
                    backoff = INITIAL_BACKOFF;
                }
            }
            Err(err) => warn!("[{}] subscription error: {}", key, err),
        }
        is_reconnect = true;
        warn!("[{}] reconnecting in {:?}", key, backoff);
        tokio::time::sleep(backoff).await;
        backoff = next_backoff(backoff);
    }
}

fn next_backoff(current: Duration) -> Duration {
    (current * 2).min(MAX_BACKOFF)
}

/// The gap a reconnect left behind: slots that passed between the last message of the
/// old connection and the first message of the new one.
fn slot_gap(last_seen: u64, first_new: u64) -> Option<u64> {
    (first_new > last_seen + 1).then(|| first_new - last_seen - 1)
}

/// Connects, subscribes, and pumps the stream until it ends. Returns how many
/// notifications were processed on this connection.
async fn connect_and_stream(
    url: &str,
    key: &str,
    kind: &SubscriptionKind,
    commitment: Option<CommitmentConfig>,
    pipeline: &LogPipeline,
    last_seen_slot: &mut Option<u64>,
    is_reconnect: bool,
) -> Result<u64> {
    let client = PubsubClient::new(url).await?;
    let mut processed: u64 = 0;
    let mut gap_checked = false;

    let observe_slot = |slot: u64, last_seen_slot: &mut Option<u64>, gap_checked: &mut bool| {
        if !*gap_checked {
            *gap_checked = true;
            if is_reconnect {
                if let Some(last_seen) = *last_seen_slot {
                    if let Some(missed) = slot_gap(last_seen, slot) {
                        warn!(
                            "[{}] possible gap after reconnect: slots {}..{} ({} slots) passed while disconnected",
                            key,
                            last_seen + 1,
                            slot - 1,
                            missed
                        );
                        #[cfg(feature = "enable_otel")]
                        crate::telemetry::record_slot_gap(missed);
                    }
                }
            }
        }
        *last_seen_slot = Some(slot.max(last_seen_slot.unwrap_or(0)));
    };

    match kind {
        SubscriptionKind::Logs(filter) => {
            let (mut notifications, _unsubscribe) = client
                .logs_subscribe(
                    filter.clone(),
                    RpcTransactionLogsConfig {
                        commitment,
                    },
                )
                .await?;
            info!("[{}] subscribed via logsSubscribe", key);

            while let Some(response) = notifications.next().await {
                observe_slot(response.context.slot, last_seen_slot, &mut gap_checked);
                match from_rpc_response(&response, &pipeline.program_selector) {
                    Ok(log_contexts) => {
                        processed += 1;
                        pipeline.process(log_contexts).await;
                    }
                    Err(err) => warn!("[{}] failed to parse notification: {}", key, err),
                }
            }
        }
        SubscriptionKind::Block(filter) => {
            let config = RpcBlockSubscribeConfig {
                commitment,
                encoding: Some(UiTransactionEncoding::Json),
                transaction_details: Some(TransactionDetails::Full),
                show_rewards: Some(false),
                max_supported_transaction_version: Some(0),
            };
            let (mut notifications, _unsubscribe) =
                client.block_subscribe(filter.clone(), Some(config)).await?;
            info!("[{}] subscribed via blockSubscribe", key);

            while let Some(response) = notifications.next().await {
                let slot = response.value.slot;
                observe_slot(slot, last_seen_slot, &mut gap_checked);
                let Some(block) = response.value.block else {
                    continue;
                };
                if block.transactions.is_none() {
                    continue;
                }
                match from_ui_confirmed_block(block, slot, &pipeline.program_selector) {
                    Ok(log_contexts) => {
                        processed += 1;
                        pipeline.process(log_contexts).await;
                    }
                    Err(err) => warn!("[{}] failed to parse block {}: {}", key, slot, err),
                }
            }
        }
    }

    Ok(processed)
}

#[cfg(feature = "enable_tokio_rt_metrics")]
fn enable_tokio_rt_metrics() {
    let handle = tokio::runtime::Handle::current();
    let runtime_monitor = tokio_metrics::RuntimeMonitor::new(&handle);

    let frequency = std::time::Duration::from_millis(1000);
    tokio::spawn(async move {
        for metrics in runtime_monitor.intervals() {
            println!("Metrics = {:?}", metrics);
            tokio::time::sleep(frequency).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sologger_config::LogSource;

    fn create_test_config() -> SologgerConfig {
        SologgerConfig {
            rpc_url: "wss://test.solana.com".to_string(),
            log4rs_config_location: "test.yml".to_string(),
            opentelemetry_config_location: "test.json".to_string(),
            ..Default::default()
        }
    }

    fn create_test_program_selector() -> ProgramsSelector {
        ProgramsSelector::new(&[
            "11111111111111111111111111111111".to_string(),
            "22222222222222222222222222222222".to_string(),
        ])
    }

    #[test]
    fn test_build_subscriptions_specific_programs() {
        let config = create_test_config();
        let selector = create_test_program_selector();

        let subscriptions = build_subscriptions(&config, &selector);
        assert_eq!(subscriptions.len(), 2);
        let keys: Vec<&String> = subscriptions.iter().map(|(key, _)| key).collect();
        assert!(keys.contains(&&"11111111111111111111111111111111".to_string()));
        assert!(keys.contains(&&"22222222222222222222222222222222".to_string()));
        for (key, kind) in &subscriptions {
            match kind {
                SubscriptionKind::Logs(RpcTransactionLogsFilter::Mentions(mentions)) => {
                    assert_eq!(mentions, &vec![key.clone()]);
                }
                other => panic!("expected Mentions filter, got {:?}", other),
            }
        }
    }

    #[test]
    fn test_build_subscriptions_all_programs() {
        let config = create_test_config();
        let selector = ProgramsSelector::new_all_programs();

        let subscriptions = build_subscriptions(&config, &selector);
        assert_eq!(subscriptions.len(), 1);
        assert_eq!(subscriptions[0].0, "all");
        assert!(matches!(
            subscriptions[0].1,
            SubscriptionKind::Logs(RpcTransactionLogsFilter::All)
        ));
    }

    #[test]
    fn test_build_subscriptions_all_with_votes() {
        let mut config = create_test_config();
        config.all_with_votes = true;
        let selector = ProgramsSelector::new_all_programs();

        let subscriptions = build_subscriptions(&config, &selector);
        assert!(matches!(
            subscriptions[0].1,
            SubscriptionKind::Logs(RpcTransactionLogsFilter::AllWithVotes)
        ));
    }

    #[test]
    fn test_build_subscriptions_block_source() {
        let mut config = create_test_config();
        config.source = LogSource::BlockSubscribe;

        let selector = ProgramsSelector::new_all_programs();
        let subscriptions = build_subscriptions(&config, &selector);
        assert!(matches!(
            subscriptions[0].1,
            SubscriptionKind::Block(RpcBlockSubscribeFilter::All)
        ));

        let selector = create_test_program_selector();
        let subscriptions = build_subscriptions(&config, &selector);
        assert_eq!(subscriptions.len(), 2);
        assert!(matches!(
            subscriptions[0].1,
            SubscriptionKind::Block(RpcBlockSubscribeFilter::MentionsAccountOrProgram(_))
        ));
    }

    #[test]
    fn test_next_backoff_doubles_and_caps() {
        let mut backoff = INITIAL_BACKOFF;
        backoff = next_backoff(backoff);
        assert_eq!(backoff, Duration::from_secs(2));
        for _ in 0..10 {
            backoff = next_backoff(backoff);
        }
        assert_eq!(backoff, MAX_BACKOFF);
    }

    #[test]
    fn test_slot_gap() {
        assert_eq!(slot_gap(100, 101), None); // contiguous
        assert_eq!(slot_gap(100, 100), None); // same slot (multiple txs)
        assert_eq!(slot_gap(100, 105), Some(4)); // slots 101..104 missed
    }

    #[test]
    fn test_commitment_config_creation() {
        let mut config = create_test_config();
        for (level, expected) in [
            ("finalized", CommitmentLevel::Finalized),
            ("confirmed", CommitmentLevel::Confirmed),
            ("processed", CommitmentLevel::Processed),
        ] {
            config.commitment_level = Some(level.to_string());
            let commitment_config = match &config.commitment_level {
                Some(level) => {
                    let commitment_level = CommitmentLevel::from_str(level).unwrap();
                    Some(CommitmentConfig {
                        commitment: commitment_level,
                    })
                }
                None => None,
            };
            assert_eq!(commitment_config.unwrap().commitment, expected);
        }
        config.commitment_level = None;
        assert!(config.commitment_level.is_none());
    }

    #[tokio::test]
    async fn test_pipeline_without_rpc_client_passes_contexts_through() {
        let pipeline = LogPipeline {
            program_selector: ProgramsSelector::new_all_programs(),
            idl_registry: IdlRegistry::new(),
            rpc_client: None,
            backfill_truncated: true,
        };

        let logs: Vec<String> = vec![
            "Program 11111111111111111111111111111111 invoke [1]".to_string(),
            "Log truncated".to_string(),
        ];
        let parse = || {
            LogContext::parse_logs(
                &logs,
                "".to_string(),
                &ProgramsSelector::new_all_programs(),
                5,
                "SIG".to_string(),
            )
        };

        // No HTTP client configured: the truncated batch is passed through unchanged
        let resolved = pipeline.refetch_truncated(parse()).await;
        assert_eq!(resolved, parse());
    }

    #[cfg(feature = "enable_tokio_rt_metrics")]
    #[tokio::test]
    async fn test_enable_tokio_rt_metrics() {
        enable_tokio_rt_metrics();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
}
