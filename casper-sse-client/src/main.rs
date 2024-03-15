use casper_sse_client::listen_to_sse;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    listen_to_sse().await?;

    Ok(())
}
