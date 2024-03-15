use casper_sse_client::DeployNotifier;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut listener = DeployNotifier::default();
    listener.run().await?;

    Ok(())
}
