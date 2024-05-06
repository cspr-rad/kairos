use kairos_l1_utils::query_state_root_hash;
use kairos_e2e::{Test, register_test, E2EResult};

async fn state_root_hash() -> E2EResult {
    let srh = query_state_root_hash("http://127.0.0.1:11101/rpc").await;
    println!("Srh: {:?}", &srh);
    E2EResult::passed()
}

register_test!("query_state_root_hash", state_root_hash);