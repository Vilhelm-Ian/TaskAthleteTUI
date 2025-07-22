// src/app/sync_actions.rs

use anyhow::Result;
use std::path::PathBuf;
use task_athlete_lib::{AppService, Config, SyncSummary};
use tokio::sync::mpsc;
use tracing::{error, info};

pub enum SyncResult {
    Success(SyncSummary, SyncSummary),
    Error(String),
}

pub async fn sync_operation_background(
    // These arguments are passed by value and are Send/Sync
    db_path: PathBuf,
    config_path: PathBuf,
    mut initial_config: Config, // Taking Config by value
    sender: mpsc::Sender<SyncResult>,
) {
    info!("Starting background sync operation...");

    // This entire block runs within the spawned Tokio task.
    // Here, we create an AppService instance that owns its own Rusqlite connection,
    // ensuring it's not shared across thread boundaries from the main UI thread.
    let sync_result: Result<SyncResult> = (|| async {
        // Create a new AppService instance. This involves opening a new rusqlite::Connection
        // which is then owned by this `service` variable, local to this async task.
        let mut service = AppService::initialize()
            .map_err(|e| anyhow::anyhow!("Failed to initialize AppService for sync: {}", e))?;

        // IMPORTANT: Update the service's config with the one passed from the main thread.
        // This ensures any unsaved config changes (e.g., sync server URL) are used.
        // The `service.config` field itself is `Clone`, so this assignment works.
        service.config = initial_config;

        match service.perform_sync(None).await {
            Ok((pushed_summary, pulled_summary)) => {
                info!("Sync completed successfully.");
                // `service` (and its internal rusqlite connection) will be dropped here,
                // closing the connection for this task.
                Ok(SyncResult::Success(pushed_summary, pulled_summary))
            }
            Err(e) => {
                error!("Sync failed: {:?}", e);
                Ok(SyncResult::Error(format!("Sync failed: {}", e)))
            }
        }
    })()
    .await; // Await the inner async block

    // Send the result back to the main thread
    if let Err(e) = sender
        .send(sync_result.unwrap_or_else(|e: anyhow::Error| SyncResult::Error(e.to_string())))
        .await
    {
        error!("Failed to send sync result: {}", e);
    }
}
