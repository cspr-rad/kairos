use deadpool_diesel::postgres::Pool;
use crate::domain::models::deposits;
use bigdecimal::BigDecimal;
use chrono::{Utc, NaiveDateTime};
use tokio::time::{sleep, Duration};
use crate::CONFIG;

use kairos_risc0_types::Deposit;
use layer_one_utils::deployments::query;

// Check for deposits, starts with index = 0, grows with on-chain index
// When on-chain index > local query L1, get deposits, add to local storage with processed = False
pub async fn sync(pool: Pool) {
    let mut deposit_index: u64 = 0;
    loop {
        let onchain_index = query::query_counter(&CONFIG.node_address(), &CONFIG.node.port.to_string(), &CONFIG.node.counter_uref).await;

        if onchain_index == 0 {
            continue
        }
        
        for i in deposit_index..onchain_index {
            let deposit = query::query_deposit(&CONFIG.node_address(), &CONFIG.node.port.to_string(), &CONFIG.node.dict_uref, i.to_string()).await;
            deposit_index += 1;
            deposits::insert(pool.clone(), deposit);
        }
        sleep(Duration::from_secs(10));
    }
}