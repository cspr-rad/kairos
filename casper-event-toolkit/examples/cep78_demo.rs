use casper_types::bytesrepr::FromBytes;

use casper_event_toolkit::fetcher::Fetcher;
use casper_event_toolkit::metadata::CesMetadataRef;
use casper_event_toolkit::rpc::client::CasperClient;

mod cep78;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = CasperClient::default_mainnet();

    let metadata = CesMetadataRef::fetch_metadata(
        &client,
        "fe03021407879ce6fc5e035b70ff6a90941afdbea325a9164c7a497827efa7ff",
    )
    .await
    .unwrap();

    let fetcher = Fetcher {
        client: CasperClient::default_mainnet(),
        ces_metadata: metadata,
    };

    let schemas = fetcher.fetch_schema().await?;
    // Alteratively - user locally defined schemas.
    // let schemas = cep78::schemas::get_local_schemas();

    let num_events = fetcher.fetch_events_count().await?;
    println!("Events count: {}", num_events);

    // Fetch each event data.
    for event_id in 0..10 {
        let dynamic_event = fetcher.fetch_event(event_id, &schemas).await;
        println!("Event data parsed dynamically: {:?}", dynamic_event);

        match dynamic_event.name.as_str() {
            "Mint" => {
                let data = dynamic_event.to_ces_bytes();
                let (parsed_further, rem) = cep78::events::Mint::from_bytes(&data).unwrap();
                assert!(rem.len() == 0);
                println!("Event data parsed statically: {:?}", parsed_further);
            }
            other => {
                println!("Unknown event type: {}", other)
            }
        }

        break;
    }

    Ok(())
}
