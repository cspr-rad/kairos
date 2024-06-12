use crate::state::ServerStateInner;

use super::error::L1SyncError;
use super::event_manager::EventManager;

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use std::sync::Arc;

pub enum SyncCommand {
    TriggerSync(oneshot::Sender<()>),
    // NOTE: More commands can be here.
}

pub struct L1SyncService {
    command_sender: mpsc::Sender<SyncCommand>,
    //event_manager_handle: tokio::task::JoinHandle<()>,
}

impl L1SyncService {
    pub async fn new(
        contract_hash: String,
        server_state: Arc<ServerStateInner>,
    ) -> Result<Self, L1SyncError> {
        let event_manager = EventManager::new(&contract_hash, server_state.clone()).await?;

        let (tx, rx) = mpsc::channel(32);
        let _handle = tokio::spawn(async move {
            run_event_manager(rx, event_manager).await;
        });

        Ok(L1SyncService {
            command_sender: tx,
            //event_manager_handle: _handle,
        })
    }

    pub async fn trigger_sync(&self) -> Result<(), L1SyncError> {
        let (tx, rx) = oneshot::channel();
        self.command_sender
            .send(SyncCommand::TriggerSync(tx))
            .await
            .map_err(|e| L1SyncError::BrokenChannel(format!("Unable to send trigger: {}", e)))?;
        rx.await.map_err(|e| {
            L1SyncError::BrokenChannel(format!("Unable to receive trigger ack: {}", e))
        })?;

        Ok(())
    }
}

/// Handles incoming commands and delegates tasks to EventManager.
async fn run_event_manager(mut rx: mpsc::Receiver<SyncCommand>, mut event_manager: EventManager) {
    tracing::debug!("Event manager running and waiting for commands");
    while let Some(command) = rx.recv().await {
        let _ = handle_command(command, &mut event_manager)
            .await
            .map_err(|e| match e {
                L1SyncError::UnexpectedError(e) => panic!("Unrecoverable error: {}", e),
                _ => tracing::error!("Transient error: {}", e),
            });
    }
}

async fn handle_command(
    command: SyncCommand,
    event_manager: &mut EventManager,
) -> Result<(), L1SyncError> {
    match command {
        SyncCommand::TriggerSync(completion_ack) => {
            event_manager.process_new_events().await?;
            completion_ack
                .send(())
                .map_err(|_| L1SyncError::BrokenChannel("Sender dropped".to_string()))?;
        }
    }

    Ok(())
}
