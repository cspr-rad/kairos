use casper_sse_client::SseListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut listener = SseListener::default();
    listener.run().await?;

    Ok(())
}
