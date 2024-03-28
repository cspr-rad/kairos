use casper_client::{
    rpcs::results::{GetDeployResult, GetStateRootHashResult, QueryGlobalStateResult},
    types::{Contract, DeployHash},
    JsonRpcId, Verbosity,
};
use casper_types::CLValue;

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

    // Fetch latest state root hash.
    pub async fn get_state_root_hash(&self) -> [u8; 32] {
        // No block given means the latest available.
        let block_identifier = None;

        let response = casper_client::get_state_root_hash(
            self.get_rpc_id(),
            &self.rpc_endpoint,
            self.get_verbosity(),
            block_identifier,
        )
        .await
        .unwrap();

        let state_root_hash = response.result.state_root_hash.unwrap();

        state_root_hash.into()
    }

    async fn query_global_state(
        &self,
        state_root_hash: &[u8; 32],
        key: casper_types::Key,
        path: Vec<String>,
    ) -> QueryGlobalStateResult {
        // Wrap state root hash.
        let global_state_identifier = casper_client::rpcs::GlobalStateIdentifier::StateRootHash(
            state_root_hash.clone().into(),
        );

        let response = casper_client::query_global_state(
            self.get_rpc_id(),
            &self.rpc_endpoint,
            self.get_verbosity(),
            global_state_identifier,
            key,
            path,
        )
        .await
        .unwrap();

        response.result
    }

    pub async fn get_contract(&self, contract_hash: &str) -> Contract {
        // Build contract hash.
        let contract_hash_bytes = hex::decode(contract_hash).unwrap();
        let contract_hash_bytes: [u8; 32] = contract_hash_bytes.try_into().unwrap();
        let contract_hash = casper_types::ContractWasmHash::new(contract_hash_bytes);

        // Fetch latest state root hash.
        let state_root_hash = self.get_state_root_hash().await;

        // Contract is stored directly at given hash.
        let key = casper_types::Key::Hash(contract_hash.value());
        let path = vec![];

        let response = self.query_global_state(&state_root_hash, key, path).await;
        let contract = match response.stored_value {
            casper_client::types::StoredValue::Contract(v) => v,
            _ => panic!("Expected contract."),
        };

        contract
    }

    pub async fn get_stored_clvalue(&self, uref_addr: &str) -> CLValue {
        // Fetch latest state root hash.
        let state_root_hash = self.get_state_root_hash().await;

        // Build uref key.
        let key =
            casper_types::Key::URef(casper_types::URef::from_formatted_str(&uref_addr).unwrap());
        let path = vec![];

        let response = self.query_global_state(&state_root_hash, key, path).await;
        let clvalue = match response.stored_value {
            casper_client::types::StoredValue::CLValue(v) => v,
            _ => panic!("Expected CLValue."),
        };

        clvalue
    }

    pub async fn get_stored_clvalue_from_dict(
        &self,
        dictionary_seed_uref: &str,
        dictionary_item_key: &str,
    ) -> CLValue {
        // Fetch latest state root hash.
        let state_root_hash = self.get_state_root_hash().await;

        // Build dictionary item identifier.
        let seed_uref = casper_types::URef::from_formatted_str(&dictionary_seed_uref).unwrap();
        let dictionary_item_key = dictionary_item_key.to_string();
        let dictionary_item_identifier =
            casper_client::rpcs::DictionaryItemIdentifier::new_from_seed_uref(
                seed_uref,
                dictionary_item_key,
            );

        let response = casper_client::get_dictionary_item(
            self.get_rpc_id(),
            &self.rpc_endpoint,
            self.get_verbosity(),
            state_root_hash.into(),
            dictionary_item_identifier,
        )
        .await
        .unwrap();
        let clvalue = match response.result.stored_value {
            casper_client::types::StoredValue::CLValue(v) => v,
            _ => panic!("Expected CLValue."),
        };

        clvalue
    }
}
