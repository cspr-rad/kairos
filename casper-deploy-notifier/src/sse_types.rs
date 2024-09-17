use casper_types::{
    execution::ExecutionResult,
    contract_messages::Messages,
        TransactionHash,
        ProtocolVersion,
        InitiatorAddr,
        BlockHash,
};
use serde::{Deserialize, Serialize};

/// Casper does not expose SSE types directly, so we have to reimplement them.
///
/// Source: https://github.com/casper-network/casper-node/blob/release-2.0.0-rc4/node/src/components/event_stream_server/sse_server.rs
///
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum SseData {
    /// The version of this node's API server. This event will always be the first sent to a new
    /// client, and will have no associated event ID provided.
    ApiVersion(ProtocolVersion),
    /// The given transaction has been executed, committed and forms part of the given block.
    TransactionProcessed(TransactionProcessed),
    /// The node is about to shut down.
    Shutdown,
    /// Other events, that we are not interested in.
    #[serde(untagged)]
    Other(serde_json::Value),
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct TransactionProcessed {
        pub transaction_hash: Box<TransactionHash>,
        pub initiator_addr: Box<InitiatorAddr>,
        pub timestamp: String,
        pub ttl: String,
        pub block_hash: Box<BlockHash>,
        pub execution_result: Box<ExecutionResult>,
        pub messages: Messages,
}
