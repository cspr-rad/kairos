use casper_types::AsymmetricType;

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
            casper_types::ExecutionResult::Failure { .. } => false,
            casper_types::ExecutionResult::Success { .. } => true,
        };
        let deploy_hash = base16::encode_lower(event_details.deploy_hash.as_bytes());
        let public_key = event_details.account.to_hex();

        Notification {
            deploy_hash,
            public_key,
            success,
        }
    }
}
