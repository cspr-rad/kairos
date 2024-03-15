use tokio::sync::mpsc;

use casper_sse_client::DeployNotifier;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = mpsc::channel(100);
    let mut listener = DeployNotifier::default();

    tokio::spawn(async move {
        if let Err(e) = listener.run(tx).await {
            eprintln!("Error listening for events: {:?}", e);
        }
    });

    while let Some(message) = rx.recv().await {
        println!("TODO: {:?}", message);
    }

    Ok(())
}
