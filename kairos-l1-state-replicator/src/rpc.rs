use casper_client::{rpcs::results::GetDeployResult, types::DeployHash, JsonRpcId, Verbosity};

const DEFAULT_MAINNET_RPC: &str = "https://mainnet.casper-node.xyz/rpc";
const DEFAULT_TESTNET_RPC: &str = "https://testnet.casper-node.xyz/rpc";

pub struct CasperClient {
    rpc_endpoint: String,
}

impl CasperClient {
    pub fn new_mainnet() -> Self {
        Self {
            rpc_endpoint: DEFAULT_MAINNET_RPC.into(),
        }
    }

    pub fn new_testnet() -> Self {
        Self {
            rpc_endpoint: DEFAULT_TESTNET_RPC.into(),
        }
    }

    pub fn from_address(rpc_endpoint: &str) -> Self {
        Self {
            rpc_endpoint: rpc_endpoint.to_string(),
        }
    }

    // ID for each RPC request.
    //
    // NOTE: We use arbitrary fixed value, because casper_client does NOT use
    // request batching, so ID is not really important.
    fn get_rpc_id(&self) -> JsonRpcId {
        1.into()
    }

    // Verbosity for each RPC request.
    //
    // NOTE: We do not want any extra output to be printed, so we use lowest
    // possible level.
    fn get_verbosity(&self) -> Verbosity {
        casper_client::Verbosity::Low
    }

    // Fetch deployment by its hash.
    //
    // TODO: check if really used.
    pub async fn get_deploy(&self, deploy_hash: &str) -> GetDeployResult {
        // Build deploy hash.
        let deploy_hash_bytes = hex::decode(deploy_hash).unwrap();
        let deploy_hash_bytes: [u8; 32] = deploy_hash_bytes.try_into().unwrap();
        let deploy_hash = DeployHash::new(deploy_hash_bytes.into());

        // We are not interested in getting finalization approvals.
        let finalized_approvals = false;

        let response = casper_client::get_deploy(
            self.get_rpc_id(),
            &self.rpc_endpoint,
            self.get_verbosity(),
            deploy_hash,
            finalized_approvals,
        )
        .await
        .unwrap();

        response.result
    }

    //     pub async fn get_state_root_hash(&self) -> Result<String, Error> {
    //         // Implementation to fetch the latest state root hash.
    //     }

    //     pub async fn query_global_state(
    //         &self,
    //         state_root_hash: &str,
    //         key: &str,
    //         path: Vec<String>,
    //     ) -> Result<QueryResult, Error> {
    //         // Implementation to query the global state.
    //     }

    //     pub async fn get_dictionary_item(
    //         &self,
    //         state_root_hash: &str,
    //         dictionary_item_key: &str,
    //     ) -> Result<DictionaryItem, Error> {
    //         // Implementation to get a dictionary item by its key.
    //     }
}
