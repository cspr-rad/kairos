use casper_types::{
    execution::{ExecutionResult, ExecutionResultV1},
};

use crate::sse_types::TransactionProcessed;

#[derive(Debug)]
pub struct Notification {
    pub deploy_hash: String,
    pub public_key: String,
    pub success: bool,
}

impl From<TransactionProcessed> for Notification {
    fn from(event_details: TransactionProcessed) -> Self {
        let success = match *event_details.execution_result {
            ExecutionResult::V1(execution_result_v1) => match execution_result_v1 {
                ExecutionResultV1::Failure { .. } => false,
                ExecutionResultV1::Success { .. } => true,
            },
            ExecutionResult::V2(execution_result_v2) => execution_result_v2.error_message.is_none(),
        };
        let deploy_hash = base16::encode_lower(&event_details.transaction_hash.digest());
        let public_key = event_details.initiator_addr.account_hash().to_string();

        Notification {
            deploy_hash,
            public_key,
            success,
        }
    }
}
