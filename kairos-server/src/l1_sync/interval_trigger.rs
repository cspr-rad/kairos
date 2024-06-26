use tokio::time::{self, Duration};

use std::sync::Arc;

use super::service::L1SyncService;

pub async fn run(sync_service: Arc<L1SyncService>) {
    let mut interval = time::interval(Duration::from_secs(30));

    loop {
        interval.tick().await;

        tracing::debug!("Triggering periodic L1 sync");
        let _ = sync_service.trigger_sync().await.map_err(|e| {
            tracing::error!("Unable to trigger sync: {}", e);
        });
    }
}
