use crate::state::BatchStateManager;

use super::error::L1SyncError;
use super::event_manager::EventManager;

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use std::sync::Arc;

pub enum SyncCommand {
    Initialize(String, String, oneshot::Sender<()>),
    TriggerSync(oneshot::Sender<()>),
}

pub struct L1SyncService {
    command_sender: mpsc::Sender<SyncCommand>,
    //event_manager_handle: JoinHandle<()>,
}

impl L1SyncService {
    pub async fn new(batch_service: Arc<BatchStateManager>) -> Self {
        let (tx, rx) = mpsc::channel(32);
        let event_manager = EventManager::new(batch_service.clone());

        let _handle = tokio::spawn(async move {
            run_event_manager(rx, event_manager).await;
        });

        L1SyncService { command_sender: tx }
    }

    pub async fn initialize(
        &self,
        rpc_url: String,
        contract_hash: String,
    ) -> Result<(), L1SyncError> {
        let (tx, rx) = oneshot::channel();
        self.command_sender
            .send(SyncCommand::Initialize(rpc_url, contract_hash, tx))
            .await
            .map_err(|e| L1SyncError::BrokenChannel(format!("Unable to send initialize: {}", e)))?;
        rx.await.map_err(|e| {
            L1SyncError::BrokenChannel(format!("Unable to receive initialize ack: {}", e))
        })?;

        Ok(())
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
        SyncCommand::Initialize(rpc_url, contract_hash, completion_ack) => {
            event_manager.initialize(&rpc_url, &contract_hash).await?;
            let _ = completion_ack.send(());
        }
        SyncCommand::TriggerSync(completion_ack) => {
            event_manager.process_new_events().await?;
            let _ = completion_ack.send(());
        }
    }

    Ok(())
}
