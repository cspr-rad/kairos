use serde::{Deserialize, Serialize};

/// Casper does not expose SSE types directly, so we have to reimplement them.
///
/// Source: https://github.com/casper-network/casper-node/blob/9f3995853204a18f17de9c022233d22aa14b9c37/node/src/components/event_stream_server/sse_server.rs#L75.
///
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum SseData {
    /// The version of node's API.
    ApiVersion(casper_types::ProtocolVersion),
    /// The given deploy has been executed, committed and forms part of the given block.
    DeployProcessed(DeployProcessed),
    /// The node is about to shut down.
    Shutdown,
    /// Other events, that we are not interested in.
    #[serde(untagged)]
    Other(serde_json::Value),
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct DeployProcessed {
    pub deploy_hash: Box<casper_types::DeployHash>,
    pub account: Box<casper_types::PublicKey>,
    pub execution_result: Box<casper_types::ExecutionResult>,
}
