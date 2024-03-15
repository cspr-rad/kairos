use serde::{Deserialize, Serialize};

///
/// NOTE: Casper does not expose SSE types directly, so we have to reimplement them.
///

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum ExecutionResult {
    Success(serde_json::Value),
    Failure(serde_json::Value),
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum SseData {
    /// The version of node's API.
    ApiVersion(String),
    /// The given deploy has been executed, committed and forms part of the given block.
    DeployProcessed {
        deploy_hash: String,
        account: String,
        execution_result: ExecutionResult,
    },
    /// Other events, that we are not interested in.
    #[serde(untagged)]
    Other(serde_json::Value),
}
