use casper_types::bytesrepr::FromBytes;

use kairos_l1_state_replicator::rpc::CasperClient;
use kairos_l1_state_replicator::CasperStateReplicator;

mod cep78;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = CasperClient::new_mainnet();
    let mut replicator = CasperStateReplicator::from_contract(
        client,
        "fe03021407879ce6fc5e035b70ff6a90941afdbea325a9164c7a497827efa7ff",
    );
    replicator.fetch_metadata().await?;
    replicator.fetch_schema().await;

    // Alteratively - user locally defined schemas.
    let local_schemas = cep78::schemas::get_local_schemas();
    //replicator.load_schema(local_schemas);

    replicator.fetch_events_count().await;

    // Fetch each event data.
    for event_id in 0..10 {
        let dynamic_event = replicator.fetch_event(event_id).await;
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
