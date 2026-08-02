use crate::log_processor::log_contexts_from_logs;
use crate::sologger_config::SologgerConfig;
use anyhow::Result;
use futures_util::StreamExt;
use log::trace;
use solana_pubsub_client::nonblocking::pubsub_client::PubsubClient;
use solana_rpc_client_api::config::{
    CommitmentConfig, CommitmentLevel, RpcTransactionLogsConfig, RpcTransactionLogsFilter,
};
use sologger_idl_decoder::IdlRegistry;
use sologger_log_context::programs_selector::ProgramsSelector;
use sologger_log_transformer::log_context_transformer::from_rpc_response;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::unbounded_channel;

#[cfg(feature = "solana_client_subscriber")]
pub async fn start_client(
    sologger_config: &SologgerConfig,
    program_selector: &ProgramsSelector,
    idl_registry: &IdlRegistry,
) -> Result<()> {
    trace!("{:?}", &program_selector);
    // Subscription tasks will send a ready signal when they have subscribed.
    let (ready_sender, mut ready_receiver) = unbounded_channel::<()>();

    // Channel to receive unsubscribe channels (actually closures).
    // These receive a pair of `(Box<dyn FnOnce() -> BoxFuture<'static, ()> + Send>), &'static str)`,
    // where the first is a closure to call to unsubscribe, the second is the subscription name.
    let (unsubscribe_sender, mut unsubscribe_receiver) = unbounded_channel::<(_, String)>();

    let url = &sologger_config.rpc_url;
    // The `PubsubClient` must be `Arc`ed to share it across tasks.
    // TODO look into the potential of creating a PubsubClient with a custom WebSocketConfig for finer tuning.
    let pubsub_client = Arc::new(PubsubClient::new(url).await?);

    let mut join_handles = Vec::with_capacity(program_selector.programs.len());

    let all_log_filter: RpcTransactionLogsFilter = if sologger_config.all_with_votes {
        RpcTransactionLogsFilter::AllWithVotes
    } else {
        RpcTransactionLogsFilter::All
    };

    let mut log_filters: HashMap<String, RpcTransactionLogsFilter> =
        HashMap::with_capacity(program_selector.programs.len());
    if program_selector.select_all_programs {
        log_filters.insert("all".to_string(), all_log_filter);
    } else {
        for program_id in &program_selector.programs {
            let program_key = bs58::encode(program_id).into_string();
            log_filters.insert(
                program_key.clone(),
                RpcTransactionLogsFilter::Mentions(vec![program_key]),
            );
        }
    }

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
    trace!("log_filters: {:?}", log_filters);

    #[cfg(feature = "enable_tokio_rt_metrics")]
    enable_tokio_rt_metrics();

    for (program_key, log_filter) in log_filters {
        trace!("starting subscribe for key {}", &program_key);
        join_handles.push((
            program_key.clone(),
            tokio::spawn({
                // Clone things we need before moving their clones into the `async move` block.
                //
                // The subscriptions have to be made from the tasks that will receive the subscription messages,
                // because the subscription streams hold a reference to the `PubsubClient`.
                // Otherwise we would just subscribe on the main task and send the receivers out to other tasks.

                let ready_sender = ready_sender.clone();
                let unsubscribe_sender = unsubscribe_sender.clone();
                let pubsub_client = Arc::clone(&pubsub_client);
                let program_key = program_key.clone();
                let program_selector = Arc::new(program_selector.clone());
                let idl_registry = Arc::new(idl_registry.clone());
                async move {
                    let (mut log_notifications, log_unsubscribe) = pubsub_client
                        .logs_subscribe(
                            log_filter,
                            RpcTransactionLogsConfig {
                                commitment: commitment_config,
                            },
                        )
                        .await?;

                    // With the subscription started,
                    // send a signal back to the main task for synchronization.
                    ready_sender.send(()).expect("channel");

                    // Send the unsubscribe closure back to the main task.
                    unsubscribe_sender
                        .send((log_unsubscribe, program_key))
                        .map_err(|e| format!("{}", e))
                        .expect("channel");

                    // Drop senders so that the channels can close.
                    // The main task will receive until channels are closed.
                    drop((ready_sender, unsubscribe_sender));

                    // Do something with the subscribed messages.
                    // This loop will end once the main task unsubscribes.
                    while let Some(log_info) = log_notifications.next().await {
                        let mut log_contexts = from_rpc_response(&log_info, &program_selector)
                            .expect("Error getting log contexts from RPC response");

                        // Decode Anchor events / resolve error names for programs with a configured IDL
                        idl_registry.enrich_all(&mut log_contexts);

                        // Export transaction traces / metrics when enabled in the OTel config
                        #[cfg(feature = "enable_otel")]
                        crate::telemetry::export(&log_contexts);

                        log_contexts_from_logs(&log_contexts)
                            .await
                            .expect("Failed to log from log contexts");
                    }

                    // This type hint is necessary to allow the `async move` block to use `?`.
                    Ok::<_, anyhow::Error>(())
                }
            }),
        ));
    }

    // Drop these senders so that the channels can close
    // and their receivers return `None` below.
    drop(ready_sender);
    drop(unsubscribe_sender);

    // Wait until all subscribers are ready before proceeding with application logic.
    while (ready_receiver.recv().await).is_some() {}

    // Do application logic here.

    // Wait for input or some application-specific shutdown condition.
    tokio::io::stdin().read_u8().await?;

    // Unsubscribe from everything, which will shutdown all the tasks.
    while let Some((unsubscribe, name)) = unsubscribe_receiver.recv().await {
        trace!("unsubscribing from {}", name);
        unsubscribe().await
    }

    // Wait for the tasks.
    for (name, handle) in join_handles {
        trace!("waiting on task {}", name);
        if let Ok(Err(e)) = handle.await {
            trace!("task {} failed: {}", name, e);
        }
    }

    Ok(())
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
    use sologger_log_context::programs_selector::ProgramsSelector;
    use std::collections::HashMap;

    fn create_test_config() -> SologgerConfig {
        SologgerConfig {
            rpc_url: "wss://test.solana.com".to_string(),
            log4rs_config_location: "test.yml".to_string(),
            opentelemetry_config_location: "test.json".to_string(),
            all_with_votes: false,
            commitment_level: None,
        }
    }

    fn create_test_program_selector() -> ProgramsSelector {
        ProgramsSelector::new(&[
            "11111111111111111111111111111111".to_string(),
            "22222222222222222222222222222222".to_string(),
        ])
    }

    fn create_test_program_selector_all() -> ProgramsSelector {
        ProgramsSelector::new_all_programs()
    }

    #[test]
    fn test_log_filters_creation_specific_programs() {
        let config = create_test_config();
        let program_selector = create_test_program_selector();

        // Simulate the log filter creation logic from start_client
        let all_log_filter: RpcTransactionLogsFilter = if config.all_with_votes {
            RpcTransactionLogsFilter::AllWithVotes
        } else {
            RpcTransactionLogsFilter::All
        };

        let mut log_filters: HashMap<String, RpcTransactionLogsFilter> =
            HashMap::with_capacity(program_selector.programs.len());

        if program_selector.select_all_programs {
            log_filters.insert("all".to_string(), all_log_filter);
        } else {
            for program_id in &program_selector.programs {
                let program_key = bs58::encode(program_id).into_string();
                log_filters.insert(
                    program_key.clone(),
                    RpcTransactionLogsFilter::Mentions(vec![program_key]),
                );
            }
        }

        assert_eq!(log_filters.len(), 2);
        assert!(log_filters.contains_key("11111111111111111111111111111111"));
        assert!(log_filters.contains_key("22222222222222222222222222222222"));

        if let Some(RpcTransactionLogsFilter::Mentions(mentions)) =
            log_filters.get("11111111111111111111111111111111")
        {
            assert_eq!(
                mentions,
                &vec!["11111111111111111111111111111111".to_string()]
            );
        } else {
            panic!("Expected Mentions filter");
        }
    }

    #[test]
    fn test_log_filters_creation_all_programs() {
        let config = create_test_config();
        let program_selector = create_test_program_selector_all();

        // Simulate the log filter creation logic from start_client
        let all_log_filter: RpcTransactionLogsFilter = if config.all_with_votes {
            RpcTransactionLogsFilter::AllWithVotes
        } else {
            RpcTransactionLogsFilter::All
        };

        let mut log_filters: HashMap<String, RpcTransactionLogsFilter> =
            HashMap::with_capacity(program_selector.programs.len());

        if program_selector.select_all_programs {
            log_filters.insert("all".to_string(), all_log_filter);
        } else {
            for program_id in &program_selector.programs {
                let program_key = bs58::encode(program_id).into_string();
                log_filters.insert(
                    program_key.clone(),
                    RpcTransactionLogsFilter::Mentions(vec![program_key]),
                );
            }
        }

        assert_eq!(log_filters.len(), 1);
        assert!(log_filters.contains_key("all"));

        if let Some(RpcTransactionLogsFilter::All) = log_filters.get("all") {
            // Expected All filter
        } else {
            panic!("Expected All filter");
        }
    }

    #[test]
    fn test_log_filters_creation_with_votes() {
        let mut config = create_test_config();
        config.all_with_votes = true;
        let program_selector = create_test_program_selector_all();

        // Simulate the log filter creation logic from start_client
        let all_log_filter: RpcTransactionLogsFilter = if config.all_with_votes {
            RpcTransactionLogsFilter::AllWithVotes
        } else {
            RpcTransactionLogsFilter::All
        };

        let mut log_filters: HashMap<String, RpcTransactionLogsFilter> =
            HashMap::with_capacity(program_selector.programs.len());

        if program_selector.select_all_programs {
            log_filters.insert("all".to_string(), all_log_filter);
        } else {
            for program_id in &program_selector.programs {
                let program_key = bs58::encode(program_id).into_string();
                log_filters.insert(
                    program_key.clone(),
                    RpcTransactionLogsFilter::Mentions(vec![program_key]),
                );
            }
        }

        assert_eq!(log_filters.len(), 1);
        assert!(log_filters.contains_key("all"));

        if let Some(RpcTransactionLogsFilter::AllWithVotes) = log_filters.get("all") {
            // Expected AllWithVotes filter
        } else {
            panic!("Expected AllWithVotes filter");
        }
    }

    #[test]
    fn test_commitment_config_creation_none() {
        let config = create_test_config();

        // Simulate the commitment config creation logic from start_client
        let commitment_config = match &config.commitment_level {
            Some(level) => {
                let commitment_level = CommitmentLevel::from_str(level).unwrap();
                Some(CommitmentConfig {
                    commitment: commitment_level,
                })
            }
            None => None,
        };

        assert!(commitment_config.is_none());
    }

    #[test]
    fn test_commitment_config_creation_finalized() {
        let mut config = create_test_config();
        config.commitment_level = Some("finalized".to_string());

        // Simulate the commitment config creation logic from start_client
        let commitment_config = match &config.commitment_level {
            Some(level) => {
                let commitment_level = CommitmentLevel::from_str(level).unwrap();
                Some(CommitmentConfig {
                    commitment: commitment_level,
                })
            }
            None => None,
        };

        assert!(commitment_config.is_some());
        let config = commitment_config.unwrap();
        assert_eq!(config.commitment, CommitmentLevel::Finalized);
    }

    #[test]
    fn test_commitment_config_creation_confirmed() {
        let mut config = create_test_config();
        config.commitment_level = Some("confirmed".to_string());

        // Simulate the commitment config creation logic from start_client
        let commitment_config = match &config.commitment_level {
            Some(level) => {
                let commitment_level = CommitmentLevel::from_str(level).unwrap();
                Some(CommitmentConfig {
                    commitment: commitment_level,
                })
            }
            None => None,
        };

        assert!(commitment_config.is_some());
        let config = commitment_config.unwrap();
        assert_eq!(config.commitment, CommitmentLevel::Confirmed);
    }

    #[test]
    fn test_commitment_config_creation_processed() {
        let mut config = create_test_config();
        config.commitment_level = Some("processed".to_string());

        // Simulate the commitment config creation logic from start_client
        let commitment_config = match &config.commitment_level {
            Some(level) => {
                let commitment_level = CommitmentLevel::from_str(level).unwrap();
                Some(CommitmentConfig {
                    commitment: commitment_level,
                })
            }
            None => None,
        };

        assert!(commitment_config.is_some());
        let config = commitment_config.unwrap();
        assert_eq!(config.commitment, CommitmentLevel::Processed);
    }

    #[cfg(feature = "enable_tokio_rt_metrics")]
    #[tokio::test]
    async fn test_enable_tokio_rt_metrics() {
        // This test verifies that the enable_tokio_rt_metrics function doesn't panic
        // Since it spawns a background task, we can't easily test its output
        // But we can verify it doesn't crash
        enable_tokio_rt_metrics();

        // Give the spawned task a moment to start
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // If we reach here without panicking, the test passes
        assert!(true);
    }
}
