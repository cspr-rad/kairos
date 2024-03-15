use casper_sse_client::SseListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = SseListener::default();
    listener.listen_to_sse().await?;

    Ok(())
}
