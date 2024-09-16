use casper_types::{
    execution::{ExecutionResult, ExecutionResultV1},
    AsymmetricType,
};

use crate::sse_types::DeployProcessed;

#[derive(Debug)]
pub struct Notification {
    pub deploy_hash: String,
    pub public_key: String,
    pub success: bool,
}

impl From<DeployProcessed> for Notification {
    fn from(event_details: DeployProcessed) -> Self {
        let success = match *event_details.execution_result {
            ExecutionResult::V1(execution_result_v1) => match execution_result_v1 {
                ExecutionResultV1::Failure { .. } => false,
                ExecutionResultV1::Success { .. } => true,
            },
            ExecutionResult::V2(execution_result_v2) => execution_result_v2.error_message.is_none(),
        };
        let deploy_hash = base16::encode_lower(event_details.deploy_hash.inner());
        let public_key = event_details.account.to_hex();

        Notification {
            deploy_hash,
            public_key,
            success,
        }
    }
}
