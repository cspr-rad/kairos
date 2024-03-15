use tokio::sync::mpsc;

use casper_deploy_notifier::DeployNotifier;

/// Example integration with Casper Deploy Notifier.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = mpsc::channel(100);
    let mut deploy_notifier = DeployNotifier::default();

    tokio::spawn(async move {
        if let Err(e) = deploy_notifier.run(tx).await {
            eprintln!("Error listening for deployment events: {:?}", e);
        }
    });

    while let Some(message) = rx.recv().await {
        println!("{:?}", message);
    }

    Ok(())
}
