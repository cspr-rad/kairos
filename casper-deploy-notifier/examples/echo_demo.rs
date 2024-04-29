use std::time::Duration;
use tokio::sync::mpsc;

use casper_deploy_notifier::DeployNotifier;

/// Example integration with Casper Deploy Notifier.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = mpsc::channel(100);
    let mut deploy_notifier = DeployNotifier::default();

    tokio::spawn(async move {
        loop {
            if let Err(e) = deploy_notifier.connect().await {
                eprintln!("Unable to connect: {:?}", e);
                continue;
            }

            if let Err(e) = deploy_notifier.run(tx.clone()).await {
                eprintln!("Error while listening to deployment events: {:?}", e);
            }

            // Connection can sometimes be lost, so we retry after a delay.
            eprintln!("Retrying in 5 seconds...",);
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    while let Some(message) = rx.recv().await {
        println!("{:?}", message);
    }

    Ok(())
}
