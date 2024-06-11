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
    command_sender: Option<mpsc::Sender<SyncCommand>>,
    //event_manager_handle: Option<tokio::task::JoinHandle<()>>,
    server_state: Arc<ServerStateInner>,
}

impl L1SyncService {
    pub fn new(server_state: Arc<ServerStateInner>) -> Self {
        L1SyncService {
            command_sender: None,
            //event_manager_handle: None,
            server_state,
        }
    }

    pub async fn initialize(
        &mut self,
        rpc_url: String,
        contract_hash: String,
    ) -> Result<(), L1SyncError> {
        let event_manager =
            EventManager::new(&rpc_url, &contract_hash, self.server_state.clone()).await?;

        let (tx, rx) = mpsc::channel(32);
        self.command_sender = Some(tx);
        let _handle = tokio::spawn(async move {
            run_event_manager(rx, event_manager).await;
        });
        //self.event_manager_handle = Some(handle);

        Ok(())
    }

    pub async fn trigger_sync(&self) -> Result<(), L1SyncError> {
        let command_sender = self.command_sender.as_ref().ok_or_else(|| {
            L1SyncError::InitializationError("Command sender not available".to_string())
        })?;

        let (tx, rx) = oneshot::channel();
        command_sender
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
